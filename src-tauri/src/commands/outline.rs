use crate::db::models::OutlineNode;
use crate::error::AppResult;
use crate::services::outline_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_outline_nodes(
    state: State<'_, AppState>,
    project_id: String,
    parent_id: Option<String>,
) -> AppResult<Vec<OutlineNode>> {
    outline_service::list_children(&state.db, &project_id, parent_id.as_deref()).await
}

#[tauri::command]
pub async fn get_outline_node(state: State<'_, AppState>, id: String) -> AppResult<OutlineNode> {
    outline_service::get_node(&state.db, &id).await
}

#[tauri::command]
pub async fn create_outline_node(
    state: State<'_, AppState>,
    project_id: String,
    parent_id: Option<String>,
    node_type: String,
    title: String,
) -> AppResult<OutlineNode> {
    outline_service::create_node(&state.db, &project_id, parent_id.as_deref(), &node_type, &title).await
}

#[tauri::command]
pub async fn update_outline_node(
    state: State<'_, AppState>,
    id: String,
    title: String,
    content_json: String,
    word_count: i64,
    status: String,
) -> AppResult<OutlineNode> {
    let content: serde_json::Value = serde_json::from_str(&content_json)
        .map_err(|_| crate::error::AppError::Validation("无效的 content_json".into()))?;
    outline_service::update_node(&state.db, &id, &title, content, word_count, &status).await
}

#[tauri::command]
pub async fn rename_outline_node(
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> AppResult<OutlineNode> {
    outline_service::rename_node(&state.db, &id, &title).await
}

#[tauri::command]
pub async fn delete_outline_node(state: State<'_, AppState>, id: String) -> AppResult<()> {
    outline_service::delete_node(&state.db, &id).await
}

#[tauri::command]
pub async fn reorder_outline_nodes(
    state: State<'_, AppState>,
    project_id: String,
    parent_id: Option<String>,
    node_ids: Vec<String>,
) -> AppResult<()> {
    outline_service::reorder_nodes(&state.db, &project_id, parent_id.as_deref(), &node_ids).await
}

#[tauri::command]
pub async fn save_diff(
    state: State<'_, AppState>,
    id: String,
    original_text: String,
    new_text: String,
    mode: String,
) -> AppResult<()> {
    outline_service::save_diff(&state.db, &id, &original_text, &new_text, &mode).await
}

#[tauri::command]
pub async fn clear_diff(state: State<'_, AppState>, id: String) -> AppResult<()> {
    outline_service::clear_diff(&state.db, &id).await
}
