use crate::db::models::WorldviewEntry;
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, params};
use uuid::Uuid;

pub fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<WorldviewEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, category, title, content, created_at FROM worldview_entries WHERE project_id = ?1 ORDER BY category, title"
    )?;
    let entries = stmt
        .query_map(params![project_id], |row| {
            Ok(WorldviewEntry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                category: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn create(
    conn: &Connection,
    project_id: &str,
    category: &str,
    title: &str,
    content: &str,
) -> AppResult<WorldviewEntry> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO worldview_entries (id, project_id, category, title, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, project_id, category, title, content, now],
    )?;
    conn.query_row(
        "SELECT id, project_id, category, title, content, created_at FROM worldview_entries WHERE id = ?1",
        params![id],
        |row| Ok(WorldviewEntry {
            id: row.get(0)?, project_id: row.get(1)?, category: row.get(2)?,
            title: row.get(3)?, content: row.get(4)?, created_at: row.get(5)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("WorldviewEntry {} not found", id)))
}

pub fn update(
    conn: &Connection,
    id: &str,
    category: &str,
    title: &str,
    content: &str,
) -> AppResult<WorldviewEntry> {
    conn.execute(
        "UPDATE worldview_entries SET category = ?1, title = ?2, content = ?3 WHERE id = ?4",
        params![category, title, content, id],
    )?;
    conn.query_row(
        "SELECT id, project_id, category, title, content, created_at FROM worldview_entries WHERE id = ?1",
        params![id],
        |row| Ok(WorldviewEntry {
            id: row.get(0)?, project_id: row.get(1)?, category: row.get(2)?,
            title: row.get(3)?, content: row.get(4)?, created_at: row.get(5)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("WorldviewEntry {} not found", id)))
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let rows = conn.execute("DELETE FROM worldview_entries WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "WorldviewEntry {} not found",
            id
        )));
    }
    Ok(())
}
