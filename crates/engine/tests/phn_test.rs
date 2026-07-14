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
