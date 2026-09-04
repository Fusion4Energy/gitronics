use crate::{
    build_model,
    error::GitronicsError,
    fs_utils::write_output_gitignore,
    mcnp_io::parse_model_file,
    model_config::ModelConfig,
    runtime::init_thread_pool,
    types::{EnvelopeName, FileName, FillerMetadata, FillerName},
};
use indexmap::IndexMap;
use log::info;
use migjorn::Model;
use rayon::prelude::*;
use std::{
    fs::{self, File, create_dir_all},
    io::Write,
    path::Path,
};

pub fn migrate_model(mcnp_input: &Path, output_path: &Path) -> Result<(), GitronicsError> {
    init_thread_pool();
    info!("Reading MCNP model from file: {}", mcnp_input.display());
    let file_name = FileName::new(mcnp_input.display().to_string());
    let model = parse_model_file(mcnp_input, &file_name)?;

    // Create the output directory tree.
    let filler_models_dir = output_path.join("reference_model/filler_models");
    create_dir_all(&filler_models_dir)
        .map_err(|source| GitronicsError::io_path(&filler_models_dir, source))?;
    let configurations_dir = output_path.join("configurations");
    create_dir_all(&configurations_dir)
        .map_err(|source| GitronicsError::io_path(&configurations_dir, source))?;
    let output_dir = output_path.join("output");
    create_dir_all(&output_dir).map_err(|source| GitronicsError::io_path(&output_dir, source))?;
    write_output_gitignore(&output_dir)?;

    // Extract every universe into its own filler model file.
    info!("Extracting universes");
    model
        .universe_ids()
        .par_iter()
        .try_for_each(|&universe_id| {
            let extracted = model.extract_universe(universe_id);
            let universe_path = output_path.join(format!(
                "reference_model/filler_models/universe_{universe_id}.mcnp"
            ));
            fs::write(&universe_path, extracted.to_source())
                .map_err(|source| GitronicsError::io_path(&universe_path, source))
        })?;

    // Extract the level-0 shell and turn its FILL cards into `@env` placeholders.
    info!("Extracting envelope structure");
    let mut envelope_structure = model.extract_level0();
    let (fillers_metadata, envelopes_metadata) =
        replace_fills_with_placeholders(&mut envelope_structure)?;

    info!("Writing envelope structure to file");
    let envelope_path = output_path.join("reference_model/envelope_structure.mcnp");
    fs::write(&envelope_path, envelope_structure.to_source())
        .map_err(|source| GitronicsError::io_path(&envelope_path, source))?;

    // Write the baseline configuration and per-filler metadata files.
    write_baseline_config(output_path, envelopes_metadata)?;
    write_metadata_files(output_path, fillers_metadata)?;

    // Write every data card of the original model to a single data-cards file.
    let data_cards_file = output_path.join("reference_model/data_cards.source");
    let mut writer = File::create(&data_cards_file)
        .map_err(|source| GitronicsError::io_path(&data_cards_file, source))?;
    writer.write_all(b"All the data cards of the original model\n")?;
    for card in model.data_cards() {
        writeln!(writer, "{}", card.text().trim_end())?;
    }

    // Assemble the migrated project to validate the migration round-trips.
    info!("Test assembling the model for the first time to validate the migration");
    let previous_level = log::max_level();
    log::set_max_level(log::LevelFilter::Warn);
    let build_result = build_model(
        &output_path.join("configurations/baseline.yaml"),
        &output_path.join("output"),
    )
    .and_then(|()| {
        let assembled = output_path.join("output/assembled.mcnp");
        parse_model_file(&assembled, &FileName::new(assembled.display().to_string())).map(|_| ())
    });
    log::set_max_level(previous_level);
    build_result?;

    info!("Model migration completed successfully");
    Ok(())
}

fn write_baseline_config(
    output_path: &Path,
    envelopes_metadata: IndexMap<EnvelopeName, Option<FillerName>>,
) -> Result<(), GitronicsError> {
    let mut baseline_config = ModelConfig::new(
        Some(FileName::new("envelope_structure")),
        envelopes_metadata,
    );
    baseline_config.set_default_project_root(Path::new("../reference_model"));
    baseline_config.set_source(FileName::new("data_cards"));
    let config_path = output_path.join("configurations/baseline.yaml");
    let yaml_content = serde_saphyr::to_string(&baseline_config).map_err(|source| {
        GitronicsError::YamlSerialize {
            path: config_path.display().to_string(),
            source,
        }
    })?;
    fs::write(&config_path, yaml_content)
        .map_err(|source| GitronicsError::io_path(&config_path, source))?;
    Ok(())
}

fn write_metadata_files(
    output_path: &Path,
    fillers_metadata: IndexMap<FillerName, FillerMetadata>,
) -> Result<(), GitronicsError> {
    for (filler_name, metadata) in fillers_metadata {
        let metadata_path = output_path
            .join("reference_model/filler_models")
            .join(format!("{filler_name}.metadata"));
        let yaml_content =
            serde_saphyr::to_string(&metadata).map_err(|source| GitronicsError::YamlSerialize {
                path: metadata_path.display().to_string(),
                source,
            })?;
        fs::write(&metadata_path, yaml_content)
            .map_err(|source| GitronicsError::io_path(&metadata_path, source))?;
    }
    Ok(())
}

type InfoForMetadata = (
    IndexMap<FillerName, FillerMetadata>,
    IndexMap<EnvelopeName, Option<FillerName>>,
);

/// For each level-0 cell that has a `fill=`, record the (envelope, filler,
/// transform) placement, strip the `fill=` parameter, and append a
/// `$ @env:envelope_<cell_id>` placeholder comment.
fn replace_fills_with_placeholders(
    envelope_structure: &mut Model,
) -> Result<InfoForMetadata, GitronicsError> {
    let mut envelopes_metadata: IndexMap<EnvelopeName, Option<FillerName>> = IndexMap::new();
    let mut fillers_metadata: IndexMap<FillerName, FillerMetadata> = IndexMap::new();

    // Snapshot the cells and their fills first; edits (parameter removal + comment
    // insertion) are token splices that keep slots stable.
    let placements: Vec<(u32, i64, migjorn::Fill)> = envelope_structure
        .cells()
        .filter_map(|cell| {
            let fill = cell.fill()?;
            Some((cell.slot(), cell.id().unwrap_or_default(), fill))
        })
        .collect();

    for (slot, cell_id, fill) in placements {
        let filler_name = FillerName::new(format!("universe_{}", fill.universe));
        let envelope_name = EnvelopeName::new(format!("envelope_{cell_id}"));
        // `Fill::transform` is the parenthesised transform exactly as written —
        // `(30)`, not `30` — so it must not be wrapped in parentheses again.
        // A starred fill is recorded as `*(30)`, which `build` turns back into
        // `*fill=<universe> (30)`.
        let transform = fill.transform.map(|inner| {
            if fill.starred {
                format!("*{inner}")
            } else {
                inner
            }
        });

        envelopes_metadata.insert(envelope_name.clone(), Some(filler_name.clone()));
        fillers_metadata
            .entry(filler_name)
            .or_insert_with(|| FillerMetadata {
                transformations: Some(IndexMap::new()),
            })
            .transformations
            .get_or_insert_with(IndexMap::new)
            .insert(envelope_name, transform);

        envelope_structure
            .remove_cell_param(slot, "fill")
            .map_err(|e| {
                GitronicsError::ValidationError(format!("Could not remove FILL card: {e}"))
            })?;
        envelope_structure
            .append_cell_comment(slot, &format!("@env:envelope_{cell_id}"))
            .map_err(|e| {
                GitronicsError::ValidationError(format!("Could not add envelope placeholder: {e}"))
            })?;
    }

    Ok((fillers_metadata, envelopes_metadata))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_migrate_model() {
        let mcnp_input = PathBuf::from("resources/simple_model.mcnp");
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("output.mcnp");
        let result = migrate_model(&mcnp_input, &output_path);
        assert!(result.is_ok(), "migration failed: {:?}", result.err());
    }
}
