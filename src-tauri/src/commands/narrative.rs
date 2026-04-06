use crate::db::models::{NarrativeBeat, NarrativeSession};
use crate::error::AppResult;
use crate::services::narrative_service;
use crate::state::AppState;
use tauri::ipc::Channel;
use tauri::State;

use crate::services::narrative_service::{BeatInput, NarrativeStreamChunk};

#[tauri::command]
pub async fn list_narrative_sessions(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<NarrativeSession>> {
    narrative_service::list_sessions(&state.db, &project_id).await
}

#[tauri::command]
pub async fn get_narrative_session(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<NarrativeSession> {
    narrative_service::get_session(&state.db, &id).await
}

#[tauri::command]
pub async fn create_narrative_session(
    state: State<'_, AppState>,
    project_id: String,
    title: String,
    scene: String,
) -> AppResult<NarrativeSession> {
    narrative_service::create_session(&state.db, &project_id, &title, &scene).await
}

#[tauri::command]
pub async fn delete_narrative_session(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    narrative_service::delete_session(&state.db, &id).await
}

#[tauri::command]
pub async fn list_narrative_beats(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<Vec<NarrativeBeat>> {
    narrative_service::list_beats(&state.db, &session_id).await
}

#[tauri::command]
pub async fn delete_narrative_beat(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    narrative_service::delete_beat(&state.db, &id).await
}

#[tauri::command]
pub async fn add_narrative_beat(
    state: State<'_, AppState>,
    session_id: String,
    beat_type: String,
    input: BeatInput,
) -> AppResult<NarrativeBeat> {
    narrative_service::add_beat(&state.db, &session_id, &beat_type, input).await
}

#[tauri::command]
pub async fn advance_narration(
    state: State<'_, AppState>,
    session_id: String,
    instruction: Option<String>,
    on_chunk: Channel<NarrativeStreamChunk>,
) -> AppResult<()> {
    narrative_service::advance_narration(&state.db, &session_id, instruction.as_deref(), &on_chunk).await
}

#[tauri::command]
pub async fn invoke_narrative_character(
    state: State<'_, AppState>,
    session_id: String,
    character_id: String,
    instruction: Option<String>,
    on_chunk: Channel<NarrativeStreamChunk>,
) -> AppResult<()> {
    narrative_service::invoke_character(&state.db, &session_id, &character_id, instruction.as_deref(), &on_chunk).await
}
