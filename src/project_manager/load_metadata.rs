use super::{FillerRecord, Metadata, ProjectManager};
use crate::error::GitronicsError;
use crate::types::{EnvelopeName, FillerName, TRANSFORMATIONS_KEY};
use indexmap::IndexMap;
use log::warn;
use serde_json::Value;
use std::{collections::HashMap, fs};

impl ProjectManager {
    /// Loads and caches metadata for the given fillers.
    ///
    /// Reads each filler's `.metadata` file generically: the reserved
    /// `transformations` key drives the build, and every other key is preserved
    /// verbatim as arbitrary, project-defined metadata.
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

        // Parse the whole file as a free-form, order-preserving map.
        let mut raw: Metadata =
            serde_saphyr::from_str(&yaml_content).map_err(|source| GitronicsError::YamlParse {
                path: metadata_path.display().to_string(),
                source: Box::new(source),
            })?;

        // Extract the reserved `transformations` key; everything else is
        // arbitrary metadata that we keep as-is.
        let transformations: HashMap<EnvelopeName, Option<String>> =
            match raw.shift_remove(TRANSFORMATIONS_KEY) {
                Some(value) if !value.is_null() => {
                    let map: IndexMap<EnvelopeName, Option<String>> = serde_json::from_value(value)
                        .map_err(|source| GitronicsError::InvalidTransformations {
                            filler_name: filler_name.clone(),
                            source,
                        })?;
                    map.into_iter().collect()
                }
                _ => HashMap::new(),
            };

        self.filler_data.insert(
            filler_name.clone(),
            FillerRecord {
                transformations,
                metadata: raw,
            },
        );
        Ok(())
    }

    /// Loads the arbitrary metadata sidecar of the envelope-structure model
    /// (`<envelope_structure>.metadata`), keyed per envelope. The only expected
    /// shape is a top-level `envelopes:` map; each envelope's value is stored
    /// verbatim as free-form metadata. Best-effort: a missing or malformed file
    /// is logged and ignored so it never blocks a build.
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

        let top: IndexMap<String, Value> = match serde_saphyr::from_str(&yaml_content) {
            Ok(parsed) => parsed,
            Err(e) => {
                warn!(
                    "Could not parse envelope metadata `{}`: {e}",
                    metadata_path.display()
                );
                return;
            }
        };

        let Some(Value::Object(envelopes)) = top.get("envelopes") else {
            warn!(
                "Envelope metadata `{}` has no `envelopes:` map; ignoring.",
                metadata_path.display()
            );
            return;
        };

        for (name, value) in envelopes {
            let fields: Metadata = match value {
                Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                // A non-map entry (e.g. just a bare description) is still kept.
                other => {
                    let mut m = Metadata::new();
                    m.insert("value".to_string(), other.clone());
                    m
                }
            };
            self.envelope_metadata
                .insert(EnvelopeName::new(name.clone()), fields);
        }
    }
}
