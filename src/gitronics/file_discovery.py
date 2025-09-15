from pathlib import Path

from gitronics.helpers import ALLOWED_SUFFIXES


def get_valid_file_paths(project_root: Path) -> dict[str, Path]:
    """Gets all the file paths with allowed suffixes in the project as a dictionary
    `name: path`."""
    all_paths = get_all_file_paths(project_root)

    valid_suffix_paths = {}
    for path in all_paths:
        if path.suffix in ALLOWED_SUFFIXES:
            file_name = path.stem
            valid_suffix_paths[file_name] = path

    return valid_suffix_paths


def get_all_file_paths(project_root: Path) -> list[Path]:
    """
    Gets all the file paths in the project, including non-allowed suffixes.
    """
    paths_list = []
    for file in project_root.rglob("*"):
        if file.is_file():
            paths_list.append(file.resolve())

    return paths_list
