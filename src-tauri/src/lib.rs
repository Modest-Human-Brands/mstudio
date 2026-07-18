#[path = "./core/crypto_token.rs"]
mod crypto_token;
#[path = "./core/http_bridge.rs"]
mod http_bridge;
#[path = "./core/overlay.rs"]
mod overlay;
#[path = "./core/stream.rs"]
mod stream;
#[path = "./core/utils.rs"]
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(stream::PreviewState(std::sync::Mutex::new(None)));

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                http_bridge::start_http_server(handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crypto_token::list_certificates,
            crypto_token::sign_hash,
            overlay::add_overlays,
            stream::get_devices,
            stream::start_stream,
            stream::stop_preview,
            utils::list_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
