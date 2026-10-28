# 이름 바꾸기 파급 (H-ID)

생성: `python3 dev-docs/experiments/rename-ripple/run.py`. 정의 하나의 이름만 바꿨을 때 해시가 바뀌는 *다른* 정의(그 정의를 부르는 L2 참조자) 수.

| 항목 | 값 |
| --- | ---: |
| 파일 | 47 |
| 정의 | 452 |
| 이름을 바꾼 정의(검사 통과) | 412 |
| 건너뜀(이름 중복·검사 실패) | 40 |
| 다른 정의를 바꾼 이름 바꾸기 | 356 (86%) |
| 본문 해시가 바뀐 참조자(합) | 611 |
| 계약 해시가 바뀐 참조자(합, 그 호출자의 증명 캐시까지 무효) | 377 |
| 참조자가 아닌데 바뀐 정의(같은 낱말, 집계에서 뺌) | 16 |

해시 참조(이름은 메타데이터)라면 위의 참조자 수는 모두 0이다.

| 파일 | 정의 | 바꾼 이름 | 파급 | 계약 해시 파급 |
| --- | ---: | ---: | ---: | ---: |
| `lib/bytes.bmb` | 25 | 25 | 19 | 36 |
| `tests/run/arith_methods.bmb` | 3 | 3 | 2 | 0 |
| `tests/run/array_if.bmb` | 4 | 4 | 3 | 0 |
| `tests/run/arrays.bmb` | 9 | 9 | 8 | 14 |
| `tests/run/basics.bmb` | 9 | 9 | 8 | 7 |
| `tests/run/bools.bmb` | 4 | 4 | 3 | 3 |
| `tests/run/box_places.bmb` | 10 | 10 | 9 | 4 |
| `tests/run/boxes.bmb` | 13 | 13 | 12 | 6 |
| `tests/run/bytes.bmb` | 6 | 6 | 5 | 4 |
| `tests/run/calc.bmb` | 8 | 7 | 6 | 9 |
| `tests/run/conversions.bmb` | 4 | 4 | 3 | 0 |
| `tests/run/expr_tree.bmb` | 11 | 11 | 10 | 8 |
| `tests/run/fields.bmb` | 5 | 5 | 4 | 4 |
| `tests/run/forest.bmb` | 10 | 10 | 9 | 7 |
| `tests/run/generic_moves.bmb` | 7 | 6 | 5 | 2 |
| `tests/run/generic_stack.bmb` | 13 | 12 | 11 | 16 |
| `tests/run/generics.bmb` | 13 | 10 | 9 | 5 |
| `tests/run/grid.bmb` | 8 | 8 | 7 | 8 |
| `tests/run/int_consts.bmb` | 5 | 5 | 3 | 2 |
| `tests/run/loop_exits.bmb` | 11 | 11 | 10 | 11 |
| `tests/run/mem_moves.bmb` | 6 | 6 | 5 | 4 |
| `tests/run/mut_match.bmb` | 8 | 8 | 7 | 4 |
| `tests/run/nested_vecs.bmb` | 6 | 6 | 5 | 4 |
| `tests/run/numlist.bmb` | 14 | 13 | 12 | 25 |
| `tests/run/owned_arrays.bmb` | 5 | 5 | 4 | 0 |
| `tests/run/owned_fields.bmb` | 7 | 7 | 6 | 1 |
| `tests/run/points.bmb` | 15 | 15 | 14 | 27 |
| `tests/run/primes.bmb` | 11 | 11 | 10 | 14 |
| `tests/run/ranges.bmb` | 14 | 14 | 13 | 12 |
| `tests/run/relaxed.bmb` | 7 | 7 | 6 | 5 |
| `tests/run/returns.bmb` | 14 | 14 | 13 | 8 |
| `tests/run/rose.bmb` | 10 | 10 | 9 | 16 |
| `tests/run/scalar_refs.bmb` | 15 | 11 | 10 | 4 |
| `tests/run/search.bmb` | 23 | 23 | 22 | 27 |
| `tests/run/shapes.bmb` | 16 | 15 | 13 | 20 |
| `tests/run/sort_generic.bmb` | 6 | 6 | 5 | 4 |
| `tests/run/stats.bmb` | 7 | 7 | 6 | 4 |
| `tests/run/swap.bmb` | 4 | 4 | 3 | 1 |
| `tests/run/trait_contracts.bmb` | 10 | 4 | 3 | 4 |
| `tests/run/trait_defaults.bmb` | 18 | 5 | 2 | 7 |
| `tests/run/traits.bmb` | 11 | 2 | 1 | 0 |
| `tests/run/tries.bmb` | 10 | 10 | 9 | 5 |
| `tests/run/values.bmb` | 9 | 9 | 8 | 9 |
| `tests/run/vec_tree.bmb` | 3 | 3 | 2 | 1 |
| `tests/run/vecs.bmb` | 15 | 15 | 14 | 17 |
| `tests/run/vm.bmb` | 5 | 5 | 4 | 5 |
| `tests/run/widths.bmb` | 5 | 5 | 4 | 3 |
