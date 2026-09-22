//! Two builds of the same inputs must produce the same model, and every comment
//! line a component contributes must reach it.
//!
//! Both properties were broken at one point by migjorn's card segmentation
//! varying with the rayon pool size, which cost a 376 MB model 11,174 comment
//! lines and made the count differ from machine to machine. That is fixed
//! upstream; these tests exist so a regression in either direction is caught
//! here rather than in a reviewer's diff.

mod common;

use common::normalise_banner;
use gitronics::build_model;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn build_to_string(config: &Path) -> String {
    let out = tempdir().unwrap();
    build_model(config, out.path()).unwrap_or_else(|e| panic!("build failed: {e}"));
    fs::read_to_string(out.path().join("assembled.mcnp")).unwrap()
}

fn comment_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| {
            let t = l.trim_start();
            t == "C" || t == "c" || t.starts_with("C ") || t.starts_with("c ")
        })
        .collect()
}

#[test]
fn repeated_builds_are_byte_identical() {
    let config = PathBuf::from("example_project/configurations/valid_configuration.yaml");
    let first = normalise_banner(&build_to_string(&config));
    let second = normalise_banner(&build_to_string(&config));
    assert_eq!(
        first, second,
        "two builds of the same configuration differed"
    );
}

/// Comments carry a component's provenance — where the geometry came from, what
/// must not be edited — so losing them silently is losing part of the model.
///
/// This uses a purpose-built project rather than `example_project`, because the
/// case that actually broke is a comment run at the *end* of a block, which
/// `example_project` happens to contain none of.
#[test]
fn comments_survive_the_build_including_at_block_ends() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(
        root.join("config.yaml"),
        "project_roots: [.]\nenvelope_structure: env\nsource: cards\nenvelopes:\n  e1: f1\n",
    )
    .unwrap();
    fs::write(
        root.join("env.mcnp"),
        "env title\n\
         c ENVELOPE CELL HEADER\n\
         1 0 -1 imp:n=1 $ @env:e1\n\
         c ENVELOPE CELL TRAILER\n\
         \n\
         1 SO 10\n\
         c ENVELOPE SURFACE TRAILER\n\
         \n\
         M1 1001 1\n",
    )
    .unwrap();
    fs::write(
        root.join("f1.mcnp"),
        "f1 title\n\
         c FILLER CELL HEADER\n\
         100 0 -100 imp:n=1 u=1\n\
         c FILLER CELL TRAILER\n\
         \n\
         100 SO 3\n\
         c FILLER SURFACE TRAILER\n\
         \n\
         M2 1001 1\n",
    )
    .unwrap();
    fs::write(root.join("f1.metadata"), "transformations:\n  e1: null\n").unwrap();
    fs::write(root.join("cards.source"), "title\nM9 1001 1\n").unwrap();

    let assembled = build_to_string(&root.join("config.yaml"));

    // Headers attach to the card that follows them; trailers sit at the end of a
    // block, with nothing after them to absorb them. Both must survive.
    for marker in [
        "ENVELOPE CELL HEADER",
        "ENVELOPE CELL TRAILER",
        "ENVELOPE SURFACE TRAILER",
        "FILLER CELL HEADER",
        "FILLER CELL TRAILER",
        "FILLER SURFACE TRAILER",
    ] {
        assert!(
            assembled.contains(marker),
            "`{marker}` was dropped from the assembled model:\n{assembled}"
        );
    }
}

/// Every comment in the example project's geometry reaches the output. A weaker
/// count-based check would pass even if the build swapped one comment for
/// another, so this compares the lines themselves.
#[test]
fn example_project_geometry_comments_all_reach_the_output() {
    let assembled = build_to_string(&PathBuf::from(
        "example_project/configurations/valid_configuration.yaml",
    ));
    let assembled_comments: Vec<&str> = comment_lines(&assembled);

    let envelope =
        fs::read_to_string("example_project/reference_model/envelope_structure.mcnp").unwrap();
    // Cells and surfaces only: the data block is replaced by the configured cards.
    let geometry = envelope
        .split("\n\n")
        .take(2)
        .collect::<Vec<_>>()
        .join("\n\n");
    let geometry_comments = comment_lines(&geometry);

    assert!(
        !geometry_comments.is_empty(),
        "fixture has no comments; this test would be vacuous"
    );
    for comment in geometry_comments {
        assert!(
            assembled_comments
                .iter()
                .any(|c| c.trim() == comment.trim()),
            "comment `{comment}` did not reach the assembled model"
        );
    }
}
