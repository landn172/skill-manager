use crate::models::agent::AgentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: String,
    pub version: Option<String>,
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
