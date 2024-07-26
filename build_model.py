"""
This script is used to generate the MCNP model. 

Adapt the CONFIGURATION and PROJECT_PATH variables to the correct paths.
"""
import logging
from pathlib import Path

from git_for_mcnp import generate_model

CONFIGURATION = Path(r"tests\example_structure\configuration.csv")
PROJECT_PATH = Path(r"tests\example_structure")
WRITE_PATH = Path(r"tests\example_structure")


def _main():
    logging.basicConfig(level=logging.INFO)
    generate_model(
        configuration_csv=CONFIGURATION,
        project_path=PROJECT_PATH,
        write_path=WRITE_PATH,
    )


if __name__ == "__main__":
    _main()
