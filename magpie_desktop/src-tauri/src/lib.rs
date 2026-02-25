// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod commands;
pub mod daemon;
pub mod db;
pub mod decision;
pub mod fetchers;
pub mod models;
pub mod queries;

use commands::ChatState;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let fetch_trigger = Arc::new(Notify::new());
    let fetch_trigger_for_daemon = fetch_trigger.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(ChatState::new()))
        .manage(fetch_trigger)
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::chat_send_message,
            commands::get_app_config,
            commands::save_app_config,
            commands::trigger_fetch_cycle,
            queries::get_latest_tickets,
            queries::get_daemon_status
        ])
        .setup(|_app| {
            // Spawn the Magpie backend daemon alongside Tauri UI!
            tauri::async_runtime::spawn(async move {
                daemon::start_background_task(fetch_trigger_for_daemon).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
