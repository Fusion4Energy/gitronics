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
| `.gitignore` | A file to ignore the contents of the directory in Git. |

The `.gitignore` file is written to the output directory to prevent accidentally committing the assembled model and report to version control. 

The top of `assembled.mcnp` contains a metadata comment block:

```
C ============================================================
C  Built by gitronics v0.1.0
C  Configuration : configurations/in_vessel_only.yaml
C  Git commit    : a1b2c3d4e5f6...
C  Date / time   : 2025-01-15T14:32:00Z
C ============================================================
```

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
