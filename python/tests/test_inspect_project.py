import tempfile
from pathlib import Path

import gitronics
import pytest

EXAMPLE_PROJECT = Path(__file__).parent.parent.parent / "example_project"


def test_inspect_project_success():
    with tempfile.TemporaryDirectory() as output_dir:
        gitronics.inspect_project(EXAMPLE_PROJECT, output_dir)
        assert (Path(output_dir) / "project_report.html").exists()
        assert (Path(output_dir) / "project_report.json").exists()


def test_inspect_project_accepts_str_paths():
    with tempfile.TemporaryDirectory() as output_dir:
        gitronics.inspect_project(str(EXAMPLE_PROJECT), output_dir)
        assert (Path(output_dir) / "project_report.json").exists()


def test_inspect_project_no_configurations_raises():
    with tempfile.TemporaryDirectory() as project_dir, tempfile.TemporaryDirectory() as output_dir:
        with pytest.raises(RuntimeError):
            gitronics.inspect_project(project_dir, output_dir)
