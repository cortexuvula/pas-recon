# CSV/PDF Export Format Selection — Design Spec

**Date:** 2026-07-24
**Status:** Approved, pending implementation

## Problem

Export only supports CSV. Users need PDF for printing and sharing with clinic staff who may not use spreadsheets.

## Solution

The save dialog offers both CSV and PDF formats. When PDF is selected, the backend generates a formatted table document.

### Export flow

1. User clicks Export CSV → save dialog opens with two format filters: "CSV" and "PDF"
2. User selects format and filename
3. Frontend detects format from the file extension or filter selection
4. Calls `exportList(rows, path, format)` with format = "csv" or "pdf"
5. Backend writes CSV (existing) or generates PDF (new)

### PDF layout

- **Title header**: list name + status filter if applied + generation date
- **Table**: same columns as on screen, bold header row, zebra-striped data rows
- **Page format**: A4/Letter width, landscape orientation for wider tables
- **Footer**: page numbers
- **Font**: built-in Helvetica (pdf-writer default, no external font files)

### What changes

| File | Change |
|---|---|
| `crates/app/Cargo.toml` | Add `pdf-writer = "0.10"` |
| `crates/app/src/commands.rs` | `export_list` gains `format: String` param. New `export_pdf()` helper. |
| `frontend/src/App.tsx` | Save dialog offers both filters. Detects format, passes to `exportList`. |
| `frontend/src/api.ts` | `exportList` gains `format` parameter. |

### What stays the same

- Status filter works for both formats
- Engine unchanged
- Save dialog still lets user choose filename/location
