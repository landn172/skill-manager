use crate::commands::skills::discover_skills;
use crate::commands::skillsmp::fetch_skillsmp_skills;
use crate::models::marketplace::{
    default_sources, MarketplaceSkill, MarketplaceSource, SourceType,
};
use crate::utils::git::{clone_repo, parse_source};
use tokio::fs;

#[tauri::command]
pub async fn get_marketplace_sources() -> Result<Vec<MarketplaceSource>, String> {
    // Return default sources (SkillsMP API + Official Git repos)
    Ok(default_sources())
}

#[tauri::command]
pub async fn fetch_marketplace_skills(
    source_id: Option<String>,
    force_refresh: bool,
) -> Result<Vec<MarketplaceSkill>, String> {
    let mut all_skills = Vec::new();
    let sources = get_marketplace_sources().await?;

    let filtered_sources = if let Some(id) = source_id {
        sources
            .into_iter()
            .filter(|s| s.id == id)
            .collect::<Vec<_>>()
    } else {
        sources
    };

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("gemini-skills-cache");
    let repos_dir = cache_dir.join("repos");
    fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(&repos_dir)
        .await
        .map_err(|e| e.to_string())?;

    for source in filtered_sources {
        if !source.enabled {
            continue;
        }

        match source.source_type {
            SourceType::Api => {
                // Fetch from SkillsMP API
                match fetch_skillsmp_skills(None, Some(1), Some(50), Some("stars".into())).await {
                    Ok(skills) => all_skills.extend(skills),
                    Err(e) => {
                        println!("Failed to fetch from SkillsMP API: {}", e);
                        // Continue with other sources rather than failing entirely
                    }
                }
            }
            SourceType::Git | SourceType::Local => {
                // Handle Git and Local sources (existing logic)
                let skills = fetch_git_source(&source, &cache_dir, &repos_dir, force_refresh).await;
                all_skills.extend(skills);
            }
        }
    }

    Ok(all_skills)
}

/// Fetch skills from a Git or Local source
async fn fetch_git_source(
    source: &MarketplaceSource,
    cache_dir: &std::path::PathBuf,
    repos_dir: &std::path::PathBuf,
    force_refresh: bool,
) -> Vec<MarketplaceSkill> {
    let cache_file = cache_dir.join(format!("{}.json", source.id));
    let repo_cache_dir = repos_dir.join(&source.id);

    // Try to load from cache first if not forcing refresh
    if !force_refresh && cache_file.exists() && repo_cache_dir.exists() {
        if let Ok(content) = fs::read_to_string(&cache_file).await {
            if let Ok(cached_skills) = serde_json::from_str::<Vec<MarketplaceSkill>>(&content) {
                return cached_skills;
            }
        }
    }

    // Check if URL is a local directory that exists
    let use_local = std::path::Path::new(&source.url).exists();

    // Parse source URL to check for subpaths (GitHub tree support)
    let parsed_source = if use_local {
        None
    } else {
        Some(parse_source(&source.url))
    };

    let clone_url = parsed_source
        .as_ref()
        .map(|p| p.url.clone())
        .unwrap_or_else(|| source.url.clone());
    let subpath = parsed_source.as_ref().and_then(|p| p.subpath.clone());

    let search_path_result: Result<String, String> = async {
        if use_local {
            Ok(source.url.clone())
        } else {
            // Clone to persistent cache directory
            if force_refresh && repo_cache_dir.exists() {
                let _ = fs::remove_dir_all(&repo_cache_dir).await;
            }

            if !repo_cache_dir.exists() {
                clone_repo(&clone_url, &repo_cache_dir).await?;
            }
            Ok(repo_cache_dir.to_str().unwrap().to_string())
        }
    }
    .await;

    let search_path = match search_path_result {
        Ok(path) => path,
        Err(e) => {
            println!("Failed to fetch source {}: {}", source.name, e);
            return Vec::new();
        }
    };

    let mut source_skills = Vec::new();
    if let Ok(skills) = discover_skills(search_path, subpath).await {
        for skill in skills {
            source_skills.push(MarketplaceSkill {
                skill,
                source_id: source.id.clone(),
                source_name: source.name.clone(),
                category: None,
                tags: Vec::new(),
                stars: 0,
                repo: None,
                repo_url: None,
            });
        }
    }

    // Save to cache
    if !source_skills.is_empty() {
        if let Ok(json) = serde_json::to_string(&source_skills) {
            let _ = fs::write(&cache_file, json).await;
        }
    }

    source_skills
}
