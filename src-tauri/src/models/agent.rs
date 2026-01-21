use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum KnownAgentType {
    Opencode,
    ClaudeCode,
    Codex,
    Cursor,
    Gemini,
    Vscode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum AgentType {
    Known(KnownAgentType),
    Custom(String),
}

impl From<KnownAgentType> for AgentType {
    fn from(k: KnownAgentType) -> Self {
        AgentType::Known(k)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub name: String,
    pub display_name: String,
    pub skills_dir: String,
    pub global_skills_dir: PathBuf,
    pub icon: String,
    pub installed: bool,
    pub is_custom: bool,
}

impl AgentConfig {
    pub fn default_known() -> Vec<Self> {
        let home = dirs::home_dir().unwrap_or_default();
        vec![
            AgentConfig {
                agent_type: KnownAgentType::Opencode.into(),
                name: "opencode".into(),
                display_name: "OpenCode".into(),
                skills_dir: ".opencode/skill".into(),
                global_skills_dir: home.join(".config/opencode/skill"),
                icon: "Terminal".into(),
                installed: false,
                is_custom: false,
            },
            AgentConfig {
                agent_type: KnownAgentType::ClaudeCode.into(),
                name: "claude-code".into(),
                display_name: "Claude Code".into(),
                skills_dir: ".claude/skills".into(),
                global_skills_dir: home.join(".claude/skills"),
                icon: "Bot".into(),
                installed: false,
                is_custom: false,
            },
            AgentConfig {
                agent_type: KnownAgentType::Codex.into(),
                name: "codex".into(),
                display_name: "Codex".into(),
                skills_dir: ".codex/skills".into(),
                global_skills_dir: home.join(".codex/skills"),
                icon: "Code".into(),
                installed: false,
                is_custom: false,
            },
            AgentConfig {
                agent_type: KnownAgentType::Cursor.into(),
                name: "cursor".into(),
                display_name: "Cursor".into(),
                skills_dir: ".cursor/skills".into(),
                global_skills_dir: home.join(".cursor/skills"),
                icon: "MousePointer".into(),
                installed: false,
                is_custom: false,
            },
            AgentConfig {
                agent_type: KnownAgentType::Gemini.into(),
                name: "gemini".into(),
                display_name: "Gemini".into(),
                skills_dir: ".gemini/skills".into(),
                global_skills_dir: home.join(".gemini/skills"),
                icon: "Sparkles".into(),
                installed: false,
                is_custom: false,
            },
            AgentConfig {
                agent_type: KnownAgentType::Vscode.into(),
                name: "vscode".into(),
                display_name: "VS Code".into(),
                skills_dir: ".vscode/skills".into(),
                global_skills_dir: home.join(".vscode/skills"),
                icon: "Code".into(),
                installed: false,
                is_custom: false,
            },
        ]
    }

    // Kept for backward compatibility, returns default known agents
    pub fn all() -> Vec<Self> {
        Self::default_known()
    }

    pub async fn detect_installed(agent: &mut AgentConfig) {
        match &agent.agent_type {
            AgentType::Known(KnownAgentType::Opencode) => {
                let home = dirs::home_dir().unwrap_or_default();
                agent.installed =
                    home.join(".config/opencode").exists() || home.join(".claude/skills").exists();
            }
            AgentType::Known(_) => {
                // For most known agents, check if global config dir exists
                agent.installed = agent
                    .global_skills_dir
                    .parent()
                    .map(|p| p.exists())
                    .unwrap_or(false);
            }
            AgentType::Custom(_) => {
                // For custom agents, we assume if the user added it and the path is valid, it's "installed"
                // But strictly, we check if the directory exists or can be created.
                // Here we just check if the directory path itself is valid/exists.
                // Actually if it's custom, let's just mark it installed if the path exists.
                agent.installed = agent.global_skills_dir.exists();
            }
        };
    }
}
