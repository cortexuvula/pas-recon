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

pub fn reconcile(_emr_csv: &[u8], _pas_csv: &[u8]) -> Result<model::ReconciliationResult, model::EngineError> {
    todo!("implemented in Task 7")
}
