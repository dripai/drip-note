use tauri_plugin_sql::{Migration, MigrationKind};

fn get_db_path() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_dir = exe
        .parent()
        .ok_or_else(|| "exe dir not found".to_string())?;
    Ok(exe_dir.join("drip-note.db"))
}

#[tauri::command]
pub fn db_exe_url() -> Result<String, String> {
    let path = get_db_path()?;
    Ok(format!("sqlite:{}", path.to_string_lossy()))
}

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration { version: 1, description: "create_app_config", sql: "CREATE TABLE IF NOT EXISTS app_config (\n  key TEXT PRIMARY KEY,\n  value TEXT NOT NULL,\n  updated_at INTEGER NOT NULL\n);", kind: MigrationKind::Up },
        Migration { version: 2, description: "create_notes", sql: "CREATE TABLE IF NOT EXISTS notes(\n  id TEXT PRIMARY KEY,\n  title TEXT NOT NULL,\n  content_md TEXT,\n  notebook_id TEXT,\n  pinned INTEGER,\n  deleted_at INTEGER,\n  created_at INTEGER NOT NULL,\n  updated_at INTEGER NOT NULL\n);", kind: MigrationKind::Up },
        Migration { version: 3, description: "create_notebooks", sql: "CREATE TABLE IF NOT EXISTS notebooks(\n  id TEXT PRIMARY KEY,\n  name TEXT NOT NULL,\n  icon TEXT,\n  sort_order INTEGER\n);", kind: MigrationKind::Up },
        Migration { version: 4, description: "create_tags", sql: "CREATE TABLE IF NOT EXISTS tags(\n  id TEXT PRIMARY KEY,\n  name TEXT NOT NULL,\n  color TEXT\n);", kind: MigrationKind::Up },
        Migration { version: 5, description: "create_note_tags", sql: "CREATE TABLE IF NOT EXISTS note_tags(\n  note_id TEXT NOT NULL,\n  tag_id TEXT NOT NULL,\n  PRIMARY KEY(note_id, tag_id)\n);", kind: MigrationKind::Up },
        Migration { version: 6, description: "add_sort_to_notes", sql: "ALTER TABLE notes ADD COLUMN sort INTEGER;", kind: MigrationKind::Up },
        Migration { version: 7, description: "create_app_config_if_missing", sql: "CREATE TABLE IF NOT EXISTS app_config (\n  key TEXT PRIMARY KEY,\n  value TEXT NOT NULL,\n  updated_at INTEGER NOT NULL\n);", kind: MigrationKind::Up },
        Migration { version: 8, description: "force_create_app_config_v2", sql: "CREATE TABLE IF NOT EXISTS app_config (\n  key TEXT PRIMARY KEY,\n  value TEXT NOT NULL,\n  updated_at INTEGER NOT NULL\n);", kind: MigrationKind::Up },
    ]
}

pub fn get_db_url() -> String {
    let path = get_db_path().expect("get db path");
    format!("sqlite:{}", path.to_string_lossy())
}
