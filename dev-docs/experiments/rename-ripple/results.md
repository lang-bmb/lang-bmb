# 이름 바꾸기 파급 (H-ID)

생성: `python3 dev-docs/experiments/rename-ripple/run.py --stats`. 정의 하나의 이름만 바꿨을 때 정체성·해시가 바뀌는 *다른* 정의(그 정의를 부르는 L2 참조자) 수.
`bmb hash`의 `id`·`body`·`iface`(ADR-0059 §1·§4).

| 항목 | 값 |
| --- | ---: |
| 파일 | 48 |
| 정의 | 464 |
| 이름을 바꾼 정의(검사 통과) | 424 |
| 건너뜀(이름 중복·검사 실패) | 40 |
| 다른 정의를 바꾼 이름 바꾸기 | 0 (0%) |
| 본문 해시가 바뀐 참조자(합) | 0 |
| 계약 해시가 바뀐 참조자(합) | 0 |
| `id`가 바뀐 참조자(합) | 0 |
| 참조자가 아닌데 바뀐 정의(같은 낱말, 집계에서 뺌) | 0 |
| 증분 빌드가 다시 인코딩한 정의(합, 자기 포함) | 847 |
| 다시 인코딩이 있었던 이름 바꾸기 | 423 |
| 증명 캐시 놓침(합) | 3560 |
| 증명 캐시 놓침이 있었던 이름 바꾸기 | 287 |

## 전후 비교 (2026-10-28, 백로그 T1-10)

같은 파일 집합(커밋 `0667a81e`의 코퍼스, `tests/run/bits.bmb`가 더해져 처음 측정의 412건보다 많다)을 ADR-0059 이전 컴파일러(`0667a81e`)와 이후 컴파일러로 쟀다. 이전 컴파일러에는 `id`가 없다.

| 항목 | 이전(`0667a81e`) | 이후(T1-10) |
| --- | ---: | ---: |
| 이름을 바꾼 정의 | 424 | 424 |
| 다른 정의를 바꾼 이름 바꾸기 | 367 (87%) | 0 (0%) |
| 본문 해시가 바뀐 참조자 | 626 | 0 |
| 계약 해시가 바뀐 참조자 | 384 | 0 |
| 참조자가 아닌데 바뀐 정의 | 16 | 0 |
| 다시 인코딩한 정의(자기 포함) | 848 | 847 |
| 증명 캐시 놓침 | 3,560 | 3,560 |

- 정체성·해시 쪽(ADR-0059 H-ID "비용 쪽"의 참조자 수)은 0이 되었다. 첫 측정(412건 중 356건·86%, 본문 611·계약 377)도 같은 경향이다.
- 증분 빌드와 증명 캐시 쪽은 아직 0이 아니다. 질의 텍스트가 이름을 담고(정의 기호, 지역 상수, 필드 선택자, 이름순 병합), 그래서 증명 키가 그 이름을 범례로 쥔다(`compiler/src/incremental.rs` 모듈 설명). 이름 바꾸기 하나당 평균 2.0개 정의를 다시 인코딩하고(자기와 그 이름을 쓰는 정의), 의무 8.4개가 캐시를 놓친다. 0이 되는 것은 질의 기호를 이름 없이 짓는 백로그 T1-10e 뒤다.
- 가독성: 코퍼스 85개 파일의 `map all`(4,783행)이 `id`를 빼고 해시 값을 가리면 이전 컴파일러의 출력과 바이트 단위로 같다(`python3 dev-docs/experiments/rename-ripple/readability.py OLD NEW`).

| 파일 | 정의 | 바꾼 이름 | 파급 | 계약 해시 파급 | 재인코딩 | 캐시 놓침 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `lib/bytes.bmb` | 25 | 25 | 0 | 0 | 34 | 184 |
| `tests/run/arith_methods.bmb` | 3 | 3 | 0 | 0 | 5 | 0 |
| `tests/run/array_if.bmb` | 4 | 4 | 0 | 0 | 7 | 0 |
| `tests/run/arrays.bmb` | 9 | 9 | 0 | 0 | 19 | 90 |
| `tests/run/basics.bmb` | 9 | 9 | 0 | 0 | 17 | 99 |
| `tests/run/bits.bmb` | 12 | 12 | 0 | 0 | 24 | 61 |
| `tests/run/bools.bmb` | 4 | 4 | 0 | 0 | 6 | 57 |
| `tests/run/box_places.bmb` | 10 | 10 | 0 | 0 | 22 | 56 |
| `tests/run/boxes.bmb` | 13 | 13 | 0 | 0 | 33 | 36 |
| `tests/run/bytes.bmb` | 6 | 6 | 0 | 0 | 11 | 30 |
| `tests/run/calc.bmb` | 8 | 7 | 0 | 0 | 18 | 49 |
| `tests/run/conversions.bmb` | 4 | 4 | 0 | 0 | 7 | 6 |
| `tests/run/expr_tree.bmb` | 11 | 11 | 0 | 0 | 30 | 99 |
| `tests/run/fields.bmb` | 5 | 5 | 0 | 0 | 8 | 7 |
| `tests/run/forest.bmb` | 10 | 10 | 0 | 0 | 29 | 13 |
| `tests/run/generic_moves.bmb` | 7 | 6 | 0 | 0 | 12 | 4 |
| `tests/run/generic_stack.bmb` | 13 | 12 | 0 | 0 | 29 | 139 |
| `tests/run/generics.bmb` | 13 | 10 | 0 | 0 | 20 | 16 |
| `tests/run/grid.bmb` | 8 | 8 | 0 | 0 | 14 | 188 |
| `tests/run/int_consts.bmb` | 5 | 5 | 0 | 0 | 7 | 6 |
| `tests/run/loop_exits.bmb` | 11 | 11 | 0 | 0 | 22 | 80 |
| `tests/run/mem_moves.bmb` | 6 | 6 | 0 | 0 | 12 | 10 |
| `tests/run/mut_match.bmb` | 8 | 8 | 0 | 0 | 16 | 9 |
| `tests/run/nested_vecs.bmb` | 6 | 6 | 0 | 0 | 10 | 82 |
| `tests/run/numlist.bmb` | 14 | 13 | 0 | 0 | 36 | 180 |
| `tests/run/owned_arrays.bmb` | 5 | 5 | 0 | 0 | 8 | 15 |
| `tests/run/owned_fields.bmb` | 7 | 7 | 0 | 0 | 12 | 28 |
| `tests/run/points.bmb` | 15 | 15 | 0 | 0 | 31 | 321 |
| `tests/run/primes.bmb` | 11 | 11 | 0 | 0 | 21 | 242 |
| `tests/run/ranges.bmb` | 14 | 14 | 0 | 0 | 29 | 363 |
| `tests/run/relaxed.bmb` | 7 | 7 | 0 | 0 | 13 | 7 |
| `tests/run/returns.bmb` | 14 | 14 | 0 | 0 | 27 | 100 |
| `tests/run/rose.bmb` | 10 | 10 | 0 | 0 | 22 | 58 |
| `tests/run/scalar_refs.bmb` | 15 | 11 | 0 | 0 | 21 | 26 |
| `tests/run/search.bmb` | 23 | 23 | 0 | 0 | 41 | 287 |
| `tests/run/shapes.bmb` | 16 | 15 | 0 | 0 | 32 | 146 |
| `tests/run/sort_generic.bmb` | 6 | 6 | 0 | 0 | 8 | 33 |
| `tests/run/stats.bmb` | 7 | 7 | 0 | 0 | 14 | 50 |
| `tests/run/swap.bmb` | 4 | 4 | 0 | 0 | 7 | 1 |
| `tests/run/trait_contracts.bmb` | 10 | 4 | 0 | 0 | 8 | 18 |
| `tests/run/trait_defaults.bmb` | 18 | 5 | 0 | 0 | 10 | 11 |
| `tests/run/traits.bmb` | 11 | 2 | 0 | 0 | 3 | 0 |
| `tests/run/tries.bmb` | 10 | 10 | 0 | 0 | 20 | 33 |
| `tests/run/values.bmb` | 9 | 9 | 0 | 0 | 18 | 29 |
| `tests/run/vec_tree.bmb` | 3 | 3 | 0 | 0 | 5 | 20 |
| `tests/run/vecs.bmb` | 15 | 15 | 0 | 0 | 29 | 195 |
| `tests/run/vm.bmb` | 5 | 5 | 0 | 0 | 11 | 28 |
| `tests/run/widths.bmb` | 5 | 5 | 0 | 0 | 9 | 48 |
