use once_cell::sync::OnceCell;
use tauri::AppHandle;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, reload, util::SubscriberInitExt, EnvFilter,
};

static LOG_GUARD: OnceCell<WorkerGuard> = OnceCell::new();

use crate::file_ops::get_app_data_dir;

pub fn init_logging(_app: &AppHandle) -> Result<(), String> {
    if LOG_GUARD.get().is_some() {
        return Ok(());
    }
    
    let base_dir = get_app_data_dir()?;
    let log_dir = base_dir.join("logs");
    std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;

    // Custom log file name format: drip-note.YYMMDD.log
    // Since tracing-appender doesn't support custom date formats for rotation out of the box (it uses yyyy-MM-dd),
    // we have to use a workaround or accept the standard format.
    // However, we can use the `time` crate to get the current local time and construct the filename manually for the initial file?
    // But rotation handles the date appending automatically.
    
    // Actually, tracing-appender's Rotation::DAILY appends .yyyy-MM-dd to the filename prefix.
    // e.g. prefix.yyyy-MM-dd
    
    // If the user wants `drip-note.260226.log`, that is `drip-note.yyMMdd.log`.
    // tracing-appender does NOT support custom date formats for the rotation suffix. It is hardcoded.
    // See: https://docs.rs/tracing-appender/latest/tracing_appender/rolling/struct.Rotation.html
    // "Files will be rotated daily. The current timestamp will be appended to the filename in the format YYYY-MM-DD."
    
    // To achieve exactly `drip-note.260226.log`, we would need to implement our own RollingFileAppender
    // or use a different crate.
    // Given the constraints, maybe we can stick to standard YYYY-MM-DD for now but ensure it is LOCAL time?
    // Or we can manually create the file writer without using `rolling` if we don't strictly need automatic rotation (or implement simple rotation).
    
    // Let's try to set the timer to Local first for the CONTENT.
    // For the filename, if the user insists on 260226 format, we have to manually compute the filename.
    // But `tracing_appender::rolling` is convenient.
    
    // Wait, if I use `NEVER` rotation, I can set the filename manually?
    // But then it won't rotate.
    
    // Let's use `time` to get the current date in the desired format and use it as the filename suffix?
    // But `rolling` appender appends the date AUTOMATICALLY.
    
    // Let's explain to the user that standard rotation uses YYYY-MM-DD.
    // BUT, the issue "Today is 2026-02-26 but log is 2026-02-25" is likely due to UTC vs Local time.
    // 2:17 AM in China (UTC+8) is still previous day 18:17 PM in UTC.
    // So fixing the timezone to Local is the priority.
    
    // tracing-appender 0.2 doesn't seem to support Local time for rotation. 
    // It uses `time::OffsetDateTime::now_utc()`.
    
    // WE MUST IMPLEMENT A CUSTOM ROLLING APPENDER OR USE A WORKAROUND.
    // Workaround: Calculate the filename manually using Local time and use `tracing_appender::non_blocking` with a standard `File`.
    // But then we lose automatic rotation.
    
    // Let's try to use `time` crate to get local offset and see if we can trick it? No.
    
    // Ok, I will implement a simple manual file appender that uses the correct name.
    // And I will use `time` crate for the content timestamp.
    
    use time::OffsetDateTime;
    
    // Get local time
    let local_time = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    // Format: YYMMDD
    let format = time::macros::format_description!("[year repr:last_two][month][day]");
    let date_str = local_time.format(&format).unwrap_or_else(|_| "unknown".to_string());
    
    let log_filename = format!("drip-note.{}.log", date_str);
    let log_path = log_dir.join(&log_filename);
    
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;
    
    let (non_blocking, guard) = tracing_appender::non_blocking(file);
    
    // Timer for log content
    let timer = tracing_subscriber::fmt::time::LocalTime::new(time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));

    // Note: We are using a static file name for the session. 
    // It won't rotate at midnight while the app is running, but it will pick up the new name on restart next day.
    // This satisfies the "correct date in filename" requirement for now.
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));
        
    let (filter_layer, _reload_handle) = reload::Layer::new(filter);
    
    let fmt_layer = fmt::layer()
        .with_timer(timer)
        .with_writer(non_blocking)
        .with_ansi(false);

    // We must construct the subscriber in a way that matches the type alias.
    // The handle expects S = Layered<FmtLayer, Registry>.
    // So we must attach reload_layer TO (registry + fmt_layer).
    
    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer);
        
    subscriber.try_init().map_err(|e| e.to_string())?;

    let _ = LOG_GUARD.set(guard);
    // let _ = LOG_HANDLE.set(reload_handle);
    
    Ok(())
}

#[tauri::command]
pub fn log_message(level: String, message: String) {
    match level.to_lowercase().as_str() {
        "error" => tracing::error!("{}", message),
        "warn" => tracing::warn!("{}", message),
        "info" => tracing::info!("{}", message),
        "debug" => tracing::debug!("{}", message),
        "trace" => tracing::trace!("{}", message),
        _ => tracing::info!("{}", message),
    }
}

// Temporary update_log_level for preview window compatibility
#[tauri::command]
pub fn update_log_level(level: String) -> Result<(), String> {
    log_message("info".to_string(), format!("Log level update requested: {}", level));
    Ok(())
}
