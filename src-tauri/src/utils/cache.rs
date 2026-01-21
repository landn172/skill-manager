use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CacheMetadata {
    pub skill_name: String,
    pub source_url: String,
    pub downloaded_at: String,
}

pub fn get_cache_root() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gemini-skills-cache")
}

pub fn get_skill_cache_dir(skill_name: &str) -> PathBuf {
    get_cache_root().join("downloads").join(skill_name)
}

pub async fn is_cache_valid(skill_name: &str) -> bool {
    let cache_dir = get_skill_cache_dir(skill_name);
    if !cache_dir.exists() {
        return false;
    }

    // Check if metadata exists
    let meta_path = cache_dir.join(".cache_meta.json");
    meta_path.exists()
}

pub async fn write_cache_metadata(skill_name: &str, source_url: &str) -> Result<(), String> {
    let cache_dir = get_skill_cache_dir(skill_name);
    fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| e.to_string())?;

    let metadata = CacheMetadata {
        skill_name: skill_name.to_string(),
        source_url: source_url.to_string(),
        downloaded_at: chrono::Utc::now().to_rfc3339(),
    };

    let meta_path = cache_dir.join(".cache_meta.json");
    let json = serde_json::to_string_pretty(&metadata).map_err(|e| e.to_string())?;

    fs::write(meta_path, json).await.map_err(|e| e.to_string())
}

pub async fn read_cache_metadata(skill_name: &str) -> Option<CacheMetadata> {
    let cache_dir = get_skill_cache_dir(skill_name);
    let meta_path = cache_dir.join(".cache_meta.json");

    if !meta_path.exists() {
        return None;
    }

    let json = fs::read_to_string(meta_path).await.ok()?;
    serde_json::from_str(&json).ok()
}

pub async fn clear_skill_cache(skill_name: &str) -> Result<(), String> {
    let cache_dir = get_skill_cache_dir(skill_name);
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn clear_all_cache() -> Result<(), String> {
    let root = get_cache_root();
    if root.exists() {
        fs::remove_dir_all(root).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn get_all_cached_skills() -> Vec<CacheMetadata> {
    let downloads_dir = get_cache_root().join("downloads");
    let mut results = Vec::new();

    if let Ok(mut entries) = fs::read_dir(downloads_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(ty) = entry.file_type().await {
                if ty.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if let Some(meta) = read_cache_metadata(&name).await {
                        results.push(meta);
                    }
                }
            }
        }
    }

    results
}
