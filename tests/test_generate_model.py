from pathlib import Path
import tempfile

import pytest

from gitronics.generate_model import generate_model, _get_envelope_structure_first_cell_id
from gitronics.model_manager import ModelManager

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


def test_get_envelope_structure_first_cell_id_no_digits():
    """Test _get_envelope_structure_first_cell_id when no line starts with digits."""
    # Create a temporary file with content that has no lines starting with digits
    content = """C Comment line 1
C Comment line 2
C Another comment
C No cell definitions
C End of file"""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.mcnp', delete=False) as tmp_file:
        tmp_file.write(content)
        tmp_file.flush()
        
        # Create a mock model manager with a project summary that points to our temp file
        project_summary = {"test_envelope": Path(tmp_file.name)}
        
        # Create a minimal configuration
        class MockConfig:
            envelope_structure = "test_envelope"
        
        class MockModelManager:
            def __init__(self):
                self.project_summary = project_summary
                self.configuration = MockConfig()
        
        model_manager = MockModelManager()
        
        # Test that the function raises ValueError when no cell ID is found
        with pytest.raises(ValueError, match="Could not find the first cell ID"):
            _get_envelope_structure_first_cell_id(model_manager)
        
        # Clean up
        Path(tmp_file.name).unlink()



