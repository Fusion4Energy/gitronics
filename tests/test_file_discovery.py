from pathlib import Path

import pytest

from gitronics.file_discovery import get_file_paths

PATH_TEST_RESOURCES = Path(__file__).parent / "test_resources"


def test_get_file_paths():
    file_paths = get_file_paths(PATH_TEST_RESOURCES / "valid_project")
    assert set(file_paths.keys()) == {
        "fine_mesh",
        "materials",
        "my_transform",
        "volumetric_source",
        "envelope_structure",
        "filler_model_1",
        "filler_model_2",
        "filler_model_3",
        "valid_configuration",
        "overrides_configuration",
    }
    for file_path in file_paths.values():
        assert file_path.is_file()

def test_get_file_paths_directory_does_not_exist():
    non_existent_path = Path("/non/existent/path")
    with pytest.raises(
        FileNotFoundError,
        match="The directory .* does not exist.",
    ):
        get_file_paths(non_existent_path)


def test_get_file_paths_duplicate_file_names():
    project_root = PATH_TEST_RESOURCES / "duplicated_filename_project"
    with pytest.raises(
        ValueError,
        match="Duplicate file name found: duplicated_name",
    ):
        get_file_paths(project_root)


def test_get_file_paths_missing_metadata():
    project_root = PATH_TEST_RESOURCES / "missing_metadata_project"
    with pytest.raises(
        FileNotFoundError,
        match="Metadata file not found for: .*",
    ):
        get_file_paths(project_root)


def test_get_file_paths_unsupported_suffix():
    project_root = PATH_TEST_RESOURCES / "invalid_suffix_project"
    with pytest.raises(
        ValueError,
        match="Unsupported file suffix for: .*",
    ):
        get_file_paths(project_root)
