# PAS Reconciliation

A cross-platform desktop app that reconciles a clinic's EMR patient panel
against the Provincial Attachment System (PAS) patient list by matching
Personal Health Numbers (PHNs).

Replaces the `PAS Rec with EMR (Excel LibreOffice Calc) TEMPLATE.xlsx`
spreadsheet with a purpose-built tool for clinic staff.

## Download

Download the latest release from the
[releases page](https://github.com/cortexuvula/pas-recon/releases/latest):

- **macOS** (Apple Silicon): `.dmg` (aarch64)
- **macOS** (Intel): `.dmg` (x64)
- **Windows**: `.exe` installer or `.msi`
- **Linux**: `.deb`, `.rpm`, or `.AppImage`

The app checks for updates automatically on launch and can self-update
in-place on macOS and Windows. Linux users on `.deb`/`.rpm` installs are
directed to download manually (Tauri v2 limitation — AppImage installs
support auto-update).

## Features

### File ingestion
- **Drag-and-drop or browse** — drop CSV files onto the app, or use the
  Browse buttons to open a native file picker. Load one file at a time
  or both at once.
- **Smart prompts** — when one file is loaded, the app tells you what's
  still needed ("EMR loaded — now select your PAS file").
- **Auto-detection** — the app inspects column headers to determine
  which file is EMR vs PAS (looks for "PAS MRP Status" / "PAS MRP
  Updated"). Falls back to a confirmation dialog if ambiguous.
- **Auto column detection** — identifies the PHN, name, DOB, status,
  and date columns by header pattern matching. Manual column picker
  fallback if auto-detection fails.

### Reconciliation engine
- **BC PHN validation** — validates PHNs using the official TELEPLAN
  MOD-11 check-digit algorithm (weights `[0, 2, 4, 8, 5, 10, 9, 7, 3]`).
  Invalid PHNs are surfaced in a dedicated tab with source provenance
  (EMR or PAS) and color-coded indicators.
- **Smart deduplication** — PAS duplicate PHNs are deduplicated keeping
  the record with the latest MRP-updated date. EMR duplicates are also
  deduplicated.
- **Unparseable date warnings** — when MRP-updated dates can't be
  parsed, the summary warns the user (dedup falls back to first-seen).

### Review lists
- **EMR No Match** — patients in your EMR but not in PAS.
- **PAS Match - Review** — matched patients with a status of Pending,
  Not the MRP, Deceased, or Removed.
- **PAS No Match** — patients in PAS but not in your EMR.
- **Invalid PHNs** — rows that failed BC PHN validation, with source
  file identification (EMR/PAS) and color-coded provenance.

### Data interaction
- **Click-to-sort** — every column header is clickable to sort ascending
  or descending. Sort by status, name, PHN, DOB, or source.
- **Search** — filter any list by PHN or name substring.
- **Resolved tracking** — click a row to toggle a yellow highlight marking
  it resolved (in-memory only, cleared on close).
- **CSV export** — export any list to CSV with optional status filter.
- **PDF export** — export as a formatted HTML table that opens in the
  browser for Print to PDF. Includes title, generation date, and
  bordered table with bold headers.
- **Status filter** — on tabs with a Status column, filter exports to
  all rows or a specific status (e.g. only Pending, only Deceased).

### Application
- **Auto-update** — checks GitHub Releases on launch; download and
  install with one click (macOS/Windows). Linux users get a manual
  download link.
- **Human-readable errors** — internal errors are translated to plain
  English messages displayed in a dismissible banner in the main panel.
- **Fully offline** — no telemetry, no data leaves the machine. The only
  network call is the update check.

## Quick Start (Development)

### Prerequisites

- Rust 1.75+ (`rustup`)
- Node.js 20+
- For Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

### Build & Run

```bash
# Install frontend dependencies
cd frontend && npm ci && cd ..

# Run in development mode (launches both Vite dev server and Tauri)
cargo tauri dev
```

### Test

```bash
# Engine unit tests
cargo test --package pas-recon-engine

# Full workspace
cargo check --workspace
```

## How It Works

The engine matches patients strictly by normalized BC PHN (10 digits
starting with 9, validated with the MOD-11 TELEPLAN check-digit algorithm
using weights `[0, 2, 4, 8, 5, 10, 9, 7, 3]`, spaces/hyphens stripped).
PAS duplicate PHNs are deduplicated keeping the record with the latest
MRP-updated date. EMR records are also deduplicated by PHN (first seen).

See `docs/spreadsheet-formulas.md` for the original spreadsheet logic this
replaces, and `docs/superpowers/specs/` for the full design specifications.

## Privacy

All processing is local. Patient data never leaves the machine. The only
network call is the update check (GitHub Releases), which transmits no
patient data. No telemetry.

## Release

See `docs/release-setup.md` for one-time signing and CI configuration.
To cut a release: `git tag v0.X.Y && git push origin v0.X.Y`.
