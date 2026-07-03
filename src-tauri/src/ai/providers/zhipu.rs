use crate::ai::types::AiModel;
use super::AiProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

pub struct ZhipuProvider;

#[async_trait]
impl AiProvider for ZhipuProvider {
    async fn route_request(
        &self,
        scene_id: &str,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let client = Client::new();
        
        match scene_id {
            "text_to_image" => {
                self.generate_image(&client, model, payload).await
            }
            "general_text" | "markdown_polish" => {
                self.chat_completion(&client, model, payload).await
            }
            "text_to_video" | "image_to_video" => {
                self.generate_video(&client, model, payload).await
            }
            _ => Err(format!("Scene '{}' not supported by Zhipu provider yet", scene_id).into()),
        }
    }
}

impl ZhipuProvider {
    async fn chat_completion(
        &self,
        client: &Client,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // Zhipu AI is OpenAI compatible
        if model.base_url.trim().is_empty() {
            return Err("Zhipu API Error: baseUrl is required".into());
        }
        let prompt = payload["prompt"].as_str().ok_or("Missing prompt")?;
        let system_prompt = payload.get("systemPrompt").and_then(|v| v.as_str());
        
        let mut messages = vec![];
        if let Some(sys) = system_prompt {
            messages.push(json!({ "role": "system", "content": sys }));
        }
        messages.push(json!({ "role": "user", "content": prompt }));

        let req_body = json!({
            "model": if model.model.is_empty() { "glm-4" } else { &model.model },
            "messages": messages,
            "temperature": payload.get("temperature").unwrap_or(&json!(0.95)),
            "max_tokens": payload.get("maxTokens").unwrap_or(&json!(1024)),
            "top_p": 0.7,
            "stream": false
        });

        // Use default base_url if not provided
        let base_url = model.base_url.trim_end_matches('/');
        let url = format!("{}/chat/completions", base_url);
        
        debug!("Zhipu Chat Request: URL={}, Body={}", url, req_body);

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("Zhipu Chat Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("Zhipu API Error: {}", error_text);
            return Err(format!("Zhipu API Error: {}", error_text).into());
        }

        let body: Value = res.json().await?;
        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("No content in response")?
            .to_string();

        Ok(json!({ "content": content }))
    }

    async fn generate_image(
        &self,
        client: &Client,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let prompt = payload["prompt"].as_str().ok_or("Missing prompt")?;
        let output_dir = payload["outputDir"].as_str().ok_or("Missing outputDir")?;
        
        let req_body = json!({
            "model": if model.model.is_empty() { "cogview-3" } else { &model.model },
            "prompt": prompt,
        });

        let base_url = if model.base_url.is_empty() {
            "https://open.bigmodel.cn/api/paas/v4"
        } else {
            model.base_url.trim_end_matches('/')
        };
        let url = format!("{}/images/generations", base_url);

        debug!("Zhipu Image Request: URL={}, Body={}", url, req_body);

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("Zhipu Image Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("Zhipu Image API Error: {}", error_text);
            return Err(format!("Zhipu API Error: {}", error_text).into());
        }

        let body: Value = res.json().await?;
        let image_url = body["data"][0]["url"]
            .as_str()
            .ok_or("No image URL in response")?;

        let path = self.download_and_save(client, image_url, output_dir).await
            .map_err(|e| {
                error!("Zhipu Image Download Failed: {}", e);
                e
            })?;
        Ok(json!({ "path": path }))
    }

    async fn generate_video(
        &self,
        client: &Client,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        if model.base_url.trim().is_empty() {
            return Err("Zhipu API Error: baseUrl is required".into());
        }
        let prompt = payload["prompt"].as_str().ok_or("Missing prompt")?;
        let output_dir = payload["outputDir"].as_str().ok_or("Missing outputDir")?;
        let image_path = payload.get("imagePath").and_then(|v| v.as_str());

        let mut req_body = json!({
            "model": if model.model.is_empty() { "cogvideox" } else { &model.model },
            "prompt": prompt,
        });

        // Add image_url if image_path provided (CogVideoX supports image input)
        if let Some(path) = image_path {
             // For Zhipu, image input usually needs to be a URL. 
             // If local file, might need upload or base64? 
             // Zhipu's documentation suggests `image_url` for image-to-video.
             // For now, let's assume the user provides a URL or we skip this until we implement file upload.
             // But if `path` is local file, Zhipu API might not accept it directly unless we upload it first.
             // TODO: Implement file upload logic or assume URL for now.
             // For simplicity in this first version, we'll try to use it as is, but it likely requires a public URL.
             // A better approach for local files is to use base64 if supported, but video APIs often require URLs.
             
             // If local file, we might warn user.
             if !path.starts_with("http") {
                 return Err("Zhipu video generation requires a public URL for input images. Local file upload not yet implemented.".into());
             }
             req_body["image_url"] = json!(path);
        }

        let base_url = model.base_url.trim_end_matches('/');
        let url = format!("{}/videos/generations", base_url);

        debug!("Zhipu Video Request: URL={}, Body={}", url, req_body);

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("Zhipu Video Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("Zhipu Video API Error: {}", error_text);
            return Err(format!("Zhipu API Error: {}", error_text).into());
        }

        let body: Value = res.json().await?;
        let task_id = body["id"]
            .as_str()
            .ok_or("No task ID in response")?;

        let video_url = self.poll_video_result(client, &base_url, &model.api_key, task_id).await?;
        let path = self.download_and_save(client, &video_url, output_dir).await
            .map_err(|e| {
                error!("Zhipu Video Download Failed: {}", e);
                e
            })?;
        
        Ok(json!({ "path": path }))
    }

    async fn poll_video_result(
        &self, 
        client: &Client, 
        base_url: &str, 
        api_key: &str, 
        task_id: &str
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/async-result/{}", base_url, task_id);
        let max_retries = 60; // 5 minutes (5s interval)
        
        for _ in 0..max_retries {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            let res = client.get(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| {
                    error!("Zhipu Video Poll Failed: {}", e);
                    e
                })?;
                
            if !res.status().is_success() {
                continue;
            }

            let body: Value = res.json().await?;
            let task_status = body["task_status"].as_str().unwrap_or("PROCESSING");

            if task_status == "SUCCESS" {
                let video_url = body["video_result"][0]["url"]
                    .as_str()
                    .ok_or("No video URL in success response")?;
                return Ok(video_url.to_string());
            } else if task_status == "FAIL" {
                error!("Zhipu Video Task Failed. Body: {}", body);
                return Err("Video generation task failed".into());
            }
        }
        
        error!("Zhipu Video Task Timeout: {}", task_id);
        Err("Video generation timed out".into())
    }

    async fn download_and_save(
        &self,
        client: &Client,
        url: &str,
        output_dir: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let bytes = client.get(url).send().await?.bytes().await?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        
        // Simple extension detection
        let ext = if url.contains(".mp4") { "mp4" } else { "png" };
        let filename = format!("zhipu_{}_{}.{}", if ext == "mp4" { "vid" } else { "img" }, timestamp, ext);
        
        let dir_path = Path::new(output_dir);
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }
        let file_path = dir_path.join(&filename);
        fs::write(&file_path, bytes)?;
        
        Ok(file_path.to_string_lossy().to_string())
    }
}
