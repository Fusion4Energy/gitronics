# Gitronics

**Gitronics** is a methodology and accompanying tool for mantaining and assembling [MCNP](https://mcnp.lanl.gov/) neutronics models from modular, version-controlled components.

Instead of maintaining a single monolithic MCNP input file, Gitronics lets you manage a collection of independent files that can be assembled into a complete MCNP model by running the command `gitronics build`.
Each file can represent a different aspect of the model, such as the geometry of a system, a set of materials, a source definition or a collection of tallies.
The assembly process is controlled by a [YAML](https://yaml.org/) configuration file, which specifies which components to include and how to combine them.
The use of configuration files allows you to create multiple variants of a model without duplicating files, and to easily swap components in and out.

This workflow enables:

- **Version control** — each component lives in its own file, making `git diff` meaningful.
- **Parametric variants** — swap geometry fillers or data cards via configuration inheritance without duplicating files.
- **Parallelism** — teams can develop different sub-models independently.
- **Reproducibility** — each assembled model records the git commit hash and build timestamp in its metadata.

---

## How it works

```
reference_model/
├── envelope_structure.mcnp   ← level-0 cells (the "shell" of the model)
├── filler_models/            ← per-universe MCNP snippets
│   ├── universe_101.mcnp
│   └── ...
└── data_cards/
    ├── materials/
    ├── sources/
    └── tallies/

configurations/
└── baseline.yaml             ← declares which fillers go where

output/
└── assembled.mcnp            ← produced by `gitronics build`
```

The `build` command reads a configuration file, loads all referenced components, inserts `FILL` cards into the envelope cells, and writes a single self-contained MCNP input file.
It is highly recommended to use [Git](https://git-scm.com/) to track changes to your project files.

If you have never used Gitronics before, please take a few minutes to read the whole [Getting Started](getting_started.md) guide to understand the methodology.

---

## Quick links

- [Installation](installation.md)
- [Getting Started](getting_started.md)
- [Configuration File](getting_started/configuration_file.md)
- [Usage](usage/build.md)
- [Examples](examples.md)
