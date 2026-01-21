use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize)]
pub struct CreateSkillResult {
    pub success: bool,
    pub path: String,
    pub message: String,
}

#[tauri::command]
pub async fn create_skill(
    name: String,
    description: String,
    parent_path: String,
) -> Result<CreateSkillResult, String> {
    let base_path = PathBuf::from(&parent_path);
    if !base_path.exists() {
        return Err(format!("Parent path does not exist: {}", parent_path));
    }

    // Sanitize name for folder use (kebab-case recommended)
    let folder_name = name
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let skill_dir = base_path.join(&folder_name);

    if skill_dir.exists() {
        return Err(format!("Skill directory already exists: {:?}", skill_dir));
    }

    // Create directory
    fs::create_dir_all(&skill_dir)
        .await
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Create README.md
    let readme_content = format!(
        r#"---
description: {}
---

# {}

{}

"#,
        description, name, description
    );

    fs::write(skill_dir.join("README.md"), readme_content)
        .await
        .map_err(|e| format!("Failed to create README.md: {}", e))?;

    // Create instructions.md (Standard prompt file)
    let instruction_content = format!(
        r#"# {} Instructions

This skill helps you with...

## Usage

Describe how to use this skill here.
"#,
        name
    );

    fs::write(skill_dir.join("instructions.md"), instruction_content)
        .await
        .map_err(|e| format!("Failed to create instructions.md: {}", e))?;

    Ok(CreateSkillResult {
        success: true,
        path: skill_dir.to_str().unwrap_or_default().to_string(),
        message: format!("Skill '{}' created successfully at {:?}", name, skill_dir),
    })
}

/// Delete a local skill directory
#[tauri::command]
pub async fn delete_local_skill(skill_path: String) -> Result<(), String> {
    let path = PathBuf::from(&skill_path);

    if !path.exists() {
        return Err(format!("Skill path does not exist: {}", skill_path));
    }

    // Safety check: ensure it's a directory and contains a README.md or SKILL.md
    if !path.is_dir() {
        return Err("Path is not a directory".into());
    }

    let has_skill_file = path.join("SKILL.md").exists() || path.join("README.md").exists();
    if !has_skill_file {
        return Err(
            "This doesn't appear to be a valid skill directory (missing SKILL.md or README.md)"
                .into(),
        );
    }

    fs::remove_dir_all(&path)
        .await
        .map_err(|e| format!("Failed to delete skill: {}", e))?;

    Ok(())
}

/// Update a local skill's metadata (name, description)
#[tauri::command]
pub async fn update_local_skill(
    skill_path: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), String> {
    let path = PathBuf::from(&skill_path);

    // Find the skill file (SKILL.md or README.md)
    let skill_file = if path.join("SKILL.md").exists() {
        path.join("SKILL.md")
    } else if path.join("README.md").exists() {
        path.join("README.md")
    } else {
        return Err("Skill file not found (SKILL.md or README.md)".into());
    };

    let content = fs::read_to_string(&skill_file)
        .await
        .map_err(|e| format!("Failed to read skill file: {}", e))?;

    // Parse and update frontmatter
    let re_fm = regex::Regex::new(r"(?s)^---\s*(.*?)\s*---(.*)").unwrap();

    if let Some(caps) = re_fm.captures(&content) {
        let mut fm = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let body = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        // Update description in frontmatter
        if let Some(desc) = description {
            let re_desc = regex::Regex::new(r"description:\s*(.+)").unwrap();
            if re_desc.is_match(&fm) {
                fm = re_desc
                    .replace(&fm, format!("description: {}", desc).as_str())
                    .to_string();
            } else {
                fm.push_str(&format!("\ndescription: {}", desc));
            }
        }

        // Update name in frontmatter
        if let Some(n) = name {
            let re_name = regex::Regex::new(r"name:\s*(.+)").unwrap();
            if re_name.is_match(&fm) {
                fm = re_name
                    .replace(&fm, format!("name: {}", n).as_str())
                    .to_string();
            } else {
                fm.push_str(&format!("\nname: {}", n));
            }
        }

        let new_content = format!("---\n{}\n---{}", fm.trim(), body);

        fs::write(&skill_file, new_content)
            .await
            .map_err(|e| format!("Failed to write skill file: {}", e))?;
    } else {
        return Err("Invalid skill file format (no frontmatter found)".into());
    }

    Ok(())
}
