mod common;

use common::copy_dir;
use gitronics::build_model;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn report_includes_unconfigured_and_explicitly_empty_envelopes() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("config.yaml"),
        "envelope_structure: shell\nenvelopes:\n  empty: null\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("shell.mcnp"),
        "shell\n1 0 -1 $ @env: empty\n2 0 1 $ @env: omitted\n\n1 so 5\n\n",
    )
    .unwrap();
    let output = dir.path().join("output");
    build_model(&dir.path().join("config.yaml"), &output).unwrap();
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output.join("build_report.json")).unwrap())
            .unwrap();
    let entries = report["envelope_entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["status"], "empty");
    assert_eq!(entries[1]["status"], "unconfigured");
    assert_eq!(entries[1]["cell_ids"], serde_json::json!([2]));
    assert_eq!(report["build_status"], "success");
    assert_eq!(report["evidence"]["inputs_unchanged"], true);
    assert_eq!(
        report["evidence"]["output"]["sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert!(
        report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning.as_str().unwrap().contains("omitted"))
    );
}

#[test]
fn report_tracks_mixed_cards_content_changes_and_failed_validation() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.yaml");
    fs::write(
        &config,
        "envelope_structure: shell\nmaterials: [mixed]\nenvelopes:\n  empty: null\n",
    )
    .unwrap();
    let shell = dir.path().join("shell.mcnp");
    fs::write(&shell, "shell\n1 1 -1 -1 $ @env: empty\n\n1 so 5\n\n").unwrap();
    let cards = dir.path().join("mixed.mat");
    fs::write(&cards, "mixed\nM1 1001 1\n*TR40 0 0 0\nMODE N P\nNPS 100\n").unwrap();
    let output = dir.path().join("output");
    let read_report = || -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(output.join("build_report.json")).unwrap())
            .unwrap()
    };
    build_model(&config, &output).unwrap();
    let first = read_report();
    let inventory = first["evidence"]["data_cards"].as_array().unwrap();
    assert!(
        inventory
            .iter()
            .any(|card| card["name"] == "*TR40" && card["category"] == "Transforms")
    );
    assert!(
        inventory
            .iter()
            .any(|card| card["name"] == "NPS" && card["category"] == "Run settings")
    );
    assert_eq!(
        inventory[0]["referenced_cell_runs"],
        serde_json::json!([[1, 1]])
    );
    build_model(&config, &output).unwrap();
    assert_eq!(
        first["evidence"]["content_fingerprint"],
        read_report()["evidence"]["content_fingerprint"]
    );
    fs::write(&cards, "mixed\nM1 1001 1\n*TR40 0 0 0\nMODE N P\nNPS 200\n").unwrap();
    build_model(&config, &output).unwrap();
    assert_ne!(
        first["evidence"]["content_fingerprint"],
        read_report()["evidence"]["content_fingerprint"]
    );
    fs::write(&shell, "shell\n1 2 -1 -1 $ @env: empty\n\n1 so 5\n\n").unwrap();
    assert!(build_model(&config, &output).is_err());
    let failed = read_report();
    assert_eq!(failed["build_status"], "failed");
    assert!(failed["evidence"]["output"].is_null());
    assert!(
        failed["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["status"] == "failed")
    );
}

#[test]
fn output_write_failure_produces_a_failed_report() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("config.yaml"),
        "envelope_structure: shell\n",
    )
    .unwrap();
    fs::write(dir.path().join("shell.mcnp"), "shell\n1 0 -1\n\n1 so 5\n\n").unwrap();
    let output = dir.path().join("output");
    fs::create_dir_all(output.join("assembled.mcnp")).unwrap();
    let error = build_model(&dir.path().join("config.yaml"), &output).unwrap_err();
    assert!(error.to_string().contains("assembled.mcnp"));
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("build_report.json")).unwrap()).unwrap();
    assert_eq!(report["build_status"], "failed");
    assert!(report["evidence"]["output"].is_null());
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["name"] == "Output artifacts" && check["status"] == "failed")
    );
}

#[test]
fn unknown_configured_envelopes_fail_before_writing_artifacts() {
    for assignment in ["filler_model_2", "null"] {
        let dir = tempdir().unwrap();
        copy_dir(Path::new("example_project"), dir.path()).unwrap();
        let config = dir.path().join("configurations/unknown_envelopes.yaml");
        fs::write(
            &config,
            format!(
                "overrides: valid_configuration.yaml\nenvelopes:\n  unknown_z: {assignment}\n  unknown_a: null\n"
            ),
        )
        .unwrap();
        let output = dir.path().join("out");

        let error = build_model(&config, &output).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("not found in the envelope structure file: unknown_a, unknown_z."),
            "unexpected error: {error}"
        );
        for artifact in ["assembled.mcnp", "build_report.html", "build_report.json"] {
            assert!(!output.join(artifact).exists(), "unexpected {artifact}");
        }
    }
}
