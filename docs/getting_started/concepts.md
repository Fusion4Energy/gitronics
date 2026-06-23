# Concepts

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