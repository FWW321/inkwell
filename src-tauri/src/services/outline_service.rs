use crate::db::models::OutlineNode;
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::RecordId;

pub async fn list_children(
    db: &Db,
    project_id: &str,
    parent_id: Option<&str>,
) -> AppResult<Vec<OutlineNode>> {
    let sql = if parent_id.is_some() {
        "SELECT * FROM outline_node WHERE project = $pid AND parent = $parent ORDER BY sort_order ASC"
    } else {
        "SELECT * FROM outline_node WHERE project = $pid AND parent IS NONE ORDER BY sort_order ASC"
    };

    let mut q = db.query(sql).bind(("pid", RecordId::new("project", project_id)));
    if let Some(pid) = parent_id {
        q = q.bind(("parent", RecordId::new("outline_node", pid)));
    }

    q.await?.check()?.take::<Vec<OutlineNode>>(0).map_err(Into::into)
}

pub async fn get_node(db: &Db, id: &str) -> AppResult<OutlineNode> {
    db.select(("outline_node", id)).await?
        .ok_or_else(|| AppError::NotFound(format!("节点 {} 不存在", id)))
}

pub async fn create_node(
    db: &Db,
    project_id: &str,
    parent_id: Option<&str>,
    node_type: &str,
    title: &str,
) -> AppResult<OutlineNode> {
    if node_type == "volume" {
        if let Some(pid) = parent_id {
            let parent = get_node(db, pid).await?;
            if parent.node_type == "chapter" {
                return Err(AppError::Validation("章节下不能创建子节点".to_string()));
            }
        }
    }

    let sort_order: i64 = if let Some(pid) = parent_id {
        let result: Option<i64> = db.query("SELECT VALUE sort_order FROM outline_node WHERE parent = $parent ORDER BY sort_order DESC LIMIT 1")
            .bind(("parent", RecordId::new("outline_node", pid)))
            .await?.check()?.take::<Option<i64>>(0)?;
        result.map(|v| v + 1).unwrap_or(0)
    } else {
        let result: Option<i64> = db.query("SELECT VALUE sort_order FROM outline_node WHERE project = $pid AND parent IS NONE ORDER BY sort_order DESC LIMIT 1")
            .bind(("pid", RecordId::new("project", project_id)))
            .await?.check()?.take::<Option<i64>>(0)?;
        result.map(|v| v + 1).unwrap_or(0)
    };

    let has_parent = parent_id.is_some();

    let mut q = db.query(
        "CREATE outline_node CONTENT { \
         project: type::record('project', $pid), \
         parent: if $has_parent { type::record('outline_node', $parent) } else { NONE }, \
         node_type: $node_type, \
         title: $title, \
         sort_order: $sort_order, \
         word_count: 0, \
         status: 'draft' \
         }"
    )
    .bind(("pid", project_id.to_string()))
    .bind(("has_parent", has_parent))
    .bind(("node_type", node_type.to_string()))
    .bind(("title", title.to_string()))
    .bind(("sort_order", sort_order));
    if let Some(pid) = parent_id {
        q = q.bind(("parent", pid.to_string()));
    }

    q.await?.check()?.take::<Option<OutlineNode>>(0)?
        .ok_or_else(|| AppError::Internal("create outline_node failed".into()))
}

pub async fn update_node(
    db: &Db,
    id: &str,
    title: &str,
    content_json: serde_json::Value,
    word_count: i64,
    status: &str,
) -> AppResult<OutlineNode> {
    db.query("UPDATE type::record($id) SET title = $title, content_json = $content_json, word_count = $word_count, status = $status, updated_at = time::now()")
        .bind(("id", RecordId::new("outline_node", id)))
        .bind(("title", title.to_string()))
        .bind(("content_json", content_json))
        .bind(("word_count", word_count))
        .bind(("status", status.to_string()))
        .await?;

    get_node(db, id).await
}

pub async fn rename_node(db: &Db, id: &str, title: &str) -> AppResult<OutlineNode> {
    db.query("UPDATE type::record($id) SET title = $title, updated_at = time::now()")
        .bind(("id", RecordId::new("outline_node", id)))
        .bind(("title", title.to_string()))
        .await?;

    get_node(db, id).await
}

pub async fn delete_node(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<OutlineNode> = db.delete(("outline_node", id)).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound(format!("节点 {} 不存在", id)));
    }
    Ok(())
}

pub async fn reorder_nodes(
    db: &Db,
    _project_id: &str,
    _parent_id: Option<&str>,
    node_ids: &[String],
) -> AppResult<()> {
    let items: Vec<(String, i64)> = node_ids
        .iter()
        .enumerate()
        .map(|(i, nid)| (nid.clone(), i as i64))
        .collect();
    db.query("FOR $item IN $items { UPDATE type::record('outline_node', $item[0]) SET sort_order = $item[1] }")
        .bind(("items", items))
        .await?;
    Ok(())
}

pub async fn save_diff(
    db: &Db,
    id: &str,
    original_text: &str,
    new_text: &str,
    mode: &str,
) -> AppResult<()> {
    db.query("UPDATE type::record($id) SET diff_original = $orig, diff_new = $new, diff_mode = $mode, updated_at = time::now()")
        .bind(("id", RecordId::new("outline_node", id)))
        .bind(("orig", original_text.to_string()))
        .bind(("new", new_text.to_string()))
        .bind(("mode", mode.to_string()))
        .await?;
    Ok(())
}

pub async fn clear_diff(db: &Db, id: &str) -> AppResult<()> {
    db.query("UPDATE type::record($id) SET diff_original = NONE, diff_new = NONE, diff_mode = NONE, updated_at = time::now()")
        .bind(("id", RecordId::new("outline_node", id)))
        .await?;
    Ok(())
}
