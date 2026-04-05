use crate::db::models::OutlineNode;
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, params};
use uuid::Uuid;

fn row_to_node(row: &rusqlite::Row) -> rusqlite::Result<OutlineNode> {
    Ok(OutlineNode {
        id: row.get(0)?,
        project_id: row.get(1)?,
        parent_id: row.get(2)?,
        node_type: row.get(3)?,
        title: row.get(4)?,
        sort_order: row.get(5)?,
        content_json: row.get(6)?,
        word_count: row.get(7)?,
        status: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

const SELECT: &str = "SELECT id, project_id, parent_id, node_type, title, sort_order, content_json, word_count, status, created_at, updated_at FROM outline_nodes";

pub fn list_children(conn: &Connection, project_id: &str, parent_id: Option<&str>) -> AppResult<Vec<OutlineNode>> {
    let mut stmt = if parent_id.is_some() {
        conn.prepare(&format!("{SELECT} WHERE project_id = ?1 AND parent_id = ?2 ORDER BY sort_order"))?
    } else {
        conn.prepare(&format!("{SELECT} WHERE project_id = ?1 AND parent_id IS NULL ORDER BY sort_order"))?
    };

    let nodes = if parent_id.is_some() {
        stmt.query_map(params![project_id, parent_id], row_to_node)?
    } else {
        stmt.query_map(params![project_id], row_to_node)?
    }
    .collect::<Result<Vec<_>, _>>()?;
    Ok(nodes)
}

pub fn get_node(conn: &Connection, id: &str) -> AppResult<OutlineNode> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), params![id], row_to_node)
        .map_err(|_| AppError::NotFound(format!("节点 {} 不存在", id)))
}

pub fn create_node(
    conn: &Connection,
    project_id: &str,
    parent_id: Option<&str>,
    node_type: &str,
    title: &str,
) -> AppResult<OutlineNode> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    if node_type == "volume" {
        if let Some(pid) = parent_id {
            let parent = get_node(conn, pid)?;
            if parent.node_type == "chapter" {
                return Err(AppError::Validation("章节下不能创建子节点".to_string()));
            }
        }
    }

    let sort_order: i64 = if parent_id.is_some() {
        conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM outline_nodes WHERE parent_id = ?1",
            params![parent_id],
            |row| row.get(0),
        )?
    } else {
        conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM outline_nodes WHERE project_id = ?1 AND parent_id IS NULL",
            params![project_id],
            |row| row.get(0),
        )?
    };

    conn.execute(
        "INSERT INTO outline_nodes (id, project_id, parent_id, node_type, title, sort_order, content_json, word_count, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '[]', 0, 'draft', ?7, ?7)",
        params![id, project_id, parent_id, node_type, title, sort_order, now],
    )?;

    Ok(OutlineNode {
        id,
        project_id: project_id.to_string(),
        parent_id: parent_id.map(|s| s.to_string()),
        node_type: node_type.to_string(),
        title: title.to_string(),
        sort_order,
        content_json: "[]".to_string(),
        word_count: 0,
        status: "draft".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_node(
    conn: &Connection,
    id: &str,
    title: &str,
    content_json: &str,
    word_count: i64,
    status: &str,
) -> AppResult<OutlineNode> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE outline_nodes SET title = ?1, content_json = ?2, word_count = ?3, status = ?4, updated_at = ?5 WHERE id = ?6",
        params![title, content_json, word_count, status, now, id],
    )?;
    get_node(conn, id)
}

pub fn rename_node(conn: &Connection, id: &str, title: &str) -> AppResult<OutlineNode> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE outline_nodes SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )?;
    get_node(conn, id)
}

pub fn delete_node(conn: &Connection, id: &str) -> AppResult<()> {
    let rows = conn.execute("DELETE FROM outline_nodes WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("节点 {} 不存在", id)));
    }
    Ok(())
}

pub fn reorder_nodes(
    conn: &Connection,
    project_id: &str,
    parent_id: Option<&str>,
    node_ids: &[String],
) -> AppResult<()> {
    for (i, nid) in node_ids.iter().enumerate() {
        conn.execute(
            "UPDATE outline_nodes SET sort_order = ?1 WHERE id = ?2 AND project_id = ?3 AND COALESCE(parent_id, '') = COALESCE(?4, '')",
            params![i as i64, nid, project_id, parent_id],
        )?;
    }
    Ok(())
}
