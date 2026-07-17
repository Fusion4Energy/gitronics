use super::ProjectManager;
use crate::types::{EnvelopeName, EnvelopeStructureMetadata, FillerMetadata, FillerName};
use crate::utils::GitronicsError;
use log::warn;
use std::{collections::HashMap, fs};

impl ProjectManager {
    /// Loads and caches metadata for the given fillers.
    ///
    /// Reads the `.metadata` files associated with each filler and stores the transformation
    /// mappings in the internal cache of the `ProjectManager`.
    pub fn load_metadata_for_fillers<'a>(
        &mut self,
        filler_names: impl IntoIterator<Item = &'a FillerName>,
    ) -> Result<(), GitronicsError> {
        for filler_name in filler_names {
            self.load_metadata(filler_name)?;
        }
        Ok(())
    }

    fn load_metadata(&mut self, filler_name: &FillerName) -> Result<(), GitronicsError> {
        let metadata_path = self
            .file_path(&filler_name.into())?
            .with_extension("metadata");
        if !metadata_path.exists() {
            return Err(GitronicsError::MetadataNotFound(filler_name.into()));
        }
        let yaml_content = fs::read_to_string(&metadata_path)
            .map_err(|source| GitronicsError::io_path(&metadata_path, source))?;
        let filler_metadata: FillerMetadata =
            serde_saphyr::from_str(&yaml_content).map_err(|e| {
                GitronicsError::YamlParse(
                    metadata_path.to_string_lossy().to_string(),
                    e.to_string(),
                )
            })?;

        let transformations: HashMap<EnvelopeName, Option<String>> = filler_metadata
            .transformations
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect();

        self.metadata.insert(filler_name.clone(), transformations);
        self.filler_details
            .insert(filler_name.clone(), filler_metadata);
        Ok(())
    }

    /// Loads the descriptive metadata sidecar of the envelope-structure model
    /// (`<envelope_structure>.metadata`), caching per-envelope description, zone
    /// and sector. Best-effort: a missing or malformed file is logged and
    /// ignored so it never blocks a build.
    pub fn load_envelope_metadata(&mut self) {
        let Some(structure_name) = self.model_config.envelope_structure() else {
            return;
        };
        let Ok(structure_path) = self.file_path(structure_name) else {
            return;
        };
        let metadata_path = structure_path.with_extension("metadata");
        if !metadata_path.exists() {
            return;
        }
        let yaml_content = match fs::read_to_string(&metadata_path) {
            Ok(content) => content,
            Err(e) => {
                warn!(
                    "Could not read envelope metadata `{}`: {e}",
                    metadata_path.display()
                );
                return;
            }
        };
        match serde_saphyr::from_str::<EnvelopeStructureMetadata>(&yaml_content) {
            Ok(parsed) => {
                self.envelope_details = parsed.envelopes.into_iter().collect();
            }
            Err(e) => warn!(
                "Could not parse envelope metadata `{}`: {e}",
                metadata_path.display()
            ),
        }
    }
}
