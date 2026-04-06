use crate::db::models::{CharacterFaction, CharacterRelation};
use crate::error::AppResult;
use crate::services::relation_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_character_relations(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<CharacterRelation>> {
    relation_service::list_relations(&state.db, &project_id).await
}

#[tauri::command]
pub async fn create_character_relation(
    state: State<'_, AppState>,
    project_id: String,
    char_a_id: String,
    char_b_id: String,
    relationship_type: String,
    description: String,
    start_chapter_id: Option<String>,
    end_chapter_id: Option<String>,
) -> AppResult<CharacterRelation> {
    relation_service::create_relation(
        &state.db,
        &project_id,
        &char_a_id,
        &char_b_id,
        &relationship_type,
        &description,
        start_chapter_id.as_deref(),
        end_chapter_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn update_character_relation(
    state: State<'_, AppState>,
    id: String,
    relationship_type: String,
    description: String,
    start_chapter_id: Option<String>,
    end_chapter_id: Option<String>,
) -> AppResult<CharacterRelation> {
    relation_service::update_relation(
        &state.db,
        &id,
        &relationship_type,
        &description,
        start_chapter_id.as_deref(),
        end_chapter_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn delete_character_relation(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    relation_service::delete_relation(&state.db, &id).await
}

#[tauri::command]
pub async fn list_character_factions(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<CharacterFaction>> {
    relation_service::list_factions(&state.db, &project_id).await
}

#[tauri::command]
pub async fn create_character_faction(
    state: State<'_, AppState>,
    project_id: String,
    character_id: String,
    faction: String,
    role: String,
    start_chapter_id: Option<String>,
    end_chapter_id: Option<String>,
) -> AppResult<CharacterFaction> {
    relation_service::create_faction(
        &state.db,
        &project_id,
        &character_id,
        &faction,
        &role,
        start_chapter_id.as_deref(),
        end_chapter_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn update_character_faction(
    state: State<'_, AppState>,
    id: String,
    faction: String,
    role: String,
    start_chapter_id: Option<String>,
    end_chapter_id: Option<String>,
) -> AppResult<CharacterFaction> {
    relation_service::update_faction(
        &state.db,
        &id,
        &faction,
        &role,
        start_chapter_id.as_deref(),
        end_chapter_id.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn delete_character_faction(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    relation_service::delete_faction(&state.db, &id).await
}
