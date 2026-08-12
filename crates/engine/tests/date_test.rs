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

#[test]
fn two_digit_slash_year_parses_as_literal_year() {
    // Documents current behaviour: 2-digit years parse literally (year 24 AD),
    // NOT as 2024. This is a known limitation, not a feature — the spreadsheet
    // used RIGHT(G8, 4) to extract the full year.
    assert_eq!(
        parse_mrp_date("15/03/24"),
        NaiveDate::from_ymd_opt(24, 3, 15),
        "2-digit slash years are NOT promoted to 2000s"
    );
}

#[test]
fn single_digit_day_and_month_in_slash_date() {
    // Real PAS exports may not zero-pad. Confirm "5/3/2024" parses.
    assert_eq!(
        parse_mrp_date("5/3/2024"),
        NaiveDate::from_ymd_opt(2024, 3, 5)
    );
}

#[test]
fn rejects_nan_serial() {
    assert!(serial_to_date(f64::NAN).is_none());
    assert!(serial_to_date(f64::INFINITY).is_none());
    assert!(serial_to_date(f64::NEG_INFINITY).is_none());
}

#[test]
fn rejects_excel_1900_off_by_one_range() {
    // Serials 1..=60 map to Excel's 1900-01-01 .. fictitious 1900-02-29, which
    // are off by one day under the 1899-12-30 epoch (and serial 60 is the
    // non-existent Feb 29). These can't be real MRP dates, so reject them
    // rather than return a silently-wrong day.
    assert!(serial_to_date(1.0).is_none());
    assert!(serial_to_date(60.0).is_none());
    assert!(parse_mrp_date("60").is_none());
    // Serial 61 = 1900-03-01, the first serial the epoch maps exactly.
    assert_eq!(serial_to_date(61.0), NaiveDate::from_ymd_opt(1900, 3, 1));
}
