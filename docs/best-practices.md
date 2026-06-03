# Best Practices

## Project layout

### Keep one project per repository

Each Gitronics project should live in its own repository (or a well-isolated subdirectory in a mono-repo). This makes branch history, tagging, and code review meaningful.

### Use `.gitignore` in `output/`

The `migrate` command creates an `output/.gitignore` that ignores all files in that directory. Never commit assembled MCNP files — they are build artefacts and can be reproduced from the sources.

### Prefer shallow directory trees for `project_roots`

Deeply nested source trees slow down file discovery. Keep filler models, data cards, and the envelope structure in predictable directories at one or two levels deep.

---

## Naming conventions

### Use descriptive, lowercase stem names

Stem names appear in configuration files and in assembled-model metadata. Choose names that describe the *physics content*, not the version or date:

| Avoid | Prefer |
|---|---|
| `model_2024_v3_final` | `blanket_tungsten_fw` |
| `mat_13` | `reduced_activation_ferritic_steel` |
| `tally1` | `tritium_breeding_ratio` |

### Match envelope names to physical regions

Envelope names in the configuration file (`envelopes:` map) must match the placeholder comments written in the envelope structure file. Naming them after the physical component they represent makes configurations self-documenting.

---

## Configuration management

### Use a baseline + override hierarchy

Define a `baseline.yaml` that represents the full reference model. Create separate configuration files for each assessment or variant that override only the fields that differ. This minimises duplication and makes it easy to see what changed between assessments.

```
configurations/
├── baseline.yaml               ← full reference model
├── void_check.yaml             ← overrides: baseline; all envelopes → null
├── sensitivity_study_A.yaml    ← overrides: baseline; changes one filler
└── assessment_specific/
    └── design_point_X.yaml     ← multi-level override chain
```

### Pin `project_roots` carefully

`project_roots` is resolved relative to the configuration file. Using `[..]` (the parent directory of `configurations/`) is the conventional choice and keeps paths stable when the project is cloned to different machines.

---

## Filler model design

### One universe per file

Each filler model file should contain exactly one universe. This keeps the granularity consistent and makes `git blame` and `git log` useful at the component level.

### Record transformation intent in metadata

If a filler model is placed with a transformation (`TR` card), document that in the `.metadata` file so the relationship between filler geometry and its placement is explicit. Avoid hardcoding transformations inside the filler MCNP file itself.

### Avoid cross-references between fillers

Filler models should be self-contained. Cells in one filler should not reference surfaces or materials defined in another filler. Use the envelope structure for any shared bounding surfaces.

---

## Reproducibility

### Tag releases

Every time you build a model for a formal assessment or publication, create a git tag. Gitronics records the commit hash in the assembled file's metadata, but a tag makes it trivially easy to check out the exact source state later.

```bash
git tag -a v1.2.0 -m "Design freeze for TDR submission"
git push origin v1.2.0
```

### Include metadata files in version control

The `.metadata` files alongside each filler model are part of the source. Commit them alongside the `.mcnp` files.

---

## Performance

### Use `RUST_LOG=info` during development

The `info` log level prints the files being loaded and the transformations being applied without being too verbose. Use `debug` only when diagnosing unexpected behaviour.

### Keep filler models small

Very large filler models (tens of thousands of cells) slow down parsing. If a sub-model has grown large, consider whether it can be further decomposed into nested universes.
