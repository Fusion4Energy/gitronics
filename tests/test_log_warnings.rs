//! Warnings the build emits through the `log` facade.
//!
//! `logtest` installs a process-global logger and drains a single global event
//! queue, so it wants a test binary to itself: with sibling tests running
//! concurrently in the same process the queue interleaves their records, and an
//! assertion can match a message another test emitted. This file therefore
//! holds exactly one test.

mod common;

use common::copy_dir;
use gitronics::build_model;
use log::Level;
use logtest::Logger;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

/// An envelope named in the configuration but absent from the envelope
/// structure is a silent no-op in the assembled model, so the build warns.
#[test]
fn envelopes_in_config_that_dont_exist_are_warned_about() {
    let mut logger = Logger::start();
    let dir = tempdir().unwrap();
    copy_dir(&PathBuf::from("example_project/"), dir.path()).unwrap();

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

    let expected = "The following envelopes were defined in the configuration file but not found in the envelope structure file";
    assert!(
        logger.any(|record| record.level() == Level::Warn && record.args().contains(expected)),
        "no warning naming the missing envelope was logged"
    );
}
