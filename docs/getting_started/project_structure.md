# Project Structure

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