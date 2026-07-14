# PAS Reconciliation App — Design Spec

**Date:** 2026-07-14
**Status:** Approved design, pending implementation plan
**Source:** Replaces the `PAS Rec with EMR (Excel LibreOffice Calc) TEMPLATE.xlsx` spreadsheet (formula reference: `docs/spreadsheet-formulas.md`).

## 1. Purpose

A cross-platform desktop application (Windows, macOS, Linux) that reconciles a clinic's EMR patient panel against the Provincial Attachment System (PAS) patient list. The user drops two CSV files into the app; the app matches patients by PHN and presents three review lists plus a summary. The app ships with a built-in auto-updater backed by GitHub Releases.

This replaces the existing Excel/LibreOffice spreadsheet with a purpose-built tool for non-technical clinic staff: no column-picking, no formula recalculation, no spreadsheet literacy required.

## 2. Requirements (confirmed during brainstorming)

| # | Requirement | Decision |
|---|---|---|
| R1 | Column detection | **Auto-detect** by header patterns, with manual column-picker fallback |
| R2 | PHN validation | **BC PHN**: exactly 10 digits, first digit `9`. Invalid rows skipped (counted, not surfaced in lists) |
| R3 | PAS dedup | **Keep newest** by `MRP Updated` date, drop duplicates **silently** (counted in summary) |
| R4 | Output | **Three lists + summary**: EMR-only, PAS-match-review, PAS-only, plus matched count and status breakdown |
| R5 | Distribution / updates | **GitHub Releases** auto-update via `tauri-plugin-updater`; alert + download + swap + relaunch |
| R6 | Privacy | **Fully offline, no telemetry**. Only network call is the update check to github.com / api.github.com. Patient data never leaves the machine |
| R7 | Target users | **Non-technical clinic staff**. Double-clickable installers (drag-to-Applications, MSI wizard, AppImage) |
| R8 | GUI framework | **Tauri** (Rust backend + system webview frontend) |
| R9 | Layout | **Split panel**: persistent sidebar (drop zone + summary + status breakdown) + main panel (list tabs + table) |
| R10 | Export | **Export CSV** button per list (explicit user action; only time data leaves memory) |
| R11 | Resolved tracking | **Click-to-toggle yellow highlight** on rows, in-memory only, cleared on close |
| R12 | Repo | Private GitHub repo `pas-recon` (created during implementation; placeholder `<owner>/pas-recon` in this spec) |

## 3. Architecture

Three crates in a Cargo workspace, plus a frontend:

```
pas-recon/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── engine/                   # pas-recon-engine: pure reconciliation logic
│   └── app/                      # pas-recon-app: Tauri shell (src-tauri)
└── frontend/                     # React + TS + Vite SPA
```

```
┌─────────────────────────────────────────────────────┐
│  Frontend (TypeScript + React, system webview)      │
│  Sidebar, drop zone, summary cards, patient tables  │
└──────────────────────┬──────────────────────────────┘
                       │ Tauri IPC (invoke / events)
┌──────────────────────┴──────────────────────────────┐
│  App Shell (Tauri commands, file dialogs, updater)  │
└──────────────────────┬──────────────────────────────┘
                       │ calls
┌──────────────────────┴──────────────────────────────┐
│  Reconciliation Engine (pure Rust library crate)    │
│  parse → normalize → validate → dedup → match       │
└─────────────────────────────────────────────────────┘
```

### 3.1 `pas-recon-engine` (pure Rust library)

The reconciliation logic as a pure pipeline. Takes bytes in, returns a structured result. No I/O, no async, no UI dependencies. Fully unit-testable.

**Dependencies:** `csv`, `serde`, `regex`, `chrono` (date parsing only).

**Public API:**
```rust
/// Entry point. Parses both CSV byte slices and runs the full pipeline.
pub fn reconcile(emr_csv: &[u8], pas_csv: &[u8]) -> Result<ReconciliationResult, EngineError>;

/// Same, but with caller-provided column overrides (when auto-detect fails / user picks manually).
pub fn reconcile_with_columns(
    emr_csv: &[u8],
    pas_csv: &[u8],
    emr_phn_column: Option<usize>,
    pas_phn_column: Option<usize>,
) -> Result<ReconciliationResult, EngineError>;
```

**Data model:**
```rust
struct ReconciliationResult {
    summary: Summary,
    emr_no_match: Vec<DisplayRow>,      // list 4
    pas_match_review: Vec<DisplayRow>,  // list 5
    pas_no_match: Vec<DisplayRow>,      // list 6
}

struct Summary {
    matched: usize,
    emr_only: usize,
    pas_only: usize,
    pas_review: usize,
    status_breakdown: StatusBreakdown,   // confirmed/pending/deceased/removed/not_mrp
    duplicates_dropped: usize,           // PAS rows dropped by dedup (silent)
    invalid_phn_skipped: usize,          // rows failing BC PHN validation
    unparseable_dates: usize,            // MRP-updated dates that couldn't be parsed
}

struct DisplayRow {
    phn: String,
    first_name: Option<String>,
    last_name: Option<String>,
    dob: Option<String>,
    mrp_status: Option<String>,          // PAS rows only
    raw_fields: Vec<String>,             // all original columns, for display/export
}

enum EngineError {
    Io(String),
    CsvParse { row: usize, source: csv::Error },
    MissingPhnColumn { source: CsvSource },   // EMR or PAS
    AmbiguousPhnColumns { source: CsvSource, candidates: Vec<String> },
}
```

### 3.2 `pas-recon-app` (Tauri shell)

Thin glue. Reads files, calls the engine, wires the updater. Exposes Tauri commands:

```rust
#[tauri::command]
fn reconcile_files(emr_path: String, pas_path: String) -> Result<ReconciliationResult, String>;

#[tauri::command]
fn reconcile_with_column_override(...) -> Result<ReconciliationResult, String>;

#[tauri::command]
fn export_list(list: ListKind, path: String) -> Result<(), String>;

#[tauri::command]
async fn check_for_updates() -> Result<Option<UpdateInfo>, String>;
```

File reading is synchronous into `Vec<u8>`; the heavy work happens on a blocking thread via `tauri::async_runtime::spawn_blocking` so the UI stays responsive. Patient data lives only in webview memory; nothing is written to disk except an explicit user export.

**Dependencies:** `tauri`, `tauri-plugin-updater`, `pas-recon-engine`, `serde`, `serde_json`.

### 3.3 Frontend (`frontend/`)

React + TypeScript SPA built with Vite, bundled into the Tauri binary. Calls Tauri commands via `@tauri-apps/api`.

**Key components:**
- `DropZone` — receives file-drop events, validates filenames, calls `reconcile_files`.
- `Sidebar` — summary counts, status breakdown, version + privacy note.
- `ListTabs` — switches between the three lists with count badges.
- `PatientTable` — sortable, searchable table per list. Click row → toggle resolved highlight.
- `UpdateToast` — non-blocking update notification.

## 4. Reconciliation Engine — matching logic

Replicates the spreadsheet's behavior, ported to Rust. Pipeline of pure functions:

### 4.1 Parse
Read CSV with the `csv` crate. Handle: BOM (`\u{FEFF}`), variable column counts (pad short rows with empty strings, ignore extras), quoted fields, CR/LF/CRLF line endings. Returns `Vec<Record>` keyed by header position. Empty files → `EngineError::Io`.

### 4.2 Column auto-detection
Scan the header row, match against known patterns (case-insensitive, whitespace-trimmed):

| Field | Header patterns |
|---|---|
| PHN | `phn`, `personal health number`, `bc phn`, `health number` |
| First name | `first`, `first name`, `given`, `given name`, `fname` |
| Last name | `last`, `last name`, `surname`, `family`, `lname` |
| DOB | `dob`, `date of birth`, `birth date`, `birthdate` |
| MRP status (PAS) | `mrp status`, `status`, `attachment status` |
| MRP updated (PAS) | `mrp updated`, `mrp updated date`, `updated`, `last updated` |

- PHN column not found → `EngineError::MissingPhnColumn`. App shell surfaces a manual column-picker; user selects a column → calls `reconcile_with_column_override`.
- Two columns both match a PHN pattern → `EngineError::AmbiguousPhnColumns` with candidates → same manual-picker fallback.
- First/last/DOB are "nice to have." Missing → those columns absent from output tables. No error.
- MRP fields are PAS-specific. Missing → no dedup tiebreak (falls back to keep-first-seen), no status breakdown. **Consequence:** if the PAS file has no status column, list 5 (`pas_match_review`) is always empty because no PAS row can be classified as "needs review." The app surfaces this as an info note, not an error.

### 4.3 Normalize & validate PHN
Strip spaces (` `), hyphens (`-`), non-breaking spaces (`\u{A0}`). Enforce **BC PHN**: exactly 10 digits, first digit `9`. Rows failing validation are **skipped entirely** — not matched, not placed in any output list, not shown in the UI tables. They are only counted in `invalid_phn_skipped` (surfaced as the "⚠ N rows skipped" note under the summary, expandable for the raw row details). (Matches the spreadsheet's `VALUE(SUBSTITUTE(...))` coercion + the `G5`/`G7` format check.)

### 4.4 PAS deduplication
Group PAS records by PHN. For each group with >1 entry, keep only the record whose `mrp_updated` is latest; drop the rest (counted in `duplicates_dropped`, not surfaced in any list). Ties on equal dates keep the first-seen row. If `mrp_updated` is missing or unparseable, fall back to keep-first-seen for that group and increment `unparseable_dates`.

Replicates spreadsheet sheet 8 cell `E8`: `COUNTIFS($F$8:$F$10007, F8, $H$8:$H$10007, ">"&H8)+1 = 1`.

### 4.5 Cross-match & classify
Build `HashSet<String>` of EMR PHNs and `HashSet<String>` of deduped PAS PHNs. Classify:

| Record | Condition | → List |
|---|---|---|
| EMR | PHN not in PAS set | `emr_no_match` (list 4) |
| PAS | PHN not in EMR set | `pas_no_match` (list 6) |
| PAS | PHN in EMR set, status ∈ {Pending, Not the MRP, Deceased, Removed} | `pas_match_review` (list 5) |
| PAS | PHN in EMR set, status = Confirmed (or unset) | matched, OK, not listed |
| EMR | PHN in PAS set | matched, OK, not listed |

Mirrors spreadsheet sheets 7 & 8 column A (`VLOOKUP`-then-classify) plus the status filter on list 5 (the `COUNTIFS(..., "<>Confirmed")` in cell `D8`).

### 4.6 Assemble
Sort each list by last name (case-insensitive), then first name. Rows with no last name sort to the end. Attach `raw_fields` so the UI can display whatever columns were in the source CSV. (The spreadsheet preserves source order; sorting is an intentional readability improvement.)

### 4.7 What the engine does NOT do
- No file I/O — caller passes bytes, gets a result.
- No fuzzy name matching — matching is strictly on normalized PHN, exactly as the spreadsheet does `VLOOKUP` on the numeric PHN.
- No date format heuristics beyond two formats the spreadsheet handles (Excel serial numbers and `D/M/YYYY` strings). Unparseable → fallback, not an error.
- No row-count caps. (The spreadsheet hard-codes 3007/10007/6007; the Rust engine handles arbitrary sizes bounded only by memory.)

## 5. UI / UX

Split-panel layout (confirmed Option B). Single window.

### 5.1 Left sidebar (fixed ~280px)
- **App title + version** (top).
- **Drop zone**: accepts the two named CSVs. Validates filename + that each parses as CSV. Shows ✓ per file once loaded. Red banner on file error.
- **Summary**: four counts with color dots — Matched (green), EMR only (red), PAS only (amber), Review (purple).
- **PAS Status Breakdown**: Confirmed / Pending / Deceased / Removed / Not the MRP, from the PAS MRP Status column. Hidden if PAS file lacks a status column.
- **Privacy note** (bottom): "Patient data stays on this machine. Closing the window clears it."

### 5.2 Main panel
- **List tabs** (header): switch between the three lists with count badges. Zero-count tabs show an empty state.
- **Context line** under tabs: one sentence of guidance per list, matching the spreadsheet's notes (e.g. "Patients in your EMR panel but not found in PAS. These may need a 98990 bill submitted, or have incorrect status/MRP in the EMR.").
- **Search box + Export CSV button**: search filters current list by PHN or name substring. Export saves the current list to a user-chosen path.
- **Patient table**: columns PHN / First / Last / DOB / Status (PAS lists only). Sticky header, sortable, scrollable. Click a row → toggle "resolved" yellow highlight (in-memory only). All original CSV columns available via a row-detail expansion if needed.
- **Status bar** (bottom): "Showing N patients · sorted by last name" on the left; "Data in memory only · not saved to disk" on the right.

### 5.3 Behaviors
- **Auto-reconcile**: the moment both files are validly loaded, reconciliation runs. No separate "Run" button. (Mirrors the spreadsheet's instant recalc.)
- **Re-dropping a file** replaces it and re-runs.
- **Resolved highlights** are per-session, not persisted. Cleared on close or on re-reconcile.

## 6. Error handling

Three failure classes, surfaced differently:

| Class | Examples | Engine mapping | UI response |
|---|---|---|---|
| **File errors** (blocking) | Wrong file, unreadable, not valid CSV, empty | `EngineError::Io`, `EngineError::CsvParse` | Red banner in drop zone. Reconciliation doesn't run until both files valid. |
| **Auto-detect fallback** (interactive) | PHN column not found; two columns both match PHN pattern | `EngineError::MissingPhnColumn`, `EngineError::AmbiguousPhnColumns` | Column-picker dialog: "Which column holds the PHN?" with dropdown of detected headers. User picks → reconcile proceeds. |
| **Row-level issues** (non-blocking) | Invalid PHN; unparseable MRP date | Counters on `ReconciliationResult` (`invalid_phn_skipped`, `unparseable_dates`) — never an `Err` | Row skipped from matching, counted in summary. "⚠ N rows skipped" line under summary, expandable for detail. Never blocks. |

The engine returns `Result<ReconciliationResult, EngineError>`. The two error variants in the first two rows are the only `Err` cases; everything else is a counter on a successful `Ok` result. A run always succeeds if the files parse and a PHN column resolves.

## 7. Auto-update (GitHub Releases)

Uses `tauri-plugin-updater`. The only network feature; patient data never traverses it.

1. On app launch (deferred ~3s after window show) and on manual "Check for updates" menu item, fetch latest release metadata from `https://github.com/<owner>/pas-recon/releases/latest`.
2. Compare release tag (`vX.Y.Z`) against built-in version via semver.
3. If newer, show non-blocking toast: "Update available — v1.2.0" with **Download & Install** and **Later**.
4. On "Download & Install": download platform artifact, verify signature (Tauri updater signs bundles with a private key; public key ships in app via `tauri.conf.json` `updater.pubkey`), swap binary, relaunch.

CSP allows only `github.com` and `api.github.com`. No other outbound connections.

Code signing is required for silent application:
- **macOS**: notarization (App Store Connect API key in CI) so Gatekeeper doesn't prompt.
- **Windows**: authenticode certificate so SmartScreen doesn't block.
- Without signing, updates still download but the OS prompts the user to approve — documented as acceptable fallback.

## 8. Packaging

Non-technical clinic staff → double-clickable installers. Tauri's bundler produces:

| Platform | Target | Artifact | Install |
|---|---|---|---|
| macOS | universal (aarch64 + x86_64) | `.dmg` → `.app` | Drag to Applications. Notarized. |
| Windows | x86_64 | `.msi` (NSIS) | Double-click wizard. Authenticode-signed. |
| Linux | x86_64 | `.deb` + `.AppImage` | `.deb` for Debian/Ubuntu; AppImage portable. |

Prerequisite: WebView2 on Windows. Ships on Win10 2004+ and Win11 by default; the installer can bootstrap it if missing via Tauri's `webview2` bootstrapper config.

### Tauri config essentials
- Single window, min size 900×600, title "PAS Reconciliation".
- `withGlobalTauri: true`.
- Updater pubkey in `tauri.conf.json`; private key in CI secrets (`TAURI_PRIVATE_KEY`, `TAURI_KEY_PASSWORD`).
- File-drop enabled (`onFileDropEvent`).
- CSP: default-src 'self'; connect-src only to `github.com` / `api.github.com`.

## 9. CI / Release pipeline (GitHub Actions)

On push of a `v*` tag:
1. Three jobs (ubuntu-latest for Linux + Windows cross, macos-latest for macOS) run `tauri build` producing the platform artifacts + updater signatures.
2. macOS job runs notarization (secrets: `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_KEY`*).
3. Windows job signs with authenticode (secret: `WINDOWS_CERTIFICATE`*).
4. A final job creates the GitHub Release, uploads all artifacts + the `latest.json` manifest the updater fetches.

(*Signing secrets are configured by the user during implementation; the pipeline supports unsigned builds as a fallback for development — they just won't auto-apply silently.)

## 10. Out of scope (YAGNI)

Explicitly excluded to keep scope tight:
- No patient database or history across sessions.
- No multi-clinic / multi-provider profiles.
- No fuzzy name matching.
- No PDF report generation (CSV export covers sharing).
- No telemetry or crash reporting.
- No localization beyond English (the spreadsheet is English-only).
- No saved "resolved" state across sessions.

## 11. Open items for implementation

- `<owner>/pas-recon` placeholder: create the private GitHub repo during implementation.
- Generate the Tauri updater keypair (`tauri signer generate`); store private key + password in repo secrets; commit the public key to `tauri.conf.json`.
- Obtain Apple notarization + Windows authenticode credentials (or decide to ship unsigned initially and document the OS-prompt fallback).
