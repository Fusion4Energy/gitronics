//! Build-report generator.
//!
//! Produces two artefacts describing the assembled model:
//!
//! * `build_report.json` — a structured, machine-readable manifest (the single
//!   source of truth) suitable for CI, regression diffing and external tooling.
//! * `build_report.html` — a single, self-contained, offline interactive viewer.
//!   The manifest is embedded verbatim into the HTML and rendered client-side by
//!   the bundled [`report.js`]/[`report.css`] assets. No network access, no build
//!   step, deterministic output.

use serde::Serialize;

use crate::project_manager::{Metadata, ProjectManager};
use crate::types::{EnvelopeName, FillerName, UniverseId};
use crate::utils::{GitronicsError, get_hash_of_project, project_dir};
use indexmap::IndexMap;
use migjorn::Model;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// Schema version of the emitted manifest. Bump on breaking changes so the
/// viewer (and downstream tooling) can adapt.
pub const SCHEMA_VERSION: u32 = 1;

// ─── Public data types ────────────────────────────────────────────────────────

/// One envelope in the assembled model.
#[derive(Debug, Serialize)]
pub struct EnvelopeEntry {
    pub envelope_name: EnvelopeName,
    /// `None` means the envelope was explicitly set to `null` in the config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filler_name: Option<FillerName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe_id: Option<UniverseId>,
    /// Raw transform text (e.g. `(40)`, `*(…)`) or `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
    /// Arbitrary, project-defined metadata (any keys the user chose to record).
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: Metadata,
}

/// One filler model in the assembled model.
#[derive(Debug, Serialize)]
pub struct FillerEntry {
    pub name: FillerName,
    pub universe_id: UniverseId,
    pub envelope_count: usize,
    pub cell_count: usize,
    pub surface_count: usize,
    /// The exact cell ids used, run-length encoded as inclusive `[start, end]`
    /// runs (sorted). Compact yet lossless — the viewer expands these to show
    /// every individual id position.
    pub cell_id_runs: Vec<[i64; 2]>,
    /// The exact surface ids used, run-length encoded as inclusive `[start, end]`.
    pub surface_id_runs: Vec<[i64; 2]>,
    /// Distinct, sorted material numbers referenced by this filler's cells.
    pub materials: Vec<i64>,
    /// Names of the envelopes this filler fills.
    pub envelopes: Vec<EnvelopeName>,
    /// Arbitrary, project-defined metadata (any keys the user chose to record).
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: Metadata,
}

/// The complete build-report manifest.
#[derive(Debug, Serialize)]
pub struct BuildReport {
    pub schema_version: u32,
    pub config_path: String,
    pub gitronics_version: &'static str,
    pub commit_hash: String,
    pub date_time: String,
    /// Total cells in the assembled model (envelope + all fillers combined).
    pub total_cells: usize,
    /// Total surfaces in the assembled model.
    pub total_surfaces: usize,
    pub envelope_entries: Vec<EnvelopeEntry>,
    pub filler_entries: Vec<FillerEntry>,
    pub materials: Vec<String>,
    pub tallies: Vec<String>,
    pub transforms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

// ─── Rendering ────────────────────────────────────────────────────────────────

impl BuildReport {
    /// Serialises the manifest to pretty-printed JSON (for `build_report.json`).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|e| format!("{{\"error\":\"failed to serialize build report: {e}\"}}"))
    }

    /// Renders the complete self-contained interactive HTML document.
    pub fn generate_html(&self) -> String {
        let data = escape_json_for_script(&self.to_json());

        let mut out = String::with_capacity(64 * 1024 + data.len());
        out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        out.push_str("  <meta charset=\"UTF-8\">\n");
        out.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        out.push_str("  <title>Build Report — ");
        push_escaped_text(&mut out, &self.config_path);
        out.push_str("</title>\n");
        // Set the theme before first paint to avoid a flash of the wrong theme.
        out.push_str(
            "  <script>try{var t=localStorage.getItem('gitronics-theme')||\
             ((window.matchMedia&&matchMedia('(prefers-color-scheme: dark)').matches)?'dark':'light');\
             document.documentElement.setAttribute('data-theme',t);}catch(e){}</script>\n",
        );
        out.push_str("  <style>\n");
        out.push_str(include_str!("report.css"));
        out.push_str("\n  </style>\n</head>\n<body>\n");
        out.push_str("  <script type=\"application/json\" id=\"report-data\">");
        out.push_str(&data);
        out.push_str("</script>\n");
        out.push_str("  <script>\n");
        out.push_str(include_str!("report.js"));
        out.push_str("\n  </script>\n</body>\n</html>\n");
        out
    }

    /// Writes the HTML report and its machine-readable JSON manifest into
    /// `output_path` (`build_report.html` / `build_report.json`).
    pub fn write(&self, output_path: &Path) -> Result<(), GitronicsError> {
        std::fs::write(output_path.join("build_report.html"), self.generate_html())?;
        std::fs::write(output_path.join("build_report.json"), self.to_json())?;
        Ok(())
    }
}

// ─── Collection ───────────────────────────────────────────────────────────────
//
// Turning the assembled pieces into a report is the reporting module's job, not
// the build pipeline's. `build_model` gathers the inputs into a [`ReportContext`]
// and calls [`emit`]; everything below stays private to this module.

/// The assembled inputs a report is built from, handed over by the build
/// pipeline. Borrows the models mutably only because reading typed cells/surfaces
/// goes through a `migjorn` view.
pub struct ReportContext<'a> {
    pub config_path: &'a Path,
    pub project_manager: &'a ProjectManager,
    pub envelope_structure: &'a Model,
    pub fillers: &'a mut [(FillerName, Model)],
    pub universe_ids: &'a HashMap<FillerName, UniverseId>,
}

/// Collects the report from the assembled pieces and writes both artefacts into
/// the project's output directory. The single entry point the build pipeline
/// calls — it owns collect → render → write.
pub fn emit(ctx: ReportContext) -> Result<(), GitronicsError> {
    let output_path = ctx.project_manager.output_path().clone();
    let report = collect(ctx)?;
    report.write(&output_path)
}

/// Assembles a [`BuildReport`] from the build inputs.
fn collect(ctx: ReportContext) -> Result<BuildReport, GitronicsError> {
    let ReportContext {
        config_path,
        project_manager,
        envelope_structure,
        fillers,
        universe_ids,
    } = ctx;

    // Cell/surface counts come from the `*_slots` accessors (one slot per card),
    // which read without a `&mut` view.
    let total_cells = envelope_structure.cell_slots().len()
        + fillers
            .iter()
            .map(|(_, m)| m.cell_slots().len())
            .sum::<usize>();
    let total_surfaces = envelope_structure.surface_slots().len()
        + fillers
            .iter()
            .map(|(_, m)| m.surface_slots().len())
            .sum::<usize>();

    let envelope_entries: Vec<EnvelopeEntry> = project_manager
        .envelopes_in_config()
        .map(|env_name| {
            let filler_name: Option<FillerName> = project_manager
                .filler_by_envelope(env_name)
                .and_then(|opt| opt.clone());

            let universe_id = filler_name
                .as_ref()
                .and_then(|f| universe_ids.get(f))
                .copied();

            let transform = filler_name.as_ref().and_then(|f| {
                project_manager
                    .transformation(f, env_name)
                    .ok()
                    .flatten()
                    .map(str::to_string)
            });

            let meta = project_manager
                .envelope_metadata(env_name)
                .cloned()
                .unwrap_or_default();
            EnvelopeEntry {
                envelope_name: env_name.clone(),
                filler_name,
                universe_id,
                transform,
                metadata: meta,
            }
        })
        .collect();

    let mut filler_envelope_counts: HashMap<FillerName, usize> = HashMap::new();
    let mut filler_envelopes: HashMap<FillerName, Vec<EnvelopeName>> = HashMap::new();
    for entry in &envelope_entries {
        if let Some(filler_name) = &entry.filler_name {
            *filler_envelope_counts
                .entry(filler_name.clone())
                .or_insert(0) += 1;
            filler_envelopes
                .entry(filler_name.clone())
                .or_default()
                .push(entry.envelope_name.clone());
        }
    }

    let filler_entries: Vec<FillerEntry> = fillers
        .iter_mut()
        .filter_map(|(name, model)| {
            let universe_id = *universe_ids.get(name)?;
            let envelope_count = *filler_envelope_counts.get(name).unwrap_or(&0);
            let stats = model_stats(model);
            Some(FillerEntry {
                universe_id,
                envelope_count,
                cell_count: stats.cell_count,
                surface_count: stats.surface_count,
                cell_id_runs: stats.cell_id_runs,
                surface_id_runs: stats.surface_id_runs,
                materials: stats.materials,
                envelopes: filler_envelopes.get(name).cloned().unwrap_or_default(),
                metadata: project_manager
                    .filler_metadata(name)
                    .cloned()
                    .unwrap_or_default(),
                name: name.clone(),
            })
        })
        .collect();

    Ok(BuildReport {
        schema_version: SCHEMA_VERSION,
        config_path: config_path.display().to_string(),
        gitronics_version: env!("CARGO_PKG_VERSION"),
        commit_hash: get_hash_of_project(project_dir(config_path)),
        date_time: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        total_cells,
        total_surfaces,
        envelope_entries,
        filler_entries,
        materials: project_manager
            .materials_names()
            .iter()
            .map(|n| n.to_string())
            .collect(),
        tallies: project_manager
            .tallies_names()
            .iter()
            .map(|n| n.to_string())
            .collect(),
        transforms: project_manager
            .transforms_names()
            .iter()
            .map(|n| n.to_string())
            .collect(),
        source: project_manager.source_name().map(|n| n.to_string()),
    })
}

/// Per-model statistics derived directly from a parsed [`Model`] (reliable,
/// independent of any metadata sidecar). Shared with the project report.
pub(crate) struct ModelStats {
    pub(crate) cell_count: usize,
    pub(crate) surface_count: usize,
    /// Exact cell ids used, run-length encoded as inclusive `[start, end]` runs.
    pub(crate) cell_id_runs: Vec<[i64; 2]>,
    /// Exact surface ids used, run-length encoded as inclusive `[start, end]`.
    pub(crate) surface_id_runs: Vec<[i64; 2]>,
    pub(crate) materials: Vec<i64>,
}

/// Coalesces a set of ids into sorted, inclusive `[start, end]` runs of
/// consecutive values (a compact, lossless encoding of the exact id positions).
fn runs_from_ids(mut ids: Vec<i64>) -> Vec<[i64; 2]> {
    ids.sort_unstable();
    ids.dedup();
    let mut runs: Vec<[i64; 2]> = Vec::new();
    for id in ids {
        match runs.last_mut() {
            Some(last) if id == last[1] + 1 => last[1] = id,
            _ => runs.push([id, id]),
        }
    }
    runs
}

/// Computes cell/surface counts, exact id runs and the distinct material set of
/// a model in a single pass over its cells and surfaces.
pub(crate) fn model_stats(model: &mut Model) -> ModelStats {
    let view = model.view();
    let mut cell_ids = Vec::new();
    let mut materials: BTreeSet<i64> = BTreeSet::new();
    for cell in view.cells() {
        cell_ids.push(cell.id);
        if let Some(m) = cell.material
            && m != 0
        {
            materials.insert(m);
        }
    }

    let surface_ids: Vec<i64> = view.surfaces().map(|s| s.id).collect();

    ModelStats {
        cell_count: cell_ids.len(),
        surface_count: surface_ids.len(),
        cell_id_runs: runs_from_ids(cell_ids),
        surface_id_runs: runs_from_ids(surface_ids),
        materials: materials.into_iter().collect(),
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Makes a JSON string safe to embed inside a `<script type="application/json">`
/// element. Escaping `<`, `>` and `&` as `\uXXXX` keeps the payload valid JSON
/// while preventing any `</script>` breakout (an HTML-injection vector).
/// Also escapes the JS line separators U+2028/U+2029.
pub(crate) fn escape_json_for_script(json: &str) -> String {
    let mut out = String::with_capacity(json.len() + 16);
    for ch in json.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c => out.push(c),
        }
    }
    out
}

/// Appends `s` to `out` with HTML-special characters escaped (used for the
/// document `<title>` only; all dynamic content otherwise lives in JSON).
fn push_escaped_text(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Builds a free-form metadata map from `(key, json-value)` pairs.
    fn meta(pairs: &[(&str, serde_json::Value)]) -> Metadata {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// A minimal report: one filled envelope with a transform, one filled
    /// envelope without, one null envelope, and one filler used by two envelopes.
    fn sample_report() -> BuildReport {
        BuildReport {
            schema_version: SCHEMA_VERSION,
            config_path: "project/config.yaml".to_string(),
            gitronics_version: "1.2.3",
            commit_hash: "abc1234".to_string(),
            date_time: "2026-01-01 00:00:00 UTC".to_string(),
            total_cells: 500,
            total_surfaces: 800,
            envelope_entries: vec![
                EnvelopeEntry {
                    envelope_name: EnvelopeName::new("env_a"),
                    filler_name: Some(FillerName::new("universe_101")),
                    universe_id: Some(UniverseId::new(101)),
                    transform: Some("TR1".to_string()),
                    metadata: meta(&[
                        ("description", json!("Blanket A")),
                        ("zone", json!("Tokamak")),
                        ("sector", json!("1")),
                    ]),
                },
                EnvelopeEntry {
                    envelope_name: EnvelopeName::new("env_b"),
                    filler_name: Some(FillerName::new("universe_101")),
                    universe_id: Some(UniverseId::new(101)),
                    transform: None,
                    metadata: meta(&[("zone", json!("Tokamak")), ("sector", json!("1"))]),
                },
                EnvelopeEntry {
                    envelope_name: EnvelopeName::new("env_null"),
                    filler_name: None,
                    universe_id: None,
                    transform: None,
                    metadata: Metadata::new(),
                },
            ],
            filler_entries: vec![FillerEntry {
                name: FillerName::new("universe_101"),
                universe_id: UniverseId::new(101),
                envelope_count: 2,
                cell_count: 120,
                surface_count: 200,
                cell_id_runs: vec![[250000, 250041], [250100, 250178]],
                surface_id_runs: vec![[250000, 250199]],
                materials: vec![110, 907],
                envelopes: vec![EnvelopeName::new("env_a"), EnvelopeName::new("env_b")],
                metadata: meta(&[
                    ("description", json!("Central solenoid")),
                    ("pbs", json!("11")),
                ]),
            }],
            materials: vec!["all_materials.mat".to_string()],
            tallies: vec!["neutron_flux.tally".to_string()],
            transforms: vec![],
            source: Some("plasma.source".to_string()),
        }
    }

    // ── HTML shell ────────────────────────────────────────────────────────────

    #[test]
    fn html_is_valid_document() {
        let html = sample_report().generate_html();
        assert!(html.starts_with("<!DOCTYPE html>"), "missing doctype");
        assert!(html.contains("<html"), "missing <html>");
        assert!(html.contains("</html>"), "missing </html>");
        assert!(html.contains("</body>"), "missing </body>");
    }

    #[test]
    fn html_embeds_data_and_assets() {
        let html = sample_report().generate_html();
        assert!(
            html.contains("id=\"report-data\""),
            "missing embedded data block"
        );
        // CSS and JS assets are inlined.
        assert!(html.contains("--accent"), "stylesheet not inlined");
        assert!(html.contains("report-data"), "viewer script not inlined");
    }

    #[test]
    fn title_contains_config_path() {
        let html = sample_report().generate_html();
        assert!(
            html.contains("project/config.yaml"),
            "config path missing from title"
        );
    }

    #[test]
    fn write_emits_both_report_files() {
        let dir = tempfile::tempdir().unwrap();
        sample_report().write(dir.path()).expect("write report");

        let html_path = dir.path().join("build_report.html");
        let json_path = dir.path().join("build_report.json");
        assert!(html_path.exists(), "build_report.html not written");
        assert!(json_path.exists(), "build_report.json not written");

        // The JSON is a valid manifest and the HTML is a full document.
        let json = std::fs::read_to_string(&json_path).unwrap();
        serde_json::from_str::<serde_json::Value>(&json).expect("valid JSON manifest");
        let html = std::fs::read_to_string(&html_path).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"), "HTML not a document");
    }

    // ── JSON manifest ─────────────────────────────────────────────────────────

    #[test]
    fn json_is_valid_and_roundtrips() {
        let json = sample_report().to_json();
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(value["schema_version"], SCHEMA_VERSION);
        assert_eq!(value["total_cells"], 500);
        assert_eq!(value["total_surfaces"], 800);
        assert_eq!(value["envelope_entries"].as_array().unwrap().len(), 3);
        assert_eq!(value["filler_entries"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn json_serializes_domain_ids_transparently() {
        let json = sample_report().to_json();
        // FillerName -> string, UniverseId -> number.
        assert!(json.contains("\"universe_101\""), "filler name missing");
        assert!(
            json.contains("\"universe_id\": 101"),
            "universe id not numeric"
        );
    }

    #[test]
    fn json_carries_arbitrary_metadata_verbatim() {
        let value: serde_json::Value = serde_json::from_str(&sample_report().to_json()).unwrap();
        let filler = &value["filler_entries"][0];
        assert_eq!(filler["metadata"]["pbs"], "11");
        assert_eq!(filler["metadata"]["description"], "Central solenoid");

        let env = &value["envelope_entries"][0];
        assert_eq!(env["metadata"]["zone"], "Tokamak");
        assert_eq!(env["metadata"]["sector"], "1");
        assert_eq!(env["metadata"]["description"], "Blanket A");
    }

    #[test]
    fn json_encodes_exact_id_runs() {
        let value: serde_json::Value = serde_json::from_str(&sample_report().to_json()).unwrap();
        let filler = &value["filler_entries"][0];
        assert_eq!(filler["cell_id_runs"][0][0], 250000);
        assert_eq!(filler["cell_id_runs"][0][1], 250041);
        assert_eq!(filler["cell_id_runs"][1][0], 250100);
        assert_eq!(filler["surface_id_runs"][0][1], 250199);
        assert_eq!(filler["materials"][0], 110);
        assert_eq!(filler["envelopes"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn null_envelope_omits_optional_fields() {
        let value: serde_json::Value = serde_json::from_str(&sample_report().to_json()).unwrap();
        let null_env = &value["envelope_entries"][2];
        assert_eq!(null_env["envelope_name"], "env_null");
        assert!(
            null_env.get("filler_name").is_none(),
            "null env must omit filler"
        );
        assert!(null_env.get("universe_id").is_none());
        assert!(
            null_env.get("metadata").is_none(),
            "empty metadata must be omitted"
        );
    }

    #[test]
    fn empty_data_sections_serialize_as_arrays() {
        let value: serde_json::Value = serde_json::from_str(&sample_report().to_json()).unwrap();
        assert!(value["transforms"].as_array().unwrap().is_empty());
        assert_eq!(value["source"], "plasma.source");
    }

    #[test]
    fn source_omitted_when_none() {
        let mut report = sample_report();
        report.source = None;
        let value: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
        assert!(value.get("source").is_none(), "source should be omitted");
    }

    // ── Injection safety ──────────────────────────────────────────────────────

    #[test]
    fn script_breakout_is_neutralised_in_html() {
        let mut report = sample_report();
        report.filler_entries[0].name = FillerName::new("</script><script>alert(1)</script>");
        let html = report.generate_html();
        // The literal closing tag must never appear inside the data block.
        assert!(
            !html.contains("</script><script>alert(1)"),
            "script breakout not neutralised"
        );
        // The angle brackets are unicode-escaped instead.
        assert!(html.contains("\\u003c"), "angle brackets not escaped");
    }

    #[test]
    fn embedded_json_data_block_has_no_raw_angle_brackets() {
        let report = sample_report();
        let html = report.generate_html();
        let start = html.find("id=\"report-data\">").unwrap() + "id=\"report-data\">".len();
        let end = html[start..].find("</script>").unwrap() + start;
        let block = &html[start..end];
        assert!(!block.contains('<'), "data block contains raw '<'");
        assert!(!block.contains('>'), "data block contains raw '>'");
    }
}
