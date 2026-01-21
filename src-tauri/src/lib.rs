mod commands;
mod models;
mod utils;

use commands::agents::*;
use commands::authoring::{create_skill, delete_local_skill, update_local_skill};
use commands::config::*;
use commands::installer::*;
use commands::marketplace::*;
use commands::projects::*;
use commands::skills::*;
use commands::skillsmp::*;
use commands::utils::*;
use utils::db::init_db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file for environment variables
    commands::skillsmp::load_dotenv();

    init_db().expect("failed to initialize history database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // Agents
            detect_agents,
            get_agent_config,
            update_agent_path,
            add_custom_agent,
            remove_custom_agent,
            open_in_agent,
            // Skills
            discover_skills,
            get_skill_content,
            save_skill_content,
            list_installed_skills,
            install_skill,
            uninstall_skill,
            is_skill_installed,
            create_skill,
            delete_local_skill,
            update_local_skill,
            // Projects
            list_projects,
            add_project,
            remove_project,
            // Marketplace
            get_marketplace_sources,
            fetch_marketplace_skills,
            add_marketplace_source,
            remove_marketplace_source,
            toggle_marketplace_source,
            // SkillsMP API
            get_skillsmp_api_key,
            get_skillsmp_api_key_masked,
            get_skillsmp_api_key_source,
            set_skillsmp_api_key,
            clear_skillsmp_api_key,
            fetch_skillsmp_skills,
            search_skillsmp_ai,
            // Config
            export_config,
            import_config,
            // Utils
            open_in_explorer,
            get_install_history,
            clear_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
