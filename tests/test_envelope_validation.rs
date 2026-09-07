mod common;

use common::copy_dir;
use gitronics::build_model;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

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
