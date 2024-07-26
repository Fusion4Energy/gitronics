"""
These functions read and parse the individual files that will make up the composed MCNP
model. The result is an instance of ParsedBlocks which holds all the sections of the 
final file.
"""
import logging
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Tuple


@dataclass
class ParsedBlocks:
    """Contains all the sections of the MCNP model."""

    cells: Dict[int, str]
    surfaces: Dict[int, str]
    tallies: Dict[int, str]
    materials: Dict[int, str]
    transforms: Dict[int, str]
    source: str

    @classmethod
    def empty_instance(cls) -> "ParsedBlocks":
        """Returns an empty instance of ParsedBlocks."""
        return ParsedBlocks(
            cells={}, surfaces={}, tallies={}, materials={}, transforms={}, source=""
        )


def read_files(files: List[Path]) -> ParsedBlocks:
    """Reads the files and returns the parsed blocks."""
    parsed_blocks = ParsedBlocks.empty_instance()
    for file in files:
        logging.info("Reading file: %s", file)
        suffix = file.suffix[1:]  # remove the dot like in ".mcnp"

        match suffix:
            case "mcnp":
                cells_block, surfaces_block = _read_mcnp(file)
                parsed_blocks.cells[cells_block.first_id] = cells_block.text
                parsed_blocks.surfaces[surfaces_block.first_id] = surfaces_block.text

            case "tally":
                block = _read_first_block(file)
                parsed_blocks.tallies[block.first_id] = block.text

            case "mat":
                block = _read_first_block(file)
                parsed_blocks.materials[block.first_id] = block.text

            case "transform":
                block = _read_first_block(file)
                parsed_blocks.transforms[block.first_id] = block.text

            case "source":
                source_block = _read_first_block(file)
                parsed_blocks.source = source_block.text

            case _:
                raise ValueError(f"Unknown file suffix: {suffix}")

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

    first_cell_id = re.search(r"^\d+", cells, flags=re.MULTILINE)
    if first_cell_id is None:
        raise ValueError(f"Could not parse the first cell ID value in {file}...")
    first_cell_id = int(first_cell_id.group())

    first_surface_id = re.search(r"^\*?\d+", surfaces, flags=re.MULTILINE)
    if first_surface_id is None:
        raise ValueError(f"Could not parse the first surface ID value in {file}...")
    first_surface_id = int(first_surface_id.group())

    return _FirstIdAndText(first_cell_id, cells), _FirstIdAndText(
        first_surface_id, surfaces
    )


def _read_first_block(file: Path) -> _FirstIdAndText:
    with open(file, encoding="utf-8") as infile:
        text = BLANK_LINE.split(infile.read())[0]

    first_id = re.search(r"^\*?[a-zA-Z]*(\d+)", text, flags=re.MULTILINE)
    if first_id is None:
        raise ValueError(f"Could not parse the first ID value in file {file}...")
    first_id = int(first_id.group(1))

    return _FirstIdAndText(first_id, text)
