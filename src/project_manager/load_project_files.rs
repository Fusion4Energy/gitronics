//! Loading of model files and data cards.
//!
//! This module extends [`ProjectManager`] with methods to load:
//! - Envelope structure models
//! - Filler models (in parallel using rayon)
//! - Data cards (transformations, materials, tallies, sources)

use std::path::PathBuf;

use log::{info, warn};
use migjorn::{DataCard, Model, load_data_cards_file};
use rayon::prelude::*;

use crate::types::{FileName, FillerName};
use crate::utils::GitronicsError;

use super::ProjectManager;

impl ProjectManager {
    /// Loads the envelope structure model from configuration.
    ///
    /// Loads the MCNP model file specified in the configuration's `envelope_structure`
    /// field. The model's data cards are cleared (they'll be populated separately).
    pub fn load_envelope_structure(&self) -> Result<Model, GitronicsError> {
        let envelope_structure_name = self
            .model_config
            .envelope_structure()
            .ok_or(GitronicsError::MissingEnvelopeStructureInConfig)?;
        let envelope_structure_path = self.file_path(envelope_structure_name)?;

        info!("Loading: {}", envelope_structure_path.display());
        let mut envelope_structure_model =
            Model::from_file(envelope_structure_path).map_err(|err| {
                GitronicsError::FailedToLoadMCNPFile {
                    file_name: envelope_structure_name.clone(),
                    error: err.to_string(),
                }
            })?;
        envelope_structure_model.data_cards.clear();
        Ok(envelope_structure_model)
    }

    /// Loads all filler models specified in the configuration.
    ///
    /// Loads filler models in parallel using rayon for better performance.
    /// Only loads unique fillers (if the same filler is used multiple times,
    /// it's only loaded once).
    pub fn load_fillers(&self) -> Result<Vec<Model>, GitronicsError> {
        // Get all filler names from the config, ordered and deduplicated
        let mut filler_names: Vec<&FillerName> =
            self.model_config.envelopes().values().flatten().collect();
        filler_names.sort();
        filler_names.dedup();

        let name_and_paths: Vec<_> = filler_names
            .iter()
            .map(|&filler_name| {
                let filler_path = self.file_path(&filler_name.into())?;
                Ok((filler_name, filler_path))
            })
            .collect::<Result<Vec<(&FillerName, &PathBuf)>, GitronicsError>>()?;

        // Log filler names in alphabetical order before parallel loading
        for (_, filler_path) in &name_and_paths {
            info!("Loading: {}", filler_path.display());
        }

        // Load each filler model in parallel using rayon
        let fillers = name_and_paths
            .into_par_iter()
            .map(|(filler_name, filler_path)| {
                Model::from_file(filler_path).map_err(|err| GitronicsError::FailedToLoadMCNPFile {
                    file_name: FileName::from(filler_name),
                    error: err.to_string(),
                })
            })
            .collect::<Result<Vec<_>, GitronicsError>>()?;

        Ok(fillers)
    }

    /// Loads transformation data cards from the specified files.
    ///
    /// # Returns
    /// A vector of data card results, one per file specified in config
    pub fn load_transforms(&self) -> Result<Vec<Vec<DataCard>>, GitronicsError> {
        self.load_data_cards_by_file_name(self.model_config.transformations())
    }

    /// Loads material data cards from the specified files.
    ///
    /// # Returns
    /// A vector of data card results, one per file specified in config
    pub fn load_materials(&self) -> Result<Vec<Vec<DataCard>>, GitronicsError> {
        self.load_data_cards_by_file_name(self.model_config.materials())
    }

    /// Loads tally data cards from the specified files.
    ///
    /// # Returns
    /// A vector of data card results, one per file specified in config
    pub fn load_tallies(&self) -> Result<Vec<Vec<DataCard>>, GitronicsError> {
        self.load_data_cards_by_file_name(self.model_config.tallies())
    }

    /// Loads source definition data cards.
    ///
    /// # Returns
    /// * Data cards from the source file, or empty vector if no source specified
    pub fn load_source(&self) -> Result<Vec<DataCard>, GitronicsError> {
        match self.model_config.source() {
            Some(source_name) => {
                let source_path = self.file_path(source_name)?;
                info!("Loading: {}", source_path.display());
                load_data_cards_file(source_path).map_err(|err| {
                    GitronicsError::FailedToLoadDataCardsFile {
                        file_name: source_name.clone(),
                        error: err.to_string(),
                    }
                })
            }
            None => {
                warn!("No source file specified in configuration");
                Ok(vec![])
            }
        }
    }

    /// Helper to load data cards from multiple files by name.
    fn load_data_cards_by_file_name(
        &self,
        names: &[FileName],
    ) -> Result<Vec<Vec<DataCard>>, GitronicsError> {
        names
            .iter()
            .map(|name| {
                let path = self.file_path(name)?;
                info!("Loading: {}", path.display());
                load_data_cards_file(path).map_err(|err| {
                    GitronicsError::FailedToLoadDataCardsFile {
                        file_name: name.clone(),
                        error: err.to_string(),
                    }
                })
            })
            .collect::<Result<Vec<Vec<DataCard>>, GitronicsError>>()
    }
}
