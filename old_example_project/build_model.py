"""
This script is used to generate the MCNP model.

Adapt the PATHS constants to the correct paths.
"""

import logging
from pathlib import Path

from old_gitronics import generate_model

ROOT_FOLDER_PATH = Path(r".")
CONFIGURATION_FILE_PATH = Path(r"configurations/configuration_1.yml")
PROJECT_SUMMARY_PATH = Path(r"./project_summary.csv")
WRITE_PATH = Path(r"./assembled")


def _main():
    logging.basicConfig(level=logging.INFO)
    generate_model(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_file_path=CONFIGURATION_FILE_PATH,
        project_summary_path=PROJECT_SUMMARY_PATH,
        write_path=WRITE_PATH,
    )


if __name__ == "__main__":
    _main()
