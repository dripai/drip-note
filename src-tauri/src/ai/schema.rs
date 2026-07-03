use crate::ai::types::AiModel;
use serde_json::Value;

pub fn normalize_payload(scene_id: &str, model: &AiModel, payload: &Value) -> Result<Value, String> {
    let mut map = payload.as_object().cloned().unwrap_or_default();
    let specs = specs_for(scene_id, &model.provider);
    for spec in specs {
        if map.get(spec.name).is_some() {
            continue;
        }
        if let Some(v) = find_alias(&map, spec.aliases) {
            map.insert(spec.name.to_string(), v);
            continue;
        }
        if spec.required {
            return Err(format!("Missing required field: {}", spec.name));
        }
        if let Some(v) = spec.default {
            map.insert(spec.name.to_string(), v);
        }
    }
    Ok(Value::Object(map))
}

struct ParamSpec {
    name: &'static str,
    aliases: &'static [&'static str],
    required: bool,
    default: Option<Value>,
}

fn specs_for(scene_id: &str, _provider: &str) -> Vec<ParamSpec> {
    match scene_id {
        "general_text" | "markdown_polish" => vec![
            ParamSpec { name: "prompt", aliases: &["text", "content"], required: true, default: None },
            ParamSpec { name: "systemPrompt", aliases: &["system_prompt"], required: false, default: None },
            ParamSpec { name: "temperature", aliases: &["temp"], required: false, default: None },
            ParamSpec { name: "maxTokens", aliases: &["max_tokens"], required: false, default: None },
        ],
        "text_to_image" => vec![
            ParamSpec { name: "prompt", aliases: &["text", "content"], required: true, default: None },
            ParamSpec { name: "outputDir", aliases: &["output_dir"], required: true, default: None },
            ParamSpec { name: "negativePrompt", aliases: &["negative_prompt"], required: false, default: None },
            ParamSpec { name: "size", aliases: &["image_size"], required: false, default: None },
            ParamSpec { name: "n", aliases: &["num_images"], required: false, default: None },
        ],
        "text_to_video" | "image_to_video" => vec![
            ParamSpec { name: "prompt", aliases: &["text", "content"], required: true, default: None },
            ParamSpec { name: "outputDir", aliases: &["output_dir"], required: true, default: None },
            ParamSpec { name: "imagePath", aliases: &["image_path"], required: false, default: None },
        ],
        _ => vec![],
    }
}

fn find_alias(map: &serde_json::Map<String, Value>, aliases: &[&'static str]) -> Option<Value> {
    for key in aliases {
        if let Some(v) = map.get(*key) {
            return Some(v.clone());
        }
    }
    None
}
