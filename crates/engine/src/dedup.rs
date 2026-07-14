//! PAS deduplication by PHN, keeping the record with the latest MRP-updated date.

use std::collections::HashMap;

use crate::model::PasRecord;

/// Deduplicate PAS records by PHN. For each group of duplicates, keep only
/// the record whose `mrp_updated` date is latest. Ties keep the first-seen.
/// Records with no date sort to the bottom of their group.
///
/// Returns (kept_records, duplicates_dropped_count). Kept records preserve
/// their original relative order.
pub fn deduplicate_pas(records: Vec<PasRecord>) -> (Vec<PasRecord>, usize) {
    // Group indices by PHN
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, rec) in records.iter().enumerate() {
        groups.entry(rec.phn.clone()).or_default().push(idx);
    }

    let mut keep_indices: Vec<usize> = Vec::new();
    let mut dropped = 0usize;

    for (_, indices) in &groups {
        if indices.len() == 1 {
            keep_indices.push(indices[0]);
        } else {
            // Find the index of the newest record (or first-seen on tie)
            let winner = indices
                .iter()
                .copied()
                .reduce(|best, candidate| {
                    let best_date = records[best].mrp_updated;
                    let cand_date = records[candidate].mrp_updated;
                    match (best_date, cand_date) {
                        (Some(b), Some(c)) => {
                            if c > b { candidate } else { best }
                        }
                        (None, Some(_)) => candidate,
                        _ => best, // both None or best is Some → keep best (first-seen)
                    }
                })
                .unwrap();

            keep_indices.push(winner);
            dropped += indices.len() - 1;
        }
    }

    // Sort keep_indices to preserve original row order
    keep_indices.sort_unstable();

    let kept = keep_indices
        .into_iter()
        .map(|idx| records[idx].clone())
        .collect();

    (kept, dropped)
}
