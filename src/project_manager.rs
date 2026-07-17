use crate::model_config::ModelConfig;
use crate::project_manager::load_model_config::load_config;
use crate::types::{EnvelopeMetadata, EnvelopeName, FileName, FillerMetadata, FillerName};
use crate::utils::{GitronicsError, get_file_paths};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs::create_dir_all;
use std::path::PathBuf;

mod load_metadata;
mod load_model_config;
mod load_project_files;

/// Manages a gitronics project, providing access to model files and configurations.
#[derive(Debug)]
pub struct ProjectManager {
    model_config: ModelConfig,
    output_path: PathBuf,
    file_paths: HashMap<FileName, PathBuf>,
    metadata: HashMap<FillerName, HashMap<EnvelopeName, Option<String>>>,
    /// Full parsed filler metadata (description, pbs, card-id range), cached per
    /// filler as its `.metadata` sidecar is loaded.
    filler_details: HashMap<FillerName, FillerMetadata>,
    /// Descriptive metadata for each envelope, loaded from the envelope-structure
    /// `.metadata` sidecar (best-effort; empty when the file is absent).
    envelope_details: HashMap<EnvelopeName, EnvelopeMetadata>,
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
            filler_details: HashMap::new(),
            envelope_details: HashMap::new(),
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

    /// Returns the free-text description of a filler, from its cached metadata.
    pub fn filler_description(&self, filler_name: &FillerName) -> Option<&str> {
        self.filler_details
            .get(filler_name)
            .and_then(|m| m.description.as_deref())
    }

    /// Returns the product-breakdown-structure (PBS) code of a filler, if any.
    pub fn filler_pbs(&self, filler_name: &FillerName) -> Option<&str> {
        self.filler_details
            .get(filler_name)
            .and_then(|m| m.pbs.as_deref())
    }

    /// Returns the descriptive metadata for an envelope, if it was loaded.
    pub fn envelope_metadata(&self, envelope_name: &EnvelopeName) -> Option<&EnvelopeMetadata> {
        self.envelope_details.get(envelope_name)
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
