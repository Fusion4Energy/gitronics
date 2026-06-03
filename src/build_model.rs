use crate::project_manager::ProjectManager;
use crate::types::{CellId, EnvelopeName, FillerName, UniverseId};
use crate::utils::GitronicsError;

use git2::Repository;
use log::{info, warn};
use migjorn::{Card, CellCard, CellParam, DataCard, Model, ParamType};
use regex::Regex;
use std::{collections::HashMap, path::Path, sync::LazyLock};
use std::{env, fs};

pub fn build_model(config_path: &Path, output_path: &Path) -> Result<(), GitronicsError> {
    info!(
        "Starting model build process for: {}",
        config_path.display()
    );

    let mut project_manager = ProjectManager::new(config_path, output_path)?;

    // Load all files
    let mut envelope_structure = project_manager.load_envelope_structure()?;
    let unordered_fillers = project_manager.load_fillers()?;
    let unordered_transforms = project_manager.load_transforms()?;
    let unordered_materials = project_manager.load_materials()?;
    let unordered_tallies = project_manager.load_tallies()?;
    let source = project_manager.load_source()?;

    // Reorder by IDs
    let fillers = order_models_by_cell_id(unordered_fillers)?;
    let transforms = order_data_cards_groups_by_id(unordered_transforms);
    let materials = order_data_cards_groups_by_id(unordered_materials);
    let tallies = order_data_cards_groups_by_id(unordered_tallies);

    // Load metadata for each filler and cache it in the ProjectManager
    project_manager.load_metadata_for_fillers(&fillers)?;

    // Get the filler universe ID of every filler model
    let universe_ids: HashMap<FillerName, UniverseId> = fillers
        .iter()
        .map(|filler| Ok((FillerName::from(filler), get_universe_id_of_model(filler)?)))
        .collect::<Result<HashMap<_, _>, GitronicsError>>()?;

    // Adapt envelope cells with the FILL cards
    info!("Adapting envelope structure with FILL cards");
    add_fill_cards_to_envelopes(&project_manager, &universe_ids, &mut envelope_structure)?;

    // Compose model
    info!("Composing model");
    fillers.into_iter().for_each(|filler| {
        envelope_structure.cells.extend(filler.cells);
        envelope_structure.surfaces.extend(filler.surfaces);
    });
    envelope_structure.data_cards.extend(transforms);
    envelope_structure.data_cards.extend(materials);
    envelope_structure.data_cards.extend(tallies);
    envelope_structure.data_cards.extend(source);

    // Perform validation checks
    info!("Performing validation checks on the assembled model");
    envelope_structure
        .validation_checks()
        .map_err(|e| GitronicsError::ValidationError(e.to_string()))?;

    // Write model
    let assembled_path = project_manager.output_path().join("assembled.mcnp");
    write_assembled_header(&mut envelope_structure, config_path)?;
    envelope_structure.write_to_file(&assembled_path)?;
    fs::write(output_path.join(".gitignore"), "*\n")?;

    info!(
        "Build completed successfully in: {}",
        assembled_path.display()
    );
    Ok(())
}

static ENVELOPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\s*@env:\s*(\w+)\s*").unwrap());

fn add_fill_cards_to_envelopes(
    project_manager: &ProjectManager,
    universe_ids: &HashMap<FillerName, UniverseId>,
    envelope_structure: &mut Model,
) -> Result<(), GitronicsError> {
    for cell in envelope_structure.cells.iter_mut() {
        let original_text = cell.original_text();
        let Some(caps) = ENVELOPE_RE.captures(original_text) else {
            continue;
        };

        let envelope_name =
            EnvelopeName::new(caps.get(1).map(|m| m.as_str()).ok_or_else(|| {
                GitronicsError::FailedToExtractEnvelopeName(
                    ENVELOPE_RE.to_string(),
                    original_text.to_string(),
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

        // Envelope explicitly set to null in config, so we skip it
        let Some(filler_name) = env_config.as_ref() else {
            continue;
        };

        let universe_id = universe_ids
            .get(filler_name)
            .ok_or_else(|| GitronicsError::FillerUniverseIdMissing(filler_name.clone()))?;

        let transform = project_manager.transformation(filler_name, &envelope_name)?;
        let transform = transform.unwrap_or("");

        let fill_card_text = if let Some(transform_without_star) = transform.strip_prefix('*') {
            format!("*FILL={} {}", universe_id, transform_without_star)
        } else {
            format!("FILL={} {}", universe_id, transform)
        };

        cell.insert_param(
            cell.params().len(),
            CellParam::try_from(fill_card_text.clone())
                .map_err(|e| GitronicsError::InvalidFillCard(fill_card_text, e.to_string()))?,
        );
    }
    Ok(())
}

fn order_models_by_cell_id(models: Vec<Model>) -> Result<Vec<Model>, GitronicsError> {
    let mut decorated: Vec<(CellId, Model)> = models
        .into_iter()
        .map(|filler| {
            let cell_id = filler
                .cells
                .first()
                .map(|cell| CellId::new(cell.cell_id()))
                .ok_or_else(|| GitronicsError::NoCellID(FillerName::from(&filler)))?;
            Ok((cell_id, filler))
        })
        .collect::<Result<Vec<_>, GitronicsError>>()?;
    decorated.sort_by_key(|(cell_id, _)| *cell_id);
    Ok(decorated.into_iter().map(|(_, filler)| filler).collect())
}

fn order_data_cards_groups_by_id(mut data_cards_groups: Vec<Vec<DataCard>>) -> Vec<DataCard> {
    data_cards_groups.sort_by_key(|data_cards| data_cards.first().map(|card| card.card_id()));
    data_cards_groups.into_iter().flatten().collect()
}

fn get_universe_id_of_model(model: &Model) -> Result<UniverseId, GitronicsError> {
    model
        .cells
        .first()
        .and_then(|cell| {
            cell.params()
                .iter()
                .find_map(|param| match param.param_type {
                    ParamType::U(id) => Some(UniverseId::new(id)),
                    _ => None,
                })
        })
        .ok_or_else(|| GitronicsError::FirstCellWithoutUniverseID(FillerName::from(model)))
}

fn write_assembled_header(
    assembled_model: &mut Model,
    config_path: &Path,
) -> Result<(), GitronicsError> {
    let configuration = config_path.display().to_string();
    let date_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let gitronics_version = env!("CARGO_PKG_VERSION");
    let commit_hash = get_hash_of_project();

    let banner = format!(
        "C ============================================================
C  Built by gitronics v{gitronics_version}
C  Configuration : {configuration}
C  Git commit    : {commit_hash}
C  Date / time   : {date_time}
C ============================================================"
    );
    let new_card_text = format!(
        "{banner}\n{}",
        assembled_model
            .cells
            .first()
            .map(|c| c.original_text())
            .unwrap_or_default()
    );

    let banner_card = CellCard::try_from(new_card_text.as_str())
        .expect("Adding comments as header should not fail");

    // Replace the first cell (previously a comment placeholder) with the banner card
    assembled_model.cells[0] = banner_card;
    Ok(())
}

fn get_hash_of_project() -> String {
    let repo = env::current_dir()
        .ok()
        .and_then(|cwd| Repository::discover(cwd).ok());
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
