use crate::db::models::WorldviewEntry;
use crate::error::AppResult;
use crate::services::worldview_service;
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
pub fn list_worldview_entries(
    state: State<AppState>,
    project_id: String,
) -> AppResult<Vec<WorldviewEntry>> {
    let conn = get_conn!(state);
    worldview_service::list(&conn, &project_id)
}

#[tauri::command]
pub fn create_worldview_entry(
    state: State<AppState>,
    project_id: String,
    category: String,
    title: String,
    content: String,
) -> AppResult<WorldviewEntry> {
    let conn = get_conn!(state);
    worldview_service::create(&conn, &project_id, &category, &title, &content)
}

#[tauri::command]
pub fn update_worldview_entry(
    state: State<AppState>,
    id: String,
    category: String,
    title: String,
    content: String,
) -> AppResult<WorldviewEntry> {
    let conn = get_conn!(state);
    worldview_service::update(&conn, &id, &category, &title, &content)
}

#[tauri::command]
pub fn delete_worldview_entry(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = get_conn!(state);
    worldview_service::delete(&conn, &id)
}
