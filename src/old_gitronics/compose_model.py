"""
These functions are used to compose the MCNP model from the blocks of cards that were
read and parsed previously.
"""

import logging
from io import StringIO
from typing import TextIO

from old_gitronics.file_readers import ParsedBlocks


def compose_model(parsed_data: ParsedBlocks) -> str:
    """
    Takes the parsed blocks obtained from the file_readers and composes the MCNP model
    as a string.
    """
    logging.info("Composing model")
    _trigger_warnings(parsed_data)
    model = StringIO()

    _write_cards_on_model(parsed_data.cells, model)
    model.write("\n")

    _write_cards_on_model(parsed_data.surfaces, model)
    model.write("\n")

    for card_dictionary in [
        parsed_data.materials,
        parsed_data.tallies,
        parsed_data.transforms,
    ]:
        _write_cards_on_model(card_dictionary, model)

    model.write(parsed_data.source)

    return model.getvalue()


def _write_cards_on_model(card_dictionary: dict[int, str], model: TextIO) -> None:
    card_ids = sorted(card_dictionary.keys())
    for card_id in card_ids:
        model.write(card_dictionary[card_id])


def _trigger_warnings(parsed_blocks: ParsedBlocks) -> None:
    if not parsed_blocks.source:
        logging.warning("No source included in the model!")
    if len(parsed_blocks.materials) == 0:
        logging.warning("No materials included in the model!")
    if len(parsed_blocks.cells) == 0:
        logging.warning("No cells included in the model!")
