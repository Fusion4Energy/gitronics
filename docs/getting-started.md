# Getting Started

This page walks you through two typical entry points:

1. **Starting fresh** — migrating an existing monolithic MCNP model into a Gitronics project.
2. **Building** — assembling a model from an existing Gitronics project.

---

## Migrating an existing model

If you already have a monolithic MCNP input file, use the `migrate` command to convert it into a Gitronics project structure automatically.

```bash
gitronics migrate path/to/my_model.mcnp --output-path ./my_project
```

This produces:

```
my_project/
├── configurations/
│   └── baseline.yaml          ← ready-to-use baseline configuration
├── output/
│   └── .gitignore
└── reference_model/
    ├── envelope_structure.mcnp
    ├── envelope_structure.metadata
    └── filler_models/
        ├── universe_101.mcnp
        ├── universe_101.metadata
        └── ...
```

!!! tip
    The `migrate` command does its best to identify universe boundaries automatically.
    Review the generated `envelope_structure.mcnp` and filler models to confirm they look correct before committing.

After migration, verify the round-trip by running a build:

```bash
gitronics build my_project/configurations/baseline.yaml --output-path my_project/output
```

---

## Building a model

Given a Gitronics project, the `build` command assembles the MCNP input file:

```bash
gitronics build configurations/baseline.yaml
```

By default the assembled file is written to the current directory. Use `--output-path` to redirect it:

```bash
gitronics build configurations/baseline.yaml --output-path output/
```

The output file is `assembled.mcnp` in the specified directory.

### What happens during a build

1. The configuration file (and any parent configs it inherits from) is loaded and merged.
2. All referenced files are located under the configured `project_roots`.
3. `FILL` cards are injected into the envelope cells to reference the correct universe IDs.
4. Filler models are appended to the envelope structure.
5. Data cards (materials, source, tallies, transformations) are concatenated.
6. A metadata comment block is written to the top of the output file recording the configuration name, build date, gitronics version, and git commit hash.

---

## Initialising a project from scratch

If you are starting a new model rather than migrating an existing one, create the directory layout manually:

```
my_project/
├── configurations/
│   └── baseline.yaml
├── output/
│   └── .gitignore   (contents: *)
└── reference_model/
    ├── envelope_structure.mcnp
    ├── envelope_structure.metadata
    ├── filler_models/
    └── data_cards/
        ├── materials/
        ├── sources/
        └── tallies/
```

Then write a minimal `baseline.yaml`:

```yaml
project_roots: [..]

envelope_structure: envelope_structure

envelopes:
  my_envelope: my_filler_model
```

See the [Configuration Reference](usage/configuration.md) for all available keys.
