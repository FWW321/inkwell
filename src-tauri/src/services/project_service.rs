use crate::db::models::Project;
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::RecordId;

pub async fn list(db: &Db) -> AppResult<Vec<Project>> {
    let projects: Vec<Project> = db
        .query("SELECT * FROM project ORDER BY updated_at DESC")
        .await?
        .take::<Vec<Project>>(0)?;
    Ok(projects)
}

pub async fn get(db: &Db, id: &str) -> AppResult<Project> {
    let record: Option<Project> = db.select(("project", id)).await?;
    record.ok_or_else(|| AppError::NotFound(format!("Project {} not found", id)))
}

pub async fn create(
    db: &Db,
    title: &str,
    description: &str,
    author: &str,
    language: &str,
    tags: &str,
    status: &str,
) -> AppResult<Project> {
    let data = serde_json::json!({
        "title": title,
        "description": description,
        "author": author,
        "language": language,
        "tags": tags,
        "status": status,
    });

    let created: Option<Project> = db.create("project").content(data).await?;
    created.ok_or_else(|| AppError::Internal("create project failed".into()))
}

pub async fn update(
    db: &Db,
    id: &str,
    title: &str,
    description: &str,
    author: &str,
    language: &str,
    tags: &str,
    status: &str,
) -> AppResult<Project> {
    db.query("UPDATE type::record($id) SET title = $title, description = $description, author = $author, language = $language, tags = $tags, status = $status, updated_at = time::now()")
        .bind(("id", RecordId::new("project", id)))
        .bind(("title", title.to_string()))
        .bind(("description", description.to_string()))
        .bind(("author", author.to_string()))
        .bind(("language", language.to_string()))
        .bind(("tags", tags.to_string()))
        .bind(("status", status.to_string()))
        .await?;

    get(db, id).await
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<Project> = db.delete(("project", id)).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound(format!("Project {} not found", id)));
    }
    Ok(())
}
