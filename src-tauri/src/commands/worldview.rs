use crate::db::models::WorldviewEntry;
use crate::error::AppResult;
use crate::services::worldview_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_worldview_entries(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<WorldviewEntry>> {
    worldview_service::list(&state.db, &project_id).await
}

#[tauri::command]
pub async fn create_worldview_entry(
    state: State<'_, AppState>,
    project_id: String,
    category: String,
    title: String,
    content: String,
) -> AppResult<WorldviewEntry> {
    worldview_service::create(&state.db, &project_id, &category, &title, &content).await
}

#[tauri::command]
pub async fn update_worldview_entry(
    state: State<'_, AppState>,
    id: String,
    category: String,
    title: String,
    content: String,
) -> AppResult<WorldviewEntry> {
    worldview_service::update(&state.db, &id, &category, &title, &content).await
}

#[tauri::command]
pub async fn delete_worldview_entry(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    worldview_service::delete(&state.db, &id).await
}
