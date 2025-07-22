"""
This script is used to generate the MCNP model.

Adapt the PATHS constants to the correct paths.
"""

import logging
from pathlib import Path

from gitronics.generate_model import generate_model

ROOT_FOLDER_PATH = Path(r".")
WRITE_PATH = Path(r"./assembled")


def _main():
    logging.basicConfig(level=logging.INFO)
    generate_model(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_name="valid_configuration",
        write_path=WRITE_PATH,
    )


if __name__ == "__main__":
    _main()
