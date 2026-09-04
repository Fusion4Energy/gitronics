//! Filesystem discovery and side effects shared across commands.

use crate::error::GitronicsError;
use crate::types::FileName;
use log::info;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const VALID_SUFFIXES: &[&str] = &["yaml", "yml", "mcnp", "mat", "tally", "transform", "source"];

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
