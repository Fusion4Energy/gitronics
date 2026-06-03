use gitronics::build_model;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_dir(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[test]
fn test_build_example_project() {
    let output_dir = tempdir().unwrap();
    let config_path = PathBuf::from("example_project/configurations/valid_configuration.yaml");

    let result = build_model(&config_path, output_dir.path());

    assert!(
        result.is_ok(),
        "Model building failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_build_example_override() {
    let output_dir = tempdir().unwrap();
    let config_path = PathBuf::from("example_project/assessment_specific/small_override.yaml");

    let result = build_model(&config_path, output_dir.path());

    assert!(
        result.is_ok(),
        "Model building failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_build_nonexistent_config() {
    let output_dir = tempdir().unwrap();
    let config_path = PathBuf::from("example_project/configurations/does_not_exist.yaml");

    let result = build_model(&config_path, output_dir.path());

    assert!(result.is_err());
}

#[test]
fn test_build_invalid_yaml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("bad.yaml");
    fs::write(&config_path, "{ invalid yaml: [unclosed").unwrap();

    let result = build_model(&config_path, dir.path().join("out").as_path());

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("YAML parsing error"),
        "expected YAML error, got: {msg}"
    );
}

#[test]
fn test_build_missing_envelope_structure_key() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("no_envelope.yaml");
    // Valid YAML but no envelope_structure field
    fs::write(&config_path, "source: some_source\n").unwrap();

    let result = build_model(&config_path, dir.path().join("out").as_path());

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("envelope_structure"),
        "expected missing envelope_structure error, got: {msg}"
    );
}

#[test]
fn test_build_config_cycle_detected() {
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

    let result = build_model(&dir.path().join("a.yaml"), dir.path().join("out").as_path());

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Cycle detected"),
        "expected cycle error, got: {msg}"
    );
}

#[test]
fn test_duplicated_filename_project() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("filler_1.mcnp"), "Title\n").unwrap();
    fs::create_dir_all(dir.path().join("other")).unwrap();
    fs::write(dir.path().join("other/filler_1.mcnp"), "Title\n").unwrap();
    let config_path = dir.path().join("config.yaml");
    fs::write(&config_path, "{ envelopes: }").unwrap();

    let result = build_model(
        &dir.path().join("config.yaml"),
        dir.path().join("out").as_path(),
    );

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Duplicate file name `filler_1` found"));
}

#[test]
fn test_file_not_found() {
    let dir = tempdir().unwrap();
    let example_project_path = PathBuf::from("example_project/");
    copy_dir(&example_project_path, dir.path()).unwrap();

    fs::write(
        dir.path().join("configurations/valid_configuration.yaml"),
        "project_roots: [.]\nenvelope_structure: ghost_file\nenvelopes: {}\n",
    )
    .unwrap();

    let result = build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path().join("out").as_path(),
    );

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("File `ghost_file` not found in project with root"));
}

#[test]
fn test_failed_to_load_mcnp_file() {
    let dir = tempdir().unwrap();
    let example_project_path = PathBuf::from("example_project/");
    copy_dir(&example_project_path, dir.path()).unwrap();

    fs::write(
        dir.path()
            .join("reference_model/filler_models/filler_model_1.mcnp"),
        "This is not a valid MCNP file content\n",
    )
    .unwrap();

    let result = build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path().join("out").as_path(),
    );

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("No cell ID found in first cell of filler model `filler_model_1`"));
}

#[test]
fn test_metadata_not_found() {
    let dir = tempdir().unwrap();
    let example_project_path = PathBuf::from("example_project/");
    copy_dir(&example_project_path, dir.path()).unwrap();

    fs::remove_file(
        dir.path()
            .join("reference_model/filler_models/filler_model_1.metadata"),
    )
    .unwrap();

    let result = build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path().join("out").as_path(),
    );

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Metadata not found for filler `filler_model_1`"));
}

#[test]
fn test_transformation_metadata_not_found() {
    let dir = tempdir().unwrap();
    let example_project_path = PathBuf::from("example_project/");
    copy_dir(&example_project_path, dir.path()).unwrap();

    fs::write(
        dir.path()
            .join("reference_model/filler_models/filler_model_1.metadata"),
        "\n",
    )
    .unwrap();

    let result = build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path().join("out").as_path(),
    );

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Transformation not specified in the metadata of filler"));
}

#[test]
fn test_filler_first_cell_without_universe_id() {
    let dir = tempdir().unwrap();
    let example_project_path = PathBuf::from("example_project/");
    copy_dir(&example_project_path, dir.path()).unwrap();

    fs::write(
        dir.path()
            .join("reference_model/filler_models/filler_model_1.mcnp"),
        "Cell cards without universe ID\n1 0 -1 imp:n=0\n",
    )
    .unwrap();

    let result = build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path().join("out").as_path(),
    );

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("No universe ID found in first cell of filler model"));
}
