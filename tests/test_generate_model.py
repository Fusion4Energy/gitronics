from pathlib import Path

from gitronics.generate_model import generate_model

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"
TEST_RESOURCES_PATH = Path(__file__).parent / "test_resources"


def test_generate_model(tmpdir):
    generate_model(
        root_folder_path=VALID_PROJECT_PATH,
        configuration_name="valid_configuration",
        write_path=tmpdir,
    )
    with open(tmpdir / "assembled.mcnp") as infile:
        result_text = infile.read()

    expected_file = TEST_RESOURCES_PATH / "expected_file_valid_configuration.mcnp"
    with open(expected_file) as infile:
        expected_text = infile.read()

    result_lines = result_text.splitlines()
    expected_lines = expected_text.splitlines()
    for i in range(len(expected_lines)):
        assert result_lines[i] == expected_lines[i]

    # Check that metadata was generated
    with open(tmpdir / "gitronics_metadata.json") as infile:
        metadata = infile.read()
    assert "version" in metadata


def test_envelope_left_empty_in_configuration(tmpdir):
    generate_model(
        root_folder_path=VALID_PROJECT_PATH,
        configuration_name="small_override",
        write_path=tmpdir,
    )
    with open(tmpdir / "assembled.mcnp") as infile:
        result_text = infile.read()

    assert "$ FILL = my_envelope_name_1" in result_text
