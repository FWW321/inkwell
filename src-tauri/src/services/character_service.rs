use crate::db::models::Character;
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, params};
use uuid::Uuid;

pub fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<Character>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, avatar_url, description, personality, background, created_at FROM characters WHERE project_id = ?1 ORDER BY name"
    )?;
    let characters = stmt
        .query_map(params![project_id], |row| {
            Ok(Character {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                description: row.get(4)?,
                personality: row.get(5)?,
                background: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(characters)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Character> {
    conn.query_row(
        "SELECT id, project_id, name, avatar_url, description, personality, background, created_at FROM characters WHERE id = ?1",
        params![id],
        |row| Ok(Character {
            id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?,
            avatar_url: row.get(3)?, description: row.get(4)?,
            personality: row.get(5)?, background: row.get(6)?, created_at: row.get(7)?,
        }),
    ).map_err(|_| AppError::NotFound(format!("Character {} not found", id)))
}

pub fn create(
    conn: &Connection,
    project_id: &str,
    name: &str,
    description: &str,
    personality: &str,
    background: &str,
) -> AppResult<Character> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO characters (id, project_id, name, description, personality, background, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, project_id, name, description, personality, background, now],
    )?;
    get(conn, &id)
}

pub fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    description: &str,
    personality: &str,
    background: &str,
) -> AppResult<Character> {
    conn.execute(
        "UPDATE characters SET name = ?1, description = ?2, personality = ?3, background = ?4 WHERE id = ?5",
        params![name, description, personality, background, id],
    )?;
    get(conn, id)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let rows = conn.execute("DELETE FROM characters WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("Character {} not found", id)));
    }
    Ok(())
}
