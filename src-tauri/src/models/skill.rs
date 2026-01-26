use crate::models::agent::AgentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: String,
    pub version: Option<String>,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallScope {
    Project,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    #[serde(flatten)]
    pub skill: Skill,
    pub install_date: String,
    pub source: String,
    pub source_id: String,
    pub scope: InstallScope,
    pub agents: Vec<AgentType>,
    pub agent_paths: HashMap<String, String>,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallReceipt {
    pub skill_name: String,
    pub source_url: String,
    pub source_type: String,              // "Git", "Local" - technical source
    pub marketplace_name: Option<String>, // "SkillsMP", "Anthropic Official", etc.
    pub marketplace_id: Option<String>,
    pub installed_at: String,
}
