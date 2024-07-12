from pathlib import Path

from git_for_mcnp.generate_model import generate_model

CONFIGURATION_PATH = (
    Path(__file__).resolve().parents[1]
    / "tests"
    / "example_structure"
    / "configuration.csv"
)
PROJECT_PATH = Path(__file__).resolve().parents[1] / "tests" / "example_structure"


def test_generate_model(tmpdir):
    generate_model(
        configuration_csv=CONFIGURATION_PATH,
        project_path=PROJECT_PATH,
        write_path=tmpdir,
    )

    with open(tmpdir / "assembled.i") as infile:
        result_text = infile.read()

    expected_file = (
        Path(__file__).resolve().parents[0]
        / "example_structure"
        / "expected_file_configuration_1.i"
    )
    with open(expected_file) as infile:
        expected_text = infile.read()

    assert result_text == expected_text
