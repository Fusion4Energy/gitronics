use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{EnvelopeName, FileName, FillerName};
use crate::utils::GitronicsError;

/// Configuration for a neutronics model, typically loaded from a YAML file.
///
/// Supports configuration inheritance: if `overrides` is specified, this config
/// will be merged on top of the base configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    project_roots: Option<Vec<PathBuf>>,
    overrides: Option<PathBuf>,
    envelope_structure: Option<FileName>,
    transformations: Option<Vec<FileName>>,
    materials: Option<Vec<FileName>>,
    tallies: Option<Vec<FileName>>,
    source: Option<FileName>,
    #[serde(default)]
    envelopes: IndexMap<EnvelopeName, Option<FillerName>>,
}

impl ModelConfig {
    /// Constructs a `ModelConfig` with only the specified fields set; all others default to `None`.
    pub fn new(
        envelope_structure: Option<FileName>,
        envelopes: IndexMap<EnvelopeName, Option<FillerName>>,
    ) -> Self {
        Self {
            envelope_structure,
            envelopes,
            ..Self::default()
        }
    }

    /// Parses a model configuration from a YAML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, GitronicsError> {
        let yaml_content = fs::read_to_string(&path)?;
        let mut config: ModelConfig = serde_saphyr::from_str(&yaml_content).map_err(|e| {
            GitronicsError::YamlParse(path.as_ref().to_string_lossy().to_string(), e.to_string())
        })?;
        config.resolve_project_roots_relative_to(path.as_ref());
        Ok(config)
    }

    /// Merges this configuration (override) on top of a base configuration.
    ///
    /// Fields present in `self` take precedence; `base` values are used as fallback.
    /// Envelope maps are merged entry-by-entry (not replaced wholesale).
    pub fn merge(self, base: Self) -> Self {
        let mut merged_envelopes = base.envelopes;
        merged_envelopes.extend(self.envelopes);
        Self {
            project_roots: self.project_roots.or(base.project_roots),
            overrides: self.overrides,
            envelope_structure: self.envelope_structure.or(base.envelope_structure),
            source: self.source.or(base.source),
            materials: self.materials.or(base.materials),
            transformations: self.transformations.or(base.transformations),
            tallies: self.tallies.or(base.tallies),
            envelopes: merged_envelopes,
        }
    }

    fn resolve_project_roots_relative_to(&mut self, config_path: &Path) {
        let Some(roots) = &self.project_roots else {
            return; // leave None so a base config's project_roots can be used during merge
        };
        let config_dir = config_path.parent().unwrap_or(Path::new("."));
        let resolved = roots
            .iter()
            .map(|root| {
                if root.is_absolute() {
                    root.clone()
                } else {
                    config_dir.join(root)
                }
            })
            .collect();
        self.project_roots = Some(resolved);
    }

    pub fn set_default_project_root(&mut self, config_dir: &Path) {
        if self.project_roots.is_none() {
            self.project_roots = Some(vec![config_dir.to_path_buf()]);
        }
    }

    pub fn project_roots(&self) -> &[PathBuf] {
        self.project_roots.as_deref().unwrap_or_default()
    }

    pub fn overrides(&self) -> Option<&PathBuf> {
        self.overrides.as_ref()
    }

    pub fn envelope_structure(&self) -> Option<&FileName> {
        self.envelope_structure.as_ref()
    }

    pub fn transformations(&self) -> &[FileName] {
        self.transformations.as_deref().unwrap_or_default()
    }

    pub fn materials(&self) -> &[FileName] {
        self.materials.as_deref().unwrap_or_default()
    }

    pub fn tallies(&self) -> &[FileName] {
        self.tallies.as_deref().unwrap_or_default()
    }

    pub fn source(&self) -> Option<&FileName> {
        self.source.as_ref()
    }

    pub fn set_source(&mut self, source: FileName) {
        self.source = Some(source);
    }

    /// Returns the envelope-to-filler mapping.
    pub fn envelopes(&self) -> &IndexMap<EnvelopeName, Option<FillerName>> {
        &self.envelopes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        fs::write(
            &config_path,
            r#"
envelope_structure: test_structure
source: test_source
materials: [test_material]
envelopes:
  env1: test_env1
  env2:
"#,
        )
        .unwrap();

        let config = ModelConfig::from_file(&config_path).unwrap();
        assert_eq!(
            config.envelope_structure().unwrap(),
            &FileName::new("test_structure")
        );
        assert_eq!(config.source().unwrap(), &FileName::new("test_source"));
        assert_eq!(config.materials(), vec![FileName::new("test_material")]);
        assert!(config.transformations().is_empty());
        assert!(config.tallies().is_empty());
        let envelopes = config.envelopes();
        assert_eq!(
            envelopes[&EnvelopeName::new("env1")],
            Some(FillerName::new("test_env1"))
        );
        assert!(envelopes[&EnvelopeName::new("env2")].is_none());
    }

    #[test]
    fn test_merge() {
        let base = ModelConfig {
            project_roots: Some(vec![PathBuf::from("base_root")]),
            overrides: None,
            envelope_structure: Some(FileName::new("base_structure")),
            source: Some(FileName::new("base_source")),
            materials: Some(vec![FileName::new("base_material")]),
            transformations: None,
            tallies: None,
            envelopes: [("env1".into(), Some(FillerName::new("base_env1")))].into(),
        };
        let overr = ModelConfig {
            project_roots: None,
            overrides: Some("base".into()),
            envelope_structure: None,
            source: Some(FileName::new("override_source")),
            materials: Some(vec![FileName::new("override_material")]),
            transformations: None,
            tallies: None,
            envelopes: [("env2".into(), Some(FillerName::new("override_env2")))].into(),
        };
        let merged = overr.merge(base);
        assert_eq!(
            merged.envelope_structure().unwrap(),
            &FileName::new("base_structure")
        );
        assert_eq!(merged.source().unwrap(), &FileName::new("override_source"));
        assert_eq!(merged.materials(), vec![FileName::new("override_material")]);
        assert_eq!(merged.project_roots(), vec![PathBuf::from("base_root")]);
        let envelopes = merged.envelopes();
        assert_eq!(
            envelopes[&EnvelopeName::new("env1")],
            Some(FillerName::new("base_env1"))
        );
        assert_eq!(
            envelopes[&EnvelopeName::new("env2")],
            Some(FillerName::new("override_env2"))
        );
    }
}
