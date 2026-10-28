# T2-02 결과

생성: `python3 dev-docs/experiments/T2-02/run.py` · Z3 version 4.15.4 - 64 bit · rlimit 5000000 · 시드 [0, 1, 2](+ 시드 0 반복)

판정은 시드 0·1·2의 답(모두 같으면 하나만). rlimit은 시드별 값(최소~최대). `unknown`은 한도 안에 결정하지 못한 것이다.

| 묶음 | 크기 | 변형 | 인코딩 | 기대 | 판정 | rlimit(최소~최대) | 질의 바이트 | 반복 같음 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SN | k=1 | proof | bits | unsat | unsat | 3408~3408 | 1082 | True |
| SN | k=1 | proof | opaque | unsat | unsat | 3352~3352 | 1829 | True |
| SN | k=1 | bug | bits | sat | sat | 4428~4604 | 1088 | True |
| SN | k=1 | bug | opaque | sat | sat | 4382~4558 | 1835 | True |
| SN | k=4 | proof | bits | unsat | unsat | 6487~13295 | 2645 | True |
| SN | k=4 | proof | opaque | unsat | unsat | 6411~18231 | 3302 | True |
| SN | k=4 | bug | bits | sat | sat | 6970~13384 | 2651 | True |
| SN | k=4 | bug | opaque | sat | sat | 6755~14680 | 3308 | True |
| SN | k=16 | proof | bits | unsat | unsat | 18493~18554 | 8973 | True |
| SN | k=16 | proof | opaque | unsat | unsat | 17866~36546 | 9270 | True |
| SN | k=16 | bug | bits | sat | sat | 30312~34923 | 8979 | True |
| SN | k=16 | bug | opaque | sat | sat | 18534~31038 | 9276 | True |
| SN | k=64 | proof | bits | unsat | unsat | 75529~128732 | 34557 | True |
| SN | k=64 | proof | opaque | unsat | unsat | 71555~99992 | 33414 | True |
| SN | k=64 | bug | bits | sat | sat | 100633~290082 | 34563 | True |
| SN | k=64 | bug | opaque | sat | sat | 77063~291870 | 33420 | True |
| NB | k=1 | proof | bits | unsat | unsat | 680~680 | 3426 | True |
| NB | k=1 | proof | opaque | unsat | unsat | 596~596 | 4044 | True |
| NB | k=1 | bug | bits | sat | sat | 2060~2060 | 3426 | True |
| NB | k=1 | bug | opaque | sat | sat | 1878~1878 | 4044 | True |
| NB | k=4 | proof | bits | unsat | unsat | 2656~2656 | 12088 | True |
| NB | k=4 | proof | opaque | unsat | unsat | 2320~2320 | 12229 | True |
| NB | k=4 | bug | bits | sat | sat | 8146~8146 | 12088 | True |
| NB | k=4 | bug | opaque | sat | sat | 7418~7418 | 12229 | True |
| NB | k=10 | proof | bits | unsat | unsat | 6604~6604 | 29446 | True |
| NB | k=10 | proof | opaque | unsat | unsat | 5764~5764 | 28633 | True |
| NB | k=10 | bug | bits | sat | sat | 19966~19966 | 29446 | True |
| NB | k=10 | bug | opaque | sat | sat | 18326~18326 | 28633 | True |
| NB | k=40 | proof | bits | unsat | unsat | 26344~26344 | 118995 | True |
| NB | k=40 | proof | opaque | unsat | unsat | 22984~22984 | 113412 | True |
| NB | k=40 | bug | bits | sat | sat | 72526~72526 | 118995 | True |
| NB | k=40 | bug | opaque | sat | sat | 64586~64586 | 113412 | True |
| MB | k=1 | proof | bits | unsat | unsat | 147~147 | 1675 | True |
| MB | k=1 | proof | opaque | unsat | unsat | 448~448 | 2423 | True |
| MB | k=1 | bug | bits | sat | sat | 284~284 | 1674 | True |
| MB | k=1 | bug | opaque | sat | sat | 3222~3222 | 2422 | True |
| MB | k=4 | proof | bits | unsat | unsat | 1870~1870 | 3797 | True |
| MB | k=4 | proof | opaque | unsat | unsat | 2172~2172 | 4944 | True |
| MB | k=4 | bug | bits | sat | unknown | 5000198~5000198 | 3796 | True |
| MB | k=4 | bug | opaque | sat | sat | 144297~188900 | 4943 | True |
| MB | k=16 | proof | bits | unsat | unknown | 5000846~5000846 | 15274 | True |
| MB | k=16 | proof | opaque | unsat | unsat | 173078~205459 | 16660 | True |
| MB | k=16 | bug | bits | sat | unknown | 5000846~5000846 | 15274 | True |
| MB | k=16 | bug | opaque | sat | unknown | 5000775~5000775 | 16660 | True |
| FP1 | - | proof | bits | unsat | unsat | 1301087~1301090 | 452 | True |
| FP1 | - | proof | opaque | sat (증명 불가) | sat | 2527~3362 | 1225 | True |
| FPk | k=1 | proof | bits | unsat | unsat | 2452468~2598137 | 1047 | True |
| FPk | k=1 | proof | opaque | sat (증명 불가) | sat | 5108~5108 | 1628 | True |
| FPk | k=1 | bug | bits | sat | unknown | 5000098~5000098 | 1727 | True |
| FPk | k=1 | bug | opaque | sat | sat | 9501~9517 | 2212 | True |
| FPk | k=2 | proof | bits | unsat | unknown | 5000090~5000090 | 1727 | True |
| FPk | k=2 | proof | opaque | sat (증명 불가) | sat | 9208~9213 | 2212 | True |
| FPk | k=2 | bug | bits | sat | unknown | 5000132~5000132 | 2407 | True |
| FPk | k=2 | bug | opaque | sat | sat | 13429~13429 | 2796 | True |
| FPk | k=4 | proof | bits | unsat | unknown | 5000161~5000161 | 2942 | True |
| FPk | k=4 | proof | opaque | sat (증명 불가) | sat | 17138~17159 | 3380 | True |
| FPk | k=4 | bug | bits | sat | unknown | 5000203~5000203 | 3622 | True |
| FPk | k=4 | bug | opaque | sat | sat | 21430~21465 | 3964 | True |
| FPk | k=8 | proof | bits | unsat | unknown | 5000297~5000297 | 5662 | True |
| FPk | k=8 | proof | opaque | sat (증명 불가) | sat | 34734~34763 | 5787 | True |
| FPk | k=8 | bug | bits | sat | unknown | 5000339~5000339 | 6342 | True |
| FPk | k=8 | bug | opaque | sat | sat | 37886~37932 | 6371 | True |
