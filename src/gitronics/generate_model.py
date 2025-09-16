"""
This file contains the generate_model function, the only function a user needs to call
to generate the MCNP model.
"""

import logging
import re
from datetime import datetime
from importlib.metadata import version

import yaml

from gitronics.compose_model import compose_model
from gitronics.file_readers import ParsedBlocks, read_files
from gitronics.helpers import Config, GitronicsError, ProjectParameters
from gitronics.project_checker import ProjectChecker
from gitronics.project_manager import ProjectManager

PLACEHOLDER_PAT = re.compile(r"\$\s+FILL\s*=\s*(\w+)\s*")


def generate_model(
    configuration_name: str, project_parameters: ProjectParameters
) -> None:
    # Set up logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(levelname)s - %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        filename=project_parameters.write_path / "model_generation.log",
        filemode="w",
    )
    # Check the whole project
    project_manager = ProjectManager(project_parameters)
    ProjectChecker(project_manager).check_project()
    # Read the configuration and assemble the model as a text string
    config = project_manager.read_configuration(configuration_name)
    text = _assemble_model(project_manager, config)
    # Write the model to a file
    with open(
        project_parameters.write_path / f"assembled_{config.name}.mcnp",
        "w",
        encoding="utf-8-sig",
    ) as infile:
        infile.write(text)
    _dump_metadata(project_parameters, config)
    logging.info("Model generation completed.")


def _assemble_model(project_manager: ProjectManager, config: Config) -> str:
    logging.info("Generating model for configuration: %s", config.name)
    file_paths_to_include = project_manager.get_included_paths(config)
    parsed_blocks = read_files(file_paths_to_include)
    _fill_envelope_cards(parsed_blocks, project_manager, config)
    text = compose_model(parsed_blocks)
    return text


def _fill_envelope_cards(
    parsed_blocks: ParsedBlocks, project_manager: ProjectManager, config: Config
) -> None:
    logging.info("Preparing FILL cards in the envelope structure.")
    envelope_structure_id = _get_envelope_structure_first_cell_id(
        project_manager, config
    )
    text = parsed_blocks.cells[envelope_structure_id]

    if not config.envelopes:
        logging.info("No envelopes to fill, skipping FILL cards.")
        return

    fill_cards = {}
    for envelope_name, filler_name in config.envelopes.items():
        # If the envelope is left empty in the configuration do not fill
        if not filler_name:
            continue

        # Create the fill card
        universe_id = project_manager.get_universe_id(filler_name)
        transform = project_manager.get_transformation(filler_name, envelope_name)
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
                lines[i] = re.sub(PLACEHOLDER_PAT, fill_cards[envelope_name], lines[i])
    text = "".join(lines)

    # Update the ParsedBlocks with the new text for the envelope structure
    parsed_blocks.cells[envelope_structure_id] = text


def _get_envelope_structure_first_cell_id(
    project_manager: ProjectManager, config: Config
) -> int:
    assert config.envelope_structure in project_manager.file_paths
    path = project_manager.file_paths[config.envelope_structure]
    with open(path, encoding="utf-8") as infile:
        for line in infile:
            match_first_cell_id = re.match(r"^(\d+)", line)
            if match_first_cell_id:
                return int(match_first_cell_id.group(1))
    raise GitronicsError(f"Could not find the first cell ID in {path}.")


def _dump_metadata(project_parameters: ProjectParameters, config: Config) -> None:
    with open(
        project_parameters.write_path / f"assembled_{config.name}.metadata",
        "w",
        encoding="utf-8",
    ) as infile:
        metadata = {
            "configuration_name": config.name,
            "gitronics_version": version("gitronics"),
            "build_date": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        }
        yaml.dump(metadata, infile, default_flow_style=False, sort_keys=False)
