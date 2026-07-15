//! BC PHN normalization and validation.
//!
//! A valid BC PHN is exactly 10 digits, first digit 9, with a valid
//! MOD 11 check digit in position 10. Normalization strips spaces,
//! hyphens, and non-breaking spaces before checking.

/// Strip spaces, hyphens, non-breaking spaces, and surrounding whitespace
/// from a PHN-like string, leaving only the digits (and any other chars).
/// Also collapses internal whitespace runs (e.g. "9876  543  210").
pub fn normalize_phn(raw: &str) -> String {
    raw.chars()
        .filter(|c| !matches!(c, ' ' | '-' | '\u{00A0}' | '\t'))
        .collect()
}

/// Verify the MOD 11 check digit for a 10-digit numeric string.
///
/// The BC PHN uses a weighted MOD 11 scheme: digits 1–9 are multiplied
/// by weights 2,1,6,3,5,4,8,7,2 (left to right), summed, and the check
/// digit (position 10) must equal 11 - (sum % 11), or 0 if that's 11.
fn verify_mod11(digits: &[u8; 10]) -> bool {
    const WEIGHTS: [u32; 9] = [2, 1, 6, 3, 5, 4, 8, 7, 2];
    let sum: u32 = digits[..9]
        .iter()
        .zip(WEIGHTS.iter())
        .map(|(&d, &w)| (d as u32) * w)
        .sum();
    let remainder = sum % 11;
    let check = (11 - remainder) % 11; // 0 if remainder == 0
    check as u8 == digits[9]
}

/// Returns true if the raw string, after normalization, is a valid BC PHN:
/// exactly 10 digits, first digit 9, valid MOD 11 check digit.
pub fn is_valid_bc_phn(raw: &str) -> bool {
    let normalized = normalize_phn(raw);
    if normalized.len() != 10 || !normalized.starts_with('9') {
        return false;
    }
    let digits: [u8; 10] = match normalized
        .bytes()
        .map(|b| b.wrapping_sub(b'0'))
        .collect::<Vec<_>>()
        .try_into()
    {
        Ok(arr) => arr,
        Err(_) => return false,
    };
    if digits.iter().any(|&d| d > 9) {
        return false;
    }
    verify_mod11(&digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_strips_separators() {
        assert_eq!(normalize_phn("9876 543 210"), "9876543210");
        assert_eq!(normalize_phn("9876-543-210"), "9876543210");
        assert_eq!(normalize_phn("9876\u{00A0}543\u{00A0}210"), "9876543210");
        assert_eq!(normalize_phn(" 9876543210 "), "9876543210");
    }

    #[test]
    fn test_valid_bc_phn_with_valid_checksum() {
        // 9876543210 — let's verify the check digit
        // Digits: 9 8 7 6 5 4 3 2 1 | check=0
        // Weights:2 1 6 3 5 4 8 7 2
        // 18+8+42+18+25+16+24+14+2 = 167; 167%11=2; check=(11-2)%11=9 ≠ 0
        assert!(!is_valid_bc_phn("9876543210")); // known test fixture, not a real PHN
    }

    #[test]
    fn test_rejects_wrong_length() {
        assert!(!is_valid_bc_phn("987654321"));
        assert!(!is_valid_bc_phn("98765432101"));
    }

    #[test]
    fn test_rejects_wrong_start_digit() {
        assert!(!is_valid_bc_phn("1234567890"));
    }

    #[test]
    fn test_rejects_non_numeric() {
        assert!(!is_valid_bc_phn("9876abc210"));
        assert!(!is_valid_bc_phn(""));
    }

    #[test]
    fn test_mod11_verification() {
        // Construct a PHN with a known-valid check digit
        // First 9 digits: 9 8 7 6 5 4 3 2 1
        // Weighted sum: 18+8+42+18+25+16+24+14+2 = 167
        // 167 % 11 = 2; check = (11-2) % 11 = 9
        // So valid PHN: 9876543219
        assert!(is_valid_bc_phn("9876543219"));
        assert!(!is_valid_bc_phn("9876543218")); // wrong check digit
    }
}
