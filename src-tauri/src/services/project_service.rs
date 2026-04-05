use crate::db::models::Project;
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, params};
use uuid::Uuid;

pub fn list(conn: &Connection) -> AppResult<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, author, language, tags, status, cover_url, created_at, updated_at FROM projects ORDER BY updated_at DESC"
    )?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                author: row.get(3)?,
                language: row.get(4)?,
                tags: row.get(5)?,
                status: row.get(6)?,
                cover_url: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(projects)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Project> {
    conn.query_row(
        "SELECT id, title, description, author, language, tags, status, cover_url, created_at, updated_at FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                author: row.get(3)?,
                language: row.get(4)?,
                tags: row.get(5)?,
                status: row.get(6)?,
                cover_url: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    ).map_err(|_| AppError::NotFound(format!("Project {} not found", id)))
}

pub fn create(conn: &Connection, title: &str, description: &str, author: &str, language: &str, tags: &str, status: &str) -> AppResult<Project> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (id, title, description, author, language, tags, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![id, title, description, author, language, tags, status, now],
    )?;
    get(conn, &id)
}

pub fn update(conn: &Connection, id: &str, title: &str, description: &str, author: &str, language: &str, tags: &str, status: &str) -> AppResult<Project> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET title = ?1, description = ?2, author = ?3, language = ?4, tags = ?5, status = ?6, updated_at = ?7 WHERE id = ?8",
        params![title, description, author, language, tags, status, now, id],
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
