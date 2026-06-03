use super::ProjectManager;
use crate::types::{EnvelopeName, FillerMetadata, FillerName};
use crate::utils::GitronicsError;
use migjorn::Model;
use std::{collections::HashMap, fs};

impl ProjectManager {
    /// Loads and caches metadata for the given filler models.
    ///
    /// Reads the `.metadata` files associated with each filler and stores the transformation
    /// mappings in the internal cache of the `ProjectManager`.
    pub fn load_metadata_for_fillers(&mut self, fillers: &[Model]) -> Result<(), GitronicsError> {
        for filler in fillers {
            let filler_name = FillerName::from(filler);
            self.load_metadata(&filler_name)?;
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
        let yaml_content = fs::read_to_string(&metadata_path)?;
        let filler_metadata: FillerMetadata =
            serde_saphyr::from_str(&yaml_content).map_err(|e| {
                GitronicsError::YamlParse(
                    metadata_path.to_string_lossy().to_string(),
                    e.to_string(),
                )
            })?;

        let transformations: HashMap<EnvelopeName, Option<String>> = filler_metadata
            .transformations
            .unwrap_or_default()
            .into_iter()
            .collect();

        self.metadata.insert(filler_name.clone(), transformations);
        Ok(())
    }
}
