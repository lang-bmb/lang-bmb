# 게이트 검출력 (1B-38)

> 백로그 1B-38, HEALTH.md §3(신뢰 기반 "퍼저의 검출력 미측정"). 증명기(`verify.rs`)와 코드 생성기(`codegen.rs`)에 그럴듯한 결함을 하나씩 넣고, 게이트 전체(`scripts/check.sh`)가 잡는지 잰다.

## 방법

- `scripts/gate_power.py`: 변이체 31개(건전성을 깨는 것 23, 완전성을 깨는 것 1, 오컴파일 7). 각각 HEAD의 분리된 작업 트리에 하나만 적용하고 `cargo fmt` 뒤 게이트를 끝까지 돌린다. 게이트가 0이 아닌 값으로 끝나면 잡은 것이고, `detected_by`에 실패한 작업 이름을 남긴다.
- 서식(`cargo fmt --check`)이나 컴파일 경고(clippy `-D warnings`)로만 실패하는 것은 시험이 잡은 것이 아니다. 첫 실행에서 이렇게 "잡힌" 변이체 3개(`v-mut-param-havoc-skipped`, `c-overflow-check-removed`, `c-bounds-check-removed`: 쓰이지 않는 변수)는 경고 없이 컴파일되게 고쳐 다시 돌렸다.
- 원자료: `results.jsonl`(변이체마다 마지막 실행, `commit`이 그 실행의 HEAD). 한 번에 약 5분이라 전체가 여러 커밋(5a50b57 → 10a8b4f)에 걸쳤다. 다시 돌린 행은 시험을 더한 커밋이다.

## 결과

첫 실행: 시험이 잡은 것 26/31. 놓친 것 5개마다 시험을 더했다.

| 변이체 | 결함 | 더한 시험 |
| --- | --- | --- |
| `v-element-range-dropped` | 루프 뒤 배열 원소를 읽어도 타입 범위를 모른다(완전성) | `tests/verify/pass/loops.bmb`의 `element_after_loop` |
| `c-bounds-check-ule` | 증명되지 않은 인덱스의 런타임 검사가 `a[len]`을 통과시킨다 | `tests/codegen.rs`의 `an_unproved_index_traps_exactly_from_the_length` |
| `v-mut-param-havoc-skipped` | 루프 안에서 `&mut` 매개변수를 넘겨도 루프 머리에서 havoc하지 않는다(건전성) | `tests/verify/fail/loops.bmb`의 `passed_on_in_loop` |
| `c-overflow-check-removed` | 증명 없이 nsw/nuw를 붙인다 | (기존 `test codegen`·`test lint`가 잡음, 경고 없는 변이체로 확인) |
| `c-bounds-check-removed` | 증명 없이 경계 검사를 뺀다 | (기존 `test codegen`이 잡음, 같은 확인) |

시험을 더한 뒤 31/31.

| 변이체 | 종류 | 결함 | 게이트 |
| --- | --- | --- | --- |
| `v-read-index-le` | unsound | reading a[len] is allowed | 잡음: bench bcb, test verify |
| `v-store-index-le` | unsound | writing a[len] is allowed | 잡음: bench bcb, test soundness_loops, test verify |
| `v-div-zero-dropped` | unsound | division by zero is never an obligation | 잡음: bench bcb, health, test verify, test codegen |
| `v-min-rem-dropped` | unsound | i64::MIN % -1 is never an obligation | 잡음: test verify |
| `v-neg-overflow-dropped` | unsound | -i64::MIN is never an obligation | 잡음: test verify |
| `v-sub-swapped` | unsound | a - b is encoded as b - a | 잡음: bench bcb, test soundness, bench brb weak, bench brb oracle, test incremental, bench m1b oracle, health, bench bvb weak, bench m1 harness tests, test lint, test stability, bench bvb oracle, test impact, test soundness_rec, test verify, bench m1 oracle, test codegen, bench m3 generator, bench bvb tests, test gate, test map |
| `v-mul-as-add` | unsound | a * b is encoded as a + b | 잡음: test soundness_unsigned, bench bvb oracle, bench bvb weak, bench bvb tests, test soundness, test lint, test verify |
| `v-spec-lt-as-le` | unsound | in contracts, a < b means a <= b | 잡음: bench bcb, test soundness_loops, bench brb oracle, health, bench m1b oracle, bench bvb weak, bench m1 harness tests, bench bvb oracle, test stability, test lint, test impact, test verify, bench bvb tests, test soundness_specs, bench m1 oracle, test codegen, bench m3 generator, test soundness, test gate |
| `v-lt-as-le` | unsound | a < b is encoded as a <= b | 잡음: bench m1b oracle, test incremental, bench m1 harness tests, bench m1 oracle, bench bvb weak, bench bvb oracle, test soundness, bench brb weak, bench brb oracle, health, test soundness_loops, test lint, test soundness_rec, test verify, test codegen, bench bvb tests, bench m3 generator, test impact, bench bcb, test stability, test gate, test map |
| `v-ge-as-gt` | unsound | a >= b is encoded as a > b | 잡음: bench m1b oracle, bench brb oracle, bench bvb weak, bench bvb oracle, bench m1 harness tests, bench brb weak, health, bench m1 oracle, test lint, bench bvb tests |
| `v-or-guard` | unsound | the right operand of || is assumed to run when the left is true | 잡음: bench brb oracle, health, test stability, test verify |
| `v-element-range-dropped` | incomplete | array elements read lose their type range | 잡음: health, test stability, test verify |
| `v-i64-range-plus-one` | unsound | i64 overflow by one is allowed | 잡음: test lint, test verify |
| `v-u64-range-minus-one` | unsound | unsigned underflow to -1 is allowed | 잡음: bench bcb, bench brb weak, bench brb oracle, health, bench m1b oracle, bench bvb weak, bench m1 harness tests, test lint, test stability, test impact, test soundness_rec, bench bvb oracle, test verify, bench m1 oracle, test codegen, bench m3 generator, bench bvb tests, test gate, test map |
| `v-invariant-entry-dropped` | unsound | invariants are assumed without holding on entry | 잡음: test verify, bench bcb |
| `v-invariant-preserved-dropped` | unsound | invariants are assumed without being preserved | 잡음: bench bcb, test soundness_loops, test incremental, test verify |
| `v-decreases-unbounded` | unsound | a loop measure may go below zero | 잡음: test verify |
| `v-havoc-skipped` | unsound | variables a loop assigns keep their entry values | 잡음: test verify |
| `v-mut-param-havoc-skipped` | unsound | a loop that writes through a &mut parameter keeps its entry contents | 잡음: test verify |
| `v-call-requires-dropped` | unsound | callee preconditions are never obligations | 잡음: test lint, test cache, test verify, test soundness_calls, test map |
| `v-ensures-dropped` | unsound | postconditions are never obligations | 잡음: bench bcb, bench m1b weak, test lint, test soundness, test uses_fill, test soundness_unsigned, bench bvb weak, bench m1 harness tests, test incremental, test soundness_rec, test verify, test soundness_specs, test codegen, bench m1 weak, test soundness_calls |
| `v-assert-dropped` | unsound | assertions are never obligations | 잡음: bench m1b weak, bench m1 harness tests, bench bvb oracle, test verify |
| `v-recursion-measure-dropped` | unsound | recursive calls need not decrease their measure | 잡음: test verify |
| `v-signed-div-rem-swapped` | unsound | signed / and % are swapped in the encoding | 잡음: bench bcb, test soundness, bench bvb weak, bench brb weak, bench brb oracle, bench m1b oracle, bench bvb oracle, test verify, bench bvb tests |
| `c-add-as-sub` | miscompile | + is compiled as - | 잡음: test soundness_loops, bench m1 harness tests, test soundness_unsigned, bench brb oracle, bench bcb, test soundness_specs, test soundness_rec, test soundness_calls, test codegen, test soundness |
| `c-slt-as-sle` | miscompile | signed < is compiled as <= | 잡음: bench brb weak, bench brb oracle, test soundness, bench m1 harness tests, test codegen, test soundness_rec |
| `c-ult-as-ule` | miscompile | unsigned < is compiled as <= | 잡음: bench brb oracle, test soundness_loops, test soundness_specs, test codegen, test soundness_rec |
| `c-sdiv-as-udiv` | miscompile | signed / is compiled as unsigned | 잡음: bench brb oracle, bench bcb, test soundness, bench m1 harness tests, test soundness_calls, test codegen, test soundness_rec |
| `c-bounds-check-ule` | miscompile | the runtime bounds check (unproved indexing) lets a[len] through | 잡음: test codegen |
| `c-overflow-check-removed` | miscompile | overflow checks are dropped without a proof (nsw/nuw on unproved arithmetic) | 잡음: test lint, test codegen |
| `c-bounds-check-removed` | miscompile | bounds checks are dropped without a proof | 잡음: test codegen |

검출: 31/31
- unsound: 23/23
- incomplete: 1/1
- miscompile: 7/7

## 관찰

- **한 작업만 잡는 결함 8개.** `v-min-rem-dropped`, `v-neg-overflow-dropped`, `v-decreases-unbounded`, `v-havoc-skipped`, `v-mut-param-havoc-skipped`, `v-recursion-measure-dropped`는 `test verify`(손으로 쓴 코퍼스)만, `c-bounds-check-*`는 `test codegen`만 잡는다. 건전성 퍼저(`soundness*`)는 이 여섯을 못 잡는다: 퍼저가 `%`의 `i64::MIN % -1`, 부정 오버플로, 루프 측도의 하한, 루프 havoc을 만들지 않는다는 뜻이다. 코퍼스 한 줄이 지워지면 결함이 들어와도 게이트가 통과한다.
- **변이체는 손으로 쓴 31개다.** 실제 결함 분포를 대표한다는 근거는 없다. 2B(데이터 타입) 인코딩이 들어오면 같은 방식으로 변이체를 더한다.
- 비용: 31개 한 바퀴 약 2.9시간(게이트 한 번 약 5분). CI에는 넣지 않는다.
