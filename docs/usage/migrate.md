# `migrate` — Convert a Monolithic Model

The `migrate` command takes a traditional single-file MCNP input deck and decomposes it into a Gitronics project structure.

## Synopsis

```
gitronics migrate <MCNP_INPUT> [--output-path <DIR>]
```

| Argument | Description |
|---|---|
| `MCNP_INPUT` | Path to the original monolithic MCNP input file. |
| `-o`, `--output-path` | Root directory for the new project. Defaults to `./project`. |

## Example

```bash
gitronics migrate models/tokamak_2025.mcnp --output-path ./tokamak_project
```

## What it produces

```
<output-path>/
├── configurations/
│   └── baseline.yaml             ← reproduces the original model exactly
├── output/
│   └── .gitignore                ← ignores all build outputs in git
└── reference_model/
    ├── envelope_structure.mcnp   ← level-0 cells
    ├── envelope_structure.metadata
    └── filler_models/
        ├── universe_<id>.mcnp    ← one file per universe
        ├── universe_<id>.metadata
        └── ...
```

### How universes are extracted

Each universe found in the model is written to its own `universe_<id>.mcnp` file. Cells belonging to universe 0 (the root lattice) are retained in the envelope structure.

### Metadata files

Each filler model is accompanied by a `.metadata` YAML file. These files describe how the filler should be placed and can specify per-envelope transformation cards. Edit them to add transformations after migration.

## Verifying the round-trip

After migration, rebuild the model and verify it matches the original:

```bash
gitronics build tokamak_project/configurations/baseline.yaml \
    --output-path tokamak_project/output
diff models/tokamak_2025.mcnp tokamak_project/output/assembled.mcnp
```

!!! note
    Minor formatting differences (whitespace, line ordering within a section) are expected. The geometry and physics data should be functionally identical.

## Limitations

- Data cards (materials, tallies, source) are written to a single `data_cards.source` file. After migration you may wish to split these into separate files under `reference_model/data_cards/`.
- The migration does not attempt to detect repeated structures or simplify the model.
