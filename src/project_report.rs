//! Project-report generator.
//!
//! Where [`crate::build_report`] describes a *single assembled build*, this
//! describes the *whole project*: the full filler library (including fillers no
//! configuration uses), the full envelope inventory (including unassigned
//! envelopes), and every configuration's assignment and coverage. It is produced
//! by [`crate::inspect`] and rendered by the committed single-file viewer.
//!
//! Two artefacts are emitted:
//! * `project_report.json` — the machine-readable manifest.
//! * `project_report.html` — the self-contained interactive dashboard (the
//!   viewer template with the manifest injected into its `#report-data` block).

use serde::Serialize;
use std::path::Path;

use crate::build_report::escape_json_for_script;
use crate::project_manager::Metadata;
use crate::types::{EnvelopeName, FillerName, UniverseId};
use crate::utils::GitronicsError;

/// Schema version of the emitted manifest. Bump on breaking changes so the
/// viewer (and downstream tooling) can adapt.
pub const SCHEMA_VERSION: u32 = 1;

/// The placeholder token in the viewer template that the manifest replaces.
const DATA_PLACEHOLDER: &str = "__GITRONICS_REPORT_DATA__";

// ─── Public data types ────────────────────────────────────────────────────────

/// One filler universe in the project library.
#[derive(Debug, Serialize)]
pub struct FillerLib {
    pub name: FillerName,
    pub universe_id: UniverseId,
    pub cell_count: usize,
    pub surface_count: usize,
    pub materials: Vec<i64>,
    pub cell_id_runs: Vec<[i64; 2]>,
    pub surface_id_runs: Vec<[i64; 2]>,
    /// The recognized first-class `description` field, if the filler set one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Remaining project-defined metadata (any keys except `description`).
    pub metadata: Metadata,
    /// Names of the configurations that place this filler in ≥1 envelope.
    pub used_by_configs: Vec<String>,
}

/// One envelope placeholder in the envelope-structure inventory.
#[derive(Debug, Serialize)]
pub struct EnvelopeInv {
    pub envelope_name: EnvelopeName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub metadata: Metadata,
}

/// One envelope's assignment within a configuration.
#[derive(Debug, Serialize)]
pub struct ConfigEnvelope {
    pub envelope_name: EnvelopeName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filler_name: Option<FillerName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe_id: Option<UniverseId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
}

/// Coverage statistics for a configuration.
#[derive(Debug, Serialize)]
pub struct ConfigStats {
    pub filled: usize,
    pub unfilled: usize,
    pub distinct_fillers: usize,
    pub unused_fillers: usize,
}

/// One configuration and its assignment of fillers to the envelope inventory.
#[derive(Debug, Serialize)]
pub struct ConfigEntry {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<String>,
    pub envelopes: Vec<ConfigEnvelope>,
    pub stats: ConfigStats,
}

/// The metadata keys present across the project, driving the viewer's group-by
/// and facet controls (`description` is excluded — it is first-class).
#[derive(Debug, Serialize)]
pub struct MetadataKeys {
    pub filler: Vec<String>,
    pub envelope: Vec<String>,
}

/// The complete project-report manifest.
#[derive(Debug, Serialize)]
pub struct ProjectReport {
    pub schema_version: u32,
    pub gitronics_version: &'static str,
    pub commit_hash: String,
    pub date_time: String,
    pub project_dir: String,
    pub filler_library: Vec<FillerLib>,
    pub envelope_inventory: Vec<EnvelopeInv>,
    pub configurations: Vec<ConfigEntry>,
    pub metadata_keys: MetadataKeys,
}

// ─── Rendering ────────────────────────────────────────────────────────────────

impl ProjectReport {
    /// Serialises the manifest to pretty-printed JSON (for `project_report.json`).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| {
            format!("{{\"error\":\"failed to serialize project report: {e}\"}}")
        })
    }

    /// Renders the self-contained dashboard by injecting the manifest into the
    /// committed viewer template (built from `viewer/` via `npm run build`).
    pub fn generate_html(&self) -> String {
        const TEMPLATE: &str = include_str!("project_report.html");
        let data = escape_json_for_script(&self.to_json());
        TEMPLATE.replacen(DATA_PLACEHOLDER, &data, 1)
    }

    /// Writes the HTML dashboard and JSON manifest into `output_path`.
    pub fn write(&self, output_path: &Path) -> Result<(), GitronicsError> {
        std::fs::write(output_path.join("project_report.html"), self.generate_html())?;
        std::fs::write(output_path.join("project_report.json"), self.to_json())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> ProjectReport {
        let mut meta = Metadata::new();
        meta.insert("component_type".to_string(), json!("blanket"));
        ProjectReport {
            schema_version: SCHEMA_VERSION,
            gitronics_version: "1.2.3",
            commit_hash: "abc1234".to_string(),
            date_time: "2026-01-01 00:00:00 UTC".to_string(),
            project_dir: "proj".to_string(),
            filler_library: vec![FillerLib {
                name: FillerName::new("universe_101"),
                universe_id: UniverseId::new(101),
                cell_count: 3,
                surface_count: 5,
                materials: vec![110],
                cell_id_runs: vec![[250000, 250002]],
                surface_id_runs: vec![[250000, 250004]],
                description: Some("Blanket".to_string()),
                metadata: meta,
                used_by_configs: vec!["baseline".to_string()],
            }],
            envelope_inventory: vec![EnvelopeInv {
                envelope_name: EnvelopeName::new("env_a"),
                description: None,
                metadata: Metadata::new(),
            }],
            configurations: vec![ConfigEntry {
                name: "baseline".to_string(),
                path: "configurations/baseline.yaml".to_string(),
                overrides: None,
                envelopes: vec![ConfigEnvelope {
                    envelope_name: EnvelopeName::new("env_a"),
                    filler_name: Some(FillerName::new("universe_101")),
                    universe_id: Some(UniverseId::new(101)),
                    transform: Some("(10)".to_string()),
                }],
                stats: ConfigStats {
                    filled: 1,
                    unfilled: 0,
                    distinct_fillers: 1,
                    unused_fillers: 0,
                },
            }],
            metadata_keys: MetadataKeys {
                filler: vec!["component_type".to_string()],
                envelope: vec![],
            },
        }
    }

    #[test]
    fn json_roundtrips_and_shapes_ids() {
        let value: serde_json::Value = serde_json::from_str(&sample().to_json()).unwrap();
        assert_eq!(value["schema_version"], SCHEMA_VERSION);
        assert_eq!(value["filler_library"][0]["universe_id"], 101);
        assert_eq!(value["filler_library"][0]["name"], "universe_101");
        assert_eq!(value["filler_library"][0]["description"], "Blanket");
        assert_eq!(value["configurations"][0]["stats"]["filled"], 1);
        assert_eq!(value["metadata_keys"]["filler"][0], "component_type");
    }

    #[test]
    fn html_injects_manifest_and_is_offline() {
        let html = sample().generate_html();
        assert!(html.starts_with("<!DOCTYPE html>"), "not a document");
        // The placeholder is replaced by escaped JSON — no raw token remains.
        assert!(
            !html.contains("__GITRONICS_REPORT_DATA__"),
            "placeholder not replaced"
        );
        assert!(html.contains("universe_101"), "manifest not embedded");
        // No external resource *loads* (fully self-contained; URL strings inside
        // the bundled JS are fine, external src/href are not).
        assert!(!html.contains("src=\"http"), "external src load");
        assert!(!html.contains("href=\"http"), "external href load");
    }

    #[test]
    fn html_neutralises_script_breakout() {
        let mut report = sample();
        report.filler_library[0].name =
            FillerName::new("</script><script>alert(1)</script>");
        let html = report.generate_html();
        assert!(
            !html.contains("</script><script>alert(1)"),
            "script breakout not neutralised"
        );
        assert!(html.contains("\\u003c"), "angle brackets not escaped");
    }

    #[test]
    fn write_emits_both_files() {
        let dir = tempfile::tempdir().unwrap();
        sample().write(dir.path()).unwrap();
        assert!(dir.path().join("project_report.html").exists());
        assert!(dir.path().join("project_report.json").exists());
    }
}
