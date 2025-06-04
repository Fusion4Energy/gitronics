from pathlib import Path

import pytest

from gitronics.file_readers import ParsedBlocks
from gitronics.generate_model import _fill_envelope_cards, generate_model

ROOT_FOLDER_PATH = Path(__file__).resolve().parents[1] / "tests" / "example_structure"
PROJECT_SUMMARY_PATH = (
    Path(__file__).resolve().parents[1]
    / "tests"
    / "example_structure"
    / "project_summary.csv"
)


def test_generate_model(tmpdir):
    configuration_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_1.yml"
    )
    generate_model(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_file_path=configuration_path,
        project_summary_path=PROJECT_SUMMARY_PATH,
        write_path=Path(tmpdir),
    )

    with open(tmpdir / "assembled.mcnp") as infile:
        result_text = infile.read()

    expected_file = (
        Path(__file__).resolve().parents[0]
        / "example_structure"
        / "expected_file_configuration_1.i"
    )
    with open(expected_file) as infile:
        expected_text = infile.read()

    assert result_text == expected_text

    # Check that metadata was generated
    with open(tmpdir / "gitronics_metadata.json") as infile:
        metadata = infile.read()
    assert "version" in metadata


def test_envelope_left_empty_in_configuration(tmpdir):
    configuration_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_empty_envelopes.yml"
    )
    generate_model(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_file_path=configuration_path,
        project_summary_path=PROJECT_SUMMARY_PATH,
        write_path=Path(tmpdir),
    )
    with open(tmpdir / "assembled.mcnp") as infile:
        result_text = infile.read()

    assert "$ FILL = My envelope name 1" in result_text


def test_envelope_name_not_found_in_envelope_structure(tmpdir):
    configuration_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/config_envelope_name_not_found.yml"
    )
    with pytest.raises(ValueError):
        generate_model(
            root_folder_path=ROOT_FOLDER_PATH,
            configuration_file_path=configuration_path,
            project_summary_path=PROJECT_SUMMARY_PATH,
            write_path=Path(tmpdir),
        )
