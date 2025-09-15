"""
This file contains the generate_model function, the only function a user needs to call
to generate the MCNP model.
"""

import logging
import re
from datetime import datetime
from importlib.metadata import version
from pathlib import Path

import yaml

from gitronics.compose_model import compose_model
from gitronics.file_readers import ParsedBlocks, read_files
from gitronics.helpers import GitronicsError
from gitronics.project_checker import ProjectChecker
from gitronics.project_manager import ProjectManager

PLACEHOLDER_PAT = re.compile(r"\$\s+FILL\s*=\s*(\w+)\s*")


class _ModelManager:
    def __init__(
        self, root_folder_path: Path, configuration_name: str, write_path: Path
    ):
        self.project_manager = ProjectManager(root_folder_path)
        self.config = self.project_manager.read_configuration(configuration_name)
        self.write_path = write_path
        ProjectChecker(self.project_manager).check_project(write_path)

    def generate_model(self) -> None:
        logging.info("Generating model for configuration: %s", self.config.name)

        file_paths_to_include = self.project_manager.get_included_paths(self.config)
        parsed_blocks = read_files(file_paths_to_include)
        self._fill_envelope_cards(parsed_blocks)
        text = compose_model(parsed_blocks)

        with open(
            self.write_path / f"assembled_{self.config.name}.mcnp",
            "w",
            encoding="utf-8-sig",
        ) as infile:
            infile.write(text)

        self._dump_metadata()
        logging.info("Model generation completed.")

    def _fill_envelope_cards(self, parsed_blocks: ParsedBlocks) -> None:
        logging.info("Preparing FILL cards in the envelope structure.")
        envelope_structure_id = self._get_envelope_structure_first_cell_id()
        text = parsed_blocks.cells[envelope_structure_id]

        if not self.config.envelopes:
            logging.info("No envelopes to fill, skipping FILL cards.")
            return

        fill_cards = {}
        for envelope_name, filler_name in self.config.envelopes.items():
            # If the envelope is left empty in the configuration do not fill
            if not filler_name:
                continue

            # Create the fill card
            universe_id = self.project_manager.get_universe_id(filler_name)
            transform = self.project_manager.get_transformation(
                filler_name, envelope_name
            )
            if transform:
                transform = transform.strip()
                if transform.startswith("*"):
                    fill_card = f" *FILL = {universe_id} {transform[1:]} "
                else:
                    fill_card = f" FILL = {universe_id} {transform} "
            else:
                fill_card = f" FILL = {universe_id} "
            fill_card += f"\n           $ {envelope_name} \n"
            fill_cards[envelope_name] = fill_card

        # Modify the text
        lines = text.splitlines(keepends=True)
        for i, line in enumerate(lines):
            match_placeholder = PLACEHOLDER_PAT.search(line)
            if match_placeholder:
                envelope_name = match_placeholder.group(1)
                if envelope_name in fill_cards:
                    lines[i] = re.sub(
                        PLACEHOLDER_PAT, fill_cards[envelope_name], lines[i]
                    )
        text = "".join(lines)

        # Update the ParsedBlocks with the new text for the envelope structure
        parsed_blocks.cells[envelope_structure_id] = text

    def _get_envelope_structure_first_cell_id(self) -> int:
        assert self.config.envelope_structure in self.project_manager.file_paths
        path = self.project_manager.file_paths[self.config.envelope_structure]
        with open(path, encoding="utf-8") as infile:
            for line in infile:
                match_first_cell_id = re.match(r"^(\d+)", line)
                if match_first_cell_id:
                    return int(match_first_cell_id.group(1))
        raise GitronicsError(f"Could not find the first cell ID in {path}.")

    def _dump_metadata(self) -> None:
        with open(
            self.write_path / "assembled.metadata", "w", encoding="utf-8"
        ) as infile:
            metadata = {
                "configuration_name": self.config.name,
                "gitronics_version": version("gitronics"),
                "build_date": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            }
            yaml.dump(metadata, infile, default_flow_style=False, sort_keys=False)


def generate_model(
    root_folder_path: Path, configuration_name: str, write_path: Path
) -> None:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(levelname)s - %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        filename=write_path / "model_generation.log",
        filemode="w",
    )
    model_manager = _ModelManager(root_folder_path, configuration_name, write_path)
    model_manager.generate_model()
