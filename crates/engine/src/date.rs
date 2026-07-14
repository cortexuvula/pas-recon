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
pub fn serial_to_date(serial: f64) -> Option<NaiveDate> {
    if serial < 1.0 || serial > 100000.0 {
        return None; // sanity bounds
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
            let day: u32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            let year: i32 = parts[2].parse().ok()?;
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
