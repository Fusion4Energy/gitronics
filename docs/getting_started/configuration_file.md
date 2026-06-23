# Configuration file

This is a [YAML](https://yaml.org/) file that declares which filler models and data cards are used to assemble a model.
The configuration file does not need to be in the `reference_model` directory, it can be anywhere in the project.
For example:

```
gitronics_project/
├── configurations/
│   ├── baseline.yaml     ← A configuration file
│   └── alternative.yaml  ← Another configuration file
|      
├── reference_model/      ← Directory containing the geometry and data card files
│   └── ...
|
├── assessment_specific/  ← Directory containing more geometry and data card files
│   └── ...
|
└── outputs/
    └── assembled.mcnp    ← An assembled MCNP input file generated with `gitronics build`
```

## Contents

A configuration file can only contain the following fields:

- **`project_roots`**: A list of directory paths to search for referenced files.
- **`overrides`**: A path to a parent configuration file to inherit from.
- **`envelope_structure`**: The file stem of the envelope structure MCNP file.
- **`source`**: The file stem of the source data card file.
- **`materials`**: A list of file stems of the materials data card files.
- **`transformations`**: A list of file stems of the transformations data card files.
- **`tallies`**: A list of file stems of the tallies data card files.
- **`envelopes`**: A dictionary that maps envelope cell names to filler model file stems. A value of empty or `null` indicates that the envelope cell should be left empty.

The only mandatory fields in a configuration file are `project_roots` and `envelope_structure`. 
If a configuration employs `overrides`, the `envelope_structure` field could be defined in the parent, only the `project_roots` field would still be mandatory.

??? info "Configuration file example"
    ```yaml
    project_roots: [..]
    overrides: null
    envelope_structure: envelope_structure
    source: volumetric_source
    materials: [materials_v2, extra_nuclides]
    transformations: [sector_transforms]
    tallies: [fine_mesh_tally, heating_tally]
    envelopes:
      blanket_sector_01: blanket_v3
      divertor_outer:    divertor_cassette
      shielding_block:   null
    ```

!!! warning "Paths"
    All paths in the configuration file are relative to the configuration file itself. If the configuration file is moved to a different directory, all paths must be updated accordingly. 

### `project_roots`

This field is a list of directory paths (relative to the configuration file) that Gitronics searches recursively for referenced files. 
Paths are resolved at load time.
This will usually point to the main `reference_model` directory, but it can also include other directories that contain additional geometry or data card files like an `assessment_specific` directory.

### `overrides`

This field is a relative path to a *parent* configuration file. If set, the parent is loaded first and the current file is merged on top of it. See [Overriding other configuration](#overriding-other-configuration) for more details.

### `envelope_structure`

This field is the file stem (file name without the extension) of the envelope structure MCNP file. 
The file must be discoverable in one of the directories specified in `project_roots`.

### `source`, `materials`, `transformations`, `tallies`

These fields are the file stems of the data card files to include in the assembled model.
The files must be discoverable in one of the directories specified in `project_roots`.
Only one source file can be specified, but any number of materials, transformations and tallies files can be included.

### `envelopes`

This field is a dictionary that maps envelope cell names to filler model file stems.
If an envelope cell name appears in the dictionary but is not found in the envelope structure file, Gitronics will raise an error.

## Overriding other configuration

If a configuration file specifies the `overrides` field, it will inherit from the parent configuration file. The parent is loaded first and the current file is merged on top of it. Fields present in the current file take precedence; the parent provides defaults.

The overriding rules are as follows:

- The field `project_roots` is not inherited, it must be specified in the current configuration file.
- The fields `envelope_structure`, `source`, `materials`, `transformations`, and `tallies` are inherited, but they can be overridden in the current configuration file. If the current file defines the field, the parent value is ignored.
- The field `envelopes` is inherited and merged entry-by-entry. The current file can add new envelopes or change specific ones without repeating the full list. If an envelope name is present in both the parent and the current configuration, the current value takes precedence.

!!! tip "Nested inheritance"
    Inheritance chains are allowed but are not considered a good practice. It is simpler and less error-prone to inherit from a single parent configuration file.
