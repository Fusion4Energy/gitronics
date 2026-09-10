mod common;

use common::copy_dir;
use gitronics::build_model;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

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
fn test_build_override_clears_inherited_source() {
    let dir = tempdir().unwrap();
    copy_dir(&PathBuf::from("example_project"), dir.path()).unwrap();
    fs::remove_file(
        dir.path()
            .join("reference_model/data_cards/volumetric_source.source"),
    )
    .unwrap();
    let config = dir.path().join("configurations/without_source.yaml");
    fs::write(
        &config,
        "overrides: valid_configuration.yaml\nsource: null\n",
    )
    .unwrap();
    let output = dir.path().join("out");

    build_model(&config, &output).unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("build_report.json")).unwrap())
            .unwrap();
    assert!(report.get("source").is_none());
    assert!(output.join("assembled.mcnp").exists());
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
        msg.contains("ID collisions")
            && msg.contains("duplicate cell id 100")
            && msg.contains("filler `f1`")
            && msg.contains("filler `f2`"),
        "expected a cell-100 collision naming both fillers, got: {msg}"
    );
}

// ── .gitignore handling ───────────────────────────────────────────────────────
// A build writes a `.gitignore` containing `*` to mark its output directory as
// build artefacts. Because `--output-path` defaults to `.`, that directory is
// very often one holding the user's own work, so the write must only ever
// create — never replace an existing file.

#[test]
fn build_does_not_clobber_existing_gitignore() {
    let dir = tempdir().unwrap();
    let example_project_path = PathBuf::from("example_project/");
    copy_dir(&example_project_path, dir.path()).unwrap();

    let gitignore = dir.path().join(".gitignore");
    let original = "target/\nsecret.txt\n";
    fs::write(&gitignore, original).unwrap();

    build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path(),
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(&gitignore).unwrap(),
        original,
        "the build replaced the project's .gitignore"
    );
    // The build still did its job.
    assert!(dir.path().join("assembled.mcnp").exists());
}

#[test]
fn build_writes_gitignore_into_fresh_output_dir() {
    let output_dir = tempdir().unwrap();
    let out = output_dir.path().join("out");

    build_model(
        &PathBuf::from("example_project/configurations/valid_configuration.yaml"),
        &out,
    )
    .unwrap();

    assert_eq!(fs::read_to_string(out.join(".gitignore")).unwrap(), "*\n");
}

#[test]
fn rebuilding_into_the_same_directory_is_idempotent() {
    let output_dir = tempdir().unwrap();
    let out = output_dir.path().join("out");
    let config = PathBuf::from("example_project/configurations/valid_configuration.yaml");

    build_model(&config, &out).unwrap();
    build_model(&config, &out).expect("second build into the same directory");

    assert_eq!(fs::read_to_string(out.join(".gitignore")).unwrap(), "*\n");
}

#[test]
fn migrate_does_not_clobber_existing_output_gitignore() {
    let dir = tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(project.join("output")).unwrap();

    let gitignore = project.join("output/.gitignore");
    let original = "# keep the assembled model\n!assembled.mcnp\n";
    fs::write(&gitignore, original).unwrap();

    gitronics::migrate_model(&PathBuf::from("resources/simple_model.mcnp"), &project).unwrap();

    assert_eq!(
        fs::read_to_string(&gitignore).unwrap(),
        original,
        "migrate replaced an existing output/.gitignore"
    );
}
