# ruff: noqa: E501
from pathlib import Path

from gitronics.compose_model import compose_model
from gitronics.file_readers import ParsedBlocks

PROJECT_PATH = Path(__file__).resolve().parents[1] / "tests" / "example_structure"


def test_compose_model():
    text_result = compose_model(BLOCKS)

    # expected text
    with open(PROJECT_PATH / "expected_file_configuration_1.i") as infile:
        expected_text = infile.read()

    assert text_result == expected_text


BLOCKS = ParsedBlocks(
    cells={
        10: "C Filler model 1\n10   14    -7.89      10 -20 40 -30\n           imp:n=1.0   imp:p=1.0  u=121\n20     0      10 -20 80 730 -70 -60 -50\n           imp:n=1.0   imp:p=1.0   u=121\n",
        100: "C Filler model 2\n100   14    -7.89      100 -200 400 -300\n           imp:n=1.0   imp:p=1.0  u=125\n200     0      100 -200 800 7300 -700 -600 -500\n           imp:n=1.0   imp:p=1.0   u=125\n",
    },
    surfaces={
        10: "C Surfaces model 1\n10     PZ  -1.4030000e+02\n20     PZ   4.6300000e+01\n30     CZ    160.700000\n40     CZ    144.900000\n",
        100: "C Surfaces model 2\n100    PZ  -1.4030000e+02\n200    PZ   4.6300000e+01\n300    CZ    160.700000\n400    CZ    144.900000\n",
    },
    tallies={},
    materials={
        14: "C Materials\nc Silicon (Pure Si)\nm14 \n     14028.31c   0.922\n     14029.31c   0.047\n     14030.31c   0.031\nc\n"
    },
    transforms={},
    source="C This file includes the sdef and all other parameters like NPS \nmode  n \nsdef sur 398 nrm=-1 dir=d1 wgt=132732289.6141\nsb1    -21  2\nlost 1000\nprdmp j 1e7 \nnps  1e9 \n",
)
