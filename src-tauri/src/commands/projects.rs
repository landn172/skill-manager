use crate::models::project::Project;
use crate::utils::db::get_db_path;
use rusqlite::{params, Connection};
use std::path::Path;

#[tauri::command]
pub async fn list_projects() -> Result<Vec<Project>, String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, path, created_at FROM projects ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let project_iter = stmt
        .query_map([], |row| {
            Ok(Project {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                created_at: Some(row.get(3)?),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut projects = Vec::new();
    for project in project_iter {
        projects.push(project.map_err(|e| e.to_string())?);
    }

    Ok(projects)
}

#[tauri::command]
pub async fn add_project(name: String, path: String) -> Result<Project, String> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // Check if path exists
    if !Path::new(&path).exists() {
        return Err("Project path does not exist".into());
    }

    conn.execute(
        "INSERT OR REPLACE INTO projects (name, path) VALUES (?1, ?2)",
        params![name, path],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let project = Project {
        id: Some(id),
        name,
        path,
        created_at: None, // Will be fetched next time
    };

    Ok(project)
}

#[tauri::command]
pub async fn remove_project(id: i64) -> Result<(), String> {
    let path = get_db_path();
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
