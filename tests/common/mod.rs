//! Shared helpers for the integration tests.
//!
//! `tests/common/mod.rs` is compiled as a module of each test binary that
//! declares `mod common;`, not as a test binary of its own.

#![allow(dead_code)] // each test binary uses only part of this module

use std::fs;
use std::path::Path;

/// Recursively copies `src` into `dst` (which must already exist).
pub fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
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

/// The banner fields whose values change from build to build: the gitronics
/// version, the invoking configuration path, the git description and the
/// timestamp. Each entry is `(line prefix, replacement placeholder)`.
const BANNER_FIELDS: &[(&str, &str)] = &[
    ("C  Built by gitronics v", "<version>"),
    ("C  Configuration : ", "<config>"),
    ("C  Git commit    : ", "<commit>"),
    ("C  Date / time   : ", "<datetime>"),
];

/// Replaces the *values* of the provenance banner with fixed placeholders,
/// leaving every other byte — including the two `C ===` rules, the banner's
/// position right after the title, and its line count — exactly as emitted.
///
/// Normalising rather than deleting keeps the banner's shape under test: a
/// regression that moved, reordered or dropped a banner line still fails the
/// golden comparison.
pub fn normalise_banner(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        match BANNER_FIELDS
            .iter()
            .find(|(prefix, _)| line.starts_with(prefix))
        {
            Some((prefix, placeholder)) => {
                out.push_str(prefix);
                out.push_str(placeholder);
            }
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

/// The build-report fields that change from build to build.
const VOLATILE_REPORT_FIELDS: &[&str] = &[
    "config_path",
    "gitronics_version",
    "commit_hash",
    "date_time",
];

/// Replaces the volatile top-level fields of a `build_report.json` with fixed
/// placeholders, keeping every other key and the document's key order intact.
///
/// Operates on the parsed value rather than on the text so that a config path
/// containing regex- or JSON-significant characters cannot break the
/// substitution.
pub fn normalise_report_json(json: &str) -> String {
    let mut value: serde_json::Value =
        serde_json::from_str(json).expect("build report is valid JSON");
    let obj = value
        .as_object_mut()
        .expect("build report is a JSON object");
    for field in VOLATILE_REPORT_FIELDS {
        if let Some(slot) = obj.get_mut(*field) {
            *slot = serde_json::Value::String(format!("<{field}>"));
        }
    }
    serde_json::to_string_pretty(&value).expect("re-serialises")
}

/// Compares `actual` against the golden file at `path`.
///
/// Run the suite with `UPDATE_GOLDEN=1` to rewrite the goldens from the current
/// behaviour instead of asserting against them.
pub fn assert_golden(path: &Path, actual: &str) {
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create golden directory");
        }
        fs::write(path, actual).expect("write golden");
        return;
    }

    let expected = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "could not read golden `{}`: {e}\n\
             Run `UPDATE_GOLDEN=1 cargo test` to create it.",
            path.display()
        )
    });

    if expected != actual {
        panic!(
            "output does not match golden `{}`.\n{}\n\
             If this change is intentional, run `UPDATE_GOLDEN=1 cargo test` and review the diff.",
            path.display(),
            first_difference(&expected, actual)
        );
    }
}

/// A short, readable description of where two texts first diverge.
fn first_difference(expected: &str, actual: &str) -> String {
    let mut expected_lines = expected.lines();
    let mut actual_lines = actual.lines();
    let mut line_no = 1;
    loop {
        match (expected_lines.next(), actual_lines.next()) {
            (Some(e), Some(a)) if e == a => line_no += 1,
            (Some(e), Some(a)) => {
                return format!(
                    "first difference at line {line_no}:\n  expected: {e}\n  actual:   {a}"
                );
            }
            (Some(e), None) => {
                return format!("actual output ends at line {line_no}; expected had: {e}");
            }
            (None, Some(a)) => {
                return format!("expected output ends at line {line_no}; actual had: {a}");
            }
            (None, None) => return "texts differ only in trailing newlines".to_string(),
        }
    }
}
