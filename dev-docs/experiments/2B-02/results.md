# 2B-02 결과

생성: `python3 dev-docs/experiments/2B-02/run.py` · Z3 version 4.15.4 - 64 bit · rlimit 5000000 · 시드 [0, 1, 2](+ 시드 0 반복)

판정은 시드 0·1·2의 답(모두 같으면 하나만). rlimit은 시드별 값(최소~최대). 반복은 시드 0을 다시 돌려 답·rlimit·모델이 같은지.

| 묶음 | 크기 | 변형 | 인코딩 | 기대 | 판정 | rlimit(최소~최대) | 질의 바이트 | 반복 같음 | 모델 항 | 모델 글자 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| S1_update_chain | k=4,n=8 | proof | adt | unsat | unsat | 2226~2226 | 2083 | True | 0 | 0 |
| S1_update_chain | k=4,n=8 | proof | flat | unsat | unsat | 362~362 | 1241 | True | 0 | 0 |
| S1_update_chain | k=4,n=8 | proof | hybrid | unsat | unsat | 386~386 | 1291 | True | 0 | 0 |
| S1_update_chain | k=4,n=8 | bug | adt | sat | sat | 2808~2808 | 2083 | True | 1 | 26 |
| S1_update_chain | k=4,n=8 | bug | flat | sat | sat | 1142~1142 | 1241 | True | 4 | 48 |
| S1_update_chain | k=4,n=8 | bug | hybrid | sat | sat | 1272~1272 | 1291 | True | 1 | 26 |
| S1_update_chain | k=4,n=32 | proof | adt | unsat | unsat | 5514~5514 | 4741 | True | 0 | 0 |
| S1_update_chain | k=4,n=32 | proof | flat | unsat | unsat | 458~458 | 1385 | True | 0 | 0 |
| S1_update_chain | k=4,n=32 | proof | hybrid | unsat | unsat | 482~482 | 1435 | True | 0 | 0 |
| S1_update_chain | k=4,n=32 | bug | adt | sat | sat | 6672~6672 | 4741 | True | 1 | 26 |
| S1_update_chain | k=4,n=32 | bug | flat | sat | sat | 1238~1238 | 1385 | True | 4 | 48 |
| S1_update_chain | k=4,n=32 | bug | hybrid | sat | sat | 1368~1368 | 1435 | True | 1 | 26 |
| S1_update_chain | k=4,n=128 | proof | adt | unsat | unsat | 18666~18666 | 15571 | True | 0 | 0 |
| S1_update_chain | k=4,n=128 | proof | flat | unsat | unsat | 842~842 | 1961 | True | 0 | 0 |
| S1_update_chain | k=4,n=128 | proof | hybrid | unsat | unsat | 1798~1798 | 2215 | True | 0 | 0 |
| S1_update_chain | k=4,n=128 | bug | adt | sat | sat | 22128~22128 | 15571 | True | 1 | 26 |
| S1_update_chain | k=4,n=128 | bug | flat | sat | sat | 1622~1622 | 1961 | True | 4 | 48 |
| S1_update_chain | k=4,n=128 | bug | hybrid | sat | sat | 2224~2224 | 2215 | True | 1 | 26 |
| S1_update_chain | k=16,n=8 | proof | adt | unsat | unsat | 6726~6726 | 5539 | True | 0 | 0 |
| S1_update_chain | k=16,n=8 | proof | flat | unsat | unsat | 866~866 | 3443 | True | 0 | 0 |
| S1_update_chain | k=16,n=8 | proof | hybrid | unsat | unsat | 962~962 | 3451 | True | 0 | 0 |
| S1_update_chain | k=16,n=8 | bug | adt | sat | sat | 8688~8688 | 5539 | True | 1 | 73 |
| S1_update_chain | k=16,n=8 | bug | flat | sat | sat | 4178~4178 | 3443 | True | 16 | 186 |
| S1_update_chain | k=16,n=8 | bug | hybrid | sat | sat | 4212~4212 | 3451 | True | 1 | 73 |
| S1_update_chain | k=16,n=32 | proof | adt | unsat | unsat | 15198~15198 | 12361 | True | 0 | 0 |
| S1_update_chain | k=16,n=32 | proof | flat | unsat | unsat | 962~962 | 3587 | True | 0 | 0 |
| S1_update_chain | k=16,n=32 | proof | hybrid | unsat | unsat | 1058~1058 | 3595 | True | 0 | 0 |
| S1_update_chain | k=16,n=32 | bug | adt | sat | sat | 18888~18888 | 12361 | True | 1 | 73 |
| S1_update_chain | k=16,n=32 | bug | flat | sat | sat | 4274~4274 | 3587 | True | 16 | 186 |
| S1_update_chain | k=16,n=32 | bug | hybrid | sat | sat | 4308~4308 | 3595 | True | 1 | 73 |
| S1_update_chain | k=16,n=128 | proof | adt | unsat | unsat | 49086~49086 | 40243 | True | 0 | 0 |
| S1_update_chain | k=16,n=128 | proof | flat | unsat | unsat | 1346~1346 | 4163 | True | 0 | 0 |
| S1_update_chain | k=16,n=128 | proof | hybrid | unsat | unsat | 1442~1442 | 4171 | True | 0 | 0 |
| S1_update_chain | k=16,n=128 | bug | adt | sat | sat | 59688~59688 | 40243 | True | 1 | 73 |
| S1_update_chain | k=16,n=128 | bug | flat | sat | sat | 4658~4658 | 4163 | True | 16 | 186 |
| S1_update_chain | k=16,n=128 | bug | hybrid | sat | sat | 4692~4692 | 4171 | True | 1 | 73 |
| E1_match | k=2 | proof | adt | unsat | unsat | 1671~1683 | 1280 | True | 0 | 0 |
| E1_match | k=2 | proof | flat | unsat | unsat | 2438~2521 | 1327 | True | 0 | 0 |
| E1_match | k=2 | bug | adt | sat | sat | 1738~1789 | 1280 | True | 1 | 19 |
| E1_match | k=2 | bug | flat | sat | sat | 2502~2566 | 1327 | True | 4 | 52 |
| E1_match | k=8 | proof | adt | unsat | unsat | 8200~9013 | 4154 | True | 0 | 0 |
| E1_match | k=8 | proof | flat | unsat | unsat | 12769~14085 | 4267 | True | 0 | 0 |
| E1_match | k=8 | bug | adt | sat | sat | 8778~9402 | 4154 | True | 1 | 19 |
| E1_match | k=8 | bug | flat | sat | sat | 12838~13306 | 4267 | True | 16 | 208 |
| E1_match | k=32 | proof | adt | unsat | unsat | 46092~49110 | 16023 | True | 0 | 0 |
| E1_match | k=32 | proof | flat | unsat | unsat | 69759~85532 | 16489 | True | 0 | 0 |
| E1_match | k=32 | bug | adt | sat | sat | 46095~53614 | 16023 | True | 1 | 19 |
| E1_match | k=32 | bug | flat | sat | sat | 65868~78362 | 16489 | True | 64 | 876 |
| E2_option_chain | n=8 | proof | adt | unsat | unsat | 3294~3298 | 1830 | True | 0 | 0 |
| E2_option_chain | n=8 | proof | flat | unsat | unsat | 2455~2455 | 2364 | True | 0 | 0 |
| E2_option_chain | n=8 | bug | adt | sat | sat | 3929~3933 | 1829 | True | 1 | 18 |
| E2_option_chain | n=8 | bug | flat | sat | sat | 1922~1922 | 2363 | True | 2 | 29 |
| E2_option_chain | n=32 | proof | adt | unsat | unsat | 11214~11218 | 5280 | True | 0 | 0 |
| E2_option_chain | n=32 | proof | flat | unsat | unsat | 15091~15229 | 7750 | True | 0 | 0 |
| E2_option_chain | n=32 | bug | adt | sat | sat | 13865~13869 | 5279 | True | 1 | 18 |
| E2_option_chain | n=32 | bug | flat | sat | sat | 15232~15733 | 7749 | True | 2 | 29 |
| E2_option_chain | n=128 | proof | adt | unsat | unsat | 42894~42898 | 19248 | True | 0 | 0 |
| E2_option_chain | n=128 | proof | flat | unsat | unsat | 89152~91361 | 29457 | True | 0 | 0 |
| E2_option_chain | n=128 | bug | adt | sat | sat | 53609~53613 | 19247 | True | 1 | 18 |
| E2_option_chain | n=128 | bug | flat | sat | sat | 87429~91695 | 29456 | True | 2 | 29 |
| E3_rebuild_eq | k=2 | proof | adt | unsat | unsat | 1087~1088 | 1048 | True | 0 | 0 |
| E3_rebuild_eq | k=2 | proof | flat | unsat | unsat | 2401~2476 | 1234 | True | 0 | 0 |
| E3_rebuild_eq | k=2 | bug | adt | sat | sat | 1131~1153 | 1048 | True | 1 | 43 |
| E3_rebuild_eq | k=2 | bug | flat | sat | sat | 2487~2610 | 1234 | True | 4 | 60 |
| E3_rebuild_eq | k=8 | proof | adt | unsat | unsat | 4371~4496 | 3052 | True | 0 | 0 |
| E3_rebuild_eq | k=8 | proof | flat | unsat | unsat | 12756~12854 | 3754 | True | 0 | 0 |
| E3_rebuild_eq | k=8 | bug | adt | sat | sat | 4342~4363 | 3052 | True | 1 | 43 |
| E3_rebuild_eq | k=8 | bug | flat | sat | sat | 11007~11385 | 3754 | True | 16 | 216 |
| E3_rebuild_eq | k=32 | proof | adt | unsat | unsat | 19890~21987 | 11331 | True | 0 | 0 |
| E3_rebuild_eq | k=32 | proof | flat | unsat | unsat | 72707~75171 | 14231 | True | 0 | 0 |
| E3_rebuild_eq | k=32 | bug | adt | sat | sat | 17109~17119 | 11331 | True | 1 | 43 |
| E3_rebuild_eq | k=32 | bug | flat | sat | sat | 44518~47911 | 14231 | True | 64 | 884 |
| E4_congruence | k=2 | proof | adt | unsat | unsat | 240~240 | 704 | True | 0 | 0 |
| E4_congruence | k=2 | proof | flat | unsat | unsat | 2282~2371 | 2026 | True | 0 | 0 |
| E4_congruence | k=2 | proof | flatj | unsat | sat ⚠ | 1777~1809 | 1778 | True | 8 | 243 |
| E4_congruence | k=8 | proof | adt | unsat | unsat | 240~240 | 956 | True | 0 | 0 |
| E4_congruence | k=8 | proof | flat | unsat | unsat | 10334~10460 | 6874 | True | 0 | 0 |
| E4_congruence | k=8 | proof | flatj | unsat | sat ⚠ | 6691~6859 | 5774 | True | 32 | 1055 |
| E4_congruence | k=32 | proof | adt | unsat | unsat | 240~240 | 2030 | True | 0 | 0 |
| E4_congruence | k=32 | proof | flat | unsat | unsat | 59897~60525 | 26994 | True | 0 | 0 |
| E4_congruence | k=32 | proof | flatj | unsat | sat ⚠ | 26338~26503 | 22354 | True | 128 | 4643 |
| A1_array_invariant | k=2 | proof | adt | unsat | unsat | 2404~2428 | 1306 | True | 0 | 0 |
| A1_array_invariant | k=2 | proof | flat | unsat | unsat | 2275~2299 | 1330 | True | 0 | 0 |
| A1_array_invariant | k=2 | bug | adt | sat | sat | 2242~2242 | 1270 | True | 4 | 108 |
| A1_array_invariant | k=2 | bug | flat | sat | sat | 2132~2132 | 1302 | True | 6 | 131 |
| A1_array_invariant | k=8 | proof | adt | unsat | unsat | 3442~3466 | 1918 | True | 0 | 0 |
| A1_array_invariant | k=8 | proof | flat | unsat | unsat | 3061~3085 | 2374 | True | 0 | 0 |
| A1_array_invariant | k=8 | bug | adt | sat | sat | 3316~3316 | 1882 | True | 4 | 414 |
| A1_array_invariant | k=8 | bug | flat | sat | sat | 2942~2942 | 2346 | True | 18 | 329 |
| A1_array_invariant | k=32 | proof | adt | unsat | unsat | 7594~7618 | 4432 | True | 0 | 0 |
| A1_array_invariant | k=32 | proof | flat | unsat | unsat | 6205~6229 | 6682 | True | 0 | 0 |
| A1_array_invariant | k=32 | bug | adt | sat | sat | 7612~7612 | 4396 | True | 4 | 1566 |
| A1_array_invariant | k=32 | bug | flat | sat | sat | 6182~6182 | 6654 | True | 66 | 1165 |
| A2_field_write_frame | k=2 | proof | adt | unsat | unsat | 1390~1390 | 1171 | True | 0 | 0 |
| A2_field_write_frame | k=2 | proof | flat | unsat | unsat | 1275~1275 | 1125 | True | 0 | 0 |
| A2_field_write_frame | k=2 | bug | adt | sat | sat | 1379~1379 | 1155 | True | 4 | 76 |
| A2_field_write_frame | k=2 | bug | flat | sat | sat | 1254~1254 | 1109 | True | 5 | 95 |
| A2_field_write_frame | k=8 | proof | adt | unsat | unsat | 1682~1682 | 3115 | True | 0 | 0 |
| A2_field_write_frame | k=8 | proof | flat | unsat | unsat | 1375~1375 | 1977 | True | 0 | 0 |
| A2_field_write_frame | k=8 | bug | adt | sat | sat | 1695~1695 | 3099 | True | 4 | 90 |
| A2_field_write_frame | k=8 | bug | flat | sat | sat | 1354~1354 | 1961 | True | 11 | 227 |
| A2_field_write_frame | k=32 | proof | adt | unsat | unsat | 5057~5057 | 3912 | True | 0 | 0 |
| A2_field_write_frame | k=32 | proof | flat | unsat | unsat | 1759~1759 | 5493 | True | 0 | 0 |
| A2_field_write_frame | k=32 | bug | adt | sat | sat | 5242~5242 | 3896 | True | 4 | 162 |
| A2_field_write_frame | k=32 | bug | flat | sat | sat | 1738~1738 | 5477 | True | 35 | 777 |
| A3_option_array | - | proof | adt | unsat | unsat | 2491~2491 | 1481 | True | 0 | 0 |
| A3_option_array | - | proof | flat | unsat | unsat | 2520~2573 | 1455 | True | 0 | 0 |
| A3_option_array | - | bug | adt | sat | sat | 2972~2995 | 1499 | True | 4 | 51 |
| A3_option_array | - | bug | flat | sat | sat | 3242~3462 | 1467 | True | 5 | 79 |

## 반례 예(시드 0, z3 `get-value` 원문)

- S1_update_chain k=4,n=8 bug `adt`: `((|p| (P 1000 0 712 217)))`
- S1_update_chain k=4,n=8 bug `flat`: `((|p.f0| 1000) (|p.f1| 0) (|p.f2| 0) (|p.f3| 0))`
- S1_update_chain k=4,n=8 bug `hybrid`: `((|p| (P 1000 0 712 217)))`
- E1_match k=2 bug `adt`: `((|e| (|E::V0| 0)))`
- E1_match k=2 bug `flat`: `((|e.tag| 0) (|e.V0.0| 0) (|e.V1.0| 0) (|e.V1.1| 0))`
- E2_option_chain n=8 bug `adt`: `((|x| (Some 992)))`
- E2_option_chain n=8 bug `flat`: `((|x.some| true) (|x.v| 992))`
- E3_rebuild_eq k=2 bug `adt`: `((|e| (|E::V1| 0 (- 9223372036854754570))))`
- E3_rebuild_eq k=2 bug `flat`: `((|e.tag| 1) (|e.V0.0| 0) (|e.V1.0| (- 2)) (|e.V1.1| (- 1)))`
- E4_congruence k=2 proof `flatj`: `((|e1.tag| 0) (|e2.tag| 0) (|e1.V0.0| (- 9223372036854766953)) (|e1.V1.0| (- 9223372036854775810)) (|e1.V1.1| (- 9223372036854777606)) (|e2.V0.0| (- 9223372036854766953)) (|e2.V1.0| (- 9223372036854775809)) (|e2.V1.1| (- 9223372036854777605)))`
- A1_array_invariant k=2 bug `adt`: `((|i| 0) (|a.len| 1) (|x| (P (- 9223372036854774571) (- 9223372036854774572))) ((select |a| 0) (P 4 (- 1))))`
- A1_array_invariant k=2 bug `flat`: `((|i| 0) (|a.len| 1) (|x.f0| (- 9223372036854774571)) (|x.f1| (- 9223372036854774572)) ((select |a.f0| 0) 4) ((select |a.f1| 0) 5))`
- A2_field_write_frame k=2 bug `adt`: `((|i| 0) (|a.len| 1) (|v| (- 9223372036854775770)) ((select |a| 0) (P 5 4)))`
- A2_field_write_frame k=2 bug `flat`: `((|i| 0) (|a.len| 1) (|v| (- 9223372036854775770)) ((select |a.f0| 0) 5) ((select |a.f1| 0) 0))`
- A3_option_array - bug `adt`: `((|i| 0) (|a.len| 1) (|x| 0) ((select |a| 0) None))`
- A3_option_array - bug `flat`: `((|i| 38) (|a.len| 39) (|x| 0) ((select |a.some| 0) true) ((select |a.v| 0) 5))`
