use crate::utils::fs::link_dir_recursive;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[tauri::command]
pub async fn migrate_existing_skills_to_hardlinks() -> Result<(), String> {
    // Check if migration already ran (using a marker file in cache dir)
    let marker_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gemini-skills-cache")
        .join(".migration_hardlinks_complete");

    if marker_path.exists() {
        return Ok(());
    }

    println!("Starting silent migration of skills to hard links...");

    let agents = crate::commands::agents::get_all_agents_impl().await;

    // 1. Discover all installed skills across all agents
    // Map<SkillName, Vec<(AgentType, SkillPath)>>
    let mut skill_locations: HashMap<String, Vec<(String, PathBuf)>> = HashMap::new();

    for agent in &agents {
        // Check global scope only for now as it's the most common candidate for duplication
        let skills_dir = agent.global_skills_dir.clone();
        if !skills_dir.exists() {
            continue;
        }

        if let Ok(mut entries) = fs::read_dir(&skills_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(ty) = entry.file_type().await {
                    if ty.is_dir() {
                        let skill_name = entry.file_name().to_string_lossy().to_string();
                        // Verify it's a valid skill (has SKILL.md)
                        if entry.path().join("SKILL.md").exists() {
                            let agent_key = serde_json::to_string(&agent.agent_type)
                                .unwrap_or_default()
                                .replace("\"", "");

                            skill_locations
                                .entry(skill_name)
                                .or_default()
                                .push((agent_key, entry.path()));
                        }
                    }
                }
            }
        }
    }

    // 2. For each skill with > 1 installation, migrate
    for (skill_name, locations) in skill_locations {
        if locations.len() < 2 {
            continue;
        }

        println!(
            "Migrating skill '{}' with {} copies",
            skill_name,
            locations.len()
        );

        // Pick primary - just use the first one
        let (_primary_agent, primary_path) = &locations[0];

        // Migrate others
        for (agent_key, path) in locations.iter().skip(1) {
            println!(
                "  Linking {} (for {}) -> {}",
                skill_name,
                agent_key,
                primary_path.display()
            );

            // Move existing to backup (just in case), or delete
            // For safety, let's try to delete and link. If link fails, we need to handle it.
            // But to be safe against data loss, we rename first.
            let backup_path = path.with_extension("bak_migration");

            if let Err(e) = fs::rename(path, &backup_path).await {
                println!("  Failed to backup {}: {}", path.display(), e);
                continue;
            }

            // Create hard link copy
            if let Err(e) = link_dir_recursive(primary_path, path).await {
                println!("  Failed to link {}: {}", path.display(), e);
                // Restore backup
                let _ = fs::rename(&backup_path, path).await;
            } else {
                // Success - remove backup
                let _ = fs::remove_dir_all(&backup_path).await;
            }
        }
    }

    // Mark complete
    if let Some(parent) = marker_path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    let _ = fs::write(marker_path, chrono::Utc::now().to_rfc3339()).await;

    println!("Migration complete.");
    Ok(())
}
