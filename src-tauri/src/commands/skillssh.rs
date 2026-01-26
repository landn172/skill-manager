use crate::models::marketplace::{MarketplaceSkill, SkillsShApiResponse};
use crate::models::skill::Skill;
use std::collections::HashMap;

const SKILLSSH_API_URL: &str = "https://skills.sh/api/skills";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Helper to create a reqwest client with proper headers
fn create_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("Failed to create API client: {}", e))
}

/// Fetch skills from Skills.sh API
#[tauri::command]
pub async fn fetch_skillssh_skills() -> Result<Vec<MarketplaceSkill>, String> {
    let client = create_client()?;

    let response = client
        .get(SKILLSSH_API_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from Skills.sh: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Skills.sh API error ({}): {}", status, text));
    }

    let api_response: SkillsShApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Skills.sh response: {}", e))?;

    Ok(convert_skillssh_skills(api_response.skills))
}

/// Search skills from Skills.sh (client-side filter since API doesn't support search)
#[tauri::command]
pub async fn search_skillssh_skills(query: String) -> Result<Vec<MarketplaceSkill>, String> {
    let all_skills = fetch_skillssh_skills().await?;

    if query.trim().is_empty() {
        return Ok(all_skills);
    }

    let query_lower = query.to_lowercase();
    let filtered: Vec<MarketplaceSkill> = all_skills
        .into_iter()
        .filter(|skill| {
            skill.skill.name.to_lowercase().contains(&query_lower)
                || skill
                    .skill
                    .description
                    .to_lowercase()
                    .contains(&query_lower)
        })
        .collect();

    Ok(filtered)
}

/// Convert Skills.sh API skills to MarketplaceSkill format
fn convert_skillssh_skills(
    skills: Vec<crate::models::marketplace::SkillsShSkill>,
) -> Vec<MarketplaceSkill> {
    skills
        .into_iter()
        .map(|s| {
            // topSource is in "owner/repo" format, construct GitHub URL
            let repo_url = format!("https://github.com/{}", s.top_source);
            let mut metadata = HashMap::new();
            metadata.insert("repo".into(), s.top_source.clone());
            metadata.insert("repo_url".into(), repo_url.clone());

            MarketplaceSkill {
                skill: Skill {
                    name: s.name.clone(),
                    description: format!("From {} - {} installs", s.top_source, s.installs),
                    path: repo_url.clone(),
                    version: None,
                    source_id: Some("skillssh".into()),
                    source_name: Some("Skills.sh".into()),
                    metadata,
                },
                source_id: "skillssh".into(),
                source_name: "Skills.sh".into(),
                category: None,
                tags: Vec::new(),
                stars: s.installs, // Use installs as popularity metric
                repo: Some(s.top_source.clone()),
                repo_url: Some(repo_url),
            }
        })
        .collect()
}
