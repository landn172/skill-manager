use crate::models::agent::AgentType;
use crate::models::skill::{InstallScope, Skill};
use crate::utils::db::log_action;
use crate::utils::fs::{copy_dir_recursive, remove_quarantine};
use crate::utils::git::{clone_repo, parse_source};
use tokio::fs;

#[derive(serde::Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub path: String,
    pub agent: AgentType,
    pub error: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct ProgressEvent {
    pub skill: String,
    pub status: String, // "downloading", "cloning", "installing", "finished", "error"
    pub message: String,
    pub agent: Option<String>, // AgentType as string
}

#[tauri::command]
pub async fn install_skill(
    app: tauri::AppHandle,
    skill: Skill,
    agents: Vec<AgentType>,
    scope: InstallScope,
) -> Result<Vec<InstallResult>, String> {
    use tauri::Emitter;

    let mut results = Vec::new();
    let all_agent_configs = crate::commands::agents::get_all_agents_impl().await;
    let total_agents = agents.len();

    // Check if we have a repo in metadata (from SkillsMP)
    let has_repo_metadata = skill
        .metadata
        .get("repo")
        .or(skill.metadata.get("repo_url"))
        .map(|r| !r.is_empty())
        .unwrap_or(false);

    // Determine if this is a remote skill (from SkillsMP or URL-based)
    let is_remote = skill.path.starts_with("http://")
        || skill.path.starts_with("https://")
        || has_repo_metadata
        || skill.path.is_empty(); // Empty path with repo metadata = remote skill

    // Emit start event
    let _ = app.emit(
        "install-progress",
        ProgressEvent {
            skill: skill.name.clone(),
            status: "start".to_string(),
            message: if is_remote {
                "Starting download...".to_string()
            } else {
                "Starting installation...".to_string()
            },
            agent: None,
        },
    );

    // For remote skills, we need to download to a temp location first
    let source_path = if is_remote {
        let _ = app.emit(
            "install-progress",
            ProgressEvent {
                skill: skill.name.clone(),
                status: "downloading".to_string(),
                message: "Finding repository...".to_string(),
                agent: None,
            },
        );

        // Check if we have a repo field (from SkillsMP)
        let repo_url = skill
            .metadata
            .get("repo")
            .filter(|r| !r.is_empty())
            .map(|r| {
                // If repo is in "owner/repo" format, convert to full URL
                if !r.contains("://") {
                    format!("https://github.com/{}", r)
                } else {
                    r.to_string()
                }
            })
            .or_else(|| {
                skill
                    .metadata
                    .get("repo_url")
                    .filter(|r| !r.is_empty())
                    .map(|r| r.to_string())
            })
            .or_else(|| {
                // Try to extract repo from path if it's a GitHub URL
                if skill.path.contains("github.com") {
                    Some(skill.path.clone())
                } else if skill.path.contains("raw.githubusercontent.com") {
                    // Convert raw.githubusercontent.com/owner/repo/branch/path to github clone URL
                    let re =
                        regex::Regex::new(r"raw\.githubusercontent\.com/([^/]+)/([^/]+)").ok()?;
                    re.captures(&skill.path)
                        .map(|c| format!("https://github.com/{}/{}.git", &c[1], &c[2]))
                } else {
                    None
                }
            });

        if let Some(repo) = repo_url {
            let _ = app.emit(
                "install-progress",
                ProgressEvent {
                    skill: skill.name.clone(),
                    status: "downloading".to_string(),
                    message: format!("Downloading from {}...", repo),
                    agent: None,
                },
            );

            // Clone the repo to a temp directory
            let cache_dir = dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                .join("gemini-skills-cache")
                .join("downloads")
                .join(&skill.name);

            // Remove old cache if exists
            let _ = fs::remove_dir_all(&cache_dir).await;

            let parsed = parse_source(&repo);
            println!("Installing remote skill {} from {}", skill.name, parsed.url);

            if let Err(e) = clone_repo(&parsed.url, &cache_dir).await {
                let err_msg = format!("Failed to download skill from {}: {}", parsed.url, e);
                let _ = app.emit(
                    "install-progress",
                    ProgressEvent {
                        skill: skill.name.clone(),
                        status: "error".to_string(),
                        message: err_msg.clone(),
                        agent: None,
                    },
                );
                return Err(err_msg);
            }

            // If there's a subpath in the repo, use that
            if let Some(subpath) = parsed.subpath {
                cache_dir.join(subpath)
            } else {
                cache_dir
            }
        } else {
            let err_msg = format!(
                "Cannot install remote skill '{}': No GitHub repository found. Path: '{}', Metadata: {:?}",
                skill.name, skill.path, skill.metadata
            );
            return Err(err_msg);
        }
    } else {
        // Local path
        std::path::PathBuf::from(&skill.path)
    };

    for (i, agent_type) in agents.iter().enumerate() {
        // Simple debug string for now, or use display trait if available
        let agent_name = serde_json::to_string(&agent_type)
            .unwrap_or_else(|_| "unknown".to_string())
            .replace("\"", "");

        let _ = app.emit(
            "install-progress",
            ProgressEvent {
                skill: skill.name.clone(),
                status: "installing".to_string(),
                message: format!("Installing for {} ({}/{})", agent_name, i + 1, total_agents),
                agent: Some(agent_name.clone()),
            },
        );

        let agent_config = all_agent_configs
            .iter()
            .find(|a| &a.agent_type == agent_type)
            .ok_or_else(|| format!("Agent configuration not found for {:?}", agent_type))?;

        let target_base = match scope {
            InstallScope::Global => agent_config.global_skills_dir.clone(),
            InstallScope::Project => {
                // In project scope, we use current directory or a configured project path
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(&agent_config.skills_dir)
            }
        };

        let target_dir = target_base.join(&skill.name);

        match copy_dir_recursive(&source_path, &target_dir).await {
            Ok(_) => {
                // Remove quarantine on macOS to avoid permissions issues
                let _ = remove_quarantine(&target_dir).await;

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
                    agent: agent_type.clone(),
                    error: Some(e),
                });
            }
        }
    }

    let _ = app.emit(
        "install-progress",
        ProgressEvent {
            skill: skill.name.clone(),
            status: "finished".to_string(),
            message: "Installation complete!".to_string(),
            agent: None,
        },
    );

    Ok(results)
}

#[tauri::command]
pub async fn uninstall_skill(
    skill_name: String,
    agent: AgentType,
    scope: InstallScope,
) -> Result<(), String> {
    let all_agent_configs = crate::commands::agents::get_all_agents_impl().await;
    let agent_config = all_agent_configs
        .iter()
        .find(|a| a.agent_type == agent)
        .ok_or_else(|| "Agent not found".to_string())?;

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
    let all_agent_configs = crate::commands::agents::get_all_agents_impl().await;
    let agent_config = all_agent_configs
        .iter()
        .find(|a| a.agent_type == agent)
        .ok_or_else(|| "Agent not found".to_string())?;

    let target_base = match scope {
        InstallScope::Global => agent_config.global_skills_dir.clone(),
        InstallScope::Project => std::env::current_dir()
            .unwrap_or_default()
            .join(&agent_config.skills_dir),
    };

    Ok(target_base.join(skill_name).exists())
}
