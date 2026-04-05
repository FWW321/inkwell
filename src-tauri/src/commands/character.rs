use crate::db::models::Character;
use crate::error::AppResult;
use crate::services::character_service;
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
pub fn list_characters(state: State<AppState>, project_id: String) -> AppResult<Vec<Character>> {
    let conn = get_conn!(state);
    character_service::list(&conn, &project_id)
}

#[tauri::command]
pub fn get_character(state: State<AppState>, id: String) -> AppResult<Character> {
    let conn = get_conn!(state);
    character_service::get(&conn, &id)
}

#[tauri::command]
pub fn create_character(
    state: State<AppState>,
    project_id: String,
    name: String,
    description: String,
    personality: String,
    background: String,
) -> AppResult<Character> {
    let conn = get_conn!(state);
    character_service::create(
        &conn,
        &project_id,
        &name,
        &description,
        &personality,
        &background,
    )
}

#[tauri::command]
pub fn update_character(
    state: State<AppState>,
    id: String,
    name: String,
    description: String,
    personality: String,
    background: String,
) -> AppResult<Character> {
    let conn = get_conn!(state);
    character_service::update(&conn, &id, &name, &description, &personality, &background)
}

#[tauri::command]
pub fn delete_character(state: State<AppState>, id: String) -> AppResult<()> {
    let conn = get_conn!(state);
    character_service::delete(&conn, &id)
}
