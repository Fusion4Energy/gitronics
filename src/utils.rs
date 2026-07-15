//! Utility functions and error types for the gitronics project.
//!
//! This module provides common utilities including:
//! - Custom error types for the application
//! - File path discovery and validation
//! - Logger initialization

use crate::types::{EnvelopeName, FileName, FillerName};
use log::LevelFilter;
use migjorn::{Model, Severity};
use path_clean::PathClean;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs;
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

    #[error(
        "ID collisions between components (each id must be unique across the envelope structure and all fillers):\n{0}"
    )]
    MergeConflicts(String),

    #[error("{0}")]
    ValidationError(String),
}

impl GitronicsError {
    pub fn io_path(path: impl AsRef<Path>, source: io::Error) -> Self {
        let path_ref = path.as_ref();

        let cleaned_path = dunce::canonicalize(path_ref)
            .unwrap_or_else(|_| path_ref.to_path_buf().clean())
            .display()
            .to_string();

        Self::IoPath {
            path: cleaned_path,
            source,
        }
    }
}

/// Reads an MCNP model file and parses it with migjorn, returning an error if
/// the parser reports any error-severity diagnostic (warnings are tolerated).
pub fn parse_model_file(path: &Path, file_name: &FileName) -> Result<Model, GitronicsError> {
    let text = fs::read_to_string(path).map_err(|source| GitronicsError::io_path(path, source))?;
    let model = Model::parse(text);
    if let Some(diag) = model
        .diagnostics()
        .iter()
        .find(|d| matches!(d.severity, Severity::Error))
    {
        return Err(GitronicsError::FailedToLoadMCNPFile {
            file_name: file_name.clone(),
            error: diag.message.clone(),
        });
    }
    Ok(model)
}

/// Reads a Gitronics data-card file and returns its data-card text.
///
/// Following the data-card file convention, the first line is treated as a
/// title and dropped unless it is an MCNP comment (in which case it is kept as a
/// header), and content stops at the first blank line — anything after it is
/// ignored.
pub fn read_data_cards_text(path: &Path, file_name: &FileName) -> Result<String, GitronicsError> {
    let content =
        fs::read_to_string(path).map_err(|source| GitronicsError::io_path(path, source))?;
    let mut kept: Vec<&str> = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            break; // MCNP section separator: stop at the first blank line
        }
        if i == 0 && !is_mcnp_comment(line) {
            continue; // drop a non-comment title line
        }
        kept.push(line);
    }
    if kept.is_empty() {
        return Err(GitronicsError::FailedToLoadDataCardsFile {
            file_name: file_name.clone(),
            error: "no data cards found before the first blank line".to_string(),
        });
    }
    Ok(kept.join("\n"))
}

/// True if `line` is an MCNP full-line comment (first non-blank character is
/// `c`/`C`, followed by whitespace or end of line) or blank.
fn is_mcnp_comment(line: &str) -> bool {
    let t = line.trim_start();
    let mut chars = t.chars();
    match chars.next() {
        None => true,
        Some('c') | Some('C') => chars.next().is_none_or(|c| c.is_whitespace()),
        _ => false,
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
    logger.try_init().ok();
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

    #[test]
    fn test_io_path_normalizes_parent_segments_when_not_canonicalizable() {
        let err = GitronicsError::io_path(
            Path::new("non_existing/../clean/path.yaml"),
            io::Error::new(io::ErrorKind::NotFound, "missing"),
        );

        match err {
            GitronicsError::IoPath { path, .. } => {
                assert_eq!(Path::new(&path), Path::new("clean/path.yaml"));
            }
            _ => panic!("Expected IoPath error"),
        }
    }
}
