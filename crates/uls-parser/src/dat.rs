//! DAT file parser for pipe-delimited ULS records.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use phf::phf_set;
use uls_core::codes::RecordType;
use uls_core::records::*;

use crate::{ParseError, Result};

/// Known valid record type codes (2 uppercase letters).
/// Uses compile-time perfect hash for O(1) lookup.
static VALID_RECORD_TYPES: phf::Set<&'static str> = phf_set! {
    "A2", "A3", "AC", "AD", "AM", "AN", "AS", "AT", "BC", "BD", "BF", "BL", "BO", "CF", "CG", "CH",
    "CL", "CO", "CP", "CS", "CW", "EM", "EN", "F2", "FA", "FC", "FF", "FH", "FR", "FT", "HD", "HS",
    "IR", "L2", "L3", "L4", "LA", "LF", "LH", "LM", "LO", "LS", "MC", "MF", "MH", "MI", "MK", "ML",
    "MP", "MW", "O2", "OP", "PA", "PC", "PF", "PH", "PI", "PL", "RA", "RC", "RE", "RG", "RI", "RS",
    "SA", "SC", "SE", "SF", "SG", "SH", "SL", "SR", "SS", "SV", "TA", "TP", "UA", "UC", "UF", "UL",
    "UM", "VC",
};

/// Check if a string is a valid record type prefix.
#[inline]
fn is_valid_record_type(s: &str) -> bool {
    s.len() == 2 && VALID_RECORD_TYPES.contains(s)
}

/// A parsed line from a DAT file.
#[derive(Debug)]
pub struct ParsedLine {
    /// The line number (1-indexed).
    pub line_number: usize,
    /// The record type code.
    pub record_type: String,
    /// The raw fields (pipe-separated).
    pub fields: Vec<String>,
}

impl ParsedLine {
    /// Parse a line into fields.
    pub fn from_line(line: &str, line_number: usize) -> Result<Self> {
        let fields: Vec<String> = line.split('|').map(|s| s.to_string()).collect();

        if fields.is_empty() {
            return Err(ParseError::InvalidFormat {
                line: line_number,
                message: "empty line".to_string(),
            });
        }

        let record_type = fields[0].clone();

        Ok(Self {
            line_number,
            record_type,
            fields,
        })
    }

    /// Get a field as a string slice, or empty string if out of bounds.
    pub fn field(&self, index: usize) -> &str {
        self.fields.get(index).map(|s| s.as_str()).unwrap_or("")
    }

    /// Get field references suitable for from_fields methods.
    pub fn field_refs(&self) -> Vec<&str> {
        self.fields.iter().map(|s| s.as_str()).collect()
    }

    /// Append a continuation line to this record.
    /// The continuation text is appended to the last non-empty field.
    pub fn append_continuation(&mut self, line: &str) {
        // Find the last field that looks like it could have content (description field)
        // For most records with continuation, this is field 5 (description) or similar
        // We append to the last field before the trailing empty fields

        // Trim trailing empty fields to find the real last field
        let mut last_content_idx = self.fields.len().saturating_sub(1);
        while last_content_idx > 0 && self.fields[last_content_idx].is_empty() {
            last_content_idx -= 1;
        }

        // Append the continuation (with a space separator if there's existing content)
        if !self.fields[last_content_idx].is_empty() {
            self.fields[last_content_idx].push(' ');
        }
        // Strip pipe delimiters from continuation line and append
        let continuation = line.trim_matches('|').trim();
        self.fields[last_content_idx].push_str(continuation);
    }

    /// Convert to a typed ULS record.
    /// Uses stack-allocated array to avoid heap allocation for field references.
    pub fn to_record(&self) -> Result<UlsRecord> {
        // Build field references slice - reuse existing from_fields methods
        // Use a stack-allocated array for small records, heap for large
        const STACK_LIMIT: usize = 64;

        if self.fields.len() <= STACK_LIMIT {
            // Stack-allocated path for most records
            let mut refs_arr: [&str; STACK_LIMIT] = [""; STACK_LIMIT];
            for (i, s) in self.fields.iter().take(STACK_LIMIT).enumerate() {
                refs_arr[i] = s.as_str();
            }
            let refs = &refs_arr[..self.fields.len()];
            self.to_record_from_refs(refs)
        } else {
            // Heap fallback for unusually large records
            let refs = self.field_refs();
            self.to_record_from_refs(&refs)
        }
    }

    /// Internal helper to convert to record from field references.
    fn to_record_from_refs(&self, refs: &[&str]) -> Result<UlsRecord> {
        match self.record_type.as_str() {
            "HD" => Ok(UlsRecord::Header(HeaderRecord::from_fields(refs))),
            "EN" => Ok(UlsRecord::Entity(EntityRecord::from_fields(refs))),
            "AM" => Ok(UlsRecord::Amateur(AmateurRecord::from_fields(refs))),
            "AD" => Ok(UlsRecord::ApplicationDetail(
                ApplicationDetailRecord::from_fields(refs),
            )),
            "HS" => Ok(UlsRecord::History(HistoryRecord::from_fields(refs))),
            "CO" => Ok(UlsRecord::Comment(CommentRecord::from_fields(refs))),
            "LO" => Ok(UlsRecord::Location(LocationRecord::from_fields(refs))),
            "FR" => Ok(UlsRecord::Frequency(FrequencyRecord::from_fields(refs))),
            "AN" => Ok(UlsRecord::Antenna(AntennaRecord::from_fields(refs))),
            "EM" => Ok(UlsRecord::Emission(EmissionRecord::from_fields(refs))),
            "SC" => Ok(UlsRecord::SpecialCondition(
                SpecialConditionRecord::from_fields(refs),
            )),
            "SF" => Ok(UlsRecord::FreeformCondition(
                FreeformConditionRecord::from_fields(refs),
            )),
            "VC" => Ok(UlsRecord::VanityCallSign(
                VanityCallSignRecord::from_fields(refs),
            )),
            "AC" => Ok(UlsRecord::Aircraft(AircraftRecord::from_fields(refs))),
            "SH" => Ok(UlsRecord::Ship(ShipRecord::from_fields(refs))),
            // For record types not yet fully implemented, return raw
            _ => {
                if let Ok(rt) = self.record_type.parse::<RecordType>() {
                    Ok(UlsRecord::Raw {
                        record_type: rt,
                        fields: self.fields.clone(),
                    })
                } else {
                    Err(ParseError::UnknownRecordType(self.record_type.clone()))
                }
            }
        }
    }
}

/// Parse a raw line string for fields, without requiring it to be a valid record.
fn parse_raw_fields(line: &str) -> Vec<String> {
    line.split('|').map(|s| s.to_string()).collect()
}

/// Check if a line is a continuation (doesn't start with a valid record type).
fn is_continuation_line(line: &str) -> bool {
    if line.is_empty() {
        return true;
    }

    let fields = parse_raw_fields(line);
    if fields.is_empty() {
        return true;
    }

    let first_field = &fields[0];
    !is_valid_record_type(first_field)
}

/// Reader for DAT files that yields parsed lines.
/// Automatically handles multi-line continuation records.
pub struct DatReader<R: Read> {
    reader: BufReader<R>,
    line_number: usize,
    buffer: String,
    /// Buffered/pending record that may receive continuation lines
    pending_record: Option<ParsedLine>,
}

impl<R: Read> DatReader<R> {
    /// Create a new DAT reader from any Read source.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_number: 0,
            buffer: String::new(),
            pending_record: None,
        }
    }

    /// Read a raw line from the file.
    fn read_raw_line(&mut self) -> Result<Option<String>> {
        self.buffer.clear();
        let bytes_read = self.reader.read_line(&mut self.buffer)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        self.line_number += 1;

        // Trim trailing newlines/carriage returns
        let line = self.buffer.trim_end_matches(&['\r', '\n'][..]).to_string();
        Ok(Some(line))
    }

    /// Read the next complete record from the file.
    /// Handles multi-line continuation by merging lines until a new record starts.
    pub fn next_line(&mut self) -> Result<Option<ParsedLine>> {
        loop {
            match self.read_raw_line()? {
                None => {
                    // EOF - return any pending record
                    return Ok(self.pending_record.take());
                }
                Some(line) => {
                    if line.is_empty() {
                        // Skip truly empty lines
                        continue;
                    }

                    if is_continuation_line(&line) {
                        // This is a continuation - append to pending record if we have one
                        if let Some(ref mut pending) = self.pending_record {
                            pending.append_continuation(&line);
                        }
                        // If no pending record, we just skip orphan continuation lines
                        continue;
                    }

                    // This is a new record
                    let new_record = ParsedLine::from_line(&line, self.line_number)?;

                    // Return the previous pending record (if any) and buffer this new one
                    let to_return = self.pending_record.replace(new_record);

                    if to_return.is_some() {
                        return Ok(to_return);
                    }
                    // If there was no pending record, loop to read more
                }
            }
        }
    }

    /// Returns the current line number.
    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

impl DatReader<File> {
    /// Open a DAT file for reading.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self::new(file))
    }
}

impl<R: Read> Iterator for DatReader<R> {
    type Item = Result<ParsedLine>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_line() {
            Ok(Some(line)) => Some(Ok(line)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Parse a single line (without continuation handling).
/// Use DatReader for proper multi-line handling.
pub fn parse_line(line: &str, line_number: usize) -> Result<ParsedLine> {
    ParsedLine::from_line(line, line_number)
}

/// Convenience function to parse a complete DAT file into records.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<UlsRecord>> {
    let reader = DatReader::open(path)?;
    let mut records = Vec::new();

    for line_result in reader {
        let line = line_result?;
        let record = line.to_record()?;
        records.push(record);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_line() {
        let line = "HD|123|456|W1AW|A|HA";
        let parsed = ParsedLine::from_line(line, 1).unwrap();
        assert_eq!(parsed.record_type, "HD");
        assert_eq!(parsed.fields.len(), 6);
        assert_eq!(parsed.field(3), "W1AW");
    }

    #[test]
    fn test_is_continuation_line() {
        assert!(!is_continuation_line("CO|123|test"));
        assert!(!is_continuation_line("HD|456|data"));
        assert!(is_continuation_line("License cancelled"));
        assert!(is_continuation_line("||"));
        assert!(is_continuation_line(""));
        assert!(is_continuation_line("Some text without record type"));
    }

    #[test]
    fn test_continuation_handling() {
        let data = "CO|123||W1AW|01/01/2024|First line of comment||\n\
                    continued text here||\n\
                    HD|456||W1AW|A|HA||\n";

        let reader = DatReader::new(data.as_bytes());
        let lines: Vec<_> = reader.collect();

        assert_eq!(lines.len(), 2);

        // First record should have continuation merged
        let co_record = lines[0].as_ref().unwrap();
        assert_eq!(co_record.record_type, "CO");
        assert!(co_record.field(5).contains("continued text here"));

        // Second record should be HD
        let hd_record = lines[1].as_ref().unwrap();
        assert_eq!(hd_record.record_type, "HD");
    }

    #[test]
    fn test_is_valid_record_type() {
        assert!(is_valid_record_type("HD"));
        assert!(is_valid_record_type("CO"));
        assert!(is_valid_record_type("EN"));
        assert!(!is_valid_record_type("XX"));
        assert!(!is_valid_record_type("License"));
        assert!(!is_valid_record_type(""));
    }

    #[test]
    fn test_from_line_empty_string_is_single_empty_field() {
        // splitting "" on '|' yields one empty field, never an empty Vec,
        // so from_line succeeds with an empty record type.
        let parsed = ParsedLine::from_line("", 7).unwrap();
        assert_eq!(parsed.line_number, 7);
        assert_eq!(parsed.record_type, "");
        assert_eq!(parsed.fields, vec![String::new()]);
    }

    #[test]
    fn test_from_line_trailing_pipe_yields_trailing_empty_field() {
        let parsed = ParsedLine::from_line("HD|123|W1AW|", 1).unwrap();
        assert_eq!(
            parsed.fields,
            vec![
                "HD".to_string(),
                "123".to_string(),
                "W1AW".to_string(),
                String::new(),
            ]
        );
        assert_eq!(parsed.field(3), "");
    }

    #[test]
    fn test_from_line_embedded_empty_fields() {
        let parsed = ParsedLine::from_line("EN|1||W1AW||John", 1).unwrap();
        assert_eq!(parsed.fields.len(), 6);
        assert_eq!(parsed.field(0), "EN");
        assert_eq!(parsed.field(1), "1");
        assert_eq!(parsed.field(2), "");
        assert_eq!(parsed.field(3), "W1AW");
        assert_eq!(parsed.field(4), "");
        assert_eq!(parsed.field(5), "John");
    }

    #[test]
    fn test_field_out_of_bounds_returns_empty() {
        let parsed = ParsedLine::from_line("AM|1|W1AW", 1).unwrap();
        assert_eq!(parsed.field(2), "W1AW");
        assert_eq!(parsed.field(3), "");
        assert_eq!(parsed.field(999), "");
    }

    #[test]
    fn test_field_refs_mirrors_fields() {
        let parsed = ParsedLine::from_line("HD|1||W1AW", 1).unwrap();
        assert_eq!(parsed.field_refs(), vec!["HD", "1", "", "W1AW"]);
    }

    #[test]
    fn test_parse_line_matches_from_line() {
        let parsed = parse_line("CO|42|note", 99).unwrap();
        assert_eq!(parsed.line_number, 99);
        assert_eq!(parsed.record_type, "CO");
        assert_eq!(parsed.field(2), "note");
    }

    #[test]
    fn test_to_record_known_type_header() {
        let parsed = ParsedLine::from_line("HD|123456789|||W1AW|A|HA", 1).unwrap();
        let record = parsed.to_record().unwrap();
        match record {
            UlsRecord::Header(h) => {
                assert_eq!(h.unique_system_identifier, 123456789);
                assert_eq!(h.call_sign, Some("W1AW".to_string()));
            }
            other => panic!("expected Header, got {other:?}"),
        }
    }

    #[test]
    fn test_to_record_amateur_type() {
        let parsed = ParsedLine::from_line("AM|254305|||KK9HIU|E|A|8", 1).unwrap();
        let record = parsed.to_record().unwrap();
        match record {
            UlsRecord::Amateur(a) => {
                assert_eq!(a.unique_system_identifier, 254305);
                assert_eq!(a.callsign, Some("KK9HIU".to_string()));
                assert_eq!(a.operator_class, Some('E'));
                assert_eq!(a.region_code, Some(8));
            }
            other => panic!("expected Amateur, got {other:?}"),
        }
    }

    #[test]
    fn test_to_record_valid_recordtype_but_unmapped_is_raw() {
        // LA parses as a RecordType but has no explicit arm in to_record_from_refs,
        // so it falls through to the Raw branch carrying every field.
        let parsed = ParsedLine::from_line("LA|780866|W1AW|L|Pleading", 1).unwrap();
        let record = parsed.to_record().unwrap();
        match record {
            UlsRecord::Raw {
                record_type,
                fields,
            } => {
                assert_eq!(record_type, RecordType::LA);
                assert_eq!(fields, vec!["LA", "780866", "W1AW", "L", "Pleading"]);
            }
            other => panic!("expected Raw, got {other:?}"),
        }
    }

    #[test]
    fn test_to_record_unknown_type_errors() {
        // ZZ is two uppercase letters but is not a known RecordType,
        // so to_record returns UnknownRecordType.
        let parsed = ParsedLine::from_line("ZZ|1|2", 1).unwrap();
        let err = parsed.to_record().unwrap_err();
        match err {
            ParseError::UnknownRecordType(rt) => assert_eq!(rt, "ZZ"),
            other => panic!("expected UnknownRecordType, got {other:?}"),
        }
    }

    #[test]
    fn test_to_record_large_record_heap_path() {
        // More than STACK_LIMIT (64) fields forces the heap fallback in to_record.
        let mut parts = vec!["HD".to_string(), "999".to_string()];
        parts.resize(80, "x".to_string());
        let line = parts.join("|");
        let parsed = ParsedLine::from_line(&line, 1).unwrap();
        assert_eq!(parsed.fields.len(), 80);
        let record = parsed.to_record().unwrap();
        match record {
            UlsRecord::Header(h) => assert_eq!(h.unique_system_identifier, 999),
            other => panic!("expected Header, got {other:?}"),
        }
    }

    #[test]
    fn test_append_continuation_to_last_nonempty_field() {
        let mut parsed = ParsedLine::from_line("CO|1||W1AW|First line||", 1).unwrap();
        parsed.append_continuation("second line");
        assert_eq!(parsed.field(4), "First line second line");
    }

    #[test]
    fn test_append_continuation_strips_pipes_and_trims() {
        let mut parsed = ParsedLine::from_line("CO|1||W1AW|First", 1).unwrap();
        parsed.append_continuation("|  more text  |");
        assert_eq!(parsed.field(4), "First more text");
    }

    #[test]
    fn test_append_continuation_no_separator_when_target_empty() {
        // Trailing empty fields are skipped; with no non-empty content the
        // continuation lands on field 0 without a leading separator space.
        let mut parsed = ParsedLine::from_line("||", 1).unwrap();
        parsed.append_continuation("orphan");
        assert_eq!(parsed.field(0), "orphan");
    }

    #[test]
    fn test_reader_skips_blank_lines_between_records() {
        let data = "HD|1||W1AW|A|HA\n\nHD|2||W2AW|A|HA\n";
        let reader = DatReader::new(data.as_bytes());
        let records: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].field(3), "W1AW");
        assert_eq!(records[1].field(3), "W2AW");
    }

    #[test]
    fn test_reader_handles_crlf_line_endings() {
        let data = "HD|1||W1AW|A|HA\r\nHD|2||W2AW|A|HA\r\n";
        let reader = DatReader::new(data.as_bytes());
        let records: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 2);
        // The trailing carriage return is stripped from the final field.
        assert_eq!(records[0].record_type, "HD");
        assert_eq!(records[1].field(3), "W2AW");
        assert_eq!(records[1].field(5), "HA");
    }

    #[test]
    fn test_reader_orphan_continuation_at_start_is_skipped() {
        // A continuation line with no preceding record is dropped.
        let data = "orphan continuation\nHD|1||W1AW|A|HA\n";
        let reader = DatReader::new(data.as_bytes());
        let records: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].record_type, "HD");
    }

    #[test]
    fn test_reader_returns_final_pending_record_at_eof() {
        // A single record with no trailing newline is still yielded.
        let data = "HD|1||W1AW|A|HA";
        let reader = DatReader::new(data.as_bytes());
        let records: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].field(3), "W1AW");
    }

    #[test]
    fn test_reader_line_number_tracks_raw_lines() {
        let data = "HD|1||W1AW|desc\ncontinuation\nHD|2||W2AW|desc2\n";
        let mut reader = DatReader::new(data.as_bytes());
        assert_eq!(reader.line_number(), 0);
        let first = reader.next_line().unwrap().unwrap();
        // First record is emitted only once the second HD is read on line 3,
        // with the line-2 continuation merged into its last description field.
        assert_eq!(first.field(3), "W1AW");
        assert_eq!(first.field(4), "desc continuation");
        assert_eq!(reader.line_number(), 3);
        let second = reader.next_line().unwrap().unwrap();
        assert_eq!(second.field(3), "W2AW");
        assert!(reader.next_line().unwrap().is_none());
    }

    #[test]
    fn test_open_nonexistent_file_errors() {
        match DatReader::<File>::open("/no/such/path/missing.dat") {
            Err(ParseError::Io(_)) => {}
            Err(other) => panic!("expected Io error, got {other:?}"),
            Ok(_) => panic!("expected error opening missing path"),
        }
    }

    #[test]
    fn test_parse_file_round_trip_from_temp() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HD.dat");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"HD|111||W1AW|A|HA\nHD|222||W2AW|A|HA\n")
            .unwrap();
        drop(f);

        let records = parse_file(&path).unwrap();
        assert_eq!(records.len(), 2);
        match &records[0] {
            UlsRecord::Header(h) => assert_eq!(h.unique_system_identifier, 111),
            other => panic!("expected Header, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_file_propagates_unknown_record_error() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.dat");
        let mut f = File::create(&path).unwrap();
        // CH is in VALID_RECORD_TYPES, so the reader treats it as a record start
        // rather than a continuation, yet it fails RecordType parsing, so
        // parse_file surfaces UnknownRecordType.
        f.write_all(b"CH|1|2\n").unwrap();
        drop(f);

        let err = parse_file(&path).unwrap_err();
        assert!(matches!(err, ParseError::UnknownRecordType(rt) if rt == "CH"));
    }
}
