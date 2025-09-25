import logging
from pathlib import Path

import pytest

from gitronics.helpers import GitronicsError, ProjectParameters
from gitronics.project_checker import ProjectChecker
from gitronics.project_manager import ProjectManager

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"
INVALID_PROJECT_PATH = (
    Path(__file__).parent
    / "test_resources"
    / "valid_project _with_invalid_configurations"
)
PATH_TEST_RESOURCES = Path(__file__).parent / "test_resources"


def test_check_configuration_valid():
    project_parameters = ProjectParameters(
        root_folder_path=VALID_PROJECT_PATH,
        write_path=Path("."),
    )
    project_manager = ProjectManager(project_parameters)
    project_checker = ProjectChecker(project_manager)
    configuration = project_manager.read_configuration("valid_configuration")
    project_checker.check_configuration(configuration)


def test_duplicate_file_names():
    project_parameters = ProjectParameters(
        root_folder_path=PATH_TEST_RESOURCES / "duplicated_filename_project",
        write_path=Path("."),
    )
    project_manager = ProjectManager(project_parameters)
    project_checker = ProjectChecker(project_manager)
    file_paths = project_checker._get_file_paths()
    with pytest.raises(
        GitronicsError, match="Duplicate file name found: duplicated_name"
    ):
        project_checker._check_no_duplicate_names(file_paths)


def test_missing_metadata_in_mcnp_file():
    project_parameters = ProjectParameters(
        root_folder_path=PATH_TEST_RESOURCES / "missing_metadata_project",
        write_path=Path("."),
    )
    project_manager = ProjectManager(project_parameters)
    project_checker = ProjectChecker(project_manager)
    file_paths = project_checker._get_file_paths()
    with pytest.raises(GitronicsError, match="Metadata file not found for: .*"):
        project_checker._check_metadata_files_exist_for_mcnp_models(file_paths)


@pytest.fixture
def invalid_project_manager():
    project_parameters = ProjectParameters(
        root_folder_path=INVALID_PROJECT_PATH,
        write_path=Path("."),
    )
    return ProjectManager(project_parameters)


@pytest.fixture
def invalid_project_checker(invalid_project_manager):
    return ProjectChecker(invalid_project_manager)


def test_check_configuration_env_struct_not_defined(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("missing_env_struct")
    with pytest.raises(
        GitronicsError, match="Envelope structure is not defined in the configuration."
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_env_struct_path(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("wrong_env_struct_path")
    with pytest.raises(
        GitronicsError, match="Envelope structure file .* not found in the project."
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_filler_path(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("missing_filler_path")
    with pytest.raises(
        GitronicsError, match="Filler file .* not found in the project."
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_envelope_name(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("wrong_envelope_name")
    with pytest.raises(
        GitronicsError, match="Envelope .* not found in the envelope structure."
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_missing_transformation_metadata_for_filler(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("missing_tr_for_filler")
    with pytest.raises(
        GitronicsError,
        match="Transformation for envelope .* not found in filler model .* metadata.",
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_source_path(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("missing_source_path")
    with pytest.raises(
        GitronicsError, match="Source file .* not found in the project."
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_tallies_path(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("missing_tallies_path")
    with pytest.raises(GitronicsError, match="Tally file .* not found in the project."):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_materials_path(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration("missing_materials_path")
    with pytest.raises(
        GitronicsError, match="Material file .* not found in the project."
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_transforms_path(
    invalid_project_manager, invalid_project_checker
):
    configuration = invalid_project_manager.read_configuration(
        "missing_transforms_path"
    )
    with pytest.raises(
        GitronicsError, match="Transform file .* not found in the project."
    ):
        invalid_project_checker.check_configuration(configuration)


def test_check_configuration_trigger_warnings(caplog):
    project_parameters = ProjectParameters(
        root_folder_path=VALID_PROJECT_PATH,
        write_path=Path("."),
    )
    project_manager = ProjectManager(project_parameters)
    project_checker = ProjectChecker(project_manager)
    configuration = project_manager.read_configuration("small_config")
    with caplog.at_level(logging.WARNING):
        project_checker.check_configuration(configuration)

    assert "No source included in the configuration!" in caplog.text
    assert "No materials included in the configuration!" in caplog.text


def test_check_configuration_envelope_not_accounted(caplog):
    project_parameters = ProjectParameters(
        root_folder_path=VALID_PROJECT_PATH,
        write_path=Path("."),
    )
    project_manager = ProjectManager(project_parameters)
    project_checker = ProjectChecker(project_manager)
    configuration = project_manager.read_configuration("envelope_not_accounted")
    with caplog.at_level(logging.WARNING):
        project_checker.check_configuration(configuration)
    assert (
        "There are empty envelopes in the structure not accounted for in the "
        "configuration: {'envelope_name_2'}" in caplog.text
    )
