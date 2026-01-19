use crate::utils::db::{get_history, HistoryEntry};
use std::process::Command;

#[tauri::command]
pub async fn open_in_editor(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Try VS Code first
        if Command::new("code").arg(&path).status().is_ok() {
            return Ok(());
        }
        // Fallback to open command (default handler)
        Command::new("open")
            .arg(&path)
            .status()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        if Command::new("code.cmd").arg(&path).status().is_ok() {
            return Ok(());
        }
        Command::new("explorer")
            .arg(&path)
            .status()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        if Command::new("code").arg(&path).status().is_ok() {
            return Ok(());
        }
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
