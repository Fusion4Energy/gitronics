use crate::build_report::BuildReport;
use crate::error::GitronicsError;
use crate::fs_utils::write_output_gitignore;
use crate::project_manager::ProjectManager;
use crate::runtime::init_thread_pool;
use crate::types::{EnvelopeName, FillerName, UniverseId};

use git2::Repository;
use log::{info, warn};
use migjorn::Model;
use regex::Regex;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::{collections::HashMap, path::Path, sync::LazyLock};

pub fn build_model(config_path: &Path, output_path: &Path) -> Result<(), GitronicsError> {
    init_thread_pool();
    info!(
        "Starting model build process for: {}",
        config_path.display()
    );

    let mut project_manager = ProjectManager::new(config_path, output_path)?;

    // Load all files.
    let mut envelope_structure = project_manager.load_envelope_structure()?;
    // Loads every filler and caches its metadata in the same call, so
    // `transformation` below is guaranteed to see it.
    let unordered_fillers = project_manager.load_fillers_with_metadata()?;
    let transforms = project_manager.load_transforms()?;
    let materials = project_manager.load_materials()?;
    let tallies = project_manager.load_tallies()?;
    let source = project_manager.load_source()?;

    // Order fillers by their first cell id for deterministic output.
    let fillers = order_fillers_by_cell_id(unordered_fillers)?;

    // Load the descriptive envelope-structure metadata (best-effort).
    project_manager.load_envelope_metadata();

    // Universe id of every filler (read from its first cell's `u=`).
    let universe_ids: HashMap<FillerName, UniverseId> = fillers
        .iter()
        .map(|(name, model)| {
            let universe = filler_universe(model)
                .ok_or_else(|| GitronicsError::FirstCellWithoutUniverseID(name.clone()))?;
            Ok((name.clone(), universe))
        })
        .collect::<Result<HashMap<_, _>, GitronicsError>>()?;

    // Insert FILL cards into the placeholder envelope cells.
    info!("Adapting envelope structure with FILL cards");
    add_fill_cards_to_envelopes(&project_manager, &universe_ids, &mut envelope_structure)?;

    // Collect build-report data before the models are consumed by composition.
    let report = BuildReport::from_build(
        config_path,
        get_hash_of_project(project_dir(config_path)),
        &project_manager,
        &envelope_structure,
        &fillers,
        &universe_ids,
    );

    // Compose: drop the ignored data blocks, then merge every filler's geometry
    // and the configured data cards into the envelope structure
    // (collision-checked against the disjoint-range convention).
    info!("Composing model");
    envelope_structure = envelope_structure.clear_data_cards();

    // `into_iter` so each filler's original model — which still carries the data
    // cards `clear_data_cards` drops — is freed as soon as its cleared clone
    // exists, instead of every original staying alive alongside every clone.
    let mut to_merge: Vec<Model> = fillers
        .into_iter()
        .map(|(_, model)| model.clear_data_cards())
        .collect();

    let data_text = [transforms, materials, tallies, source]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if !data_text.is_empty() {
        // Block structure is positional — title line, then the cell, surface and
        // data blocks separated by blank lines — so this parses to a model whose
        // only content cards are the data cards. `merge` appends them at the end
        // of the data block and indexes their material/transform ids, which is
        // exactly what re-parsing the whole assembled source used to buy, for the
        // size of the data cards rather than the size of the whole model.
        to_merge.push(Model::parse(&format!(
            "gitronics data cards\n\n\n{data_text}\n"
        )));
    }

    envelope_structure
        .merge(to_merge)
        .map_err(|conflicts| GitronicsError::MergeConflicts(conflicts.join("\n")))?;

    // Validate the assembled model. `merge` indexed every card it absorbed, so
    // this reads the same ids a re-parse would have built.
    info!("Performing validation checks on the assembled model");
    let problems = envelope_structure.validate();
    if !problems.is_empty() {
        return Err(GitronicsError::ValidationError(problems.join("\n")));
    }

    // Write the assembled model with the provenance banner.
    info!("Writing assembled model to file");
    let assembled_path = project_manager.output_path().join("assembled.mcnp");
    write_assembled(
        &envelope_structure,
        &assembled_path,
        &banner_text(config_path),
    )?;
    write_output_gitignore(project_manager.output_path())?;

    // Write the HTML build report and its machine-readable JSON manifest.
    info!("Writing build report");
    let json = report.to_json();
    let report_path = project_manager.output_path().join("build_report.html");
    fs::write(&report_path, report.generate_html_from_json(&json))
        .map_err(|source| GitronicsError::io_path(&report_path, source))?;
    let json_path = project_manager.output_path().join("build_report.json");
    fs::write(&json_path, &json).map_err(|source| GitronicsError::io_path(&json_path, source))?;

    info!(
        "Build completed successfully in: {}",
        assembled_path.display()
    );
    Ok(())
}

static ENVELOPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\s*@env:\s*([[:alnum:]_.-]+)\s*").unwrap());

/// The directory that contains the configuration file — the anchor for git
/// repository discovery (the project, not the process's working directory).
fn project_dir(config_path: &Path) -> &Path {
    config_path.parent().unwrap_or(Path::new("."))
}

/// The universe id declared by a filler's first cell (`u=`), if any.
fn filler_universe(model: &Model) -> Option<UniverseId> {
    model
        .cells()
        .next()?
        .universe()
        .map(|u| UniverseId::new(u as u32))
}

/// Order fillers by the id of their first cell, erroring on a filler with no
/// cells.
fn order_fillers_by_cell_id(
    fillers: Vec<(FillerName, Model)>,
) -> Result<Vec<(FillerName, Model)>, GitronicsError> {
    let mut keyed: Vec<(i64, FillerName, Model)> = fillers
        .into_iter()
        .map(|(name, model)| {
            let cell_id = model
                .cells()
                .next()
                .and_then(|c| c.id())
                .ok_or_else(|| GitronicsError::NoCellID(name.clone()))?;
            Ok((cell_id, name, model))
        })
        .collect::<Result<Vec<_>, GitronicsError>>()?;
    keyed.sort_by_key(|(cell_id, _, _)| *cell_id);
    Ok(keyed
        .into_iter()
        .map(|(_, name, model)| (name, model))
        .collect())
}

fn add_fill_cards_to_envelopes(
    project_manager: &ProjectManager,
    universe_ids: &HashMap<FillerName, UniverseId>,
    envelope_structure: &mut Model,
) -> Result<(), GitronicsError> {
    let mut missing_envelopes_in_file: HashSet<EnvelopeName> =
        project_manager.envelopes_in_config().cloned().collect();

    // Collect cell slots up front: FILL insertion is a token splice that leaves
    // slots stable, so we can read then mutate by the same slot.
    let cell_slots: Vec<u32> = envelope_structure.cells().map(|c| c.slot()).collect();

    for slot in cell_slots {
        let Some(view) = envelope_structure.cell_at(slot) else {
            continue;
        };
        let original_text = view.text().to_owned();
        let Some(caps) = ENVELOPE_RE.captures(&original_text) else {
            continue;
        };
        let envelope_name =
            EnvelopeName::new(caps.get(1).map(|m| m.as_str()).ok_or_else(|| {
                GitronicsError::FailedToExtractEnvelopeName(
                    ENVELOPE_RE.to_string(),
                    original_text.clone(),
                )
            })?);

        let Some(env_config) = project_manager.filler_by_envelope(&envelope_name) else {
            warn!(
                "The envelope `{envelope_name}` is not considered in the configuration file. \
                 It is better to explicitly set it as `{envelope_name}: null` if you want the \
                 envelope to not be filled with any model."
            );
            continue;
        };

        // We found the envelope in the file.
        missing_envelopes_in_file.remove(&envelope_name);

        // Envelope explicitly set to null in config: leave it unfilled.
        let Some(filler_name) = env_config.as_ref() else {
            continue;
        };

        let universe_id = universe_ids
            .get(filler_name)
            .ok_or_else(|| GitronicsError::FillerUniverseIdMissing(filler_name.clone()))?;

        let transform = project_manager
            .transformation(filler_name, &envelope_name)?
            .unwrap_or("");
        let fill_card_text = if let Some(transform_without_star) = transform.strip_prefix('*') {
            format!("*fill={universe_id} {transform_without_star}")
        } else {
            format!("fill={universe_id} {transform}")
        };

        envelope_structure
            .add_cell_param(slot, fill_card_text.trim())
            .map_err(|e| GitronicsError::InvalidFillCard(fill_card_text, e.to_string()))?;
    }

    if !missing_envelopes_in_file.is_empty() {
        warn!(
            "The following envelopes were defined in the configuration file but not found in the envelope structure file: {}. \
             Please check that the `$ @env:envelope_name` pattern is satisfied.",
            missing_envelopes_in_file
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}

/// The provenance banner: how, when and from what this model was assembled.
/// No trailing newline.
fn banner_text(config_path: &Path) -> String {
    let configuration = config_path.display().to_string();
    let date_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let gitronics_version = env!("CARGO_PKG_VERSION");
    let commit_hash = get_hash_of_project(project_dir(config_path));

    format!(
        "C ============================================================\n\
         C  Built by gitronics v{gitronics_version}\n\
         C  Configuration : {configuration}\n\
         C  Git commit    : {commit_hash}\n\
         C  Date / time   : {date_time}\n\
         C ============================================================"
    )
}

/// Insert the provenance banner as comment lines just after the model's title.
///
/// The build itself streams the model to disk through [`write_assembled`]; this
/// is the same transformation expressed over a whole string, kept so a test can
/// hold the two against each other.
#[cfg(test)]
fn insert_banner(source: &str, banner: &str) -> String {
    match source.split_once('\n') {
        Some((title, rest)) => format!("{title}\n{banner}\n{rest}"),
        None => format!("{source}\n{banner}\n"),
    }
}

/// Write `model`, with `banner` inserted after its title line, to `path`.
///
/// Streams the model card by card into a buffered writer rather than building
/// the whole source in memory. `Cst::to_source` is a plain concatenation of each
/// card's text, so the bytes written here are exactly
/// `insert_banner(&model.to_source(), banner)` — for a 376 MB model that is
/// three whole copies of the output not allocated.
fn write_assembled(model: &Model, path: &Path, banner: &str) -> Result<(), GitronicsError> {
    let file = File::create(path).map_err(|source| GitronicsError::io_path(path, source))?;
    let mut writer = BufWriter::with_capacity(1 << 20, file);

    // Buffer only as far as the first newline, so the split point is the one
    // `insert_banner` would choose on the fully concatenated source.
    let mut cards = model.cst().cards();
    let mut head = String::new();
    let mut newline_at = None;
    for card in cards.by_ref() {
        head.push_str(card.text());
        if let Some(i) = head.find('\n') {
            newline_at = Some(i);
            break;
        }
    }

    match newline_at {
        Some(i) => {
            writer.write_all(&head.as_bytes()[..=i])?; // title line, including '\n'
            writer.write_all(banner.as_bytes())?;
            writer.write_all(b"\n")?;
            writer.write_all(&head.as_bytes()[i + 1..])?; // rest of that card
        }
        // The entire source has no newline: mirrors `insert_banner`'s `None` arm.
        None => {
            writer.write_all(head.as_bytes())?;
            writer.write_all(b"\n")?;
            writer.write_all(banner.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }

    for card in cards {
        writer.write_all(card.text().as_bytes())?;
    }
    writer.flush()?;
    Ok(())
}

/// Describe the git state of the repository that contains `start_dir` (the
/// project being built), not the process's current working directory — so the
/// recorded commit reflects what was actually assembled regardless of where the
/// binary was invoked from.
fn get_hash_of_project(start_dir: &Path) -> String {
    let repo = Repository::discover(start_dir).ok();
    repo.as_ref()
        .and_then(|r| {
            let mut opts = git2::DescribeOptions::new();
            opts.describe_tags(); // Look for tags

            // Configure formatting options (this adds the -dirty suffix automatically!)
            let mut format_opts = git2::DescribeFormatOptions::new();
            format_opts.dirty_suffix("-dirty");

            // Try to describe the current state, fallback to a short hash if no tags exist
            r.describe(&opts)
                .and_then(|format| format.format(Some(&format_opts)))
                .ok()
                .or_else(|| {
                    // Fallback: If the repo has no tags at all, just grab the short SHA
                    let head = r.head().ok()?;
                    let commit = head.peel_to_commit().ok()?;
                    let short_id = commit.as_object().short_id().ok()?;
                    let mut hash = short_id.as_str().map(|s| s.to_string()).unwrap_or_default();

                    // Manually check dirty state for fallback
                    if let Ok(statuses) = r.statuses(None)
                        && !statuses.is_empty()
                    {
                        hash.push_str("-dirty");
                    }
                    Some(hash)
                })
        })
        .unwrap_or_else(|| "GIT repository not found".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ── Banner ────────────────────────────────────────────────────────────────

    #[test]
    fn banner_lands_after_the_title_line() {
        let out = insert_banner("title\ncell cards\n", "C banner");
        assert_eq!(out, "title\nC banner\ncell cards\n");
    }

    #[test]
    fn banner_appends_to_a_source_with_no_newline() {
        assert_eq!(insert_banner("title", "C banner"), "title\nC banner\n");
    }

    #[test]
    fn banner_text_records_version_config_and_time() {
        let banner = banner_text(Path::new("configurations/baseline.yaml"));
        let lines: Vec<&str> = banner.lines().collect();
        assert_eq!(lines.len(), 6, "banner is two rules around four fields");
        assert!(lines[1].starts_with("C  Built by gitronics v"));
        assert!(lines[2].contains("configurations/baseline.yaml"));
        assert!(lines[3].starts_with("C  Git commit    : "));
        assert!(lines[4].starts_with("C  Date / time   : "));
        assert_eq!(lines[0], lines[5], "opening and closing rules match");
    }

    /// `write_assembled` streams what `insert_banner` would have built in
    /// memory. The two must not drift apart.
    #[test]
    fn streamed_output_equals_insert_banner() {
        let dir = tempdir().unwrap();
        let banner = "C banner line 1\nC banner line 2";

        for source in [
            "title\n1 0 -1 imp:n=1\n\n1 SO 5\n\nM1 1001 1\n",
            "just a title with no newline",
            "title\n",
        ] {
            let model = Model::parse(source);
            let path = dir.path().join("out.mcnp");
            write_assembled(&model, &path, banner).unwrap();

            assert_eq!(
                fs::read_to_string(&path).unwrap(),
                insert_banner(&model.to_source(), banner),
                "streamed and in-memory banner insertion differ for {source:?}"
            );
        }
    }

    // ── Filler ordering ───────────────────────────────────────────────────────

    #[test]
    fn fillers_are_ordered_by_first_cell_id() {
        let mk = |id: i64| {
            Model::parse(&format!(
                "t\n{id} 0 -1 imp:n=1 u=1\n\n1 SO 5\n\nM1 1001 1\n"
            ))
        };
        let fillers = vec![
            (FillerName::new("c"), mk(300)),
            (FillerName::new("a"), mk(100)),
            (FillerName::new("b"), mk(200)),
        ];

        let ordered = order_fillers_by_cell_id(fillers).unwrap();
        let names: Vec<String> = ordered.iter().map(|(n, _)| n.to_string()).collect();
        assert_eq!(names, ["a", "b", "c"]);
    }

    #[test]
    fn a_filler_without_cells_is_rejected() {
        let fillers = vec![(FillerName::new("empty"), Model::parse("just a title\n"))];
        // `Model` is not `Debug`, so match rather than `unwrap_err`.
        let Err(err) = order_fillers_by_cell_id(fillers) else {
            panic!("a filler with no cells must be rejected");
        };
        assert!(
            err.to_string().contains("No cell ID found"),
            "unexpected error: {err}"
        );
    }

    // ── Git provenance ────────────────────────────────────────────────────────

    #[test]
    fn git_hash_falls_back_when_there_is_no_repository() {
        // A directory outside any repository — `tempdir` is not under this one.
        let dir = tempdir().unwrap();
        assert_eq!(get_hash_of_project(dir.path()), "GIT repository not found");
    }

    #[test]
    fn git_hash_describes_a_repository_without_tags() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("f.txt"), "x").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("f.txt")).unwrap();
        index.write().unwrap();
        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        let sig = git2::Signature::now("t", "t@example.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        let hash = get_hash_of_project(dir.path());
        assert_ne!(hash, "GIT repository not found");
        // An untagged repository falls back to the short SHA.
        assert!(
            hash.len() >= 7 && hash.chars().next().unwrap().is_ascii_hexdigit(),
            "expected a short SHA, got {hash:?}"
        );
    }
}
