"""
This script is used to generate the MCNP model.

Adapt the PATHS constants to the correct paths.
"""

from pathlib import Path

from gitronics import generate_model

ROOT_FOLDER_PATH = Path(r".")
WRITE_PATH = Path(r"./assembled")


def _main():
    generate_model(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_name="valid_configuration",
        write_path=WRITE_PATH,
    )


if __name__ == "__main__":
    _main()
