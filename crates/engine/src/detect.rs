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
pub const PHN_PATTERNS: &[&str] = &["phn", "personal health number", "bc phn", "health number"];
pub const FIRST_PATTERNS: &[&str] = &["first", "first name", "given", "given name", "fname"];
pub const LAST_PATTERNS: &[&str] = &["last", "last name", "surname", "family", "lname"];
pub const DOB_PATTERNS: &[&str] = &["dob", "date of birth", "birth date", "birthdate"];
pub const STATUS_PATTERNS: &[&str] = &["mrp status", "status", "attachment status"];
pub const UPDATED_PATTERNS: &[&str] = &["mrp updated", "mrp updated date", "updated", "last updated"];

/// Check if a header matches any of the patterns using substring matching
/// (contains), like the spreadsheet's COUNTIF(..., "*pattern*").
/// Case-insensitive after trimming whitespace.
fn header_matches(normalized_header: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| normalized_header.contains(p))
}

pub fn find_column(headers: &[String], patterns: &[&str]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let normalized = header.trim().to_lowercase();
        if header_matches(&normalized, patterns) {
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
            header_matches(&normalized, patterns)
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
