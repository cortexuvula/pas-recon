#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod update;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::reconcile_files,
            commands::reconcile_with_column_override,
            commands::export_list,
            commands::check_for_updates,
            commands::get_csv_headers,
        ])
        .setup(|app| {
            // Check for updates 3s after launch (non-blocking)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = update::check_and_notify(&handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
