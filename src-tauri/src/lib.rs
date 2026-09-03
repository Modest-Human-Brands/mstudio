mod crypto_token;
mod http_bridge;
// mod iroh_core;
mod overlay;
mod stream;
// mod sync;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
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
            // sync::seed_path,
            // sync::download_ticket,
            utils::list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
