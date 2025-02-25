from pathlib import Path

import pytest

from gitronics.model_manager import ModelManager

PROJECT_PATH = Path(__file__).resolve().parents[1] / "tests" / "example_structure"
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


def test_get_included_paths():
    model_manager = ModelManager(PROJECT_PATH, CONFIGURATION_PATH, PROJECT_SUMMARY_PATH)
    result_paths = model_manager.get_included_paths()
    some_expected_file_paths = [
        PROJECT_PATH / "models" / "main_input.mcnp",
        PROJECT_PATH / "models" / "filler_model_1.mcnp",
        PROJECT_PATH / "models" / "filler_model_2.mcnp",
        PROJECT_PATH / "data_cards" / "materials.mat",
        PROJECT_PATH / "data_cards" / "fine_mesh.tally",
        PROJECT_PATH / "data_cards" / "volumetric_source.source",
    ]

    for file_path in some_expected_file_paths:
        assert file_path in result_paths

    not_expected_file_paths = [
        PROJECT_PATH / "models" / "path_to_be_excluded" / "filler_model_3.mcnp",
        PROJECT_PATH / "data_cards" / "materials_bad_library.mat",
        PROJECT_PATH / "data_cards" / "ring_source.source",
    ]
    for file_path in not_expected_file_paths:
        assert file_path not in result_paths


def test_project_summary_repeated_names():
    project_summary_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "project_summary_repeated_name.csv"
    )
    with pytest.raises(ValueError):
        ModelManager(PROJECT_PATH, CONFIGURATION_PATH, project_summary_path)


def test_project_summary_path_not_found():
    project_summary_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "project_summary_path_not_found.csv"
    )
    with pytest.raises(ValueError):
        ModelManager(PROJECT_PATH, CONFIGURATION_PATH, project_summary_path)


def test_override_configuration_small():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_override_small.yml"
    )
    model_manager = ModelManager(
        PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH
    )
    result_paths = model_manager.get_included_paths()
    assert PROJECT_PATH / "models" / "filler_model_1.mcnp" in result_paths
    assert PROJECT_PATH / "data_cards" / "ring_source.source" in result_paths
    assert PROJECT_PATH / "data_cards" / "materials_bad_library.mat" in result_paths

    assert PROJECT_PATH / "models" / "filler_model_2.mcnp" not in result_paths
    assert PROJECT_PATH / "data_cards" / "my_transform.transform" not in result_paths
    assert PROJECT_PATH / "data_cards" / "fine_mesh.tally" not in result_paths
    assert PROJECT_PATH / "data_cards" / "materials.mat" not in result_paths


def test_override_configuration_not_found():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_override_not_found.yml"
    )

    with pytest.raises(KeyError):
        ModelManager(PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH)


def test_envelope_structure_not_found():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_envelope_structure_not_found.yml"
    )
    model_manager = ModelManager(
        PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH
    )
    with pytest.raises(KeyError):
        model_manager.get_included_paths()


def test_envelope_filler_not_found():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_envelope_filler_not_found.yml"
    )
    model_manager = ModelManager(
        PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH
    )
    with pytest.raises(KeyError):
        model_manager.get_included_paths()


def test_source_not_found():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_source_not_found.yml"
    )
    model_manager = ModelManager(
        PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH
    )
    with pytest.raises(KeyError):
        model_manager.get_included_paths()


def test_tally_not_found():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_tally_not_found.yml"
    )
    model_manager = ModelManager(
        PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH
    )
    with pytest.raises(KeyError):
        model_manager.get_included_paths()


def test_material_not_found():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_material_not_found.yml"
    )
    model_manager = ModelManager(
        PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH
    )
    with pytest.raises(KeyError):
        model_manager.get_included_paths()


def test_transform_not_found():
    configuration_override_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_transform_not_found.yml"
    )
    model_manager = ModelManager(
        PROJECT_PATH, configuration_override_path, PROJECT_SUMMARY_PATH
    )
    with pytest.raises(KeyError):
        model_manager.get_included_paths()


def test_get_universe_id():
    model_manager = ModelManager(PROJECT_PATH, CONFIGURATION_PATH, PROJECT_SUMMARY_PATH)
    model_1_id = 121
    model_2_id = 125
    assert model_manager.get_universe_id("My envelope name 1") == model_1_id
    assert model_manager.get_universe_id("Envelope_2") == model_2_id


def test_get_universe_id_not_found():
    configuration_path = (
        Path(__file__).resolve().parents[1]
        / "tests"
        / "example_structure"
        / "configurations/configuration_model_3.yml"
    )
    model_manager = ModelManager(PROJECT_PATH, configuration_path, PROJECT_SUMMARY_PATH)
    with pytest.raises(ValueError):
        model_manager.get_universe_id("My envelope name 1")
