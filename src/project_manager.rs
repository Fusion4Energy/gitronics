use crate::model_config::ModelConfig;
use crate::project_manager::load_model_config::load_config;
use crate::types::{EnvelopeName, FileName, FillerName};
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
                    .map(|p| {
                        dunce::canonicalize(p)
                            .unwrap_or_else(|_| p.clone())
                            .to_string_lossy()
                            .to_string()
                    })
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
            .expect("metadata should exist after successful load_metadata call");
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
