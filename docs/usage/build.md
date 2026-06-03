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

Gitronics searches all directories listed in `project_roots` (defined in the configuration) for files referenced by stem name. Any file whose stem (name without extension) matches a key in the configuration is a candidate. This means you can organise your project files into subdirectories however you like, as long as each stem name is unique across all `project_roots`.

## Logging

Set the `RUST_LOG` environment variable to control verbosity:

```bash
RUST_LOG=info gitronics build configurations/baseline.yaml
RUST_LOG=debug gitronics build configurations/baseline.yaml
```
