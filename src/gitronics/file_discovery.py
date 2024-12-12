"""
These functions take care of reading the configuration file and returning the paths of
the files that should be included in the model.
"""

import logging
from enum import StrEnum
from pathlib import Path
from typing import Any, List

import pandas as pd


class ConfigColumns(StrEnum):
    """Column names of the configuration csv file."""

    FOLDER_PATH = "Relative folder path"
    FILE_NAME = "File name"
    SUFFIX = "Suffix"
    INCLUDE = "Include"
    COMMENT = "Comment"


def get_included_paths(configuration_csv: Path, project_path: Path) -> List[Path]:
    """
    Returns a list with the paths of all the files that should be included in the model
    according to the configuration file.
    """
    logging.info("Reading configuration file")
    configuration = pd.read_csv(configuration_csv)
    included_paths = []
    for _, row in configuration.iterrows():
        if not _file_is_included(row):
            continue

        path = (
            project_path
            / Path(row[ConfigColumns.FOLDER_PATH])
            / (row[ConfigColumns.FILE_NAME] + row[ConfigColumns.SUFFIX])
        )
        included_paths.append(path)
    return included_paths


def _file_is_included(row: pd.Series[Any]) -> bool:
    include_value = row[ConfigColumns.INCLUDE].upper()
    if include_value == "NO":
        return False
    if include_value == "YES":
        return True
    raise ValueError(
        f"Include column should be either 'YES' or 'NO'. Got: {include_value}"
    )
