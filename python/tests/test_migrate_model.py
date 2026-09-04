"""Coverage for the ``migrate_model`` binding, which had none."""

import tempfile
from pathlib import Path

import gitronics
import pytest

REPO_ROOT = Path(__file__).parent.parent.parent
FILLED_MODEL = REPO_ROOT / "resources" / "filled_model.mcnp"

EXPECTED_ARTEFACTS = [
    "configurations/baseline.yaml",
    "reference_model/envelope_structure.mcnp",
    "reference_model/data_cards.source",
    "reference_model/filler_models/universe_1.mcnp",
    "reference_model/filler_models/universe_1.metadata",
    "reference_model/filler_models/universe_2.mcnp",
    "output/assembled.mcnp",
]


def test_migrate_model_creates_a_buildable_project():
    with tempfile.TemporaryDirectory() as out:
        gitronics.migrate_model(FILLED_MODEL, out)
        project = Path(out)
        for artefact in EXPECTED_ARTEFACTS:
            assert (project / artefact).exists(), f"missing {artefact}"


def test_migrate_model_accepts_str_paths():
    with tempfile.TemporaryDirectory() as out:
        gitronics.migrate_model(str(FILLED_MODEL), out)
        assert (Path(out) / "output" / "assembled.mcnp").exists()


def test_migrate_model_records_envelopes_in_the_baseline_config():
    with tempfile.TemporaryDirectory() as out:
        gitronics.migrate_model(FILLED_MODEL, out)
        baseline = (Path(out) / "configurations" / "baseline.yaml").read_text()
        # Both filled level-0 cells become envelopes mapped to their universes.
        assert "envelope_1: universe_1" in baseline
        assert "envelope_2: universe_2" in baseline


def test_migrate_model_missing_input_raises():
    with tempfile.TemporaryDirectory() as out:
        with pytest.raises(RuntimeError):
            gitronics.migrate_model("does_not_exist.mcnp", out)
