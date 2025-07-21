from pathlib import Path

ALLOWED_SUFFIXES = {".mcnp", ".transform", ".mat", ".source", ".tally", ".yaml", ".yml"}


def get_file_paths(project_root: Path) -> dict[str, Path]:
    if not project_root.exists() or not project_root.is_dir():
        raise FileNotFoundError(f"The directory {project_root} does not exist.")

    # Get all file paths
    paths_list = []
    for file in project_root.rglob("*"):
        if file.is_file():
            paths_list.append(file.resolve())

    # Build dictionary while checking for duplicates and that metadata exists
    file_paths = {}
    for path in paths_list:
        if path.suffix in ALLOWED_SUFFIXES:
            file_name = path.stem
            if file_name in file_paths:
                raise ValueError(f"Duplicate file name found: {file_name}")
            file_paths[file_name] = path
            if not path.with_suffix(".metadata").exists():
                raise FileNotFoundError(f"Metadata file not found for: {path}")
        elif path.suffix == ".metadata":
            continue
        else:
            raise ValueError(f"Unsupported file suffix for: {path}")

    return file_paths
