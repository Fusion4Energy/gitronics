from pathlib import Path

from gitronics.helpers import ALLOWED_SUFFIXES


def discover_file_paths(project_root: Path) -> dict[str, Path]:
    all_paths = get_all_file_paths(project_root)

    # Build dictionary while checking for duplicates and that metadata exists
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
