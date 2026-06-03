"""
gitronics – Python wrapper around the Rust extension module.

The Rust ``run`` function accepts a ``sys.argv``-style list of strings and
passes them through the same clap parser used by the native CLI, so there is
only one place to maintain subcommands, flags, and help text.
"""

from __future__ import annotations

import sys

from .gitronics import run  # type: ignore[import]

__all__ = ["run"]


def _cli_main() -> None:
    """Command-line entry-point installed by ``pip install gitronics``."""
    # Pass argv as-is; clap handles --help, errors, and exit codes.
    try:
        run(sys.argv)
    except RuntimeError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        sys.exit(1)
