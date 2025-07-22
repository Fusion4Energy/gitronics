"""
This file contains the generate_model function, the only function a user needs to call
to generate the MCNP model.
"""

import logging
import re
from importlib.metadata import version
from pathlib import Path

import yaml

from gitronics.compose_model import compose_model
from gitronics.file_readers import ParsedBlocks, read_files
from gitronics.project_manager import ProjectManager


class _ModelManager:
    def __init__(
        self, root_folder_path: Path, configuration_name: str, write_path: Path
    ):
        self.project_manager = ProjectManager(root_folder_path)
        self.config = self.project_manager.read_configuration(configuration_name)
        self.write_path = write_path

    def generate_model(self) -> None:
        file_paths_to_include = self.project_manager.get_included_paths(self.config)
        parsed_blocks = read_files(file_paths_to_include)
        self._fill_envelope_cards(parsed_blocks)
        text = compose_model(parsed_blocks)
        with open(self.write_path / "assembled.mcnp", "w", encoding="utf-8") as infile:
            infile.write(text)
        self._dump_metadata()

    def _fill_envelope_cards(self, parsed_blocks: ParsedBlocks) -> None:
        logging.info("Preparing FILL cards in the envelope structure.")
        envelope_structure_id = self._get_envelope_structure_first_cell_id()
        text = parsed_blocks.cells[envelope_structure_id]

        if not self.config.envelopes:
            logging.info("No envelopes to fill, skipping FILL cards.")
            return

        for envelope_name, filler_name in self.config.envelopes.items():
            # If the envelope is left empty in the configuration do not fill
            if not filler_name:
                continue

            # Search for the placeholder in the envelope structure
            placeholder = rf"\$\s+FILL\s*=\s*{envelope_name}\s*\n"
            if not re.search(placeholder, text):
                raise ValueError(
                    f"Envelope name {envelope_name} not found in envelope "
                    f"structure. The pattern $ FILL = {envelope_name} does not"
                    " appear in the envelope structure MCNP file."
                )

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

            # Modify the text
            text = re.sub(placeholder, fill_card, text)

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
        raise ValueError(f"Could not find the first cell ID in {path}.")

    def _dump_metadata(self) -> None:
        with open(
            self.write_path / "assembled.metadata", "w", encoding="utf-8"
        ) as infile:
            metadata = {
                "gitronics_version": version("gitronics"),
            }
            yaml.dump(metadata, infile, default_flow_style=False)


def generate_model(
    root_folder_path: Path, configuration_name: str, write_path: Path
) -> None:
    model_manager = _ModelManager(root_folder_path, configuration_name, write_path)
    model_manager.generate_model()
