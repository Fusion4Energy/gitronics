//! Loading of model files and data cards.
//!
//! This module extends [`ProjectManager`] with methods to load:
//! - The envelope structure model
//! - Filler models (in parallel using rayon)
//! - Data-card text (transformations, materials, tallies, sources)

use log::{info, warn};
use migjorn::Model;
use rayon::prelude::*;

use crate::error::GitronicsError;
use crate::mcnp_io::{parse_model_file, read_data_cards_text, sort_data_card_chunks};
use crate::types::{FileName, FillerName};

use super::ProjectManager;

impl ProjectManager {
    /// Loads and parses the envelope structure model named in the configuration.
    pub fn load_envelope_structure(&self) -> Result<Model, GitronicsError> {
        let envelope_structure_name = self
            .model_config
            .envelope_structure()
            .ok_or(GitronicsError::MissingEnvelopeStructureInConfig)?;
        let envelope_structure_path = self.file_path(envelope_structure_name)?;

        info!("Loading: {}", envelope_structure_path.display());
        parse_model_file(envelope_structure_path, envelope_structure_name)
    }

    /// Loads all filler models referenced by the configuration, paired with
    /// their names. Fillers are deduplicated (a filler used by several envelopes
    /// is loaded once) and parsed in parallel using rayon.
    pub fn load_fillers(&self) -> Result<Vec<(FillerName, Model)>, GitronicsError> {
        // Filler names from the config, ordered and deduplicated.
        let mut filler_names: Vec<&FillerName> =
            self.model_config.envelopes().values().flatten().collect();
        filler_names.sort();
        filler_names.dedup();

        let name_and_paths = filler_names
            .iter()
            .map(|&filler_name| {
                let filler_path = self.file_path(&filler_name.into())?;
                Ok((filler_name, filler_path))
            })
            .collect::<Result<Vec<_>, GitronicsError>>()?;

        for (_, filler_path) in &name_and_paths {
            info!("Loading: {}", filler_path.display());
        }

        name_and_paths
            .into_par_iter()
            .map(|(filler_name, filler_path)| {
                let file_name = FileName::from(filler_name);
                let model = parse_model_file(filler_path, &file_name)?;
                Ok((filler_name.clone(), model))
            })
            .collect::<Result<Vec<_>, GitronicsError>>()
    }

    /// Loads all filler models (as [`Self::load_fillers`]) and, in the same
    /// call, caches each one's metadata so [`Self::transformation`] is safe to
    /// call for any of the returned fillers immediately afterwards.
    ///
    /// Pairing the two loads here — rather than leaving callers to sequence
    /// `load_fillers` and `load_metadata_for_fillers` themselves — means the
    /// precondition `transformation` documents can't be forgotten.
    pub fn load_fillers_with_metadata(
        &mut self,
    ) -> Result<Vec<(FillerName, Model)>, GitronicsError> {
        let fillers = self.load_fillers()?;
        self.load_metadata_for_fillers(fillers.iter().map(|(name, _)| name))?;
        Ok(fillers)
    }

    /// Loads the transformation data-card text from the configured files.
    pub fn load_transforms(&self) -> Result<String, GitronicsError> {
        self.load_data_cards_text(self.model_config.transformations())
    }

    /// Loads the material data-card text from the configured files.
    pub fn load_materials(&self) -> Result<String, GitronicsError> {
        self.load_data_cards_text(self.model_config.materials())
    }

    /// Loads the tally data-card text from the configured files.
    pub fn load_tallies(&self) -> Result<String, GitronicsError> {
        self.load_data_cards_text(self.model_config.tallies())
    }

    /// Loads the source data-card text, or an empty string if no source file is
    /// specified in the configuration.
    pub fn load_source(&self) -> Result<String, GitronicsError> {
        match self.model_config.source() {
            Some(source_name) => {
                let source_path = self.file_path(source_name)?;
                info!("Loading: {}", source_path.display());
                read_data_cards_text(source_path, source_name)
            }
            None => {
                warn!("No source file specified in configuration");
                Ok(String::new())
            }
        }
    }

    /// Concatenates the data-card text of every file in `names`, one blank-line
    /// separated block per file. Blocks are ordered by the id of each file's
    /// first data card (deterministic output, independent of configuration
    /// order); cards inside a file keep their original order.
    fn load_data_cards_text(&self, names: &[FileName]) -> Result<String, GitronicsError> {
        let mut chunks = Vec::new();
        for name in names {
            let path = self.file_path(name)?;
            info!("Loading: {}", path.display());
            chunks.push(read_data_cards_text(path, name)?);
        }
        sort_data_card_chunks(&mut chunks);
        Ok(chunks.join("\n"))
    }
}
