use std::{
    collections::HashSet,
    env::current_dir,
    path::{Path, PathBuf},
};

use super::ModelConfig;
use crate::utils::GitronicsError;

/// Loads a model configuration by path.
///
/// If the configuration specifies an `overrides` field, recursively loads
/// the base configuration and merges them. Detects and prevents circular
/// override chains.
pub fn load_config<P: AsRef<std::path::Path>>(
    config_path: P,
) -> Result<ModelConfig, GitronicsError> {
    let config_path = if config_path.as_ref().is_absolute() {
        config_path.as_ref().to_path_buf()
    } else {
        current_dir()?.join(config_path)
    };
    load_config_inner(&config_path, &mut HashSet::new())
}

fn load_config_inner(
    config_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<ModelConfig, GitronicsError> {
    let config_path = config_path.to_path_buf();
    if !visited.insert(config_path.clone()) {
        return Err(GitronicsError::ConfigCycle(config_path));
    }
    let mut config = ModelConfig::from_file(&config_path)?;
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
    let base_path = base_path
        .canonicalize()
        .map_err(|source| GitronicsError::io_path(&base_path, source))?;
    // Resolve the base config recursively so the full chain is applied.
    let base = load_config_inner(&base_path, visited)?;
    let merged = config.merge(base);
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::load_config;
    use crate::types::{EnvelopeName, FileName, FillerName};
    use std::fs;
    use tempfile::tempdir;
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

        // Create minimal metadata files for all referenced MCNP/model files
        for name in [
            "base_structure",
            "base_env1",
            "mid_source",
            "mid_env2",
            "mid",
            "top_env3",
            "top",
            "base",
        ] {
            fs::write(dir.path().join(format!("{}.metadata", name)), "{}").unwrap();
        }
        let pm =
            crate::project_manager::ProjectManager::new(dir.path().join("top.yaml"), dir.path())
                .unwrap();
        let config = &pm.model_config;
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

        let config = load_config(dir.path().join("override.yaml")).unwrap();

        // project_roots must point to the models subdirectory resolved from base.yaml,
        // not to dir itself (which would happen if override.yaml's directory were used).
        assert_eq!(config.project_roots(), &[models_dir]);
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

        let err = crate::project_manager::load_model_config::load_config(dir.path().join("a.yaml"))
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("Cycle detected in configuration overrides")
        );
    }
}
