import logging
from pathlib import Path

import pytest

from gitronics.helpers import GitronicsError
from gitronics.project_checker import ProjectChecker
from gitronics.project_manager import ProjectManager

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"
PATH_TEST_RESOURCES = Path(__file__).parent / "test_resources"


@pytest.fixture
def project_manager():
    return ProjectManager(VALID_PROJECT_PATH)


@pytest.fixture
def project_checker(project_manager):
    return ProjectChecker(project_manager)


def test_check_configuration_valid(project_manager, project_checker):
    configuration = project_manager.read_configuration("valid_configuration")
    project_checker.check_configuration(configuration)


def test_duplicate_file_names():
    project_manager = ProjectManager(
        PATH_TEST_RESOURCES / "duplicated_filename_project"
    )
    project_checker = ProjectChecker(project_manager)
    file_paths = project_checker._get_file_paths()
    with pytest.raises(
        GitronicsError, match="Duplicate file name found: duplicated_name"
    ):
        project_checker._check_no_duplicate_names(file_paths)


def test_missing_metadata_in_mcnp_file():
    project_manager = ProjectManager(PATH_TEST_RESOURCES / "missing_metadata_project")
    project_checker = ProjectChecker(project_manager)
    file_paths = project_checker._get_file_paths()
    with pytest.raises(GitronicsError, match="Metadata file not found for: .*"):
        project_checker._check_metadata_files_exist_for_mcnp_models(file_paths)


def test_check_configuration_env_struct_not_defined(project_manager, project_checker):
    configuration = project_manager.read_configuration("missing_env_struct")
    with pytest.raises(
        GitronicsError, match="Envelope structure is not defined in the configuration."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_env_struct_path(project_manager, project_checker):
    configuration = project_manager.read_configuration("wrong_env_struct_path")
    with pytest.raises(
        GitronicsError, match="Envelope structure file .* not found in the project."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_filler_path(project_manager, project_checker):
    configuration = project_manager.read_configuration("missing_filler_path")
    with pytest.raises(
        GitronicsError, match="Filler file .* not found in the project."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_envelope_name(project_manager, project_checker):
    configuration = project_manager.read_configuration("wrong_envelope_name")
    with pytest.raises(
        GitronicsError, match="Envelope .* not found in the envelope structure."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_missing_transformation_metadata_for_filler(
    project_manager, project_checker
):
    configuration = project_manager.read_configuration("missing_tr_for_filler")
    with pytest.raises(
        GitronicsError,
        match="Transformation for envelope .* not found in filler model .* metadata.",
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_source_path(project_manager, project_checker):
    configuration = project_manager.read_configuration("missing_source_path")
    with pytest.raises(
        GitronicsError, match="Source file .* not found in the project."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_tallies_path(project_manager, project_checker):
    configuration = project_manager.read_configuration("missing_tallies_path")
    with pytest.raises(GitronicsError, match="Tally file .* not found in the project."):
        project_checker.check_configuration(configuration)


def test_check_configuration_materials_path(project_manager, project_checker):
    configuration = project_manager.read_configuration("missing_materials_path")
    with pytest.raises(
        GitronicsError, match="Material file .* not found in the project."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_transforms_path(project_manager, project_checker):
    configuration = project_manager.read_configuration("missing_transforms_path")
    with pytest.raises(
        GitronicsError, match="Transform file .* not found in the project."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_trigger_warnings(project_manager, project_checker, caplog):
    configuration = project_manager.read_configuration("small_config")
    with caplog.at_level(logging.WARNING):
        project_checker.check_configuration(configuration)

    assert "No source included in the configuration!" in caplog.text
    assert "No materials included in the configuration!" in caplog.text
