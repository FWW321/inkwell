use crate::db::Store;
use crate::db::models::Project;
use crate::error::{AppError, AppResult};
use crate::state::Db;

pub async fn list(db: &Db) -> AppResult<Vec<Project>> {
    Store::new(db)
        .find("project")
        .order("updated_at DESC")
        .all()
        .await
}

pub async fn get(db: &Db, id: &str) -> AppResult<Project> {
    Store::new(db).get("project", id).await
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
    Store::new(db)
        .content("project")
        .field("title", title)
        .field("description", description)
        .field("author", author)
        .field("language", language)
        .field("tags", tags)
        .field("status", status)
        .exec::<Project>()
        .await
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
    Store::new(db)
        .update("project", id)
        .set("title", title)
        .set("description", description)
        .set("author", author)
        .set("language", language)
        .set("tags", tags)
        .set("status", status)
        .touch()
        .get::<Project>()
        .await
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    Store::new(db).delete("project", id).await
}
