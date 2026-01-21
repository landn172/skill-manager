use crate::models::agent::{AgentConfig, AgentType};
use crate::utils::db;
use std::collections::HashMap;
use std::path::PathBuf;

const AGENT_OVERRIDES_KEY: &str = "agent_path_overrides";
const CUSTOM_AGENTS_KEY: &str = "custom_agents";

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct CustomAgentDef {
    name: String,
    path: String,
    icon: Option<String>,
}

pub async fn get_all_agents_impl() -> Vec<AgentConfig> {
    // 1. Start with default known agents
    let mut agents = AgentConfig::default_known();

    // 2. Load custom agents from DB
    let custom_agents: HashMap<String, CustomAgentDef> =
        if let Ok(Some(json)) = db::get_config(CUSTOM_AGENTS_KEY) {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            HashMap::new()
        };

    for (_, def) in custom_agents {
        agents.push(AgentConfig {
            agent_type: AgentType::Custom(def.name.clone()),
            name: def.name.clone(),         // ID
            display_name: def.name.clone(), // Display
            skills_dir: "".into(), // Custom agents might not have project-local skills logic defined yet
            global_skills_dir: PathBuf::from(def.path),
            icon: def.icon.unwrap_or_else(|| "Terminal".into()), // Use custom icon or default
            installed: false,                                    // Will be detected
            is_custom: true,
        });
    }

    // 3. Load overrides map: AgentType string -> Path string
    let overrides: HashMap<String, String> =
        if let Ok(Some(json)) = db::get_config(AGENT_OVERRIDES_KEY) {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            HashMap::new()
        };

    // 4. Apply overrides and detection
    for agent in &mut agents {
        // Check for override
        let agent_key = serde_json::to_string(&agent.agent_type)
            .unwrap()
            .replace("\"", "");

        if let Some(path_str) = overrides.get(&agent_key) {
            agent.global_skills_dir = PathBuf::from(path_str);
        }

        AgentConfig::detect_installed(agent).await;
    }

    agents
}

#[tauri::command]
pub async fn detect_agents() -> Result<Vec<AgentConfig>, String> {
    Ok(get_all_agents_impl().await)
}

#[tauri::command]
pub async fn get_agent_config(agent_type: AgentType) -> Result<AgentConfig, String> {
    let agents = get_all_agents_impl().await;
    agents
        .into_iter()
        .find(|a| a.agent_type == agent_type)
        .ok_or_else(|| "Agent not found".into())
}

#[tauri::command]
pub async fn update_agent_path(
    agent_type: AgentType,
    path: String,
) -> Result<Vec<AgentConfig>, String> {
    // Check if it is a custom agent first?
    // Actually, overrides work for custom agents too, but for custom agents we ideally update the definition itself.
    // But keeping it consistent: overrides key takes precedence.

    // Load existing overrides
    let mut overrides: HashMap<String, String> =
        if let Ok(Some(json)) = db::get_config(AGENT_OVERRIDES_KEY) {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            HashMap::new()
        };

    let agent_key = serde_json::to_string(&agent_type)
        .unwrap()
        .replace("\"", "");

    if path.is_empty() {
        overrides.remove(&agent_key);
    } else {
        overrides.insert(agent_key, path);
    }

    let json = serde_json::to_string(&overrides).map_err(|e| e.to_string())?;
    db::set_config(AGENT_OVERRIDES_KEY, &json)?;

    // Return updated list
    detect_agents().await
}

#[tauri::command]
pub async fn add_custom_agent(
    name: String,
    path: String,
    icon: Option<String>,
) -> Result<Vec<AgentConfig>, String> {
    if name.trim().is_empty() || path.trim().is_empty() {
        return Err("Name and path are required".into());
    }

    let mut custom_agents: HashMap<String, CustomAgentDef> =
        if let Ok(Some(json)) = db::get_config(CUSTOM_AGENTS_KEY) {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            HashMap::new()
        };

    let id = name.trim().to_lowercase().replace(" ", "-");

    // Iterate to check for conflicts with known agents ideally, but simplified for now

    custom_agents.insert(
        id.clone(),
        CustomAgentDef {
            name: name.trim().to_string(),
            path: path.trim().to_string(),
            icon,
        },
    );

    let json = serde_json::to_string(&custom_agents).map_err(|e| e.to_string())?;
    db::set_config(CUSTOM_AGENTS_KEY, &json)?;

    detect_agents().await
}

#[tauri::command]
pub async fn remove_custom_agent(agent_type: AgentType) -> Result<Vec<AgentConfig>, String> {
    if let AgentType::Custom(name) = agent_type {
        let mut custom_agents: HashMap<String, CustomAgentDef> =
            if let Ok(Some(json)) = db::get_config(CUSTOM_AGENTS_KEY) {
                serde_json::from_str(&json).unwrap_or_default()
            } else {
                HashMap::new()
            };

        // We need to find the key. The 'name' in Custom(name) might be the Display Name if I used that as ID.
        // In add_custom_agent I used name as Name for AgentType::Custom.
        // Let's iterate and find.

        let mut key_to_remove = None;
        for (k, v) in &custom_agents {
            if v.name == name {
                key_to_remove = Some(k.clone());
                break;
            }
        }

        if let Some(k) = key_to_remove {
            custom_agents.remove(&k);
            let json = serde_json::to_string(&custom_agents).map_err(|e| e.to_string())?;
            db::set_config(CUSTOM_AGENTS_KEY, &json)?;
        }
    } else {
        return Err("Cannot remove built-in agents".into());
    }

    detect_agents().await
}

#[tauri::command]
pub async fn open_in_agent(path: String, agent: AgentType) -> Result<(), String> {
    use crate::models::agent::KnownAgentType;
    use tokio::process::Command;

    match agent {
        AgentType::Known(KnownAgentType::Vscode) => {
            Command::new("code")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Failed to open VS Code: {}", e))?;
        }
        AgentType::Known(KnownAgentType::Cursor) => {
            Command::new("cursor")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("Failed to open Cursor: {}", e))?;
        }
        _ => {
            // For others or custom, fallback to opening folder
            crate::commands::utils::open_in_explorer(path).await?;
        }
    }
    Ok(())
}
