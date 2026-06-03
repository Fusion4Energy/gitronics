import subprocess
import sys


def test_cli_works():
    # Run in a subprocess so clap's process::exit(0) on --help doesn't kill pytest.
    # Actual behaviour is covered by the Rust unit tests.
    result = subprocess.run(
        [sys.executable, "-c", "from gitronics import _cli_main; _cli_main()", "--help"],
        capture_output=True,
    )
    assert result.returncode == 0
    