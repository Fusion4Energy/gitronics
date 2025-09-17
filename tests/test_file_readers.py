from pathlib import Path

import pytest

from gitronics.file_readers import read_files

VALID_PROJECT_PATH = Path(__file__).parent / "test_resources" / "valid_project"
WRONG_FILES_PATH = Path(__file__).parent / "test_resources" / "wrong_files"


def test_read_files_mcnp():
    parsed_blocks = read_files(
        [
            VALID_PROJECT_PATH / "models" / "envelope_structure.mcnp",
            VALID_PROJECT_PATH / "models" / "filler_model_1.mcnp",
        ]
    )
    assert parsed_blocks.cells[1] == MAIN_INPUT_CELLS
    assert parsed_blocks.cells[10] == FILLER_MODEL_1_CELLS
    assert parsed_blocks.surfaces[10] == FILLER_MODEL_1_SURFACES


def test_read_wrong_mcnp_files():
    with pytest.raises(
        ValueError,
        match="File .* does not contain the two blocks: cells and surfaces.",
    ):
        read_files([WRONG_FILES_PATH / "only_cells_block.mcnp"])

    with pytest.raises(
        ValueError,
        match="Could not parse the first cell ID value in .*",
    ):
        read_files([WRONG_FILES_PATH / "wrong_cells_block.mcnp"])

    with pytest.raises(
        ValueError,
        match="Could not parse the first surface ID value in .*",
    ):
        read_files([WRONG_FILES_PATH / "wrong_surfaces_block.mcnp"])


def test_read_files_data_cards():
    parsed_blocks = read_files(
        [
            Path(VALID_PROJECT_PATH / "data_cards" / "materials.mat"),
            Path(VALID_PROJECT_PATH / "data_cards" / "my_transform.transform"),
            Path(VALID_PROJECT_PATH / "data_cards" / "fine_mesh.tally"),
            Path(VALID_PROJECT_PATH / "data_cards" / "volumetric_source.source"),
        ]
    )
    assert parsed_blocks.materials[14] == MATERIALS_MAT
    assert parsed_blocks.tallies[24] == FINE_MESH_TALLY
    assert parsed_blocks.source == VOLUMETRIC_SOURCE


def test_read_files_wrong_data_card():
    with pytest.raises(
        ValueError,
        match="Could not parse the first ID value in file .*",
    ):
        read_files([WRONG_FILES_PATH / "wrong_data_card.mat"])


def test_read_file_wrong_suffix():
    with pytest.raises(
        ValueError,
        match="Unknown file suffix for: .*",
    ):
        read_files([WRONG_FILES_PATH / "wrong_suffix.wrong"])


MAIN_INPUT_CELLS = """Title of the MCNP model
C
C
C *************************************************************
C Original Step file : Design1.stp
C
C Creation Date : 2024-02-21 11:49:58.182497
C Solid Cells   : 9
C Total Cells   : 14
C Surfaces      : 102
C Materials     : 0
C
C **************************************************************
1   14 -7.89      1 -2 4 -3
           imp:n=1.0   imp:p=1.0   
           $/61/SOLID1
2     0      1 -2 8 73 -7 -6 -5
           imp:n=1.0   imp:p=1.0   $ FILL = my_envelope_name_1
           $/61/SOLID0011
3     0      1 -2 8 9 73 -7 -6
           imp:n=1.0   imp:p=1.0   $ FILL = my_envelope_name_1
           $/61/SOLID0021
4     0      1 -2 8 73 -11 -10 -7
           imp:n=1.0   imp:p=1.0   $ FILL = my_envelope_name_1
           $/61/SOLID0031
5     0      1 -2 8 73 -19 18 -7:1 8 73 -21 -20 22 -7:1 8 -23 73 -18 24 -7:1 
           -26 -25 8 73 24 -7:-2 -27 8 -22 74 12 -7:-28 8 75 22 24 -7:1 -30 -29 
           8 73 -7 28:-32 -31 1 8 73 22 -7 28:33 8 -22 76 -13 -7 -34:33 -2 8 74 
           12 -7 -34:-2 34 8 -22 -21 77 -7:-26 -24 8 78 22 -7 27:35 -27 -24 8 
           76 -13 -7:-2 -24 8 77 -18 -7 -35:-2 35 8 74 12 22 -7:-28 -24 8 -14 
           79 22 -7 26:8 80 14 22 -7 26 -36:-2 8 -22 74 -7 27 -36:-2 36 8 74 15 
           -7 -37:-2 37 8 -22 81 -7 -38:8 -22 -14 82 -7 -37:38 8 -22 -16 83 -7 
           -39:-2 38 8 74 17 -7 -39:-2 39 8 -22 78 -7 -33
           imp:n=1.0   imp:p=1.0   $ FILL = envelope_name_2
           $/61/SOLID0041
6     0      1 8 73 43 44 45 -7:1 8 73 44 46 47 -40 -7:1 8 73 44 48 -46 -7:8 
           -49 50 -44 84 -41 -7:-2 8 43 49 -44 85 -7:-2 8 76 -50 51 -44 -7:-2 8 
           74 42 -51 46 -7:8 -51 46 -44 86 -40 -7:-2 8 48 -46 -44 87 -7
           imp:n=1.0   imp:p=1.0  $ FILL = envelope_name_2
           $/61/SOLID0051
7     0      1 8 73 -55 52 53 54 -7:8 -56 -53 76 54 -7:8 -54 -21 53 88 -7:1 -57 
           8 73 -56 53 -7:1 -58 8 73 -21 54 -7:89 -2 -59 8 -53 -21 -7:-60 8 -53 
           -14 -7 90 59:-2 -60 8 74 15 -7 59:8 -56 -54 -53 -41 -7 91 60:-2 -61 
           8 41 74 -7 60:-2 -62 -56 41 8 74 -7
           imp:n=1.0   imp:p=1.0 $ FILL = envelope_name_2
           $/61/SOLID0061
8     0      64 1 8 73 -56 -7 63:65 1 8 73 -18 -7 63:1 66 8 73 18 -7 63:-63 -2 
           66 8 18 -7 92:-2 -61 8 41 74 -18 -7:-63 8 -56 76 -18 -41 -7:-62 -2 8 
           -56 74 41 -7
           imp:n=1.0   imp:p=1.0 $ FILL = envelope_name_2
           $/61/SOLID0071
9     0      1 68 8 73 -7 -69 -67:1 68 8 73 -11 -7 -70:8 93 -11 -41 -7 -68 -67:
           -2 8 41 74 -7 -71 -67:-2 8 41 74 -11 -72 -7
           imp:n=1.0   imp:p=1.0   
           $/61/SOLID0081
C 
C ##########################################################
C              VOID CELLS
C ##########################################################
C 
10    0      94 -95 96 -97 98 -99 ((-76:56:53:-54) (-76:63:56:18:41) ((-44:(46:
           -48) (-43:-45) (-47:40:-46)) (-53:(55:-52:-54) (56:57)) (58:21:-54) 
           (-63:(-18:-66) (18:-65)):-1:-73) (2:(44:(50:-76:-51) (-85:-49:-43) 
           (-87:-48:46)) (-74:(-41:(-60:61) (56:62)) (-15:-59:60) (-41:(56:62) 
           (61:18)) (-42:51:-46)) (-89:53:21:59) (-92:63:-66:-18)) (-84:41:49:
           -50:44) (-86:51:-46:44:40) (-88:54:21:-53) (-90:14:60:53:-59) (-91:
           56:54:53:41:-60) (-9:6:-1:2:-73) (11:10:-1:2:-73):7:-8) (-4:3:-1:2)
           imp:n=1.0   imp:p=1.0   
           $Automatic Generated Void Cell. Enclosure(-527.6840293694487, 527.6840293694487, -4.917823186510304, 523.7361257498748, -252.624, 252.624)
           $Enclosed cells : (1, 3, 4, 6, 7, 8)
11    0      94 -100 101 -96 98 -99 ((21:(20:-73:-1:-22) (-77:-34:2:22)) (56:
           (62:-74:2:-41) (-64:-73:-1:-63) (-76:18:63:41)) ((67:69) (11:70):-73:
           -1:-68) ((67:71) (11:72):-74:2:-41) (-93:11:41:68:67) (6:5:-1:2:-73):
           7:-8) (-4:3:-1:2)
           imp:n=1.0   imp:p=1.0   
           $Automatic Generated Void Cell. Enclosure(-527.6840293694487, 0.0, -533.5717721228954, -4.917823186510304, -252.624, 252.624)
           $Enclosed cells : (1, 2, 5, 8, 9)
12    0      100 -95 101 -96 98 -99 (((-22:(21:20) (32:31:-28)) (-24:(18:23) 
           (26:25)) (19:2:-18) (30:29:-28):-1:-73) ((22:34:-33) (27:24:-35):13:
           -76) ((22:-27:36) (-12:(22:27) (-33:34) (-22:-35)) (-15:37:-36) (-17:
           39:-38):2:-74) ((22:21:-34) (24:18:35):2:-77) (-78:(-39:2:33:22) 
           (-27:26:24:-22)) (-75:28:-22:-24) (-79:28:24:14:-22:-26) (-80:-14:
           -22:-26:36) (-81:2:-37:22:38) (-82:22:14:37) (-83:16:-38:22:39):7:-8)
            (-4:3:-1:2)
           imp:n=1.0   imp:p=1.0   
           $Automatic Generated Void Cell. Enclosure(0.0, 527.6840293694487, -533.5717721228954, -4.917823186510304, -252.624, 252.624)
           $Enclosed cells : (1, 5)
13    0      -102 (-94:95:-101:97:-98:99)
           imp:n=1.0   imp:p=1.0   
           $Graveyard_in
14    0      102
           imp:n=0     imp:p=0     
           $Graveyard
"""  # noqa: E501

FILLER_MODEL_1_CELLS = """C Filler model 1
10   14    -7.89      10 -20 40 -30
           imp:n=1.0   imp:p=1.0  u=121
20     0      10 -20 80 730 -70 -60 -50
           imp:n=1.0   imp:p=1.0   u=121
"""
FILLER_MODEL_1_SURFACES = """C Surfaces model 1
10     PZ  -1.4030000e+02
20     PZ   4.6300000e+01
30     CZ    160.700000
40     CZ    144.900000
"""
MATERIALS_MAT = """C Materials
c Silicon (Pure Si)
m14 
     14028.31c   0.922
     14029.31c   0.047
     14030.31c   0.031
c
"""
FINE_MESH_TALLY = """C Tallies
FMESH24:N  geom=xyz  origin -2200 -2200 -1800 
         imesh 2200  iints 176 
         jmesh 2200  jints 176 
         kmesh 3200  kints 200
         emesh 0.1 20
FM24 1.5e17
FC24 Neutron flux 
C
C Multiplier calculated as:
C 1.5e17*6.976e-2*(1/6.242e12)*60*60*1000/2.32
C source intens*atdens*MeVtoJ*stohur*gtokg*1/gramdens
FMESH34:N  geom=xyz  origin -2200 -2200 -1800 
         imesh 2200  iints 176 
         jmesh 2200  jints 176 
         kmesh 3200  kints 200
FM34 2.60131e9 992 1 -4
FC34 Neutron dose in silica Gy/h
C
"""
VOLUMETRIC_SOURCE = """C This file includes the sdef and all other parameters like NPS 
mode  n 
sdef sur 398 nrm=-1 dir=d1 wgt=132732289.6141
sb1    -21  2
lost 1000
prdmp j 1e7 
nps  1e9 
"""
