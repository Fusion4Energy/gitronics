//! The envelope-structure `.metadata` sidecar is descriptive and best-effort:
//! mismatches with the `$ @env:` markers are reported as warnings, never errors.

use gitronics::build_model;
use std::fs;
use tempfile::tempdir;

/// Builds a two-envelope project with the given sidecar (or none) and returns
/// the parsed `build_report.json`.
fn build_with_sidecar(sidecar: Option<&str>) -> serde_json::Value {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("config.yaml"),
        "envelope_structure: shell\nenvelopes:\n  alpha: null\n  beta: null\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("shell.mcnp"),
        "shell\n1 0 -1 $ @env: alpha\n2 0 1 $ @env: beta\n\n1 so 5\n\n",
    )
    .unwrap();
    if let Some(sidecar) = sidecar {
        fs::write(dir.path().join("shell.metadata"), sidecar).unwrap();
    }
    let output = dir.path().join("output");
    build_model(&dir.path().join("config.yaml"), &output).unwrap();
    serde_json::from_str(&fs::read_to_string(output.join("build_report.json")).unwrap()).unwrap()
}

/// Warnings about the envelope metadata (the fixture has no source file, so
/// the report always carries an unrelated "No source file selected").
fn warnings(report: &serde_json::Value) -> Vec<&str> {
    report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|warning| warning.as_str().unwrap())
        .filter(|warning| warning.starts_with("Envelope metadata"))
        .collect()
}

fn entry<'a>(report: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    report["envelope_entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["envelope_name"] == name)
        .unwrap()
}

#[test]
fn metadata_is_attached_to_the_matching_envelope() {
    let report = build_with_sidecar(Some("envelopes:\n  alpha:\n    zone: A\n"));

    assert_eq!(entry(&report, "alpha")["metadata"]["zone"], "A");
    // No metadata for `beta` is not worth a warning.
    assert!(entry(&report, "beta").get("metadata").is_none());
    assert!(warnings(&report).is_empty(), "{:?}", warnings(&report));
}

#[test]
fn metadata_for_an_unmarked_envelope_is_warned_about_once_with_sorted_names() {
    let report = build_with_sidecar(Some(
        "envelopes:\n  alpha:\n    zone: A\n  zeta:\n    zone: Z\n  gamma:\n    zone: G\n",
    ));

    let unmatched: Vec<_> = warnings(&report)
        .into_iter()
        .filter(|warning| warning.contains("not marked in the envelope structure"))
        .collect();
    assert_eq!(unmatched.len(), 1, "{:?}", warnings(&report));
    assert!(unmatched[0].ends_with(": gamma, zeta"), "{}", unmatched[0]);
    // The orphans never reach the envelope entries.
    assert_eq!(report["envelope_entries"].as_array().unwrap().len(), 2);
    assert_eq!(report["build_status"], "success");
}

#[test]
fn other_top_level_keys_are_left_alone() {
    let report = build_with_sidecar(Some(
        "owner: neutronics team\nrevision: 3\nenvelopes:\n  alpha:\n    zone: A\n",
    ));

    assert_eq!(entry(&report, "alpha")["metadata"]["zone"], "A");
    assert!(warnings(&report).is_empty(), "{:?}", warnings(&report));
    assert_eq!(report["build_status"], "success");
}

#[test]
fn bare_values_are_kept_a_string_as_the_description() {
    let report = build_with_sidecar(Some("envelopes:\n  alpha: Central blanket\n  beta: 3\n"));

    assert_eq!(
        entry(&report, "alpha")["metadata"]["description"],
        "Central blanket"
    );
    assert!(entry(&report, "alpha")["metadata"].get("value").is_none());
    assert_eq!(entry(&report, "beta")["metadata"]["value"], 3);
    assert!(warnings(&report).is_empty(), "{:?}", warnings(&report));
}

#[test]
fn a_missing_sidecar_is_silent() {
    let report = build_with_sidecar(None);

    assert!(warnings(&report).is_empty(), "{:?}", warnings(&report));
    assert!(entry(&report, "alpha").get("metadata").is_none());
}
