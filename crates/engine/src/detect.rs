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
//! Two additional rules keep mappings sane:
//! - When two columns tie at the top score for a field, detection is ambiguous
//!   and surfaces an error instead of silently picking the first.
//! - A column claimed with an *exact* match by one field is authoritative: no
//!   other field may claim it at a lower score. This stops e.g. the bare
//!   `"last"` whole-word match from mapping `last_name` onto a `"Last Updated"`
//!   column (dates would render as surnames).

use std::collections::HashSet;

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
    Single { idx: usize, score: u8 },
    /// Two or more headers tied at the top specificity score.
    Ambiguous { idxs: Vec<usize>, score: u8 },
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
        1 => BestMatch::Single { idx: matches[0], score: best_score },
        _ => BestMatch::Ambiguous { idxs: matches, score: best_score },
    }
}

/// A field's claim on a column: (column index, specificity score it won with).
type Claim = Option<(usize, u8)>;

/// Resolve an optional field to a claim, erroring on top-score ties.
fn require_unambiguous(
    headers: &[String],
    patterns: &[&str],
    field: &'static str,
) -> Result<Claim, DetectionError> {
    match find_best_column(headers, patterns) {
        BestMatch::None => Ok(None),
        BestMatch::Single { idx, score } => Ok(Some((idx, score))),
        BestMatch::Ambiguous { idxs, .. } => Err(DetectionError::AmbiguousColumn {
            field,
            candidates: idxs.iter().map(|&i| headers[i].clone()).collect(),
        }),
    }
}

/// Best-effort claim: ties resolve to the first candidate by position. Used
/// where ambiguity must not block the run (the manual-override path, and
/// advisory status/updated detection in EMR files).
fn best_effort(headers: &[String], patterns: &[&str]) -> Claim {
    match find_best_column(headers, patterns) {
        BestMatch::None => None,
        BestMatch::Single { idx, score } => Some((idx, score)),
        BestMatch::Ambiguous { idxs, score } => Some((idxs[0], score)),
    }
}

/// Drop lower-specificity claims on columns that another field claimed with an
/// exact header match. Exact matches are authoritative: with headers like
/// ["PHN", "DOB", "Last Updated"] and no "Last Name" column, the bare "last"
/// whole-word match must not map `last_name` onto the date column that
/// `mrp_updated` owns exactly — dates would render as surnames.
fn resolve_conflicts(claims: &mut [Claim]) {
    let exact: HashSet<usize> = claims
        .iter()
        .flatten()
        .filter(|(_, score)| *score == 3)
        .map(|(idx, _)| *idx)
        .collect();
    for claim in claims.iter_mut() {
        if let Some((idx, score)) = *claim {
            if score < 3 && exact.contains(&idx) {
                *claim = None;
            }
        }
    }
}

/// Detect column mapping from headers. `is_pas` controls whether MRP
/// status/updated columns are surfaced. Returns an error if PHN is missing or
/// any recognized field is ambiguous.
pub fn detect_columns(headers: &[String], is_pas: bool) -> Result<ColumnMapping, DetectionError> {
    let phn = match find_best_column(headers, PHN_PATTERNS) {
        BestMatch::None => return Err(DetectionError::MissingPhnColumn),
        BestMatch::Ambiguous { idxs, .. } => {
            return Err(DetectionError::AmbiguousPhnColumns {
                candidates: idxs.iter().map(|&i| headers[i].clone()).collect(),
            })
        }
        BestMatch::Single { idx, .. } => idx,
    };

    let mut claims = [
        require_unambiguous(headers, FIRST_PATTERNS, "first name")?,
        require_unambiguous(headers, LAST_PATTERNS, "last name")?,
        require_unambiguous(headers, DOB_PATTERNS, "date of birth")?,
        if is_pas {
            require_unambiguous(headers, STATUS_PATTERNS, "MRP status")?
        } else {
            // Advisory in EMR files (never surfaced): lets the conflict guard
            // keep status-shaped columns out of the name fields without
            // making ambiguous EMR status headers a hard error.
            best_effort(headers, STATUS_PATTERNS)
        },
        if is_pas {
            require_unambiguous(headers, UPDATED_PATTERNS, "MRP updated")?
        } else {
            best_effort(headers, UPDATED_PATTERNS) // advisory, as above
        },
    ];
    resolve_conflicts(&mut claims);
    let [first, last, dob, status, updated] = claims;

    Ok(ColumnMapping {
        phn,
        first_name: first.map(|(i, _)| i),
        last_name: last.map(|(i, _)| i),
        dob: dob.map(|(i, _)| i),
        mrp_status: if is_pas { status.map(|(i, _)| i) } else { None },
        mrp_updated: if is_pas { updated.map(|(i, _)| i) } else { None },
    })
}

/// Best-effort detection of every column except PHN, whose index comes from
/// the manual column picker. Optional-field ambiguity resolves to the first
/// candidate (the picker only selects PHN, so ambiguity must not block the
/// run); the same exact-claim conflict guard as [`detect_columns`] applies.
pub fn detect_columns_with_user_phn(
    headers: &[String],
    is_pas: bool,
    phn_idx: usize,
) -> ColumnMapping {
    let mut claims = [
        best_effort(headers, FIRST_PATTERNS),
        best_effort(headers, LAST_PATTERNS),
        best_effort(headers, DOB_PATTERNS),
        // Advisory when not surfaced, as in detect_columns.
        best_effort(headers, STATUS_PATTERNS),
        best_effort(headers, UPDATED_PATTERNS),
    ];
    resolve_conflicts(&mut claims);
    let [first, last, dob, status, updated] = claims;

    ColumnMapping {
        phn: phn_idx,
        first_name: first.map(|(i, _)| i),
        last_name: last.map(|(i, _)| i),
        dob: dob.map(|(i, _)| i),
        mrp_status: if is_pas { status.map(|(i, _)| i) } else { None },
        mrp_updated: if is_pas { updated.map(|(i, _)| i) } else { None },
    }
}
