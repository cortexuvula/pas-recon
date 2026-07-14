use pas_recon_engine::parse::{parse_csv, ParsedCsv};

#[test]
fn parses_basic_csv_with_header_and_rows() {
    let csv_bytes = b"PHN,First Name,Last Name,DOB\n9876543210,John,Smith,1965-03-12\n9871111222,Mary,Jones,1978-11-04\n";
    let result: ParsedCsv = parse_csv(csv_bytes).unwrap();

    assert_eq!(result.headers, vec!["PHN", "First Name", "Last Name", "DOB"]);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].fields, vec!["9876543210", "John", "Smith", "1965-03-12"]);
    assert_eq!(result.rows[0].row_index, 0);
    assert_eq!(result.rows[1].row_index, 1);
}

#[test]
fn strips_bom_from_start_of_file() {
    let bom = "\u{FEFF}";
    let csv_bytes = format!("{bom}PHN,Name\n9876543210,John\n").into_bytes();
    let result = parse_csv(&csv_bytes).unwrap();
    assert_eq!(result.headers[0], "PHN"); // not "\u{FEFF}PHN"
}

#[test]
fn pads_short_rows_with_empty_strings() {
    let csv_bytes = b"A,B,C\n1,2\n";
    let result = parse_csv(csv_bytes).unwrap();
    assert_eq!(result.rows[0].fields, vec!["1", "2", ""]); // padded to 3
}

#[test]
fn ignores_extra_columns_beyond_header() {
    let csv_bytes = b"A,B\n1,2,3,4\n";
    let result = parse_csv(csv_bytes).unwrap();
    assert_eq!(result.rows[0].fields, vec!["1", "2"]); // truncated to header length
}

#[test]
fn handles_crlf_line_endings() {
    let csv_bytes = b"A,B\r\n1,2\r\n3,4\r\n";
    let result = parse_csv(csv_bytes).unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn returns_error_for_empty_input() {
    let result = parse_csv(b"");
    assert!(result.is_err());
}

#[test]
fn returns_error_for_header_only() {
    let result = parse_csv(b"A,B,C\n");
    assert!(result.is_err());
}
