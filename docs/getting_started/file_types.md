# File types

The files inside a Gitronics project can be classified into three types:

- Geometry files
- Data card files
- Metadata files

## Geometry files

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

### Envelope structure

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

### Filler models

This kind of geometry files define a *filler universe*, that is, a set of cells and surfaces that can be used to fill envelope cells.
The cells of the files should already include the `U=<universe_id>` card, where `<universe_id>` is the universe ID number that will be used to fill the envelope cell.

## Data card files

These kind of files contain MCNP data cards and they must have one of the following extensions: `.mat`, `.source`, `.tally`, or `.transform`.
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
    There are dozens of different types of data cards in MCNP and all of them can be included in any data card file. However, the recommended practice is to group the run parameters like `NPS`, `MODE`, `WWINP`, `PRNTDMP` and the like into the `.source` source file.

!!! warning "Number of data card files"
    Any number of data card files can be used to assemble a model with only one limitation. Only one `.source` file can be specified in the configuration.

## Metadata files

This kind of files are [YAML](https://yaml.org/) files that contain metadata related to other *geometry* or *data card* file.
The metadata file must have the same name as the file it describes, but with the `.metadata` extension.
For example, the file `toroidal_field_coil.mcnp` can have a metadata file called `toroidal_field_coil.metadata`.

These metadata files can contain any information, Gitronics will not enforce any restriction on the content of these files.

These files are optional for any file except for filler models.
Filler models must have a metadata file with at least the `transformation` field, which is a dictionary that maps any envelope name that the filler model can be applied to, to a transformation string.
The transformation string can be a transformation card like `(123)` or a transformation definition like `(10.1 0 0)`.
If the transformation definition is preceded by a `*`, it will be interpreted as a transformation in degrees instead of radians like in `*(0.001 0.001 0.001 70 20 90 160 70 90 90 90 0)`.
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