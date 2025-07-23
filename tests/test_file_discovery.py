from pathlib import Path

from gitronics.file_discovery import discover_file_paths

PATH_TEST_RESOURCES = Path(__file__).parent / "test_resources"


def test_discover_file_paths():
    file_paths = discover_file_paths(PATH_TEST_RESOURCES / "valid_project")
    assert {
        "fine_mesh",
        "materials",
        "my_transform",
        "volumetric_source",
        "envelope_structure",
        "filler_model_1",
        "filler_model_2",
        "filler_model_3",
        "valid_configuration",
        "overrides_configuration",
    }.issubset(set(file_paths.keys()))
    for file_path in file_paths.values():
        assert file_path.is_file()
