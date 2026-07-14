use pas_recon_engine::detect::{detect_columns, DetectionError};
use pas_recon_engine::parse::parse_csv;

fn headers_csv(headers: &[&str]) -> Vec<u8> {
    let mut s = headers.join(",");
    s.push_str("\n,x,\n"); // one data row so parse doesn't complain
    s.into_bytes()
}

#[test]
fn detects_phn_by_exact_match() {
    let parsed = parse_csv(&headers_csv(&["PHN", "First", "Last"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.phn, 0);
}

#[test]
fn detects_phn_by_case_insensitive_match() {
    let parsed = parse_csv(&headers_csv(&["first name", "phn", "last"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.phn, 1);
}

#[test]
fn detects_phn_by_long_form() {
    let parsed = parse_csv(&headers_csv(&["Personal Health Number", "Name"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.phn, 0);
}

#[test]
fn detects_all_fields_at_once() {
    let parsed = parse_csv(&headers_csv(&["PHN", "First Name", "Last Name", "DOB", "Status", "MRP Updated"])).unwrap();
    let mapping = detect_columns(&parsed.headers, true).unwrap(); // true = PAS mode
    assert_eq!(mapping.phn, 0);
    assert_eq!(mapping.first_name, Some(1));
    assert_eq!(mapping.last_name, Some(2));
    assert_eq!(mapping.dob, Some(3));
    assert_eq!(mapping.mrp_status, Some(4));
    assert_eq!(mapping.mrp_updated, Some(5));
}

#[test]
fn missing_phn_returns_error() {
    let parsed = parse_csv(&headers_csv(&["Name", "Age", "City"])).unwrap();
    let result = detect_columns(&parsed.headers, false);
    assert!(matches!(result, Err(DetectionError::MissingPhnColumn)));
}

#[test]
fn ambiguous_phn_columns_returns_error() {
    let parsed = parse_csv(&headers_csv(&["PHN", "Personal Health Number"])).unwrap();
    let result = detect_columns(&parsed.headers, false);
    assert!(matches!(result, Err(DetectionError::AmbiguousPhnColumns { .. })));
}

#[test]
fn pas_fields_not_detected_in_emr_mode() {
    // EMR mode (is_pas=false) should not try to detect status/updated
    let parsed = parse_csv(&headers_csv(&["PHN", "Status", "MRP Updated"])).unwrap();
    let mapping = detect_columns(&parsed.headers, false).unwrap();
    assert_eq!(mapping.mrp_status, None);
    assert_eq!(mapping.mrp_updated, None);
}
