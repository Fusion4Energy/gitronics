# Migrate an Existing Model

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

See also: [Migrate command details](../usage/migrate.md).
