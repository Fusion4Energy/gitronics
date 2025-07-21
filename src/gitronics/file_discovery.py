from pathlib import Path


def get_file_paths(project_root: Path) -> dict[str, Path]:
    if not project_root.exists() or not project_root.is_dir():
        raise FileNotFoundError(f"The directory {project_root} does not exist.")

    file_paths = {}
    for file in project_root.rglob("*"):
        if file.is_file():
            file_paths[file.name] = file.resolve()

    return file_paths
