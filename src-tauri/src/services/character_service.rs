use crate::db::models::{Character, CharacterWithModelName};
use crate::error::{AppError, AppResult};
use crate::state::Db;
use surrealdb::types::RecordId;

pub async fn list(db: &Db, project_id: &str) -> AppResult<Vec<CharacterWithModelName>> {
    db.query("SELECT *, model.name AS model_name FROM character WHERE project = $pid ORDER BY name")
        .bind(("pid", RecordId::new("project", project_id)))
        .await?.take::<Vec<CharacterWithModelName>>(0).map_err(Into::into)
}

pub async fn get(db: &Db, id: &str) -> AppResult<CharacterWithModelName> {
    db.query("SELECT *, model.name AS model_name FROM character WHERE id = $id")
        .bind(("id", RecordId::new("character", id)))
        .await?.take::<Option<CharacterWithModelName>>(0)?
        .ok_or_else(|| AppError::NotFound(format!("Character {} not found", id)))
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
    let has_model = model_id.is_some();

    let mut q = db.query(
        "CREATE character CONTENT { \
         project: type::record('project', $pid), \
         name: $name, \
         aliases: $aliases, \
         description: $description, \
         personality: $personality, \
         background: $background, \
         race: $race, \
         model: if $has_model { type::record('ai_model', $mid) } else { NONE } \
         }"
    )
    .bind(("pid", project_id.to_string()))
    .bind(("name", name.to_string()))
    .bind(("aliases", aliases))
    .bind(("description", description.to_string()))
    .bind(("personality", personality.to_string()))
    .bind(("background", background.to_string()))
    .bind(("race", race.to_string()))
    .bind(("has_model", has_model));
    if let Some(mid) = model_id {
        q = q.bind(("mid", mid.to_string()));
    }

    q.await?.take::<Option<Character>>(0)?
        .ok_or_else(|| AppError::Internal("create character failed".into()))
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
    let has_model = model_id.is_some();

    let mut q = db.query(
        "UPDATE type::record($id) MERGE { \
         name: $name, \
         aliases: $aliases, \
         description: $description, \
         personality: $personality, \
         background: $background, \
         race: $race, \
         model: if $has_model { type::record('ai_model', $mid) } else { NONE } \
         }"
    )
    .bind(("id", RecordId::new("character", id)))
    .bind(("name", name.to_string()))
    .bind(("aliases", aliases))
    .bind(("description", description.to_string()))
    .bind(("personality", personality.to_string()))
    .bind(("background", background.to_string()))
    .bind(("race", race.to_string()))
    .bind(("has_model", has_model));
    if let Some(mid) = model_id {
        q = q.bind(("mid", mid.to_string()));
    }

    q.await?.take::<Option<Character>>(0)?
        .ok_or_else(|| AppError::NotFound(format!("Character {} not found", id)))
}

pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let deleted: Option<Character> = db.delete(("character", id)).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound(format!("Character {} not found", id)));
    }
    Ok(())
}
