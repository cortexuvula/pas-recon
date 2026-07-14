//! CSV parsing into raw rows. Handles BOM, flexible column counts, CRLF.

use crate::model::RawRow;

/// The result of parsing a CSV: headers + data rows.
#[derive(Debug, Clone)]
pub struct ParsedCsv {
    pub headers: Vec<String>,
    pub rows: Vec<RawRow>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("empty file — no content")]
    Empty,
    #[error("file contains only a header row, no data")]
    HeaderOnly,
    #[error("CSV read error at line {line}: {message}")]
    Read { line: usize, message: String },
}

/// Parse CSV bytes into headers + raw rows.
///
/// - Strips a leading BOM.
/// - Pads rows shorter than the header with empty strings.
/// - Truncates rows longer than the header.
pub fn parse_csv(input: &[u8]) -> Result<ParsedCsv, ParseError> {
    // Strip BOM if present
    let input = input.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(input);

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true) // allow variable column counts
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(input);

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| ParseError::Read { line: 1, message: e.to_string() })?
        .iter()
        .map(|s| s.to_string())
        .collect();

    if headers.is_empty() {
        return Err(ParseError::Empty);
    }

    let header_len = headers.len();
    let mut rows = Vec::new();
    let mut row_index = 0usize;

    for (line_no, record) in rdr.records().enumerate() {
        let record = record.map_err(|e| ParseError::Read {
            line: line_no + 2, // +2: 1-based, +1 for header
            message: e.to_string(),
        })?;

        let mut fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // Pad or truncate to match header length
        if fields.len() < header_len {
            fields.resize(header_len, String::new());
        } else if fields.len() > header_len {
            fields.truncate(header_len);
        }

        rows.push(RawRow { fields, row_index });
        row_index += 1;
    }

    if rows.is_empty() {
        return Err(ParseError::HeaderOnly);
    }

    Ok(ParsedCsv { headers, rows })
}
