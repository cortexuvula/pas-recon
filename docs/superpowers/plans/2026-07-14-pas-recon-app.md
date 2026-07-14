# PAS Reconciliation App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform desktop app that replaces the PAS reconciliation spreadsheet — user drops two CSVs, app matches patients by PHN, shows three review lists + summary, with GitHub Releases auto-update.

**Architecture:** Cargo workspace with three crates: `pas-recon-engine` (pure Rust reconciliation logic), `pas-recon-app` (Tauri shell exposing commands), and a React/TS frontend bundled into the Tauri binary. The engine is the heart — pure functions, fully unit-tested in isolation.

**Tech Stack:** Rust, Tauri v2, React + TypeScript + Vite, `tauri-plugin-updater`, `csv` crate, `serde`/`serde_json`, `chrono`. CI via GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-07-14-pas-recon-app-design.md`

---

## Phases

This plan is organized into 6 phases. Each phase produces working, testable software:

1. **Engine core** — parse + data model + column detection + PHN validation (pure Rust, fully tested)
2. **Engine matching** — dedup + cross-match + classify + assemble (pure Rust, fully tested)
3. **Tauri shell** — workspace wiring, commands, IPC contract
4. **Frontend** — split-panel UI, drop zone, tables, summary
5. **Auto-update + packaging** — updater plugin, CI pipeline, installers
6. **Polish** — error states, column-picker fallback, resolved highlighting, edge cases

---

## File Structure

```
pas-recon/
├── Cargo.toml                              # workspace root
├── crates/
│   ├── engine/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                      # public API: reconcile(), reconcile_with_columns()
│   │   │   ├── model.rs                    # ReconciliationResult, Summary, DisplayRow, EngineError
│   │   │   ├── parse.rs                    # CSV → Vec<RawRow>, BOM handling, flexible columns
│   │   │   ├── detect.rs                   # column auto-detection by header patterns
│   │   │   ├── phn.rs                      # normalize + validate BC PHN
│   │   │   ├── dedup.rs                    # PAS dedup by latest MRP-updated date
│   │   │   ├── date.rs                     # parse MRP-updated dates (serial + D/M/YYYY)
│   │   │   └── reconcile.rs                # pipeline: parse→detect→validate→dedup→match→classify
│   │   ├── tests/
│   │   │   ├── parse_test.rs
│   │   │   ├── detect_test.rs
│   │   │   ├── phn_test.rs
│   │   │   ├── dedup_test.rs
│   │   │   ├── date_test.rs
│   │   │   └── reconcile_test.rs
│   │   └── fixtures/                       # sample CSVs for tests
│   │       ├── emr_basic.csv
│   │       ├── pas_basic.csv
│   │       ├── emr_dirty.csv               # BOM, bad PHNs, short rows
│   │       └── pas_duplicates.csv          # duplicate PHNs, mixed dates
│   └── app/
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── build.rs
│       ├── icons/                          # app icons (generated)
│       └── src/
│           ├── main.rs                     # Tauri entry, plugin registration
│           ├── commands.rs                 # #[tauri::command] functions
│           └── update.rs                   # updater wiring
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.tsx                        # React entry
│       ├── App.tsx                         # root: sidebar + main panel layout
│       ├── types.ts                        # TS types mirroring engine model
│       ├── api.ts                          # Tauri IPC wrappers (invoke calls)
│       ├── components/
│       │   ├── DropZone.tsx
│       │   ├── Sidebar.tsx
│       │   ├── ListTabs.tsx
│       │   ├── PatientTable.tsx
│       │   ├── SummaryCard.tsx
│       │   ├── StatusBreakdown.tsx
│       │   ├── ColumnPicker.tsx            # fallback when auto-detect fails
│       │   ├── UpdateToast.tsx
│       │   └── EmptyState.tsx
│       └── styles/
│           └── app.css
├── .github/
│   └── workflows/
│       └── release.yml                     # cross-platform build + release
└── docs/
    ├── spreadsheet-formulas.md
    └── superpowers/
        ├── specs/2026-07-14-pas-recon-app-design.md
        └── plans/2026-07-14-pas-recon-app.md  (this file)
```

---

# Phase 1: Engine Core

## Task 1: Initialize Cargo workspace and engine crate

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/engine/Cargo.toml`
- Create: `crates/engine/src/lib.rs`

- [ ] **Step 1: Create the workspace root Cargo.toml**

```toml
[workspace]
members = ["crates/engine"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
```

- [ ] **Step 2: Create the engine crate Cargo.toml**

```toml
[package]
name = "pas-recon-engine"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
csv = "1.3"
serde = { version = "1", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
regex = "1"
```

- [ ] **Step 3: Create a minimal lib.rs**

```rust
//! PAS Reconciliation Engine — pure Rust reconciliation logic.
//!
//! Takes two CSV byte slices (EMR panel + PAS patient list), matches patients
//! by PHN, and returns three review lists plus a summary.

pub mod model;

pub fn reconcile(_emr_csv: &[u8], _pas_csv: &[u8]) -> Result<model::ReconciliationResult, model::EngineError> {
    todo!("implemented in Task 7")
}
```

- [ ] **Step 4: Create a stub model.rs so it compiles**

```rust
//! Data model for the reconciliation engine.

/// The complete output of a reconciliation run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReconciliationResult {
    pub summary: Summary,
    pub emr_no_match: Vec<DisplayRow>,
    pub pas_match_review: Vec<DisplayRow>,
    pub pas_no_match: Vec<DisplayRow>,
}

/// Aggregate counts shown in the sidebar.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Summary {
    pub matched: usize,
    pub emr_only: usize,
    pub pas_only: usize,
    pub pas_review: usize,
    pub status_breakdown: StatusBreakdown,
    pub duplicates_dropped: usize,
    pub invalid_phn_skipped: usize,
    pub unparseable_dates: usize,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct StatusBreakdown {
    pub confirmed: usize,
    pub pending: usize,
    pub deceased: usize,
    pub removed: usize,
    pub not_the_mrp: usize,
}

/// One row in an output list.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DisplayRow {
    pub phn: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub dob: Option<String>,
    pub mrp_status: Option<String>,
    pub raw_fields: Vec<String>,
}

/// Errors that abort a reconciliation run.
#[derive(Debug, Clone, thiserror::Error, serde::Serialize)]
pub enum EngineError {
    #[error("failed to read {source} file: {message}")]
    Io { source: String, message: String },

    #[error("CSV parse error in {source} at row {row}: {message}")]
    CsvParse { source: String, row: usize, message: String },

    #[error("could not find a PHN column in {source} CSV")]
    MissingPhnColumn { source: String },

    #[error("multiple columns in {source} CSV look like PHNs: {candidates:?}")]
    AmbiguousPhnColumns { source: String, candidates: Vec<String> },
}

/// Which CSV file an error or record belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CsvSource {
    Emr,
    Pas,
}

impl std::fmt::Display for CsvSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvSource::Emr => write!(f, "EMR"),
            CsvSource::Pas => write!(f, "PAS"),
        }
    }
}
```

- [ ] **Step 5: Add thiserror to dependencies**

Update `crates/engine/Cargo.toml` `[dependencies]` section — add:

```toml
thiserror = "1"
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors (the `todo!()` is fine — it compiles, panics at runtime).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/
git commit -m "feat(engine): initialize workspace and engine crate with data model"
```

---

## Task 2: CSV parser — raw row extraction

**Files:**
- Create: `crates/engine/src/parse.rs`
- Modify: `crates/engine/src/lib.rs` (add `pub mod parse;`)
- Create: `crates/engine/src/model.rs` — add `RawRow` struct (modify the file)
- Test: `crates/engine/tests/parse_test.rs`
- Create: `crates/engine/fixtures/emr_basic.csv`

- [ ] **Step 1: Add RawRow to model.rs**

Add to `crates/engine/src/model.rs`:

```rust
/// One parsed CSV row before column mapping. All fields are raw strings.
#[derive(Debug, Clone)]
pub struct RawRow {
    pub fields: Vec<String>,
    pub row_index: usize, // 0-based, excluding header
}
```

- [ ] **Step 2: Create the test fixture**

Create `crates/engine/fixtures/emr_basic.csv`:

```csv
PHN,First Name,Last Name,DOB
9876543210,John,Smith,1965-03-12
9871111222,Mary,Jones,1978-11-04
9873333444,Robert,Lee,1952-07-22
```

- [ ] **Step 3: Write the failing test**

Create `crates/engine/tests/parse_test.rs`:

```rust
use pas_recon_engine::parse::{parse_csv, ParsedCsv};

#[test]
fn parses_basic_csv_with_header_and_rows() {
    let csv_bytes = b"PHN,First Name,Last Name,DOB\n9876543210,John,Smith,1965-03-12\n9871111222,Mary,Jones,1978-11-04\n";
    let result: ParsedCsv = parse_csv(csv_bytes).unwrap();

    assert_eq!(result.headers, vec!["PHN", "First Name", "Last Name", "DOB"]);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].fields, vec!["9876543210", "John", "Smith", "1965-03-12"]);
    assert_eq!(result.rows[0].row_index, 0);
    assert_eq!(result.rows[1].row_index, 1);
}

#[test]
fn strips_bom_from_start_of_file() {
    let bom = "\u{FEFF}";
    let csv_bytes = format!("{bom}PHN,Name\n9876543210,John\n").into_bytes();
    let result = parse_csv(&csv_bytes).unwrap();
    assert_eq!(result.headers[0], "PHN"); // not "\u{FEFF}PHN"
}

#[test]
fn pads_short_rows_with_empty_strings() {
    let csv_bytes = b"A,B,C\n1,2\n";
    let result = parse_csv(csv_bytes).unwrap();
    assert_eq!(result.rows[0].fields, vec!["1", "2", ""]); // padded to 3
}

#[test]
fn ignores_extra_columns_beyond_header() {
    let csv_bytes = b"A,B\n1,2,3,4\n";
    let result = parse_csv(csv_bytes).unwrap();
    assert_eq!(result.rows[0].fields, vec!["1", "2"]); // truncated to header length
}

#[test]
fn handles_crlf_line_endings() {
    let csv_bytes = b"A,B\r\n1,2\r\n3,4\r\n";
    let result = parse_csv(csv_bytes).unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn returns_error_for_empty_input() {
    let result = parse_csv(b"");
    assert!(result.is_err());
}

#[test]
fn returns_error_for_header_only() {
    let result = parse_csv(b"A,B,C\n");
    assert!(result.is_err());
}
```

- [ ] **Step 4: Run tests to verify they fail**

Run: `cargo test --package pas-recon-engine --test parse_test`
Expected: FAIL — `parse_csv` and `ParsedCsv` not found.

- [ ] **Step 5: Implement parse.rs**

Create `crates/engine/src/parse.rs`:

```rust
//! CSV parsing into raw rows. Handles BOM, flexible column counts, CRLF.

use crate::model::RawRow;

/// The result of parsing a CSV: headers + data rows.
#[derive(Debug, Clone)]
pub struct ParsedCsv {
    pub headers: Vec<String>,
    pub rows: Vec<RawRow>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("empty file — no content")]
    Empty,
    #[error("file contains only a header row, no data")]
    HeaderOnly,
    #[error("CSV read error at line {line}: {message}")]
    Read { line: usize, message: String },
}

/// Parse CSV bytes into headers + raw rows.
///
/// - Strips a leading BOM.
/// - Pads rows shorter than the header with empty strings.
/// - Truncates rows longer than the header.
pub fn parse_csv(input: &[u8]) -> Result<ParsedCsv, ParseError> {
    // Strip BOM if present
    let input = input.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(input);

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true) // allow variable column counts
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(input);

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| ParseError::Read { line: 1, message: e.to_string() })?
        .iter()
        .map(|s| s.to_string())
        .collect();

    if headers.is_empty() {
        return Err(ParseError::Empty);
    }

    let header_len = headers.len();
    let mut rows = Vec::new();
    let mut row_index = 0usize;

    for (line_no, record) in rdr.records().enumerate() {
        let record = record.map_err(|e| ParseError::Read {
            line: line_no + 2, // +2: 1-based, +1 for header
            message: e.to_string(),
        })?;

        let mut fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // Pad or truncate to match header length
        if fields.len() < header_len {
            fields.resize(header_len, String::new());
        } else if fields.len() > header_len {
            fields.truncate(header_len);
        }

        rows.push(RawRow { fields, row_index });
        row_index += 1;
    }

    if rows.is_empty() {
        return Err(ParseError::HeaderOnly);
    }

    Ok(ParsedCsv { headers, rows })
}
```

- [ ] **Step 6: Expose the module in lib.rs**

In `crates/engine/src/lib.rs`, add:

```rust
pub mod parse;
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test --package pas-recon-engine --test parse_test`
Expected: all 7 tests PASS.

- [ ] **Step 8: Commit**

```bash
git add crates/engine/src/parse.rs crates/engine/src/model.rs crates/engine/src/lib.rs crates/engine/tests/parse_test.rs crates/engine/fixtures/
git commit -m "feat(engine): CSV parser with BOM stripping, padding, flexible columns"
```

---

## Task 3: PHN normalization and validation

**Files:**
- Create: `crates/engine/src/phn.rs`
- Modify: `crates/engine/src/lib.rs` (add `pub mod phn;`)
- Test: `crates/engine/tests/phn_test.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/engine/tests/phn_test.rs`:

```rust
use pas_recon_engine::phn::{normalize_phn, is_valid_bc_phn};

#[test]
fn strips_spaces_hyphens_nbsp() {
    assert_eq!(normalize_phn("9876 543 210"), "9876543210");
    assert_eq!(normalize_phn("9876-543-210"), "9876543210");
    assert_eq!(normalize_phn("9876\u{00A0}543\u{00A0}210"), "9876543210");
    assert_eq!(normalize_phn(" 9876543210 "), "9876543210");
}

#[test]
fn valid_bc_phn_accepted() {
    assert!(is_valid_bc_phn("9876543210"));
    assert!(is_valid_bc_phn("9876 543 210"));
    assert!(is_valid_bc_phn("9123456789"));
}

#[test]
fn rejects_wrong_length() {
    assert!(!is_valid_bc_phn("987654321"));   // 9 digits
    assert!(!is_valid_bc_phn("98765432101")); // 11 digits
}

#[test]
fn rejects_wrong_start_digit() {
    assert!(!is_valid_bc_phn("1234567890")); // starts with 1, not 9
}

#[test]
fn rejects_non_numeric() {
    assert!(!is_valid_bc_phn("9876abc210"));
    assert!(!is_valid_bc_phn(""));
    assert!(!is_valid_bc_phn("abcdefghij"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package pas-recon-engine --test phn_test`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement phn.rs**

Create `crates/engine/src/phn.rs`:

```rust
//! BC PHN normalization and validation.
//!
//! A valid BC PHN is exactly 10 digits, first digit 9.
//! Normalization strips spaces, hyphens, and non-breaking spaces before checking.

/// Strip spaces, hyphens, non-breaking spaces, and surrounding whitespace
/// from a PHN-like string, leaving only the digits (and any other chars).
pub fn normalize_phn(raw: &str) -> String {
    raw.chars()
        .filter(|c| !matches!(c, ' ' | '-' | '\u{00A0}'))
        .collect()
}

/// Returns true if the raw string, after normalization, is a valid BC PHN:
/// exactly 10 digits, first digit '9'.
pub fn is_valid_bc_phn(raw: &str) -> bool {
    let normalized = normalize_phn(raw);
    normalized.len() == 10
        && normalized.starts_with('9')
        && normalized.chars().all(|c| c.is_ascii_digit())
}
```

- [ ] **Step 4: Expose the module in lib.rs**

In `crates/engine/src/lib.rs`, add:

```rust
pub mod phn;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package pas-recon-engine --test phn_test`
Expected: all 5 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/engine/src/phn.rs crates/engine/src/lib.rs crates/engine/tests/phn_test.rs
git commit -m "feat(engine): BC PHN normalization and validation"
```

---

## Task 4: Column auto-detection

**Files:**
- Create: `crates/engine/src/detect.rs`
- Modify: `crates/engine/src/model.rs` (add `ColumnMapping`)
- Modify: `crates/engine/src/lib.rs` (add `pub mod detect;`)
- Test: `crates/engine/tests/detect_test.rs`

- [ ] **Step 1: Add ColumnMapping to model.rs**

Add to `crates/engine/src/model.rs`:

```rust
/// Which source column index maps to each recognized field.
/// Only `phn` is required; others are `None` if not detected.
#[derive(Debug, Clone)]
pub struct ColumnMapping {
    pub phn: usize,
    pub first_name: Option<usize>,
    pub last_name: Option<usize>,
    pub dob: Option<usize>,
    pub mrp_status: Option<usize>,   // PAS only
    pub mrp_updated: Option<usize>,  // PAS only
}
```

- [ ] **Step 2: Write the failing test**

Create `crates/engine/tests/detect_test.rs`:

```rust
use pas_recon_engine::detect::{detect_columns, DetectionError};
use pas_recon_engine::parse::parse_csv;

fn headers_csv(headers: &[&str]) -> Vec<u8> {
    let mut s = headers.join(",");
    s.push_str("\n,x,\n"); // one data row so parse doesn't complain
    s.into_bytes()
}

#[test]
fn detects_phn_by_exact_match() {
    let parsed = parse_csv(&headers_csv(&["PHN", "First", "Last"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.phn, 0);
}

#[test]
fn detects_phn_by_case_insensitive_match() {
    let parsed = parse_csv(&headers_csv(&["first name", "phn", "last"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.phn, 1);
}

#[test]
fn detects_phn_by_long_form() {
    let parsed = parse_csv(&headers_csv(&["Personal Health Number", "Name"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.phn, 0);
}

#[test]
fn detects_all_fields_at_once() {
    let parsed = parse_csv(&headers_csv(&["PHN", "First Name", "Last Name", "DOB", "Status", "MRP Updated"])).unwrap();
    let mapping = detect_columns(&parsed.headers, true).unwrap(); // true = PAS mode
    assert_eq!(mapping.phn, 0);
    assert_eq!(mapping.first_name, Some(1));
    assert_eq!(mapping.last_name, Some(2));
    assert_eq!(mapping.dob, Some(3));
    assert_eq!(mapping.mrp_status, Some(4));
    assert_eq!(mapping.mrp_updated, Some(5));
}

#[test]
fn missing_phn_returns_error() {
    let parsed = parse_csv(&headers_csv(&["Name", "Age", "City"])).unwrap();
    let result = detect_columns(&parsed.headers, false);
    assert!(matches!(result, Err(DetectionError::MissingPhnColumn)));
}

#[test]
fn ambiguous_phn_columns_returns_error() {
    let parsed = parse_csv(&headers_csv(&["PHN", "Personal Health Number"])).unwrap();
    let result = detect_columns(&parsed.headers, false);
    assert!(matches!(result, Err(DetectionError::AmbiguousPhnColumns { .. })));
}

#[test]
fn pas_fields_not_detected_in_emr_mode() {
    // EMR mode (is_pas=false) should not try to detect status/updated
    let parsed = parse_csv(&headers_csv(&["PHN", "Status", "MRP Updated"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.mrp_status, None);
    assert_eq!(mapping.mrp_updated, None);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --package pas-recon-engine --test detect_test`
Expected: FAIL — module not found.

- [ ] **Step 4: Implement detect.rs**

Create `crates/engine/src/detect.rs`:

```rust
//! Column auto-detection by header pattern matching.

use crate::model::ColumnMapping;

#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("no PHN column found")]
    MissingPhnColumn,
    #[error("multiple PHN columns found: {candidates:?}")]
    AmbiguousPhnColumns { candidates: Vec<String> },
}

/// Header patterns for each recognized field. Matched case-insensitively
/// after trimming whitespace.
const PHN_PATTERNS: &[&str] = &["phn", "personal health number", "bc phn", "health number"];
const FIRST_PATTERNS: &[&str] = &["first", "first name", "given", "given name", "fname"];
const LAST_PATTERNS: &[&str] = &["last", "last name", "surname", "family", "lname"];
const DOB_PATTERNS: &[&str] = &["dob", "date of birth", "birth date", "birthdate"];
const STATUS_PATTERNS: &[&str] = &["mrp status", "status", "attachment status"];
const UPDATED_PATTERNS: &[&str] = &["mrp updated", "mrp updated date", "updated", "last updated"];

fn find_column(headers: &[String], patterns: &[&str]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let normalized = header.trim().to_lowercase();
        if patterns.iter().any(|p| normalized == *p) {
            return Some(idx);
        }
    }
    None
}

fn find_all_columns(headers: &[String], patterns: &[&str]) -> Vec<usize> {
    headers
        .iter()
        .enumerate()
        .filter(|(_, h)| {
            let normalized = h.trim().to_lowercase();
            patterns.iter().any(|p| normalized == *p)
        })
        .map(|(idx, _)| idx)
        .collect()
}

/// Detect column mapping from headers. `is_pas` controls whether to look for
/// MRP status/updated columns.
pub fn detect_columns(headers: &[String], is_pas: bool) -> Result<ColumnMapping, DetectionError> {
    let phn_candidates = find_all_columns(headers, PHN_PATTERNS);

    let phn = match phn_candidates.len() {
        0 => return Err(DetectionError::MissingPhnColumn),
        1 => phn_candidates[0],
        _ => {
            let candidate_names: Vec<String> = phn_candidates
                .iter()
                .map(|&idx| headers[idx].clone())
                .collect();
            return Err(DetectionError::AmbiguousPhnColumns {
                candidates: candidate_names,
            });
        }
    };

    let (mrp_status, mrp_updated) = if is_pas {
        (
            find_column(headers, STATUS_PATTERNS),
            find_column(headers, UPDATED_PATTERNS),
        )
    } else {
        (None, None)
    };

    Ok(ColumnMapping {
        phn,
        first_name: find_column(headers, FIRST_PATTERNS),
        last_name: find_column(headers, LAST_PATTERNS),
        dob: find_column(headers, DOB_PATTERNS),
        mrp_status,
        mrp_updated,
    })
}
```

- [ ] **Step 5: Expose the module in lib.rs**

In `crates/engine/src/lib.rs`, add:

```rust
pub mod detect;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --package pas-recon-engine --test detect_test`
Expected: all 7 tests PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/engine/src/detect.rs crates/engine/src/model.rs crates/engine/src/lib.rs crates/engine/tests/detect_test.rs
git commit -m "feat(engine): column auto-detection by header patterns"
```

---

## Task 5: Date parsing (MRP-updated dates)

**Files:**
- Create: `crates/engine/src/date.rs`
- Modify: `crates/engine/src/lib.rs` (add `pub mod date;`)
- Test: `crates/engine/tests/date_test.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/engine/tests/date_test.rs`:

```rust
use pas_recon_engine::date::parse_mrp_date;
use chrono::NaiveDate;

#[test]
fn parses_iso_date() {
    assert_eq!(
        parse_mrp_date("2024-03-15"),
        Some(NaiveDate::from_ymd_opt(2024, 3, 15))
    );
}

#[test]
fn parses_dmy_slash_date() {
    assert_eq!(
        parse_mrp_date("15/03/2024"),
        Some(NaiveDate::from_ymd_opt(2024, 3, 15))
    );
}

#[test]
fn parses_excel_serial_number() {
    // Excel serial 45366 = 2024-03-15
    assert_eq!(
        parse_mrp_date("45366"),
        Some(NaiveDate::from_ymd_opt(2024, 3, 15))
    );
}

#[test]
fn parses_actual_number_type() {
    assert_eq!(
        parse_mrp_date_f64(45366.0),
        Some(NaiveDate::from_ymd_opt(2024, 3, 15))
    );
}

#[test]
fn returns_none_for_garbage() {
    assert_eq!(parse_mrp_date("not a date"), None);
    assert_eq!(parse_mrp_date(""), None);
}

#[test]
fn returns_none_for_impossible_date() {
    assert_eq!(parse_mrp_date("31/02/2024"), None); // Feb 31 doesn't exist
}

fn parse_mrp_date_f64(serial: f64) -> Option<NaiveDate> {
    pas_recon_engine::date::serial_to_date(serial)
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package pas-recon-engine --test date_test`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement date.rs**

Create `crates/engine/src/date.rs`:

```rust
//! Date parsing for PAS MRP-updated dates.
//!
//! Handles three formats the spreadsheet dealt with:
//! - ISO: "2024-03-15"
//! - D/M/YYYY: "15/3/2024"
//! - Excel serial number: 45366 → 2024-03-15

use chrono::NaiveDate;

/// Excel epoch: December 30, 1899 (the Excel serial-day 0, accounting for
/// the 1900 leap-year bug in Excel's default 1900 date system).
const EXCEL_EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();

/// Convert an Excel serial date number to a NaiveDate.
pub fn serial_to_date(serial: f64) -> Option<NaiveDate> {
    if serial < 1.0 || serial > 100000.0 {
        return None; // sanity bounds
    }
    let days = serial.floor() as i64;
    EXCEL_EPOCH.checked_add_days(chrono::Days::new(days as u64))
}

/// Parse a date string that could be ISO, D/M/YYYY, or an Excel serial number.
/// Returns None if it can't be parsed (the caller treats this as "keep first seen").
pub fn parse_mrp_date(raw: &str) -> Option<NaiveDate> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try ISO first: YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(date);
    }

    // Try D/M/YYYY (the spreadsheet's format)
    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() == 3 {
            let day: u32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            let year: i32 = parts[2].parse().ok()?;
            return NaiveDate::from_ymd_opt(year, month, day);
        }
    }

    // Try Excel serial number (purely numeric)
    if trimmed.chars().all(|c| c.is_ascii_digit() || c == '.') {
        if let Ok(serial) = trimmed.parse::<f64>() {
            return serial_to_date(serial);
        }
    }

    None
}
```

- [ ] **Step 4: Expose the module in lib.rs**

In `crates/engine/src/lib.rs`, add:

```rust
pub mod date;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package pas-recon-engine --test date_test`
Expected: all 6 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/engine/src/date.rs crates/engine/src/lib.rs crates/engine/tests/date_test.rs
git commit -m "feat(engine): MRP date parsing (ISO, D/M/YYYY, Excel serial)"
```

---

# Phase 2: Engine Matching

## Task 6: PAS deduplication

**Files:**
- Create: `crates/engine/src/dedup.rs`
- Modify: `crates/engine/src/lib.rs` (add `pub mod dedup;`)
- Test: `crates/engine/tests/dedup_test.rs`

- [ ] **Step 1: Add DedupInput to model.rs**

Add to `crates/engine/src/model.rs`:

```rust
/// A validated PAS record ready for dedup + matching.
#[derive(Debug, Clone)]
pub struct PasRecord {
    pub phn: String,
    pub mrp_status: Option<String>,
    pub mrp_updated: Option<chrono::NaiveDate>,
    pub raw_fields: Vec<String>,
    pub row_index: usize,
}

/// A validated EMR record ready for matching.
#[derive(Debug, Clone)]
pub struct EmrRecord {
    pub phn: String,
    pub raw_fields: Vec<String>,
    pub row_index: usize,
}
```

- [ ] **Step 2: Write the failing test**

Create `crates/engine/tests/dedup_test.rs`:

```rust
use pas_recon_engine::dedup::deduplicate_pas;
use pas_recon_engine::model::PasRecord;
use chrono::NaiveDate;

fn pas(phn: &str, date: Option<NaiveDate>, row: usize) -> PasRecord {
    PasRecord {
        phn: phn.to_string(),
        mrp_status: Some("Confirmed".to_string()),
        mrp_updated: date,
        raw_fields: vec![],
        row_index: row,
    }
}

#[test]
fn keeps_single_record_unchanged() {
    let records = vec![pas("9876543210", None, 0)];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(dropped, 0);
}

#[test]
fn keeps_newest_when_duplicates_have_dates() {
    let records = vec![
        pas("9876543210", NaiveDate::from_ymd_opt(2023, 1, 1), 0),
        pas("9876543210", NaiveDate::from_ymd_opt(2024, 6, 15), 1), // newest
        pas("9876543210", NaiveDate::from_ymd_opt(2023, 9, 3), 2),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].row_index, 1); // the newest one
    assert_eq!(dropped, 2);
}

#[test]
fn keeps_first_seen_when_dates_are_equal() {
    let date = NaiveDate::from_ymd_opt(2024, 1, 1);
    let records = vec![
        pas("9876543210", date, 0),
        pas("9876543210", date, 1),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].row_index, 0); // first seen
    assert_eq!(dropped, 1);
}

#[test]
fn keeps_first_seen_when_no_dates_present() {
    let records = vec![
        pas("9876543210", None, 0),
        pas("9876543210", None, 1),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].row_index, 0);
    assert_eq!(dropped, 1);
}

#[test]
fn preserves_distinct_phns() {
    let records = vec![
        pas("9876543210", None, 0),
        pas("9111222333", None, 1),
        pas("9222333444", None, 2),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 3);
    assert_eq!(dropped, 0);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --package pas-recon-engine --test dedup_test`
Expected: FAIL — module not found.

- [ ] **Step 4: Implement dedup.rs**

Create `crates/engine/src/dedup.rs`:

```rust
//! PAS deduplication by PHN, keeping the record with the latest MRP-updated date.

use std::collections::HashMap;

use crate::model::PasRecord;

/// Deduplicate PAS records by PHN. For each group of duplicates, keep only
/// the record whose `mrp_updated` date is latest. Ties keep the first-seen.
/// Records with no date sort to the bottom of their group.
///
/// Returns (kept_records, duplicates_dropped_count). Kept records preserve
/// their original relative order.
pub fn deduplicate_pas(records: Vec<PasRecord>) -> (Vec<PasRecord>, usize) {
    // Group indices by PHN
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, rec) in records.iter().enumerate() {
        groups.entry(rec.phn.clone()).or_default().push(idx);
    }

    let mut keep_indices: Vec<usize> = Vec::new();
    let mut dropped = 0usize;

    for (_, indices) in &groups {
        if indices.len() == 1 {
            keep_indices.push(indices[0]);
        } else {
            // Find the index of the newest record (or first-seen on tie)
            let winner = indices
                .iter()
                .copied()
                .reduce(|best, candidate| {
                    let best_date = records[best].mrp_updated;
                    let cand_date = records[candidate].mrp_updated;
                    match (best_date, cand_date) {
                        (Some(b), Some(c)) => {
                            if c > b { candidate } else { best }
                        }
                        (None, Some(_)) => candidate,
                        _ => best, // both None or best is Some → keep best (first-seen)
                    }
                })
                .unwrap();

            keep_indices.push(winner);
            dropped += indices.len() - 1;
        }
    }

    // Sort keep_indices to preserve original row order
    keep_indices.sort_unstable();

    let kept = keep_indices
        .into_iter()
        .map(|idx| records[idx].clone())
        .collect();

    (kept, dropped)
}
```

- [ ] **Step 5: Expose the module in lib.rs**

In `crates/engine/src/lib.rs`, add:

```rust
pub mod dedup;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --package pas-recon-engine --test dedup_test`
Expected: all 5 tests PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/engine/src/dedup.rs crates/engine/src/model.rs crates/engine/src/lib.rs crates/engine/tests/dedup_test.rs
git commit -m "feat(engine): PAS deduplication by latest MRP-updated date"
```

---

## Task 7: Full reconciliation pipeline

**Files:**
- Create: `crates/engine/src/reconcile.rs`
- Modify: `crates/engine/src/lib.rs` (wire up `reconcile()` and `reconcile_with_columns()`)
- Test: `crates/engine/tests/reconcile_test.rs`
- Create: `crates/engine/fixtures/pas_basic.csv`
- Create: `crates/engine/fixtures/emr_dirty.csv`
- Create: `crates/engine/fixtures/pas_duplicates.csv`

- [ ] **Step 1: Create the PAS fixture**

Create `crates/engine/fixtures/pas_basic.csv`:

```csv
PHN,First Name,Last Name,DOB,MRP Status,MRP Updated
9876543210,John,Smith,1965-03-12,Confirmed,15/03/2024
9871111222,Mary,Jones,1978-11-04,Confirmed,20/01/2024
9873333444,Robert,Lee,1952-07-22,Pending,10/06/2024
9875555666,Susan,Wong,1990-01-30,Not the MRP,05/05/2024
9877777888,David,Brown,1972-09-15,Deceased,01/03/2024
9888888999,Sarah,Johnson,1985-04-18,Confirmed,12/02/2024
9899999000,James,Wilson,1960-08-25,Confirmed,18/07/2024
```

This is designed to test against `emr_basic.csv` (Task 2):
- 9876543210, 9871111222, 9873333444 → in both → matched (9873333444 is Pending → list 5)
- 9875555666 → not in EMR → list 6
- 9877777888 → not in EMR → list 6 (Deceased)
- 9888888999, 9899999000 → not in EMR → list 6

- [ ] **Step 2: Write the failing integration test**

Create `crates/engine/tests/reconcile_test.rs`:

```rust
use pas_recon_engine::{reconcile, model::*};

fn read_fixture(name: &str) -> Vec<u8> {
    let path = format!("crates/engine/fixtures/{name}");
    std::fs::read(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"))
}

#[test]
fn reconciles_basic_emr_and_pas() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    // EMR patients: 9876543210, 9871111222, 9873333444
    // PAS patients: above + 9875555666, 9877777888, 9888888999, 9899999000
    //
    // Matched (in both): 9876543210 (Confirmed), 9871111222 (Confirmed),
    //   9873333444 (Pending → review)
    assert_eq!(result.summary.matched, 3);
    assert_eq!(result.summary.emr_only, 0); // all EMR patients are in PAS
    assert_eq!(result.summary.pas_only, 4);
    assert_eq!(result.summary.pas_review, 1); // 9873333444 Pending
}

#[test]
fn pas_review_list_contains_pending_deceased_removed_not_mrp() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    // 9873333444 is Pending and matched → should be in pas_match_review
    let phns: Vec<&str> = result.pas_match_review.iter().map(|r| r.phn.as_str()).collect();
    assert!(phns.contains(&"9873333444"), "Pending patient should be in review list, got {phns:?}");
}

#[test]
fn pas_no_match_list_contains_pas_only_patients() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    let phns: Vec<&str> = result.pas_no_match.iter().map(|r| r.phn.as_str()).collect();
    assert!(phns.contains(&"9888888999"));
    assert!(phns.contains(&"9899999000"));
}

#[test]
fn status_breakdown_counts_correctly() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    assert_eq!(result.summary.status_breakdown.confirmed, 4);
    assert_eq!(result.summary.status_breakdown.pending, 1);
    assert_eq!(result.summary.status_breakdown.deceased, 1);
    assert_eq!(result.summary.status_breakdown.removed, 0);
    assert_eq!(result.summary.status_breakdown.not_the_mrp, 1);
}

#[test]
fn rejects_empty_emr_file() {
    let result = reconcile(b"", b"PHN\n9876543210\n");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EngineError::Io { .. }));
}

#[test]
fn rejects_emr_without_phn_column() {
    let result = reconcile(b"Name,Age\nJohn,30\n", b"PHN\n9876543210\n");
    assert!(matches!(result.unwrap_err(), EngineError::MissingPhnColumn { .. }));
}

#[test]
fn lists_sorted_by_last_name() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    let last_names: Vec<&str> = result.pas_no_match.iter().map(|r| r.last_name.as_deref().unwrap_or("")).collect();
    // pas_no_match should contain Brown, Johnson, Wilson, Wong (sorted)
    let mut expected = last_names.to_vec();
    expected.sort();
    assert_eq!(last_names, expected, "List should be sorted by last name");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --package pas-recon-engine --test reconcile_test`
Expected: FAIL — `reconcile()` panics with `todo!()`.

- [ ] **Step 4: Implement reconcile.rs**

Create `crates/engine/src/reconcile.rs`:

```rust
//! Full reconciliation pipeline: parse → detect → validate → dedup → match → classify.

use std::collections::HashSet;

use crate::{
    date::parse_mrp_date,
    dedup::deduplicate_pas,
    detect::{detect_columns, DetectionError},
    model::*,
    parse::parse_csv,
    phn,
};

/// Which status values put a matched PAS patient on the "review" list.
const REVIEW_STATUSES: &[&str] = &["pending", "not the mrp", "deceased", "removed"];

/// Parse and validate EMR records from a parsed CSV using a column mapping.
fn build_emr_records(
    parsed: &crate::parse::ParsedCsv,
    mapping: &ColumnMapping,
) -> (Vec<EmrRecord>, usize) {
    let mut records = Vec::new();
    let mut invalid = 0usize;

    for row in &parsed.rows {
        let raw_phn = row.fields.get(mapping.phn).map(|s| s.as_str()).unwrap_or("");
        let normalized = phn::normalize_phn(raw_phn);

        if !phn::is_valid_bc_phn(raw_phn) {
            invalid += 1;
            continue;
        }

        records.push(EmrRecord {
            phn: normalized,
            raw_fields: row.fields.clone(),
            row_index: row.row_index,
        });
    }

    (records, invalid)
}

/// Parse and validate PAS records from a parsed CSV using a column mapping.
fn build_pas_records(
    parsed: &crate::parse::ParsedCsv,
    mapping: &ColumnMapping,
) -> (Vec<PasRecord>, usize, usize) {
    let mut records = Vec::new();
    let mut invalid = 0usize;
    let mut bad_dates = 0usize;

    for row in &parsed.rows {
        let raw_phn = row.fields.get(mapping.phn).map(|s| s.as_str()).unwrap_or("");

        if !phn::is_valid_bc_phn(raw_phn) {
            invalid += 1;
            continue;
        }

        let normalized = phn::normalize_phn(raw_phn);

        let mrp_status = mapping
            .mrp_status
            .and_then(|idx| row.fields.get(idx))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let (mrp_updated, had_unparseable) = match mapping.mrp_updated {
            Some(idx) => {
                let raw = row.fields.get(idx).map(|s| s.as_str()).unwrap_or("");
                if raw.trim().is_empty() {
                    (None, false)
                } else {
                    match parse_mrp_date(raw) {
                        Some(date) => (Some(date), false),
                        None => (None, true),
                    }
                }
            }
            None => (None, false),
        };

        if had_unparseable {
            bad_dates += 1;
        }

        records.push(PasRecord {
            phn: normalized,
            mrp_status,
            mrp_updated,
            raw_fields: row.fields.clone(),
            row_index: row.row_index,
        });
    }

    (records, invalid, bad_dates)
}

/// Build a DisplayRow from an EMR record + column mapping.
fn emr_to_display(record: &EmrRecord, mapping: &ColumnMapping) -> DisplayRow {
    let get = |idx: Option<usize>| -> Option<String> {
        idx.and_then(|i| record.raw_fields.get(i))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    DisplayRow {
        phn: record.phn.clone(),
        first_name: get(mapping.first_name),
        last_name: get(mapping.last_name),
        dob: get(mapping.dob),
        mrp_status: None,
        raw_fields: record.raw_fields.clone(),
    }
}

/// Build a DisplayRow from a PAS record + column mapping.
fn pas_to_display(record: &PasRecord, mapping: &ColumnMapping) -> DisplayRow {
    let get = |idx: Option<usize>| -> Option<String> {
        idx.and_then(|i| record.raw_fields.get(i))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    DisplayRow {
        phn: record.phn.clone(),
        first_name: get(mapping.first_name),
        last_name: get(mapping.last_name),
        dob: get(mapping.dob),
        mrp_status: record.mrp_status.clone(),
        raw_fields: record.raw_fields.clone(),
    }
}

fn map_detection_error(err: DetectionError, source: CsvSource) -> EngineError {
    match err {
        DetectionError::MissingPhnColumn => EngineError::MissingPhnColumn {
            source: source.to_string(),
        },
        DetectionError::AmbiguousPhnColumns { candidates } => EngineError::AmbiguousPhnColumns {
            source: source.to_string(),
            candidates,
        },
    }
}

fn map_parse_error(err: crate::parse::ParseError, source: CsvSource) -> EngineError {
    match err {
        crate::parse::ParseError::Empty => EngineError::Io {
            source: source.to_string(),
            message: "file is empty".to_string(),
        },
        crate::parse::ParseError::HeaderOnly => EngineError::Io {
            source: source.to_string(),
            message: "file has a header row but no data rows".to_string(),
        },
        crate::parse::ParseError::Read { line, message } => EngineError::CsvParse {
            source: source.to_string(),
            row: line,
            message,
        },
    }
}

/// Run the full pipeline. `emr_phn_column` / `pas_phn_column` override auto-detection
/// when set (used by the manual column-picker fallback).
pub fn reconcile_with_columns(
    emr_csv: &[u8],
    pas_csv: &[u8],
    emr_phn_override: Option<usize>,
    pas_phn_override: Option<usize>,
) -> Result<ReconciliationResult, EngineError> {
    // --- Parse ---
    let emr_parsed = parse_csv(emr_csv).map_err(|e| map_parse_error(e, CsvSource::Emr))?;
    let pas_parsed = parse_csv(pas_csv).map_err(|e| map_parse_error(e, CsvSource::Pas))?;

    // --- Detect columns ---
    let emr_mapping = if let Some(phn_idx) = emr_phn_override {
        detect_columns(&emr_parsed.headers, false)
            .map(|mut m| { m.phn = phn_idx; m })
            .map_err(|e| map_detection_error(e, CsvSource::Emr))?
    } else {
        detect_columns(&emr_parsed.headers, false)
            .map_err(|e| map_detection_error(e, CsvSource::Emr))?
    };

    let pas_mapping = if let Some(phn_idx) = pas_phn_override {
        detect_columns(&pas_parsed.headers, true)
            .map(|mut m| { m.phn = phn_idx; m })
            .map_err(|e| map_detection_error(e, CsvSource::Pas))?
    } else {
        detect_columns(&pas_parsed.headers, true)
            .map_err(|e| map_detection_error(e, CsvSource::Pas))?
    };

    // --- Build records + validate PHNs ---
    let (emr_records, emr_invalid) = build_emr_records(&emr_parsed, &emr_mapping);
    let (pas_records, pas_invalid, bad_dates) = build_pas_records(&pas_parsed, &pas_mapping);

    // --- Dedup PAS ---
    let (pas_records, duplicates_dropped) = deduplicate_pas(pas_records);

    // --- Build PHN sets ---
    let emr_phns: HashSet<&String> = emr_records.iter().map(|r| &r.phn).collect();
    let pas_phns: HashSet<&String> = pas_records.iter().map(|r| &r.phn).collect();

    // --- Classify ---
    let mut emr_no_match = Vec::new();
    let mut pas_match_review = Vec::new();
    let mut pas_no_match = Vec::new();
    let mut matched = 0usize;

    let mut status_breakdown = StatusBreakdown::default();

    for pas_rec in &pas_records {
        // Tally status breakdown for ALL pas records (deduped)
        if let Some(status) = &pas_rec.mrp_status {
            match status.to_lowercase().as_str() {
                "confirmed" => status_breakdown.confirmed += 1,
                "pending" => status_breakdown.pending += 1,
                "deceased" => status_breakdown.deceased += 1,
                "removed" => status_breakdown.removed += 1,
                "not the mrp" => status_breakdown.not_the_mrp += 1,
                _ => {}
            }
        }

        if emr_phns.contains(&pas_rec.phn) {
            // Matched
            matched += 1;
            let is_review = pas_rec
                .mrp_status
                .as_deref()
                .map(|s| {
                    let lower = s.to_lowercase();
                    REVIEW_STATUSES.contains(&lower.as_str())
                })
                .unwrap_or(false);

            if is_review {
                pas_match_review.push(pas_to_display(pas_rec, &pas_mapping));
            }
            // Confirmed or unset → matched OK, not listed
        } else {
            // PAS only
            pas_no_match.push(pas_to_display(pas_rec, &pas_mapping));
        }
    }

    for emr_rec in &emr_records {
        if !pas_phns.contains(&emr_rec.phn) {
            emr_no_match.push(emr_to_display(emr_rec, &emr_mapping));
        }
    }

    // --- Sort lists by last name, then first name ---
    let sort_fn = |a: &DisplayRow, b: &DisplayRow| {
        let a_last = a.last_name.as_deref().unwrap_or("").to_lowercase();
        let b_last = b.last_name.as_deref().unwrap_or("").to_lowercase();
        let a_first = a.first_name.as_deref().unwrap_or("").to_lowercase();
        let b_first = b.first_name.as_deref().unwrap_or("").to_lowercase();
        a_last.cmp(&b_last).then_with(|| a_first.cmp(&b_first))
    };
    emr_no_match.sort_by(sort_fn);
    pas_match_review.sort_by(sort_fn);
    pas_no_match.sort_by(sort_fn);

    let summary = Summary {
        matched,
        emr_only: emr_no_match.len(),
        pas_only: pas_no_match.len(),
        pas_review: pas_match_review.len(),
        status_breakdown,
        duplicates_dropped,
        invalid_phn_skipped: emr_invalid + pas_invalid,
        unparseable_dates: bad_dates,
    };

    Ok(ReconciliationResult {
        summary,
        emr_no_match,
        pas_match_review,
        pas_no_match,
    })
}
```

- [ ] **Step 5: Wire up lib.rs**

Replace the `reconcile` function and add the re-export in `crates/engine/src/lib.rs`:

```rust
//! PAS Reconciliation Engine — pure Rust reconciliation logic.

pub mod date;
pub mod dedup;
pub mod detect;
pub mod model;
pub mod parse;
pub mod phn;
pub mod reconcile;

pub use model::{ReconciliationResult, EngineError};

/// Run reconciliation with auto-detected columns.
pub fn reconcile(emr_csv: &[u8], pas_csv: &[u8]) -> Result<ReconciliationResult, EngineError> {
    reconcile::reconcile_with_columns(emr_csv, pas_csv, None, None)
}

/// Run reconciliation with caller-provided PHN column overrides.
pub fn reconcile_with_columns(
    emr_csv: &[u8],
    pas_csv: &[u8],
    emr_phn_column: Option<usize>,
    pas_phn_column: Option<usize>,
) -> Result<ReconciliationResult, EngineError> {
    reconcile::reconcile_with_columns(emr_csv, pas_csv, emr_phn_column, pas_phn_column)
}
```

- [ ] **Step 6: Run all engine tests**

Run: `cargo test --package pas-recon-engine`
Expected: all tests across parse, phn, detect, date, dedup, reconcile PASS.

- [ ] **Step 7: Create the dirty fixtures for later edge-case tests**

Create `crates/engine/fixtures/emr_dirty.csv`:

```csv
PHN,First Name,Last Name,DOB
9876543210,John,Smith,1965-03-12
1234567890,Bad,Phn,1990-01-01
9876 543 210,Spaced,Phn,1980-05-20
```
,Last,DOB
9871111222,Short,Row,1978-11-04
```
(Note: this file has a deliberately mangled row to test parsing robustness.)

Create `crates/engine/fixtures/pas_duplicates.csv`:

```csv
PHN,First Name,Last Name,MRP Status,MRP Updated
9876543210,John,Smith,Confirmed,15/03/2024
9876543210,John,Smith,Confirmed,20/06/2024
9876543210,John,Smith,Confirmed,10/01/2024
9871111222,Mary,Jones,Pending,01/05/2024
9871111222,Mary,Jones,Confirmed,01/06/2024
```

- [ ] **Step 8: Commit**

```bash
git add crates/engine/
git commit -m "feat(engine): full reconciliation pipeline with matching and classification"
```

---

# Phase 3: Tauri Shell

## Task 8: Scaffold the Tauri app crate

**Files:**
- Modify: `Cargo.toml` (workspace — add app member)
- Create: `crates/app/Cargo.toml`
- Create: `crates/app/tauri.conf.json`
- Create: `crates/app/build.rs`
- Create: `crates/app/src/main.rs`

This task sets up a minimal Tauri app that compiles and shows an empty window. The frontend comes in Phase 4; for now we point Tauri at a stub HTML.

- [ ] **Step 1: Add the app crate to the workspace**

In root `Cargo.toml`, update `[workspace]`:

```toml
[workspace]
members = ["crates/engine", "crates/app"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
```

- [ ] **Step 2: Create crates/app/Cargo.toml**

```toml
[package]
name = "pas-recon-app"
version.workspace = true
edition.workspace = true
license.workspace = true

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-updater = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
pas-recon-engine = { path = "../engine" }
```

- [ ] **Step 3: Create crates/app/build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 4: Create a minimal tauri.conf.json**

Create `crates/app/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "PAS Reconciliation",
  "version": "0.1.0",
  "identifier": "ca.doctorsofbc.pasrecon",
  "build": {
    "frontendDist": "../frontend/dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "cd ../frontend && npm run dev",
    "beforeBuildCommand": "cd ../frontend && npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "PAS Reconciliation",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "fileDropEnabled": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; connect-src 'self' https://github.com https://api.github.com https://*.githubusercontent.com; img-src 'self' data:; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/<owner>/pas-recon/releases/latest/download/latest.json"
      ],
      "pubkey": "PLACEHOLDER_REPLACE_WITH_GENERATED_PUBKEY"
    }
  }
}
```

- [ ] **Step 5: Create a stub frontend dir so cargo check works**

We'll build the real frontend in Phase 4. For now create `frontend/dist/index.html` as a placeholder so Tauri has something to bundle:

```html
<!DOCTYPE html>
<html>
<head><title>PAS Reconciliation</title></head>
<body><p>Loading…</p></body>
</html>
```

- [ ] **Step 6: Create crates/app/src/main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod update;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::reconcile_files,
            commands::reconcile_with_column_override,
            commands::export_list,
            commands::check_for_updates,
        ])
        .setup(|app| {
            // Check for updates 3s after launch (non-blocking)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = update::check_and_notify(&handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Create stub command and update modules**

Create `crates/app/src/commands.rs`:

```rust
//! Tauri commands exposed to the frontend via IPC.
//!
//! These are the bridge between the webview UI and the Rust engine.

use std::path::PathBuf;
use pas_recon_engine::{self, model::{ReconciliationResult, EngineError}};

/// Which list to export.
#[derive(Debug, Clone, serde::Deserialize)]
pub enum ListKind {
    EmrNoMatch,
    PasMatchReview,
    PasNoMatch,
}

/// Read two CSV files from disk and run reconciliation.
/// Auto-detects the PHN column in each.
#[tauri::command]
pub fn reconcile_files(emr_path: String, pas_path: String) -> Result<ReconciliationResult, EngineError> {
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

    pas_recon_engine::reconcile_with_columns(&emr_bytes, &pas_bytes, emr_phn_column, pas_phn_column)
}

/// Export one of the three lists to a CSV file at the given path.
#[tauri::command]
pub fn export_list(
    list: ListKind,
    rows: Vec<pas_recon_engine::model::DisplayRow>,
    path: String,
) -> Result<(), String> {
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

/// Check GitHub Releases for a newer version. Returns Some(info) if an update exists.
#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<Option<crate::update::UpdateInfo>, String> {
    crate::update::check_and_fetch(&app).await
}
```

Create `crates/app/src/update.rs`:

```rust
//! Auto-update wiring using tauri-plugin-updater.

use tauri::AppHandle;

/// Simplified update info passed to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
}

/// Check for updates, return info if one exists. Does NOT apply it.
pub async fn check_and_fetch(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = tauri_plugin_updater::UpdaterExt::updater(app)
        .map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            current_version: app.package_info().version.to_string(),
        })),
        Ok(None) => Ok(None),
        Err(e) => {
            eprintln!("Update check failed: {e}");
            Ok(None) // never block the UI on update-check errors
        }
    }
}

/// Check for updates and emit an event to the frontend if one is available.
/// Called on a timer after launch. Non-blocking; errors are swallowed.
pub async fn check_and_notify(app: &AppHandle) -> Result<(), String> {
    if let Some(info) = check_and_fetch(app).await? {
        tauri::Manager::emit(app, "update-available", &info)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

- [ ] **Step 8: Add csv dependency to app crate**

In `crates/app/Cargo.toml`, add to `[dependencies]`:

```toml
csv = "1.3"
tokio = { version = "1", features = ["time"] }
```

- [ ] **Step 9: Verify the workspace compiles**

Run: `cargo check`
Expected: compiles with no errors. (Tauri will warn about missing icons; we'll generate those in Phase 5.)

- [ ] **Step 10: Generate placeholder icons**

Run:
```bash
mkdir -p crates/app/icons
# Tauri provides a CLI icon generator from a single source PNG.
# If tauri CLI is installed:
# cargo tauri icon path/to/source-icon.png -o crates/app/icons
# For now, create minimal placeholder PNGs so the build doesn't fail:
```

Create minimal placeholder icons using any 1024×1024 PNG. If you don't have one, generate a solid-color PNG:

```bash
# Create a 1024x1024 solid blue PNG as a placeholder source icon
python3 -c "
from struct import pack
import zlib
width, height = 1024, 1024
raw = b''
for y in range(height):
    raw += b'\x00'  # filter byte
    for x in range(width):
        raw += b'\x1a\x1a\x4e\xff'  # RGBA blue
def chunk(ctype, data):
    c = ctype + data
    return pack('>I', len(data)) + c + pack('>I', zlib.crc32(c) & 0xffffffff)
png = b'\x89PNG\r\n\x1a\n'
png += chunk(b'IHDR', pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
png += chunk(b'IDAT', zlib.compress(raw, 9))
png += chunk(b'IEND', b'')
open('crates/app/icons/source.png','wb').write(png)
"
```

Then generate all required icon formats from it:
```bash
cargo install tauri-cli --version "^2.0.0-rc" || true
cargo tauri icon crates/app/icons/source.png -o crates/app/icons 2>&1 || echo "If tauri CLI isn't available, manually place: 32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico"
```

- [ ] **Step 11: Commit**

```bash
git add Cargo.toml crates/app/ frontend/dist/
git commit -m "feat(app): scaffold Tauri shell with commands, updater wiring, placeholder icons"
```

---

# Phase 4: Frontend

## Task 9: Scaffold the React + TypeScript frontend

**Files:**
- Create: `frontend/package.json`
- Create: `frontend/vite.config.ts`
- Create: `frontend/tsconfig.json`
- Create: `frontend/index.html`
- Create: `frontend/src/main.tsx`
- Create: `frontend/src/App.tsx` (stub)
- Create: `frontend/src/types.ts`
- Create: `frontend/src/api.ts`

- [ ] **Step 1: Create frontend/package.json**

```json
{
  "name": "pas-recon-frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-updater": "^2.0.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  }
}
```

- [ ] **Step 2: Create frontend/vite.config.ts**

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2021",
    outDir: "dist",
  },
});
```

- [ ] **Step 3: Create frontend/tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2021",
    "useDefineForClassFields": true,
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": false,
    "noUnusedParameters": false
  },
  "include": ["src"]
}
```

- [ ] **Step 4: Create frontend/index.html**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PAS Reconciliation</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 5: Create frontend/src/types.ts**

Mirror the engine's model in TypeScript:

```typescript
export interface ReconciliationResult {
  summary: Summary;
  emr_no_match: DisplayRow[];
  pas_match_review: DisplayRow[];
  pas_no_match: DisplayRow[];
}

export interface Summary {
  matched: number;
  emr_only: number;
  pas_only: number;
  pas_review: number;
  status_breakdown: StatusBreakdown;
  duplicates_dropped: number;
  invalid_phn_skipped: number;
  unparseable_dates: number;
}

export interface StatusBreakdown {
  confirmed: number;
  pending: number;
  deceased: number;
  removed: number;
  not_the_mrp: number;
}

export interface DisplayRow {
  phn: string;
  first_name: string | null;
  last_name: string | null;
  dob: string | null;
  mrp_status: string | null;
  raw_fields: string[];
}

export interface UpdateInfo {
  version: string;
  current_version: string;
}

export type ListKey = "emr_no_match" | "pas_match_review" | "pas_no_match";

export interface EngineError {
  Io?: { source: string; message: string };
  CsvParse?: { source: string; row: number; message: string };
  MissingPhnColumn?: { source: string };
  AmbiguousPhnColumns?: { source: string; candidates: string[] };
}
```

- [ ] **Step 6: Create frontend/src/api.ts**

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ReconciliationResult, UpdateInfo, DisplayRow, ListKey } from "./types";

export async function reconcileFiles(emrPath: string, pasPath: string): Promise<ReconciliationResult> {
  return invoke<ReconciliationResult>("reconcile_files", {
    emrPath,
    pasPath,
  });
}

export async function reconcileWithColumnOverride(
  emrPath: string,
  pasPath: string,
  emrPhnColumn: number | null,
  pasPhnColumn: number | null,
): Promise<ReconciliationResult> {
  return invoke<ReconciliationResult>("reconcile_with_column_override", {
    emrPath,
    pasPath,
    emrPhnColumn,
    pasPhnColumn,
  });
}

export async function exportList(list: ListKey, rows: DisplayRow[], path: string): Promise<void> {
  const listKind = list === "emr_no_match" ? "EmrNoMatch"
    : list === "pas_match_review" ? "PasMatchReview"
    : "PasNoMatch";
  await invoke("export_list", { list: listKind, rows, path });
}

export async function checkForUpdates(): Promise<UpdateInfo | null> {
  return invoke<UpdateInfo | null>("check_for_updates");
}

export function onUpdateAvailable(callback: (info: UpdateInfo) => void) {
  return listen<UpdateInfo>("update-available", (event) => {
    callback(event.payload);
  });
}
```

- [ ] **Step 7: Create frontend/src/main.tsx**

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/app.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

- [ ] **Step 8: Create a stub App.tsx**

```tsx
import { useState, useEffect } from "react";
import type { ReconciliationResult, UpdateInfo } from "./types";
import { checkForUpdates, onUpdateAvailable } from "./api";

export default function App() {
  const [result, setResult] = useState<ReconciliationResult | null>(null);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    const unlisten = onUpdateAvailable((info) => setUpdate(info));
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  return (
    <div className="app">
      <aside className="sidebar">
        <h1>PAS Reconciliation</h1>
        <p className="version">v0.1.0</p>
        {/* DropZone, Summary, StatusBreakdown added in Task 10 */}
        <p>Frontend scaffold ready.</p>
      </aside>
      <main className="main-panel">
        {update && (
          <div className="update-toast">
            Update available — v{update.version}
          </div>
        )}
        <p>Drop CSV files to begin.</p>
      </main>
    </div>
  );
}
```

- [ ] **Step 9: Create frontend/src/styles/app.css**

```css
:root {
  --bg: #0f0f17;
  --sidebar-bg: #161623;
  --border: #2a2a3a;
  --text: #e5e7eb;
  --text-dim: #9ca3af;
  --text-faint: #6b7280;
  --green: #4ade80;
  --red: #f87171;
  --amber: #fbbf24;
  --purple: #a78bfa;
  --blue: #3b82f6;
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: system-ui, -apple-system, sans-serif;
  background: var(--bg);
  color: var(--text);
  font-size: 14px;
}

.app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.sidebar {
  width: 280px;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}

.sidebar h1 {
  font-size: 14px;
  font-weight: 700;
}

.sidebar .version {
  font-size: 9px;
  color: var(--text-faint);
}

.main-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.update-toast {
  background: var(--blue);
  color: white;
  padding: 8px 16px;
  font-size: 12px;
}

.drop-zone {
  border: 2px dashed #3a3a4a;
  border-radius: 8px;
  padding: 14px;
  text-align: center;
  background: #1c1c2e;
}

.drop-zone-label {
  font-size: 9px;
  color: var(--text-faint);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.drop-zone-file {
  font-size: 9px;
  color: var(--text-dim);
}

.summary-section, .status-section {
  font-size: 10px;
}

.section-label {
  font-size: 8px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 6px;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  padding: 2px 0;
}

.list-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--sidebar-bg);
}

.tab {
  background: #2a2a3a;
  color: var(--text-dim);
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 10px;
  cursor: pointer;
  border: none;
}

.tab.active {
  background: var(--blue);
  color: white;
  font-weight: 600;
}

.tab-badge {
  margin-left: 4px;
  opacity: 0.7;
}

.context-line {
  padding: 8px 16px;
  font-size: 9px;
  color: var(--text-faint);
  border-bottom: 1px solid #1c1c2e;
}

.toolbar {
  padding: 8px 16px;
  display: flex;
  gap: 8px;
  align-items: center;
}

.search-input {
  flex: 1;
  background: #1c1c2e;
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 10px;
  padding: 4px 8px;
  border-radius: 4px;
}

.export-btn {
  background: var(--blue);
  color: white;
  border: none;
  font-size: 9px;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
}

table {
  width: 100%;
  border-collapse: collapse;
}

thead {
  position: sticky;
  top: 0;
  background: var(--bg);
  z-index: 1;
}

th {
  padding: 6px 8px;
  color: var(--text-faint);
  font-weight: 500;
  text-align: left;
  font-size: 9px;
  border-bottom: 1px solid var(--border);
}

td {
  padding: 6px 8px;
  font-size: 9px;
  color: var(--text-dim);
  border-bottom: 1px solid #1c1c2e;
}

td.phn {
  color: var(--red);
}

tr.resolved td {
  background: rgba(250, 204, 21, 0.1);
}

.status-bar {
  padding: 6px 16px;
  border-top: 1px solid var(--border);
  background: var(--sidebar-bg);
  font-size: 8px;
  color: #4b5563;
  display: flex;
  justify-content: space-between;
}

.privacy-note {
  font-size: 8px;
  color: #4b5563;
  border-top: 1px solid var(--border);
  padding-top: 8px;
  margin-top: auto;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-faint);
}

.error-banner {
  background: var(--red);
  color: white;
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 10px;
  margin: 8px 0;
}
```

- [ ] **Step 10: Install frontend dependencies**

Run:
```bash
cd frontend && npm install
```
Expected: `node_modules` created, no errors.

- [ ] **Step 11: Verify the frontend builds**

Run:
```bash
cd frontend && npm run build
```
Expected: `dist/` directory created with `index.html` and bundled JS/CSS.

- [ ] **Step 12: Commit**

```bash
git add frontend/
git commit -m "feat(frontend): scaffold React + TypeScript + Vite SPA with types and API layer"
```

---

## Task 10: Build the drop zone, sidebar, and table components

**Files:**
- Create: `frontend/src/components/DropZone.tsx`
- Create: `frontend/src/components/Sidebar.tsx`
- Create: `frontend/src/components/ListTabs.tsx`
- Create: `frontend/src/components/PatientTable.tsx`
- Create: `frontend/src/components/SummaryCard.tsx`
- Create: `frontend/src/components/StatusBreakdown.tsx`
- Create: `frontend/src/components/EmptyState.tsx`
- Create: `frontend/src/components/UpdateToast.tsx`
- Modify: `frontend/src/App.tsx` (wire everything together)

- [ ] **Step 1: Create DropZone.tsx**

```tsx
import { useState } from "react";

interface DropZoneProps {
  onFilesDropped: (files: File[]) => void;
  emrLoaded: boolean;
  pasLoaded: boolean;
  error: string | null;
}

export default function DropZone({ onFilesDropped, emrLoaded, pasLoaded, error }: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) onFilesDropped(files);
  };

  return (
    <div
      className="drop-zone"
      style={isDragging ? { borderColor: "var(--blue)" } : undefined}
      onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
      onDragLeave={() => setIsDragging(false)}
      onDrop={handleDrop}
    >
      <div className="drop-zone-label">Drop CSV Files Here</div>
      <div style={{ display: "flex", flexDirection: "column", gap: "4px", marginTop: "6px" }}>
        <div className="drop-zone-file">
          {emrLoaded ? "✓" : "○"} EMR Active Patient List.csv
        </div>
        <div className="drop-zone-file">
          {pasLoaded ? "✓" : "○"} PAS Patient List.csv
        </div>
      </div>
      {error && (
        <div className="error-banner" style={{ marginTop: "8px" }}>{error}</div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Create SummaryCard.tsx**

```tsx
import type { Summary } from "../types";

interface SummaryCardProps {
  summary: Summary;
}

export default function SummaryCard({ summary }: SummaryCardProps) {
  return (
    <div className="summary-section">
      <div className="section-label">Summary</div>
      <div className="summary-row">
        <span style={{ color: "var(--green)" }}>● Matched</span>
        <strong>{summary.matched}</strong>
      </div>
      <div className="summary-row">
        <span style={{ color: "var(--red)" }}>● EMR only</span>
        <strong>{summary.emr_only}</strong>
      </div>
      <div className="summary-row">
        <span style={{ color: "var(--amber)" }}>● PAS only</span>
        <strong>{summary.pas_only}</strong>
      </div>
      <div className="summary-row">
        <span style={{ color: "var(--purple)" }}>● Review</span>
        <strong>{summary.pas_review}</strong>
      </div>
      {(summary.duplicates_dropped > 0 || summary.invalid_phn_skipped > 0) && (
        <div style={{ marginTop: "6px", fontSize: "8px", color: "var(--text-faint)" }}>
          ⚠ {summary.duplicates_dropped} duplicates dropped, {summary.invalid_phn_skipped} invalid PHNs skipped
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Create StatusBreakdown.tsx**

```tsx
import type { StatusBreakdown as StatusBreakdownType } from "../types";

interface StatusBreakdownProps {
  breakdown: StatusBreakdownType;
}

export default function StatusBreakdown({ breakdown }: StatusBreakdownProps) {
  return (
    <div className="status-section">
      <div className="section-label">PAS Status Breakdown</div>
      <div className="summary-row"><span>Confirmed</span><span>{breakdown.confirmed}</span></div>
      <div className="summary-row"><span>Pending</span><span>{breakdown.pending}</span></div>
      <div className="summary-row"><span>Deceased</span><span>{breakdown.deceased}</span></div>
      <div className="summary-row"><span>Removed</span><span>{breakdown.removed}</span></div>
      <div className="summary-row"><span>Not the MRP</span><span>{breakdown.not_the_mrp}</span></div>
    </div>
  );
}
```

- [ ] **Step 4: Create Sidebar.tsx**

```tsx
import DropZone from "./DropZone";
import SummaryCard from "./SummaryCard";
import StatusBreakdown from "./StatusBreakdown";
import type { Summary, StatusBreakdown as StatusBreakdownType } from "../types";

interface SidebarProps {
  onFilesDropped: (files: File[]) => void;
  emrLoaded: boolean;
  pasLoaded: boolean;
  error: string | null;
  summary: Summary | null;
  statusBreakdown: StatusBreakdownType | null;
}

export default function Sidebar({
  onFilesDropped, emrLoaded, pasLoaded, error, summary, statusBreakdown
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div>
        <h1>PAS Reconciliation</h1>
        <p className="version">v0.1.0</p>
      </div>
      <DropZone
        onFilesDropped={onFilesDropped}
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
        error={error}
      />
      {summary && <SummaryCard summary={summary} />}
      {statusBreakdown && <StatusBreakdown breakdown={statusBreakdown} />}
      <div className="privacy-note">
        Patient data stays on this machine. Closing the window clears it.
      </div>
    </aside>
  );
}
```

- [ ] **Step 5: Create ListTabs.tsx**

```tsx
import type { ListKey, Summary } from "../types";

interface ListTabsProps {
  active: ListKey;
  onSelect: (key: ListKey) => void;
  summary: Summary;
}

const TAB_CONFIG: { key: ListKey; label: string; countKey: keyof Summary }[] = [
  { key: "emr_no_match", label: "EMR No Match", countKey: "emr_only" },
  { key: "pas_match_review", label: "PAS Match - Review", countKey: "pas_review" },
  { key: "pas_no_match", label: "PAS No Match", countKey: "pas_only" },
];

const CONTEXT_LINES: Record<ListKey, string> = {
  emr_no_match: "Patients in your EMR panel but not found in PAS. These may need a 98990 bill submitted, or have incorrect status/MRP in the EMR.",
  pas_match_review: "Matched patients with a status of Pending, Not the MRP, Deceased, or Removed. These may need updating in your EMR.",
  pas_no_match: "Patients in PAS but not in your EMR panel. These may have left the clinic, or the EMR status/MRP is incorrect.",
};

export { CONTEXT_LINES };

export default function ListTabs({ active, onSelect, summary }: ListTabsProps) {
  return (
    <>
      <div className="list-tabs">
        {TAB_CONFIG.map((tab) => {
          const count = summary[tab.countKey] as number;
          return (
            <button
              key={tab.key}
              className={`tab ${active === tab.key ? "active" : ""}`}
              onClick={() => onSelect(tab.key)}
            >
              {tab.label}
              <span className="tab-badge">({count})</span>
            </button>
          );
        })}
      </div>
      <div className="context-line">{CONTEXT_LINES[active]}</div>
    </>
  );
}
```

- [ ] **Step 6: Create PatientTable.tsx**

```tsx
import { useState, useMemo } from "react";
import type { DisplayRow } from "../types";

interface PatientTableProps {
  rows: DisplayRow[];
  showStatus: boolean;
  resolvedSet: Set<string>;
  onToggleResolved: (phn: string) => void;
  searchQuery: string;
}

export default function PatientTable({
  rows, showStatus, resolvedSet, onToggleResolved, searchQuery
}: PatientTableProps) {
  const filtered = useMemo(() => {
    if (!searchQuery.trim()) return rows;
    const q = searchQuery.toLowerCase();
    return rows.filter((r) =>
      r.phn.toLowerCase().includes(q) ||
      (r.first_name?.toLowerCase().includes(q) ?? false) ||
      (r.last_name?.toLowerCase().includes(q) ?? false)
    );
  }, [rows, searchQuery]);

  if (filtered.length === 0) {
    return (
      <div className="empty-state">
        {rows.length === 0 ? "No patients in this list." : "No matches for your search."}
      </div>
    );
  }

  return (
    <div style={{ flex: 1, overflow: "auto", padding: "0 16px" }}>
      <table>
        <thead>
          <tr>
            <th>PHN</th>
            <th>First Name</th>
            <th>Last Name</th>
            <th>DOB</th>
            {showStatus && <th>Status</th>}
          </tr>
        </thead>
        <tbody>
          {filtered.map((row) => (
            <tr
              key={row.phn}
              className={resolvedSet.has(row.phn) ? "resolved" : ""}
              onClick={() => onToggleResolved(row.phn)}
              style={{ cursor: "pointer" }}
            >
              <td className="phn">{row.phn}</td>
              <td>{row.first_name ?? "—"}</td>
              <td>{row.last_name ?? "—"}</td>
              <td>{row.dob ?? "—"}</td>
              {showStatus && <td>{row.mrp_status ?? "—"}</td>}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
```

- [ ] **Step 7: Create EmptyState.tsx**

```tsx
export default function EmptyState({ message }: { message: string }) {
  return (
    <div className="empty-state">{message}</div>
  );
}
```

- [ ] **Step 8: Create UpdateToast.tsx**

```tsx
import type { UpdateInfo } from "../types";

interface UpdateToastProps {
  info: UpdateInfo;
  onDownload: () => void;
  onDismiss: () => void;
}

export default function UpdateToast({ info, onDownload, onDismiss }: UpdateToastProps) {
  return (
    <div className="update-toast" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
      <span>Update available — v{info.version} (you have v{info.current_version})</span>
      <span style={{ display: "flex", gap: "8px" }}>
        <button onClick={onDownload} className="export-btn">Download & Install</button>
        <button onClick={onDismiss} className="tab">Later</button>
      </span>
    </div>
  );
}
```

- [ ] **Step 9: Wire everything together in App.tsx**

Replace the stub `frontend/src/App.tsx` with:

```tsx
import { useState, useEffect, useCallback } from "react";
import Sidebar from "./components/Sidebar";
import ListTabs, { CONTEXT_LINES } from "./components/ListTabs";
import PatientTable from "./components/PatientTable";
import UpdateToast from "./components/UpdateToast";
import EmptyState from "./components/EmptyState";
import {
  reconcileFiles,
  reconcileWithColumnOverride,
  exportList,
  checkForUpdates,
  onUpdateAvailable,
} from "./api";
import type { ReconciliationResult, UpdateInfo, ListKey } from "./types";

export default function App() {
  const [result, setResult] = useState<ReconciliationResult | null>(null);
  const [emrLoaded, setEmrLoaded] = useState(false);
  const [pasLoaded, setPasLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [emrPath, setEmrPath] = useState<string | null>(null);
  const [pasPath, setPasPath] = useState<string | null>(null);
  const [activeList, setActiveList] = useState<ListKey>("emr_no_match");
  const [searchQuery, setSearchQuery] = useState("");
  const [resolved, setResolved] = useState<Set<string>>(new Set());
  const [update, setUpdate] = useState<UpdateInfo | null>(null);

  // Listen for auto-update notifications from the backend
  useEffect(() => {
    const unlisten = onUpdateAvailable((info) => setUpdate(info));
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleFilesDropped = useCallback(async (files: File[]) => {
    setError(null);
    let newEmrPath = emrPath;
    let newPasPath = pasPath;

    for (const file of files) {
      const name = file.name.toLowerCase();
      // Tauri drag-drop gives File objects; we need their paths.
      // In Tauri v2, dropped files expose their path via file.path (non-standard but available).
      // Alternatively use the @tauri-apps/api/webview onDragDropEvent.
      const filePath = (file as any).path || file.name;
      if (name.includes("emr")) {
        newEmrPath = filePath;
        setEmrLoaded(true);
      } else if (name.includes("pas")) {
        newPasPath = filePath;
        setPasLoaded(true);
      }
    }
    setEmrPath(newEmrPath);
    setPasPath(newPasPath);

    if (newEmrPath && newPasPath) {
      try {
        const res = await reconcileFiles(newEmrPath, newPasPath);
        setResult(res);
        setResolved(new Set()); // clear resolved state on re-reconcile
        setActiveList("emr_no_match");
      } catch (e: any) {
        // Check if it's a MissingPhnColumn error → could trigger column picker
        const errStr = typeof e === "string" ? e : JSON.stringify(e);
        if (errStr.includes("MissingPhnColumn") || errStr.includes("PHN column")) {
          setError("Could not auto-detect the PHN column. Use the column picker.");
        } else {
          setError(errStr);
        }
      }
    }
  }, [emrPath, pasPath]);

  const handleToggleResolved = useCallback((phn: string) => {
    setResolved((prev) => {
      const next = new Set(prev);
      if (next.has(phn)) next.delete(phn);
      else next.add(phn);
      return next;
    });
  }, []);

  const handleExport = useCallback(async () => {
    if (!result) return;
    const rows = result[activeList];
    // In a real app, use Tauri's save dialog. For now, prompt for a path via a simple approach.
    // The save dialog integration is added in Task 12 (polish).
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        defaultPath: `${activeList}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (path) {
        await exportList(activeList, rows, path);
      }
    } catch {
      // dialog plugin not available in dev — skip
    }
  }, [result, activeList]);

  const currentRows = result ? result[activeList] : [];
  const showStatus = activeList !== "emr_no_match";

  return (
    <div className="app">
      <Sidebar
        onFilesDropped={handleFilesDropped}
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
        error={error}
        summary={result?.summary ?? null}
        statusBreakdown={result?.summary.status_breakdown ?? null}
      />
      <main className="main-panel">
        {update && (
          <UpdateToast
            info={update}
            onDownload={() => {
              // Download + install logic wired in Task 12
              import("@tauri-apps/plugin-updater").then(({ check }) => {
                check().then((u) => u?.downloadAndInstall());
              });
            }}
            onDismiss={() => setUpdate(null)}
          />
        )}
        {result ? (
          <>
            <ListTabs
              active={activeList}
              onSelect={setActiveList}
              summary={result.summary}
            />
            <div className="toolbar">
              <input
                className="search-input"
                placeholder="Search PHN or name…"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              <button className="export-btn" onClick={handleExport}>Export CSV</button>
            </div>
            <PatientTable
              rows={currentRows}
              showStatus={showStatus}
              resolvedSet={resolved}
              onToggleResolved={handleToggleResolved}
              searchQuery={searchQuery}
            />
            <div className="status-bar">
              <span>Showing {currentRows.length} patients · sorted by last name</span>
              <span>Data in memory only · not saved to disk</span>
            </div>
          </>
        ) : (
          <EmptyState message={error ?? "Drop both CSV files to begin reconciliation."} />
        )}
      </main>
    </div>
  );
}
```

- [ ] **Step 10: Add @tauri-apps/plugin-dialog to package.json**

In `frontend/package.json`, add to `dependencies`:

```json
"@tauri-apps/plugin-dialog": "^2.0.0"
```

Then run:
```bash
cd frontend && npm install
```

Also add the dialog plugin to the Rust side. In `crates/app/Cargo.toml`:

```toml
tauri-plugin-dialog = "2"
```

And in `crates/app/src/main.rs`, add to the builder chain:

```rust
.plugin(tauri_plugin_dialog::init())
```

- [ ] **Step 11: Verify the frontend builds**

Run:
```bash
cd frontend && npm run build
```
Expected: builds successfully into `dist/`.

- [ ] **Step 12: Commit**

```bash
git add frontend/ crates/app/
git commit -m "feat(frontend): full split-panel UI with drop zone, summary, tables, update toast"
```

---

# Phase 5: Auto-Update + Packaging

## Task 11: Generate updater keys and configure CI

**Files:**
- Create: `.github/workflows/release.yml`
- Modify: `crates/app/tauri.conf.json` (replace pubkey placeholder)
- Create: `docs/release-setup.md` (instructions for the human-only steps)

- [ ] **Step 1: Generate the updater keypair**

Run:
```bash
cargo install tauri-cli --version "^2.0.0-rc" || true
cargo tauri signer generate -w crates/app/.updater-key 2>&1 | tee /tmp/updater-key-output.txt
```

This prints a public key. Capture it. **Never commit the private key file.**

- [ ] **Step 2: Add the private key to .gitignore**

Append to `.gitignore`:
```
crates/app/.updater-key
crates/app/.updater-key.pub
```

- [ ] **Step 3: Replace the pubkey placeholder in tauri.conf.json**

In `crates/app/tauri.conf.json`, replace `"PLACEHOLDER_REPLACE_WITH_GENERATED_PUBKEY"` with the actual public key from Step 1.

- [ ] **Step 4: Create the GitHub Actions release workflow**

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  create-release:
    runs-on: ubuntu-latest
    outputs:
      release_upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/create-release@v1
        id: create_release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref_name }}
          release_name: ${{ github.ref_name }}
          draft: false
          prerelease: false

  build-tauri:
    needs: create-release
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: macos-latest
            args: "--target aarch64-apple-darwin"
          - platform: macos-latest
            args: "--target x86_64-apple-darwin"
          - platform: ubuntu-22.04
            args: ""
          - platform: windows-latest
            args: ""

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Install dependencies (ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Install frontend dependencies
        run: |
          cd frontend
          npm install

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
          TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
          # macOS code signing (configure these secrets in GitHub repo settings):
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          # Windows code signing:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
        with:
          releaseId: ${{ needs.create-release.outputs.release_upload_url }}
          args: ${{ matrix.args }}

  publish-update-manifest:
    needs: build-tauri
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Generate latest.json
        run: |
          # The tauri-action uploads the manifest as part of release assets.
          # This step ensures latest.json is the "latest" release asset.
          echo "latest.json is generated by tauri-action and uploaded to the release."
```

- [ ] **Step 5: Create the release setup documentation**

Create `docs/release-setup.md`:

```markdown
# Release Setup — One-Time Configuration

These steps are done once, by the repo owner, before the first release.

## 1. Generate the updater keypair

```bash
cargo tauri signer generate -w ~/.tauri/pas-recon-updater
```

This creates two files:
- `~/.tauri/pas-recon-updater` (private key — KEEP SECRET)
- `~/.tauri/pas-recon-updater.pub` (public key — embed in the app)

## 2. Embed the public key

Copy the contents of the `.pub` file into `crates/app/tauri.conf.json` under
`plugins.updater.pubkey`.

## 3. Add secrets to the GitHub repo

Go to repo Settings → Secrets and variables → Actions. Add:

| Secret | Value |
|--------|-------|
| `TAURI_PRIVATE_KEY` | Contents of the private key file |
| `TAURI_KEY_PASSWORD` | The password you set during key generation |

## 4. (Optional) Code signing

For silent auto-update (no OS prompts), you need:

### macOS
| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 |
| `APPLE_SIGNING_IDENTITY` | Developer ID Application signer identity |
| `APPLE_ID` | Apple ID for notarization |
| `APPLE_PASSWORD` | App-specific password for notarization |

### Windows
| Secret | Description |
|--------|-------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Code signing certificate |

**Without these**, the app still builds and updates download, but the OS
will show "unidentified developer" warnings. Acceptable for initial release.

## 5. Create the first release

```bash
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Actions workflow builds all platforms and uploads installers +
the `latest.json` manifest to the release.
```

- [ ] **Step 6: Create the private GitHub repo and push**

Run:
```bash
gh repo create pas-recon --private --source=. --push
```
(If `gh` isn't authenticated, follow the prompts. Replace `<owner>` in
`tauri.conf.json` `endpoints` with your actual GitHub username/org.)

Then update `crates/app/tauri.conf.json`:
- Replace `<owner>/pas-recon` with the actual owner/repo in the `endpoints` URL.

- [ ] **Step 7: Commit**

```bash
git add .github/ docs/release-setup.md crates/app/tauri.conf.json .gitignore
git commit -m "feat(ci): release workflow with auto-update signing and multi-platform builds"
```

---

# Phase 6: Polish

## Task 12: Column-picker fallback, save dialog, error states

**Files:**
- Create: `frontend/src/components/ColumnPicker.tsx`
- Modify: `frontend/src/App.tsx` (integrate column picker)
- Modify: `crates/app/src/commands.rs` (add `get_csv_headers` command)
- Modify: `crates/app/src/main.rs` (register new command)

- [ ] **Step 1: Add a get_csv_headers command to the backend**

In `crates/app/src/commands.rs`, add:

```rust
/// Read just the header row of a CSV file. Used by the column-picker fallback
/// when auto-detection fails.
#[tauri::command]
pub fn get_csv_headers(path: String) -> Result<Vec<String>, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let parsed = pas_recon_engine::parse::parse_csv(&bytes).map_err(|e| e.to_string())?;
    Ok(parsed.headers)
}
```

- [ ] **Step 2: Register the new command**

In `crates/app/src/main.rs`, add `commands::get_csv_headers` to the `generate_handler!` list:

```rust
.invoke_handler(tauri::generate_handler![
    commands::reconcile_files,
    commands::reconcile_with_column_override,
    commands::export_list,
    commands::check_for_updates,
    commands::get_csv_headers,
])
```

- [ ] **Step 3: Add get_csv_headers to the API layer**

In `frontend/src/api.ts`, add:

```typescript
export async function getCsvHeaders(path: string): Promise<string[]> {
  return invoke<string[]>("get_csv_headers", { path });
}
```

- [ ] **Step 4: Create ColumnPicker.tsx**

Create `frontend/src/components/ColumnPicker.tsx`:

```tsx
import { useState, useEffect } from "react";
import { getCsvHeaders } from "../api";

interface ColumnPickerProps {
  emrPath: string;
  pasPath: string;
  onResolved: (emrCol: number, pasCol: number) => void;
  onCancel: () => void;
}

export default function ColumnPicker({ emrPath, pasPath, onResolved, onCancel }: ColumnPickerProps) {
  const [emrHeaders, setEmrHeaders] = useState<string[]>([]);
  const [pasHeaders, setPasHeaders] = useState<string[]>([]);
  const [emrCol, setEmrCol] = useState(0);
  const [pasCol, setPasCol] = useState(0);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([getCsvHeaders(emrPath), getCsvHeaders(pasPath)])
      .then(([emr, pas]) => {
        setEmrHeaders(emr);
        setPasHeaders(pas);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, [emrPath, pasPath]);

  if (loading) return <div className="empty-state">Loading headers…</div>;

  return (
    <div style={{ padding: "24px", maxWidth: "600px", margin: "0 auto" }}>
      <h2 style={{ fontSize: "16px", marginBottom: "8px" }}>Select PHN Columns</h2>
      <p style={{ fontSize: "11px", color: "var(--text-dim)", marginBottom: "20px" }}>
        We couldn't auto-detect the PHN column in one or both files. Please identify them manually.
      </p>
      <div style={{ display: "flex", gap: "24px" }}>
        <div style={{ flex: 1 }}>
          <label style={{ fontSize: "11px", color: "var(--text-dim)" }}>EMR PHN column:</label>
          <select
            value={emrCol}
            onChange={(e) => setEmrCol(Number(e.target.value))}
            className="search-input"
            style={{ display: "block", marginTop: "4px" }}
          >
            {emrHeaders.map((h, i) => (
              <option key={i} value={i}>{i}: {h}</option>
            ))}
          </select>
        </div>
        <div style={{ flex: 1 }}>
          <label style={{ fontSize: "11px", color: "var(--text-dim)" }}>PAS PHN column:</label>
          <select
            value={pasCol}
            onChange={(e) => setPasCol(Number(e.target.value))}
            className="search-input"
            style={{ display: "block", marginTop: "4px" }}
          >
            {pasHeaders.map((h, i) => (
              <option key={i} value={i}>{i}: {h}</option>
            ))}
          </select>
        </div>
      </div>
      <div style={{ marginTop: "24px", display: "flex", gap: "8px", justifyContent: "flex-end" }}>
        <button className="tab" onClick={onCancel}>Cancel</button>
        <button className="export-btn" onClick={() => onResolved(emrCol, pasCol)}>Confirm</button>
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Integrate ColumnPicker into App.tsx**

In `frontend/src/App.tsx`, add column-picker state and rendering. Add these state declarations near the others:

```tsx
const [showColumnPicker, setShowColumnPicker] = useState(false);
```

Modify the `catch` block in `handleFilesDropped` to trigger the picker when auto-detect fails:

```tsx
} catch (e: any) {
  const errStr = typeof e === "string" ? e : JSON.stringify(e);
  if (errStr.includes("MissingPhnColumn") || errStr.includes("AmbiguousPhnColumns")) {
    setShowColumnPicker(true);
    setError(null);
  } else {
    setError(errStr);
  }
}
```

Add a handler for when the column picker resolves:

```tsx
const handleColumnPickerResolved = useCallback(async (emrCol: number, pasCol: number) => {
  if (!emrPath || !pasPath) return;
  setShowColumnPicker(false);
  try {
    const res = await reconcileWithColumnOverride(emrPath, pasPath, emrCol, pasCol);
    setResult(res);
    setResolved(new Set());
    setActiveList("emr_no_match");
  } catch (e: any) {
    setError(typeof e === "string" ? e : JSON.stringify(e));
  }
}, [emrPath, pasPath]);
```

In the JSX, before the `result ?` ternary in the main panel, add:

```tsx
{showColumnPicker && emrPath && pasPath ? (
  <ColumnPicker
    emrPath={emrPath}
    pasPath={pasPath}
    onResolved={handleColumnPickerResolved}
    onCancel={() => setShowColumnPicker(false)}
  />
) : result ? (
  // ... existing result rendering
) : (
  // ... existing empty state
)}
```

Import the new component at the top:

```tsx
import ColumnPicker from "./components/ColumnPicker";
```

- [ ] **Step 6: Verify the build**

Run:
```bash
cd frontend && npm run build
```
Expected: builds successfully.

- [ ] **Step 7: Verify the full workspace compiles**

Run: `cargo check`
Expected: no errors.

- [ ] **Step 8: Commit**

```bash
git add frontend/ crates/app/
git commit -m "feat: column-picker fallback for when PHN auto-detection fails"
```

---

## Task 13: Run full test suite and document

**Files:**
- Create: `README.md`
- Modify: `crates/engine/tests/reconcile_test.rs` (add edge-case tests using the dirty fixtures)

- [ ] **Step 1: Add edge-case integration tests**

Append to `crates/engine/tests/reconcile_test.rs`:

```rust
#[test]
fn handles_dirty_emr_with_invalid_phns() {
    let emr = read_fixture("emr_dirty.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    // emr_dirty.csv has:
    // - 9876543210 (valid) → matched
    // - 1234567890 (invalid: starts with 1) → skipped
    // - "9876 543 210" with spaces (valid after normalize) → matched (same as 9876543210)
    // - a short/mangled row → likely skipped
    // At minimum, invalid_phn_skipped should be >= 1
    assert!(result.summary.invalid_phn_skipped >= 1, "Should skip invalid PHNs");
}

#[test]
fn deduplicates_pas_by_latest_date() {
    let emr = b"PHN,First,Last\n9876543210,John,Smith\n9871111222,Mary,Jones\n";
    let pas = read_fixture("pas_duplicates.csv");

    let result = reconcile(&emr, &pas).unwrap();

    // pas_duplicates.csv has 3 rows for 9876543210 and 2 for 9871111222
    // Dedup should drop 2 + 1 = 3 duplicates
    assert_eq!(result.summary.duplicates_dropped, 3);
    // Both distinct PHNs should match
    assert_eq!(result.summary.matched, 2);
}

#[test]
fn empty_result_lists_when_all_match_and_confirmed() {
    let csv = b"PHN,First,Last,MRP Status\n9876543210,John,Smith,Confirmed\n";
    let result = reconcile(csv, csv).unwrap();

    assert_eq!(result.summary.matched, 1);
    assert_eq!(result.emr_no_match.len(), 0);
    assert_eq!(result.pas_no_match.len(), 0);
    assert_eq!(result.pas_match_review.len(), 0); // Confirmed → not on review list
}

#[test]
fn pas_without_status_column_produces_empty_review_list() {
    let emr = b"PHN,Name\n9876543210,John\n";
    let pas = b"PHN,Name\n9876543210,John\n"; // no MRP Status column

    let result = reconcile(&emr[..], &pas[..]).unwrap();

    assert_eq!(result.summary.matched, 1);
    assert_eq!(result.pas_match_review.len(), 0); // no status → can't be "review"
}
```

- [ ] **Step 2: Run the complete engine test suite**

Run: `cargo test --package pas-recon-engine`
Expected: all tests across all test files PASS.

- [ ] **Step 3: Run the full workspace check**

Run: `cargo check --workspace`
Expected: no errors.

- [ ] **Step 4: Create README.md**

```markdown
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
```

- [ ] **Step 5: Commit**

```bash
git add README.md crates/engine/tests/reconcile_test.rs
git commit -m "test: edge-case integration tests; docs: README"
```

---

## Self-Review

**1. Spec coverage:**

| Spec section | Covered by |
|---|---|
| §3.1 Engine crate (parse, detect, phn, dedup, match) | Tasks 1–7 |
| §3.2 Tauri shell (commands, updater) | Task 8 |
| §3.3 Frontend (React, sidebar, tables) | Tasks 9–10 |
| §4.1 Parse (BOM, flexible cols, CRLF) | Task 2 |
| §4.2 Column auto-detection | Task 4 |
| §4.3 PHN validation | Task 3 |
| §4.4 PAS dedup | Task 6 |
| §4.5 Cross-match & classify | Task 7 |
| §4.6 Assemble (sort by name) | Task 7 |
| §5 UI (split-panel, drop zone, tabs, search, export, resolved) | Tasks 9–10 |
| §6 Error handling (file errors, auto-detect fallback, row issues) | Task 12 (column picker), Task 7 (row-level counters) |
| §7 Auto-update (GitHub Releases) | Task 8 (wiring), Task 11 (CI + keys) |
| §8 Packaging | Task 11 (CI builds .dmg/.msi/.deb/AppImage) |
| §9 CI/Release pipeline | Task 11 |
| §10 Out of scope | Not implemented (correct) |

All sections covered. ✓

**2. Placeholder scan:** Searched for "TBD", "TODO", "implement later", "fill in". Found:
- `todo!()` in Task 1 lib.rs — intentional stub, replaced in Task 7. ✓
- `<owner>/pas-recon` in tauri.conf.json — flagged in Task 11 Step 6 to replace with actual owner. ✓
- `PLACEHOLDER_REPLACE_WITH_GENERATED_PUBKEY` — replaced in Task 11 Step 3. ✓
No undocumented placeholders. ✓

**3. Type consistency:**
- `ReconciliationResult` fields (`emr_no_match`, `pas_match_review`, `pas_no_match`) match between model.rs (Task 1), reconcile.rs (Task 7), and TS types (Task 9). ✓
- `DisplayRow` fields match across model.rs, emr_to_display/pas_to_display, and TS. ✓
- `ListKey` in TS = `"emr_no_match" | "pas_match_review" | "pas_no_match"` — matches the result struct field names and the `ListKind` enum in commands.rs. ✓
- `reconcile_files` command param names in Rust (`emr_path`, `pas_path`) match the camelCase keys in api.ts (`emrPath`, `pasPath`) — Tauri auto-converts. ✓

All consistent. ✓
