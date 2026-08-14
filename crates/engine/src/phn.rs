//! BC PHN normalization and validation.
//!
//! A valid BC PHN is exactly 10 digits, first digit 9, with a valid
//! MOD 11 check digit in position 10. Normalization strips spaces,
//! hyphens, and non-breaking spaces before checking.

/// Remove whitespace and hyphens from a PHN-like string, leaving every other
/// character in place. Validation ([`is_valid_bc_phn`]) separately rejects the
/// result if it isn't exactly 10 digits.
///
/// This covers the full range of separators seen when PHNs are pasted from
/// spreadsheets, PDFs, or web pages: ASCII spaces and tabs, non-breaking
/// spaces, newlines, and other Unicode whitespace, plus hyphens. Characters
/// are removed (not collapsed), so `"9876  543  210"` becomes `"9876543210"`.
pub fn normalize_phn(raw: &str) -> String {
    raw.chars().filter(|c| !c.is_whitespace() && *c != '-').collect()
}

/// Verify the MOD 11 check digit for a 10-digit BC PHN.
///
/// Per BC Government TELEPLAN specification (section 1.14.2):
/// - Weights for positions 1-9: [0, 2, 4, 8, 5, 10, 9, 7, 3]
/// - Multiply each digit by its weight, sum the results
/// - remainder = sum % 11
/// - check digit = 11 - remainder
/// - If remainder is 0, check digit is 11 (invalid — can't be single digit)
/// - Example: PHN 9012372173 → weighted sum = 151, 151/11=13r8, 11-8=3 ✓
///
/// Note: BC's routine has NO final `% 11` (unlike generic ISBN-style MOD-11,
/// where remainder 0 yields check digit 0). Remainder 0 → check digit 11 →
/// invalid, and such numbers are never issued. Cross-referenced against
/// EYDS-CA/phn-validator, which implements the BC Client Registry vol-4b
/// rules the same way — see `test_remainder_zero_and_one_are_invalid`.
fn verify_mod11(digits: &[u8; 10]) -> bool {
    const WEIGHTS: [u32; 9] = [0, 2, 4, 8, 5, 10, 9, 7, 3];
    let sum: u32 = digits[..9]
        .iter()
        .zip(WEIGHTS.iter())
        .map(|(&d, &w)| (d as u32) * w)
        .sum();
    let remainder = sum % 11;
    let check = 11 - remainder;
    // If remainder is 0, check would be 11 (can't be a single digit → invalid PHN)
    if check >= 10 {
        return false;
    }
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
    fn test_normalize_strips_any_whitespace_runs() {
        // Tabs, newlines, and multi-space runs are all removed (not collapsed).
        assert_eq!(normalize_phn("9876\t543\n210"), "9876543210");
        assert_eq!(normalize_phn("9876  543  210"), "9876543210");
        assert_eq!(normalize_phn("9876\r\n543 210"), "9876543210");
    }

    #[test]
    fn test_known_fixture_not_a_real_phn() {
        // 9876543210 is a test fixture, not a real BC PHN — it fails MOD-11.
        // Weights: [0, 2, 4, 8, 5, 10, 9, 7, 3]
        // 9*0+8*2+7*4+6*8+5*5+4*10+3*9+2*7+1*3 = 0+16+28+48+25+40+27+14+3 = 201
        // 201 % 11 = 3; check = 11 - 3 = 8 ≠ 0 (the last digit)
        assert!(!is_valid_bc_phn("9876543210"));
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
        // Official BC TELEPLAN sample PHN (section 1.14.2)
        // Weights: [0, 2, 4, 8, 5, 10, 9, 7, 3]
        // 9*0 + 0*2 + 1*4 + 2*8 + 3*5 + 7*10 + 2*9 + 1*7 + 7*3 = 0+0+4+16+15+70+18+7+21 = 151
        // 151 % 11 = 8; check = 11 - 8 = 3
        assert!(is_valid_bc_phn("9012372173"));
        assert!(!is_valid_bc_phn("9012372174")); // wrong check digit
    }

    #[test]
    fn test_remainder_zero_and_one_are_invalid() {
        // Locks the BC-specific MOD-11 semantics: check = 11 - remainder with
        // NO final % 11 (unlike ISBN-style MOD-11, where remainder 0 would
        // yield check digit 0 and be valid). Cross-checked against
        // EYDS-CA/phn-validator (implements the BC Client Registry vol-4b
        // application rules) and the TELEPLAN manual §1.14.2 summary. If this
        // test ever needs to change, change it together with the rule and
        // cite a BC source.
        //
        // "9000000000": all weighted digits 0 → sum 0 → remainder 0 → check
        // would be 11 — not a single digit, so invalid.
        assert!(!is_valid_bc_phn("9000000000"));
        // "9000000040": 4*3 = 12 ≡ 1 (mod 11) → check would be 10 — likewise
        // impossible as a single digit.
        assert!(!is_valid_bc_phn("9000000040"));
    }
}
