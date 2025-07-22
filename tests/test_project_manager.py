from pathlib import Path

import pytest

from gitronics.project_manager import ProjectManager

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"


def test_read_configuration_valid():
    model_manager = ProjectManager(VALID_PROJECT_PATH)
    configuration = model_manager.read_configuration("valid_configuration")
    assert configuration.overrides is None
    assert configuration.envelope_structure == "envelope_structure"
    assert configuration.envelopes == {
        "my_envelope_name_1": "filler_model_1",
        "envelope_name_2": "filler_model_2",
    }
    assert configuration.source == "volumetric_source"
    assert configuration.tallies == ["fine_mesh"]
    assert configuration.materials == ["materials"]
    assert configuration.transforms == ["my_transform"]


def test_read_configuration_overrides():
    model_manager = ProjectManager(VALID_PROJECT_PATH)
    configuration = model_manager.read_configuration("overrides_configuration")

    assert configuration.envelope_structure == "my_envelope_structure"
    assert configuration.envelopes == {
        "my_envelope_name_1": "filler_model_2",
        "envelope_name_2": None,
    }
    assert configuration.source == "my_source"
    assert configuration.tallies == []
    assert configuration.materials == []
    assert configuration.transforms == ["my_transform"]
    assert configuration.source == "my_source"


def test_read_configuration_not_found():
    model_manager = ProjectManager(VALID_PROJECT_PATH)
    with pytest.raises(ValueError, match="Configuration file .* not found."):
        configuration = model_manager.read_configuration("non_existent_configuration")
