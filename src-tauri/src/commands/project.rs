use crate::db::models::Project;
use crate::error::AppResult;
use crate::services::project_service;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> AppResult<Vec<Project>> {
    project_service::list(&state.db).await
}

#[tauri::command]
pub async fn get_project(state: State<'_, AppState>, id: String) -> AppResult<Project> {
    project_service::get(&state.db, &id).await
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    title: String,
    description: String,
    author: String,
    language: String,
    tags: String,
    status: String,
) -> AppResult<Project> {
    project_service::create(&state.db, &title, &description, &author, &language, &tags, &status).await
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    id: String,
    title: String,
    description: String,
    author: String,
    language: String,
    tags: String,
    status: String,
) -> AppResult<Project> {
    project_service::update(&state.db, &id, &title, &description, &author, &language, &tags, &status).await
}

#[tauri::command]
pub async fn delete_project(state: State<'_, AppState>, id: String) -> AppResult<()> {
    project_service::delete(&state.db, &id).await
}
