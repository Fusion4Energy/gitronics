# Start a New Project

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

See the [Configuration Reference](configuration_file.md) for all available keys.
