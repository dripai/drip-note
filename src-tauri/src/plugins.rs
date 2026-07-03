use tauri::AppHandle;
use crate::file_ops::get_app_data_dir;
use std::fs;

#[tauri::command]
pub fn get_plugins_dir(_app: AppHandle) -> String {
    // Always use AppData directory for plugins to ensure consistency
    if let Ok(dir) = get_app_data_dir() {
        let plugin_dir = dir.join("plugins");
        // Ensure the directory exists
        let _ = fs::create_dir_all(&plugin_dir);
        return plugin_dir.to_string_lossy().to_string();
    }
    
    // Fallback (should rarely happen)
    "plugins".to_string()
}
