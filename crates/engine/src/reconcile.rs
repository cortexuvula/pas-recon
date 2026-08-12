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

/// Deduplicate EMR records by PHN, keeping first seen.
/// EMR exports have no date column for tiebreaking, unlike PAS.
fn dedup_emr(records: Vec<EmrRecord>) -> Vec<EmrRecord> {
    let mut seen = HashSet::new();
    records.into_iter().filter(|r| seen.insert(r.phn.clone())).collect()
}

/// Which status values put a matched PAS patient on the "review" list.
const REVIEW_STATUSES: &[&str] = &["pending", "not the mrp", "deceased", "removed"];

/// Extract a trimmed, non-empty field from a row by optional column index.
fn field(fields: &[String], idx: Option<usize>) -> Option<String> {
    idx.and_then(|i| fields.get(i))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Case-insensitive string comparison without allocating lowercased strings
/// (used by the list sorter, which runs O(n log n) comparisons).
fn ci_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ai = a.chars().flat_map(|c| c.to_lowercase());
    let mut bi = b.chars().flat_map(|c| c.to_lowercase());
    loop {
        match (ai.next(), bi.next()) {
            (Some(x), Some(y)) => match x.cmp(&y) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            },
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return Ordering::Equal,
        }
    }
}

/// Build a DisplayRow from a raw CSV row that failed PHN validation.
/// Shows the raw (invalid) PHN and any detected name/date fields for context.
fn raw_row_to_display(
    row: &crate::model::RawRow,
    mapping: &ColumnMapping,
    raw_phn: &str,
    source: &str,
) -> DisplayRow {
    DisplayRow {
        phn: raw_phn.trim().to_string(),
        first_name: field(&row.fields, mapping.first_name),
        last_name: field(&row.fields, mapping.last_name),
        dob: field(&row.fields, mapping.dob),
        mrp_status: field(&row.fields, mapping.mrp_status),
        raw_fields: row.fields.clone(),
        source: Some(source.to_string()),
    }
}

/// Parse and validate EMR records from a parsed CSV using a column mapping.
/// Returns (valid_records, invalid_rows_as_display_rows).
fn build_emr_records(
    parsed: &crate::parse::ParsedCsv,
    mapping: &ColumnMapping,
) -> (Vec<EmrRecord>, Vec<DisplayRow>) {
    let mut records = Vec::new();
    let mut invalid_rows = Vec::new();

    for row in &parsed.rows {
        let raw_phn = row.fields.get(mapping.phn).map(|s| s.as_str()).unwrap_or("");

        if !phn::is_valid_bc_phn(raw_phn) {
            invalid_rows.push(raw_row_to_display(row, mapping, raw_phn, "EMR"));
            continue;
        }

        let normalized = phn::normalize_phn(raw_phn);
        records.push(EmrRecord {
            phn: normalized,
            raw_fields: row.fields.clone(),
            row_index: row.row_index,
        });
    }

    (records, invalid_rows)
}

/// Parse and validate PAS records from a parsed CSV using a column mapping.
/// Returns (valid_records, invalid_rows_as_display_rows, unparseable_date_count).
fn build_pas_records(
    parsed: &crate::parse::ParsedCsv,
    mapping: &ColumnMapping,
) -> (Vec<PasRecord>, Vec<DisplayRow>, usize) {
    let mut records = Vec::new();
    let mut invalid_rows = Vec::new();
    let mut bad_dates = 0usize;

    for row in &parsed.rows {
        let raw_phn = row.fields.get(mapping.phn).map(|s| s.as_str()).unwrap_or("");

        if !phn::is_valid_bc_phn(raw_phn) {
            invalid_rows.push(raw_row_to_display(row, mapping, raw_phn, "PAS"));
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

    (records, invalid_rows, bad_dates)
}

/// Build a DisplayRow from an EMR record + column mapping.
fn emr_to_display(record: &EmrRecord, mapping: &ColumnMapping) -> DisplayRow {
    DisplayRow {
        phn: record.phn.clone(),
        first_name: field(&record.raw_fields, mapping.first_name),
        last_name: field(&record.raw_fields, mapping.last_name),
        dob: field(&record.raw_fields, mapping.dob),
        mrp_status: None,
        raw_fields: record.raw_fields.clone(),
        source: None,
    }
}

/// Build a DisplayRow from a PAS record + column mapping.
fn pas_to_display(record: &PasRecord, mapping: &ColumnMapping) -> DisplayRow {
    DisplayRow {
        phn: record.phn.clone(),
        first_name: field(&record.raw_fields, mapping.first_name),
        last_name: field(&record.raw_fields, mapping.last_name),
        dob: field(&record.raw_fields, mapping.dob),
        mrp_status: record.mrp_status.clone(),
        raw_fields: record.raw_fields.clone(),
        source: None,
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
        DetectionError::AmbiguousColumn { field, candidates } => EngineError::AmbiguousColumn {
            source: source.to_string(),
            field: field.to_string(),
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

/// Detect all columns except PHN, then inject the user-provided PHN index.
/// This is used when auto-detection can't find a PHN header — the manual
/// column picker provides the index directly.
///
/// Returns an error if `phn_idx` is out of range for the file's headers
/// (previously this was silently clamped to the last column, which hid picker
/// mistakes by reading the wrong column).
fn detect_columns_with_phn_override(
    headers: &[String],
    is_pas: bool,
    phn_idx: usize,
    source: CsvSource,
) -> Result<ColumnMapping, EngineError> {
    use crate::detect::{
        find_column, DOB_PATTERNS, FIRST_PATTERNS, LAST_PATTERNS, STATUS_PATTERNS,
        UPDATED_PATTERNS,
    };

    if headers.is_empty() || phn_idx >= headers.len() {
        return Err(EngineError::InvalidColumnIndex {
            source: source.to_string(),
            index: phn_idx,
            header_count: headers.len(),
        });
    }

    // Optional fields use best-effort matching (first among ties). The manual
    // picker only selects PHN, so ambiguity here must not block the run.
    let (mrp_status, mrp_updated) = if is_pas {
        (
            find_column(headers, STATUS_PATTERNS),
            find_column(headers, UPDATED_PATTERNS),
        )
    } else {
        (None, None)
    };

    Ok(ColumnMapping {
        phn: phn_idx,
        first_name: find_column(headers, FIRST_PATTERNS),
        last_name: find_column(headers, LAST_PATTERNS),
        dob: find_column(headers, DOB_PATTERNS),
        mrp_status,
        mrp_updated,
    })
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
    // When a PHN override is provided (manual column picker), we skip PHN
    // auto-detection entirely and only detect the other fields. This avoids
    // MissingPhnColumn errors when the header doesn't match known patterns.
    let emr_mapping = if let Some(phn_idx) = emr_phn_override {
        detect_columns_with_phn_override(&emr_parsed.headers, false, phn_idx, CsvSource::Emr)?
    } else {
        detect_columns(&emr_parsed.headers, false)
            .map_err(|e| map_detection_error(e, CsvSource::Emr))?
    };

    let pas_mapping = if let Some(phn_idx) = pas_phn_override {
        detect_columns_with_phn_override(&pas_parsed.headers, true, phn_idx, CsvSource::Pas)?
    } else {
        detect_columns(&pas_parsed.headers, true)
            .map_err(|e| map_detection_error(e, CsvSource::Pas))?
    };

    // --- Build records + validate PHNs ---
    let (emr_records, emr_invalid_rows) = build_emr_records(&emr_parsed, &emr_mapping);
    let (pas_records, pas_invalid_rows, bad_dates) = build_pas_records(&pas_parsed, &pas_mapping);

    // Combine invalid rows from both sources
    let mut invalid_phns = emr_invalid_rows;
    invalid_phns.extend(pas_invalid_rows);

    // --- Dedup EMR (keep first seen — no date column to compare) ---
    let emr_records = dedup_emr(emr_records);

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
        ci_cmp(
            a.last_name.as_deref().unwrap_or(""),
            b.last_name.as_deref().unwrap_or(""),
        )
        .then_with(|| {
            ci_cmp(
                a.first_name.as_deref().unwrap_or(""),
                b.first_name.as_deref().unwrap_or(""),
            )
        })
    };
    emr_no_match.sort_by(sort_fn);
    pas_match_review.sort_by(sort_fn);
    pas_no_match.sort_by(sort_fn);
    invalid_phns.sort_by(sort_fn);

    let summary = Summary {
        matched,
        emr_only: emr_no_match.len(),
        pas_only: pas_no_match.len(),
        pas_review: pas_match_review.len(),
        status_breakdown,
        duplicates_dropped,
        invalid_phn_skipped: invalid_phns.len(),
        unparseable_dates: bad_dates,
        truncated_rows: emr_parsed.truncated_rows + pas_parsed.truncated_rows,
    };

    Ok(ReconciliationResult {
        summary,
        emr_no_match,
        pas_match_review,
        pas_no_match,
        invalid_phns,
    })
}
