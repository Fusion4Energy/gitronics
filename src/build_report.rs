//! HTML build-report generator.
//!
//! Produces a single self-contained `build_report.html` file that summarises
//! what was assembled: envelope/filler assignments, filler model stats, and
//! every data-card file that was included.
//!
//! The report uses an internal stylesheet and vanilla JS for the per-section
//! search bars. All sections are collapsible via native `<details>`.

use std::fmt::Write;

use crate::types::{EnvelopeName, FillerName, UniverseId};

// ─── Public data types ────────────────────────────────────────────────────────

/// One row in the Envelope Assignments table.
pub struct EnvelopeEntry {
    pub envelope_name: EnvelopeName,
    /// `None` means the envelope was explicitly set to `null` in the config.
    pub filler_name: Option<FillerName>,
    pub universe_id: Option<UniverseId>,
    /// Raw transform text (e.g. `TR1`, `*TR2 …`) or empty.
    pub transform: Option<String>,
}

/// One row in the Filler Models table.
pub struct FillerEntry {
    pub name: FillerName,
    pub universe_id: UniverseId,
    pub envelope_count: usize,
    pub cell_count: usize,
    pub surface_count: usize,
}

/// All data needed to render a build report.
pub struct BuildReport {
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
    pub source: Option<String>,
}

// ─── HTML generation ──────────────────────────────────────────────────────────

impl BuildReport {
    /// Renders the complete HTML document as a `String`.
    pub fn generate_html(&self) -> String {
        let mut out = String::with_capacity(512 * 1024);

        let n_envelopes = self.envelope_entries.len();
        let n_filled = self
            .envelope_entries
            .iter()
            .filter(|e| e.filler_name.is_some())
            .count();
        let n_null = n_envelopes - n_filled;
        let n_fillers = self.filler_entries.len();
        let n_data_files = self.materials.len()
            + self.tallies.len()
            + self.transforms.len()
            + usize::from(self.source.is_some());

        self.write_head(&mut out);
        self.write_banner(&mut out);
        self.write_stat_cards(
            &mut out,
            n_envelopes,
            n_filled,
            n_null,
            n_fillers,
            n_data_files,
        );
        self.write_main(&mut out);
        out.push_str("</body>\n</html>\n");
        out
    }

    // ── Head ──────────────────────────────────────────────────────────────────

    fn write_head(&self, out: &mut String) {
        out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        out.push_str("  <meta charset=\"UTF-8\">\n");
        out.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        writeln!(
            out,
            "  <title>Build Report — {}</title>",
            h(&self.config_path)
        )
        .unwrap();
        out.push_str("  <style>\n");
        out.push_str(include_str!("report.css"));
        out.push_str("  </style>\n");
        out.push_str("</head>\n<body class=\"bg-slate-50 min-h-screen font-sans text-slate-900 antialiased\">\n\n");
    }

    // ── Banner ────────────────────────────────────────────────────────────────

    fn write_banner(&self, out: &mut String) {
        out.push_str(
            "<header class=\"bg-slate-900 text-white\">\n\
             <div class=\"max-w-7xl mx-auto px-6 py-8\">\n\
             <div class=\"flex flex-wrap items-center gap-3 mb-5\">\n",
        );
        out.push_str(
            "  <span class=\"text-xl font-bold tracking-tight text-white\">gitronics</span>\n",
        );
        writeln!(
            out,
            "  <span class=\"font-mono text-slate-400 text-sm\">v{}</span>",
            h(self.gitronics_version)
        )
        .unwrap();
        out.push_str(
            "  <span class=\"ml-1 px-3 py-1 bg-blue-600 text-white text-xs rounded-full font-medium\">Build Report</span>\n",
        );
        out.push_str("</div>\n");

        out.push_str(
            "<dl class=\"grid grid-cols-1 sm:grid-cols-2 gap-x-16 gap-y-1 text-sm font-mono\">\n",
        );
        for (label, value) in [
            ("config", &self.config_path),
            ("commit", &self.commit_hash),
            ("date", &self.date_time),
        ] {
            writeln!(
                out,
                "<div class=\"flex gap-2\"><dt class=\"text-slate-500\">{label}</dt><dd class=\"text-slate-300 truncate\">{value}</dd></div>",
                label = h(label),
                value = h(value),
            )
            .unwrap();
        }
        writeln!(
            out,
            "<div class=\"flex gap-2\"><dt class=\"text-slate-500\">version</dt><dd class=\"text-slate-300\">v{}</dd></div>",
            h(self.gitronics_version)
        )
        .unwrap();
        out.push_str("</dl>\n");
        out.push_str("</div>\n</header>\n\n");
    }

    // ── Stat cards ────────────────────────────────────────────────────────────

    fn write_stat_cards(
        &self,
        out: &mut String,
        n_envelopes: usize,
        n_filled: usize,
        n_null: usize,
        n_fillers: usize,
        n_data_files: usize,
    ) {
        out.push_str(
            "<div class=\"max-w-7xl mx-auto px-6 py-6\">\n\
             <div class=\"grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4\">\n",
        );

        stat_card(out, "Cells", &self.total_cells.to_string(), None);
        stat_card(out, "Surfaces", &self.total_surfaces.to_string(), None);
        stat_card(
            out,
            "Envelopes",
            &n_envelopes.to_string(),
            Some(&format!("{n_filled} filled · {n_null} null")),
        );
        stat_card(out, "Filler Models", &n_fillers.to_string(), None);
        stat_card(out, "Data Files", &n_data_files.to_string(), None);

        out.push_str("</div>\n</div>\n\n");
    }

    // ── Main content ──────────────────────────────────────────────────────────

    fn write_main(&self, out: &mut String) {
        out.push_str("<main class=\"max-w-7xl mx-auto px-6 pb-12 space-y-4\">\n\n");

        self.write_envelope_section(out);
        self.write_fillers_section(out);

        if !self.materials.is_empty() {
            write_list_section(out, "Materials", "materials", &self.materials);
        }
        if !self.tallies.is_empty() {
            write_list_section(out, "Tallies", "tallies", &self.tallies);
        }
        if !self.transforms.is_empty() {
            write_list_section(out, "Transforms", "transforms", &self.transforms);
        }
        if let Some(ref src) = self.source {
            write_source_section(out, src);
        }

        out.push_str("</main>\n\n");
        write_footer_and_scripts(out);
    }

    // ── Envelope section ──────────────────────────────────────────────────────

    fn write_envelope_section(&self, out: &mut String) {
        let n = self.envelope_entries.len();
        let n_filled = self
            .envelope_entries
            .iter()
            .filter(|e| e.filler_name.is_some())
            .count();
        let n_null = n - n_filled;

        // Section open
        out.push_str(
            "<details open class=\"bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden\">\n",
        );

        // Summary row
        out.push_str(
            "<summary class=\"flex flex-wrap items-center gap-2 px-6 py-4 hover:bg-slate-50 select-none\">\n\
             <span class=\"chevron text-slate-400\">&#9654;</span>\n\
             <span class=\"font-semibold text-slate-900\">Envelope Assignments</span>\n",
        );
        writeln!(
            out,
            "<span class=\"px-2 py-0.5 bg-slate-100 text-slate-600 text-xs rounded-full font-medium\">{n}</span>"
        )
        .unwrap();
        writeln!(
            out,
            "<span class=\"px-2 py-0.5 bg-emerald-100 text-emerald-700 text-xs rounded-full font-medium\">{n_filled} filled</span>"
        )
        .unwrap();
        if n_null > 0 {
            writeln!(
                out,
                "<span class=\"px-2 py-0.5 bg-slate-100 text-slate-500 text-xs rounded-full font-medium\">{n_null} null</span>"
            )
            .unwrap();
        }
        out.push_str("</summary>\n");

        // Search bar
        out.push_str(
            "<div class=\"border-t border-slate-100 px-4 py-3 bg-slate-50\">\n\
             <input type=\"text\"\n\
             oninput=\"filterRows(this,'envelopes-tbody')\"\n\
             placeholder=\"Filter by envelope, filler, universe ID or transform…\"\n\
             class=\"w-full px-3 py-1.5 text-sm border border-slate-200 rounded-lg bg-white \
             focus:outline-none focus:ring-2 focus:ring-blue-500\">\n\
             </div>\n",
        );

        // Table
        out.push_str(
            "<div class=\"overflow-x-auto\">\n\
             <table class=\"w-full text-sm\">\n\
             <thead>\n\
             <tr class=\"bg-slate-50 border-y border-slate-200\">\n\
             <th class=\"text-left px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide w-1/4\">Envelope</th>\n\
             <th class=\"text-left px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide w-1/4\">Filler</th>\n\
             <th class=\"text-left px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide w-17\">Universe ID</th>\n\
             <th class=\"text-left px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide\">Transform</th>\n\
             </tr>\n\
             </thead>\n\
             <tbody id=\"envelopes-tbody\" class=\"divide-y divide-slate-100\">\n",
        );

        for entry in &self.envelope_entries {
            let env = h(&entry.envelope_name.to_string());
            let filler_str = entry
                .filler_name
                .as_ref()
                .map(|f| f.to_string())
                .unwrap_or_default();
            let filler_esc = h(&filler_str);
            let uid_str = entry.universe_id.map(|u| u.to_string()).unwrap_or_default();
            let transform_str = entry.transform.as_deref().unwrap_or("").to_string();
            let transform_esc = h(&transform_str);

            let search_raw = format!("{env} {filler_esc} {uid_str} {transform_esc}").to_lowercase();
            let search_esc = h(&search_raw);

            if entry.filler_name.is_some() {
                let transform_disp = if transform_str.is_empty() {
                    "<span class=\"text-slate-300\">—</span>".to_string()
                } else {
                    format!(
                        "<code class=\"text-xs bg-slate-100 px-1 py-0.5 rounded\">{transform_esc}</code>"
                    )
                };
                write!(
                    out,
                    "<tr data-search=\"{search_esc}\" class=\"hover:bg-emerald-50/40\">\n\
                     <td class=\"px-4 py-2.5 font-mono text-slate-700\">{env}</td>\n\
                     <td class=\"px-4 py-2.5 font-mono text-emerald-700 font-medium\">{filler_esc}</td>\n\
                     <td class=\"px-4 py-2.5 font-mono text-slate-500 text-xs\">{uid_str}</td>\n\
                     <td class=\"px-4 py-2.5\">{transform_disp}</td>\n\
                     </tr>\n"
                )
                .unwrap();
            } else {
                write!(
                    out,
                    "<tr data-search=\"{search_esc}\" class=\"hover:bg-slate-50\">\n\
                     <td class=\"px-4 py-2.5 font-mono text-slate-400\">{env}</td>\n\
                     <td class=\"px-4 py-2.5 text-xs text-slate-400 italic\">null</td>\n\
                     <td class=\"px-4 py-2.5 text-slate-300\">—</td>\n\
                     <td class=\"px-4 py-2.5 text-slate-300\">—</td>\n\
                     </tr>\n"
                )
                .unwrap();
            }
        }

        out.push_str(
            "</tbody>\n</table>\n</div>\n\
             </details>\n\n",
        );
    }

    // ── Fillers section ───────────────────────────────────────────────────────

    fn write_fillers_section(&self, out: &mut String) {
        let n = self.filler_entries.len();

        out.push_str(
            "<details open class=\"bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden\">\n",
        );
        out.push_str(
            "<summary class=\"flex flex-wrap items-center gap-2 px-6 py-4 hover:bg-slate-50 select-none\">\n\
             <span class=\"chevron text-slate-400\">&#9654;</span>\n\
             <span class=\"font-semibold text-slate-900\">Filler Models</span>\n",
        );
        writeln!(
            out,
            "<span class=\"px-2 py-0.5 bg-slate-100 text-slate-600 text-xs rounded-full font-medium\">{n}</span>"
        )
        .unwrap();
        out.push_str("</summary>\n");

        out.push_str(
            "<div class=\"border-t border-slate-100 px-4 py-3 bg-slate-50\">\n\
             <input type=\"text\"\n\
             oninput=\"filterRows(this,'fillers-tbody')\"\n\
             placeholder=\"Filter by filler name or universe ID…\"\n\
             class=\"w-full px-3 py-1.5 text-sm border border-slate-200 rounded-lg bg-white \
             focus:outline-none focus:ring-2 focus:ring-blue-500\">\n\
             </div>\n",
        );

        out.push_str(
            "<div class=\"overflow-x-auto\">\n\
             <table class=\"w-full text-sm\">\n\
             <thead>\n\
             <tr class=\"bg-slate-50 border-y border-slate-200\">\n\
             <th class=\"text-left px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide\">Filler</th>\n\
             <th class=\"text-right px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide w-25\">Universe ID</th>\n\
             <th class=\"text-right px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide w-24\">Envelopes</th>\n\
             <th class=\"text-right px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide w-20\">Cells</th>\n\
             <th class=\"text-right px-4 py-2 font-medium text-slate-500 text-xs uppercase tracking-wide w-24\">Surfaces</th>\n\
             </tr>\n\
             </thead>\n\
             <tbody id=\"fillers-tbody\" class=\"divide-y divide-slate-100\">\n",
        );

        for entry in &self.filler_entries {
            let name = h(&entry.name.to_string());
            let uid = entry.universe_id.to_string();
            let search_raw = format!("{name} {uid} {}", entry.envelope_count).to_lowercase();
            let search_esc = h(&search_raw);

            write!(
                out,
                "<tr data-search=\"{search_esc}\" class=\"hover:bg-blue-50/30\">\n\
                 <td class=\"px-4 py-2.5 font-mono text-slate-700\">{name}</td>\n\
                 <td class=\"px-4 py-2.5 text-right tabular-nums text-slate-600\">{uid}</td>\n\
                 <td class=\"px-4 py-2.5 text-right tabular-nums text-slate-600\">{envelopes}</td>\n\
                 <td class=\"px-4 py-2.5 text-right tabular-nums text-slate-600\">{cells}</td>\n\
                 <td class=\"px-4 py-2.5 text-right tabular-nums text-slate-600\">{surfaces}</td>\n\
                 </tr>\n",
                envelopes = entry.envelope_count,
                cells = entry.cell_count,
                surfaces = entry.surface_count,
            )
            .unwrap();
        }

        out.push_str("</tbody>\n</table>\n</div>\n</details>\n\n");
    }
}

// ── Stand-alone section helpers ───────────────────────────────────────────────

fn write_list_section(out: &mut String, title: &str, id: &str, items: &[String]) {
    let n = items.len();
    let tbody_id = format!("{id}-tbody");

    write!(
        out,
        "<details open class=\"bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden\">\n\
         <summary class=\"flex flex-wrap items-center gap-2 px-6 py-4 hover:bg-slate-50 select-none\">\n\
         <span class=\"chevron text-slate-400\">&#9654;</span>\n\
         <span class=\"font-semibold text-slate-900\">{title}</span>\n\
         <span class=\"px-2 py-0.5 bg-slate-100 text-slate-600 text-xs rounded-full font-medium\">{n}</span>\n\
         </summary>\n",
        title = h(title),
    )
    .unwrap();

    write!(
        out,
        "<div class=\"border-t border-slate-100 px-4 py-3 bg-slate-50\">\n\
         <input type=\"text\"\n\
         oninput=\"filterRows(this,'{tbody_id}')\"\n\
         placeholder=\"Filter {title} files…\"\n\
         class=\"w-full px-3 py-1.5 text-sm border border-slate-200 rounded-lg bg-white \
         focus:outline-none focus:ring-2 focus:ring-blue-500\">\n\
         </div>\n\
         <table class=\"w-full text-sm\">\n\
         <tbody id=\"{tbody_id}\" class=\"divide-y divide-slate-100\">\n",
        title = h(title),
    )
    .unwrap();

    for item in items {
        let item_esc = h(item);
        let search_esc = h(&item.to_lowercase());
        write!(
            out,
            "<tr data-search=\"{search_esc}\" class=\"hover:bg-slate-50\">\n\
             <td class=\"px-4 py-2.5 font-mono text-slate-700\">{item_esc}</td>\n\
             </tr>\n"
        )
        .unwrap();
    }

    out.push_str("</tbody>\n</table>\n</details>\n\n");
}

fn write_source_section(out: &mut String, source: &str) {
    let source_esc = h(source);
    write!(
        out,
        "<details open class=\"bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden\">\n\
         <summary class=\"flex items-center gap-2 px-6 py-4 hover:bg-slate-50 select-none\">\n\
         <span class=\"chevron text-slate-400\">&#9654;</span>\n\
         <span class=\"font-semibold text-slate-900\">Source</span>\n\
         </summary>\n\
         <div class=\"border-t border-slate-100 px-6 py-4\">\n\
         <span class=\"font-mono text-sm text-slate-700\">{source_esc}</span>\n\
         </div>\n\
         </details>\n\n"
    )
    .unwrap();
}

fn write_footer_and_scripts(out: &mut String) {
    out.push_str(
        "<footer class=\"max-w-7xl mx-auto px-6 py-8 text-center text-xs text-slate-400\">\n\
         Generated by <a href=\"https://fusion4energy.github.io/gitronics/latest\" \
         class=\"underline hover:text-slate-600\">gitronics</a>\n\
         </footer>\n\n",
    );

    // Tiny vanilla-JS filter: hides rows whose data-search doesn't contain the query.
    out.push_str(
        "<script>\n\
         function filterRows(input, tbodyId) {\n\
         var q = input.value.toLowerCase();\n\
         document.getElementById(tbodyId).querySelectorAll('tr').forEach(function(row) {\n\
         var text = (row.getAttribute('data-search') || '').toLowerCase();\n\
         row.style.display = text.includes(q) ? '' : 'none';\n\
         });\n\
         }\n\
         </script>\n",
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Stat card widget.
fn stat_card(out: &mut String, label: &str, value: &str, sub: Option<&str>) {
    let sub_html = sub.map_or(String::new(), |s| {
        format!("<p class=\"text-xs text-slate-400 mt-0.5\">{}</p>", h(s))
    });
    write!(
        out,
        "<div class=\"bg-white rounded-xl border border-slate-200 shadow-sm p-4\">\n\
         <p class=\"text-2xl font-bold text-slate-900\">{value}</p>\n\
         <p class=\"text-xs font-medium text-slate-500 uppercase tracking-wide mt-1\">{label}</p>\n\
         {sub_html}\n\
         </div>\n",
        value = h(value),
        label = h(label),
    )
    .unwrap();
}

/// Minimal HTML escaping — prevents XSS and broken markup.
fn h(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal `BuildReport` with one filled envelope, one null envelope,
    /// and one filler model used by two envelopes.
    fn sample_report() -> BuildReport {
        BuildReport {
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
                },
                EnvelopeEntry {
                    envelope_name: EnvelopeName::new("env_b"),
                    filler_name: Some(FillerName::new("universe_101")),
                    universe_id: Some(UniverseId::new(101)),
                    transform: None,
                },
                EnvelopeEntry {
                    envelope_name: EnvelopeName::new("env_null"),
                    filler_name: None,
                    universe_id: None,
                    transform: None,
                },
            ],
            filler_entries: vec![FillerEntry {
                name: FillerName::new("universe_101"),
                universe_id: UniverseId::new(101),
                envelope_count: 2,
                cell_count: 120,
                surface_count: 200,
            }],
            materials: vec!["all_materials.mat".to_string()],
            tallies: vec!["neutron_flux.tally".to_string()],
            transforms: vec![],
            source: Some("plasma.source".to_string()),
        }
    }

    // ── Structural ────────────────────────────────────────────────────────────

    #[test]
    fn html_is_valid_document() {
        let html = sample_report().generate_html();
        assert!(html.starts_with("<!DOCTYPE html>"), "missing doctype");
        assert!(html.contains("<html"), "missing <html>");
        assert!(html.contains("</html>"), "missing </html>");
        assert!(html.contains("</body>"), "missing </body>");
    }

    #[test]
    fn title_contains_config_path() {
        let html = sample_report().generate_html();
        assert!(
            html.contains("project/config.yaml"),
            "config path missing from title"
        );
    }

    // ── Stat cards ────────────────────────────────────────────────────────────

    #[test]
    fn stat_cards_show_totals() {
        let html = sample_report().generate_html();
        assert!(html.contains(">500<"), "total cells not rendered");
        assert!(html.contains(">800<"), "total surfaces not rendered");
        assert!(html.contains(">1<"), "filler model count not rendered");
    }

    // ── Envelope table ────────────────────────────────────────────────────────

    #[test]
    fn filled_envelope_shows_filler_and_universe_id() {
        let html = sample_report().generate_html();
        assert!(html.contains("env_a"), "filled envelope name missing");
        assert!(html.contains("universe_101"), "filler name missing");
        assert!(html.contains(">101<"), "universe ID missing");
    }

    #[test]
    fn filled_envelope_shows_transform() {
        let html = sample_report().generate_html();
        assert!(html.contains("TR1"), "transform text missing");
    }

    #[test]
    fn filled_envelope_without_transform_shows_dash() {
        let html = sample_report().generate_html();
        // env_b has no transform — the em-dash placeholder should appear
        assert!(html.contains("—"), "em-dash missing for empty transform");
    }

    #[test]
    fn null_envelope_renders_null_marker() {
        let html = sample_report().generate_html();
        assert!(html.contains("env_null"), "null envelope name missing");
        assert!(html.contains(">null<"), "null marker missing");
    }

    // ── Filler table ──────────────────────────────────────────────────────────

    #[test]
    fn filler_row_shows_envelope_count() {
        let html = sample_report().generate_html();
        // envelope_count = 2 should appear as a right-aligned cell
        assert!(html.contains(">2<"), "filler envelope count missing");
    }

    #[test]
    fn filler_row_shows_cell_and_surface_counts() {
        let html = sample_report().generate_html();
        assert!(html.contains(">120<"), "filler cell count missing");
        assert!(html.contains(">200<"), "filler surface count missing");
    }

    // ── Optional sections ─────────────────────────────────────────────────────

    #[test]
    fn materials_section_present_when_non_empty() {
        let html = sample_report().generate_html();
        assert!(html.contains("all_materials.mat"), "materials file missing");
    }

    #[test]
    fn tallies_section_present_when_non_empty() {
        let html = sample_report().generate_html();
        assert!(html.contains("neutron_flux.tally"), "tally file missing");
    }

    #[test]
    fn transforms_section_absent_when_empty() {
        let html = sample_report().generate_html();
        assert!(
            !html.contains("<span class=\"font-semibold text-slate-900\">Transforms</span>"),
            "transforms section should not appear when empty"
        );
    }

    #[test]
    fn source_section_present_when_some() {
        let html = sample_report().generate_html();
        assert!(html.contains("plasma.source"), "source file missing");
    }

    #[test]
    fn source_section_absent_when_none() {
        let mut report = sample_report();
        report.source = None;
        let html = report.generate_html();
        assert!(
            !html.contains("<span class=\"font-semibold text-slate-900\">Source</span>"),
            "source section should not appear when None"
        );
    }

    // ── HTML escaping ─────────────────────────────────────────────────────────

    #[test]
    fn html_special_chars_in_filler_name_are_escaped() {
        let mut report = sample_report();
        report.filler_entries[0].name = FillerName::new("<script>alert(1)</script>");
        let html = report.generate_html();
        assert!(
            !html.contains("<script>alert(1)"),
            "unescaped XSS payload in filler name"
        );
        assert!(
            html.contains("&lt;script&gt;"),
            "filler name not HTML-escaped"
        );
    }

    #[test]
    fn html_special_chars_in_envelope_name_are_escaped() {
        let mut report = sample_report();
        report.envelope_entries[0].envelope_name = EnvelopeName::new("env&name<test>");
        let html = report.generate_html();
        assert!(!html.contains("env&name"), "raw ampersand in envelope name");
        assert!(html.contains("env&amp;name"), "envelope name not escaped");
    }

    #[test]
    fn html_special_chars_in_config_path_are_escaped() {
        let mut report = sample_report();
        report.config_path = "path/<config>.yaml".to_string();
        let html = report.generate_html();
        assert!(!html.contains("path/<config>"), "raw angle brackets leaked");
        assert!(
            html.contains("path/&lt;config&gt;"),
            "config path not escaped"
        );
    }
}
