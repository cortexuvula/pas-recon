use pas_recon_engine::date::{parse_mrp_date, serial_to_date};
use chrono::NaiveDate;

#[test]
fn parses_iso_date() {
    assert_eq!(
        parse_mrp_date("2024-03-15"),
        NaiveDate::from_ymd_opt(2024, 3, 15)
    );
}

#[test]
fn parses_dmy_slash_date() {
    assert_eq!(
        parse_mrp_date("15/03/2024"),
        NaiveDate::from_ymd_opt(2024, 3, 15)
    );
}

#[test]
fn parses_excel_serial_number() {
    // Excel serial 45366 = 2024-03-15
    assert_eq!(
        parse_mrp_date("45366"),
        NaiveDate::from_ymd_opt(2024, 3, 15)
    );
}

#[test]
fn parses_actual_number_type() {
    assert_eq!(
        serial_to_date(45366.0),
        NaiveDate::from_ymd_opt(2024, 3, 15)
    );
}

#[test]
fn returns_none_for_garbage() {
    assert_eq!(parse_mrp_date("not a date"), None);
    assert_eq!(parse_mrp_date(""), None);
}

#[test]
fn returns_none_for_impossible_date() {
    assert_eq!(parse_mrp_date("31/02/2024"), None); // Feb 31 doesn't exist
}
