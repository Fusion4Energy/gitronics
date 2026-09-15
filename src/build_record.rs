use crate::build_report::{BuildReport, BuildSnapshot};
use crate::error::GitronicsError;
use crate::project_manager::ProjectManager;
use crate::provenance::Evidence;
use crate::types::{FillerName, UniverseId};
use migjorn::Model;
use std::{collections::HashMap, path::Path};

#[derive(Clone, Copy, Debug)]
pub(crate) enum CheckKind {
    GeometryPhysics,
    EnvelopeAssignments,
    DataParsing,
    Collisions,
    References,
    InputStability,
    Output,
}

impl CheckKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::GeometryPhysics => "Geometry and transport physics",
            Self::EnvelopeAssignments => "Envelope assignments",
            Self::DataParsing => "Data card parsing",
            Self::Collisions => "Card ID collisions",
            Self::References => "Assembled model references",
            Self::InputStability => "Input stability",
            Self::Output => "Output artifacts",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CheckStatus {
    Passed,
    Failed,
    NotApplicable,
    NotPerformed,
}

impl CheckStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotApplicable => "not_applicable",
            Self::NotPerformed => "not_performed",
        }
    }
}

#[derive(Debug)]
pub(crate) struct RecordedCheck {
    pub kind: CheckKind,
    pub status: CheckStatus,
    pub details: Vec<String>,
}

#[derive(Default)]
pub(crate) enum BuildStatus {
    #[default]
    Incomplete,
    Success,
    Failed,
}

impl BuildStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }
}

pub(crate) struct BuildRecord {
    pub snapshot: Option<BuildSnapshot>,
    pub status: BuildStatus,
    pub evidence: Evidence,
    pub checks: Vec<RecordedCheck>,
    pub warnings: Vec<String>,
}

impl Default for BuildRecord {
    fn default() -> Self {
        Self {
            snapshot: None,
            status: BuildStatus::Incomplete,
            evidence: Evidence::default(),
            checks: vec![RecordedCheck {
                kind: CheckKind::GeometryPhysics,
                status: CheckStatus::NotPerformed,
                details: vec![],
            }],
            warnings: vec![],
        }
    }
}

impl BuildRecord {
    pub fn capture(
        &mut self,
        config_path: &Path,
        commit_hash: String,
        manager: &ProjectManager,
        envelope: &Model,
        fillers: &[(FillerName, Model)],
        universes: &HashMap<FillerName, UniverseId>,
    ) {
        self.snapshot = Some(BuildSnapshot::from_build(
            config_path,
            commit_hash,
            manager,
            envelope,
            fillers,
            universes,
        ));
    }

    pub fn check<T>(
        &mut self,
        kind: CheckKind,
        result: Result<T, GitronicsError>,
    ) -> Result<T, GitronicsError> {
        self.checks.push(RecordedCheck {
            kind,
            status: if result.is_ok() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            details: match result.as_ref().err() {
                Some(GitronicsError::InvalidModel(problems)) => {
                    problems.iter().map(ToString::to_string).collect()
                }
                Some(error) => vec![error.to_string()],
                None => vec![],
            },
        });
        result
    }

    pub fn skip(&mut self, kind: CheckKind) {
        self.checks.push(RecordedCheck {
            kind,
            status: CheckStatus::NotApplicable,
            details: vec![],
        });
    }

    pub fn parsing_check(&mut self, errors: Vec<String>) -> Result<(), GitronicsError> {
        let result = if errors.is_empty() {
            Ok(())
        } else {
            Err(GitronicsError::Io(std::io::Error::other(errors.join("\n"))))
        };
        self.checks.push(RecordedCheck {
            kind: CheckKind::DataParsing,
            status: if result.is_ok() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            details: errors,
        });
        result
    }

    pub fn verify_inputs(&mut self) -> Result<(), GitronicsError> {
        let unchanged = self.evidence.verify_inputs();
        self.checks.push(RecordedCheck {
            kind: CheckKind::InputStability,
            status: if unchanged {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            details: if unchanged {
                vec![]
            } else {
                vec!["Input files changed during assembly".into()]
            },
        });
        if unchanged {
            Ok(())
        } else {
            Err(GitronicsError::Io(std::io::Error::other(
                "Input files changed during assembly; rebuild before using the output",
            )))
        }
    }

    pub fn finish(
        mut self,
        result: Result<(), GitronicsError>,
        output: &Path,
    ) -> Result<(), GitronicsError> {
        if self.snapshot.is_none() {
            return result;
        }
        let failed_check = self
            .checks
            .iter()
            .any(|check| matches!(check.status, CheckStatus::Failed));
        if let Err(error) = &result {
            self.status = BuildStatus::Failed;
            self.evidence.output = None;
            if !failed_check {
                self.checks.push(RecordedCheck {
                    kind: CheckKind::Output,
                    status: CheckStatus::Failed,
                    details: vec![error.to_string()],
                });
            }
        } else {
            self.status = BuildStatus::Success;
        }
        let written = BuildReport::from_record(self)
            .expect("snapshot exists")
            .write(output);
        match result {
            Err(error) => {
                if let Err(report_error) = written {
                    log::warn!("Could not write the failed-build report: {report_error}");
                }
                Err(error)
            }
            Ok(()) => written,
        }
    }
}

pub(crate) fn runs_from_ids(mut ids: Vec<i64>) -> Vec<[i64; 2]> {
    ids.sort_unstable();
    ids.dedup();
    let mut runs: Vec<[i64; 2]> = Vec::new();
    for id in ids {
        match runs.last_mut() {
            Some(last) if last[1].checked_add(1) == Some(id) => last[1] = id,
            _ => runs.push([id, id]),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> BuildRecord {
        let manager = ProjectManager::new(
            Path::new("example_project/configurations/valid_configuration.yaml"),
            &std::env::temp_dir(),
        )
        .unwrap();
        let mut record = BuildRecord::default();
        record.capture(
            Path::new("test.yaml"),
            "unknown".into(),
            &manager,
            &Model::parse("empty\n\n\n"),
            &[],
            &HashMap::new(),
        );
        record
    }

    #[test]
    fn parser_failures_keep_individual_messages() {
        let mut record = record();
        let errors = vec!["first parser error".into(), "second parser error".into()];
        let result = record.parsing_check(errors.clone());
        let output = tempfile::tempdir().unwrap();
        assert!(record.finish(result, output.path()).is_err());
        let report: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output.path().join("build_report.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["checks"][1]["details"], serde_json::json!(errors));
    }

    #[test]
    fn report_write_failure_preserves_build_error() {
        let output = tempfile::tempdir().unwrap();
        std::fs::create_dir(output.path().join("build_report.json")).unwrap();
        let result = record().finish(
            Err(GitronicsError::ValidationError("original failure".into())),
            output.path(),
        );
        assert!(
            matches!(result, Err(GitronicsError::ValidationError(message)) if message == "original failure")
        );
    }

    #[test]
    fn report_write_failure_fails_an_otherwise_successful_build() {
        let output = tempfile::tempdir().unwrap();
        std::fs::create_dir(output.path().join("build_report.json")).unwrap();
        assert!(matches!(
            record().finish(Ok(()), output.path()),
            Err(GitronicsError::IoPath { .. })
        ));
    }

    #[test]
    fn output_failure_is_recorded() {
        let output = tempfile::tempdir().unwrap();
        assert!(
            record()
                .finish(
                    Err(GitronicsError::Io(std::io::Error::other("output failed"))),
                    output.path()
                )
                .is_err()
        );
        let report: serde_json::Value = serde_json::from_slice(
            &std::fs::read(output.path().join("build_report.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["build_status"], "failed");
        assert_eq!(report["checks"][1]["name"], "Output artifacts");
        assert!(report["evidence"]["output"].is_null());
    }
}
