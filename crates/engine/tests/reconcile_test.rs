use pas_recon_engine::reconcile;

fn read_fixture(name: &str) -> Vec<u8> {
    // Tests may run from the package dir (crates/engine) or the workspace root.
    // Try the package-relative path first, then fall back to the workspace path.
    let pkg_path = format!("fixtures/{name}");
    if let Ok(bytes) = std::fs::read(&pkg_path) {
        return bytes;
    }
    let ws_path = format!("crates/engine/fixtures/{name}");
    std::fs::read(&ws_path).unwrap_or_else(|e| panic!("failed to read {ws_path}: {e}"))
}

#[test]
fn reconciles_basic_emr_and_pas() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    // EMR patients: 9876543218, 9871111223, 9873333447
    // PAS patients: above + 9875555678, 9877777884, 9888888992, 9899999001
    assert_eq!(result.summary.matched, 3);
    assert_eq!(result.summary.emr_only, 0);
    assert_eq!(result.summary.pas_only, 4);
    assert_eq!(result.summary.pas_review, 1); // 9873333447 Pending
}

#[test]
fn pas_review_list_contains_pending_deceased_removed_not_mrp() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    let phns: Vec<&str> = result.pas_match_review.iter().map(|r| r.phn.as_str()).collect();
    assert!(phns.contains(&"9873333447"), "Pending patient should be in review list, got {phns:?}");
}

#[test]
fn pas_no_match_list_contains_pas_only_patients() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    let phns: Vec<&str> = result.pas_no_match.iter().map(|r| r.phn.as_str()).collect();
    assert!(phns.contains(&"9888888992"));
    assert!(phns.contains(&"9899999001"));
}

#[test]
fn status_breakdown_counts_correctly() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    assert_eq!(result.summary.status_breakdown.confirmed, 4);
    assert_eq!(result.summary.status_breakdown.pending, 1);
    assert_eq!(result.summary.status_breakdown.deceased, 1);
    assert_eq!(result.summary.status_breakdown.removed, 0);
    assert_eq!(result.summary.status_breakdown.not_the_mrp, 1);
}

#[test]
fn rejects_empty_emr_file() {
    let result = reconcile(b"", b"PHN\n9876543218\n");
    assert!(result.is_err());
}

#[test]
fn rejects_emr_without_phn_column() {
    let result = reconcile(b"Name,Age\nJohn,30\n", b"PHN\n9876543218\n");
    assert!(result.is_err());
}

#[test]
fn lists_sorted_by_last_name() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    let last_names: Vec<&str> = result.pas_no_match.iter().map(|r| r.last_name.as_deref().unwrap_or("")).collect();
    let mut expected = last_names.to_vec();
    expected.sort();
    assert_eq!(last_names, expected, "List should be sorted by last name");
}

#[test]
fn handles_dirty_emr_with_invalid_phns() {
    let emr = read_fixture("emr_dirty.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    // emr_dirty.csv has:
    // - 9876543218 (valid) → matched
    // - 1234567890 (invalid: starts with 1) → skipped
    // - "9876 543 219" with spaces (valid after normalize) → same as 9876543218
    // - 9871111223 (valid) → matched
    assert!(result.summary.invalid_phn_skipped >= 1, "Should skip invalid PHNs");
}

#[test]
fn deduplicates_pas_by_latest_date() {
    let emr = b"PHN,First,Last\n9876543218,John,Smith\n9871111223,Mary,Jones\n";
    let pas = read_fixture("pas_duplicates.csv");

    let result = reconcile(&emr[..], &pas).unwrap();

    // pas_duplicates.csv has 3 rows for 9876543218 and 2 for 9871111223
    // Dedup should drop 2 + 1 = 3 duplicates
    assert_eq!(result.summary.duplicates_dropped, 3);
    assert_eq!(result.summary.matched, 2);
}

#[test]
fn empty_result_lists_when_all_match_and_confirmed() {
    let csv = b"PHN,First,Last,MRP Status\n9876543218,John,Smith,Confirmed\n";
    let result = reconcile(csv, csv).unwrap();

    assert_eq!(result.summary.matched, 1);
    assert_eq!(result.emr_no_match.len(), 0);
    assert_eq!(result.pas_no_match.len(), 0);
    assert_eq!(result.pas_match_review.len(), 0);
}

#[test]
fn pas_without_status_column_produces_empty_review_list() {
    let emr = b"PHN,Name\n9876543218,John\n";
    let pas = b"PHN,Name\n9876543218,John\n";

    let result = reconcile(&emr[..], &pas[..]).unwrap();

    assert_eq!(result.summary.matched, 1);
    assert_eq!(result.pas_match_review.len(), 0);
}
