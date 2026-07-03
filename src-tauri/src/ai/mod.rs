pub mod providers;
pub mod schema;
pub mod types;

use crate::ai::providers::get_provider;
use crate::ai::schema::normalize_payload;
use crate::ai::types::AiModel;
use serde_json::Value;
use tauri::command;

#[command]
pub async fn ai_route_request(
    scene_id: String,
    model: AiModel,
    payload: Value,
) -> Result<Value, String> {
    let provider = get_provider(&model.provider)
        .ok_or_else(|| format!("Provider '{}' not supported", model.provider))?;

    let payload = normalize_payload(&scene_id, &model, &payload)?;

    let result = provider
        .route_request(&scene_id, &model, &payload)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}
