//! Utility functions and error types for the gitronics project.
//!
//! This module provides common utilities including:
//! - Custom error types for the application
//! - File path discovery and validation
//! - Logger initialization

use crate::types::{EnvelopeName, FileName, FillerName};
use log::LevelFilter;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

const VALID_SUFFIXES: &[&str] = &["yaml", "yml", "mcnp", "mat", "tally", "transform", "source"];

#[derive(Debug, Error)]
pub enum GitronicsError {
    #[error("I/O error at `{path}`: {source}")]
    IoPath {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("I/O error occurred: `{0}`")]
    Io(#[from] io::Error),

    #[error("YAML parsing error of file `{0}`.\n{1}")]
    YamlParse(String, String),

    #[error("YAML serialization error of file `{0}`.\n{1}")]
    YamlSerialize(String, String),

    #[error("Failed to load MCNP file: `{file_name}`.\n{error}")]
    FailedToLoadMCNPFile { file_name: FileName, error: String },

    #[error("Failed to load data cards file: `{file_name}`.\n{error}")]
    FailedToLoadDataCardsFile { file_name: FileName, error: String },

    #[error("Duplicate file name `{0}` found")]
    DuplicateFileName(FileName),

    #[error("File `{file_name}` not found in project with root `{project_root}`")]
    FileNotFound {
        file_name: FileName,
        project_root: String,
    },

    #[error("Cycle detected in configuration overrides: `{0}`")]
    ConfigCycle(PathBuf),

    #[error("No `envelope_structure` key found in model configuration")]
    MissingEnvelopeStructureInConfig,

    #[error("No cell ID found in first cell of filler model `{0}`")]
    NoCellID(FillerName),

    #[error("No universe ID found in first cell of filler model `{0}`")]
    FirstCellWithoutUniverseID(FillerName),

    #[error(
        "Metadata not found for filler `{0}`. A file named `{0}.metadata` should be present in the same directory as the filler."
    )]
    MetadataNotFound(FileName),

    #[error("Failed to extract envelope name from placeholder: `{0}` in card `{1}`")]
    FailedToExtractEnvelopeName(String, String),

    #[error(
        "Transformation not specified in the metadata of filler `{filler_name}` for envelope `{envelope_name}`."
    )]
    TransformationNotFound {
        filler_name: FillerName,
        envelope_name: EnvelopeName,
    },

    #[error("Universe ID not found for filler `{0}` — filler was not loaded from configuration")]
    FillerUniverseIdMissing(FillerName),

    #[error("Failed to construct FILL card parameter from `{0}`: `{1}`")]
    InvalidFillCard(String, String),

    #[error("{0}")]
    ValidationError(String),
}

impl GitronicsError {
    pub fn io_path(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::IoPath {
            path: path.as_ref().display().to_string(),
            source,
        }
    }
}

/// Recursively discovers and indexes all project files by their stem names.
///
/// Walks through the given directory tree and collects files with valid suffixes
/// (yaml, yml, mcnp, mat, tally, transform, source). The files are indexed by their
/// stem name (filename without extension) in a HashMap.
pub fn get_file_paths<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<FileName, PathBuf>, GitronicsError> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?.to_str()?;

            if !VALID_SUFFIXES.contains(&ext) {
                return None;
            }

            let filename = path.file_stem()?.to_string_lossy().to_string();
            Some((FileName::new(filename), path.to_path_buf()))
        })
        .try_fold(HashMap::new(), |mut acc, (filename, path)| {
            match acc.entry(filename) {
                Entry::Occupied(entry) => {
                    Err(GitronicsError::DuplicateFileName(entry.key().clone()))
                }
                Entry::Vacant(entry) => {
                    entry.insert(path);
                    Ok(acc)
                }
            }
        })
}

/// Initializes the application logger with custom formatting.
///
/// Sets up env_logger with INFO level by default (can be overridden by RUST_LOG).
/// Log format: `[YYYY-MM-DD HH:MM:SS LEVEL target] message`
pub fn init_logger() {
    let mut logger = env_logger::Builder::new();

    // Default to INFO from code, but still allow RUST_LOG to override it.
    logger.filter_level(LevelFilter::Info);
    logger.parse_default_env();

    // Custom format: [2026-05-20 09:27:04 INFO gitronics_rs::build_model] message
    logger.format(|buf, record| {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(
            buf,
            "[{} {} gitronics] {}",
            ts,
            record.level(),
            record.args()
        )
    });
    logger.init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_get_file_paths() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("file1.yaml");
        let file2_path = dir.path().join("file2.yml");
        let subdir_path = dir.path().join("subdir");
        fs::create_dir(&subdir_path).unwrap();
        // We dont read the metadata here
        let file3_path = subdir_path.join("file3.metadata");
        let file4_path = dir.path().join("file4.txt"); // Invalid suffix
        let file5_path = subdir_path.join("file5.mcnp");
        File::create(&file1_path).unwrap();
        File::create(&file2_path).unwrap();
        File::create(&file3_path).unwrap();
        File::create(&file4_path).unwrap();
        File::create(&file5_path).unwrap();
        let file_paths = get_file_paths(dir.path()).unwrap();
        assert_eq!(file_paths.len(), 3);
        assert_eq!(
            file_paths.get(&FileName::new("file1")).unwrap(),
            &file1_path
        );
        assert_eq!(
            file_paths.get(&FileName::new("file2")).unwrap(),
            &file2_path
        );
        assert!(!file_paths.contains_key(&FileName::new("file3")));
        assert!(!file_paths.contains_key(&FileName::new("file4")));
        assert_eq!(
            file_paths.get(&FileName::new("file5")).unwrap(),
            &file5_path
        );
    }

    #[test]
    fn test_duplicate_stem_error() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("file1.yaml");
        let file2_path = dir.path().join("file1.yml");
        File::create(&file1_path).unwrap();
        File::create(&file2_path).unwrap();
        let err = get_file_paths(dir.path()).unwrap_err();
        assert!(err.to_string().contains("Duplicate file name `file1`"));
    }
}
