use pas_recon_engine::phn::{normalize_phn, is_valid_bc_phn};

#[test]
fn strips_spaces_hyphens_nbsp() {
    assert_eq!(normalize_phn("9876 543 219"), "9876543219");
    assert_eq!(normalize_phn("9876-543-219"), "9876543219");
    assert_eq!(normalize_phn("9876\u{00A0}543\u{00A0}219"), "9876543219");
    assert_eq!(normalize_phn(" 9876543219 "), "9876543219");
}

#[test]
fn valid_bc_phn_accepted() {
    // 9876543219 has valid MOD 11 check digit
    assert!(is_valid_bc_phn("9876543219"));
    assert!(is_valid_bc_phn("9876 543 219"));
    // 9123456785 has valid MOD 11 check digit
    assert!(is_valid_bc_phn("9123456785"));
}

#[test]
fn rejects_wrong_length() {
    assert!(!is_valid_bc_phn("987654321"));   // 9 digits
    assert!(!is_valid_bc_phn("98765432190")); // 11 digits
}

#[test]
fn rejects_wrong_start_digit() {
    assert!(!is_valid_bc_phn("1234567890")); // starts with 1, not 9
}

#[test]
fn rejects_non_numeric() {
    assert!(!is_valid_bc_phn("9876abc219"));
    assert!(!is_valid_bc_phn(""));
    assert!(!is_valid_bc_phn("abcdefghij"));
}

#[test]
fn rejects_invalid_check_digit() {
    // Same digits as 9876543219 but last digit changed
    assert!(!is_valid_bc_phn("9876543210")); // wrong check digit
    assert!(!is_valid_bc_phn("9876543218")); // wrong check digit
}
