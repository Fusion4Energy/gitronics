# Changelog

All notable changes to Gitronics are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

<!-- TODO: Keep this file updated with each release. -->

## [Unreleased]

---

## [0.1.0] — initial release

### Added

- `build` command: assemble an MCNP model from modular components and a YAML configuration.
- `migrate` command: decompose a monolithic MCNP input file into a Gitronics project structure.
- Configuration inheritance via the `overrides` key.
- Multi-root project support via `project_roots`.
- Per-envelope void override (set filler to `null`).
- Python package (`pip install gitronics`) with `gitronics` CLI entry-point.
- Assembled model metadata header (version, git commit hash, build timestamp).
