use crate::db::models::Project;
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, params};
use uuid::Uuid;

pub fn list(conn: &Connection) -> AppResult<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, cover_url, created_at, updated_at FROM projects ORDER BY updated_at DESC"
    )?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cover_url: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(projects)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Project> {
    conn.query_row(
        "SELECT id, title, description, cover_url, created_at, updated_at FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cover_url: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    ).map_err(|_| AppError::NotFound(format!("Project {} not found", id)))
}

pub fn create(conn: &Connection, title: &str, description: &str) -> AppResult<Project> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (id, title, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, title, description, now],
    )?;
    get(conn, &id)
}

pub fn update(conn: &Connection, id: &str, title: &str, description: &str) -> AppResult<Project> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET title = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, description, now, id],
    )?;
    get(conn, id)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let rows = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Project {} not found", id)));
    }
    Ok(())
}
