#[path = "./core/http_bridge.rs"]
mod http_bridge;
#[path = "./core/overlay.rs"]
mod overlay;
#[path = "./core/security.rs"]
mod security;
#[path = "./core/utils.rs"]
mod utils; // <-- Add module

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // Spawn the Axum server in the background without blocking the UI
            tauri::async_runtime::spawn(async {
                http_bridge::start_http_server().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            utils::list_files,
            overlay::add_overlays,
            security::list_certificates,
            security::sign_hash
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
