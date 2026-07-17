use crate::model_config::ModelConfig;
use crate::project_manager::load_model_config::load_config;
use crate::types::{EnvelopeName, FileName, FillerName};
use crate::utils::{GitronicsError, get_file_paths};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs::create_dir_all;
use std::path::PathBuf;

mod load_metadata;
mod load_model_config;
mod load_project_files;

/// Free-form metadata: an ordered map of arbitrary, project-defined keys to
/// their (possibly structured) values. Key order follows the source file.
pub type Metadata = IndexMap<String, Value>;

/// Manages a gitronics project, providing access to model files and configurations.
#[derive(Debug)]
pub struct ProjectManager {
    model_config: ModelConfig,
    output_path: PathBuf,
    file_paths: HashMap<FileName, PathBuf>,
    metadata: HashMap<FillerName, HashMap<EnvelopeName, Option<String>>>,
    /// Arbitrary, project-defined metadata for each filler (everything in the
    /// `.metadata` file except the reserved `transformations` key).
    filler_metadata: HashMap<FillerName, Metadata>,
    /// Arbitrary, project-defined metadata for each envelope, from the
    /// envelope-structure `.metadata` sidecar (best-effort; empty when absent).
    envelope_metadata: HashMap<EnvelopeName, Metadata>,
}

impl ProjectManager {
    /// Creates a new `ProjectManager` for the given output directory.
    pub fn new<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        config_path: P,
        output_path: Q,
    ) -> Result<Self, GitronicsError> {
        let output_path = output_path.as_ref().to_path_buf();
        create_dir_all(&output_path)?;
        let model_config = load_config(&config_path)?;
        let file_paths = index_project_files(&model_config)?;
        Ok(Self {
            file_paths,
            output_path,
            model_config,
            metadata: HashMap::new(),
            filler_metadata: HashMap::new(),
            envelope_metadata: HashMap::new(),
        })
    }

    /// Retrieves the full path for a project file by its stem name.
    pub fn file_path(&self, file_name: &FileName) -> Result<&PathBuf, GitronicsError> {
        self.file_paths
            .get(file_name)
            .ok_or_else(|| GitronicsError::FileNotFound {
                file_name: file_name.clone(),
                project_root: self
                    .model_config
                    .project_roots()
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            })
    }

    /// Returns the configured output path.
    pub fn output_path(&self) -> &PathBuf {
        &self.output_path
    }

    /// Retrieves the envelope-to-filler mapping from the configuration.
    pub fn filler_by_envelope(&self, envelope_name: &EnvelopeName) -> Option<&Option<FillerName>> {
        self.model_config.envelopes().get(envelope_name)
    }

    /// Retrieves the transformation name for a filler in a specific envelope.
    /// Loads the filler's metadata if not already cached.
    pub fn transformation(
        &self,
        filler_name: &FillerName,
        envelope_name: &EnvelopeName,
    ) -> Result<Option<&str>, GitronicsError> {
        let filler_metadata = self
            .metadata
            .get(filler_name)
            .ok_or_else(|| GitronicsError::MetadataNotFound(filler_name.into()))?;
        if !filler_metadata.contains_key(envelope_name) {
            return Err(GitronicsError::TransformationNotFound {
                filler_name: filler_name.clone(),
                envelope_name: envelope_name.clone(),
            });
        }
        Ok(filler_metadata
            .get(envelope_name)
            .and_then(|opt| opt.as_deref()))
    }

    /// Returns an iterator over the envelope names defined in the configuration.
    pub fn envelopes_in_config(&self) -> impl Iterator<Item = &EnvelopeName> {
        self.model_config.envelopes().keys()
    }

    /// Returns the list of material file names from the configuration.
    pub fn materials_names(&self) -> &[FileName] {
        self.model_config.materials()
    }

    /// Returns the list of tally file names from the configuration.
    pub fn tallies_names(&self) -> &[FileName] {
        self.model_config.tallies()
    }

    /// Returns the list of transformation file names from the configuration.
    pub fn transforms_names(&self) -> &[FileName] {
        self.model_config.transformations()
    }

    /// Returns the source file name from the configuration, if any.
    pub fn source_name(&self) -> Option<&FileName> {
        self.model_config.source()
    }

    /// Returns the arbitrary, project-defined metadata of a filler, if loaded.
    pub fn filler_metadata(&self, filler_name: &FillerName) -> Option<&Metadata> {
        self.filler_metadata.get(filler_name)
    }

    /// Returns the arbitrary, project-defined metadata of an envelope, if loaded.
    pub fn envelope_metadata(&self, envelope_name: &EnvelopeName) -> Option<&Metadata> {
        self.envelope_metadata.get(envelope_name)
    }
}

/// Indexes project files using roots resolved from a loaded configuration.
fn index_project_files(
    model_config: &ModelConfig,
) -> Result<HashMap<FileName, PathBuf>, GitronicsError> {
    let roots = model_config.project_roots();

    let mut merged: HashMap<FileName, PathBuf> = HashMap::new();
    for root in roots.iter() {
        let root_files = get_file_paths(root)?;
        for (name, path) in root_files {
            match merged.entry(name) {
                Entry::Occupied(entry) => {
                    return Err(GitronicsError::DuplicateFileName(entry.key().clone()));
                }
                Entry::Vacant(entry) => {
                    entry.insert(dunce::canonicalize(path)?);
                }
            }
        }
    }

    Ok(merged)
}
