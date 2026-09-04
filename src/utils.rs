//! Utility functions and error types for the gitronics project.
//!
//! This module provides common utilities including:
//! - Custom error types for the application
//! - File path discovery and validation
//! - Logger initialization

use crate::types::{EnvelopeName, FileName, FillerName};
use log::{LevelFilter, info};
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
    let model = Model::parse(&text);
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

/// Sort key mirroring a data card's id: cards whose mnemonic ends in a number
/// (`M1`, `F4`, `TR1`) order by that number and come first; mnemonics without a
/// trailing number (`SDEF`, `MODE`) fall back to alphabetical order and come
/// last — matching the old `DataCardId::{Int, String}` ordering.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DataCardKey {
    Int(u32),
    Str(String),
}

/// The sort key of a data-card file block, derived from the id of its first
/// data card. Full-line comments (and blank lines) are skipped so a comment
/// header does not decide the order.
fn first_data_card_key(chunk: &str) -> DataCardKey {
    for line in chunk.lines() {
        if is_mcnp_comment(line) {
            continue;
        }
        // First whitespace token is the mnemonic, minus any leading `*`.
        let mnemonic = line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_start_matches('*');
        // Split the trailing id number off the mnemonic letters (`TR1` -> 1,
        // `F4:N` -> 4).
        let digits: String = mnemonic
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        return match digits.parse::<u32>() {
            Ok(id) => DataCardKey::Int(id),
            Err(_) => DataCardKey::Str(mnemonic.to_ascii_uppercase()),
        };
    }
    DataCardKey::Str(String::new())
}

/// Order data-card file blocks by the id of each block's first data card,
/// keeping the cards inside a block in their original order (a stable sort over
/// whole blocks). Restores the deterministic output the pre-migjorn-0.2 build
/// produced via `order_data_cards_groups_by_id`.
pub fn sort_data_card_chunks(chunks: &mut [String]) {
    chunks.sort_by_key(|chunk| first_data_card_key(chunk));
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

/// Marks a build-output directory as ignored by git, by writing a `.gitignore`
/// containing `*`.
///
/// Only ever *creates*: an existing `.gitignore` is never modified. A build
/// writes into a directory the user chose — with `--output-path` defaulting to
/// `.`, very often a directory that already holds their own work — and
/// overwriting the ignore rules of such a directory silently discards them.
///
/// Uses `create_new` rather than an `exists()` check so the decision and the
/// write are one atomic operation.
pub fn write_output_gitignore(dir: &Path) -> Result<(), GitronicsError> {
    let path = dir.join(".gitignore");
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => file
            .write_all(b"*\n")
            .map_err(|source| GitronicsError::io_path(&path, source)),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            info!(
                "`{}` already exists; leaving it unchanged. Build artefacts in this \
                 directory are only ignored by git if that file says so.",
                path.display()
            );
            Ok(())
        }
        Err(source) => Err(GitronicsError::io_path(&path, source)),
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

/// Upper bound on the global rayon pool — see [`init_thread_pool`]. Past a
/// handful of threads the nesting overhead grows faster than the work shrinks,
/// and a machine with fewer cores than this should not be oversubscribed.
const MAX_RAYON_THREADS: usize = 8;

/// Sizes the global rayon pool once, before any parsing touches it.
///
/// gitronics parallelises across *files* on the same pool migjorn parallelises
/// within a file. Nesting the two on a pool sized to the core count spends most
/// of its time parking and waking threads rather than doing work: on a 96-core
/// machine, assembling a 376 MB model costs 12.1 s of user time against 17.3 s
/// of system time, which capping the pool brings down to 7.3 s and 3.8 s.
/// migjorn's own module documentation names this case and prescribes the cap.
///
/// This does not affect the output. migjorn's segmentation is independent of
/// the pool size, so a build depends only on its inputs — `tests/test_determinism.rs`
/// pins that.
///
/// An explicit `RAYON_NUM_THREADS` is the user's decision and is left alone.
/// Failure to build the pool means one already exists — nothing we need to act
/// on.
pub fn init_thread_pool() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if std::env::var_os("RAYON_NUM_THREADS").is_some() {
            return;
        }
        let threads = std::thread::available_parallelism()
            .map_or(MAX_RAYON_THREADS, |p| p.get().min(MAX_RAYON_THREADS));
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global();
    });
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
    fn test_sort_data_card_chunks_by_first_card_id() {
        // Blocks arrive in arbitrary configuration order.
        let mut chunks = vec![
            "m10 1001 1\nm2 8016 1".to_string(), // first card M10 -> 10
            "C header\nm1 6000 1".to_string(),   // comment skipped, M1 -> 1
            "sdef pos=0 0 0".to_string(),        // no id -> sorts last
            "m3 26000 1".to_string(),            // M3 -> 3
        ];
        sort_data_card_chunks(&mut chunks);
        assert_eq!(
            chunks,
            vec![
                "C header\nm1 6000 1".to_string(),
                "m3 26000 1".to_string(),
                "m10 1001 1\nm2 8016 1".to_string(),
                "sdef pos=0 0 0".to_string(),
            ]
        );
    }

    #[test]
    fn test_sort_data_card_chunks_is_stable_within_block() {
        // Cards inside a block are never reordered, even if not id-sorted.
        let mut chunks = vec!["m5 1001 1\nm2 8016 1".to_string()];
        sort_data_card_chunks(&mut chunks);
        assert_eq!(chunks, vec!["m5 1001 1\nm2 8016 1".to_string()]);
    }

    #[test]
    fn first_data_card_key_reads_the_trailing_id_of_the_mnemonic() {
        assert_eq!(first_data_card_key("m1 1001 1"), DataCardKey::Int(1));
        assert_eq!(first_data_card_key("M100 1001 1"), DataCardKey::Int(100));
        assert_eq!(first_data_card_key("*TR1 0 0 0"), DataCardKey::Int(1));
        assert_eq!(first_data_card_key("F4:N 1"), DataCardKey::Int(4));
        assert_eq!(first_data_card_key("FMESH14:n"), DataCardKey::Int(14));
    }

    #[test]
    fn first_data_card_key_falls_back_to_the_uppercased_mnemonic() {
        assert_eq!(
            first_data_card_key("sdef pos=0 0 0"),
            DataCardKey::Str("SDEF".to_string())
        );
        assert_eq!(
            first_data_card_key("MODE N P"),
            DataCardKey::Str("MODE".to_string())
        );
    }

    #[test]
    fn first_data_card_key_skips_comments_and_blanks() {
        assert_eq!(
            first_data_card_key("C a header\nc another\n\nm7 1001 1"),
            DataCardKey::Int(7)
        );
        // Nothing but comments has no id at all.
        assert_eq!(
            first_data_card_key("C only a comment"),
            DataCardKey::Str(String::new())
        );
        assert_eq!(first_data_card_key(""), DataCardKey::Str(String::new()));
    }

    #[test]
    fn numbered_cards_sort_before_unnumbered_ones() {
        assert!(DataCardKey::Int(9999) < DataCardKey::Str("AAAA".to_string()));
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
