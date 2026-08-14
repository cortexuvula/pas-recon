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
/// Only emits once per app session. Called after launch.
///
/// Deduplication uses an atomic compare-exchange (test-and-set) rather than a
/// load/store pair, so overlapping checks can't both pass and emit duplicate
/// events. The flag is only left set when an update was actually emitted; on
/// a fetch error, a no-update result, or an emit failure the slot is released
/// so a later check can still notify.
pub async fn check_and_notify(app: &AppHandle) -> Result<(), String> {
    // Atomically claim the notification slot. If already claimed, another check
    // is in flight or has already emitted.
    if NOTIFIED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(());
    }

    let mut emitted = false;
    let outcome = match check_and_fetch(app).await {
        Err(e) => Err(e),
        Ok(Some(info)) => match app.emit("update-available", &info) {
            Ok(()) => {
                emitted = true;
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
        Ok(None) => Ok(()),
    };

    // Release the slot unless we actually emitted, so a later check can still
    // discover an update that appears mid-session.
    if !emitted {
        NOTIFIED.store(false, Ordering::SeqCst);
    }
    outcome
}
