"""
These functions read and parse the individual files that will make up the composed MCNP
model. The result is an instance of ParsedBlocks which holds all the sections of the
final file.
"""

import logging
import re
from dataclasses import dataclass
from pathlib import Path


@dataclass
class _FirstIdAndText:
    first_id: int
    text: str


@dataclass
class ParsedBlocks:
    """Contains all the sections of the MCNP input model."""

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

    def add_cells(self, block: _FirstIdAndText) -> None:
        if block.first_id in self.cells:
            raise ValueError(
                "Overwriting cells block with the same first cell ID:"
                f" {block.first_id}"
            )
        self.cells[block.first_id] = block.text

    def add_surfaces(self, block: _FirstIdAndText) -> None:
        if block.first_id in self.surfaces:
            raise ValueError(
                "Overwriting surfaces block with the same first surface ID:"
                f" {block.first_id}"
            )
        self.surfaces[block.first_id] = block.text

    def add_tallies(self, block: _FirstIdAndText) -> None:
        # Tally files are allowed to not have an ID (e.g. a SSW card)
        if block.first_id == 0:
            if 0 in self.tallies:
                self.tallies[0] += block.text
            else:
                self.tallies[0] = block.text
            return
        
        if block.first_id in self.tallies:
            raise ValueError(
                "Overwriting tally block with the same first tally ID:"
                f" {block.first_id}"
            )
        self.tallies[block.first_id] = block.text

    def add_materials(self, block: _FirstIdAndText) -> None:
        if block.first_id in self.materials:
            raise ValueError(
                "Overwriting material block with the same first material ID:"
                f" {block.first_id}"
            )
        self.materials[block.first_id] = block.text

    def add_transforms(self, block: _FirstIdAndText) -> None:
        if block.first_id in self.transforms:
            raise ValueError(
                "Overwriting transform block with the same first transform ID:"
                f" {block.first_id}"
            )
        self.transforms[block.first_id] = block.text

    def add_source(self, text: str) -> None:
        if self.source:
            raise ValueError("Overwriting source block which is already set.")
        self.source = text


BLANK_LINE = re.compile(r"^\s*\n", flags=re.MULTILINE)
MCNP_FILE_NEEDED_BLOCKS = 2


def _read_mcnp(file: Path) -> tuple[_FirstIdAndText, _FirstIdAndText]:
    with open(file, encoding="utf-8") as infile:
        blocks = BLANK_LINE.split(infile.read())

    if len(blocks) < MCNP_FILE_NEEDED_BLOCKS:
        raise ValueError(
            f"File {file} does not contain the two blocks: cells and surfaces."
        )

    cell_block, surfaces_block = blocks[:2]

    match_first_cell_id = re.search(r"^\d+", cell_block, flags=re.MULTILINE)
    if not match_first_cell_id:
        raise ValueError(f"Could not parse the first cell ID value in {file}.")
    first_cell_id = int(match_first_cell_id.group())

    match_first_surface_id = re.search(r"^\*?\d+", surfaces_block, flags=re.MULTILINE)
    if not match_first_surface_id:
        raise ValueError(f"Could not parse the first surface ID value in {file}.")
    first_surface_id = int(match_first_surface_id.group())

    return _FirstIdAndText(first_cell_id, cell_block), _FirstIdAndText(
        first_surface_id, surfaces_block
    )


def _read_first_block(file: Path) -> _FirstIdAndText:
    with open(file, encoding="utf-8") as infile:
        text = BLANK_LINE.split(infile.read())[0]
        if text[-1] != "\n":
            text += "\n"

    match_first_id = re.search(r"^\*?[a-zA-Z]*(\d+)", text, flags=re.MULTILINE)
    if not match_first_id:
        raise ValueError(f"Could not parse the first ID value in file {file}.")
    first_id = int(match_first_id.group(1))

    return _FirstIdAndText(first_id, text)


def _read_first_block_without_id(file: Path) -> str:
    with open(file, encoding="utf-8") as infile:
        text = BLANK_LINE.split(infile.read())[0]
        if text[-1] != "\n":
            text += "\n"
    return text


def read_files(files: list[Path]) -> ParsedBlocks:
    """Reads the files and returns the parsed blocks."""
    parsed_blocks = ParsedBlocks.empty_instance()

    for file in files:
        logging.info("Reading file: %s", file)
        suffix = file.suffix[1:]  # remove the dot like in ".mcnp"

        match suffix:
            case "mcnp":
                cells_block, surfaces_block = _read_mcnp(file)
                parsed_blocks.add_cells(cells_block)
                parsed_blocks.add_surfaces(surfaces_block)
            case "tally":
                try:
                    block = _read_first_block(file)
                except ValueError:
                    block = _FirstIdAndText(0, _read_first_block_without_id(file))

                parsed_blocks.add_tallies(block)
            case "mat":
                block = _read_first_block(file)
                parsed_blocks.add_materials(block)
            case "transform":
                block = _read_first_block(file)
                parsed_blocks.add_transforms(block)
            case "source":
                text = _read_first_block_without_id(file)
                parsed_blocks.add_source(text)
            case _:
                raise ValueError(f"Unknown file suffix for: {file}")

    return parsed_blocks
