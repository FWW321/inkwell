use crate::db::Store;
use crate::db::models::WorldviewEntry;
use crate::error::AppResult;
use crate::state::Db;

pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<WorldviewEntry>> {
    Store::new(db)
        .find("worldview_entry")
        .filter_ref("project", "project", project_id)
        .order("category, title")
        .all()
        .await
}

pub async fn create(
    db: &Db,
    project_id: &str,
    category: &str,
    title: &str,
    content: &str,
) -> AppResult<WorldviewEntry> {
    Store::new(db)
        .content("worldview_entry")
        .ref_id("project", "project", project_id)
        .field("category", category)
        .field("title", title)
        .field("content", content)
        .exec::<WorldviewEntry>()
        .await
}

pub async fn update(
    db: &Db,
    id: &str,
    category: &str,
    title: &str,
    content: &str,
) -> AppResult<WorldviewEntry> {
    Store::new(db)
        .update("worldview_entry", id)
        .set("category", category)
        .set("title", title)
        .set("content", content)
        .touch()
        .get::<WorldviewEntry>()
        .await
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    Store::new(db).delete("worldview_entry", id).await
}
