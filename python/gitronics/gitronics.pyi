"""Type stubs for the gitronics Rust extension module."""

from pathlib import Path

def run(args: list[str]) -> None:
    """Run the gitronics CLI with the given argument list.

    This is equivalent to running ``gitronics`` from the command line.
    ``args[0]`` should be the program name (i.e. ``sys.argv``).

    Args:
        args: Argument list in ``sys.argv`` format, e.g.
            ``["gitronics", "build", "config.yaml"]``.

    Raises:
        RuntimeError: If the command fails.
    """
    ...

def py_build_model(config_path: Path | str, output_path: Path | str) -> None:
    """Assemble an MCNP model from a gitronics configuration file.

    Reads the YAML configuration at *config_path*, resolves all referenced
    filler models and data-card files, and writes ``assembled.mcnp`` (plus a
    ``.gitignore``) into *output_path*.

    Args:
        config_path: Path to the YAML configuration file.
        output_path: Directory where ``assembled.mcnp`` will be written.
            The directory must already exist.

    Raises:
        RuntimeError: If any step of the build fails (e.g. missing file,
            invalid MCNP syntax, validation error).

    Example::

        import gitronics

        gitronics.build_model(
            "configurations/baseline.yaml",
            "output/",
        )
    """
    ...

def py_migrate_model(mcnp_input: Path | str, output_path: Path | str) -> None:
    """Migrate a monolithic MCNP model into a new gitronics project.

    Splits the deck at *mcnp_input* into an envelope structure, per-universe
    filler models, and a data-cards file, and writes a baseline configuration
    that rebuilds the original model with :func:`build_model`.

    Args:
        mcnp_input: Path to the monolithic MCNP input file.
        output_path: Directory where the new project will be created.

    Raises:
        RuntimeError: If any step of the migration fails.
    """
    ...

def py_inspect_project(project_dir: Path | str, output_path: Path | str) -> None:
    """Inspect a whole gitronics project and write an interactive report.

    Scans every configuration in *project_dir*, the full filler library
    (including unused fillers) and the full envelope inventory (including
    unassigned envelopes), then writes ``project_report.json`` and a
    self-contained ``project_report.html`` composition dashboard.

    Args:
        project_dir: Path to the project directory (containing
            ``configurations/``).
        output_path: Directory where ``project_report.json`` and
            ``project_report.html`` will be written.

    Raises:
        RuntimeError: If no configuration is found or a scan step fails.

    Example::

        import gitronics

        gitronics.inspect_project(".", "output/")
    """
    ...
