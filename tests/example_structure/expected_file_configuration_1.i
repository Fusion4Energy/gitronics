C Filler model 1
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
