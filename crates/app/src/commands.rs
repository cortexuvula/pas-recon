//! Tauri commands exposed to the frontend via IPC.
//!
//! These are the bridge between the webview UI and the Rust engine.
//! All blocking I/O (file reads, CSV parsing, reconciliation) is offloaded
//! to a blocking thread pool to keep the webview responsive.

use pas_recon_engine::{
    self,
    model::{ReconciliationResult, DisplayRow, EngineError},
};

/// Read two CSV files from disk and run reconciliation.
/// Auto-detects the PHN column in each. Offloaded to a blocking thread.
#[tauri::command]
pub async fn reconcile_files(
    emr_path: String,
    pas_path: String,
) -> Result<ReconciliationResult, EngineError> {
    tauri::async_runtime::spawn_blocking(move || {
        let emr_bytes = std::fs::read(&emr_path).map_err(|e| EngineError::Io {
            source: "EMR".to_string(),
            message: e.to_string(),
        })?;
        let pas_bytes = std::fs::read(&pas_path).map_err(|e| EngineError::Io {
            source: "PAS".to_string(),
            message: e.to_string(),
        })?;

        pas_recon_engine::reconcile(&emr_bytes, &pas_bytes)
    })
    .await
    .map_err(|e| EngineError::Io {
        source: "Internal".to_string(),
        message: format!("background task failed: {e}"),
    })?
}

/// Reconcile with user-provided PHN column overrides (manual picker fallback).
/// Offloaded to a blocking thread.
#[tauri::command]
pub async fn reconcile_with_column_override(
    emr_path: String,
    pas_path: String,
    emr_phn_column: Option<usize>,
    pas_phn_column: Option<usize>,
) -> Result<ReconciliationResult, EngineError> {
    tauri::async_runtime::spawn_blocking(move || {
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
    })
    .await
    .map_err(|e| EngineError::Io {
        source: "Internal".to_string(),
        message: format!("background task failed: {e}"),
    })?
}

/// Export the list to a CSV or PDF file at the given path.
/// Format is determined by the `format` param ("csv" or "pdf").
/// PDF is generated as a print-ready HTML file that opens in the browser.
/// Offloaded to a blocking thread.
#[tauri::command]
pub async fn export_list(
    rows: Vec<DisplayRow>,
    path: String,
    format: String,
    title: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        match format.as_str() {
            "pdf" => export_html(&rows, &path, &title),
            _ => export_csv(&rows, &path),
        }
    })
    .await
    .map_err(|e| format!("background task failed: {e}"))?
}

/// Write rows to a CSV file.
fn export_csv(rows: &[DisplayRow], path: &str) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
    wtr.write_record(["PHN", "First Name", "Last Name", "DOB", "MRP Status"])
        .map_err(|e| e.to_string())?;
    for row in rows {
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

/// Generate a print-ready HTML file from rows. The user opens it in a browser
/// and uses "Print to PDF" — this avoids bundling font files for a Rust PDF
/// library and works cross-platform with the system's native print dialog.
fn export_html(rows: &[DisplayRow], path: &str, title: &str) -> Result<(), String> {
    let date = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Simple date formatting from epoch seconds (YYYY-MM-DD approximation)
        let days = secs / 86400;
        let year = 1970 + (days / 365);
        let day_of_year = days % 365;
        let month = ((day_of_year / 30) as u8).max(1).min(12);
        let day = ((day_of_year % 30) as u8).max(1);
        format!("{year}-{month:02}-{day:02}")
    };

    let mut body = String::with_capacity(8192 + rows.len() * 200);
    body.push_str(&format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>{title}</title>
<style>
@page {{ size: landscape; margin: 1.5cm; }}
body {{ font-family: -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; color: #1a1a2e; }}
h1 {{ font-size: 18pt; text-align: center; margin-bottom: 4px; }}
.subtitle {{ font-size: 10pt; text-align: center; color: #6b7280; margin-bottom: 20px; }}
table {{ width: 100%; border-collapse: collapse; }}
th {{ background: #2a2a3a; color: white; padding: 8px 10px; text-align: left; font-size: 10pt; }}
td {{ padding: 6px 10px; border-bottom: 1px solid #e5e7eb; font-size: 10pt; }}
tr:nth-child(even) td {{ background: #f9fafb; }}
.btn-print {{ display: block; margin: 20px auto; padding: 10px 24px; font-size: 13pt; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer; }}
.btn-print:hover {{ background: #2563eb; }}
@media print {{ .btn-print {{ display: none; }} }}
</style>
</head>
<body>
<h1>{title}</h1>
<p class="subtitle">{count} patients &mdash; generated {date}</p>
<button class="btn-print" onclick="window.print()">Print / Save as PDF</button>
<table>
<thead><tr><th>PHN</th><th>First Name</th><th>Last Name</th><th>DOB</th><th>MRP Status</th></tr></thead>
<tbody>
"#,
        title = html_escape(title),
        count = rows.len(),
        date = date,
    ));

    for row in rows {
        body.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            html_escape(&row.phn),
            html_escape(row.first_name.as_deref().unwrap_or("")),
            html_escape(row.last_name.as_deref().unwrap_or("")),
            html_escape(row.dob.as_deref().unwrap_or("")),
            html_escape(row.mrp_status.as_deref().unwrap_or("")),
        ));
    }

    body.push_str("</tbody></table>\n</body>\n</html>\n");

    std::fs::write(path, body).map_err(|e| format!("Failed to write file: {e}"))?;

    // Open the file in the default browser using the OS's native command
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", path]).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();

    Ok(())
}

/// Escape HTML special characters to prevent injection in the generated HTML.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Read just the header row of a CSV file. Used by the column-picker fallback
/// when auto-detection fails. Offloaded to a blocking thread.
#[tauri::command]
pub async fn get_csv_headers(path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let parsed = pas_recon_engine::parse::parse_csv(&bytes).map_err(|e| e.to_string())?;
        Ok(parsed.headers)
    })
    .await
    .map_err(|e| format!("background task failed: {e}"))?
}

/// Check GitHub Releases for a newer version. Returns Some(info) if an update exists.
#[tauri::command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
) -> Result<Option<crate::update::UpdateInfo>, String> {
    crate::update::check_and_fetch(&app).await
}
