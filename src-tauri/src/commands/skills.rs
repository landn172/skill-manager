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
    let fm_content = if let Some(caps) = re_fm.captures(&content) {
        caps.get(1).map(|m| m.as_str()).unwrap_or("")
    } else {
        ""
    };

    let re_name = Regex::new(r"name:\s*(.+)").unwrap();
    let re_desc = Regex::new(r"description:\s*(.+)").unwrap();
    let re_ver = Regex::new(r"version:\s*(.+)").unwrap();

    let mut name = re_name
        .captures(fm_content)
        .and_then(|c| c.get(1))
        .map(|m| {
            m.as_str()
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        });

    let mut description = re_desc
        .captures(fm_content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().trim_matches('\'').to_string());

    // Use pulldown-cmark for robust extraction if frontmatter is missing some fields
    if name.is_none() || description.is_none() {
        use pulldown_cmark::{Event, Parser, Tag, TagEnd};
        let parser = Parser::new(&content);
        let mut in_h1 = false;
        let mut in_p = false;
        let mut h1_text = String::new();
        let mut p_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. })
                    if level == pulldown_cmark::HeadingLevel::H1 =>
                {
                    if name.is_none() && h1_text.is_empty() {
                        in_h1 = true;
                    }
                }
                Event::End(TagEnd::Heading(level)) if level == pulldown_cmark::HeadingLevel::H1 => {
                    in_h1 = false;
                }
                Event::Start(Tag::Paragraph) => {
                    if description.is_none() && p_text.is_empty() {
                        in_p = true;
                    }
                }
                Event::End(TagEnd::Paragraph) => {
                    in_p = false;
                    // Stop after first paragraph if we found one
                    if !p_text.is_empty() && description.is_none() {
                        description = Some(p_text.clone());
                    }
                }
                Event::Text(text) => {
                    if in_h1 {
                        h1_text.push_str(&text);
                    } else if in_p {
                        p_text.push_str(&text);
                    }
                }
                _ => {}
            }
        }

        if name.is_none() && !h1_text.is_empty() {
            let potential_name = h1_text.trim().to_string();
            let lower_name = potential_name.to_lowercase();
            // Skip extremely generic titles
            if lower_name != "skills" && lower_name != "readme" && lower_name != "getting started" {
                name = Some(potential_name);
            }
        }
    }

    // Fallbacks
    let final_name = name.unwrap_or_else(|| {
        // Derive name from parent directory
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown-skill")
            .to_string()
    });

    let final_description = description.unwrap_or_else(|| "No description".to_string());

    let version = re_ver.captures(fm_content).and_then(|c| c.get(1)).map(|m| {
        m.as_str()
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string()
    });

    Some(Skill {
        name: final_name,
        description: final_description,
        path: path.parent().unwrap().to_str().unwrap().to_string(),
        version,
        source_id: None,
        source_name: None,
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
    let agents = crate::commands::agents::get_all_agents_impl().await;
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
                        // Use get_skill_file to check for either SKILL.md or README.md
                        if let Some(skill_file) = get_skill_file(&skill_dir).await {
                            if let Some(skill) = parse_skill_md(&skill_file).await {
                                let name = skill.name.clone();
                                let agent_key = serde_json::to_string(&agent.agent_type)
                                    .unwrap()
                                    .replace("\"", "");
                                let skill_path = skill.path.clone();

                                if let Some(existing) = skill_map.get_mut(&name) {
                                    existing.agents.push(agent.agent_type.clone());
                                    existing.agent_paths.insert(agent_key, skill_path);
                                } else {
                                    let mut agent_paths = std::collections::HashMap::new();
                                    agent_paths.insert(agent_key, skill_path.clone());

                                    // Try to read install receipt
                                    let receipt_path =
                                        PathBuf::from(&skill_path).join(".install_receipt.json");
                                    let (source, source_id, install_date) = if let Ok(content) =
                                        fs::read_to_string(receipt_path).await
                                    {
                                        if let Ok(receipt) =
                                            serde_json::from_str::<
                                                crate::models::skill::InstallReceipt,
                                            >(&content)
                                        {
                                            (
                                                receipt
                                                    .marketplace_name
                                                    .unwrap_or(receipt.source_type),
                                                receipt
                                                    .marketplace_id
                                                    .unwrap_or(receipt.source_url),
                                                receipt.installed_at,
                                            )
                                        } else {
                                            ("Local".into(), "".into(), "".into())
                                        }
                                    } else {
                                        ("Local".into(), "".into(), "".into())
                                    };

                                    skill_map.insert(
                                        name.clone(),
                                        InstalledSkill {
                                            installed_version: skill.version.clone(),
                                            skill,
                                            install_date,
                                            source,
                                            source_id,
                                            scope: scope.clone(),
                                            agents: vec![agent.agent_type.clone()],
                                            agent_paths,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut skills: Vec<InstalledSkill> = skill_map.into_values().collect();
    skills.sort_by(|a, b| {
        a.skill
            .name
            .to_lowercase()
            .cmp(&b.skill.name.to_lowercase())
    });
    Ok(skills)
}

#[derive(serde::Serialize)]
pub struct SkillContent {
    content: String,
    filename: String,
}

#[tauri::command]
pub async fn get_skill_content(skill_path: String) -> Result<SkillContent, String> {
    let dir = PathBuf::from(skill_path);
    let (path, filename) = if dir.join("SKILL.md").exists() {
        (dir.join("SKILL.md"), "SKILL.md".to_string())
    } else if dir.join("README.md").exists() {
        (dir.join("README.md"), "README.md".to_string())
    } else {
        // Default to SKILL.md if neither exists (though checking existence first is better)
        (dir.join("SKILL.md"), "SKILL.md".to_string())
    };

    let content = fs::read_to_string(path).await.map_err(|e| e.to_string())?;

    Ok(SkillContent { content, filename })
}

#[tauri::command]
pub async fn save_skill_content(
    skill_path: String,
    content: String,
    filename: Option<String>,
) -> Result<(), String> {
    let fname = filename.unwrap_or_else(|| "SKILL.md".to_string());
    let path = PathBuf::from(skill_path).join(fname);
    fs::write(path, content).await.map_err(|e| e.to_string())
}
