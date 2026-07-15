use crate::build_report::{BuildReport, EnvelopeEntry, FillerEntry};
use crate::project_manager::ProjectManager;
use crate::types::{EnvelopeName, FillerName, UniverseId};
use crate::utils::GitronicsError;

use git2::Repository;
use log::{info, warn};
use migjorn::Model;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::{collections::HashMap, path::Path, sync::LazyLock};

pub fn build_model(config_path: &Path, output_path: &Path) -> Result<(), GitronicsError> {
    info!(
        "Starting model build process for: {}",
        config_path.display()
    );

    let mut project_manager = ProjectManager::new(config_path, output_path)?;

    // Load all files.
    let mut envelope_structure = project_manager.load_envelope_structure()?;
    let unordered_fillers = project_manager.load_fillers()?;
    let transforms = project_manager.load_transforms()?;
    let materials = project_manager.load_materials()?;
    let tallies = project_manager.load_tallies()?;
    let source = project_manager.load_source()?;

    // Order fillers by their first cell id for deterministic output.
    let fillers = order_fillers_by_cell_id(unordered_fillers)?;

    // Load metadata for each filler and cache it in the ProjectManager.
    project_manager.load_metadata_for_fillers(fillers.iter().map(|(name, _)| name))?;

    // Universe id of every filler (read from its first cell's `u=`).
    let universe_ids: HashMap<FillerName, UniverseId> = fillers
        .iter()
        .map(|(name, model)| {
            let universe = filler_universe(model)
                .ok_or_else(|| GitronicsError::FirstCellWithoutUniverseID(name.clone()))?;
            Ok((name.clone(), universe))
        })
        .collect::<Result<HashMap<_, _>, GitronicsError>>()?;

    // Insert FILL cards into the placeholder envelope cells.
    info!("Adapting envelope structure with FILL cards");
    add_fill_cards_to_envelopes(&project_manager, &universe_ids, &mut envelope_structure)?;

    // Collect build-report data before the models are consumed by composition.
    let report = collect_build_report(
        config_path,
        &project_manager,
        &envelope_structure,
        &fillers,
        &universe_ids,
    )?;

    // Compose: drop the ignored data blocks, then merge every filler's geometry
    // into the envelope structure (collision-checked against the disjoint-range
    // convention).
    info!("Composing model");
    envelope_structure.clear_data_cards();
    let mut filler_models = fillers;
    for (_, model) in filler_models.iter_mut() {
        model.clear_data_cards();
    }
    let filler_refs: Vec<&Model> = filler_models.iter().map(|(_, model)| model).collect();
    envelope_structure
        .merge(&filler_refs)
        .map_err(|conflicts| GitronicsError::MergeConflicts(format_conflicts(&conflicts)))?;

    // Append the configured data cards to the data block.
    let data_text = [transforms, materials, tallies, source]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let mut assembled_source = envelope_structure.to_source();
    if !data_text.is_empty() {
        if !assembled_source.ends_with('\n') {
            assembled_source.push('\n');
        }
        assembled_source.push_str(&data_text);
        assembled_source.push('\n');
    }
    let assembled_model = Model::parse(assembled_source);

    // Validate the assembled model.
    info!("Performing validation checks on the assembled model");
    let problems = assembled_model.validate();
    if !problems.is_empty() {
        return Err(GitronicsError::ValidationError(problems.join("\n")));
    }

    // Write the assembled model with the provenance banner.
    info!("Writing assembled model to file");
    let assembled_path = project_manager.output_path().join("assembled.mcnp");
    let final_source = insert_banner(&assembled_model.to_source(), config_path);
    fs::write(&assembled_path, final_source)?;
    fs::write(output_path.join(".gitignore"), "*\n")?;

    // Write the HTML build report.
    info!("Writing HTML build report");
    let report_path = project_manager.output_path().join("build_report.html");
    fs::write(&report_path, report.generate_html())?;

    info!(
        "Build completed successfully in: {}",
        assembled_path.display()
    );
    Ok(())
}

static ENVELOPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\s*@env:\s*([[:alnum:]_.-]+)\s*").unwrap());

/// The directory that contains the configuration file — the anchor for git
/// repository discovery (the project, not the process's working directory).
fn project_dir(config_path: &Path) -> &Path {
    config_path.parent().unwrap_or(Path::new("."))
}

/// The universe id declared by a filler's first cell (`u=`), if any.
fn filler_universe(model: &Model) -> Option<UniverseId> {
    let first = model.cells().next()?;
    model
        .cell_universe(first.card_index)
        .map(|u| UniverseId::new(u as u32))
}

/// Order fillers by the id of their first cell, erroring on a filler with no
/// cells.
fn order_fillers_by_cell_id(
    fillers: Vec<(FillerName, Model)>,
) -> Result<Vec<(FillerName, Model)>, GitronicsError> {
    let mut keyed: Vec<(i64, FillerName, Model)> = fillers
        .into_iter()
        .map(|(name, model)| {
            let cell_id = model
                .cells()
                .next()
                .map(|c| c.id)
                .ok_or_else(|| GitronicsError::NoCellID(name.clone()))?;
            Ok((cell_id, name, model))
        })
        .collect::<Result<Vec<_>, GitronicsError>>()?;
    keyed.sort_by_key(|(cell_id, _, _)| *cell_id);
    Ok(keyed
        .into_iter()
        .map(|(_, name, model)| (name, model))
        .collect())
}

fn add_fill_cards_to_envelopes(
    project_manager: &ProjectManager,
    universe_ids: &HashMap<FillerName, UniverseId>,
    envelope_structure: &mut Model,
) -> Result<(), GitronicsError> {
    let mut missing_envelopes_in_file: HashSet<EnvelopeName> =
        project_manager.envelopes_in_config().cloned().collect();

    // Collect cell card indices up front: FILL insertion is a token splice that
    // leaves indices stable, so we can read then mutate by the same index.
    let cell_indices: Vec<usize> = envelope_structure.cells().map(|c| c.card_index).collect();

    for card_index in cell_indices {
        let original_text = envelope_structure.card_source(card_index);
        let Some(caps) = ENVELOPE_RE.captures(&original_text) else {
            continue;
        };
        let envelope_name =
            EnvelopeName::new(caps.get(1).map(|m| m.as_str()).ok_or_else(|| {
                GitronicsError::FailedToExtractEnvelopeName(
                    ENVELOPE_RE.to_string(),
                    original_text.clone(),
                )
            })?);

        let Some(env_config) = project_manager.filler_by_envelope(&envelope_name) else {
            warn!(
                "The envelope `{envelope_name}` is not considered in the configuration file. \
                 It is better to explicitly set it as `{envelope_name}: null` if you want the \
                 envelope to not be filled with any model."
            );
            continue;
        };

        // We found the envelope in the file.
        missing_envelopes_in_file.remove(&envelope_name);

        // Envelope explicitly set to null in config: leave it unfilled.
        let Some(filler_name) = env_config.as_ref() else {
            continue;
        };

        let universe_id = universe_ids
            .get(filler_name)
            .ok_or_else(|| GitronicsError::FillerUniverseIdMissing(filler_name.clone()))?;

        let transform = project_manager
            .transformation(filler_name, &envelope_name)?
            .unwrap_or("");
        let fill_card_text = if let Some(transform_without_star) = transform.strip_prefix('*') {
            format!("*fill={universe_id} {transform_without_star}")
        } else {
            format!("fill={universe_id} {transform}")
        };

        envelope_structure
            .add_cell_param(card_index, fill_card_text.trim())
            .map_err(|e| GitronicsError::InvalidFillCard(fill_card_text, e.to_string()))?;
    }

    if !missing_envelopes_in_file.is_empty() {
        warn!(
            "The following envelopes were defined in the configuration file but not found in the envelope structure file: {}. \
             Please check that the `$ @env:envelope_name` pattern is satisfied.",
            missing_envelopes_in_file
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

fn collect_build_report(
    config_path: &Path,
    project_manager: &ProjectManager,
    envelope_structure: &Model,
    fillers: &[(FillerName, Model)],
    universe_ids: &HashMap<FillerName, UniverseId>,
) -> Result<BuildReport, GitronicsError> {
    let total_cells = envelope_structure.cells().count()
        + fillers
            .iter()
            .map(|(_, m)| m.cells().count())
            .sum::<usize>();
    let total_surfaces = envelope_structure.surfaces().count()
        + fillers
            .iter()
            .map(|(_, m)| m.surfaces().count())
            .sum::<usize>();

    let envelope_entries: Vec<EnvelopeEntry> = project_manager
        .envelopes_in_config()
        .map(|env_name| {
            let filler_name: Option<FillerName> = project_manager
                .filler_by_envelope(env_name)
                .and_then(|opt| opt.clone());

            let universe_id = filler_name
                .as_ref()
                .and_then(|f| universe_ids.get(f))
                .copied();

            let transform = filler_name.as_ref().and_then(|f| {
                project_manager
                    .transformation(f, env_name)
                    .ok()
                    .flatten()
                    .map(str::to_string)
            });

            EnvelopeEntry {
                envelope_name: env_name.clone(),
                filler_name,
                universe_id,
                transform,
            }
        })
        .collect();

    let mut filler_envelope_counts: HashMap<FillerName, usize> = HashMap::new();
    for entry in &envelope_entries {
        if let Some(filler_name) = &entry.filler_name {
            *filler_envelope_counts
                .entry(filler_name.clone())
                .or_insert(0) += 1;
        }
    }

    let filler_entries: Vec<FillerEntry> = fillers
        .iter()
        .filter_map(|(name, model)| {
            let universe_id = *universe_ids.get(name)?;
            let envelope_count = *filler_envelope_counts.get(name).unwrap_or(&0);
            Some(FillerEntry {
                universe_id,
                envelope_count,
                cell_count: model.cells().count(),
                surface_count: model.surfaces().count(),
                name: name.clone(),
            })
        })
        .collect();

    Ok(BuildReport {
        config_path: config_path.display().to_string(),
        gitronics_version: env!("CARGO_PKG_VERSION"),
        commit_hash: get_hash_of_project(project_dir(config_path)),
        date_time: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        total_cells,
        total_surfaces,
        envelope_entries,
        filler_entries,
        materials: project_manager
            .materials_names()
            .iter()
            .map(|n| n.to_string())
            .collect(),
        tallies: project_manager
            .tallies_names()
            .iter()
            .map(|n| n.to_string())
            .collect(),
        transforms: project_manager
            .transforms_names()
            .iter()
            .map(|n| n.to_string())
            .collect(),
        source: project_manager.source_name().map(|n| n.to_string()),
    })
}

/// Render merge conflicts as a human-readable, newline-separated list.
fn format_conflicts(conflicts: &[migjorn::MergeConflict]) -> String {
    conflicts
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Insert the provenance banner as comment lines just after the model's title.
fn insert_banner(source: &str, config_path: &Path) -> String {
    let configuration = config_path.display().to_string();
    let date_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let gitronics_version = env!("CARGO_PKG_VERSION");
    let commit_hash = get_hash_of_project(project_dir(config_path));

    let banner = format!(
        "C ============================================================\n\
         C  Built by gitronics v{gitronics_version}\n\
         C  Configuration : {configuration}\n\
         C  Git commit    : {commit_hash}\n\
         C  Date / time   : {date_time}\n\
         C ============================================================"
    );

    match source.split_once('\n') {
        Some((title, rest)) => format!("{title}\n{banner}\n{rest}"),
        None => format!("{source}\n{banner}\n"),
    }
}

/// Describe the git state of the repository that contains `start_dir` (the
/// project being built), not the process's current working directory — so the
/// recorded commit reflects what was actually assembled regardless of where the
/// binary was invoked from.
fn get_hash_of_project(start_dir: &Path) -> String {
    let repo = Repository::discover(start_dir).ok();
    repo.as_ref()
        .and_then(|r| {
            let mut opts = git2::DescribeOptions::new();
            opts.describe_tags(); // Look for tags

            // Configure formatting options (this adds the -dirty suffix automatically!)
            let mut format_opts = git2::DescribeFormatOptions::new();
            format_opts.dirty_suffix("-dirty");

            // Try to describe the current state, fallback to a short hash if no tags exist
            r.describe(&opts)
                .and_then(|format| format.format(Some(&format_opts)))
                .ok()
                .or_else(|| {
                    // Fallback: If the repo has no tags at all, just grab the short SHA
                    let head = r.head().ok()?;
                    let commit = head.peel_to_commit().ok()?;
                    let short_id = commit.as_object().short_id().ok()?;
                    let mut hash = short_id.as_str().map(|s| s.to_string()).unwrap_or_default();

                    // Manually check dirty state for fallback
                    if let Ok(statuses) = r.statuses(None)
                        && !statuses.is_empty()
                    {
                        hash.push_str("-dirty");
                    }
                    Some(hash)
                })
        })
        .unwrap_or_else(|| "GIT repository not found".to_string())
}
