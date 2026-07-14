//! Data model for the reconciliation engine.

/// The complete output of a reconciliation run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReconciliationResult {
    pub summary: Summary,
    pub emr_no_match: Vec<DisplayRow>,
    pub pas_match_review: Vec<DisplayRow>,
    pub pas_no_match: Vec<DisplayRow>,
}

/// Aggregate counts shown in the sidebar.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Summary {
    pub matched: usize,
    pub emr_only: usize,
    pub pas_only: usize,
    pub pas_review: usize,
    pub status_breakdown: StatusBreakdown,
    pub duplicates_dropped: usize,
    pub invalid_phn_skipped: usize,
    pub unparseable_dates: usize,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct StatusBreakdown {
    pub confirmed: usize,
    pub pending: usize,
    pub deceased: usize,
    pub removed: usize,
    pub not_the_mrp: usize,
}

/// One row in an output list.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DisplayRow {
    pub phn: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub dob: Option<String>,
    pub mrp_status: Option<String>,
    pub raw_fields: Vec<String>,
}

/// Errors that abort a reconciliation run.
///
/// Note: this enum intentionally does NOT use `thiserror::Error`. Its fields
/// are named `source` to mean "which CSV (EMR/PAS)", but thiserror reserves
/// the field name `source` for the `std::error::Error::source()` chain, which
/// requires a value implementing `std::error::Error` (a `String` does not).
/// The downstream tasks (4-8) construct these variants using the `source`
/// field name, so we keep the name and implement `Display`/`Error` manually.
#[derive(Debug, Clone, serde::Serialize)]
pub enum EngineError {
    Io { source: String, message: String },

    CsvParse { source: String, row: usize, message: String },

    MissingPhnColumn { source: String },

    AmbiguousPhnColumns { source: String, candidates: Vec<String> },
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Io { source, message } => {
                write!(f, "failed to read {source} file: {message}")
            }
            EngineError::CsvParse { source, row, message } => {
                write!(f, "CSV parse error in {source} at row {row}: {message}")
            }
            EngineError::MissingPhnColumn { source } => {
                write!(f, "could not find a PHN column in {source} CSV")
            }
            EngineError::AmbiguousPhnColumns { source, candidates } => {
                write!(f, "multiple columns in {source} CSV look like PHNs: {candidates:?}")
            }
        }
    }
}

impl std::error::Error for EngineError {}

/// Which CSV file an error or record belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CsvSource {
    Emr,
    Pas,
}

impl std::fmt::Display for CsvSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvSource::Emr => write!(f, "EMR"),
            CsvSource::Pas => write!(f, "PAS"),
        }
    }
}

/// One parsed CSV row before column mapping. All fields are raw strings.
#[derive(Debug, Clone)]
pub struct RawRow {
    pub fields: Vec<String>,
    pub row_index: usize, // 0-based, excluding header
}
