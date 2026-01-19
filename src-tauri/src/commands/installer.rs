use crate::models::agent::{AgentConfig, AgentType};
use crate::models::skill::{InstallScope, Skill};
use crate::utils::db::log_action;
use crate::utils::fs::copy_dir_recursive;
use tokio::fs;

#[derive(serde::Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub path: String,
    pub agent: AgentType,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn install_skill(
    skill: Skill,
    agents: Vec<AgentType>,
    scope: InstallScope,
) -> Result<Vec<InstallResult>, String> {
    let mut results = Vec::new();
    let all_agent_configs = AgentConfig::all();

    for agent_type in agents {
        let agent_config = all_agent_configs
            .iter()
            .find(|a| a.agent_type == agent_type)
            .unwrap();

        let target_base = match scope {
            InstallScope::Global => agent_config.global_skills_dir.clone(),
            InstallScope::Project => {
                // In project scope, we use current directory or a configured project path
                // For simplicity now assume current dir or passed in path (not implemented yet)
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(&agent_config.skills_dir)
            }
        };

        let target_dir = target_base.join(&skill.name);

        match copy_dir_recursive(&skill.path, &target_dir).await {
            Ok(_) => {
                results.push(InstallResult {
                    success: true,
                    path: target_dir.to_str().unwrap_or_default().to_string(),
                    agent: agent_type.clone(),
                    error: None,
                });

                // Log to history
                let _ = log_action(
                    &skill.name,
                    &format!("{:?}", agent_type),
                    &format!("{:?}", scope),
                    skill.version.as_deref(),
                    "install",
                );
            }
            Err(e) => {
                results.push(InstallResult {
                    success: false,
                    path: target_dir.to_str().unwrap_or_default().to_string(),
                    agent: agent_type,
                    error: Some(e),
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn uninstall_skill(
    skill_name: String,
    agent: AgentType,
    scope: InstallScope,
) -> Result<(), String> {
    let all_agent_configs = AgentConfig::all();
    let agent_config = all_agent_configs
        .iter()
        .find(|a| a.agent_type == agent)
        .unwrap();

    let target_base = match scope {
        InstallScope::Global => agent_config.global_skills_dir.clone(),
        InstallScope::Project => std::env::current_dir()
            .unwrap_or_default()
            .join(&agent_config.skills_dir),
    };

    let target_dir = target_base.join(&skill_name);
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)
            .await
            .map_err(|e| e.to_string())?;

        // Log to history
        let _ = log_action(
            &skill_name,
            &format!("{:?}", agent),
            &format!("{:?}", scope),
            None,
            "uninstall",
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn is_skill_installed(
    skill_name: String,
    agent: AgentType,
    scope: InstallScope,
) -> Result<bool, String> {
    let all_agent_configs = AgentConfig::all();
    let agent_config = all_agent_configs
        .iter()
        .find(|a| a.agent_type == agent)
        .unwrap();

    let target_base = match scope {
        InstallScope::Global => agent_config.global_skills_dir.clone(),
        InstallScope::Project => std::env::current_dir()
            .unwrap_or_default()
            .join(&agent_config.skills_dir),
    };

    Ok(target_base.join(skill_name).exists())
}
