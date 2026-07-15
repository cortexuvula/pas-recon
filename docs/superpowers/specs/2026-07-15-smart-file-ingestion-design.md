# Smart File Ingestion — Design Spec

**Date:** 2026-07-15
**Status:** Approved design, pending implementation plan

## 1. Problem

The current file ingestion logic classifies dropped CSVs by checking if the filename contains "emr" or "pas" (`name.includes("emr")`). This fails when clinic staff drop files with generic names like `export.csv`, `report_2026-07-15.csv`, or `patient_panel.csv`. Unrecognized files are silently ignored — no error, no reconciliation, just nothing happens.

## 2. Solution

Replace filename-based classification with header-based detection. The PAS export has distinctive column headers ("PAS MRP Status", "PAS MRP Updated") that no EMR export would have. This provides a reliable signal regardless of filename.

### 2.1 Detection algorithm

When two files are dropped, the app reads each file's header row (via the existing `get_csv_headers` backend command) and classifies:

1. **PAS detection:** A file is the PAS file if its headers contain "PAS MRP Status" or "PAS MRP Updated" (case-insensitive substring match).
2. **EMR by elimination:** The other file is the EMR file.
3. **Filename fallback:** If neither file has PAS headers, fall back to the current filename check (`includes("emr")` / `includes("pas")`).
4. **Ambiguous:** If both files have PAS headers or neither can be classified → show a confirmation dialog.

### 2.2 Confidence levels

| Situation | Action |
|---|---|
| One file has PAS headers, other doesn't | Auto-classify, proceed silently |
| Neither file has PAS headers, but filenames contain "emr"/"pas" | Auto-classify, proceed silently |
| Neither file has PAS headers, filenames don't match | Show confirmation dialog |
| Both files have PAS headers | Show confirmation dialog (unusual) |
| Only one file dropped | Wait for second file (current behavior) |

### 2.3 Confirmation dialog

When classification is ambiguous, show a modal in the main panel:
- Each filename shown with its detected column headers (first 5)
- Assignment shown: "File A → EMR, File B → PAS"
- Two buttons: **"Confirm"** (proceed with current assignment) and **"Swap"** (flip and proceed)

### 2.4 What changes

| Layer | Change |
|---|---|
| **Frontend `App.tsx`** | Replace `name.includes("emr")` logic with header-based scoring. Add `showFileConfirm` state and pending file paths. |
| **Frontend `FileConfirm.tsx`** | New component: shows two files with headers, Confirm/Swap buttons. |
| **Frontend `api.ts`** | No change — `get_csv_headers` already exists. |
| **Engine** | No change. |
| **Backend commands** | No change. |

### 2.5 What stays the same

- Engine reconciliation pipeline, PHN validation, column detection — all unchanged.
- Backend commands — all unchanged.
- Drop zone visual — unchanged.
- The `reconcileFiles(emrPath, pasPath)` call — same interface, we just ensure the right path goes to the right parameter before calling it.

## 3. Out of scope

- No file content preview beyond headers.
- No persistent file assignment memory across sessions.
- No support for more than two files at once.
