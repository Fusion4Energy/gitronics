from pathlib import Path

import pytest

from gitronics.helpers import GitronicsError
from gitronics.project_manager import ProjectManager

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"


def test_check_configuration_wrong_root():
    with pytest.raises(GitronicsError, match="The directory .* does not exist."):
        ProjectManager(Path("/wrong/root"))


def test_read_configuration_valid():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    configuration = project_manager.read_configuration("valid_configuration")
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
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    configuration = project_manager.read_configuration("overrides_configuration")

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
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    with pytest.raises(GitronicsError, match="Configuration file .* not found."):
        project_manager.read_configuration("non_existent_configuration")


def test_get_included_paths():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    config = project_manager.read_configuration("valid_configuration")
    included_paths = project_manager.get_included_paths(config)
    included_names = [path.name for path in included_paths]
    expected_names = [
        "envelope_structure.mcnp",
        "filler_model_1.mcnp",
        "filler_model_2.mcnp",
        "volumetric_source.source",
        "fine_mesh.tally",
        "materials.mat",
        "my_transform.transform",
    ]
    assert set(included_names) == set(expected_names)


def test_get_included_paths_small_config():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    config = project_manager.read_configuration("small_config")
    included_paths = project_manager.get_included_paths(config)
    included_names = [path.name for path in included_paths]
    expected_names = ["envelope_structure.mcnp"]
    assert set(included_names) == set(expected_names)


def test_get_metadata():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    metadata = project_manager.get_metadata("filler_model_1")
    assert metadata["transformations"]["my_envelope_name_1"] == "*(10)"


def test_get_metadata_file_not_in_project():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    with pytest.raises(GitronicsError, match="File .* not found in the project."):
        project_manager.get_metadata("wrong")


def test_get_metadata_file_not_found():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    with pytest.raises(GitronicsError, match="Metadata file not found for .*"):
        project_manager.get_metadata("volumetric_source")


def test_get_transformation():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    transformation = project_manager.get_transformation(
        "filler_model_1", "my_envelope_name_1"
    )
    assert transformation == "*(10)"


def test_get_universe_id():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    universe_id = project_manager.get_universe_id("filler_model_1")
    expected_id = 121
    assert universe_id == expected_id


def test_get_universe_id_not_found():
    project_manager = ProjectManager(VALID_PROJECT_PATH)
    with pytest.raises(
        GitronicsError, match="Universe ID not found in filler model .*"
    ):
        project_manager.get_universe_id("envelope_structure")
