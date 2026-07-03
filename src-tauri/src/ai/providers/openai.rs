use crate::ai::types::AiModel;
use super::AiProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::general_purpose};
use tracing::{debug, error};

pub struct OpenAiProvider;

#[async_trait]
impl AiProvider for OpenAiProvider {
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
            _ => Err(format!("Scene '{}' not supported by OpenAI provider yet", scene_id).into()),
        }
    }
}

impl OpenAiProvider {
    async fn chat_completion(
        &self,
        client: &Client,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let prompt = payload["prompt"].as_str().ok_or("Missing prompt")?;
        let system_prompt = payload.get("systemPrompt").and_then(|v| v.as_str());
        
        let mut messages = vec![];
        if let Some(sys) = system_prompt {
            messages.push(json!({ "role": "system", "content": sys }));
        }
        messages.push(json!({ "role": "user", "content": prompt }));

        let req_body = json!({
            "model": model.model,
            "messages": messages,
            "temperature": payload.get("temperature").unwrap_or(&json!(0.7)),
            "max_tokens": payload.get("maxTokens").unwrap_or(&json!(2000)),
        });

        let url = format!("{}/chat/completions", model.base_url.trim_end_matches('/'));
        
        debug!("OpenAI Chat Request: URL={}, Body={}", url, req_body);

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("OpenAI Chat Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("OpenAI API Error: {}", error_text);
            // Try to parse standard OpenAI error format: { "error": { "message": "..." } }
            if let Ok(err_json) = serde_json::from_str::<Value>(&error_text) {
                if let Some(msg) = err_json["error"]["message"].as_str() {
                    return Err(format!("OpenAI API Error: {}", msg).into());
                }
            }
            return Err(format!("OpenAI API Error: {}", error_text).into());
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
        
        // Check if we should use Chat Completions (for non-DALL-E models, e.g. Gemini via OpenAI proxy)
        let use_chat_completions = !model.model.is_empty() && !model.model.to_lowercase().starts_with("dall-e");

        let (url, req_body) = if use_chat_completions {
            let url = format!("{}/chat/completions", model.base_url.trim_end_matches('/'));
            let body = json!({
                "model": model.model,
                "messages": [
                    { "role": "user", "content": prompt }
                ]
            });
            (url, body)
        } else {
            let url = format!("{}/images/generations", model.base_url.trim_end_matches('/'));
            let body = json!({
                "model": if model.model.is_empty() { "dall-e-3" } else { &model.model },
                "prompt": prompt,
                "n": 1,
                "size": "1024x1024"
            });
            (url, body)
        };

        debug!("OpenAI Image Request: URL={}, Body={}", url, req_body);

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", model.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                error!("OpenAI Image Request Failed: {}", e);
                e
            })?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            error!("OpenAI Image API Error: {}", error_text);
            // Try to parse standard OpenAI error format: { "error": { "message": "..." } }
            if let Ok(err_json) = serde_json::from_str::<Value>(&error_text) {
                if let Some(msg) = err_json["error"]["message"].as_str() {
                    return Err(format!("OpenAI API Error: {}", msg).into());
                }
            }
            return Err(format!("OpenAI API Error: {}", error_text).into());
        }

        let raw_body = res.text().await?;
        debug!("OpenAI Raw Response: {}", raw_body);

        let body: Value = serde_json::from_str(&raw_body)
            .map_err(|e| format!("Failed to parse JSON response: {}. Raw: {:.200}...", e, raw_body))?;

        let image_url = if use_chat_completions {
            // Check for content filter or empty content
            let choice = &body["choices"][0];
            if let Some(finish_reason) = choice["finish_reason"].as_str() {
                if finish_reason == "content_filter" {
                    return Err("Image generation failed: Content filtered by safety policy.".into());
                }
            }

            // Extract URL from chat content
            let content = choice["message"]["content"]
                .as_str()
                .ok_or("No content in chat response")?;
            
            // Try to find URL in markdown: ![alt](url) or just https://...
            // Or base64 data: ![image](data:image/png;base64,...)
            if let Some(start) = content.find("https://") {
                let rest = &content[start..];
                let end = rest.find(|c: char| c == ')' || c == ' ' || c == '\n' || c == '"' || c == ']').unwrap_or(rest.len());
                rest[..end].to_string()
            } else if let Some(start) = content.find("data:image/") {
                // Base64 image
                let rest = &content[start..];
                let end = rest.find(|c: char| c == ')' || c == ' ' || c == '\n' || c == '"' || c == ']').unwrap_or(rest.len());
                rest[..end].to_string()
            } else {
                return Err(format!("Could not find image URL or Base64 in chat response: {}", content).into());
            }
        } else {
            body["data"][0]["url"]
                .as_str()
                .ok_or("No image URL in response")?
                .to_string()
        };

        // Download image or decode base64
        let img_bytes = if image_url.starts_with("data:image/") {
            // data:image/png;base64,....
            let parts: Vec<&str> = image_url.split(',').collect();
            if parts.len() != 2 {
                return Err("Invalid base64 image data".into());
            }
            general_purpose::STANDARD.decode(parts[1])?
        } else {
            client.get(&image_url).send().await?.bytes().await?.to_vec()
        };
        
        // Save to file
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let filename = format!("img_{}.png", timestamp);
        let dir_path = Path::new(output_dir);
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)?;
        }
        let file_path = dir_path.join(&filename);
        
        fs::write(&file_path, img_bytes)?;

        Ok(json!({ "path": file_path.to_string_lossy() }))
    }
}
