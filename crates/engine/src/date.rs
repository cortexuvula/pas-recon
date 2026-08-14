//! Date parsing for PAS MRP-updated dates.
//!
//! Handles three formats the spreadsheet dealt with:
//! - ISO: "2024-03-15"
//! - D/M/YYYY: "15/3/2024"
//! - Excel serial number: 45366 → 2024-03-15

use chrono::NaiveDate;

/// Excel epoch: December 30, 1899 (the Excel serial-day 0, accounting for
/// the 1900 leap-year bug in Excel's default 1900 date system).
const EXCEL_EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();

/// Convert an Excel serial date number to a NaiveDate.
///
/// Uses the 1899-12-30 epoch, which is exactly correct for serials >= 61
/// (1900-03-01 onward). Excel wrongly treats 1900 as a leap year, so serials
/// 1..=60 render as 1900-01-01 .. 1900-02-29 in Excel but are off by one day
/// under this epoch (and serial 60 is the non-existent 1900-02-29). Those are
/// not real MRP dates, so we reject serial < 61 rather than return a wrong day.
pub fn serial_to_date(serial: f64) -> Option<NaiveDate> {
    // Reject NaN/inf first. NaN comparisons against the bounds below are always
    // false, so without this guard NaN would pass and `NaN as i64` becomes 0,
    // silently yielding the Excel epoch date. is_finite() also excludes
    // +/- infinity.
    if !serial.is_finite() {
        return None;
    }
    if serial < 61.0 || serial > 100000.0 {
        return None; // sanity bounds; < 61 covers the 1900 fictitious-Feb-29 range
    }
    let days = serial.floor() as i64;
    EXCEL_EPOCH.checked_add_days(chrono::Days::new(days as u64))
}

/// Parse a date string that could be ISO, D/M/YYYY, or an Excel serial number.
/// Returns None if it can't be parsed (the caller treats this as "keep first seen").
pub fn parse_mrp_date(raw: &str) -> Option<NaiveDate> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try ISO first: YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(date);
    }

    // Try D/M/YYYY (the spreadsheet's format)
    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() == 3 {
            let year_str = parts[2];
            // Require a literal 4-digit year. Two-digit years ("15/03/24")
            // previously parsed as year 24 AD — older than every real date,
            // silently corrupting dedup ordering. Rejecting them routes the
            // row into the unparseable-dates warning instead.
            if year_str.len() != 4 || !year_str.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let day: u32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            let year: i32 = year_str.parse().ok()?;
            return NaiveDate::from_ymd_opt(year, month, day);
        }
    }

    // Try Excel serial number (purely numeric)
    if trimmed.chars().all(|c| c.is_ascii_digit() || c == '.') {
        if let Ok(serial) = trimmed.parse::<f64>() {
            return serial_to_date(serial);
        }
    }

    None
}
