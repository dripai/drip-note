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

pub struct FalProvider;

#[async_trait]
impl AiProvider for FalProvider {
    async fn route_request(
        &self,
        scene_id: &str,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let client = Client::new();
        if model.base_url.trim().is_empty() {
            return Err("Fal API Error: baseUrl is required".into());
        }
        let base_url = model.base_url.trim_end_matches('/');
        
        // Map scene to Fal endpoint if not specified in model.model
        // But usually model.model should be the Fal endpoint like "fal-ai/flux/dev"
        // If user just put "flux-dev", we might need a mapping. 
        // For now, assume model.model IS the Fal function ID (e.g. "fal-ai/flux/dev")
        
        let endpoint_id = if model.model.is_empty() {
            "fal-ai/flux/dev"
        } else {
            &model.model
        };

        // Fal uses a queue-based system for most generation models
        let queue_url = format!("{}/{}", base_url, endpoint_id);

        match scene_id {
            "text_to_image" => {
                self.generate_image(&client, base_url, &queue_url, model, payload).await
            }
            "text_to_video" | "image_to_video" => {
                // Fal also supports video models like "fal-ai/kling-video/v1/standard/text-to-video"
                // So we can use the same logic, just different payload processing
                self.generate_video(&client, base_url, &queue_url, model, payload).await
            }
            _ => Err(format!("Scene '{}' not supported by Fal provider yet", scene_id).into()),
        }
    }
}

impl FalProvider {
    async fn generate_image(
        &self,
        client: &Client,
        base_url: &str,
        queue_url: &str,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let prompt = payload["prompt"].as_str().ok_or("Missing prompt")?;
        let output_dir = payload["outputDir"].as_str().ok_or("Missing outputDir")?;
        
        let req_body = json!({
            "prompt": prompt,
            "image_size": "landscape_4_3", // Default, maybe make configurable via inputParams
            "num_inference_steps": 28,
            "guidance_scale": 3.5,
            "enable_safety_checker": true
        });

        debug!("Fal Image Request: URL={}, Body={}", queue_url, req_body);

        // 1. Submit request
        let res = client.post(queue_url)
            .header("Authorization", format!("Key {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("Fal Image Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("Fal API Error: {}", error_text);
            return Err(format!("Fal API Error: {}", error_text).into());
        }

        let body: Value = res.json().await?;
        let request_id = body["request_id"].as_str().ok_or("No request_id in response")?;

        let endpoint_id = if model.model.is_empty() {
            "fal-ai/flux/dev"
        } else {
            &model.model
        };

        // 2. Poll for result
        let image_url = self.poll_result(client, base_url, request_id, &model.api_key, endpoint_id).await?;

        // 3. Download and Save
        let img_bytes = client.get(&image_url).send().await?.bytes().await?;
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let filename = format!("flux_img_{}.png", timestamp);
        let dir_path = Path::new(output_dir);
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }
        let file_path = dir_path.join(&filename);
        
        fs::write(&file_path, img_bytes)?;

        Ok(json!({ "path": file_path.to_string_lossy() }))
    }

    async fn generate_video(
        &self,
        client: &Client,
        base_url: &str,
        queue_url: &str,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let prompt = payload["prompt"].as_str().ok_or("Missing prompt")?;
        let output_dir = payload["outputDir"].as_str().ok_or("Missing outputDir")?;
        let image_path = payload.get("imagePath").and_then(|v| v.as_str());

        let mut req_body = json!({
            "prompt": prompt,
            "duration": "5",
            "aspect_ratio": "16:9"
        });

        // If image-to-video (Kling on Fal, or Luma on Fal)
        if let Some(path) = image_path {
             // Similar to Zhipu, Fal usually expects a URL.
             // If local file, we need to upload it. 
             // Fal has a temporary storage API, but for now we'll assume URL or fail.
             // TODO: Implement Fal storage upload.
             if !path.starts_with("http") {
                 return Err("Fal video generation requires a public URL for input images. Local file upload not yet implemented.".into());
             }
             req_body["image_url"] = json!(path);
        }

        debug!("Fal Video Request: URL={}, Body={}", queue_url, req_body);

        // 1. Submit request
        let res = client.post(queue_url)
            .header("Authorization", format!("Key {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("Fal Video Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("Fal Video API Error: {}", error_text);
            return Err(format!("Fal API Error: {}", error_text).into());
        }

        let body: Value = res.json().await?;
        let request_id = body["request_id"].as_str().ok_or("No request_id in response")?;

        let endpoint_id = if model.model.is_empty() {
            "fal-ai/flux/dev"
        } else {
            &model.model
        };

        // 2. Poll for result
        let video_url = self.poll_result(client, base_url, request_id, &model.api_key, endpoint_id).await?;

        // 3. Download and Save
        let video_bytes = client.get(&video_url).send().await?.bytes().await?;
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let filename = format!("fal_vid_{}.mp4", timestamp);
        let dir_path = Path::new(output_dir);
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }
        let file_path = dir_path.join(&filename);
        
        fs::write(&file_path, video_bytes)?;

        Ok(json!({ "path": file_path.to_string_lossy() }))
    }

    async fn poll_result(
        &self, 
        client: &Client, 
        base_url: &str,
        request_id: &str, 
        api_key: &str,
        endpoint_id: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let status_url = format!("{}/{}/requests/{}/status", base_url.trim_end_matches('/'), endpoint_id, request_id);
        
        let max_retries = 120; // 10 minutes (5s interval)
        
        for _ in 0..max_retries {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            let res = client.get(&status_url)
                .header("Authorization", format!("Key {}", api_key))
                .send()
                .await
                .map_err(|e| {
                    error!("Fal Poll Failed: {}", e);
                    e
                })?;
                
            if !res.status().is_success() {
                let status_code = res.status();
                let err_text = res.text().await?;
                // Log but retry? Or fail? Usually fail if status check fails (e.g. 404/401)
                // 404 might mean queue delay, but unlikely for status endpoint.
                // 401 means auth error.
                error!("Fal Status Check Error ({}): {}", status_code, err_text);
                return Err(format!("Fal Status Check Error ({}): {}", status_code, err_text).into());
            }

            let body: Value = res.json().await?;
            let status = body["status"].as_str().unwrap_or("IN_QUEUE");

            if status == "COMPLETED" {
                // Fal returns result in `response` field, or `images` array inside it
                let response = &body["response"];
                
                // Check for image
                if let Some(images) = response["images"].as_array() {
                    if let Some(first) = images.first() {
                         if let Some(url) = first["url"].as_str() {
                             return Ok(url.to_string());
                         }
                    }
                }
                
                // Check for video
                if let Some(video) = response["video"].as_object() {
                    if let Some(url) = video["url"].as_str() {
                        return Ok(url.to_string());
                    }
                }

                // Fallback for simple url field
                if let Some(url) = response["url"].as_str() {
                    return Ok(url.to_string());
                }

                return Err(format!("Unknown Fal response format: {:?}", response).into());
            } else if status == "FAILED" {
                return Err(format!("Fal task failed: {:?}", body["error"]).into());
            } else {
                // IN_QUEUE, IN_PROGRESS, etc.
                // Just continue polling
            }
        }
        
        Err("Fal task timed out".into())
    }
}
