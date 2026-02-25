// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod daemon;
pub mod db;
pub mod decision;
pub mod fetchers;
pub mod models;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
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
