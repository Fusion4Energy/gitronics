from pathlib import Path

from gitronics.generate_model import generate_model

ROOT_FOLDER_PATH = Path(__file__).resolve().parents[1] / "tests" / "example_structure"
CONFIGURATION_PATH = (
    Path(__file__).resolve().parents[1]
    / "tests"
    / "example_structure"
    / "configurations/configuration_1.yml"
)
PROJECT_SUMMARY_PATH = (
    Path(__file__).resolve().parents[1]
    / "tests"
    / "example_structure"
    / "project_summary.csv"
)


def test_generate_model(tmpdir):
    generate_model(
        root_folder_path=ROOT_FOLDER_PATH,
        configuration_file_path=CONFIGURATION_PATH,
        project_summary_path=PROJECT_SUMMARY_PATH,
        # write_path=Path(""),
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
