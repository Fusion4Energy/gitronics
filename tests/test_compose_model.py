# ruff: noqa: E501
import logging
from pathlib import Path

from gitronics.compose_model import compose_model
from gitronics.file_readers import ParsedBlocks

PROJECT_PATH = Path(__file__).resolve().parents[1] / "tests" / "example_structure"


def test_compose_model():
    text_result = compose_model(BLOCKS)
    assert text_result == EXPECTED_TEXT


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
EXPECTED_TEXT = """C Filler model 1
10   14    -7.89      10 -20 40 -30
           imp:n=1.0   imp:p=1.0  u=121
20     0      10 -20 80 730 -70 -60 -50
           imp:n=1.0   imp:p=1.0   u=121
C Filler model 2
100   14    -7.89      100 -200 400 -300
           imp:n=1.0   imp:p=1.0  u=125
200     0      100 -200 800 7300 -700 -600 -500
           imp:n=1.0   imp:p=1.0   u=125

C Surfaces model 1
10     PZ  -1.4030000e+02
20     PZ   4.6300000e+01
30     CZ    160.700000
40     CZ    144.900000
C Surfaces model 2
100    PZ  -1.4030000e+02
200    PZ   4.6300000e+01
300    CZ    160.700000
400    CZ    144.900000

C Materials
c Silicon (Pure Si)
m14 
     14028.31c   0.922
     14029.31c   0.047
     14030.31c   0.031
c
C This file includes the sdef and all other parameters like NPS 
mode  n 
sdef sur 398 nrm=-1 dir=d1 wgt=132732289.6141
sb1    -21  2
lost 1000
prdmp j 1e7 
nps  1e9 
"""
