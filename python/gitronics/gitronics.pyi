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
