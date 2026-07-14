//! Tauri commands exposed to the frontend via IPC.
//!
//! These are the bridge between the webview UI and the Rust engine.

use pas_recon_engine::{
    self,
    model::{ReconciliationResult, EngineError},
};

/// Which list to export.
#[derive(Debug, Clone, serde::Deserialize)]
pub enum ListKind {
    EmrNoMatch,
    PasMatchReview,
    PasNoMatch,
}

/// One row received from the frontend for CSV export.
///
/// `pas_recon_engine::model::DisplayRow` only derives `Serialize` (it is an
/// output type), but Tauri commands must `Deserialize` their arguments. We
/// accept a local deserializable mirror here and write it to CSV.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExportRow {
    pub phn: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub dob: Option<String>,
    pub mrp_status: Option<String>,
}

/// Read two CSV files from disk and run reconciliation.
/// Auto-detects the PHN column in each.
#[tauri::command]
pub fn reconcile_files(
    emr_path: String,
    pas_path: String,
) -> Result<ReconciliationResult, EngineError> {
    let emr_bytes = std::fs::read(&emr_path).map_err(|e| EngineError::Io {
        source: "EMR".to_string(),
        message: e.to_string(),
    })?;
    let pas_bytes = std::fs::read(&pas_path).map_err(|e| EngineError::Io {
        source: "PAS".to_string(),
        message: e.to_string(),
    })?;

    pas_recon_engine::reconcile(&emr_bytes, &pas_bytes)
}

/// Reconcile with user-provided PHN column overrides (manual picker fallback).
#[tauri::command]
pub fn reconcile_with_column_override(
    emr_path: String,
    pas_path: String,
    emr_phn_column: Option<usize>,
    pas_phn_column: Option<usize>,
) -> Result<ReconciliationResult, EngineError> {
    let emr_bytes = std::fs::read(&emr_path).map_err(|e| EngineError::Io {
        source: "EMR".to_string(),
        message: e.to_string(),
    })?;
    let pas_bytes = std::fs::read(&pas_path).map_err(|e| EngineError::Io {
        source: "PAS".to_string(),
        message: e.to_string(),
    })?;

    pas_recon_engine::reconcile_with_columns(
        &emr_bytes,
        &pas_bytes,
        emr_phn_column,
        pas_phn_column,
    )
}

/// Export one of the three lists to a CSV file at the given path.
#[tauri::command]
pub fn export_list(
    list: ListKind,
    rows: Vec<ExportRow>,
    path: String,
) -> Result<(), String> {
    let _ = list; // list kind selects which rows the frontend passes in
    let mut wtr = csv::Writer::from_path(&path).map_err(|e| e.to_string())?;
    wtr.write_record(["PHN", "First Name", "Last Name", "DOB", "MRP Status"])
        .map_err(|e| e.to_string())?;
    for row in &rows {
        wtr.write_record([
            row.phn.as_str(),
            row.first_name.as_deref().unwrap_or(""),
            row.last_name.as_deref().unwrap_or(""),
            row.dob.as_deref().unwrap_or(""),
            row.mrp_status.as_deref().unwrap_or(""),
        ])
        .map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Read just the header row of a CSV file. Used by the column-picker fallback
/// when auto-detection fails.
#[tauri::command]
pub fn get_csv_headers(path: String) -> Result<Vec<String>, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let parsed = pas_recon_engine::parse::parse_csv(&bytes).map_err(|e| e.to_string())?;
    Ok(parsed.headers)
}

/// Check GitHub Releases for a newer version. Returns Some(info) if an update exists.
#[tauri::command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
) -> Result<Option<crate::update::UpdateInfo>, String> {
    crate::update::check_and_fetch(&app).await
}
