use pas_recon_engine::phn::{normalize_phn, is_valid_bc_phn};

#[test]
fn strips_spaces_hyphens_nbsp() {
    assert_eq!(normalize_phn("9876 543 218"), "9876543218");
    assert_eq!(normalize_phn("9876-543-218"), "9876543218");
    assert_eq!(normalize_phn("9876\u{00A0}543\u{00A0}218"), "9876543218");
    assert_eq!(normalize_phn(" 9876543218 "), "9876543218");
}

#[test]
fn valid_bc_phn_accepted() {
    // Official BC TELEPLAN sample PHN
    assert!(is_valid_bc_phn("9012372173"));
    // Generated valid PHNs
    assert!(is_valid_bc_phn("9876543218"));
    assert!(is_valid_bc_phn("9876 543 218"));
    assert!(is_valid_bc_phn("9123456798"));
}

#[test]
fn rejects_wrong_length() {
    assert!(!is_valid_bc_phn("987654321"));   // 9 digits
    assert!(!is_valid_bc_phn("98765432180")); // 11 digits
}

#[test]
fn rejects_wrong_start_digit() {
    assert!(!is_valid_bc_phn("1234567890")); // starts with 1, not 9
}

#[test]
fn rejects_non_numeric() {
    assert!(!is_valid_bc_phn("9876abc218"));
    assert!(!is_valid_bc_phn(""));
    assert!(!is_valid_bc_phn("abcdefghij"));
}

#[test]
fn rejects_invalid_check_digit() {
    // Same first 9 digits as 9876543218 but wrong check digit
    assert!(!is_valid_bc_phn("9876543210"));
    assert!(!is_valid_bc_phn("9876543219"));
}
