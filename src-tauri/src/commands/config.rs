use crate::models::config::AppConfig;
use crate::models::marketplace::MarketplaceSource;
use crate::utils::git::parse_source;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfigExport {
    pub sources: Vec<MarketplaceSource>,
    pub theme: Option<String>,
    // Add more fields as needed (e.g. agents config if customized)
}

#[tauri::command]
pub async fn export_config(_app: tauri::AppHandle) -> Result<String, String> {
    // 1. Fetch Marketplace Sources
    // For now just export sources
    // let mut sources = crate::commands::marketplace::get_marketplace_sources().await?;

    // 2. Fetch Source States (enabled/disabled) - actually get_marketplace_sources handles this already!
    // But we might want the raw custom sources to avoid re-exporting default ones if we want a clean import.
    // However, exporting the final state (including defaults) is often safer for restoration,
    // OR we only export custom sources.
    // Let's rely on `get_marketplace_sources` but we might need to filter if we only want custom ones.
    // Actually, `get_marketplace_sources` merges defaults + custom.
    // Let's filter for custom ones only if we want to be minimal, but full snapshot is better.
    // BUT `add_marketplace_source` only adds to custom.
    // So if we import, we should probably only import custom ones?
    // Let's try to read the raw custom config key directly to be precise.

    let custom_sources_json = crate::utils::db::get_config("custom_marketplace_sources")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "[]".to_string());

    let custom_sources: Vec<MarketplaceSource> =
        serde_json::from_str(&custom_sources_json).unwrap_or_default();

    let config = AppConfigExport {
        sources: custom_sources,
        theme: None,
    };

    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_config(json: String, _app: tauri::AppHandle) -> Result<(), String> {
    let config: AppConfigExport =
        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Restore Sources
    // We overwrite the custom sources key entirely for a clean restore
    let sources_json = serde_json::to_string(&config.sources).map_err(|e| e.to_string())?;

    crate::utils::db::set_config("custom_marketplace_sources", &sources_json)
        .map_err(|e| e.to_string())?;

    Ok(())
}
