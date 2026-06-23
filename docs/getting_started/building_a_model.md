# Build Model

Once a configuration file is ready, the model can be built with the `gitronics build` command.
After the [installation of Gitronics](../installation.md) is complete, the command can be run from the terminal.
For example:

```bash
gitronics build configurations/baseline.yaml -o output/
```

This will assemble the model defined in `baseline.yaml` and write an `assembled.mcnp` file to the `output/` directory.

!!! tip "HTML report"
    The `gitronics build` command can also generate an HTML report of the assembled model. This report includes a summary of the configuration, a list of all files used, and the filler models assignation of the envelope structure.

If there is any problem during the assembly (configuration wrongly defined, missing files, duplicated card IDs, etc.), Gitronics will raise an error and the build will fail. The error message will indicate the cause of the failure and the file where it occurred.

See also: [Build command details](../usage/build.md).

## What happens during a build

When running the `gitronics build` command, the following steps are performed:

1. The configuration file (and any parent configs it inherits from) is loaded and merged.
2. All the files referenced in the configuration file are read and parsed. This includes the envelope structure, filler models, and data card files.
3. All fillers and data card files are reordered by their card type and ID numbers. Every `assembled.mcnp` file will have the same deterministic order of cards. Only the first ID number of the first card in each file is used to determine the order of the files.
4. The envelope structure is adapted to include the `FILL` cards for the envelope cells that have a filler model assigned. The `FILL` cards will reference the correct universe ID of the filler model by parsing the filler model in search of the first `U` card. The transformation associated to each `FILL` card is also applied as defined in the metadata of the filler model.
5. The envelope structure, filler models, and data card files are concatenated into a single MCNP input file.
6. Validations checks are performed on the assembled model to ensure that it is a valid MCNP input file. This includes checking for duplicate card IDs, missing cards, and other potential issues. Failure to pass the checks will crash the build with an error message.
7. The assembled model is written to the output file: `assembled.mcnp`.
8. An HTML report is written to the output directory with name: `build_report.html`.

## Logging

While the build is running, Gitronics will print `INFO` and `WARNING` messages to the terminal indicating the progress of the assembly.

!!! tip "`WARNING` messages"
    Watch out for `WARNING` messages, they indicate potential mistakes that will not stop the build, for example, the existence of envelope cells in the envelope structure that are not referenced in the configuration file. 
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

## Header

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