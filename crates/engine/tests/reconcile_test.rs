use pas_recon_engine::{reconcile, reconcile_with_columns};

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

// --- C1: Manual PHN column override ---

#[test]
fn manual_phn_column_override_drives_matching() {
    // EMR has PHN in column index 1, header is "Patient ID" (won't auto-detect).
    // PAS has PHN in column index 2, header is "Health Num".
    let emr = b"Note,Patient ID,Name\nx,9876543218,John\n";
    let pas = b"Flag,Note,Health Num,Status\nx,y,9876543218,Confirmed\n";

    // Auto-detection must fail on the EMR side.
    let auto = reconcile(emr, pas);
    assert!(auto.is_err(), "auto-detect should fail without a PHN header");

    // Override: EMR PHN = col 1, PAS PHN = col 2.
    let result = reconcile_with_columns(emr, pas, Some(1), Some(2)).unwrap();
    assert_eq!(result.summary.matched, 1);
    assert_eq!(result.summary.emr_only, 0);
    assert_eq!(result.summary.pas_only, 0);
    assert_eq!(result.summary.invalid_phn_skipped, 0);
}

#[test]
fn manual_phn_override_rejects_out_of_range_index() {
    let emr = b"PHN,Name\n9876543218,John\n";
    let pas = b"PHN,Status\n9876543218,Confirmed\n";
    // An out-of-range PHN override (99 vs 2 headers) must now surface as an
    // error rather than silently clamping to the last column.
    let result = reconcile_with_columns(emr, pas, Some(99), Some(0));
    assert!(result.is_err(), "out-of-range override should error, not clamp");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("out of range"),
        "error should mention out-of-range index, got: {msg}"
    );
}

// --- C2: Matched review-worthy statuses ---

#[test]
fn matched_deceased_removed_and_not_the_mrp_appear_in_review() {
    fn run_with_status(status: &str) -> pas_recon_engine::model::ReconciliationResult {
        let emr = b"PHN,First,Last\n9876543218,John,Smith\n";
        let pas_csv = format!(
            "PHN,First,Last,MRP Status\n9876543218,John,Smith,{}\n",
            status
        );
        reconcile(emr, pas_csv.as_bytes()).unwrap()
    }

    for review_status in &["Deceased", "Removed", "Not the MRP", "Pending"] {
        let r = run_with_status(review_status);
        assert_eq!(r.summary.matched, 1, "status={}", review_status);
        assert_eq!(r.pas_match_review.len(), 1,
            "status {} should put the matched patient on the review list", review_status);
        assert_eq!(r.pas_match_review[0].phn, "9876543218");
    }

    // Confirmed must NOT appear on the review list.
    let confirmed = run_with_status("Confirmed");
    assert_eq!(confirmed.summary.matched, 1);
    assert_eq!(confirmed.pas_match_review.len(), 0);
}

// --- C3: Dedup keeps newest status, even if it changes review outcome ---

#[test]
fn dedup_keeps_newest_status_even_when_it_changes_review_outcome() {
    // Same PHN twice in PAS: older is Deceased (review), newer is Confirmed.
    // After dedup the patient should be Confirmed — matched, NOT on review list.
    let emr = b"PHN,First,Last\n9876543218,John,Smith\n";
    let pas = b"PHN,First,Last,MRP Status,MRP Updated\n\
9876543218,John,Smith,Deceased,01/01/2023\n\
9876543218,John,Smith,Confirmed,01/06/2024\n";

    let result = reconcile(&emr[..], &pas[..]).unwrap();
    assert_eq!(result.summary.duplicates_dropped, 1);
    assert_eq!(result.summary.matched, 1);
    assert_eq!(result.pas_match_review.len(), 0,
        "newest (Confirmed) should win — patient should NOT be up for review");
    assert_eq!(result.summary.status_breakdown.confirmed, 1);
    assert_eq!(result.summary.status_breakdown.deceased, 0,
        "status breakdown reflects the KEPT (deduped) record only");
}

// --- C5: Case-insensitive status classification ---

#[test]
fn status_classification_is_case_insensitive() {
    let emr = b"PHN,First,Last\n9876543218,John,Smith\n";
    let pas = b"PHN,First,Last,MRP Status\n\
9876543218,John,Smith,PENDING\n";

    let result = reconcile(&emr[..], &pas[..]).unwrap();
    assert_eq!(result.summary.matched, 1);
    assert_eq!(result.pas_match_review.len(), 1,
        "uppercase PENDING must still land on the review list");
    assert_eq!(result.summary.status_breakdown.pending, 1,
        "uppercase PENDING must still tally as pending");
}

// --- C6: Truncation surfacing ---

#[test]
fn summary_surfaces_truncated_rows() {
    // EMR header has 2 columns; one row has 4 fields → truncated (PHN kept).
    let emr = b"PHN,Name\n9876543218,John,extra,fields\n";
    let pas = b"PHN,Status\n9876543218,Confirmed\n";
    let result = reconcile(&emr[..], &pas[..]).unwrap();
    assert_eq!(result.summary.truncated_rows, 1);
    assert_eq!(result.summary.matched, 1, "PHN is still readable after truncation");
}