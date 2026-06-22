# Build a Model

Given a Gitronics project, the `build` command assembles the MCNP input file:

```bash
gitronics build configurations/baseline.yaml
```

By default the assembled file is written to the current directory. Use `--output-path` to redirect it:

```bash
gitronics build configurations/baseline.yaml --output-path output/
```

The output file is `assembled.mcnp` in the specified directory.

## What Happens During a Build

1. The configuration file (and any parent configs it inherits from) is loaded and merged.
2. All referenced files are located under the configured `project_roots`.
3. `FILL` cards are injected into the envelope cells to reference the correct universe IDs.
4. Filler models are appended to the envelope structure.
5. Data cards (materials, source, tallies, transformations) are concatenated.
6. A metadata comment block is written to the top of the output file recording the configuration name, build date, gitronics version, and git commit hash.

See also: [Build command details](../usage/build.md).
