//! PAS Reconciliation Engine — pure Rust reconciliation logic.
//!
//! Takes two CSV byte slices (EMR panel + PAS patient list), matches patients
//! by PHN, and returns three review lists plus a summary.

pub mod date;
pub mod dedup;
pub mod detect;
pub mod model;
pub mod parse;
pub mod phn;
pub mod reconcile;

pub use model::{ReconciliationResult, EngineError};

/// Run reconciliation with auto-detected columns.
pub fn reconcile(emr_csv: &[u8], pas_csv: &[u8]) -> Result<ReconciliationResult, EngineError> {
    reconcile::reconcile_with_columns(emr_csv, pas_csv, None, None)
}

/// Run reconciliation with caller-provided PHN column overrides.
pub fn reconcile_with_columns(
    emr_csv: &[u8],
    pas_csv: &[u8],
    emr_phn_column: Option<usize>,
    pas_phn_column: Option<usize>,
) -> Result<ReconciliationResult, EngineError> {
    reconcile::reconcile_with_columns(emr_csv, pas_csv, emr_phn_column, pas_phn_column)
}
