"""
This file contains the generate_model function, the only function a user needs to call
to generate the MCNP model.
"""

import json
import logging
import re
from importlib.metadata import version
from pathlib import Path

from gitronics.compose_model import compose_model
from gitronics.file_readers import ParsedBlocks, read_files
from gitronics.model_manager import ModelManager


def generate_model(
    root_folder_path: Path,
    configuration_file_path: Path,
    project_summary_path: Path,
    write_path: Path,
) -> None:
    """Generates the MCNP model based on the configuration file and the project summary
    file and writes it in a new file called "assembled.mcnp" in the write_path. Also
    writes metadata in a file called "gitronics_metadata.json" with the version of the
    module.

    Parameters
    ----------
    root_folder_path : Path
        Root folder of the project. The relative paths of the project summary file
        originate from this folder.
    configuration_file_path : Path
        YAML file with information about which files to include and which envelopes to
        fill.
    project_summary_path : Path
        CSV file with all the given name and the relative path of all the files that
        live in the project.
    write_path : Path
        Path where the assembled MCNP model and metadata will be written.
    """
    model_manager = ModelManager(
        root_folder_path=root_folder_path,
        configuration_file_path=configuration_file_path,
        project_summary_path=project_summary_path,
    )
    file_paths = model_manager.get_included_paths()
    parsed_blocks = read_files(file_paths)
    fill_envelope_cards(parsed_blocks, model_manager)
    text = compose_model(parsed_blocks)
    with open(write_path / "assembled.mcnp", "w", encoding="utf-8") as infile:
        infile.write(text)
    _dump_metadata(write_path)


def fill_envelope_cards(
    parsed_blocks: ParsedBlocks, model_manager: ModelManager
) -> None:
    """Modifies the ParseBlocks by filling the envelope cards with a placeholder of the
    form "$ FILL = envelope_name" with the actual universe id and transformation card
    if needed.
    """
    logging.info("Preparing FILL cards in the envelope structure.")
    envelope_structure_id = _get_envelope_structure_first_cell_id(model_manager)
    text = parsed_blocks.cells[envelope_structure_id]

    for envelope_name, envelope_data in model_manager.configuration.envelopes.items():
        # If the envelope is left empty in the configuration do not fill
        if not envelope_data:
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
        universe_id = model_manager.get_universe_id(envelope_name)
        fill_card = f" FILL = {universe_id} "
        if envelope_data.transform:
            fill_card += f" {envelope_data.transform} "
        fill_card += f"\n           $ {envelope_name} \n"

        # Modify the text
        text = re.sub(placeholder, fill_card, text)

    # Update the ParsedBlocks with the new text for the envelope structure
    parsed_blocks.cells[envelope_structure_id] = text


def _get_envelope_structure_first_cell_id(model_manager: ModelManager) -> int:
    path = model_manager.project_summary[model_manager.configuration.envelope_structure]
    with open(path, encoding="utf-8") as infile:
        for line in infile:
            match_first_cell_id = re.match(r"^(\d+)", line)
            if match_first_cell_id:
                return int(match_first_cell_id.group(1))
    raise ValueError(f"Could not find the first cell ID in {path}.")


def _dump_metadata(write_path: Path) -> None:
    with open(write_path / "gitronics_metadata.json", "w", encoding="utf-8") as infile:
        metadata = {
            "gitronics_version": version("gitronics"),
        }
        json.dump(metadata, infile, indent=4)
