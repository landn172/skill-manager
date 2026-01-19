use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

pub fn get_db_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("skill-manager");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path.push("history.db");
    path
}

pub fn init_db() -> Result<(), String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS installation_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            skill_name TEXT NOT NULL,
            agent_type TEXT NOT NULL,
            scope TEXT NOT NULL,
            version TEXT,
            action TEXT NOT NULL, -- 'install', 'uninstall', 'update'
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // App configuration table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn log_action(
    skill_name: &str,
    agent_type: &str,
    scope: &str,
    version: Option<&str>,
    action: &str,
) -> Result<(), String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO installation_history (skill_name, agent_type, scope, version, action)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![skill_name, agent_type, scope, version, action],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(serde::Serialize)]
pub struct HistoryEntry {
    pub id: i32,
    pub skill_name: String,
    pub agent_type: String,
    pub scope: String,
    pub version: Option<String>,
    pub action: String,
    pub timestamp: String,
}

pub fn get_history() -> Result<Vec<HistoryEntry>, String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, skill_name, agent_type, scope, version, action, timestamp 
                  FROM installation_history ORDER BY timestamp DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map([], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                skill_name: row.get(1)?,
                agent_type: row.get(2)?,
                scope: row.get(3)?,
                version: row.get(4)?,
                action: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

// Config storage functions
pub fn get_config(key: &str) -> Result<Option<String>, String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        params![key],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn set_config(key: &str, value: &str) -> Result<(), String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_config(key: &str) -> Result<(), String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM app_config WHERE key = ?1", params![key])
        .map_err(|e| e.to_string())?;

    Ok(())
}
