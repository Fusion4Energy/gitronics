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

/// An envelope omitted from the configuration remains unfilled with a warning.
#[test]
fn envelopes_omitted_from_config_are_warned_about() {
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
",
    )
    .unwrap();

    build_model(
        &dir.path().join("configurations/valid_configuration.yaml"),
        dir.path().join("out").as_path(),
    )
    .unwrap();

    let expected = "The envelope `envelope_name_2` is not considered in the configuration file";
    assert!(
        logger.any(|record| record.level() == Level::Warn && record.args().contains(expected)),
        "no warning naming the omitted envelope was logged"
    );
}
