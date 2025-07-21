from pathlib import Path

from gitronics.project_manager import ProjectManager

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"


def test_read_configuration_valid():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    configuration = project_manager.read_configuration("valid_configuration")

    assert configuration.overrides is None
    assert configuration.envelope_structure == "envelope_structure"
    assert configuration.envelopes == {
        "my_envelope_name_1": "filler_model_1",
        "envelope_2": "filler_model_2",
    }
    assert configuration.source == "volumetric_source"
    assert configuration.tallies == ["fine_mesh"]
    assert configuration.materials == ["materials"]
    assert configuration.transforms == ["my_transform"]


def test_read_configuration_overrides():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    configuration = project_manager.read_configuration("overrides_configuration")

    assert configuration.envelope_structure == "envelope_structure"
    assert configuration.envelopes == {
        "my_envelope_name_1": "filler_model_2",
        "envelope_2": None,
    }
    assert configuration.source == "volumetric_source"
    assert configuration.tallies == []
    assert configuration.materials == ["materials"]
    assert configuration.transforms == ["my_transform"]
