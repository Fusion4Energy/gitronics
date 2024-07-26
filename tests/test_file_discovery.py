from pathlib import Path

import pytest
from git_for_mcnp.file_discovery import get_included_paths

CONFIGURATION_PATH = (
    Path(__file__).resolve().parents[1]
    / "tests"
    / "example_structure"
    / "configuration.csv"
)
PROJECT_PATH = Path(__file__).resolve().parents[1] / "tests" / "example_structure"


def test_get_included_paths():
    result_paths = get_included_paths(CONFIGURATION_PATH, PROJECT_PATH)
    some_expected_file_paths = [
        PROJECT_PATH / "models" / "filler_model_1.mcnp",
        PROJECT_PATH / "models" / "filler_model_2.mcnp",
        PROJECT_PATH / "data_cards" / "materials.mat",
        PROJECT_PATH / "data_cards" / "volumetric_source.source",
    ]

    for file_path in some_expected_file_paths:
        assert file_path in result_paths

    not_expected_file_paths = [
        PROJECT_PATH / "models" / "path_to_be_excluded" / "filler_model_3.mcnp",
        PROJECT_PATH / "data_cards" / "materials_bad_library.mat",
        PROJECT_PATH / "data_cards" / "fine_mesh.tally",
        PROJECT_PATH / "data_cards" / "ring_source.source",
    ]
    for file_path in not_expected_file_paths:
        assert file_path not in result_paths


def test_wrong_configuration_include_value():
    with pytest.raises(ValueError) as e:
        get_included_paths(
            PROJECT_PATH / "wrong_configuration_include_value.csv", PROJECT_PATH
        )
    assert "Include column should be either 'YES' or 'NO'" in str(e.value)
