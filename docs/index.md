# Gitronics

**Gitronics** is a tool for assembling [MCNP](https://mcnp.lanl.gov/) neutronics models from modular, version-controlled components.

Instead of maintaining a single monolithic MCNP input file, Gitronics lets you decompose a model into independent universe *filler models*, an *envelope structure*, and separate data cards (materials, sources, tallies, transformations). These components are assembled at build time according to a YAML configuration file.

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

If you have never used Gitronics before, please take five minutes to read the whole [Getting Started](getting-started.md) guide to understand the program.

---

## Quick links

- [Installation](installation.md)
- [Getting Started](getting-started.md)
- [Configuration Reference](usage/configuration.md)
- [Examples](examples.md)
