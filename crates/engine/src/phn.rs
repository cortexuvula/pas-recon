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
