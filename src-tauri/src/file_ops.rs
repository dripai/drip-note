use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use directories::ProjectDirs;

/// Get the application data directory
/// Windows: AppData/Roaming/com.drip.note
pub fn get_app_data_dir() -> Result<PathBuf, String> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "com.drip.note") {
        // ProjectDirs::data_dir() returns ".../com.drip.note/data" on Windows
        // We want the root ".../com.drip.note"
        let data_dir = proj_dirs.data_dir();
        if let Some(parent) = data_dir.parent() {
             fs::create_dir_all(parent).map_err(|e| e.to_string())?;
             return Ok(parent.to_path_buf());
        }
        // Fallback if parent fails (should not happen)
        fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
        return Ok(data_dir.to_path_buf());
    }
    
    // Fallback to executable directory if ProjectDirs fails
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or_else(|| "exe dir not found".to_string())?;
    Ok(dir.to_path_buf())
}

#[derive(Serialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn fs_list_dir(path: String) -> Result<Vec<FsEntry>, String> {
    let p = PathBuf::from(path);
    let mut out = Vec::new();
    let rd = fs::read_dir(&p).map_err(|e| e.to_string())?;
    for ent in rd {
        let ent = ent.map_err(|e| e.to_string())?;
        let name = ent.file_name().to_string_lossy().to_string();
        let child_path = ent.path().to_string_lossy().to_string();
        let md = ent.metadata().map_err(|e| e.to_string())?;
        out.push(FsEntry {
            name,
            path: child_path,
            is_dir: md.is_dir(),
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn fs_move(src_path: String, dest_dir: String, new_name: Option<String>) -> Result<(), String> {
    let src = PathBuf::from(src_path);
    let name = new_name.unwrap_or_else(|| src.file_name().unwrap().to_string_lossy().to_string());
    let dest = PathBuf::from(dest_dir).join(name);
    if fs::rename(&src, &dest).is_err() {
        let md = fs::metadata(&src).map_err(|e| e.to_string())?;
        if md.is_dir() {
            copy_dir_all(&src, &dest)?;
            fs::remove_dir_all(&src).map_err(|e| e.to_string())?;
        } else {
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
            fs::remove_file(&src).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn fs_remove(path: String) -> Result<(), String> {
    let p = PathBuf::from(path);
    if p.is_dir() {
        fs::remove_dir_all(&p).map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn fs_new_folder(parent_dir: String, name: String) -> Result<(), String> {
    let dir = PathBuf::from(parent_dir).join(name);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn fs_ensure_dir(path: String) -> Result<(), String> {
    let p = PathBuf::from(path);
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn fs_write_binary(path: String, bytes: Vec<u8>) -> Result<(), String> {
    let p = PathBuf::from(path);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&p, bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn fs_read_text(path: String) -> Result<String, String> {
    fs::read_to_string(PathBuf::from(path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fs_write_text(path: String, content: String) -> Result<(), String> {
    let mut f = fs::File::create(PathBuf::from(path)).map_err(|e| e.to_string())?;
    f.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}
