use tauri::ipc::Channel;

use crate::db::models::{Workflow, WorkflowStep};
use crate::error::AppResult;
use crate::services::workflow_service::{
    self, CreateStepInput, WorkflowProgress, WorkflowResult,
};
use crate::state::AppState;

#[tauri::command]
pub async fn list_workflows(state: State<'_, AppState>) -> AppResult<Vec<Workflow>> {
    workflow_service::list_workflows(state.db()).await
}

#[tauri::command]
pub async fn list_workflow_steps(
    state: State<'_, AppState>,
    workflow_id: String,
) -> AppResult<Vec<WorkflowStep>> {
    workflow_service::list_steps(state.db(), &workflow_id).await
}

#[tauri::command]
pub async fn get_workflow(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Workflow> {
    workflow_service::get_workflow(state.db(), &id).await
}

#[tauri::command]
pub async fn create_workflow(
    state: State<'_, AppState>,
    name: String,
    description: String,
    steps: Vec<CreateStepInput>,
) -> AppResult<Workflow> {
    workflow_service::create_workflow(state.db(), &name, &description, steps).await
}

#[tauri::command]
pub async fn update_workflow(
    state: State<'_, AppState>,
    id: String,
    name: String,
    description: String,
    steps: Vec<CreateStepInput>,
) -> AppResult<Workflow> {
    workflow_service::update_workflow(state.db(), &id, &name, &description, steps).await
}

#[tauri::command]
pub async fn delete_workflow(state: State<'_, AppState>, id: String) -> AppResult<()> {
    workflow_service::delete_workflow(state.db(), &id).await
}

#[tauri::command]
pub async fn set_default_workflow(state: State<'_, AppState>, id: String) -> AppResult<()> {
    workflow_service::set_default_workflow(state.db(), &id).await
}

#[tauri::command]
pub async fn run_workflow(
    state: State<'_, AppState>,
    workflow_id: String,
    project_id: String,
    session_id: Option<String>,
    text: Option<String>,
    instruction: Option<String>,
    character_id: Option<String>,
    on_progress: Channel<WorkflowProgress>,
) -> AppResult<WorkflowResult> {
    let params = workflow_service::WorkflowRunParams {
        project_id,
        session_id,
        text,
        instruction,
        character_id,
    };
    workflow_service::run_workflow(state.db(), state.http(), &workflow_id, params, &on_progress).await
}

use tauri::State;
