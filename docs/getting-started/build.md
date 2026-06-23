# Getting started

## Concepts

The following is a list of concepts or keywords that are used throughout the documentation. 

- **Gitronics**. The name of the tool and methodology.
- **Gitronics project**. A reactor/s or other nuclear project that is managed with the Gitronics methodology.
- **Git**. A distributed version control system that is used to track changes in the files of a **Gitronics project**.
- **Reference model directory**. A directory containing files that can be used to assemble an MCNP model.
- **Assembled model**. The output of the `gitronics build` command, a single MCNP input file generated from a **configuration** file and the contents of the **reference model directory**.
- **Configuration file**. A YAML file that declares which filler models and data cards are used to assemble a model.
- **Envelope structure**. An MCNP input file that defines the *level 0* geometry cells, that is, cells that do not belong to a filler universe. It is the only mandatory file in a Gitronics project.
- **Envelope cell**. A MCNP cell that can be filled with a filler model. They are defined in the envelope structure file as cells with an inline comment of the form `$ @env:<envelope_name>`, where `<envelope_name>` is the name for the envelope cell.
- **Filler model**. An MCNP input file that defines a *filler universe*, which is a set of geometry cells that can be inserted into an envelope cell. A filler model must have a metadata file with the same name and the `.metadata` extension.
- **Data card file**. A file that contains MCNP data cards, such as materials, sources, tallies, transformations or others. These files can have the following extensions: `.mat`, `.src`, `.tally`, or `.transform`.
- **Metadata file**. A YAML file that contains metadata related to a geometry or data card file. A metadata file must have the same name as the file it describes, but with the `.metadata` extension. These files are optional except for filler models and they can hold any arbitrary information.

## Project Structure

A Gitronics project contains a directory (usually called `reference_model`) that contains components of MCNP models.
The project directory can contain any number of subdirectories and files, with any arbitrary names.
Use the structure that better adapts to your project, below there is a typical layout:

```
reference_model/
├── envelope_structure.mcnp   
├── filler_models/            
│   ├── divertor.mcnp
│   ├── divertor.metadata
│   ├── toroidal_field_coil.mcnp
│   ├── toroidal_field_coil.metadata
│   ├── vacuum_vessel.mcnp
│   └── ...
└── data_cards/
    ├── materials/
    │   ├── general_materials.mat
    │   └── alternative_library.mat
    ├── sources/
    │   ├── plasma_source.src
    │   └── spherical_void.src
    ├── tallies/
    │   ├── radmaps.tally
    │   └── stochastic_volume.tally
    └── transformations/
        └── general_rotations.transform
```

Gitronics will only discover files that have the correct file extension, as follows:

- Geometry: `.mcnp`
- Materials: `.mat`
- Source: `.src`
- Tallies: `.tally`
- Transformations: `.transform`
- Metadata: `.metadata`

!!! warning "Duplicate file names"
    Gitronics will crash with an error message if it encounters duplicated file names in the project directory. This is a design choice to avoid ambiguity. Please give unique and meaningful names to your files, regardless of the subdirectory they are in. For example, `first_wall_ss.mcnp` and `first_wall_tungsten.mcnp`.

## File types

The files inside a Gitronics project can be classified into three types:

- Geometry files
- Data card files
- Metadata files

### Geometry files

Files with the `.mcnp` extension are considered geometry files.
These files can be either an [envelope structure](#envelope-structure) file or a [filler model](#filler-models) file. 
The only parts of the files that Gitronics will consider in a geometry file are the first two sections of an MCNP input file: the *cell cards* and the *surface cards*.

Even though an assembled MCNP file may need cards like materials and transformations, these cards will be read from the data card files instead of the geometry files. 
This is a design choice to avoid duplication of information and to keep the geometry files focused on the geometry definition.

??? note "Geometry file example"
    ```
    C Filler model 121                         │ ← Title
    1001   14    -7.89  1001 -1002 1004 -1003  │ ← Cells section
               IMP:N=1   IMP:P=1  U=121        │
    1002     0      1001 -1002                 │
               IMP:N=1   IMP:P=1  U=121        │

    C Surfaces model 121 
    1001     PZ  -1.4030000e+02                │ ← Surface section           
    1002     PZ   4.6300000e+01                │
    1003     CZ    160.700000                  │
    1004     CZ    144.900000                  │

    C Data cards (not considered)            
    m14   1001.31c  2.379E-02  $ H-1           │ ← Data cards section 
          5010.31c  2.350E-04  $ B10           │   This will not be 
          5011.31c  9.469E-04  $ B11           │   considered by 
          8016.31c  4.276E-02  $ O             │   gitronics
         11023.31c  2.068E-04  $ Na            │
    c     12000.31c  3.768E-05  $ Mg           │
         12024.31c  2.976E-05  $ Mg-24         │
         12025.31c  3.768E-06  $ Mg-25         │
         12026.31c  4.149E-06  $ Mg-26         │
         13027.31c  6.034E-04  $ Al            │
    c     14000.31c  1.239E-02  $ Si           │
         14028.31c  1.143E-02  $ Si-28         │
         14029.31c  5.786E-04  $ Si-29         │
    c                                          │
    mode  n                                    │
    sdef sur 398 nrm=-1                        │
         dir=d1 wgt=132732289.6                │
    sb1    -21  2                              │
    lost 1000                                  │
    prdmp j 1e7                                │
    nps  1e9                                   │
    ```

!!! warning "Title card"
    The title card of a geometry file is mandatory. Omitting it may cause an invalid parsing. Only the title card of the envelope structure file will be preserved in the assembled output file. The title cards of filler files will be ignored.

!!! tip "Data cards in geometry files"
    A geometry file may contain data cards, making it a complete and valid MCNP input file by itself. While Gitronics will ignore these data cards, they can be useful for testing the geometry without the need to create a configuration file. It can be especially useful to have a stochastic volume calculation source in the geometry to check for lost particles and calculate the volumes of the cells in a filler model.

#### Envelope structure

This is a special MCNP input file, it is the only mandatory file in a Gitronics project.
This file defines the *Level 0* or *block/envelope structure* of the model, that is, all the geometry cells that do not belong to a filler universe.
A Gitronics project can contain any number of envelope structure files, but one (and only one) must be used to build a model.

In this file, envelope cells can be defined if necessary. To do so, simply add an inline comment in the cell definition of the form `$ @env:<envelope_name>`, where `<envelope_name>` is the name for the envelope cell. For example:

```
21055 0    1 -2 8 -7 -6 -5
           imp:n=1.0   imp:p=1.0   
        $ @env:toroidal_field_coil_18
```

!!! tip "Multiple envelopes with the same name"
    It is possible to assign the same name to multiple envelope cells. This way, the configuration file would only need to reference the envelope name once, and all cells with that name will be filled with the same filler model. This is useful when a logical envelope cell needs to be decomposed into multiple cells to avoid lost particles or any other reason, but they all represent the same physical component.

#### Filler models

This kind of geometry files define a *filler universe*, that is, a set of cells and surfaces that can be used to fill envelope cells.
The cells of the files should already include the `U=<universe_id>` card, where `<universe_id>` is the universe ID number that will be used to fill the envelope cell.

### Data card files

These kind of files contain MCNP data cards and they must have one of the following extensions: `.mat`, `.src`, `.tally`, or `.transform`.
The choice of the file extension is only for organizational purposes, Gitronics does not enforce any restriction on the content of these files. For example, a file with the `.mat` extension can contain tally cards, and it will still be read by Gitronics.

A data card file is not a valid MCNP input file on its own.
It consists on a title card followed by a list of MCNP data cards. 
The title card is mandatory, and it will be ignored by Gitronics when assembling the model.
The data cards will be read until encountering a blank line or the end of the file. 
Any content after that will be ignored.

??? note "Data card file example"
    ```
    Tallies for radmaps                     │ ← Title (ignored)
    C These talllies will produce radmaps   │ ← Useful header comment 
    C  throughout the reactor geometry      │   (included)
    FMESH24:N  geom=xyz                     | ← Data cards section
             origin -2200 -2200 -1800       |
             imesh 2200  iints 176          |
             jmesh 2200  jints 176          |
             kmesh 3200  kints 200          |
             emesh 0.1 20                   |
    FM24 1.5e17                             |
    FC24 Neutron flux                       |
    C                                       |
    FMESH34:N  geom=xyz                     |
             origin -2200 -2200 -1800       |
             imesh 2200  iints 176          |
             jmesh 2200  jints 176          |
             kmesh 3200  kints 200          |
    FM34 2.60131e9 992 1 -4                 |
    FC34 Neutron dose in silica Gy/h        |
    C                                       |
                                            
    This block will not be read:            | ← Not included
    ...
    ```

!!! note "Arbitrary data card files"
    There are dozens of different types of data cards in MCNP and all of them can be included in any data card file. However, the recommended practice is to group the run parameters like `NPS`, `MODE`, `WWINP`, `PRNTDMP` and the like into the `.src` source file.

!!! warning "Number of data card files"
    Any number of data card files can be used to assemble a model with only one limitation. Only one `.src` file can be specified in the configuration.

### Metadata files

This kind of files are [YAML](https://yaml.org/) files that contain metadata related to other *geometry* or *data card* file.
The metadata file must have the same name as the file it describes, but with the `.metadata` extension.
For example, the file `toroidal_field_coil.mcnp` can have a metadata file called `toroidal_field_coil.metadata`.

These metadata files can contain any information, Gitronics will not enforce any restriction on the content of these files.

These files are optional for any file except for filler models.
Filler models must have a metadata file with at least the `transformation` field, which is a dictionary that maps any envelope name that the filler model can be applied to, to a transformation string.
The transformation string can be a transformation card like `(123)` or a transformation definition like `(10.1 0 0)`.
If the transformation definition is preceeded by a `*`, it will be interpreted as a transformation in degrees instead of radians like in `*(0.001 0.001 0.001 70 20 90 160 70 90 90 90 0)`.
The string can also be left empty or with the `null` value to indicate that no transformation should be applied to the filler model when it is inserted into that envelope cell.

??? note "Metadata file example"
    ```yaml
    description: "Toroidal field coil model..."  │ ← Arbitrary field (ignored)
    reference: "HA73HF"                          │ ← Arbitrary field (ignored)
    transformations:                             │ ← Only mandatory field for fillers
        my_envelope_name_1: "(10)"               │ ← When applied to this envelope, the FILL will use TR card 10
        my_envelope_name_2:                      │ ← No transformation for this envelope
        my_envelope_name_3: null                 │ ← No transformation for this envelope
        my_envelope_name_4: "(1.23 0.2 0.5)"     │ ← Transformation of (1.23 0.2 0.5)
        env_5: "*(0 0 0 130 40 90 140 130 90 90 90 0)" │ ← Transformation in degrees
    ```

It is a design choice to put the responsibility of selecting the correct transformation for each envelope cell into the metadata of the filler model.
An envelope should be agnostic of the filler models that may be applied to it, which could be developed independently by different teams.
When preparing a new filler model, the developer has to specify in the metadata how that filler model should be used when applied to each potential envelope cell.
If a filler model has to be applied to a new envelope cell, this system will require the explicit consideration of the developer/integrator to add the new envelope name to the metadata of the filler model, which is a good practice to avoid mistakes.

## Configuration file

This is a YAML file that declares which filler models and data cards are used to assemble a model.
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

### Contents

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

#### `project_roots`

This field is a list of directory paths (relative to the configuration file) that Gitronics searches recursively for referenced files. 
Paths are resolved at load time.
This will usually point to the main `reference_model` directory, but it can also include other directories that contain additional geometry or data card files like an `assessment_specific` directory.

#### `overrides`

This field is a relative path to a *parent* configuration file. If set, the parent is loaded first and the current file is merged on top of it. See [Overriding other configuration](#overriding-other-configuration) for more details.

#### `envelope_structure`

This field is the file stem (file name without the extension) of the envelope structure MCNP file. 
The file must be discoverable in one of the directories specified in `project_roots`.

#### `source`, `materials`, `transformations`, `tallies`

These fields are the file stems of the data card files to include in the assembled model.
The files must be discoverable in one of the directories specified in `project_roots`.
Only one source file can be specified, but any number of materials, transformations and tallies files can be included.

#### `envelopes`

This field is a dictionary that maps envelope cell names to filler model file stems.
If an envelope cell name appears in the dictionary but is not found in the envelope structure, Gitronics will raise an error.

### Overriding other configuration

If a configuration file specifies the `overrides` field, it will inherit from the parent configuration file. The parent is loaded first and the current file is merged on top of it. Fields present in the current file take precedence; the parent provides defaults.

The overriding rules are as follows:
- The field `project_roots` is not inherited, it must be specified in the current configuration file.
- The fields `envelope_structure`, `source`, `materials`, `transformations`, and `tallies` are inherited, but they can be overridden in the current configuration file. If the current file defines the field, the parent value is ignored.
- The field `envelopes` is inherited and merged entry-by-entry. The current file can add new envelopes or change specific ones without repeating the full list. If an envelope name is present in both the parent and the current configuration, the current value takes precedence.

!!! tip "Nested inheritance"
    Inheritance chains are allowed but are not considered a good practice. It is simpler and less error-prone to inherit from a single parent configuration file.

## Build Model

Once a configuration file is ready, the model can be built with the `gitronics build` command.
After the [installation of Gitronics](installation.md) is complete, the command can be run from the terminal.
For example:

```bash
gitronics build configurations/baseline.yaml -o output/
```

This will assemble the model defined in `baseline.yaml` and write an `assembled.mcnp` file to the `output/` directory.

!!! tip "HTML report"
    The `gitronics build` command can also generate an HTML report of the assembled model. This report includes a summary of the configuration, a list of all files used, and the filler models assignation of the envelope structure.

If there is any problem during the assembly (configuration wrongly defined, missing files, duplicated card IDs, etc.), Gitronics will raise an error and the build will fail. The error message will indicate the cause of the failure and the file where it occurred.

See also: [Build command details](../usage/build.md).

### What happens during a build

When running the `gitronics build` command, the following steps are performed:

1. The configuration file (and any parent configs it inherits from) is loaded and merged.
2. All the files referenced in the configuration file are read and parsed. This includes the envelope structure, filler models, and data card files.
3. All fillers and data card files are reordered by their card type and ID numbers. Every `assembled.mcnp` file will have the same deterministic order of cards. Only the first ID number of the first card in each file is used to determine the order of the files.
4. The envelope structure is adapted to include the `FILL` cards for the envelope cells that have a filler model assigned. The `FILL` cards will reference the correct universe ID of the filler model by parsing the filler model in search of the first `U` card. The transformation associated to each `FILL` card is also applied as defined in the metadata of the filler model.
5. The envelope structure, filler models, and data card files are concatenated into a single MCNP input file.
6. Validations checks are performed on the assembled model to ensure that it is a valid MCNP input file. This includes checking for duplicate card IDs, missing cards, and other potential issues. Failure to pass the checks will crash the build with an error message.
7. The assembled model is written to the output file: `assembled.mcnp`.
8. An HTML report is written to the output directory with name: `build_report.html`.

### Logging

While the build is running, Gitronics will print `INFO` and `WARNING` messages to the terminal indicating the progress of the assembly.

!!! tip "`WARNING` messages"
    Watch out for `WARNING` messages, they indicate potential mistakes that will not stop the build, for example, the existance of envelope cells in the envelope structure that are not referenced in the configuration file. 
    If the intention was to leave them empty, it is better to explicitly declare them as `null` in the configuration file to make sure they are not forgotten via `envelope_name: null`.

??? note "Example of `build` logging"
    ```
    [2026-06-23 14:38:02 INFO gitronics] Starting model build process for: configurations/valid_configuration.yaml
    [2026-06-23 14:38:02 INFO gitronics] Loading: reference_model/envelope_structure.mcnp
    [2026-06-23 14:38:02 INFO gitronics] Loading: reference_model/filler_models/filler_model_1.mcnp
    [2026-06-23 14:38:02 INFO gitronics] Loading: reference_model/filler_models/filler_model_2.mcnp
    [2026-06-23 14:38:02 INFO gitronics] Loading: reference_model/data_cards/my_transform.transform
    [2026-06-23 14:38:02 INFO gitronics] Loading: reference_model/data_cards/materials.mat
    [2026-06-23 14:38:02 INFO gitronics] Loading: reference_model/data_cards/fine_mesh.tally
    [2026-06-23 14:38:02 INFO gitronics] Loading: reference_model/data_cards/volumetric_source.source
    [2026-06-23 14:38:02 INFO gitronics] Adapting envelope structure with FILL cards
    [2026-06-23 14:38:02 INFO gitronics] Composing model
    [2026-06-23 14:38:02 INFO gitronics] Performing validation checks on the assembled model
    [2026-06-23 14:38:02 INFO gitronics] Build completed successfully in: output/assembled.mcnp
    ```

### Header

The `assembled.mcnp` file will contain a header comment at the top of the file like:

```
C ============================================================
C  Built by gitronics v0.1.0
C  Configuration : configurations/baseline.yaml
C  Git commit    : v0.5.18-3-g04d555a
C  Date / time   : 2026-06-23 12:34:51
C ============================================================
```

The first information line shows the Gitronics version used to build the model.
The second line shows the configuration file used to build the model, relative to the current working directory.
The third line shows the Git commit hash of the repository at the time of the build.
The fourth line shows the date and time when the build was performed.

## Card ID numbers

Whenever a file of any type is loaded by Gitronics during a `build` command, Gitronics will copy and paste the cards into a single model.
If two different files share the same card ID number, for example, two different filler models share a surface, Gitronics will include that surface twice in the assembled model, which will cause a validation error crashing the build process.
Gitronics will warn about which card IDs are duplicated with an error message.

[Migjorn](https://github.com/Fusion4Energy/migjorn), the parser used internally by Gitronics, is capable of renumbering on the fly any card ID.
However, as a design choice, Gitronics does not perform any automatic renumbering of card IDs. 
This is to avoid unexpected changes in the assembled model that could be difficult to track and debug.

Therefore, it is the responsibility of the user to ensure that all card IDs are unique across all files in the project.
The recommended practice is to keep a consisten numbering scheme for all the individual files in the project, so that they can be assembled without conflicts.
Tools like [Migjorn](https://github.com/Fusion4Energy/migjorn) or [F4Enix](https://github.com/Fusion4Energy/F4Enix) make the renumbering of MCNP files effortless.

!!! tip "A single card ID per system"
    A good practice is to assign the same card ID number to the different cards related to the same system. That is, make the universe ID of a filler model the same as the first cell ID, which is also the first surface ID, the first material ID (if the system needs non-standard materials). Same with tallies and transformations that are specific to that system. 

## Assessments

Gitronics can also be used to manage the specific changes required to the model for a specific assessment.
The recommended practice is to create a new Git branch for the assessment.
Then, create a new directory that will contain all the assessment-specific files (e.g. new filler models, tallies, configuration files, etc.).
This directory could also contain post-processing scripts, organization files and others.

Remember that the `project_roots` field in the configuration file can contain multiple directories, so the assessment-specific directory can be added to the list of roots to be searched for files.

```
gitronics_project/
├── configurations/
│   └── baseline.yaml             ← The reference configuration file
|      
├── reference_model/              ← Directory containing the reference files
│   └── ...
|
└── assessment_specific/
    ├── configurations/
    │   └── modified.yaml         ← This configuration overrides baseline.yaml applying changes
    │
    ├── changes/                  ← Directory containing new files
    │   ├── specific_tally.tally  ← New tally only for this assessment
    │   └── new_diagnostic.yaml   ← New filler only for this assessment
    |
    ├── output/
    │   └── assembled.mcnp        ← Assembled model for this assessment
    │   
    └── postprocessing/           ← Scripts to process the results of this assessment
        └── ... 
```

After the assessment is complete, the branch can be stored as a file via `git bundle`, and the assessment-specific files can be deleted from the main branch to keep it clean.
