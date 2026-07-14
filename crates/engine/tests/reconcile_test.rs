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

    // EMR patients: 9876543210, 9871111222, 9873333444
    // PAS patients: above + 9875555666, 9877777888, 9888888999, 9899999000
    assert_eq!(result.summary.matched, 3);
    assert_eq!(result.summary.emr_only, 0);
    assert_eq!(result.summary.pas_only, 4);
    assert_eq!(result.summary.pas_review, 1); // 9873333444 Pending
}

#[test]
fn pas_review_list_contains_pending_deceased_removed_not_mrp() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    let phns: Vec<&str> = result.pas_match_review.iter().map(|r| r.phn.as_str()).collect();
    assert!(phns.contains(&"9873333444"), "Pending patient should be in review list, got {phns:?}");
}

#[test]
fn pas_no_match_list_contains_pas_only_patients() {
    let emr = read_fixture("emr_basic.csv");
    let pas = read_fixture("pas_basic.csv");

    let result = reconcile(&emr, &pas).unwrap();

    let phns: Vec<&str> = result.pas_no_match.iter().map(|r| r.phn.as_str()).collect();
    assert!(phns.contains(&"9888888999"));
    assert!(phns.contains(&"9899999000"));
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
    let result = reconcile(b"", b"PHN\n9876543210\n");
    assert!(result.is_err());
}

#[test]
fn rejects_emr_without_phn_column() {
    let result = reconcile(b"Name,Age\nJohn,30\n", b"PHN\n9876543210\n");
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
