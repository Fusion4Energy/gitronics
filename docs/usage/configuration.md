# Configuration Reference

Gitronics configurations are YAML files that declare which components to include in an assembled model.

## Full example

```yaml
# Where to search for referenced files (relative to this config file's directory)
project_roots: [..]

# Inherit from a parent config and override selected fields
overrides: null

# File stem of the envelope structure MCNP file
envelope_structure: envelope_structure

# Data card files (stems) to include
source: volumetric_source
materials: [materials_v2, extra_nuclides]
transformations: [sector_transforms]
tallies: [fine_mesh_tally, heating_tally]

# Maps envelope cell names to filler model stems
# Set a value to null to leave that envelope void (no FILL card inserted)
envelopes:
  blanket_sector_01: blanket_v3
  divertor_outer:    divertor_cassette
  shielding_block:   null
```

---

## Field reference

### `project_roots`

```yaml
project_roots: [.., ../shared_models]
```

A list of directory paths (relative to the config file or absolute) that Gitronics searches recursively for referenced files. Paths are resolved at load time.

- **Type:** `list[path]` or `null`
- **Default:** `null` (must be provided by the base config in an inheritance chain)

---

### `overrides`

```yaml
overrides: ../configurations/baseline.yaml
```

Path to a *parent* configuration. If set, the parent is loaded first and the current file is merged on top of it. Fields present in the current file take precedence; the parent provides defaults.

- **Type:** `path` or `null`
- **Default:** `null` (standalone config)

!!! info "Inheritance"
    Inheritance chains can be arbitrarily deep. The `envelopes` map is merged entry-by-entry: the override can add new envelopes or change specific ones without repeating the full list.

---

### `envelope_structure`

```yaml
envelope_structure: envelope_structure
```

Stem name of the MCNP file that contains the level-0 cells. Each cell that should be filled with a filler universe must have a placeholder comment `C @<envelope_name>` on the line before it (inserted automatically by `migrate`).

- **Type:** `string` or `null`

---

### `source`

```yaml
source: volumetric_plasma_dt
```

Stem name of the file containing the SDEF / source data card.

- **Type:** `string` or `null`

---

### `materials`

```yaml
materials: [structural_materials, tritium_breeding_materials]
```

List of stem names for material definition files. Files are concatenated in the listed order.

- **Type:** `list[string]` or `null`

---

### `transformations`

```yaml
transformations: [sector_transforms, assembly_transforms]
```

List of stem names for transformation card files (`TR` cards). Files are concatenated in the listed order.

- **Type:** `list[string]` or `null`

---

### `tallies`

```yaml
tallies: [flux_tally, heating_tally]
```

List of stem names for tally card files. Files are concatenated in the listed order.

- **Type:** `list[string]` or `null`

---

### `envelopes`

```yaml
envelopes:
  blanket_sector_01: blanket_v3
  divertor_outer:    null
```

A mapping from **envelope cell name** (as declared in the envelope structure file) to a **filler model stem name**. Setting a value to `null` leaves the envelope void — no `FILL` card is inserted for that cell.

- **Type:** `map[string, string | null]`
- **Default:** `{}` (empty map)

---

## Configuration inheritance example

A common pattern is to have a `baseline.yaml` that declares the full reference model, and assessment-specific configs that override only what changes:

```yaml title="configurations/baseline.yaml"
project_roots: [..]
envelope_structure: envelope_structure
source: dt_plasma
materials: [all_materials]
tallies: []
envelopes:
  blanket_inner: blanket_reference
  blanket_outer: blanket_reference
  divertor: divertor_reference
```

```yaml title="configurations/assessment_A.yaml"
overrides: baseline.yaml
tallies: [heating_mesh]
envelopes:
  blanket_inner: blanket_variant_A
```

When `assessment_A.yaml` is built, `blanket_inner` uses `blanket_variant_A` while `blanket_outer` and `divertor` fall back to the baseline values.
