use crate::models::marketplace::{MarketplaceSkill, SkillsmpApiResponse, SkillsmpSkill};
use crate::models::skill::Skill;
use crate::utils::db;
use std::collections::HashMap;
use std::env;

const SKILLSMP_API_BASE: &str = "https://skillsmp.com/api/v1";
const SKILLSMP_API_KEY_CONFIG: &str = "skillsmp_api_key";
const SKILLSMP_API_KEY_ENV: &str = "SKILLSMP_API_KEY";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Helper to create a reqwest client with proper headers
fn create_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("Failed to create API client: {}", e))
}

/// Load .env file if it exists (call once at startup)
pub fn load_dotenv() {
    // Try loading from current directory first, then look for common locations
    let _ = dotenvy::dotenv(); // Silently ignore if .env doesn't exist
}

/// Get API key from environment variable first, then from database
fn get_api_key() -> Result<Option<String>, String> {
    // Priority 1: Environment variable (from .env or system)
    if let Ok(key) = env::var(SKILLSMP_API_KEY_ENV) {
        if !key.trim().is_empty() {
            return Ok(Some(key));
        }
    }

    // Priority 2: Database config
    db::get_config(SKILLSMP_API_KEY_CONFIG)
}

/// Get the stored SkillsMP API key
#[tauri::command]
pub async fn get_skillsmp_api_key() -> Result<Option<String>, String> {
    get_api_key()
}

/// Get masked API key for display (shows only first 4 and last 4 chars)
#[tauri::command]
pub async fn get_skillsmp_api_key_masked() -> Result<Option<String>, String> {
    let key = get_api_key()?;
    Ok(key.map(|k| {
        if k.len() > 8 {
            format!("{}...{}", &k[..4], &k[k.len() - 4..])
        } else {
            "****".to_string()
        }
    }))
}

/// Get the source of the API key (env or db)
#[tauri::command]
pub async fn get_skillsmp_api_key_source() -> Result<Option<String>, String> {
    // Check environment first
    if let Ok(key) = env::var(SKILLSMP_API_KEY_ENV) {
        if !key.trim().is_empty() {
            return Ok(Some("env".to_string()));
        }
    }

    // Check database
    if let Ok(Some(_)) = db::get_config(SKILLSMP_API_KEY_CONFIG) {
        return Ok(Some("db".to_string()));
    }

    Ok(None)
}

/// Set the SkillsMP API key (saves to database)
#[tauri::command]
pub async fn set_skillsmp_api_key(key: String) -> Result<(), String> {
    if key.trim().is_empty() {
        db::delete_config(SKILLSMP_API_KEY_CONFIG)
    } else {
        db::set_config(SKILLSMP_API_KEY_CONFIG, &key)
    }
}

/// Clear the SkillsMP API key from database
#[tauri::command]
pub async fn clear_skillsmp_api_key() -> Result<(), String> {
    db::delete_config(SKILLSMP_API_KEY_CONFIG)
}

/// Fetch skills from SkillsMP API using keyword search
#[tauri::command]
pub async fn fetch_skillsmp_skills(
    query: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
) -> Result<Vec<MarketplaceSkill>, String> {
    let api_key = get_api_key()?.ok_or(
        "SkillsMP API key not configured. Add SKILLSMP_API_KEY to .env or configure in Settings.",
    )?;

    let client = create_client()?;

    let mut url = format!("{}/skills/search", SKILLSMP_API_BASE);
    let mut params = Vec::new();

    let search_query = query.as_deref().unwrap_or("*");
    let effective_query = if search_query.is_empty() {
        "*"
    } else {
        search_query
    };
    params.push(format!("q={}", urlencoding::encode(effective_query)));
    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(l) = limit {
        params.push(format!("limit={}", l.min(100)));
    }
    if let Some(s) = &sort_by {
        params.push(format!("sortBy={}", s));
    }

    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from SkillsMP: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("SkillsMP API error ({}): {}", status, text));
    }

    let api_response: SkillsmpApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse SkillsMP response: {}", e))?;

    if !api_response.success {
        if let Some(err) = api_response.error {
            return Err(format!("SkillsMP error: {} - {}", err.code, err.message));
        }
        return Err("SkillsMP request failed".into());
    }

    Ok(convert_skillsmp_skills(api_response.data))
}

/// Semantic AI search using SkillsMP's AI-powered endpoint
#[tauri::command]
pub async fn search_skillsmp_ai(query: String) -> Result<Vec<MarketplaceSkill>, String> {
    if query.trim().is_empty() {
        return Err("Search query cannot be empty for AI search".into());
    }

    let api_key = get_api_key()?.ok_or(
        "SkillsMP API key not configured. Add SKILLSMP_API_KEY to .env or configure in Settings.",
    )?;

    let client = create_client()?;

    let url = format!(
        "{}/skills/ai-search?q={}",
        SKILLSMP_API_BASE,
        urlencoding::encode(&query)
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from SkillsMP AI: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("SkillsMP AI API error ({}): {}", status, text));
    }

    let api_response: SkillsmpApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse SkillsMP AI response: {}", e))?;

    if !api_response.success {
        if let Some(err) = api_response.error {
            return Err(format!("SkillsMP AI error: {} - {}", err.code, err.message));
        }
        return Err("SkillsMP AI search failed".into());
    }

    Ok(convert_skillsmp_skills(api_response.data))
}

/// Convert SkillsMP API skills to MarketplaceSkill format
fn convert_skillsmp_skills(skills: Vec<SkillsmpSkill>) -> Vec<MarketplaceSkill> {
    skills
        .into_iter()
        .map(|s| {
            let name = s.name.clone();
            let description = s.description.clone().unwrap_or_default();
            let path = s.url.clone().unwrap_or_default();

            let mut metadata = HashMap::new();
            if let Some(r) = &s.repo {
                metadata.insert("repo".into(), r.clone());
                metadata.insert("repo_url".into(), format!("https://github.com/{}", r));
            }

            MarketplaceSkill {
                skill: Skill {
                    name,
                    description,
                    path,
                    version: None,
                    source_id: Some("skillsmp".into()),
                    source_name: Some("SkillsMP".into()),
                    metadata,
                },
                source_id: "skillsmp".into(),
                source_name: "SkillsMP".into(),
                category: None,
                tags: Vec::new(),
                stars: s.stars,
                repo: s.repo.clone(),
                repo_url: s.repo.map(|r| format!("https://github.com/{}", r)),
            }
        })
        .collect()
}
