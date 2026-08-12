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
    // Track only the winning index per PHN, instead of storing every index.
    let mut winner_by_phn: HashMap<String, usize> = HashMap::new();
    let mut dropped = 0usize;

    for (idx, rec) in records.iter().enumerate() {
        match winner_by_phn.get(&rec.phn) {
            None => {
                winner_by_phn.insert(rec.phn.clone(), idx);
            }
            Some(&best) => {
                // Keep the record with the newest `mrp_updated` date. Ties and
                // missing dates resolve to first-seen (the current best).
                let best_date = records[best].mrp_updated;
                let cand_date = rec.mrp_updated;
                let prefer_candidate = match (best_date, cand_date) {
                    (Some(b), Some(c)) => c > b,
                    (None, Some(_)) => true, // candidate has a date, best doesn't
                    _ => false,              // best has a date, or both None
                };
                if prefer_candidate {
                    winner_by_phn.insert(rec.phn.clone(), idx);
                }
                dropped += 1;
            }
        }
    }

    // Winning indices, restored to original row order.
    let mut keep_indices: Vec<usize> = winner_by_phn.into_values().collect();
    keep_indices.sort_unstable();

    // Move owned records out by index instead of cloning each kept record.
    // Wrapping in Option lets us take() each winner; non-winners are dropped
    // when `slots` goes out of scope.
    let mut slots: Vec<Option<PasRecord>> = records.into_iter().map(Some).collect();
    let kept = keep_indices
        .into_iter()
        .map(|idx| slots[idx].take().expect("winner index always valid"))
        .collect();

    (kept, dropped)
}
