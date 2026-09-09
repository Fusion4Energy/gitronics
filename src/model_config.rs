use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::GitronicsError;
use crate::types::{EnvelopeName, FileName, FillerName};

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
    #[serde(
        default,
        deserialize_with = "deserialize_source",
        skip_serializing_if = "Option::is_none"
    )]
    source: Option<Option<FileName>>,
    #[serde(default)]
    envelopes: IndexMap<EnvelopeName, Option<FillerName>>,
}

fn deserialize_source<'de, Deserializer>(
    deserializer: Deserializer,
) -> Result<Option<Option<FileName>>, Deserializer::Error>
where
    Deserializer: serde::Deserializer<'de>,
{
    Option::<FileName>::deserialize(deserializer).map(Some)
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
        let yaml_content =
            fs::read_to_string(&path).map_err(|source| GitronicsError::io_path(&path, source))?;
        let mut config: ModelConfig =
            serde_saphyr::from_str(&yaml_content).map_err(|source| GitronicsError::YamlParse {
                path: path.as_ref().to_string_lossy().to_string(),
                source: Box::new(source),
            })?;
        config.resolve_project_roots_relative_to(path.as_ref());
        Ok(config)
    }

    /// Loads a model configuration by path.
    ///
    /// If the configuration specifies an `overrides` field, recursively loads
    /// the base configuration and merges them. Detects and prevents circular
    /// override chains.
    pub fn load<P: AsRef<Path>>(config_path: P) -> Result<Self, GitronicsError> {
        let config_path = if config_path.as_ref().is_absolute() {
            config_path.as_ref().to_path_buf()
        } else {
            current_dir()?.join(config_path)
        };
        let config_path = dunce::canonicalize(&config_path)
            .map_err(|source| GitronicsError::io_path(&config_path, source))?;
        Self::load_inner(&config_path, &mut HashSet::new())
    }

    fn load_inner(
        config_path: &Path,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<Self, GitronicsError> {
        let config_path = config_path.to_path_buf();
        if !visited.insert(config_path.clone()) {
            return Err(GitronicsError::ConfigCycle(config_path));
        }
        let mut config = Self::from_file(&config_path)?;
        // If there is no `overrides` key, apply default project root and return.
        let Some(base_path) = config.overrides() else {
            let config_dir = config_path.parent().unwrap_or(Path::new("."));
            config.set_default_project_root(config_dir);
            return Ok(config);
        };
        let base_path = if base_path.is_absolute() {
            base_path.to_path_buf()
        } else {
            config_path
                .parent()
                .unwrap_or(Path::new("."))
                .join(base_path)
        };
        let base_path = dunce::canonicalize(&base_path)
            .map_err(|source| GitronicsError::io_path(&base_path, source))?;
        // Resolve the base config recursively so the full chain is applied.
        let base = Self::load_inner(&base_path, visited)?;
        Ok(config.merge(base))
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
        self.source.as_ref().and_then(Option::as_ref)
    }

    pub fn set_source(&mut self, source: FileName) {
        self.source = Some(Some(source));
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
            source: Some(Some(FileName::new("base_source"))),
            materials: Some(vec![FileName::new("base_material")]),
            transformations: None,
            tallies: None,
            envelopes: [("env1".into(), Some(FillerName::new("base_env1")))].into(),
        };
        let overr = ModelConfig {
            project_roots: None,
            overrides: Some("base".into()),
            envelope_structure: None,
            source: Some(Some(FileName::new("override_source"))),
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

    #[test]
    fn source_inheritance_distinguishes_omitted_null_and_assigned() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("base.yaml"), "source: base_source\n").unwrap();

        for (source_yaml, expected) in [
            ("", Some("base_source")),
            ("source: null\n", None),
            ("source:\n", None),
            ("source: replacement\n", Some("replacement")),
        ] {
            let config_path = dir.path().join("override.yaml");
            fs::write(&config_path, format!("overrides: base.yaml\n{source_yaml}")).unwrap();

            let config = ModelConfig::load(&config_path).unwrap();

            assert_eq!(config.source().map(|name| &**name), expected);
        }
    }

    #[test]
    fn cleared_source_stays_cleared_through_override_chain() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("base.yaml"), "source: base_source\n").unwrap();
        fs::write(
            dir.path().join("mid.yaml"),
            "overrides: base.yaml\nsource: null\n",
        )
        .unwrap();
        let top_path = dir.path().join("top.yaml");
        fs::write(&top_path, "overrides: mid.yaml\n").unwrap();

        assert!(ModelConfig::load(&top_path).unwrap().source().is_none());

        fs::write(&top_path, "overrides: mid.yaml\nsource: replacement\n").unwrap();
        assert_eq!(
            ModelConfig::load(&top_path).unwrap().source(),
            Some(&FileName::new("replacement"))
        );
    }

    #[test]
    fn source_states_survive_yaml_round_trip() {
        for yaml in ["{}", "source: null", "source: named_source"] {
            let config: ModelConfig = serde_saphyr::from_str(yaml).unwrap();
            let serialized = serde_saphyr::to_string(&config).unwrap();
            let restored: ModelConfig = serde_saphyr::from_str(&serialized).unwrap();

            assert_eq!(restored, config, "source state changed for {yaml}");
        }
    }

    #[test]
    fn test_load_config_override_chain() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("base.yaml"),
            r#"
envelope_structure: base_structure
source: base_source
envelopes:
  env1: base_env1
"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("mid.yaml"),
            r#"
overrides: ./base.yaml
source: mid_source
envelopes:
  env2: mid_env2
"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("top.yaml"),
            r#"
overrides: ./mid.yaml
envelopes:
  env3: top_env3
"#,
        )
        .unwrap();

        let config = ModelConfig::load(dir.path().join("top.yaml")).unwrap();
        assert_eq!(
            config.envelope_structure().unwrap(),
            &FileName::new("base_structure")
        );
        assert_eq!(config.source().unwrap(), &FileName::new("mid_source"));
        let envelopes = config.envelopes();

        assert_eq!(
            envelopes[&EnvelopeName::new("env1")].as_ref().unwrap(),
            &FillerName::new("base_env1")
        );
        assert_eq!(
            envelopes[&EnvelopeName::new("env2")].as_ref().unwrap(),
            &FillerName::new("mid_env2")
        );
        assert_eq!(
            envelopes[&EnvelopeName::new("env3")].as_ref().unwrap(),
            &FillerName::new("top_env3")
        );
    }

    #[test]
    fn test_project_roots_inherited_from_base() {
        // Regression test: an override config without project_roots should inherit
        // the base config's resolved project_roots, not default to its own directory.
        let dir = tempdir().unwrap();
        let models_dir = dir.path().join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        fs::write(
            dir.path().join("base.yaml"),
            "project_roots: [./models]\nenvelope_structure: my_structure\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("override.yaml"),
            "overrides: ./base.yaml\nsource: my_source\n",
        )
        .unwrap();

        let config = ModelConfig::load(dir.path().join("override.yaml")).unwrap();

        // Canonicalize models_dir so it resolves symlinks (like /var -> /private/var on macOS)
        let expected_models_dir = models_dir.canonicalize().unwrap();

        // project_roots must point to the models subdirectory resolved from base.yaml,
        // not to dir itself (which would happen if override.yaml's directory were used).
        assert_eq!(config.project_roots(), &[expected_models_dir]);
    }

    #[test]
    fn test_load_config_cycle_detected() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("a.yaml"),
            "overrides: ./b.yaml\nenvelopes:\n  e: v\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("b.yaml"),
            "overrides: ./a.yaml\nenvelopes:\n  e: v\n",
        )
        .unwrap();

        let err = ModelConfig::load(dir.path().join("a.yaml")).unwrap_err();
        assert!(
            err.to_string()
                .contains("Cycle detected in configuration overrides")
        );
    }
}
