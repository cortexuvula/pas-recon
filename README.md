# PAS Reconciliation

A cross-platform desktop app that reconciles a clinic's EMR patient panel
against the Provincial Attachment System (PAS) patient list by matching
Personal Health Numbers (PHNs).

Replaces the `PAS Rec with EMR (Excel LibreOffice Calc) TEMPLATE.xlsx`
spreadsheet with a purpose-built tool for clinic staff.

## Quick Start (Development)

### Prerequisites

- Rust 1.75+ (`rustup`)
- Node.js 20+
- For Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

### Build & Run

```bash
# Install frontend dependencies
cd frontend && npm install && cd ..

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

## Usage

1. Launch the app.
2. Drag `EMR Active Patient List.csv` and `PAS Patient List.csv` into the
   drop zone (sidebar).
3. Reconciliation runs automatically once both files are loaded.
4. Review the three lists:
   - **EMR No Match**: patients in your EMR but not in PAS.
   - **PAS Match - Review**: matched patients with a status of Pending,
     Not the MRP, Deceased, or Removed.
   - **PAS No Match**: patients in PAS but not in your EMR.
5. Click a row to mark it resolved (yellow highlight).
6. Use Export CSV to save a list for printing or sharing.

## How It Works

The engine matches patients strictly by normalized BC PHN (10 digits,
starts with 9, spaces/hyphens stripped). PAS duplicate PHNs are deduplicated
keeping the record with the latest MRP-updated date.

See `docs/spreadsheet-formulas.md` for the original spreadsheet logic this
replaces, and `docs/superpowers/specs/2026-07-14-pas-recon-app-design.md`
for the full design.

## Privacy

All processing is local. Patient data never leaves the machine. The only
network call is the update check (GitHub Releases), which transmits no
patient data. No telemetry.

## Release

See `docs/release-setup.md` for one-time signing and CI configuration.
To cut a release: `git tag v0.X.Y && git push origin v0.X.Y`.
