use crate::utils::cache;
use crate::utils::cache::CacheMetadata;
use tauri::command;

#[command]
pub async fn get_cached_skills() -> Result<Vec<CacheMetadata>, String> {
    Ok(cache::get_all_cached_skills().await)
}

#[command]
pub async fn clear_skill_cache(skill_name: String) -> Result<(), String> {
    cache::clear_skill_cache(&skill_name).await
}

#[command]
pub async fn clear_all_cache() -> Result<(), String> {
    cache::clear_all_cache().await
}
