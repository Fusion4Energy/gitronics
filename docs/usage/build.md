# `build` — Assemble a Model

The `build` command reads a YAML configuration file and assembles a ready-to-run MCNP input file.

## Synopsis

```
gitronics build <CONFIG> [--output-path <DIR>]
```

| Argument | Description |
|---|---|
| `CONFIG` | Path to the configuration YAML file (absolute or relative to cwd). |
| `-o`, `--output-path` | Directory to write `assembled.mcnp` into. Defaults to `.` (current directory). |

## Example

```bash
gitronics build configurations/in_vessel_only.yaml --output-path output/
```

## Output

On success, the following files are written to `--output-path`:

| File | Description |
|---|---|
| `assembled.mcnp` | The complete, self-contained MCNP input deck. |
| `build_report.html` | A report of the build process, including a list of all files used and the envelope assignation. |
| `build_report.json` | The machine-readable build manifest, also used as the baseline for report comparisons. |
| `.gitignore` | A file to ignore the contents of the directory in Git. |

The `.gitignore` file is written to the output directory to prevent accidentally committing the assembled model and report to version control. 

The top of `assembled.mcnp` contains a metadata comment block:

```
C ============================================================
C  Built by gitronics v0.1.0
C  Configuration : configurations/in_vessel_only.yaml
C  Git commit    : v0.6.2-4-g04d555a
C  Date / time   : 2025-01-15 14:32:00
C ============================================================
```

## Comparing builds

Open the current build's `build_report.html`, select **Diff**, and choose or drop
the baseline `build_report.json`. Both reports stay local to your browser; the
HTML includes its assets and works offline. Rebuild or regenerate older HTML
reports to use the updated viewer. The JSON schema is unchanged.

The summary shows envelope changes, input and data-card changes, warning-message
changes, and signed cell/surface deltas (`current - baseline`). Click a count or
use the section controls to inspect the corresponding changes. **Build summary
changes** displays list-valued selections as bullet lists; an empty selection is
shown as **None**.

- **Assignments** groups repeated baseline/current filler substitutions, sorted
    by affected placement count. Expand a group to inspect its placements, or use
    **By envelope**. Search names or metadata, filter added/removed/reassigned
    envelopes or transform/universe/status changes, and select metadata values
    from either build. Results are paginated in groups of 25.
- **Inputs** separates content changes and added/removed inputs from collapsed
    path-only changes with identical hashes. Search includes full paths; select
    an input to compare paths, hashes and sizes. Missing hashes are marked as not
    comparable, not treated as unchanged content.
- **Components** compares same-name components and explicit assignment
    replacements. Replacements are inferred from shared envelope assignments,
    not guessed from similar filenames, and do not imply a rename. Added and
    removed component inventories remain available separately.
- **Data cards** lists modifications before removals and additions. Filter by
    category or change type and select a card for a line-numbered diff. Related
    tally cards such as `FMESH1014:N`, `FC1014`, and `FM1014` appear together under
    **Tally 1014**. A member addition, removal, or edit marks an existing tally as
    modified. Open a tally to select individual member comparisons, including
    unchanged members; modified members are selected first. Counts distinguish
    grouped items from individual card changes, and searches include all members.
    Unchanged line context is folded by default; full definitions remain available.
    Very costly line comparisons fall back to full definitions with a notice.
- **Checks & warnings** compares check results and warning-message occurrence
    counts, including new, resolved and repeated warnings. A warning count is not
    a geometry or physics validation result.

Placement details show baseline and current values together, including metadata,
assignment origins, and assigned components. Changed fields are highlighted;
unchanged fields are de-emphasized. Removed placements can still be inspected.
Schema 1 reports remain supported, with notices where input hashes or diagnostics
were not recorded. Input and card comparisons use recorded evidence; they do not
constitute a geometric or physical equivalence test.

The standalone **Data Cards** tab also groups tally members by ID within each
source file. Expand a tally and then a member to inspect its definition and
references. Searching for one member keeps its tally family visible. Global
defaults such as `E0` remain separate, as do non-tally cards with the same numeric
ID. Diff combines related tally members across files while preserving each
member's baseline and current file provenance.

## How files are located

Gitronics recursively searches all directories listed in `project_roots` (defined in the configuration) for files referenced by stem name. 
This means you can organise your project files into subdirectories however you like, as long as each stem name is unique across all `project_roots`.

## Python API

`build_model` is also available as a Python function for use in scripts and notebooks:

```python
import gitronics

gitronics.build_model(
    "configurations/in_vessel_only.yaml",
    "output/",
)
```

| Parameter | Type | Description |
|---|---|---|
| `config_path` | `str` or `Path` | Path to the configuration YAML file. |
| `output_path` | `str` or `Path` | Directory to write `assembled.mcnp` into. |

Raises `RuntimeError` on failure with a descriptive message.

## Logging

Set the `RUST_LOG` environment variable to control verbosity:

```bash
RUST_LOG=info gitronics build configurations/baseline.yaml
RUST_LOG=debug gitronics build configurations/baseline.yaml
```
