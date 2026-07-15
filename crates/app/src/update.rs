//! Auto-update wiring using tauri-plugin-updater.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

/// Simplified update info passed to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
}

/// Guards against re-emitting the update-available event on every check.
static NOTIFIED: AtomicBool = AtomicBool::new(false);

/// Check for updates, return info if one exists. Does NOT apply it.
/// Update-check errors are logged and swallowed — never block the UI.
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
            Ok(None)
        }
    }
}

/// Check for updates and emit an event to the frontend if one is available.
/// Only emits once per app session (deduplication via atomic flag).
/// Called on a timer after launch.
pub async fn check_and_notify(app: &AppHandle) -> Result<(), String> {
    if NOTIFIED.load(Ordering::Relaxed) {
        return Ok(());
    }
    if let Some(info) = check_and_fetch(app).await? {
        NOTIFIED.store(true, Ordering::Relaxed);
        app.emit("update-available", &info).map_err(|e| e.to_string())?;
    }
    Ok(())
}
