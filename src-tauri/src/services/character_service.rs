use crate::db::Store;
use crate::db::models::{Character, CharacterWithModelName};
use crate::error::AppResult;
use crate::state::Db;

pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<CharacterWithModelName>> {
    Store::new(db)
        .find("character")
        .project("*, model.name AS model_name")
        .filter_ref("project", "project", project_id)
        .order("name")
        .all()
        .await
}

pub async fn get(db: &Db, id: &str) -> AppResult<CharacterWithModelName> {
    Store::new(db)
        .find("character")
        .project("*, model.name AS model_name")
        .filter_ref("id", "character", id)
        .one()
        .await
}

pub async fn create(
    db: &Db,
    project_id: &str,
    name: &str,
    aliases: serde_json::Value,
    description: &str,
    personality: &str,
    background: &str,
    race: &str,
    model_id: Option<&str>,
) -> AppResult<Character> {
    Store::new(db)
        .content("character")
        .ref_id("project", "project", project_id)
        .field("name", name)
        .field("aliases", aliases)
        .field("description", description)
        .field("personality", personality)
        .field("background", background)
        .field("race", race)
        .opt_ref("model", "ai_model", model_id)
        .exec::<Character>()
        .await
}

pub async fn update(
    db: &Db,
    id: &str,
    name: &str,
    aliases: serde_json::Value,
    description: &str,
    personality: &str,
    background: &str,
    race: &str,
    model_id: Option<&str>,
) -> AppResult<Character> {
    Store::new(db)
        .content("character")
        .merge_mode(id)
        .field("name", name)
        .field("aliases", aliases)
        .field("description", description)
        .field("personality", personality)
        .field("background", background)
        .field("race", race)
        .opt_ref("model", "ai_model", model_id)
        .exec::<Character>()
        .await
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    Store::new(db).delete("character", id).await
}
