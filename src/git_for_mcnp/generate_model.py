"""
This file contains the generate_model function, the only function a user needs to call
to generate the MCNP model.
"""
import json
from importlib.metadata import version
from pathlib import Path

from git_for_mcnp.compose_model import compose_model
from git_for_mcnp.file_discovery import get_included_paths
from git_for_mcnp.file_readers import read_files


def generate_model(
    configuration_csv: Path,
    project_path: Path,
    write_path: Path,
):
    """
    Generates the MCNP model based on the configuration file and the project path and
    writes it in a new file called "assembled.i" in the write_path. Also writes metadata
    in a file called "git_for_mcnp_metadata.json" with the version of the module.
    """
    files = get_included_paths(configuration_csv, project_path)
    parsed_blocks = read_files(files)
    text = compose_model(parsed_blocks)
    with open(write_path / "assembled.i", "w", encoding="utf-8") as infile:
        infile.write(text)
    _dump_metadata(write_path)


def _dump_metadata(write_path: Path):
    with open(
        write_path / "git_for_mcnp_metadata.json", "w", encoding="utf-8"
    ) as infile:
        metadata = {
            "git_for_mcnp_version": version("git_for_mcnp"),
        }
        json.dump(metadata, infile, indent=4)
