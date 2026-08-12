//! Data model for the reconciliation engine.

/// The complete output of a reconciliation run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReconciliationResult {
    pub summary: Summary,
    pub emr_no_match: Vec<DisplayRow>,
    pub pas_match_review: Vec<DisplayRow>,
    pub pas_no_match: Vec<DisplayRow>,
    pub invalid_phns: Vec<DisplayRow>,
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DisplayRow {
    pub phn: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub dob: Option<String>,
    pub mrp_status: Option<String>,
    pub raw_fields: Vec<String>,
    /// Which file this row came from ("EMR" or "PAS"). Only set for
    /// invalid_phns where provenance isn't obvious from the list name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
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

    /// A non-PHN field matched multiple columns equally well.
    AmbiguousColumn {
        source: String,
        field: String,
        candidates: Vec<String>,
    },

    /// A user-provided PHN column index was out of range for the file's headers.
    InvalidColumnIndex {
        source: String,
        index: usize,
        header_count: usize,
    },
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
            EngineError::AmbiguousColumn { source, field, candidates } => {
                write!(
                    f,
                    "multiple {field} columns in {source} CSV: {candidates:?}"
                )
            }
            EngineError::InvalidColumnIndex {
                source,
                index,
                header_count,
            } => {
                write!(
                    f,
                    "selected {source} PHN column index {index} is out of range (file has {header_count} columns)"
                )
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

/// Which source column index maps to each recognized field.
/// Only `phn` is required; others are `None` if not detected.
#[derive(Debug, Clone)]
pub struct ColumnMapping {
    pub phn: usize,
    pub first_name: Option<usize>,
    pub last_name: Option<usize>,
    pub dob: Option<usize>,
    pub mrp_status: Option<usize>,   // PAS only
    pub mrp_updated: Option<usize>,  // PAS only
}

/// A validated PAS record ready for dedup + matching.
#[derive(Debug, Clone)]
pub struct PasRecord {
    pub phn: String,
    pub mrp_status: Option<String>,
    pub mrp_updated: Option<chrono::NaiveDate>,
    pub raw_fields: Vec<String>,
    pub row_index: usize,
}

/// A validated EMR record ready for matching.
#[derive(Debug, Clone)]
pub struct EmrRecord {
    pub phn: String,
    pub raw_fields: Vec<String>,
    pub row_index: usize,
}
