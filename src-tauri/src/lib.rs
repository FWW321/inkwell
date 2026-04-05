mod commands;
mod db;
mod error;
mod services;
mod state;

use state::AppState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Inkwell.", name)
}

pub fn run() {
    let app_state = AppState::new().expect("Failed to initialize app state");

    {
        let conn = app_state.db.lock().expect("Failed to acquire db lock");
        db::run_migrations(&conn).expect("Failed to run migrations");
    }

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::project::list_projects,
            commands::project::get_project,
            commands::project::create_project,
            commands::project::update_project,
            commands::project::delete_project,
            commands::outline::list_outline_nodes,
            commands::outline::get_outline_node,
            commands::outline::create_outline_node,
            commands::outline::update_outline_node,
            commands::outline::rename_outline_node,
            commands::outline::delete_outline_node,
            commands::outline::reorder_outline_nodes,
            commands::outline::save_diff,
            commands::outline::clear_diff,
            commands::character::list_characters,
            commands::character::get_character,
            commands::character::create_character,
            commands::character::update_character,
            commands::character::delete_character,
            commands::worldview::list_worldview_entries,
            commands::worldview::create_worldview_entry,
            commands::worldview::update_worldview_entry,
            commands::worldview::delete_worldview_entry,
            commands::ai::list_ai_models,
            commands::ai::create_ai_model,
            commands::ai::update_ai_model,
            commands::ai::delete_ai_model,
            commands::ai::set_default_ai_model,
            commands::ai::list_models,
            commands::ai::ai_continue_writing,
            commands::ai::ai_rewrite,
            commands::ai::ai_polish,
            commands::ai::ai_generate_dialogue,
            commands::ai::ai_chat,
            commands::ai::ai_stream,
            commands::ai::get_chat_history,
            commands::ai::clear_chat_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
