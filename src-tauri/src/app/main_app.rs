use crate::{
    ai,
    db,
    file_ops,
    logs,
    plugins,
};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(
                    Box::leak(db::get_db_url().into_boxed_str()),
                    db::get_migrations()
                )
                .build(),
        )
        .setup(|app| {
            logs::init_logging(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::greet,
            plugins::get_plugins_dir,
            logs::log_message,
            logs::update_log_level,
            file_ops::fs_list_dir,
            file_ops::fs_move,
            file_ops::fs_remove,
            file_ops::fs_new_folder,
            file_ops::fs_ensure_dir,
            file_ops::fs_write_binary,
            file_ops::fs_read_text,
            file_ops::fs_write_text,
            db::db_exe_url,
            ai::ai_route_request
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
