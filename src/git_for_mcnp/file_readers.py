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


def read_files(files: List[Path]) -> ParsedBlocks:
    """Reads the files and returns the parsed blocks."""
    parsed_data = {}
    for file in files:
        logging.info("Reading file: %s", file)
        suffix = file.suffix[1:]  # remove the dot like in ".mcnp"

        if suffix == "mcnp":
            cells_block, surfaces_block = _read_mcnp(file)
            cells_dict = parsed_data.get("cells", {})
            cells_dict[cells_block.first_id] = cells_block.text
            parsed_data["cells"] = cells_dict

            surfs_dict = parsed_data.get("surfaces", {})
            surfs_dict[surfaces_block.first_id] = surfaces_block.text
            parsed_data["surfaces"] = surfs_dict

        elif suffix == "source":
            source_block = _read_first_block(file)
            parsed_data["source"] = source_block.text

        else:
            block = _read_first_block(file)
            card_type_dict = parsed_data.get(suffix, {})
            card_type_dict[block.first_id] = block.text
            parsed_data[suffix] = card_type_dict

    return ParsedBlocks(
        cells=parsed_data.get("cells", {}),
        surfaces=parsed_data.get("surfaces", {}),
        tallies=parsed_data.get("tally", {}),
        materials=parsed_data.get("mat", {}),
        transforms=parsed_data.get("transform", {}),
        source=parsed_data.get("source", ""),
    )


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
        raise ValueError("Could not parse the first cell ID value...")
    first_cell_id = int(first_cell_id.group())

    first_surface_id = re.search(r"^\*?\d+", surfaces, flags=re.MULTILINE)
    if first_surface_id is None:
        raise ValueError("Could not parse the first surface ID value...")
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
