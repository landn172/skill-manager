use crate::models::agent::{AgentConfig, AgentType};

#[tauri::command]
pub async fn detect_agents() -> Result<Vec<AgentConfig>, String> {
    let mut agents = AgentConfig::all();
    for agent in &mut agents {
        AgentConfig::detect_installed(agent).await;
    }
    Ok(agents)
}

#[tauri::command]
pub fn get_agent_config(agent_type: AgentType) -> Result<AgentConfig, String> {
    AgentConfig::all()
        .into_iter()
        .find(|a| a.agent_type == agent_type)
        .ok_or_else(|| "Agent not found".into())
}
