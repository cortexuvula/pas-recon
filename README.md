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
in-place.

## Features

- **Drag-and-drop CSV ingestion** — drop both files; the app auto-detects
  which is EMR vs PAS by inspecting column headers (looks for "PAS MRP
  Status" / "PAS MRP Updated"). Falls back to a confirmation dialog if
  ambiguous.
- **Auto column detection** — identifies the PHN, name, DOB, status, and
  date columns by header pattern matching. Manual column picker fallback
  if auto-detection fails.
- **BC PHN validation** — validates PHNs using the official TELEPLAN
  MOD-11 check-digit algorithm. Invalid PHNs are surfaced in a dedicated
  tab with source provenance (EMR or PAS).
- **Smart deduplication** — PAS duplicate PHNs are deduplicated keeping
  the record with the latest MRP-updated date. EMR duplicates are also
  deduplicated.
- **Three review lists + invalid PHNs tab:**
  - **EMR No Match** — patients in your EMR but not in PAS.
  - **PAS Match - Review** — matched patients with a status of Pending,
    Not the MRP, Deceased, or Removed.
  - **PAS No Match** — patients in PAS but not in your EMR.
  - **Invalid PHNs** — rows that failed BC PHN validation, with source
    file identification.
- **Click-to-sort** — every column header is clickable to sort ascending
  or descending. Sort by status, name, PHN, DOB, or source.
- **Search** — filter any list by PHN or name substring.
- **Resolved tracking** — click a row to toggle a yellow highlight marking
  it resolved.
- **CSV export** — export any list to CSV for printing or sharing.
- **Auto-update** — checks GitHub Releases on launch; download and install
  with one click.
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
# Engine unit tests (48 tests)
cargo test --package pas-recon-engine

# Full workspace
cargo check --workspace
```

## How It Works

The engine matches patients strictly by normalized BC PHN (10 digits
starting with 9, validated with the MOD-11 TELEPLAN check-digit algorithm,
spaces/hyphens stripped). PAS duplicate PHNs are deduplicated keeping the
record with the latest MRP-updated date.

See `docs/spreadsheet-formulas.md` for the original spreadsheet logic this
replaces, and `docs/superpowers/specs/` for the full design specifications.

## Privacy

All processing is local. Patient data never leaves the machine. The only
network call is the update check (GitHub Releases), which transmits no
patient data. No telemetry.

## Release

See `docs/release-setup.md` for one-time signing and CI configuration.
To cut a release: `git tag v0.X.Y && git push origin v0.X.Y`.
