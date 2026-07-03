use crate::ai::types::AiModel;
use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;

pub mod openai;
pub mod qwen;
pub mod zhipu;
pub mod fal;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn route_request(
        &self,
        scene_id: &str,
        model: &AiModel,
        payload: &Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>>;
}

pub fn get_provider(provider_id: &str) -> Option<Box<dyn AiProvider>> {
    match provider_id.to_lowercase().as_str() {
        "qwen" | "tongyi" | "dashscope" => Some(Box::new(qwen::QwenProvider)),
        "zhipu" | "zhipuai" | "glm" => Some(Box::new(zhipu::ZhipuProvider)),
        "fal" | "flux" => Some(Box::new(fal::FalProvider)),
        // OpenAI and all compatible providers (DeepSeek, Moonshot, OneAPI, etc.)
        _ => Some(Box::new(openai::OpenAiProvider)),
    }
}
