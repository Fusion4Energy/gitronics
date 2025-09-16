from pathlib import Path

import pytest
import yaml

from gitronics.generate_model import generate_model
from gitronics.helpers import ProjectParameters

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"
TEST_RESOURCES_PATH = Path(__file__).parent / "test_resources"


@pytest.fixture
def project_parameters(tmpdir) -> ProjectParameters:
    return ProjectParameters(
        root_folder_path=VALID_PROJECT_PATH,
        write_path=tmpdir,
        extra_metadata_fields=None,
    )


def test_generate_model(tmpdir, project_parameters):
    generate_model("valid_configuration", project_parameters)
    with open(tmpdir / "assembled_valid_configuration.mcnp") as infile:
        result_text = infile.read()

    expected_file = TEST_RESOURCES_PATH / "expected_file_valid_configuration.mcnp"
    with open(expected_file) as infile:
        expected_text = infile.read()

    result_lines = result_text.splitlines()
    expected_lines = expected_text.splitlines()
    for i in range(len(expected_lines)):
        assert result_lines[i] == expected_lines[i]

    # Check that metadata was generated
    with open(tmpdir / "assembled_valid_configuration.metadata") as infile:
        metadata = yaml.safe_load(infile)
    assert "gitronics_version" in metadata


def test_envelope_left_empty_in_configuration(tmpdir, project_parameters):
    generate_model("small_override", project_parameters)
    with open(tmpdir / "assembled_small_override.mcnp") as infile:
        result_text = infile.read()

    assert "$ FILL = my_envelope_name_1" in result_text
