use crate::models::skill::Skill;
use serde::{Deserialize, Serialize};

/// Type of marketplace source
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Git, // GitHub/Git repo cloning
    #[default]
    Api, // SkillsMP API
    Local, // Local directory
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub official: bool,
    pub enabled: bool,
    pub last_fetched: Option<String>,
    #[serde(default)]
    pub source_type: SourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSkill {
    #[serde(flatten)]
    pub skill: Skill,
    pub source_id: String,
    pub source_name: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    // Additional fields from SkillsMP API
    #[serde(default)]
    pub stars: u32,
    pub repo: Option<String>,
    pub repo_url: Option<String>,
}

// SkillsMP API Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsmpApiResponse {
    pub success: bool,
    #[serde(default)]
    pub data: Vec<SkillsmpSkill>,
    pub pagination: Option<SkillsmpPagination>,
    pub error: Option<SkillsmpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsmpSkill {
    pub name: String,
    #[serde(alias = "fileName")]
    pub file_name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub stars: u32,
    pub repo: Option<String>,
    pub url: Option<String>,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsmpPagination {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsmpError {
    pub code: String,
    pub message: String,
}

/// Default sources include only Official repos and the SkillsMP API
pub fn default_sources() -> Vec<MarketplaceSource> {
    vec![
        // SkillsMP API - Primary source with 65,000+ skills
        MarketplaceSource {
            id: "skillsmp".into(),
            name: "SkillsMP".into(),
            url: "https://skillsmp.com/api/v1".into(),
            description: Some("Skills Marketplace API with 65,000+ indexed skills".into()),
            official: false,
            enabled: true,
            last_fetched: None,
            source_type: SourceType::Api,
        },
        // Official sources (Git-based)
        MarketplaceSource {
            id: "anthropics".into(),
            name: "Anthropic Official".into(),
            url: "https://github.com/anthropics/skills.git".into(),
            description: Some("Official skills from Anthropic".into()),
            official: true,
            enabled: true,
            last_fetched: None,
            source_type: SourceType::Git,
        },
        MarketplaceSource {
            id: "vercel-labs".into(),
            name: "Vercel Labs".into(),
            url: "https://github.com/vercel-labs/agent-skills.git".into(),
            description: Some("Official curated skills from Vercel Labs".into()),
            official: true,
            enabled: true,
            last_fetched: None,
            source_type: SourceType::Git,
        },
    ]
}
