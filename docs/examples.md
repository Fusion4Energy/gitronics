# Examples

## `example_project` — minimal working project

The repository ships with a small example project in [`example_project/`](https://github.com/Fusion4Energy/gitronics/tree/main/example_project). It demonstrates the essential project layout and a valid configuration file.

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

## Using the Python API

Gitronics can also be driven from Python, which is useful when integrating with automated workflows or parametric studies:

```python
import gitronics

# Build a model — equivalent to running the CLI
gitronics.run(["gitronics", "build", "configurations/baseline.yaml", "-o", "output/"])
```

<!-- TODO: Expand with richer Python workflow examples if the Python API grows -->
