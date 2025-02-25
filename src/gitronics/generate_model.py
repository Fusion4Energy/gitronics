"""
This file contains the generate_model function, the only function a user needs to call
to generate the MCNP model.
"""

import json
from importlib.metadata import version
from pathlib import Path

from gitronics.compose_model import compose_model
# from gitronics.model_manager import get_included_paths
from gitronics.file_readers import read_files


def generate_model(
    configuration_csv: Path,
    project_path: Path,
    write_path: Path,
) -> None:
    """Generates the MCNP model based on the configuration file and the project path and
    writes it in a new file called "assembled.i" in the write_path. Also writes metadata
    in a file called "gitronics_metadata.json" with the version of the module.

    Parameters
    ----------
    configuration_csv : Path
        Information about which files should be included in the model and their paths.
    project_path : Path
        Root path of the folder with the files, the paths in the configuration file are
        relative to this path.
    write_path : Path
        Path where the assembled MCNP model and metadata will be written.
    """
    # files = get_included_paths(configuration_csv, project_path)
    # parsed_blocks = read_files(files)
    # text = compose_model(parsed_blocks)
    # with open(write_path / "assembled.i", "w", encoding="utf-8") as infile:
    #     infile.write(text)
    # _dump_metadata(write_path)


def _dump_metadata(write_path: Path) -> None:
    with open(write_path / "gitronics_metadata.json", "w", encoding="utf-8") as infile:
        metadata = {
            "gitronics_version": version("gitronics"),
        }
        json.dump(metadata, infile, indent=4)
