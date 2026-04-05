use crate::db::models::OutlineNode;
use crate::error::AppResult;
use crate::services::outline_service;
use crate::state::AppState;
use tauri::State;

macro_rules! get_conn {
    ($state:expr) => {
        $state
            .db
            .lock()
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
    };
}

#[tauri::command]
pub fn list_outline_nodes(
    state: State<AppState>,
    project_id: String,
    parent_id: Option<String>,
) -> AppResult<Vec<OutlineNode>> {
    let conn = get_conn!(state);
    outline_service::list_children(&conn, &project_id, parent_id.as_deref())
}

#[tauri::command]
pub fn get_outline_node(state: State<AppState>, id: String) -> AppResult<OutlineNode> {
    let conn = get_conn!(state);
    outline_service::get_node(&conn, &id)
}

#[tauri::command]
pub fn create_outline_node(
    state: State<AppState>,
    project_id: String,
    parent_id: Option<String>,
    node_type: String,
    title: String,
) -> AppResult<OutlineNode> {
    let conn = get_conn!(state);
    outline_service::create_node(&conn, &project_id, parent_id.as_deref(), &node_type, &title)
}

#[tauri::command]
pub fn update_outline_node(
    state: State<AppState>,
    id: String,
    title: String,
    content_json: String,
    word_count: i64,
    status: String,
) -> AppResult<OutlineNode> {
    let conn = get_conn!(state);
    outline_service::update_node(&conn, &id, &title, &content_json, word_count, &status)
}

#[tauri::command]
pub fn rename_outline_node(
    state: State<AppState>,
    id: String,
    title: String,
) -> AppResult<OutlineNode> {
    let conn = get_conn!(state);
    outline_service::rename_node(&conn, &id, &title)
}

#[tauri::command]
pub fn delete_outline_node(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = get_conn!(state);
    outline_service::delete_node(&conn, &id)
}

#[tauri::command]
pub fn reorder_outline_nodes(
    state: State<AppState>,
    project_id: String,
    parent_id: Option<String>,
    node_ids: Vec<String>,
) -> AppResult<()> {
    let conn = get_conn!(state);
    outline_service::reorder_nodes(&conn, &project_id, parent_id.as_deref(), &node_ids)
}
