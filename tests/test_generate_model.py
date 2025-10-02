from pathlib import Path
from unittest.mock import patch, mock_open

import pytest

from gitronics.generate_model import generate_model, _get_envelope_structure_first_cell_id

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


def test_envelope_with_transform_branch(tmpdir):
    """Test case to ensure the transform branch is properly covered.
    This tests the if envelope_data.transform branch on line 84-85."""
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
    
    # Verify the transform was included in the output
    with open(tmpdir / "assembled.mcnp") as infile:
        result_text = infile.read()
    
    # Check that the transform "(1)" was included in the FILL card for Envelope_2
    # From the configuration: Envelope_2: { filler: Model 2, transform: (1) }
    # Model 2 has universe ID u=125
    assert " FILL = 125  (1) " in result_text
    assert "$ Envelope_2" in result_text


def test_get_envelope_structure_first_cell_id_no_match():
    """Test the exception case in _get_envelope_structure_first_cell_id 
    when no line starts with a digit."""
    from gitronics.model_manager import ModelManager
    
    # Create a minimal configuration and project that will work
    configuration_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_1.yml"
    )
    
    model_manager = ModelManager(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_file_path=configuration_path,
        project_summary_path=PROJECT_SUMMARY_PATH,
    )
    
    # Mock the file content to have no lines starting with digits
    mock_file_content = "Title\nC Comment line\nC Another comment\n   Some indented text\n"
    
    with patch("builtins.open", mock_open(read_data=mock_file_content)):
        with pytest.raises(ValueError, match="Could not find the first cell ID"):
            _get_envelope_structure_first_cell_id(model_manager)


def test_envelope_without_transform_branch(tmpdir):
    """Test case to ensure the no-transform branch is properly covered.
    This tests the else branch when envelope_data.transform is None/empty."""
    configuration_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_no_transform.yml"
    )
    generate_model(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_file_path=configuration_path,
        project_summary_path=PROJECT_SUMMARY_PATH,
        write_path=Path(tmpdir),
    )
    
    # Verify the transform was NOT included in the output for Envelope_2
    with open(tmpdir / "assembled.mcnp") as infile:
        result_text = infile.read()
    
    # Check that there's no transform in the FILL card for Envelope_2
    # Should be "FILL = 125 " without any transform
    assert " FILL = 125 \n           $ Envelope_2" in result_text
    assert "$ Envelope_2" in result_text
