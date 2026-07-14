//! Auto-update wiring using tauri-plugin-updater.

use tauri::{AppHandle, Emitter};

/// Simplified update info passed to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
}

/// Check for updates, return info if one exists. Does NOT apply it.
pub async fn check_and_fetch(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = tauri_plugin_updater::UpdaterExt::updater(app).map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            current_version: app.package_info().version.to_string(),
        })),
        Ok(None) => Ok(None),
        Err(e) => {
            eprintln!("Update check failed: {e}");
            Ok(None) // never block the UI on update-check errors
        }
    }
}

/// Check for updates and emit an event to the frontend if one is available.
/// Called on a timer after launch. Non-blocking; errors are swallowed.
pub async fn check_and_notify(app: &AppHandle) -> Result<(), String> {
    if let Some(info) = check_and_fetch(app).await? {
        app.emit("update-available", &info).map_err(|e| e.to_string())?;
    }
    Ok(())
}
