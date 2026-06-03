# Examples

## `example_project` — minimal working project

The repository ships with a small example project in `example_project/`. It demonstrates the essential project layout and a valid configuration file.

### Structure

```
example_project/
├── configurations/
│   └── valid_configuration.yaml
├── output/
│   └── .gitignore
└── reference_model/
    ├── envelope_structure.mcnp
    ├── envelope_structure.metadata
    └── filler_models/
        ├── filler_model_1.mcnp
        ├── filler_model_1.metadata
        └── filler_model_2.mcnp
```

### Configuration

```yaml title="example_project/configurations/valid_configuration.yaml"
project_roots: [..]

envelope_structure: envelope_structure

source: volumetric_source
materials: [materials]
transformations: [my_transform]
tallies: [fine_mesh]

envelopes:
  my_envelope_name_1: filler_model_1
  envelope_name_2: filler_model_2
```

### Building it

```bash
cd example_project
gitronics build configurations/valid_configuration.yaml --output-path output/
```

---

## `big_example` — large-scale fusion model

The `big_example/` directory contains a real-world-scale configuration for a tokamak neutronics model, showcasing how Gitronics handles large numbers of envelopes and multiple configuration variants.

### Configuration variants

| File | Description |
|---|---|
| `baseline.yaml` | Full reference model with all blanket sectors and fillers. |
| `baseline_void_check.yaml` | Same geometry, all fillers replaced with void — useful for checking cell volumes. |
| `in_vessel_only.yaml` | Only the in-vessel components; useful for fast scoping calculations. |

### Envelope / filler pattern

The baseline configuration maps blanket sectors to their corresponding filler models row by row:

```yaml
envelopes:
  blanket_sector_01_r1_c03: blk_dt1_w_fd_row_1_c03
  blanket_sector_01_r1_c02: blk_dt1_w_fd_row_1_c02
  blanket_sector_01_r1_c01: blk_dt1_w_fd_row_1_c01
  # ...
```

The `baseline_void_check.yaml` uses `overrides: baseline.yaml` and sets all envelopes to `null`, leaving the geometry intact but removing all materials — a standard MCNP void-check technique.

### Running a build

```bash
cd big_example
gitronics build configurations/baseline.yaml --output-path output/
```

---

## Migrating your own model

<!-- TODO: Add a worked example using a publicly available MCNP benchmark model -->

!!! note "Fill in this section"
    If you have a representative public MCNP model you can share, add a migration walkthrough here showing:

    1. The original monolithic input file
    2. The `gitronics migrate` command
    3. The resulting project structure
    4. Any manual clean-up steps needed

---

## Using the Python API

Gitronics can also be driven from Python, which is useful when integrating with automated workflows or parametric studies:

```python
import gitronics

# Build a model — equivalent to running the CLI
gitronics.run(["gitronics", "build", "configurations/baseline.yaml", "-o", "output/"])
```

<!-- TODO: Expand with richer Python workflow examples if the Python API grows -->
