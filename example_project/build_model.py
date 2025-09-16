"""
This script is used to generate the MCNP model.

Adapt the PATHS constants to the correct paths.
"""

from pathlib import Path

from gitronics import ProjectParameters, generate_model

ROOT_FOLDER_PATH = Path(r".")
WRITE_PATH = Path(r"./assembled")


def _main():
    project_parameters = ProjectParameters(
        root_folder_path=ROOT_FOLDER_PATH,
        write_path=WRITE_PATH,
        extra_metadata_fields=None,
    )
    generate_model(
        configuration_name="valid_configuration", project_parameters=project_parameters
    )


if __name__ == "__main__":
    _main()
