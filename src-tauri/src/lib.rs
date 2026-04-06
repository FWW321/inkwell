mod commands;
mod db;
mod error;
mod serde_helpers;
mod services;
mod state;

use state::AppState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Inkwell.", name)
}

pub fn run() {
    tauri::async_runtime::block_on(async {
        let app_state = AppState::new()
            .await
            .expect("Failed to initialize app state");

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
                commands::ai::list_ai_agents,
                commands::ai::create_ai_agent,
                commands::ai::update_ai_agent,
                commands::ai::delete_ai_agent,
                commands::ai::set_default_ai_agent,
                commands::ai::list_models,
                commands::ai::ai_continue_writing,
                commands::ai::ai_rewrite,
                commands::ai::ai_polish,
                commands::ai::ai_generate_dialogue,
                commands::ai::ai_chat,
                commands::ai::ai_stream,
                commands::ai::get_chat_history,
                commands::ai::clear_chat_history,
                commands::narrative::list_narrative_sessions,
                commands::narrative::get_narrative_session,
                commands::narrative::create_narrative_session,
                commands::narrative::delete_narrative_session,
                commands::narrative::list_narrative_beats,
                commands::narrative::list_narrative_events,
                commands::narrative::delete_narrative_beat,
                commands::narrative::add_narrative_beat,
                commands::narrative::advance_narration,
                commands::narrative::invoke_narrative_character,
                commands::relation::list_character_relations,
                commands::relation::create_character_relation,
                commands::relation::update_character_relation,
                commands::relation::delete_character_relation,
                commands::relation::list_character_factions,
                commands::relation::create_character_faction,
                commands::relation::update_character_faction,
                commands::relation::delete_character_faction,
                commands::review::review_beat,
                commands::review::list_writing_reviews,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
