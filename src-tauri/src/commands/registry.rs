use crate::models::marketplace::MarketplaceSkill;
use crate::models::skill::Skill;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub version: String,
    pub updated_at: Option<String>,
    pub skills: Vec<RegistrySkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySkill {
    pub name: String,
    pub description: String,
    pub path: String, // URL to raw file or git clone url
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Fetch skills from a remote JSON registry
#[tauri::command]
pub async fn fetch_registry_skills(url: String) -> Result<Vec<MarketplaceSkill>, String> {
    // Basic caching logic
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gemini-skills-cache")
        .join("registry");

    fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Use urlencoding to create a filesystem-safe filename from the URL
    let filename = format!("{}.json", urlencoding::encode(&url));
    let cache_file = cache_dir.join(filename);

    let index: RegistryIndex;

    // Try to read from cache first if it's recent (e.g., < 1 hour) - for now just check existence
    // Real implementation: check Mtime
    // For now: Always fetch to ensure freshness, or fallback to cache on error

    match reqwest::get(&url).await {
        Ok(res) => {
            if res.status().is_success() {
                let text = res
                    .text()
                    .await
                    .map_err(|e| format!("Failed to read registry: {}", e))?;

                // Cache it
                let _ = fs::write(&cache_file, &text).await;

                index = serde_json::from_str(&text)
                    .map_err(|e| format!("Invalid registry JSON: {}", e))?;
            } else {
                // Try fallback to cache
                if cache_file.exists() {
                    let text = fs::read_to_string(&cache_file)
                        .await
                        .map_err(|e| e.to_string())?;
                    index = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                } else {
                    return Err(format!("Failed to fetch registry: HTTP {}", res.status()));
                }
            }
        }
        Err(e) => {
            // Try fallback to cache
            if cache_file.exists() {
                let text = fs::read_to_string(&cache_file)
                    .await
                    .map_err(|e| e.to_string())?;
                index = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            } else {
                return Err(format!("Failed to connect to registry: {}", e));
            }
        }
    };

    let skills = index
        .skills
        .into_iter()
        .map(|s| {
            MarketplaceSkill {
                skill: Skill {
                    name: s.name.clone(),
                    description: s.description,
                    path: s.path,
                    version: None,
                    source_id: Some("registry".to_string()),
                    source_name: Some("Registry".to_string()),
                    metadata: HashMap::new(),
                },
                source_id: "registry".to_string(), // This will be overwritten by caller with actual source ID
                source_name: "Registry".to_string(), // Overwritten by caller
                category: s.category,
                tags: s.tags.unwrap_or_default(),
                stars: 0,
                repo: None,
                repo_url: None,
            }
        })
        .collect();

    Ok(skills)
}
