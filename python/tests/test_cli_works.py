"""End-to-end coverage of the CLI entry point installed by ``pip install``."""

import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent.parent
EXAMPLE_CONFIG = (
    REPO_ROOT / "example_project" / "configurations" / "valid_configuration.yaml"
)

# `run(sys.argv)` hands argv straight to clap, whose `parse_from` discards the
# program name — so `-c` standing in for it is fine.
CLI = [sys.executable, "-c", "from gitronics import _cli_main; _cli_main()"]


def run_cli(*args):
    # A subprocess, so clap's process::exit on --help does not kill pytest.
    return subprocess.run([*CLI, *args], capture_output=True)


def test_cli_help():
    assert run_cli("--help").returncode == 0


def test_cli_builds_a_model():
    with tempfile.TemporaryDirectory() as out:
        result = run_cli("build", str(EXAMPLE_CONFIG), "-o", out)
        assert result.returncode == 0, result.stderr.decode()

        produced = Path(out)
        assert (produced / "assembled.mcnp").exists()
        assert (produced / "build_report.html").exists()
        assert (produced / "build_report.json").exists()
        # The build marks its own output directory as ignored by git.
        assert (produced / ".gitignore").read_text() == "*\n"


def test_cli_reports_failure_on_the_error_path():
    with tempfile.TemporaryDirectory() as out:
        result = run_cli("build", "does_not_exist.yaml", "-o", out)
        assert result.returncode == 1
        assert b"Error:" in result.stderr
