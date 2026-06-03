use crate::{
    build_model,
    model_config::ModelConfig,
    types::{EnvelopeName, FileName, FillerMetadata, FillerName},
    utils::GitronicsError,
};
use indexmap::IndexMap;
use log::info;
use migjorn::{Card, CellCard, GeoElement, Model, ParamType};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    fs::{self, File, create_dir_all},
    io::Write,
    path::Path,
};

pub fn migrate_model(mcnp_input: &Path, output_path: &Path) -> Result<(), GitronicsError> {
    info!("Reading MCNP model from file: {}", mcnp_input.display());
    let model =
        Model::from_file(mcnp_input).map_err(|err| GitronicsError::FailedToLoadMCNPFile {
            file_name: mcnp_input.display().to_string().into(),
            error: err.to_string(),
        })?;

    // Create the output directory and subdirectory
    create_dir_all(output_path.join("reference_model/filler_models"))?;
    create_dir_all(output_path.join("configurations"))?;
    create_dir_all(output_path.join("output"))?;
    // Add a .gitignore file to ignore the output directory
    fs::write(output_path.join("output/.gitignore"), "*\n")?;

    info!("Extracting universes");
    model
        .universe_ids()
        .par_iter()
        .try_for_each(|&universe_id| {
            let extracted_universe = model.extract_universe(universe_id);
            let universe_path = output_path.join(format!(
                "reference_model/filler_models/universe_{}.mcnp",
                universe_id
            ));
            extracted_universe.write_to_file(&universe_path)?;
            Ok::<(), GitronicsError>(())
        })?;

    info!("Extracting envelope structure");
    let mut envelope_structure = extract_envelope_structure(&model);

    // Modify envelopes adding the @ placeholder
    let (fillers_metadata, envelopes_metadata) =
        replace_fills_with_placeholders(&mut envelope_structure)?;

    info!("Writing envelope structure to file");
    envelope_structure
        .write_to_file(output_path.join("reference_model/envelope_structure.mcnp"))?;

    // Write baseline configuration file
    write_baseline_config(output_path, envelopes_metadata)?;

    // Write the metadata files
    write_metadata_files(output_path, fillers_metadata)?;

    // Write the data cards files
    let data_cards_file = output_path.join("reference_model/data_cards.source");
    let mut writer = File::create(&data_cards_file)?;
    writer.write_all(b"All the data cards of the original model\n")?;
    for card in &model.data_cards {
        card.write_into(&mut writer)?;
    }

    // Assemble the model with the new configuration
    info!("Test assembling the model for the first time to validate the migration");
    log::set_max_level(log::LevelFilter::Warn);
    build_model(
        &output_path.join("configurations/baseline.yaml"),
        &output_path.join("output"),
    )?;
    Model::from_file(output_path.join("output/assembled.mcnp")).map_err(|err| {
        GitronicsError::FailedToLoadMCNPFile {
            file_name: output_path
                .join("output/assembled.mcnp")
                .display()
                .to_string()
                .into(),
            error: err.to_string(),
        }
    })?;
    log::set_max_level(log::LevelFilter::Info);

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
    let yaml_content = serde_saphyr::to_string(&baseline_config).map_err(|e| {
        GitronicsError::YamlSerialize(config_path.display().to_string(), e.to_string())
    })?;
    fs::write(&config_path, yaml_content)?;
    Ok(())
}

fn write_metadata_files(
    output_path: &Path,
    fillers_metadata: IndexMap<FillerName, FillerMetadata>,
) -> Result<(), GitronicsError> {
    for (filler_name, metadata) in fillers_metadata {
        let metadata_path = output_path
            .join("reference_model/filler_models")
            .join(format!("{}.metadata", filler_name));
        let yaml_content = serde_saphyr::to_string(&metadata).map_err(|e| {
            GitronicsError::YamlSerialize(metadata_path.display().to_string(), e.to_string())
        })?;
        fs::write(&metadata_path, yaml_content)?;
    }
    Ok(())
}

fn extract_envelope_structure(model: &Model) -> Model {
    let mut env_struct = model.clone();
    env_struct.cells = env_struct
        .cells
        .iter()
        .filter(|cell| is_level_0(cell))
        .cloned()
        .collect();

    let surfaces_to_keep: HashSet<u32> = env_struct
        .cells
        .iter()
        .flat_map(|cell| {
            cell.geometry().filter_map(|geo| match geo {
                GeoElement::Surface(surface_id) => Some(surface_id.unsigned_abs()),
                _ => None,
            })
        })
        .collect();
    env_struct.surfaces = env_struct
        .surfaces
        .iter()
        .filter(|surface| surfaces_to_keep.contains(&surface.surface_id()))
        .cloned()
        .collect();
    env_struct
}

fn is_level_0(cell: &CellCard) -> bool {
    cell.get_universe().is_none()
}

type InfoForMetadata = (
    IndexMap<FillerName, FillerMetadata>,
    IndexMap<EnvelopeName, Option<FillerName>>,
);

fn replace_fills_with_placeholders(
    envelope_structure: &mut Model,
) -> Result<InfoForMetadata, GitronicsError> {
    let mut envelopes_metadata: IndexMap<EnvelopeName, Option<FillerName>> = IndexMap::new();
    let mut fillers_metadata: IndexMap<FillerName, FillerMetadata> = IndexMap::new();
    for cell in envelope_structure.cells.iter_mut() {
        if is_level_0(cell)
            && let Some(fill_data) = cell.get_fill()
        {
            let filler_name = FillerName::new(format!("universe_{}", fill_data.universe));
            let envelope_name = EnvelopeName::new(format!("envelope_{}", cell.cell_id()));
            let star = if fill_data.starred { "*" } else { "" };
            let transform: Option<String> = match (fill_data.transform, &fill_data.coeffs) {
                (Some(tr), _) => Some(format!("{star}({tr})")),
                (None, None) => None,
                (None, Some(coeffs)) => {
                    let coeffs_str = coeffs
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    Some(format!("{star}({coeffs_str})"))
                }
            };

            envelopes_metadata.insert(envelope_name.clone(), Some(filler_name.clone()));
            fillers_metadata
                .entry(filler_name)
                .or_insert_with(|| FillerMetadata {
                    transformations: Some(IndexMap::new()),
                })
                .transformations
                .get_or_insert_with(IndexMap::new)
                .insert(envelope_name, transform);

            let fill_index = cell
                .params()
                .iter()
                .position(|p| matches!(p.param_type, ParamType::Fill(_)))
                .expect("fill param must exist since get_fill() returned Some");
            cell.remove_param(fill_index);

            let modified_text = format!(
                "{}           $ @env:envelope_{} \n",
                cell.updated_text(),
                cell.cell_id()
            );
            *cell = CellCard::try_from(modified_text.as_str()).map_err(|err| {
                GitronicsError::ValidationError(format!("Could not adapt envelope cell: {err}"))
            })?;
        }
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
        dbg!(&result);
        assert!(result.is_ok());
    }
}
