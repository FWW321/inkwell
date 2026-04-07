use crate::db::Store;
use crate::db::models::OutlineNode;
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::{RecordId, ToSql};

pub async fn list_children(
    db: &Db,
    project_id: &str,
    parent_id: Option<&str>,
) -> AppResult<Vec<OutlineNode>> {
    let store = Store::new(db);
    match parent_id {
        Some(pid) => {
            store
                .find("outline_node")
                .filter_ref("project", "project", project_id)
                .filter_ref("parent", "outline_node", pid)
                .order("sort_order ASC")
                .all()
                .await
        }
        None => {
            store
                .find("outline_node")
                .filter_ref("project", "project", project_id)
                .filter_is_none("parent")
                .order("sort_order ASC")
                .all()
                .await
        }
    }
}

pub async fn get_node(db: &Db, id: &str) -> AppResult<OutlineNode> {
    Store::new(db).get("outline_node", id).await
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

    Store::new(db)
        .content("outline_node")
        .ref_id("project", "project", project_id)
        .opt_ref("parent", "outline_node", parent_id)
        .field("node_type", node_type)
        .field("title", title)
        .field("sort_order", sort_order)
        .field("word_count", 0_i64)
        .field("status", "draft")
        .exec::<OutlineNode>()
        .await
}

pub async fn update_node(
    db: &Db,
    id: &str,
    title: &str,
    content_json: serde_json::Value,
    word_count: i64,
    status: &str,
) -> AppResult<OutlineNode> {
    Store::new(db)
        .update("outline_node", id)
        .set("title", title)
        .set("content_json", content_json)
        .set("word_count", word_count)
        .set("status", status)
        .touch()
        .get::<OutlineNode>()
        .await
}

pub async fn rename_node(db: &Db, id: &str, title: &str) -> AppResult<OutlineNode> {
    Store::new(db)
        .update("outline_node", id)
        .set("title", title)
        .touch()
        .get::<OutlineNode>()
        .await
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
    Store::new(db)
        .update("outline_node", id)
        .set("diff_original", original_text)
        .set("diff_new", new_text)
        .set("diff_mode", mode)
        .touch()
        .exec()
        .await
}

pub async fn clear_diff(db: &Db, id: &str) -> AppResult<()> {
    Store::new(db)
        .update("outline_node", id)
        .set_none("diff_original")
        .set_none("diff_new")
        .set_none("diff_mode")
        .touch()
        .exec()
        .await
}

async fn next_sort_order(db: &Db, project_id: &str, parent_id: Option<&str>) -> AppResult<i64> {
    let result: Option<i64> = if let Some(pid) = parent_id {
        Store::new(db)
            .find("outline_node")
            .project("VALUE sort_order")
            .filter_ref("parent", "outline_node", pid)
            .order("sort_order DESC")
            .limit(1)
            .one()
            .await
            .ok()
    } else {
        Store::new(db)
            .find("outline_node")
            .project("VALUE sort_order")
            .filter_ref("project", "project", project_id)
            .filter_is_none("parent")
            .order("sort_order DESC")
            .limit(1)
            .one()
            .await
            .ok()
    };
    Ok(result.map(|v| v + 1).unwrap_or(0))
}

async fn list_children_by_parent(db: &Db, parent_id: &str) -> AppResult<Vec<OutlineNode>> {
    Store::new(db)
        .find("outline_node")
        .project("id")
        .filter_ref("parent", "outline_node", parent_id)
        .all()
        .await
}
