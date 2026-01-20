use crate::utils::db::{get_history, HistoryEntry};
use std::process::Command;

#[tauri::command]
pub async fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .status()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path)
            .status()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .status()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_install_history() -> Result<Vec<HistoryEntry>, String> {
    get_history()
}

#[tauri::command]
pub async fn clear_cache() -> Result<String, String> {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("gemini-skills-cache");

    if cache_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&cache_dir) {
            return Err(format!("Failed to clear cache directory: {}", e));
        }
        Ok("Cache cleared successfully".into())
    } else {
        Ok("Cache is already empty".into())
    }
}
