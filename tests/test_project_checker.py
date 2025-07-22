from pathlib import Path

import pytest

from gitronics.project_checker import ProjectChecker
from gitronics.project_manager import ProjectManager

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"


@pytest.fixture
def model_manager():
    return ProjectManager(VALID_PROJECT_PATH)


@pytest.fixture
def project_checker(model_manager):
    return ProjectChecker(model_manager.file_paths)


def test_check_configuration_env_struct_missing(model_manager, project_checker):
    configuration = model_manager.read_configuration("valid_configuration")
    project_checker.check_configuration(configuration)


def test_check_configuration_invalid(model_manager, project_checker):
    configuration = model_manager.read_configuration("missing_env_struct")
    with pytest.raises(
        ValueError, match="Envelope structure is not defined in the configuration."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_env_struct_path(model_manager, project_checker):
    configuration = model_manager.read_configuration("wrong_env_struct_path")
    with pytest.raises(
        ValueError, match="Envelope structure file .* not found in the project."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_filler_path(model_manager, project_checker):
    configuration = model_manager.read_configuration("missing_filler_path")
    with pytest.raises(ValueError, match="Filler file .* not found in the project."):
        project_checker.check_configuration(configuration)


def test_check_configuration_envelope_name(model_manager, project_checker):
    configuration = model_manager.read_configuration("wrong_envelope_name")
    with pytest.raises(
        ValueError, match="Envelope .* not found in the envelope structure."
    ):
        project_checker.check_configuration(configuration)


def test_check_configuration_source_path(model_manager, project_checker):
    configuration = model_manager.read_configuration("missing_source_path")
    with pytest.raises(ValueError, match="Source file .* not found in the project."):
        project_checker.check_configuration(configuration)


def test_check_configuration_tallies_path(model_manager, project_checker):
    configuration = model_manager.read_configuration("missing_tallies_path")
    with pytest.raises(ValueError, match="Tally file .* not found in the project."):
        project_checker.check_configuration(configuration)


def test_check_configuration_materials_path(model_manager, project_checker):
    configuration = model_manager.read_configuration("missing_materials_path")
    with pytest.raises(ValueError, match="Material file .* not found in the project."):
        project_checker.check_configuration(configuration)


def test_check_configuration_transforms_path(model_manager, project_checker):
    configuration = model_manager.read_configuration("missing_transforms_path")
    with pytest.raises(ValueError, match="Transform file .* not found in the project."):
        project_checker.check_configuration(configuration)
