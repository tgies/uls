//! ZIP archive handling with streaming support.
//!
//! This module provides functionality to stream DAT files directly from
//! ZIP archives without extracting to disk, minimizing I/O overhead.

use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use thiserror::Error;
use zip::ZipArchive;

use crate::dat::{DatReader, ParsedLine};

/// ZIP-specific error types.
#[derive(Error, Debug)]
pub enum ZipError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("DAT file not found in archive: {0}")]
    DatFileNotFound(String),
}

/// A ULS ZIP archive that can stream DAT files.
pub struct ZipExtractor<R: Read + Seek> {
    archive: ZipArchive<R>,
}

impl ZipExtractor<BufReader<File>> {
    /// Open a ULS ZIP file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ZipError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let archive = ZipArchive::new(reader)?;
        Ok(Self { archive })
    }

    /// Get statistics about the archive.
    pub fn stats(&mut self) -> Result<ArchiveStats, ZipError> {
        let dat_files = self.list_dat_files();
        let total_files = self.archive.len();
        let mut total_size = 0u64;

        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i)?;
            total_size += file.size();
        }

        Ok(ArchiveStats {
            total_files,
            dat_files,
            total_size_bytes: total_size,
        })
    }

    /// Count records in all DAT files in the archive.
    pub fn count_all_records(
        &mut self,
    ) -> Result<std::collections::HashMap<String, usize>, crate::ParseError> {
        let dat_files = self.list_dat_files();
        let mut counts = std::collections::HashMap::new();

        for dat_file in dat_files {
            let count = self.process_dat_streaming(&dat_file, |_| true)?;
            counts.insert(dat_file, count);
        }

        Ok(counts)
    }
}

impl<R: Read + Seek> ZipExtractor<R> {
    /// Create from an existing archive reader.
    pub fn new(archive: ZipArchive<R>) -> Self {
        Self { archive }
    }

    /// List all DAT files in the archive.
    pub fn list_dat_files(&mut self) -> Vec<String> {
        let mut files = Vec::new();
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                let name = file.name().to_string();
                if name.to_uppercase().ends_with(".DAT") {
                    files.push(name);
                }
            }
        }
        files
    }

    /// List all files in the archive.
    pub fn list_files(&mut self) -> Vec<String> {
        let mut files = Vec::new();
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                files.push(file.name().to_string());
            }
        }
        files
    }

    /// Get the size of a file in the archive.
    pub fn file_size(&mut self, name: &str) -> Result<u64, ZipError> {
        let file = self.archive.by_name(name)?;
        Ok(file.size())
    }

    /// Find the index of a file by name (case-insensitive).
    fn find_file_index(&mut self, name: &str) -> Option<usize> {
        // Try exact name first
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                if file.name() == name {
                    return Some(i);
                }
            }
        }

        // Try case-insensitive match
        let name_upper = name.to_uppercase();
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                if file.name().to_uppercase() == name_upper {
                    return Some(i);
                }
            }
        }

        None
    }

    /// Stream a DAT file from the archive without extracting to disk.
    /// Returns a reader that can be used with DatReader.
    pub fn stream_dat(&mut self, name: &str) -> Result<impl Read + '_, ZipError> {
        // Find the index first to avoid borrow issues
        let index = self
            .find_file_index(name)
            .ok_or_else(|| ZipError::DatFileNotFound(name.to_string()))?;

        self.archive.by_index(index).map_err(ZipError::Zip)
    }

    /// Process a DAT file streaming from the archive, calling a callback for each record.
    /// This is the most memory-efficient way to process large ULS archives.
    pub fn process_dat_streaming<F>(
        &mut self,
        dat_name: &str,
        mut callback: F,
    ) -> Result<usize, crate::ParseError>
    where
        F: FnMut(ParsedLine) -> bool,
    {
        let reader = self.stream_dat(dat_name)?;
        let mut dat_reader = DatReader::new(reader);
        let mut count = 0;

        while let Some(line) = dat_reader.next_line()? {
            count += 1;
            if !callback(line) {
                break;
            }
        }

        Ok(count)
    }
}

impl From<ZipError> for crate::ParseError {
    fn from(err: ZipError) -> Self {
        match err {
            ZipError::Io(e) => crate::ParseError::Io(e),
            ZipError::Zip(e) => crate::ParseError::Zip(e),
            ZipError::DatFileNotFound(name) => crate::ParseError::InvalidFormat {
                line: 0,
                message: format!("DAT file not found: {}", name),
            },
        }
    }
}

/// Statistics about a ULS archive.
#[derive(Debug, Clone)]
pub struct ArchiveStats {
    pub total_files: usize,
    pub dat_files: Vec<String>,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn create_test_zip() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = zip::ZipWriter::new(cursor);

            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            writer.start_file("HD.dat", options).unwrap();
            writer.write_all(b"HD|1|||TEST|A|HA|\n").unwrap();
            writer.write_all(b"HD|2|||TEST2|A|HA|\n").unwrap();

            writer.start_file("EN.dat", options).unwrap();
            writer.write_all(b"EN|1|||TEST|L||John||\n").unwrap();

            writer.finish().unwrap();
        }
        buf
    }

    fn create_zip_with_mixed_case() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut writer = zip::ZipWriter::new(cursor);

            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            // Mixed case filenames
            writer.start_file("hd.DAT", options).unwrap();
            writer.write_all(b"HD|1|||LOWERCASE|A|HA|\n").unwrap();

            writer.start_file("en.Dat", options).unwrap();
            writer.write_all(b"EN|1|||MIXEDCASE|L||Test||\n").unwrap();

            writer.start_file("readme.txt", options).unwrap();
            writer.write_all(b"Not a DAT file\n").unwrap();

            writer.finish().unwrap();
        }
        buf
    }

    #[test]
    fn test_list_dat_files() {
        let data = create_test_zip();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        let files = extractor.list_dat_files();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"HD.dat".to_string()));
        assert!(files.contains(&"EN.dat".to_string()));
    }

    #[test]
    fn test_stream_dat() {
        let data = create_test_zip();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        let count = extractor
            .process_dat_streaming("HD.dat", |line| {
                assert_eq!(line.record_type, "HD");
                true
            })
            .unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_list_all_files() {
        let data = create_zip_with_mixed_case();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        let files = extractor.list_files();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"hd.DAT".to_string()));
        assert!(files.contains(&"en.Dat".to_string()));
        assert!(files.contains(&"readme.txt".to_string()));
    }

    #[test]
    fn test_list_dat_files_mixed_case() {
        let data = create_zip_with_mixed_case();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        // Should find DAT files regardless of case
        let dat_files = extractor.list_dat_files();
        assert_eq!(dat_files.len(), 2);
    }

    #[test]
    fn test_file_size() {
        let data = create_test_zip();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        let size = extractor.file_size("HD.dat").unwrap();
        assert!(size > 0);
    }

    #[test]
    fn test_file_size_not_found() {
        let data = create_test_zip();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        let result = extractor.file_size("nonexistent.dat");
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_dat_case_insensitive() {
        let data = create_zip_with_mixed_case();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        // Should find "hd.DAT" when looking for "HD.dat"
        let count = extractor
            .process_dat_streaming("HD.dat", |line| {
                assert_eq!(line.record_type, "HD");
                true
            })
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_stream_dat_not_found() {
        let data = create_test_zip();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        let result = extractor.stream_dat("NONEXISTENT.dat");
        assert!(result.is_err());

        match result {
            Err(ZipError::DatFileNotFound(name)) => {
                assert_eq!(name, "NONEXISTENT.dat");
            }
            _ => panic!("Expected DatFileNotFound error"),
        }
    }

    #[test]
    fn test_process_dat_early_termination() {
        let data = create_test_zip();
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor).unwrap();
        let mut extractor = ZipExtractor::new(archive);

        let mut processed = 0;
        let count = extractor
            .process_dat_streaming("HD.dat", |_line| {
                processed += 1;
                false // Stop after first record
            })
            .unwrap();

        // Should have processed exactly 1 record before stopping
        assert_eq!(count, 1);
        assert_eq!(processed, 1);
    }

    #[test]
    fn test_zip_error_to_parse_error() {
        let err = ZipError::DatFileNotFound("test.dat".to_string());
        let parse_err: crate::ParseError = err.into();
        let msg = parse_err.to_string();
        assert!(msg.contains("test.dat"));
    }

    #[test]
    fn test_zip_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let zip_err = ZipError::from(io_err);
        let parse_err: crate::ParseError = zip_err.into();
        assert!(matches!(parse_err, crate::ParseError::Io(_)));
    }
}
