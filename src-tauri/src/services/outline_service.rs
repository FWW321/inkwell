use crate::db::models::OutlineNode;
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::{RecordId, ToSql};

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

    let mut q = db
        .query(sql)
        .bind(("pid", RecordId::new("project", project_id)));
    if let Some(pid) = parent_id {
        q = q.bind(("parent", RecordId::new("outline_node", pid)));
    }

    q.await?
        .check()?
        .take::<Vec<OutlineNode>>(0)
        .map_err(Into::into)
}

pub async fn get_node(db: &Db, id: &str) -> AppResult<OutlineNode> {
    db.select(("outline_node", id))
        .await?
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

    let sort_order = next_sort_order(db, project_id, parent_id).await?;
    let has_parent = parent_id.is_some();

    let mut q = db
        .query(
            "CREATE outline_node CONTENT { \
         project: type::record('project', $pid), \
         parent: if $has_parent { type::record('outline_node', $parent) } else { NONE }, \
         node_type: $node_type, \
         title: $title, \
         sort_order: $sort_order, \
         word_count: 0, \
         status: 'draft' \
         }",
        )
        .bind(("pid", project_id.to_string()))
        .bind(("has_parent", has_parent))
        .bind(("node_type", node_type.to_string()))
        .bind(("title", title.to_string()))
        .bind(("sort_order", sort_order));
    if let Some(pid) = parent_id {
        q = q.bind(("parent", pid.to_string()));
    }

    q.await?
        .check()?
        .take::<Option<OutlineNode>>(0)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("create outline_node failed")))
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
        .await?
        .check()?;

    get_node(db, id).await
}

pub async fn rename_node(db: &Db, id: &str, title: &str) -> AppResult<OutlineNode> {
    db.query("UPDATE type::record($id) SET title = $title, updated_at = time::now()")
        .bind(("id", RecordId::new("outline_node", id)))
        .bind(("title", title.to_string()))
        .await?
        .check()?;

    get_node(db, id).await
}

pub async fn delete_node(db: &Db, id: &str) -> AppResult<()> {
    let node: Option<OutlineNode> = db.select(("outline_node", id)).await?;
    if node.is_none() {
        return Err(AppError::NotFound(format!("节点 {} 不存在", id)));
    }

    let mut all_ids = collect_descendant_ids(db, id).await?;
    all_ids.push(id.to_string());

    let items: Vec<RecordId> = all_ids
        .iter()
        .map(|s| RecordId::new("outline_node", s.as_str()))
        .collect();
    db.query("FOR $id IN $ids { DELETE $id }")
        .bind(("ids", items))
        .await?
        .check()?;

    Ok(())
}

async fn collect_descendant_ids(db: &Db, parent_id: &str) -> AppResult<Vec<String>> {
    let mut all_ids = Vec::new();
    let mut stack = vec![parent_id.to_string()];

    while let Some(current_id) = stack.pop() {
        let children = list_children_by_parent(db, &current_id).await?;
        for child in &children {
            let child_id = child.id.key.to_sql();
            all_ids.push(child_id.clone());
            stack.push(child_id);
        }
    }

    Ok(all_ids)
}

pub async fn reorder_nodes(
    db: &Db,
    _project_id: &str,
    parent_id: Option<&str>,
    node_ids: &[String],
) -> AppResult<()> {
    let items: Vec<(String, i64)> = node_ids
        .iter()
        .enumerate()
        .map(|(i, nid)| (nid.clone(), i as i64))
        .collect();
    db.query("FOR $item IN $items { UPDATE type::record('outline_node', $item[0]) SET sort_order = $item[1] }")
        .bind(("items", items))
        .await?
        .check()?;
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
        .await?
        .check()?;
    Ok(())
}

pub async fn clear_diff(db: &Db, id: &str) -> AppResult<()> {
    db.query("UPDATE type::record($id) SET diff_original = NONE, diff_new = NONE, diff_mode = NONE, updated_at = time::now()")
        .bind(("id", RecordId::new("outline_node", id)))
        .await?
        .check()?;
    Ok(())
}

async fn next_sort_order(db: &Db, project_id: &str, parent_id: Option<&str>) -> AppResult<i64> {
    let result: Option<i64> = if let Some(pid) = parent_id {
        db.query("SELECT VALUE sort_order FROM outline_node WHERE parent = $parent ORDER BY sort_order DESC LIMIT 1")
            .bind(("parent", RecordId::new("outline_node", pid)))
            .await?.check()?.take::<Option<i64>>(0)?
    } else {
        db.query("SELECT VALUE sort_order FROM outline_node WHERE project = $pid AND parent IS NONE ORDER BY sort_order DESC LIMIT 1")
            .bind(("pid", RecordId::new("project", project_id)))
            .await?.check()?.take::<Option<i64>>(0)?
    };
    Ok(result.map(|v| v + 1).unwrap_or(0))
}

async fn list_children_by_parent(db: &Db, parent_id: &str) -> AppResult<Vec<OutlineNode>> {
    db.query("SELECT id FROM outline_node WHERE parent = $parent")
        .bind(("parent", RecordId::new("outline_node", parent_id)))
        .await?
        .check()?
        .take::<Vec<OutlineNode>>(0)
        .map_err(Into::into)
}
