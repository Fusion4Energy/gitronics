//! The `inspect` command: scan a whole gitronics project and emit a project-wide
//! report (`project_report.json` + `project_report.html`).
//!
//! Unlike `build`, this is not tied to a single configuration. It gathers the
//! full filler *library* (including fillers no configuration uses), the full
//! envelope *inventory* (including unassigned envelopes), and every
//! configuration's assignment and coverage — the data the composition dashboard
//! visualises.
//!
//! The orchestrator below reads as a table of contents: discover → scan →
//! collect → render → write. All gathering lives in dedicated helpers that
//! return plain data; the manifest + HTML rendering live in
//! [`crate::project_report`].

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use log::info;
use serde_json::Value;
use walkdir::WalkDir;

use crate::build_model::{envelope_names_in_structure, filler_universe};
use crate::build_report::model_stats;
use crate::model_config::ModelConfig;
use crate::project_manager::{ProjectManager, load_config};
use crate::project_report::{
    ConfigEntry, ConfigEnvelope, ConfigStats, EnvelopeInv, FillerLib, MetadataKeys, ProjectReport,
    SCHEMA_VERSION,
};
use crate::types::{EnvelopeName, FillerName, UniverseId};
use crate::utils::{GitronicsError, get_hash_of_project};

/// Scans `project_dir` and writes `project_report.{json,html}` to `output_path`.
pub fn inspect_project(project_dir: &Path, output_path: &Path) -> Result<(), GitronicsError> {
    info!("Inspecting project: {}", project_dir.display());
    create_dir_all(output_path)?;

    // Discover every configuration in the project.
    let configs = discover_configurations(project_dir);
    if configs.is_empty() {
        return Err(GitronicsError::NoConfigurationsFound(
            project_dir.display().to_string(),
        ));
    }
    info!("Found {} configuration(s)", configs.len());

    // The primary configuration provides the roots + envelope structure used to
    // enumerate the library and inventory (config-independent facts).
    let primary = pick_primary(&configs);
    let mut pm = ProjectManager::new(&configs[primary].path, output_path)?;

    // Envelope inventory (every `$ @env:` placeholder, assigned or not).
    let mut structure = pm.load_envelope_structure()?;
    let envelope_names = envelope_names_in_structure(&mut structure);
    pm.load_envelope_metadata();

    // Filler library (every `.mcnp` except the structure), parsed once.
    let mut library = pm.load_library_fillers()?;
    pm.load_metadata_for_fillers_best_effort(library.iter().map(|(n, _)| n));

    // Universe id of every library filler; files whose first cell has no `u=`
    // are not filler universes and are dropped from the library.
    let mut universe_of: HashMap<FillerName, UniverseId> = HashMap::new();
    for (name, model) in library.iter_mut() {
        if let Some(u) = filler_universe(model) {
            universe_of.insert(name.clone(), u);
        }
    }

    // Per-configuration assignment + coverage, and the fillers each config uses.
    let mut configurations: Vec<ConfigEntry> = Vec::new();
    let mut used_by: HashMap<FillerName, Vec<String>> = HashMap::new();
    for c in &configs {
        let (entry, used) =
            collect_config(c, project_dir, &envelope_names, &universe_of, &pm);
        for filler in used {
            used_by.entry(filler).or_default().push(c.name.clone());
        }
        configurations.push(entry);
    }

    // Assemble the manifest.
    let report = ProjectReport {
        schema_version: SCHEMA_VERSION,
        gitronics_version: env!("CARGO_PKG_VERSION"),
        commit_hash: get_hash_of_project(project_dir),
        date_time: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        project_dir: project_dir.display().to_string(),
        filler_library: collect_library(&mut library, &universe_of, &used_by, &pm),
        envelope_inventory: collect_inventory(&envelope_names, &pm),
        metadata_keys: collect_metadata_keys(&library, &envelope_names, &pm),
        configurations,
    };

    report.write(output_path)?;
    info!(
        "Project report written to {}",
        output_path.join("project_report.html").display()
    );
    Ok(())
}

// ─── Discovery ────────────────────────────────────────────────────────────────

/// A configuration discovered in the project.
struct DiscoveredConfig {
    name: String,
    path: PathBuf,
    /// The effective (override-merged) configuration.
    config: ModelConfig,
    /// The raw `overrides:` path this config declares, if any.
    overrides: Option<String>,
}

/// Walks the project for YAML files that parse as gitronics configurations. A
/// file that is not a valid configuration (e.g. some other YAML) is skipped.
fn discover_configurations(project_dir: &Path) -> Vec<DiscoveredConfig> {
    let mut found: Vec<DiscoveredConfig> = WalkDir::new(project_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            matches!(
                e.path().extension().and_then(|x| x.to_str()),
                Some("yaml") | Some("yml")
            )
        })
        .filter_map(|e| {
            let path = e.path();
            let config = load_config(path).ok()?;
            // A real configuration resolves an envelope structure (directly or
            // via inheritance).
            config.envelope_structure()?;
            let overrides = ModelConfig::from_file(path)
                .ok()
                .and_then(|c| c.overrides().map(|p| p.display().to_string()));
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("config")
                .to_string();
            Some(DiscoveredConfig {
                name,
                path: path.to_path_buf(),
                config,
                overrides,
            })
        })
        .collect();
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

/// Picks the primary configuration whose roots/structure seed the library and
/// inventory: a config literally named `baseline`, else the fullest one (most
/// envelopes). The library scan needs roots that cover the whole project.
fn pick_primary(configs: &[DiscoveredConfig]) -> usize {
    if let Some(i) = configs.iter().position(|c| c.name == "baseline") {
        return i;
    }
    configs
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| c.config.envelopes().len())
        .map(|(i, _)| i)
        .unwrap_or(0)
}

// ─── Collection ───────────────────────────────────────────────────────────────

/// Builds a configuration's per-envelope assignment and coverage stats, and
/// returns the set of fillers it places.
fn collect_config(
    c: &DiscoveredConfig,
    project_dir: &Path,
    envelope_names: &[EnvelopeName],
    universe_of: &HashMap<FillerName, UniverseId>,
    pm: &ProjectManager,
) -> (ConfigEntry, HashSet<FillerName>) {
    let mut envelopes = Vec::with_capacity(envelope_names.len());
    let mut used: HashSet<FillerName> = HashSet::new();

    for env in envelope_names {
        let filler = c.config.envelopes().get(env).and_then(|o| o.clone());
        if let Some(f) = &filler {
            used.insert(f.clone());
        }
        let universe_id = filler.as_ref().and_then(|f| universe_of.get(f).copied());
        let transform = filler.as_ref().and_then(|f| {
            pm.transformation(f, env).ok().flatten().map(str::to_string)
        });
        envelopes.push(ConfigEnvelope {
            envelope_name: env.clone(),
            filler_name: filler,
            universe_id,
            transform,
        });
    }

    // `filled` counts envelopes that received a filler (a filler may fill several).
    let filled = envelopes.iter().filter(|e| e.filler_name.is_some()).count();
    let unfilled = envelope_names.len() - filled;
    let unused_fillers = universe_of.keys().filter(|f| !used.contains(*f)).count();

    let stats = ConfigStats {
        filled,
        unfilled,
        distinct_fillers: used.len(),
        unused_fillers,
    };

    let path = c
        .path
        .strip_prefix(project_dir)
        .unwrap_or(&c.path)
        .display()
        .to_string();

    (
        ConfigEntry {
            name: c.name.clone(),
            path,
            overrides: c.overrides.clone(),
            envelopes,
            stats,
        },
        used,
    )
}

/// Builds the filler-library entries (only files that are real universes).
fn collect_library(
    library: &mut [(FillerName, migjorn::Model)],
    universe_of: &HashMap<FillerName, UniverseId>,
    used_by: &HashMap<FillerName, Vec<String>>,
    pm: &ProjectManager,
) -> Vec<FillerLib> {
    let mut entries: Vec<FillerLib> = library
        .iter_mut()
        .filter_map(|(name, model)| {
            let universe_id = *universe_of.get(name)?;
            let stats = model_stats(model);
            let (description, metadata) = split_description(
                pm.filler_metadata(name).cloned().unwrap_or_default(),
            );
            Some(FillerLib {
                universe_id,
                cell_count: stats.cell_count,
                surface_count: stats.surface_count,
                materials: stats.materials,
                cell_id_runs: stats.cell_id_runs,
                surface_id_runs: stats.surface_id_runs,
                description,
                metadata,
                used_by_configs: used_by.get(name).cloned().unwrap_or_default(),
                name: name.clone(),
            })
        })
        .collect();
    entries.sort_by(|a, b| a.universe_id.cmp(&b.universe_id));
    entries
}

/// Builds the envelope inventory with any descriptive metadata.
fn collect_inventory(
    envelope_names: &[EnvelopeName],
    pm: &ProjectManager,
) -> Vec<EnvelopeInv> {
    envelope_names
        .iter()
        .map(|env| {
            let (description, metadata) = split_description(
                pm.envelope_metadata(env).cloned().unwrap_or_default(),
            );
            EnvelopeInv {
                envelope_name: env.clone(),
                description,
                metadata,
            }
        })
        .collect()
}

/// Collects the distinct metadata keys present across the project (sorted,
/// `description` excluded — it is first-class), for the viewer's facet controls.
fn collect_metadata_keys(
    library: &[(FillerName, migjorn::Model)],
    envelope_names: &[EnvelopeName],
    pm: &ProjectManager,
) -> MetadataKeys {
    let mut filler: BTreeSet<String> = BTreeSet::new();
    for (name, _) in library {
        if let Some(meta) = pm.filler_metadata(name) {
            filler.extend(meta.keys().filter(|k| *k != "description").cloned());
        }
    }
    let mut envelope: BTreeSet<String> = BTreeSet::new();
    for env in envelope_names {
        if let Some(meta) = pm.envelope_metadata(env) {
            envelope.extend(meta.keys().filter(|k| *k != "description").cloned());
        }
    }
    MetadataKeys {
        filler: filler.into_iter().collect(),
        envelope: envelope.into_iter().collect(),
    }
}

/// Splits the recognized first-class `description` out of a metadata map,
/// returning `(description, remaining_metadata)`.
fn split_description(
    mut metadata: crate::project_manager::Metadata,
) -> (Option<String>, crate::project_manager::Metadata) {
    let description = metadata.shift_remove("description").and_then(value_to_string);
    (description, metadata)
}

/// Renders a JSON metadata value as a display string (`null` → `None`).
fn value_to_string(v: Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => Some(s),
        other => Some(other.to_string()),
    }
}
