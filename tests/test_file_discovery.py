from pathlib import Path

import pytest

from gitronics.file_discovery import get_file_paths

PATH_TEST_PROJECT = Path(__file__).parent / "test_resources" / "test_project"


def test_get_file_paths():
    file_paths = get_file_paths(PATH_TEST_PROJECT)
    pass


def test_get_file_paths_directory_does_not_exist():
    non_existent_path = Path("/non/existent/path")
    with pytest.raises(FileNotFoundError, match="The directory .* does not exist."):
        get_file_paths(non_existent_path)
