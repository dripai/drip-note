use crate::ai::types::AiModel;
use super::{AiProvider, openai::OpenAiProvider};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};
use tracing::{debug, error};

pub struct QwenProvider;

#[async_trait]
impl AiProvider for QwenProvider {
    async fn route_request(
        &self,
        scene_id: &str,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        match scene_id {
            "text_to_image" => {
                self.generate_image(model, payload).await
            }
            _ => {
                let compat = OpenAiProvider;
                compat.route_request(scene_id, model, payload).await
            }
        }
    }
}

impl QwenProvider {
    async fn generate_image(
        &self,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let client = Client::new();
        if model.base_url.trim().is_empty() {
            return Err("Qwen API Error: baseUrl is required".into());
        }
        let prompt = payload["prompt"].as_str().ok_or("Missing prompt")?;
        let output_dir = payload["outputDir"].as_str().ok_or("Missing outputDir")?;
        let size = payload.get("size").and_then(|v| v.as_str()).unwrap_or("1024x1024");
        let size = size.replace('x', "*");
        let negative = payload.get("negativePrompt").and_then(|v| v.as_str());
        let n = payload.get("n").and_then(|v| v.as_i64());

        let mut parameters = json!({ "size": size });
        if let Some(neg) = negative {
            parameters["negative_prompt"] = json!(neg);
        }
        if let Some(n) = n {
            parameters["n"] = json!(n);
        }

        let req_body = json!({
            "model": if model.model.is_empty() { "qwen-image-max" } else { &model.model },
            "input": { "prompt": prompt },
            "parameters": parameters
        });

        let base = model.base_url.trim_end_matches('/');
        // DashScope API for image synthesis might differ based on version
        // Standard endpoint: https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis
        let url = if base.ends_with("/api/v1") {
            format!("{}/services/aigc/text2image/image-synthesis", base)
        } else {
            format!("{}/api/v1/services/aigc/text2image/image-synthesis", base)
        };
        
        debug!("Qwen Image Request: URL={}, Body={}", url, req_body);

        // ============ 第一步：提交异步任务 ============
        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("X-DashScope-Async", "enable")  // ✅ 强制开启异步任务
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("Qwen API Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("Qwen API Error: {}", error_text);
            return Err(format!("Qwen API Error: {}", error_text).into());
        }

        let body: Value = res.json().await?;
        
        // 异步任务模式下，直接获取 task_id
        if let Some(task_id) = body["output"]["task_id"].as_str() {
            debug!("Qwen Task Submitted: {}", task_id);
            let path = poll_task_and_save(&client, base, task_id, output_dir, &model.api_key).await?;
            return Ok(json!({ "path": path }));
        }

        // 兼容同步返回（虽然加上 Async 头后应该不会走到这里）
        if let Some(url) = extract_image_url(&body) {
            let path = download_and_save(&client, &url, output_dir).await?;
            return Ok(json!({ "path": path }));
        }

        error!("Qwen API Error: missing output results. Body: {}", body);
        Err("Qwen API Error: missing output results".into())
    }
}

fn extract_image_url(body: &Value) -> Option<String> {
    body["output"]["results"][0]["url"].as_str().map(|s| s.to_string())
}

async fn poll_task_and_save(
    client: &Client,
    base_url: &str,
    task_id: &str,
    output_dir: &str,
    api_key: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    // 任务查询 URL 修正：https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}
    let url = if base_url.ends_with("/api/v1") {
        format!("{}/tasks/{}", base_url, task_id)
    } else {
        format!("{}/api/v1/tasks/{}", base_url, task_id)
    };

    debug!("Polling Qwen Task: {}", url);

    for i in 0..60 { // 增加轮询次数到 60 次（2分钟）
        sleep(Duration::from_secs(2)).await;

        let res = client.get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| {
                error!("Qwen Poll Request Failed: {}", e);
                e
            })?;
            
        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("Qwen Poll API Error: {}", error_text);
            // 某些情况下 404 可能代表任务还没准备好？暂且认为失败
            return Err(format!("Qwen API Error: {}", error_text).into());
        }
        
        let body: Value = res.json().await?;
        let status = body["output"]["task_status"].as_str().unwrap_or("UNKNOWN");
        
        debug!("Qwen Task Status ({}): {}", i, status);

        if status == "SUCCEEDED" {
            if let Some(url) = extract_image_url(&body) {
                debug!("Qwen Task Succeeded. Downloading: {}", url);
                let path = download_and_save(client, &url, output_dir).await?;
                return Ok(path);
            }
            error!("Qwen Poll Error: missing result url. Body: {}", body);
            return Err("Qwen API Error: missing result url".into());
        }
        
        if status == "FAILED" {
            error!("Qwen Task Failed. Body: {}", body);
            let code = body["output"]["code"].as_str().unwrap_or("Unknown");
            let msg = body["output"]["message"].as_str().unwrap_or("Task failed");
            return Err(format!("Qwen Task Failed: {} - {}", code, msg).into());
        }
        // PENDING, RUNNING, etc. continue polling
    }
    error!("Qwen Task Timeout: {}", task_id);
    Err("Qwen API Error: task timeout".into())
}

async fn download_and_save(
    client: &Client,
    image_url: &str,
    output_dir: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let img_bytes = if image_url.starts_with("data:image/") {
        let parts: Vec<&str> = image_url.split(',').collect();
        if parts.len() != 2 {
            return Err("Invalid base64 image data".into());
        }
        general_purpose::STANDARD.decode(parts[1])?
    } else {
        client.get(image_url).send().await?.bytes().await?.to_vec()
    };

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let filename = format!("img_{}.png", timestamp);
    let dir_path = Path::new(output_dir);
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }
    let file_path = dir_path.join(&filename);
    fs::write(&file_path, img_bytes)?;
    Ok(file_path.to_string_lossy().to_string())
}
