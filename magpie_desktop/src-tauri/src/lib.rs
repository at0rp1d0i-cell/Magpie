// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod commands;
pub mod daemon;
pub mod db;
pub mod decision;
pub mod fetchers;
pub mod models;
pub mod queries;

use commands::ChatState;
use std::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(ChatState::new()))
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::chat_send_message,
            queries::get_latest_tickets,
            queries::get_daemon_status
        ])
        .setup(|_app| {
            // Spawn the Magpie backend daemon alongside Tauri UI!
            tauri::async_runtime::spawn(async move {
                daemon::start_background_task().await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
