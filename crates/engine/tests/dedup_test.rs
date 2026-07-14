use pas_recon_engine::dedup::deduplicate_pas;
use pas_recon_engine::model::PasRecord;
use chrono::NaiveDate;

fn pas(phn: &str, date: Option<NaiveDate>, row: usize) -> PasRecord {
    PasRecord {
        phn: phn.to_string(),
        mrp_status: Some("Confirmed".to_string()),
        mrp_updated: date,
        raw_fields: vec![],
        row_index: row,
    }
}

#[test]
fn keeps_single_record_unchanged() {
    let records = vec![pas("9876543210", None, 0)];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(dropped, 0);
}

#[test]
fn keeps_newest_when_duplicates_have_dates() {
    let records = vec![
        pas("9876543210", NaiveDate::from_ymd_opt(2023, 1, 1), 0),
        pas("9876543210", NaiveDate::from_ymd_opt(2024, 6, 15), 1), // newest
        pas("9876543210", NaiveDate::from_ymd_opt(2023, 9, 3), 2),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].row_index, 1); // the newest one
    assert_eq!(dropped, 2);
}

#[test]
fn keeps_first_seen_when_dates_are_equal() {
    let date = NaiveDate::from_ymd_opt(2024, 1, 1);
    let records = vec![
        pas("9876543210", date, 0),
        pas("9876543210", date, 1),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].row_index, 0); // first seen
    assert_eq!(dropped, 1);
}

#[test]
fn keeps_first_seen_when_no_dates_present() {
    let records = vec![
        pas("9876543210", None, 0),
        pas("9876543210", None, 1),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].row_index, 0);
    assert_eq!(dropped, 1);
}

#[test]
fn preserves_distinct_phns() {
    let records = vec![
        pas("9876543210", None, 0),
        pas("9111222333", None, 1),
        pas("9222333444", None, 2),
    ];
    let (kept, dropped) = deduplicate_pas(records);
    assert_eq!(kept.len(), 3);
    assert_eq!(dropped, 0);
}
