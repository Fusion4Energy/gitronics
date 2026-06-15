import tempfile
from pathlib import Path

import gitronics
import pytest

EXAMPLE_CONFIG = (
    Path(__file__).parent.parent.parent
    / "example_project"
    / "configurations"
    / "valid_configuration.yaml"
)


def test_build_model_success():
    with tempfile.TemporaryDirectory() as output_dir:
        gitronics.build_model(EXAMPLE_CONFIG, output_dir)
        assert (Path(output_dir) / "assembled.mcnp").exists()


def test_build_model_accepts_str_paths():
    with tempfile.TemporaryDirectory() as output_dir:
        gitronics.build_model(str(EXAMPLE_CONFIG), output_dir)
        assert (Path(output_dir) / "assembled.mcnp").exists()


def test_build_model_missing_config_raises():
    with tempfile.TemporaryDirectory() as output_dir:
        with pytest.raises(RuntimeError):
            gitronics.build_model("does_not_exist.yaml", output_dir)
