use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    Opencode,
    ClaudeCode,
    Codex,
    Cursor,
    Gemini,
    Vscode,
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
}

impl AgentConfig {
    pub fn all() -> Vec<Self> {
        let home = dirs::home_dir().unwrap_or_default();
        vec![
            AgentConfig {
                agent_type: AgentType::Opencode,
                name: "opencode".into(),
                display_name: "OpenCode".into(),
                skills_dir: ".opencode/skill".into(),
                global_skills_dir: home.join(".config/opencode/skill"),
                icon: "Terminal".into(),
                installed: false,
            },
            AgentConfig {
                agent_type: AgentType::ClaudeCode,
                name: "claude-code".into(),
                display_name: "Claude Code".into(),
                skills_dir: ".claude/skills".into(),
                global_skills_dir: home.join(".claude/skills"),
                icon: "Bot".into(),
                installed: false,
            },
            AgentConfig {
                agent_type: AgentType::Codex,
                name: "codex".into(),
                display_name: "Codex".into(),
                skills_dir: ".codex/skills".into(),
                global_skills_dir: home.join(".codex/skills"),
                icon: "Code".into(),
                installed: false,
            },
            AgentConfig {
                agent_type: AgentType::Cursor,
                name: "cursor".into(),
                display_name: "Cursor".into(),
                skills_dir: ".cursor/skills".into(),
                global_skills_dir: home.join(".cursor/skills"),
                icon: "MousePointer".into(),
                installed: false,
            },
            AgentConfig {
                agent_type: AgentType::Gemini,
                name: "gemini".into(),
                display_name: "Gemini".into(),
                skills_dir: ".gemini/skills".into(),
                global_skills_dir: home.join(".gemini/skills"),
                icon: "Sparkles".into(),
                installed: false,
            },
            AgentConfig {
                agent_type: AgentType::Vscode,
                name: "vscode".into(),
                display_name: "VS Code".into(),
                skills_dir: ".vscode/skills".into(),
                global_skills_dir: home.join(".vscode/skills"),
                icon: "Code".into(),
                installed: false,
            },
        ]
    }

    pub async fn detect_installed(agent: &mut AgentConfig) {
        agent.installed = match agent.agent_type {
            AgentType::Opencode => {
                let home = dirs::home_dir().unwrap_or_default();
                home.join(".config/opencode").exists() || home.join(".claude/skills").exists()
            }
            _ => {
                // For most agents, checking if the global config directory exists is enough
                agent
                    .global_skills_dir
                    .parent()
                    .map(|p| p.exists())
                    .unwrap_or(false)
            }
        };
    }
}
