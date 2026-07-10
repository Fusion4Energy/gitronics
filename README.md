<div align="center">
  <img src="docs/assets/logo.png" alt="Gitronics logo" width="140"/>
  <h1>Gitronics</h1>
  <p><strong>Build MCNP neutronics models from modular, version-controlled components.</strong></p>

  [![CI](https://github.com/Fusion4Energy/gitronics/actions/workflows/ci.yml/badge.svg)](https://github.com/Fusion4Energy/gitronics/actions/workflows/ci.yml)
  [![Docs](https://github.com/Fusion4Energy/gitronics/actions/workflows/docs.yml/badge.svg)](https://fusion4energy.github.io/gitronics/latest/)
  [![PyPI](https://img.shields.io/pypi/v/gitronics)](https://pypi.org/project/gitronics/)
  [![License: EUPL-1.2](https://img.shields.io/badge/License-EUPL--1.2-blue.svg)](LICENSE)
</div>

---

Gitronics lets you decompose a monolithic [MCNP](https://mcnp.lanl.gov/) input file into independent, version-controllable components — universe filler models, an envelope structure, and separate data cards — and reassemble them at build time via a YAML configuration.

**Full documentation: [fusion4energy.github.io/gitronics](https://fusion4energy.github.io/gitronics/latest/)**

The Gitronics methodology is already used to maintain the most complex neutronics models of the [ITER](https://www.iter.org/) fusion reactor, and [JT-60SA](https://www.jt60sa.org/) tokamak.

## Why?

| Problem with monolithic models | Gitronics solution |
|---|---|
| `git diff` is unreadable | Each component is a separate file |
| Running variants means copying the whole file | Override only the fields that change |
| Teams can't work on sub-models in parallel | Each filler model is independent |
| Hard to know exactly what was run | Commit hash and timestamp written into every assembled file |

## Installation

```bash
pip install gitronics
```

Requires Python 3.9+. Pre-built wheels for Linux, macOS, and Windows are published to PyPI — no Rust toolchain needed.

## Quick start

**Migrate an existing model:**

```bash
gitronics migrate path/to/my_model.mcnp --output-path ./my_project
```

**Build a model:**

```bash
gitronics build my_project/configurations/baseline.yaml --output-path my_project/output/
```

**Get help:**

```bash
gitronics --help
gitronics build --help
gitronics migrate --help
```

## Project layout

```
my_project/
├── configurations/
│   ├── baseline.yaml          ← declares which fillers go where
│   └── variant_A.yaml         ← inherits baseline, overrides some fields
├── output/
│   └── .gitignore
└── reference_model/
    ├── envelope_structure.mcnp
    ├── filler_models/
    │   ├── component_A.mcnp
    │   └── component_B.mcnp
    └── data_cards/
        ├── materials/
        ├── sources/
        └── tallies/
```

## Configuration example

```yaml
project_roots: [..]  # Relative to this file's location

envelope_structure: envelope_structure
source: dt_plasma
materials: [all_materials]
tallies: [tritium_breeding_ratio]

envelopes:
  blanket_inner: blanket_v3
  blanket_outer: blanket_reference
  divertor: null  # void — no FILL card inserted
```

Configurations support inheritance: a variant config can set `overrides: baseline.yaml` and override only the fields that differ.

## Documentation

The full documentation covers installation, CLI reference, configuration options, best practices, and worked examples:

**[fusion4energy.github.io/gitronics](https://fusion4energy.github.io/gitronics/latest/)**

## Publication

Gitronics is described in the following publication:

- Cubi, A., et al. *Novel modular and Git-based approach for the management and development of radiation transport models*. [Fusion Engineering and Design](https://www.sciencedirect.com/journal/fusion-engineering-and-design), 2025. [https://doi.org/10.1016/j.fusengdes.2025.115248](https://doi.org/10.1016/j.fusengdes.2025.115248)


## License

[EUPL-1.2](LICENSE)
