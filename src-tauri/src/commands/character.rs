use crate::db::models::{Character, CharacterWithModelName};
use crate::error::AppResult;
use crate::services::character_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_characters(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<CharacterWithModelName>> {
    character_service::list(state.db(), &project_id).await
}

#[tauri::command]
pub async fn get_character(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<CharacterWithModelName> {
    character_service::get(state.db(), &id).await
}

#[tauri::command]
pub async fn create_character(
    state: State<'_, AppState>,
    project_id: String,
    name: String,
    aliases: serde_json::Value,
    description: String,
    personality: String,
    background: String,
    race: String,
    model_id: Option<String>,
) -> AppResult<Character> {
    character_service::create(
        state.db(),
        &project_id,
        &name,
        aliases,
        &description,
        &personality,
        &background,
        &race,
        model_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn update_character(
    state: State<'_, AppState>,
    id: String,
    name: String,
    aliases: serde_json::Value,
    description: String,
    personality: String,
    background: String,
    race: String,
    model_id: Option<String>,
) -> AppResult<Character> {
    character_service::update(
        state.db(),
        &id,
        &name,
        aliases,
        &description,
        &personality,
        &background,
        &race,
        model_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn delete_character(state: State<'_, AppState>, id: String) -> AppResult<()> {
    character_service::delete(state.db(), &id).await
}
