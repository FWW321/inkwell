use crate::db::models::WorldviewEntry;
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::RecordId;

pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<WorldviewEntry>> {
    db.query("SELECT * FROM worldview_entry WHERE project = $pid ORDER BY category, title")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?.take::<Vec<WorldviewEntry>>(0).map_err(Into::into)
}

pub async fn create(
    db: &Db,
    project_id: &str,
    category: &str,
    title: &str,
    content: &str,
) -> AppResult<WorldviewEntry> {
    db.query("CREATE worldview_entry CONTENT { project: type::record('project', $pid), category: $category, title: $title, content: $content }")
        .bind(("pid", project_id.to_string()))
        .bind(("category", category.to_string()))
        .bind(("title", title.to_string()))
        .bind(("content", content.to_string()))
        .await?.take::<Option<WorldviewEntry>>(0)?
        .ok_or_else(|| AppError::Internal("create worldview_entry failed".into()))
}

pub async fn update(
    db: &Db,
    id: &str,
    category: &str,
    title: &str,
    content: &str,
) -> AppResult<WorldviewEntry> {
    db.query("UPDATE type::record($id) SET category = $category, title = $title, content = $content, updated_at = time::now()")
        .bind(("id", RecordId::new("worldview_entry", id)))
        .bind(("category", category.to_string()))
        .bind(("title", title.to_string()))
        .bind(("content", content.to_string()))
        .await?;

    db.select(("worldview_entry", id)).await?
        .ok_or_else(|| AppError::NotFound(format!("WorldviewEntry {} not found", id)))
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<WorldviewEntry> = db.delete(("worldview_entry", id)).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound(format!("WorldviewEntry {} not found", id)));
    }
    Ok(())
}
