# BVB 출처·라이선스·선정 기록 (백로그 2A-08, ADR-0034 §2)

## 출처와 라이선스

| 원본 | 저장소 | 고정 커밋 | 라이선스 | 쓴 것 |
| --- | --- | --- | --- | --- |
| Verus-Bench (AutoVerus 아티팩트) | https://github.com/microsoft/verus-proof-synthesis `benchmarks/Verus-Bench/` | `cbf9c0c6337b224fd8e5b7cb4e01ae65c0f98bc1` | MIT (Copyright (c) 2024 Microsoft), `licenses/verus-proof-synthesis.LICENSE` | 과제 목록(`tasks.jsonl`), 명세, 검증된 해법(`*/verified/*.rs`) |
| Vericoding (Verus 부분) | https://github.com/Beneficial-AI-Foundation/vericoding-benchmark | `387cd69996792d452ead7b0460f36ee4c5cdd148` | MIT (Copyright (c) 2025 Beneficial AI Foundation), `licenses/vericoding-benchmark.LICENSE` | 명세(`jsonl/verus_tasks.jsonl`), 검증된 해법(`vericoded/*_vericoded.rs`) |
| VeriContest (NeurIPS 2026, arXiv 2605.08553) | 공개 데이터 저장소를 찾지 못함(2026-10-25 검색) | — | — | 쓰지 않음. 문제 지문이 LeetCode·Codeforces 원문이라 공개되더라도 재배포 조건을 따로 확인해야 한다 |

- 두 저장소 모두 MIT라 Verus 원문(명세, 해법)을 과제 파일의 `verus` 키에 그대로 담아 재배포한다. 위 표의 저작권 고지와 `licenses/`의 전문이 MIT 조건(고지 포함)을 충족한다.
- 두 저장소는 다시 다른 공개 집합에서 왔다(Verus-Bench: CloverBench, MBPP-DFY-153, SV-COMP array-fpi(Diffy), Verus 튜토리얼; Vericoding: APPS, DafnyBench, HumanEval, Verina 등, 각 저장소 README). BVB는 그 원본들의 문제 지문을 옮기지 않는다. 과제 설명(`description`)은 명세에서 새로 쓴 것이고, 가져온 것은 위 MIT 저장소가 배포하는 Verus 코드뿐이다.
- 출처 확인: `python3 bench/bvb/sources.py fetch DIR`로 고정 커밋을 받고 `sources.py check DIR`가 과제 파일의 Verus 원문(`reference_origin: upstream`)과 주어진 명세 함수가 원본과 같은지 확인한다. `sources.py select DIR`은 아래 선정을 원본에서 다시 계산해 `tasks/`와 대조한다(네트워크가 필요해 게이트 밖에서 돌린다. 2026-10-25 실행: 둘 다 통과).

## 선정 절차

**Verus-Bench(150개) 전수.** 과제마다 결정 하나: 이식하거나, 아래 사유 하나로 제외한다. 중복(같은 시그니처 꼴·같은 명세)은 `tasks.jsonl`에서 먼저 나온 것을 남긴다.

**Vericoding Verus(2,166개, 품질 문제 목록 `verus_issues.jsonl`의 168개는 원본이 이미 분리) 표본.**
1. 자동 필터(`sources.py`의 `vericoding_filter`): 구현할 함수가 하나이고, 매개변수가 정수·`bool`·그 슬라이스/`Vec`(가변 `Vec` 제외), 반환이 정수·`bool`, 명세에 구조체·문자열·실수·집합 타입과 시퀀스 생성이 없는 것. 386개가 남는다.
2. 표본 순서: `sha256("bvb:" + id)` 오름차순(작성자가 고를 수 없는 고정 순서).
3. 그 순서로 하나씩 보며 이식하거나 사유를 달아 제외하고, 15개를 이식하면 멈춘다. 37개를 봤다.

| | Verus-Bench | Vericoding |
| --- | --- | --- |
| 원본 과제 | 150 | 2,166 (+168 품질 문제) |
| 자동 필터 통과 | — (전수 검토) | 386 |
| 검토 | 150 | 37 (표본 순서 앞쪽) |
| **이식** | **30** | **15** |
| 제외: `Vec` 반환·증가 | 49 | — |
| 제외: 고정 프로그램 불변식 과제(Diffy 38 + `cell_2_sum`) | 39 | — |
| 제외: 튜플·`Option` 반환 | 6 | — |
| 제외: 중첩 `Vec` 입력 | 6 | — |
| 제외: 중복 | 5 | 3 |
| 제외: `i128` 결과 | 3 | — |
| 제외: 증명만 있는 과제 | 3 | — |
| 제외: `Seq::subrange` 존재 명세 | 2 | — |
| 제외: 배열 밖 첨자를 양화하는 명세 | 2 | — |
| 제외: 부호 있는 첨자 반환(−1) | 1 | 2 |
| 제외: `&mut` 스칼라 | 1 | 1 |
| 제외: 문자열, 순열 명세, `assume` | 1 + 1 + 1 | — |
| 제외: 원래 타입으로는 풀 수 없음(`unsat`) | — | 11 |
| 제외: 자명한 명세(`true` 또는 타입상 항상 참) | — | 2 |
| 제외: 입력 없음(상수) | — | 1 |
| 제외: **BMB 기준 해법을 못 씀** | — | **2** |

**선택 편향 공개(ADR-0034 공정성 규칙).**
- 이식률: Verus-Bench 30/150(20%), Vericoding 15/37 검토(41%), 필터까지 포함하면 Vericoding 전체의 0.7%. 제외의 대부분은 M1에 없는 기능(`Vec`·문자열·튜플·구조체)이고 그 과제들은 BVB가 재지 않는다. BVB 결과는 "정수·슬라이스·루프·재귀 과제"에 대한 것이다.
- **BMB에 유리한 제외 2개:** `VH0013`(gcd의 최대성: 모든 공약수에 대한 귀납 보조정리가 필요)과 `VV0128`(완전제곱: 제곱의 단조성을 양화사 아래에서 써야 함)은 M1에 보조정리 함수·`assert forall`이 없어 작성자가 BMB 기준 해법을 쓰지 못했다(`VV0128`은 시도 후 실패). Verus로는 풀 수 있는 과제이므로 이 둘을 뺀 것은 BMB 쪽으로 기운 선택이다. 결과 보고에 함께 적는다.
- **양쪽 모두에서 뺀 것:** `unsat` 11개는 원래 타입으로는 어떤 구현도 명세를 만족할 수 없는 원본 결함이다(아래 표에 반례 입력). BMB로 옮기면(64비트) 풀 수 있게 되므로 비교에서 뺐다.
- 기준 해법·약한 대조군·명세 번역은 모두 작성자 한 명이 썼다(m1b와 같은 편향).

## 결정 표

`python3 bench/bvb/sources.py table`의 출력이다(테스트가 이 표와 `sources.py`가 같은지 확인한다). 사유 코드의 뜻은 `sources.py`의 `REASONS`.

| 원본 | 결정 |
| --- | --- |
| Verus-Bench `cloverbench_all_digits_strong` | 제외: string |
| Verus-Bench `cloverbench_array_append_strong` | 제외: vec-out |
| Verus-Bench `cloverbench_array_concat_strong` | 제외: vec-out |
| Verus-Bench `cloverbench_array_copy_strong` | 제외: vec-out |
| Verus-Bench `cloverbench_array_product_strong` | 제외: vec-out |
| Verus-Bench `cloverbench_array_sum_strong` | 제외: vec-out |
| Verus-Bench `cloverbench_binary_search` | `vb-clover-binary-search` |
| Verus-Bench `cloverbench_cal_div` | 제외: tuple-option |
| Verus-Bench `cloverbench_is_prime` | `vb-clover-is-prime` |
| Verus-Bench `cloverbench_linear_search2` | `vb-clover-linear-search` |
| Verus-Bench `cloverbench_two_sum` | 제외: tuple-option |
| Verus-Bench `diffy_brs1` | 제외: fixed-program |
| Verus-Bench `diffy_brs2` | 제외: fixed-program |
| Verus-Bench `diffy_brs3` | 제외: fixed-program |
| Verus-Bench `diffy_brs4` | 제외: fixed-program |
| Verus-Bench `diffy_brs5` | 제외: fixed-program |
| Verus-Bench `diffy_conda` | 제외: fixed-program |
| Verus-Bench `diffy_condg` | 제외: fixed-program |
| Verus-Bench `diffy_condm` | 제외: fixed-program |
| Verus-Bench `diffy_condn` | 제외: fixed-program |
| Verus-Bench `diffy_ms1` | 제외: fixed-program |
| Verus-Bench `diffy_ms2` | 제외: fixed-program |
| Verus-Bench `diffy_ms3` | 제외: fixed-program |
| Verus-Bench `diffy_ms4` | 제외: fixed-program |
| Verus-Bench `diffy_ms5` | 제외: fixed-program |
| Verus-Bench `diffy_res1` | 제외: fixed-program |
| Verus-Bench `diffy_res1o` | 제외: fixed-program |
| Verus-Bench `diffy_res2` | 제외: fixed-program |
| Verus-Bench `diffy_res2o` | 제외: fixed-program |
| Verus-Bench `diffy_s12if` | 제외: fixed-program |
| Verus-Bench `diffy_s1if` | 제외: fixed-program |
| Verus-Bench `diffy_s1lif` | 제외: fixed-program |
| Verus-Bench `diffy_s22if` | 제외: fixed-program |
| Verus-Bench `diffy_s2if` | 제외: fixed-program |
| Verus-Bench `diffy_s2lif` | 제외: fixed-program |
| Verus-Bench `diffy_s32if` | 제외: fixed-program |
| Verus-Bench `diffy_s3if` | 제외: fixed-program |
| Verus-Bench `diffy_s3lif` | 제외: fixed-program |
| Verus-Bench `diffy_s42if` | 제외: fixed-program |
| Verus-Bench `diffy_s4if` | 제외: fixed-program |
| Verus-Bench `diffy_s4lif` | 제외: fixed-program |
| Verus-Bench `diffy_s52if` | 제외: fixed-program |
| Verus-Bench `diffy_s5if` | 제외: fixed-program |
| Verus-Bench `diffy_s5lif` | 제외: fixed-program |
| Verus-Bench `diffy_sina1` | 제외: fixed-program |
| Verus-Bench `diffy_sina2` | 제외: fixed-program |
| Verus-Bench `diffy_sina3` | 제외: fixed-program |
| Verus-Bench `diffy_sina4` | 제외: fixed-program |
| Verus-Bench `diffy_sina5` | 제외: fixed-program |
| Verus-Bench `mbpp_task_id_105` | `vb-mbpp-105-count-true` |
| Verus-Bench `mbpp_task_id_113` | `vb-mbpp-113-is-integer` |
| Verus-Bench `mbpp_task_id_133` | 제외: i128 |
| Verus-Bench `mbpp_task_id_142` | `vb-mbpp-142-count-identical` |
| Verus-Bench `mbpp_task_id_145` | `vb-mbpp-145-max-difference` |
| Verus-Bench `mbpp_task_id_161` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_170` | 제외: i128 |
| Verus-Bench `mbpp_task_id_18` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_2` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_230` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_240` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_249` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_251` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_261` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_262` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_273` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_282` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_284` | `vb-mbpp-284-all-equal` |
| Verus-Bench `mbpp_task_id_290` | 제외: nested-vec |
| Verus-Bench `mbpp_task_id_3` | `vb-mbpp-3-is-non-prime` |
| Verus-Bench `mbpp_task_id_307` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_399` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_412` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_414` | `vb-mbpp-414-any-value-exists` |
| Verus-Bench `mbpp_task_id_424` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_426` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_431` | 제외: dup (= `mbpp_task_id_414`) |
| Verus-Bench `mbpp_task_id_433` | `vb-mbpp-433-is-greater` |
| Verus-Bench `mbpp_task_id_436` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_445` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_447` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_454` | `vb-mbpp-454-contains-z` |
| Verus-Bench `mbpp_task_id_457` | 제외: nested-vec |
| Verus-Bench `mbpp_task_id_460` | 제외: nested-vec |
| Verus-Bench `mbpp_task_id_461` | `vb-mbpp-461-count-uppercase` |
| Verus-Bench `mbpp_task_id_472` | `vb-mbpp-472-consecutive` |
| Verus-Bench `mbpp_task_id_474` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_476` | `vb-mbpp-476-sum-min-max` |
| Verus-Bench `mbpp_task_id_477` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_554` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_557` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_567` | `vb-mbpp-567-is-sorted` |
| Verus-Bench `mbpp_task_id_572` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_576` | 제외: subrange-exists |
| Verus-Bench `mbpp_task_id_578` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_579` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_586` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_588` | `vb-mbpp-588-max-min-diff` |
| Verus-Bench `mbpp_task_id_602` | 제외: tuple-option |
| Verus-Bench `mbpp_task_id_605` | 제외: dup (= `cloverbench_is_prime`) |
| Verus-Bench `mbpp_task_id_610` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_616` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_618` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_62` | `vb-mbpp-62-smallest` |
| Verus-Bench `mbpp_task_id_624` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_629` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_644` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_69` | 제외: subrange-exists |
| Verus-Bench `mbpp_task_id_70` | 제외: nested-vec |
| Verus-Bench `mbpp_task_id_728` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_732` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_733` | 제외: tuple-option |
| Verus-Bench `mbpp_task_id_741` | `vb-mbpp-741-all-same` |
| Verus-Bench `mbpp_task_id_743` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_755` | 제외: tuple-option |
| Verus-Bench `mbpp_task_id_760` | 제외: dup (= `mbpp_task_id_741`) |
| Verus-Bench `mbpp_task_id_764` | `vb-mbpp-764-count-digits` |
| Verus-Bench `mbpp_task_id_769` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_775` | `vb-mbpp-775-odd-at-odd` |
| Verus-Bench `mbpp_task_id_790` | 제외: dup (= `mbpp_task_id_775`) |
| Verus-Bench `mbpp_task_id_798` | 제외: i128 |
| Verus-Bench `mbpp_task_id_8` | 제외: vec-out |
| Verus-Bench `mbpp_task_id_804` | `vb-mbpp-804-product-even` |
| Verus-Bench `mbpp_task_id_807` | 제외: tuple-option |
| Verus-Bench `mbpp_task_id_808` | `vb-mbpp-808-contains-k` |
| Verus-Bench `mbpp_task_id_809` | `vb-mbpp-809-is-smaller` |
| Verus-Bench `mbpp_task_id_94` | 제외: nested-vec |
| Verus-Bench `mbpp_task_id_95` | 제외: nested-vec |
| Verus-Bench `misc_arg_free` | 제외: no-exec |
| Verus-Bench `misc_basic_nonlinear` | 제외: no-exec |
| Verus-Bench `misc_binary_search` | 제외: dup (= `cloverbench_binary_search`) |
| Verus-Bench `misc_bubble` | 제외: permutation |
| Verus-Bench `misc_cell_2_sum` | 제외: fixed-program |
| Verus-Bench `misc_choose_odd` | `vb-misc-choose-odd` |
| Verus-Bench `misc_conditional_average` | 제외: vec-out |
| Verus-Bench `misc_deduplicate` | 제외: vec-out |
| Verus-Bench `misc_fib` | 제외: vec-out |
| Verus-Bench `misc_filter` | 제외: vec-out |
| Verus-Bench `misc_filter_weak` | 제외: vec-out |
| Verus-Bench `misc_findmax` | `vb-misc-findmax` |
| Verus-Bench `misc_havoc_inline_post` | 제외: escape-hatch |
| Verus-Bench `misc_len_intersect` | 제외: no-exec |
| Verus-Bench `misc_linearsearch` | 제외: signed-index |
| Verus-Bench `misc_map` | `vb-misc-map` |
| Verus-Bench `misc_max_index` | `vb-misc-max-index` |
| Verus-Bench `misc_remove_all_greater` | 제외: vec-out |
| Verus-Bench `misc_reverse` | `vb-misc-reverse` |
| Verus-Bench `misc_simple_nested` | 제외: out-of-range |
| Verus-Bench `misc_sum` | `vb-misc-sum` |
| Verus-Bench `misc_tail_triangle` | 제외: mut-scalar |
| Verus-Bench `misc_trigger` | 제외: out-of-range |
| Vericoding `VJ0020` | 제외: dup (= `misc_findmax`) |
| Vericoding `VH0047` | 제외: unsat (base = height = 255: area 32512 > u8::MAX) |
| Vericoding `VD0791` | `vc-vd0791-month-30-days` |
| Vericoding `VA0523` | `vc-va0523-min-abs-diff` |
| Vericoding `VD0722` | `vc-vd0722-last-digit` |
| Vericoding `VH0013` | 제외: no-bmb-ref |
| Vericoding `VA0590` | `vc-va0590-boring-apartment` |
| Vericoding `VA0676` | 제외: unsat (r = 0, g = 127: 2g - r = 254 > i8::MAX) |
| Vericoding `VV0104` | 제외: unsat (a = [i32::MAX - 1, i32::MIN + 1]: even - odd overflows i32) |
| Vericoding `VA0205` | 제외: unsat (n = 127, m = 2: 253 days > i8::MAX) |
| Vericoding `VD0356` | 제외: unsat (n = 2^63: f(n) = 3 * 2^63 - 1 > u64::MAX) |
| Vericoding `VT0094` | 제외: no-input |
| Vericoding `VJ0055` | 제외: mut-scalar |
| Vericoding `VD0114` | 제외: dup (= `misc_max_index`) |
| Vericoding `VV0128` | 제외: no-bmb-ref |
| Vericoding `VV0033` | 제외: unsat (len >= 2^31: `nums.len() as i32` is negative, so result >= 0 && result <= it fails) |
| Vericoding `VA0532` | `vc-va0532-crackers` |
| Vericoding `VA0093` | `vc-va0093-max-balance` |
| Vericoding `VA0053` | `vc-va0053-aspect-pairs` |
| Vericoding `VD0727` | `vc-vd0727-calculate-loss` |
| Vericoding `VV0039` | `vc-vv0039-max-of-list` |
| Vericoding `VA0399` | `vc-va0399-late-brother` |
| Vericoding `VD0131` | 제외: unsat (v = [i32::MAX, i32::MAX]: both are peeks, sum 2 * i32::MAX > i32::MAX) |
| Vericoding `VA0219` | `vc-va0219-pyramid-height` |
| Vericoding `VH0043` | 제외: unsat (n = 127: 127 * 127 > i8::MAX) |
| Vericoding `VH0085` | 제외: unsat (n = 3: 100 + 90 - 10 = 180 > i8::MAX) |
| Vericoding `VD0126` | `vc-vd0126-first-zero` |
| Vericoding `VV0100` | 제외: trivial |
| Vericoding `VD0465` | `vc-vd0465-increasing-run` |
| Vericoding `VD0670` | 제외: unsat (a = [i32::MIN, -1]: sum < i32::MIN) |
| Vericoding `VS0021` | 제외: unsat (a = -128, b = 0: gcd 128 > i8::MAX) |
| Vericoding `VV0135` | `vc-vv0135-compare` |
| Vericoding `VD0199` | 제외: signed-index |
| Vericoding `VV0073` | 제외: trivial |
| Vericoding `VV0149` | 제외: dup (= `cloverbench_linear_search2`) |
| Vericoding `VD0239` | 제외: signed-index |
| Vericoding `VA0644` | `vc-va0644-takahashi-dates` |

## 사유 코드

| 코드 | 뜻 |
| --- | --- |
| `vec-out` | returns or grows a Vec (M1 has no heap or Vec) |
| `nested-vec` | takes nested Vecs (M1 slices hold only integers or bool) |
| `string` | takes a String or char (no strings in M1) |
| `tuple-option` | returns a tuple or an Option (no tuples, structs or enums in M1) |
| `i128` | result needs i128 (largest M1 integer is 64-bit) |
| `subrange-exists` | spec is an existential over Seq::subrange equality; in BMB it needs a named window predicate the agent cannot know (trigger matching), so it is not portable faithfully |
| `signed-index` | returns an index as a signed integer with a -1 sentinel; M1 has no integer conversion, so a spec cannot index with the result |
| `mut-scalar` | &mut scalar parameter (M1 references are slices only) |
| `permutation` | spec is a permutation of a Seq (needs sequence values) |
| `out-of-range` | spec quantifies over indices outside the array (Seq semantics) |
| `fixed-program` | invariant-synthesis task on a fixed program whose spec is a weak property of its output; as spec-to-code it is solved by a constant |
| `no-exec` | proof-only task or no specification to implement |
| `escape-hatch` | the original uses assume |
| `dup` | same signature shape and spec as an earlier upstream task (kept the first) |
| `unsat` | unsolvable as stated: the spec forces a result outside the original return type for some valid input |
| `trivial` | postcondition holds for every value of the result type |
| `no-input` | no input (a constant) |
| `no-bmb-ref` | the author found no BMB reference proof: M1 has no lemma functions or quantifier introduction (bias toward BMB, disclosed) |
| `upstream-issue` | listed in vericoding verus_issues.jsonl (upstream quality issue) |
