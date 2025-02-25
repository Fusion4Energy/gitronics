"""
These functions read and parse the individual files that will make up the composed MCNP
model. The result is an instance of ParsedBlocks which holds all the sections of the
final file.
"""

import logging
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Tuple


@dataclass
class ParsedBlocks:
    """Contains all the sections of the MCNP model."""

    cells: dict[int, str]
    surfaces: dict[int, str]
    tallies: dict[int, str]
    materials: dict[int, str]
    transforms: dict[int, str]
    source: str

    @classmethod
    def empty_instance(cls) -> "ParsedBlocks":
        """Returns an empty instance of ParsedBlocks."""
        return ParsedBlocks(
            cells={}, surfaces={}, tallies={}, materials={}, transforms={}, source=""
        )

    def add_file(self, file: Path) -> None:
        """Adds the file if the suffix is recognized."""
        suffix = file.suffix[1:]  # remove the dot like in ".mcnp"

        match suffix:
            case "mcnp":
                self._add_mcnp_file(file)

            case "tally":
                self._add_tally_file(file)

            case "mat":
                self._add_material_file(file)

            case "transform":
                self._add_transform_file(file)

            case "source":
                self._add_source_file(file)

            case _:
                raise ValueError(f"Unknown file suffix: {suffix}")

    def _add_mcnp_file(self, file: Path) -> None:
        cells_block, surfaces_block = _read_mcnp(file)
        self.cells[cells_block.first_id] = cells_block.text
        self.surfaces[surfaces_block.first_id] = surfaces_block.text

    def _add_tally_file(self, file: Path) -> None:
        block = _read_first_block(file)
        self.tallies[block.first_id] = block.text

    def _add_material_file(self, file: Path) -> None:
        block = _read_first_block(file)
        self.materials[block.first_id] = block.text

    def _add_transform_file(self, file: Path) -> None:
        block = _read_first_block(file)
        self.transforms[block.first_id] = block.text

    def _add_source_file(self, file: Path) -> None:
        self.source = _read_first_block(file).text


def read_files(files: list[Path]) -> ParsedBlocks:
    """Reads the files and returns the parsed blocks."""
    parsed_blocks = ParsedBlocks.empty_instance()

    for file in files:
        logging.info("Reading file: %s", file)
        parsed_blocks.add_file(file)

    return parsed_blocks


@dataclass
class _FirstIdAndText:
    first_id: int
    text: str


BLANK_LINE = re.compile(r"^\s*\n", flags=re.MULTILINE)
MCNP_FILE_NEEDED_BLOCKS = 2


def _read_mcnp(file: Path) -> Tuple[_FirstIdAndText, _FirstIdAndText]:
    with open(file, encoding="utf-8") as infile:
        blocks = BLANK_LINE.split(infile.read())

    if len(blocks) < MCNP_FILE_NEEDED_BLOCKS:
        raise ValueError(
            f"File {file} does not contain the two blocks: cells and surfaces..."
        )

    cells, surfaces = blocks[:2]

    match_first_cell_id = re.search(r"^\d+", cells, flags=re.MULTILINE)
    if not match_first_cell_id:
        raise ValueError(f"Could not parse the first cell ID value in {file}...")
    first_cell_id = int(match_first_cell_id.group())

    match_first_surface_id = re.search(r"^\*?\d+", surfaces, flags=re.MULTILINE)
    if not match_first_surface_id:
        raise ValueError(f"Could not parse the first surface ID value in {file}...")
    first_surface_id = int(match_first_surface_id.group())

    return _FirstIdAndText(first_cell_id, cells), _FirstIdAndText(
        first_surface_id, surfaces
    )


def _read_first_block(file: Path) -> _FirstIdAndText:
    with open(file, encoding="utf-8") as infile:
        text = BLANK_LINE.split(infile.read())[0]

    match_first_id = re.search(r"^\*?[a-zA-Z]*(\d+)", text, flags=re.MULTILINE)
    if not match_first_id:
        raise ValueError(f"Could not parse the first ID value in file {file}...")
    first_id = int(match_first_id.group(1))

    return _FirstIdAndText(first_id, text)
