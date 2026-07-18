use gitronics::{build_model, inspect_project};
use log::Level;
use logtest::Logger;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn test_inspect_example_project() {
    let output_dir = tempdir().unwrap();
    let project_dir = PathBuf::from("example_project");

    let result = inspect_project(&project_dir, output_dir.path());
    assert!(result.is_ok(), "inspect failed: {:?}", result.err());

    let json = fs::read_to_string(output_dir.path().join("project_report.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Full filler library, including the filler that no primary config uses.
    let fillers = v["filler_library"].as_array().unwrap();
    assert_eq!(fillers.len(), 3, "expected the full 3-filler library");
    let f3 = fillers
        .iter()
        .find(|f| f["name"] == "filler_model_3")
        .expect("filler_model_3 present in library");
    assert!(
        f3["used_by_configs"].as_array().unwrap().is_empty(),
        "filler_model_3 is unused (its envelope is not a real @env placeholder)"
    );

    // Both configurations are discovered (across configurations/ and assessment_specific/).
    let configs = v["configurations"].as_array().unwrap();
    assert_eq!(configs.len(), 2);

    // The envelope inventory is the two real @env placeholders in the structure.
    assert_eq!(v["envelope_inventory"].as_array().unwrap().len(), 2);

    // The primary config fills both envelopes and flags an unused filler.
    let valid = configs
        .iter()
        .find(|c| c["name"] == "valid_configuration")
        .unwrap();
    assert_eq!(valid["stats"]["filled"], 2);
    assert_eq!(valid["stats"]["unfilled"], 0);
    assert!(valid["stats"]["unused_fillers"].as_u64().unwrap() >= 1);

    // The HTML dashboard is written, self-contained, and carries the manifest.
    let html = fs::read_to_string(output_dir.path().join("project_report.html")).unwrap();
    assert!(
        !html.contains("__GITRONICS_REPORT_DATA__"),
        "manifest placeholder must be replaced"
    );
    assert!(html.contains("filler_model_3"), "manifest embedded in HTML");
}

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

#[test]
fn test_id_collision_between_fillers_is_reported() {
    // Two fillers that both define cell id 100 violate the disjoint-range
    // convention; the build must fail and name the clashing id.
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("config.yaml"),
        "project_roots: [.]\nenvelope_structure: env\nenvelopes:\n  e1: f1\n  e2: f2\n",
    )
    .unwrap();
    fs::write(
        root.join("env.mcnp"),
        "env title\n1 0 -1 imp:n=1 $ @env:e1\n2 0 -2 imp:n=1 $ @env:e2\n\n1 SO 5\n2 SO 6\n\nM1 1001 1\n",
    )
    .unwrap();
    fs::write(
        root.join("f1.mcnp"),
        "f1\n100 0 -100 imp:n=1 u=1\n\n100 SO 3\n\nM1 1001 1\n",
    )
    .unwrap();
    fs::write(
        root.join("f2.mcnp"),
        "f2\n100 0 -200 imp:n=1 u=2\n\n200 SO 3\n\nM1 1001 1\n",
    )
    .unwrap();
    fs::write(root.join("f1.metadata"), "transformations:\n  e1: null\n").unwrap();
    fs::write(root.join("f2.metadata"), "transformations:\n  e2: null\n").unwrap();

    let result = build_model(&root.join("config.yaml"), &root.join("out"));

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("ID collisions") && msg.contains("cell 100"),
        "expected a cell-100 collision error, got: {msg}"
    );
}

#[test]
fn test_envelopes_in_config_that_dont_exist() {
    let mut logger = Logger::start();
    let dir = tempdir().unwrap();
    let example_project_path = PathBuf::from("example_project/");
    copy_dir(&example_project_path, dir.path()).unwrap();

    fs::write(
        dir.path().join("configurations/valid_configuration.yaml"),
        "project_roots: [..]
overrides: null

envelope_structure: envelope_structure
source: volumetric_source
materials: [materials]
transformations: [my_transform]
tallies: [fine_mesh]
envelopes:
  my_envelope_name_1: filler_model_1
  wrong_envelope_name: filler_model_2
",
    )
    .unwrap();

    build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path().join("out").as_path(),
    )
    .unwrap();

    let warn_message = "The following envelopes were defined in the configuration file but not found in the envelope structure file";

    assert!(
        logger.any(|record| {
            record.level() == Level::Warn && record.args().contains(warn_message)
        })
    );
}
