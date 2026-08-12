//! Column auto-detection by header pattern matching.
//!
//! Matching uses a specificity score so the *best* (most specific) match wins
//! rather than the first header positionally:
//!   3 = exact match (normalized header equals a pattern)
//!   2 = whole-word match (pattern bounded by non-alphanumerics)
//!   1 = substring match
//! This prevents collisions like the bare pattern `"last"` silently grabbing a
//! `"Last Updated"` column when a `"Last Name"` column is the intended target.
//!
//! When two columns tie at the top score for a field, detection is ambiguous
//! and surfaces an error instead of silently picking the first (PHN has always
//! done this; it now applies to every recognized field).

use crate::model::ColumnMapping;

#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("no PHN column found")]
    MissingPhnColumn,
    #[error("multiple PHN columns found: {candidates:?}")]
    AmbiguousPhnColumns { candidates: Vec<String> },
    #[error("multiple {field} columns found: {candidates:?}")]
    AmbiguousColumn {
        field: &'static str,
        candidates: Vec<String>,
    },
}

/// Header patterns for each recognized field. Matched case-insensitively
/// after trimming whitespace.
pub const PHN_PATTERNS: &[&str] = &["phn", "personal health number", "bc phn", "health number"];
pub const FIRST_PATTERNS: &[&str] = &["first", "first name", "given", "given name", "fname"];
pub const LAST_PATTERNS: &[&str] = &["last", "last name", "surname", "family", "lname"];
pub const DOB_PATTERNS: &[&str] = &["dob", "date of birth", "birth date", "birthdate"];
pub const STATUS_PATTERNS: &[&str] = &["mrp status", "status", "attachment status"];
pub const UPDATED_PATTERNS: &[&str] = &["mrp updated", "mrp updated date", "updated", "last updated"];

/// Highest specificity score for a normalized header against a pattern set.
fn match_score(normalized_header: &str, patterns: &[&str]) -> u8 {
    let mut best = 0u8;
    for &p in patterns {
        if normalized_header == p {
            return 3; // exact
        }
        let s = if is_whole_word_match(normalized_header, p) {
            2
        } else if normalized_header.contains(p) {
            1
        } else {
            0
        };
        if s > best {
            best = s;
        }
    }
    best
}

/// True if `needle` occurs in `haystack` bounded on both sides by
/// non-alphanumeric characters (or the string start/end).
fn is_whole_word_match(haystack: &str, needle: &str) -> bool {
    // find returns a valid char boundary.
    let start = match haystack.find(needle) {
        Some(i) => i,
        None => return false,
    };
    let end = start + needle.len();
    let before_ok = start == 0
        || !haystack[..start].ends_with(|c: char| c.is_alphanumeric());
    let after_ok = end == haystack.len()
        || !haystack[end..].starts_with(|c: char| c.is_alphanumeric());
    before_ok && after_ok
}

/// The outcome of searching for one field's column across all headers.
enum BestMatch {
    None,
    Single(usize),
    /// Two or more headers tied at the top specificity score.
    Ambiguous(Vec<usize>),
}

/// Find the best-matching column for a field. Ties at the top score are
/// reported as ambiguous rather than silently resolved by position.
fn find_best_column(headers: &[String], patterns: &[&str]) -> BestMatch {
    let mut best_score = 0u8;
    let mut matches: Vec<usize> = Vec::new();
    for (idx, header) in headers.iter().enumerate() {
        let normalized = header.trim().to_lowercase();
        let score = match_score(&normalized, patterns);
        if score == 0 {
            continue;
        }
        if score > best_score {
            best_score = score;
            matches.clear();
            matches.push(idx);
        } else if score == best_score {
            matches.push(idx);
        }
    }
    match matches.len() {
        0 => BestMatch::None,
        1 => BestMatch::Single(matches[0]),
        _ => BestMatch::Ambiguous(matches),
    }
}

/// Best-effort single column find: the highest-scoring match, tie-broken by
/// position. Kept `pub` for the manual-override path in `reconcile`, where the
/// user has already intervened to pick the PHN and ambiguity in an optional
/// column must not block the run (the picker only selects PHN).
pub fn find_column(headers: &[String], patterns: &[&str]) -> Option<usize> {
    match find_best_column(headers, patterns) {
        BestMatch::None => None,
        BestMatch::Single(i) => Some(i),
        BestMatch::Ambiguous(idxs) => Some(idxs[0]),
    }
}

/// Resolve an optional field, erroring on ambiguity.
fn require_unambiguous(
    headers: &[String],
    patterns: &[&str],
    field: &'static str,
) -> Result<Option<usize>, DetectionError> {
    match find_best_column(headers, patterns) {
        BestMatch::None => Ok(None),
        BestMatch::Single(i) => Ok(Some(i)),
        BestMatch::Ambiguous(idxs) => Err(DetectionError::AmbiguousColumn {
            field,
            candidates: idxs.iter().map(|&i| headers[i].clone()).collect(),
        }),
    }
}

/// Detect column mapping from headers. `is_pas` controls whether to look for
/// MRP status/updated columns. Returns an error if any recognized field is
/// missing (PHN only) or ambiguous (all fields, including PHN).
pub fn detect_columns(headers: &[String], is_pas: bool) -> Result<ColumnMapping, DetectionError> {
    let phn = match find_best_column(headers, PHN_PATTERNS) {
        BestMatch::None => return Err(DetectionError::MissingPhnColumn),
        BestMatch::Ambiguous(idxs) => {
            return Err(DetectionError::AmbiguousPhnColumns {
                candidates: idxs.iter().map(|&i| headers[i].clone()).collect(),
            })
        }
        BestMatch::Single(i) => i,
    };

    let first_name = require_unambiguous(headers, FIRST_PATTERNS, "first name")?;
    let last_name = require_unambiguous(headers, LAST_PATTERNS, "last name")?;
    let dob = require_unambiguous(headers, DOB_PATTERNS, "date of birth")?;

    let (mrp_status, mrp_updated) = if is_pas {
        (
            require_unambiguous(headers, STATUS_PATTERNS, "MRP status")?,
            require_unambiguous(headers, UPDATED_PATTERNS, "MRP updated")?,
        )
    } else {
        (None, None)
    };

    Ok(ColumnMapping {
        phn,
        first_name,
        last_name,
        dob,
        mrp_status,
        mrp_updated,
    })
}
