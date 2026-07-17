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

use crate::types::{EnvelopeName, FillerName, UniverseId};

/// Schema version of the emitted manifest. Bump on breaking changes so the
/// viewer (and downstream tooling) can adapt.
pub const SCHEMA_VERSION: u32 = 1;

// ─── Public data types ────────────────────────────────────────────────────────

/// An inclusive `[min, max]` id range.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct IdRange {
    pub min: i64,
    pub max: i64,
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
}

/// One filler model in the assembled model.
#[derive(Debug, Serialize)]
pub struct FillerEntry {
    pub name: FillerName,
    pub universe_id: UniverseId,
    pub envelope_count: usize,
    pub cell_count: usize,
    pub surface_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pbs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_id_range: Option<IdRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_id_range: Option<IdRange>,
    /// Distinct, sorted material numbers referenced by this filler's cells.
    pub materials: Vec<i64>,
    /// Names of the envelopes this filler fills.
    pub envelopes: Vec<EnvelopeName>,
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
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Makes a JSON string safe to embed inside a `<script type="application/json">`
/// element. Escaping `<`, `>` and `&` as `\uXXXX` keeps the payload valid JSON
/// while preventing any `</script>` breakout (an HTML-injection vector).
/// Also escapes the JS line separators U+2028/U+2029.
fn escape_json_for_script(json: &str) -> String {
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
                    description: Some("Blanket A".to_string()),
                    zone: Some("Tokamak".to_string()),
                    sector: Some("1".to_string()),
                },
                EnvelopeEntry {
                    envelope_name: EnvelopeName::new("env_b"),
                    filler_name: Some(FillerName::new("universe_101")),
                    universe_id: Some(UniverseId::new(101)),
                    transform: None,
                    description: None,
                    zone: Some("Tokamak".to_string()),
                    sector: Some("1".to_string()),
                },
                EnvelopeEntry {
                    envelope_name: EnvelopeName::new("env_null"),
                    filler_name: None,
                    universe_id: None,
                    transform: None,
                    description: None,
                    zone: None,
                    sector: None,
                },
            ],
            filler_entries: vec![FillerEntry {
                name: FillerName::new("universe_101"),
                universe_id: UniverseId::new(101),
                envelope_count: 2,
                cell_count: 120,
                surface_count: 200,
                description: Some("Central solenoid".to_string()),
                pbs: Some("11".to_string()),
                cell_id_range: Some(IdRange {
                    min: 250000,
                    max: 250041,
                }),
                surface_id_range: Some(IdRange {
                    min: 250000,
                    max: 250129,
                }),
                materials: vec![110, 907],
                envelopes: vec![EnvelopeName::new("env_a"), EnvelopeName::new("env_b")],
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
    fn json_includes_enriched_metadata() {
        let value: serde_json::Value = serde_json::from_str(&sample_report().to_json()).unwrap();
        let filler = &value["filler_entries"][0];
        assert_eq!(filler["pbs"], "11");
        assert_eq!(filler["description"], "Central solenoid");
        assert_eq!(filler["cell_id_range"]["min"], 250000);
        assert_eq!(filler["cell_id_range"]["max"], 250041);
        assert_eq!(filler["materials"][0], 110);
        assert_eq!(filler["envelopes"].as_array().unwrap().len(), 2);

        let env = &value["envelope_entries"][0];
        assert_eq!(env["zone"], "Tokamak");
        assert_eq!(env["sector"], "1");
        assert_eq!(env["description"], "Blanket A");
    }

    #[test]
    fn null_envelope_omits_filler_fields() {
        let value: serde_json::Value = serde_json::from_str(&sample_report().to_json()).unwrap();
        let null_env = &value["envelope_entries"][2];
        assert_eq!(null_env["envelope_name"], "env_null");
        assert!(
            null_env.get("filler_name").is_none(),
            "null env must omit filler"
        );
        assert!(null_env.get("universe_id").is_none());
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
