# `inspect` — Explore a Whole Project

The `inspect` command scans an **entire** gitronics project and writes an interactive report of its
composition. Where [`build`](build.md) assembles a single configuration into one deck, `inspect` looks
across the whole project at once: every configuration, the full filler *library* (including fillers no
configuration currently uses), and the full envelope *inventory* (including envelopes that are left
unfilled).

It never assembles a model, so it is fast and safe to run at any time.

## Synopsis

```
gitronics inspect [PROJECT_DIR] [--output-path <DIR>]
```

| Argument | Description |
|---|---|
| `PROJECT_DIR` | Path to the project directory. Defaults to `.` (current directory). Every YAML file under it that parses as a gitronics configuration is discovered — including override configurations in any subdirectory. |
| `-o`, `--output-path` | Directory to write the report into. Defaults to `.` (current directory). |

## Example

```bash
gitronics inspect . --output-path output/
```

## Output

Two files are written to `--output-path`:

| File | Description |
|---|---|
| `project_report.html` | A self-contained, offline interactive dashboard (open it in any browser). |
| `project_report.json` | The same data as a machine-readable manifest, for CI or external tooling. |

The dashboard has four views:

- **Overview** — a treemap of the selected configuration's composition, grouped by any metadata field
  you choose and sized by cell/surface/envelope count, plus coverage stat tiles.
- **Fillers** — the whole filler library as a searchable, metadata-faceted table with per-filler cell
  and surface counts and an envelope-reuse bar.
- **Envelopes** — the envelope inventory with each envelope's assignment, highlighting unfilled
  envelopes and listing fillers unused by the selected configuration.
- **Config diff** — pick two configurations and see exactly which envelopes were assigned differently.

The dashboard is a single HTML file with no external requests, so it can be committed, emailed, or
shared like any other artefact.

## Organising the model with metadata

The Overview grouping and the Fillers facets are driven by the metadata **you** record — nothing is
hardcoded, so the same tool works for a tokamak, a particle accelerator, or anything else.

Add any fields you like to a filler's `.metadata` file, alongside the reserved `transformations` key:

```yaml
transformations:
  envelope_427010: (41)

# Free-form, project-defined metadata — becomes groupable and facetable:
component_type: blanket
status: frozen
owner: neutronics-team
description: Inboard blanket module, revision 3
```

The `description` field is recognised specially and shown as a first-class column and tooltip; every
other field appears as a facet in the dashboard. You can describe envelopes the same way, using the
envelope structure's `.metadata` sidecar under an `envelopes:` map:

```yaml
envelopes:
  envelope_427010:
    sector: 1
    system: blanket
    description: Sector 1 inboard
```

See [File Types](../getting_started/file_types.md) for more on metadata files.

## Python API

`inspect_project` is also available as a Python function:

```python
import gitronics

gitronics.inspect_project(".", "output/")
```

| Parameter | Type | Description |
|---|---|---|
| `project_dir` | `str` or `Path` | Path to the project directory (containing `configurations/`). |
| `output_path` | `str` or `Path` | Directory to write `project_report.{html,json}` into. |

Raises `RuntimeError` if no configuration is found or a scan step fails.

## Logging

Set the `RUST_LOG` environment variable to control verbosity:

```bash
RUST_LOG=info gitronics inspect .
```
