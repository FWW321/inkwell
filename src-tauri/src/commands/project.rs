use crate::db::models::Project;
use crate::error::AppResult;
use crate::services::project_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> AppResult<Vec<Project>> {
    let conn = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    project_service::list(&conn)
}

#[tauri::command]
pub fn get_project(state: State<AppState>, id: String) -> AppResult<Project> {
    let conn = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    project_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    title: String,
    description: String,
    author: String,
    language: String,
    tags: String,
    status: String,
) -> AppResult<Project> {
    let conn = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    project_service::create(&conn, &title, &description, &author, &language, &tags, &status)
}

#[tauri::command]
pub fn update_project(
    state: State<AppState>,
    id: String,
    title: String,
    description: String,
    author: String,
    language: String,
    tags: String,
    status: String,
) -> AppResult<Project> {
    let conn = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    project_service::update(&conn, &id, &title, &description, &author, &language, &tags, &status)
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = state
        .db
        .lock()
        .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
    project_service::delete(&conn, &id)
}
