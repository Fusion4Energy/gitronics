//! The crate-wide error type.

use crate::types::{EnvelopeName, FileName, FillerName};
use migjorn::{IdKind, Problem};
use path_clean::PathClean;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

fn join_lines<T: fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// A [`migjorn::Collision`] with its model positions already resolved to the
/// names `build_model::merge_labeled` was given for them, so a display site
/// never has to index back into a side table to say which component collided.
#[derive(Debug)]
pub struct MergeConflict {
    pub kind: IdKind,
    pub id: i64,
    pub models: Vec<String>,
}

fn format_merge_conflicts(conflicts: &[MergeConflict]) -> String {
    conflicts
        .iter()
        .map(|c| {
            let defined_by = c.models.join(" and ");
            format!("duplicate {} id {} defined by {defined_by}", c.kind, c.id)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

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

    #[error("YAML parsing error of file `{path}`.\n{source}")]
    YamlParse {
        path: String,
        // Boxed: `serde_saphyr::Error` carries per-variant source `Location`s and
        // is the largest field in this enum by a wide margin (128+ bytes) —
        // inlining it would bloat every `Result<_, GitronicsError>` in the crate.
        #[source]
        source: Box<serde_saphyr::Error>,
    },

    #[error("YAML serialization error of file `{path}`.\n{source}")]
    YamlSerialize {
        path: String,
        #[source]
        source: serde_saphyr::ser_error::Error,
    },

    #[error("Invalid `transformations` in metadata of filler `{filler_name}`.\n{source}")]
    InvalidTransformations {
        filler_name: FillerName,
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to load MCNP file `{file_name}` at line {line}:\n{message}")]
    FailedToLoadMCNPFile {
        file_name: FileName,
        line: usize,
        message: String,
    },

    #[error(
        "Failed to load data cards file `{0}`: no data cards found before the first blank line"
    )]
    FailedToLoadDataCardsFile(FileName),

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
        "ID collisions between components (each id must be unique across the envelope structure and all fillers):\n{}",
        format_merge_conflicts(.0)
    )]
    MergeConflicts(Vec<MergeConflict>),

    #[error("The assembled model has invalid references:\n{}", join_lines(.0))]
    InvalidModel(Vec<Problem>),

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

#[cfg(test)]
mod tests {
    use super::*;

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
