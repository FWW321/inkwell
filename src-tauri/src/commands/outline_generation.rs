use crate::db::models::OutlineNode;
use crate::error::AppResult;
use crate::services::outline_generation_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn generate_volume_structure(
    state: State<'_, AppState>,
    project_id: String,
    concept: String,
    agent_id: String,
) -> AppResult<Vec<OutlineNode>> {
    outline_generation_service::generate_volume_structure(
        state.db(),
        state.http(),
        &project_id,
        &concept,
        &agent_id,
    )
    .await
}

#[tauri::command]
pub async fn generate_chapter_structure(
    state: State<'_, AppState>,
    volume_id: String,
    agent_id: String,
) -> AppResult<Vec<OutlineNode>> {
    outline_generation_service::generate_chapter_structure(
        state.db(),
        state.http(),
        &volume_id,
        &agent_id,
    )
    .await
}

#[tauri::command]
pub async fn expand_chapter_outline(
    state: State<'_, AppState>,
    chapter_id: String,
    agent_id: String,
) -> AppResult<OutlineNode> {
    outline_generation_service::expand_chapter_outline(
        state.db(),
        state.http(),
        &chapter_id,
        &agent_id,
    )
    .await
}
