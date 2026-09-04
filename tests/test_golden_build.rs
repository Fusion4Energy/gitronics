//! Golden-file tests pinning the exact bytes `gitronics build` produces.
//!
//! The other integration tests assert only that a build *succeeds*; these assert
//! that it produces the same output it produced before. They are what makes a
//! refactor of the composition pipeline verifiable rather than hopeful.
//!
//! Regenerate after an intentional change with:
//!
//! ```sh
//! UPDATE_GOLDEN=1 cargo test --test test_golden_build
//! ```
//!
//! and review the resulting diff.

mod common;

use common::{assert_golden, normalise_banner, normalise_report_json};
use gitronics::build_model;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Builds `config` into a temporary directory and asserts both the assembled
/// model and the build-report manifest match their goldens.
fn assert_build_matches_goldens(config: &str, golden_stem: &str) {
    let output_dir = tempdir().unwrap();
    let config_path = PathBuf::from(config);

    build_model(&config_path, output_dir.path())
        .unwrap_or_else(|e| panic!("build of `{config}` failed: {e}"));

    let assembled = fs::read_to_string(output_dir.path().join("assembled.mcnp")).unwrap();
    assert_golden(
        &Path::new("tests/golden").join(format!("{golden_stem}.assembled.mcnp")),
        &normalise_banner(&assembled),
    );

    let report = fs::read_to_string(output_dir.path().join("build_report.json")).unwrap();
    assert_golden(
        &Path::new("tests/golden").join(format!("{golden_stem}.build_report.json")),
        &normalise_report_json(&report),
    );
}

#[test]
fn golden_valid_configuration() {
    assert_build_matches_goldens(
        "example_project/configurations/valid_configuration.yaml",
        "valid_configuration",
    );
}

#[test]
fn golden_small_override() {
    assert_build_matches_goldens(
        "example_project/assessment_specific/small_override.yaml",
        "small_override",
    );
}

/// The banner must keep its shape: two rule lines around four labelled fields,
/// immediately after the model title. `normalise_banner` deliberately preserves
/// all of that, so a golden alone would not tell us the normaliser is honest.
#[test]
fn normalised_banner_keeps_structure() {
    let output_dir = tempdir().unwrap();
    build_model(
        &PathBuf::from("example_project/configurations/valid_configuration.yaml"),
        output_dir.path(),
    )
    .unwrap();

    let assembled = fs::read_to_string(output_dir.path().join("assembled.mcnp")).unwrap();
    let normalised = normalise_banner(&assembled);
    let lines: Vec<&str> = normalised.lines().take(7).collect();

    assert_eq!(
        lines[1],
        "C ============================================================"
    );
    assert_eq!(lines[2], "C  Built by gitronics v<version>");
    assert_eq!(lines[3], "C  Configuration : <config>");
    assert_eq!(lines[4], "C  Git commit    : <commit>");
    assert_eq!(lines[5], "C  Date / time   : <datetime>");
    assert_eq!(
        lines[6],
        "C ============================================================"
    );

    // The volatile values really are gone.
    assert!(!normalised.contains("Built by gitronics v0."));
    assert!(!normalised.contains(" UTC"));
}
