# Best Practices

This document lists some recommended practices for managing your Gitronics project. 
Following these practices will help you avoid common pitfalls and make your project easier to maintain.

## Modularization

Every file (or system) in a Gitronics project should be independent of the others.
This means that a filler model should not reference cells or surfaces from other filler models or the envelope structure directly. 

- Filler models should not share surfaces.
- Filler models should not share surfaces with the envelope structure.
- Avoid the use of `#` cells. If used, they should be defined in the same file, not a reference to a cell defined in other file.

## Envelope cell definition

It is recommended to place the envelope name place holder at the end of the cell definition.

??? note "Example"
    ```
    21055 0    1 -2 8 -7 -6 -5
               imp:n=1.0   imp:p=1.0   
            $ @env:toroidal_field_coil_18
    ```

!!! Warning "Space for the `FILL` card"
    The `FILL` card will be inserted by Gitronics right after the last cell parameter of the cell definition (after the `imp:p=1.0` in the example above). If there is not space to the right of the last parameter, the newly added `FILL` card may break the maximum line length of MCNP. To avoid this, it is recommended to leave some space after the last parameter of the cell definition before the envelope name placeholder as in the example above.

## Title cards

All the Gitronic files (geometry and data card files) should have a title card at the top of the file. 
The title card is a single line that will be ignored by Gitronics when assembling the model, but it is useful to provide a brief description for the user when viewing the independent file.

## Do not commit large files

The Gitronics methodology is designed to manage the source files of a model, not the assembled MCNP input files.
Via the information in the header of the `assembled.mcnp` file, it is possible to reproduce the exact same model from the source files. Therefore, it is not necessary to commit the assembled MCNP input files to the repository.

Commiting large files to the repository will make it slower to clone and checkout branches, and it will also make it harder to track changes.

!!! tip "Use `.gitignore`"
    Make use of the `.gitignore` file to avoid committing unnecessary files. You can place a `.gitignore` file with the content `*` in a folder to ignore all files in that folder (like the `output/` folder).
    You can ignore specific files by name or extension. 
    For example, to ignore all `.mcnp` files of a folder, you can add the line `*.mcnp` to the `.gitignore` file.

!!! Warning "Do not commit binary files"
    It is considered a bad practice to commit binary files to the repository (Excel files, runtpe, etc). 
    Binary files cannot be diffed, and they will make the repository size grow unnecessarily.
    Even if the files are text-based, do not commit large files like the `output` file of an MCNP run.

## Use descriptive, lowercase stem names

Stem names appear in configuration files and in assembled-model metadata. Choose names that describe the *physics content*, not the version or date:

| Avoid | Prefer |
|---|---|
| `model_2024_v3_final` | `blanket_tungsten_fw` |
| `mat_13` | `reduced_activation_ferritic_steel` |
| `tally1` | `tritium_breeding_ratio` |

## Nested universes

Each filler model file should contain only one universe.
The `$ @env:<envelope_name>` placeholder will only work on envelope structure files.

If a nested universe is absolutely needed, place the level 2 universe in the same file as the level 1 universe.
Make sure that the level 1 universe have the `FILL` cards correctly defined for the level 2 universe, which is placed right after the definition of the level 1.

## Track origin of the model in the metadata

Add a field like `reference` or similar to the `.metadata` files of each filler model to track the origin of the model. 
This is useful for future reference and for understanding the provenance of the model.

## Tag releases

Every time you build a model for a formal assessment or publication, create a git tag. Gitronics records the commit hash in the assembled file's metadata, but a tag makes it trivially easy to check out the exact source state later.

```bash
git tag -a v1.2.0 -m "Design freeze for FDR submission"
git push origin v1.2.0
```

## Include metadata files in version control

The `.metadata` files alongside each filler model are part of the source. Commit them alongside the `.mcnp` files.
