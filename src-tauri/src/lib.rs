mod commands;
mod database;
mod services;

use database::Database;
use services::ollama::{OllamaConfig, OllamaService};
use std::path::PathBuf;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
            let db_path: PathBuf = app_dir.join("specmaker.db");

            let db = Database::new(db_path).expect("Failed to initialize database");
            app.manage(db);

            let ollama_service = OllamaService::new(OllamaConfig::default());
            app.manage(ollama_service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::get_projects,
            commands::get_project,
            commands::delete_project,
            commands::create_conversation,
            commands::get_conversation_messages,
            commands::send_message,
            commands::check_ollama_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
