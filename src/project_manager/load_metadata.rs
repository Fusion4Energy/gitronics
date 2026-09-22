use super::{FillerRecord, Metadata, ProjectManager};
use crate::error::GitronicsError;
use crate::provenance::SourceFile;
use crate::types::{EnvelopeName, FillerName, TRANSFORMATIONS_KEY};
use indexmap::IndexMap;
use log::warn;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// The only top-level key gitronics reads from an envelope-structure metadata
/// sidecar; every other top-level key is project-specific and left alone.
const ENVELOPES_KEY: &str = "envelopes";

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
        let (yaml_content, source) = SourceFile::read(&metadata_path)?;
        self.evidence
            .record_input("filler_metadata", filler_name.as_ref(), source);

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
    /// (`<envelope_structure>.metadata`), keyed per envelope. Only the top-level
    /// `envelopes:` map is read, and each envelope's value is stored verbatim as
    /// free-form metadata; any other top-level key is the project's own and is
    /// ignored. Best-effort: a missing or malformed file is logged and ignored
    /// so it never blocks a build.
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
        let yaml_content = match SourceFile::read(&metadata_path) {
            Ok((content, source)) => {
                self.evidence
                    .record_input("envelope_metadata", structure_name.as_ref(), source);
                content
            }
            Err(e) => {
                self.report_warnings.push(format!(
                    "Could not read envelope metadata {}: {e}",
                    metadata_path.display()
                ));
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
                self.report_warnings.push(format!(
                    "Could not parse envelope metadata {}: {e}",
                    metadata_path.display()
                ));
                warn!(
                    "Could not parse envelope metadata `{}`: {e}",
                    metadata_path.display()
                );
                return;
            }
        };

        // No `envelopes:` map: the sidecar holds only project-specific fields.
        let Some(Value::Object(envelopes)) = top.get(ENVELOPES_KEY) else {
            return;
        };

        for (name, value) in envelopes {
            let fields: Metadata = match value {
                Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                // A non-map entry is still kept. A bare string is a description
                // (the report shows that key specially); anything else is
                // stored under a generic `value` key.
                other => {
                    let key = if other.is_string() {
                        "description"
                    } else {
                        "value"
                    };
                    let mut m = Metadata::new();
                    m.insert(key.to_string(), other.clone());
                    m
                }
            };
            self.envelope_metadata
                .insert(EnvelopeName::new(name.clone()), fields);
        }
    }

    /// Warns about envelope metadata whose name matches no `$ @env:` marker in
    /// the envelope structure: such an entry is never shown anywhere, so it is
    /// usually a misspelt name or an envelope that was removed from the file.
    /// Call after [`Self::load_envelope_metadata`].
    pub fn warn_about_unmatched_envelope_metadata(
        &mut self,
        marked_envelopes: &HashSet<EnvelopeName>,
    ) {
        let mut unmatched: Vec<String> = self
            .envelope_metadata
            .keys()
            .filter(|name| !marked_envelopes.contains(*name))
            .map(ToString::to_string)
            .collect();
        if unmatched.is_empty() {
            return;
        }
        unmatched.sort();
        let names = unmatched.join(", ");
        self.report_warnings.push(format!(
            "Envelope metadata defines envelopes that are not marked in the envelope structure: {names}"
        ));
        warn!(
            "The envelope metadata defines envelopes that have no `$ @env:name` marker in the \
             envelope structure file, so they are ignored: {names}"
        );
    }
}
