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
/// Only emits once per app session. Called on a timer after launch.
///
/// Deduplication uses an atomic compare-exchange (test-and-set) rather than a
/// load/store pair, so overlapping timer ticks can't both pass the check and
/// emit duplicate events. The flag is only left set when an update is actually
/// found and emitted; on a fetch error, no-update result, or emit failure the
/// slot is released so a later check can still notify.
pub async fn check_and_notify(app: &AppHandle) -> Result<(), String> {
    // Atomically claim the notification slot. If already claimed, another check
    // is in flight or has already emitted.
    if NOTIFIED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(());
    }

    let outcome = async {
        if let Some(info) = check_and_fetch(app).await? {
            app.emit("update-available", &info).map_err(|e| e.to_string())?;
        }
        Ok::<(), String>(())
    }
    .await;

    // Release the slot unless we emitted successfully, so a future tick can
    // still notify (no update yet, transient error, or emit failure).
    if outcome.is_err() {
        NOTIFIED.store(false, Ordering::SeqCst);
    }
    outcome
}
