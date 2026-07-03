#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiModel {
    pub id: String,
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    // Optional fields not strictly needed for the request but might be passed
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiRouteResponse<T> {
    pub data: T,
}

// Common payload structures
#[derive(Debug, Serialize, Deserialize)]
pub struct TextGenerationPayload {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageGenerationPayload {
    pub prompt: String,
    pub output_dir: String,
    pub negative_prompt: Option<String>,
    pub size: Option<String>,
    pub n: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoGenerationPayload {
    pub prompt: String,
    pub output_dir: String,
    pub image_path: Option<String>, // For image-to-video
}
