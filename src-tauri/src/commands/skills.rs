use crate::models::agent::AgentConfig;
use crate::models::skill::{InstallScope, InstalledSkill, Skill};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &["node_modules", ".git", "dist", "build", "__pycache__"];

async fn has_skill_md(dir: &Path) -> bool {
    dir.join("SKILL.md").exists() || dir.join("README.md").exists()
}

async fn get_skill_file(dir: &Path) -> Option<std::path::PathBuf> {
    if dir.join("SKILL.md").exists() {
        Some(dir.join("SKILL.md"))
    } else if dir.join("README.md").exists() {
        Some(dir.join("README.md"))
    } else {
        None
    }
}

async fn parse_skill_md(path: &Path) -> Option<Skill> {
    let content = fs::read_to_string(path).await.ok()?;

    // Very basic frontmatter parser using regex
    let re_fm = Regex::new(r"(?s)^---\s*(.*?)\s*---").unwrap();
    let caps = re_fm.captures(&content)?;
    let fm_content = caps.get(1)?.as_str();

    let re_name = Regex::new(r"name:\s*(.+)").unwrap();
    let re_desc = Regex::new(r"description:\s*(.+)").unwrap();
    let re_ver = Regex::new(r"version:\s*(.+)").unwrap();

    // Name is optional - derive from directory name if not present
    let name = re_name
        .captures(fm_content)
        .and_then(|c| c.get(1))
        .map(|m| {
            m.as_str()
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .unwrap_or_else(|| {
            // Derive name from parent directory
            path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown-skill")
                .to_string()
        });

    // Description is also optional now
    let description = re_desc
        .captures(fm_content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().trim_matches('\'').to_string())
        .unwrap_or_else(|| "No description".to_string());

    let version = re_ver.captures(fm_content).and_then(|c| c.get(1)).map(|m| {
        m.as_str()
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string()
    });

    Some(Skill {
        name,
        description,
        path: path.parent().unwrap().to_str().unwrap().to_string(),
        version,
        metadata: HashMap::new(),
    })
}

#[tauri::command]
pub async fn discover_skills(path: String, subpath: Option<String>) -> Result<Vec<Skill>, String> {
    let base_path = PathBuf::from(&path);
    let search_path = match subpath {
        Some(sub) => base_path.join(sub),
        None => base_path,
    };

    let mut skills = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    // Check directly
    if has_skill_md(&search_path).await {
        if let Some(skill_file) = get_skill_file(&search_path).await {
            if let Some(skill) = parse_skill_md(&skill_file).await {
                skills.push(skill);
                return Ok(skills);
            }
        }
    }

    // Recursive search with walkdir
    for entry in WalkDir::new(&search_path)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| !SKIP_DIRS.contains(&e.file_name().to_str().unwrap_or("")))
        .filter_map(|e| e.ok())
    {
        let file_name = entry.file_name().to_str().unwrap_or("");
        if file_name == "SKILL.md" || file_name == "README.md" {
            if let Some(skill) = parse_skill_md(entry.path()).await {
                if !seen_names.contains(&skill.name) {
                    seen_names.insert(skill.name.clone());
                    skills.push(skill);
                }
            }
        }
    }

    Ok(skills)
}

#[tauri::command]
pub async fn list_installed_skills(
    scope: InstallScope,
    project_path: Option<String>,
) -> Result<Vec<InstalledSkill>, String> {
    let agents = AgentConfig::all();
    let mut skill_map: std::collections::HashMap<String, InstalledSkill> =
        std::collections::HashMap::new();

    for agent in agents {
        let skills_dir = match scope {
            InstallScope::Global => agent.global_skills_dir.clone(),
            InstallScope::Project => {
                let base_path = project_path
                    .as_ref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                base_path.join(&agent.skills_dir)
            }
        };

        if !skills_dir.exists() {
            continue;
        }

        if let Ok(mut entries) = fs::read_dir(skills_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(ty) = entry.file_type().await {
                    if ty.is_dir() {
                        let skill_dir = entry.path();
                        if let Some(skill) = parse_skill_md(&skill_dir.join("SKILL.md")).await {
                            let name = skill.name.clone();
                            if let Some(existing) = skill_map.get_mut(&name) {
                                existing.agents.push(agent.agent_type.clone());
                            } else {
                                skill_map.insert(
                                    name.clone(),
                                    InstalledSkill {
                                        installed_version: skill.version.clone(),
                                        skill,
                                        install_date: "".into(),
                                        source: "Local".into(),
                                        source_id: "".into(),
                                        scope: scope.clone(),
                                        agents: vec![agent.agent_type.clone()],
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(skill_map.into_values().collect())
}

#[tauri::command]
pub async fn get_skill_content(skill_path: String) -> Result<String, String> {
    let path = PathBuf::from(skill_path).join("SKILL.md");
    fs::read_to_string(path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_skill_content(skill_path: String, content: String) -> Result<(), String> {
    let path = PathBuf::from(skill_path).join("SKILL.md");
    fs::write(path, content).await.map_err(|e| e.to_string())
}
