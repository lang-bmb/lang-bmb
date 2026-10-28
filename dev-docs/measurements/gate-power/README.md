# 게이트 검출력 (1B-38)

> 백로그 1B-38, HEALTH.md §3(신뢰 기반 "퍼저의 검출력 미측정"). 증명기(`verify.rs`)와 코드 생성기(`codegen.rs`)에 그럴듯한 결함을 하나씩 넣고, 게이트 전체(`scripts/check.sh`)가 잡는지 잰다.

## 방법

- `scripts/gate_power.py`: 변이체 53개(건전성을 깨는 것 37, 완전성을 깨는 것 2, 오컴파일 14). 31개는 1B-38, 4개(trait)는 2T-05, 4개(소유 타입)는 2O-05, 2개(구조체만으로 된 재귀)는 2O-03d, 12개(제어 흐름)는 2K. 각각 HEAD의 분리된 작업 트리에 하나만 적용하고 `cargo fmt` 뒤 게이트를 끝까지 돌린다. 게이트가 0이 아닌 값으로 끝나면 잡은 것이고, `detected_by`에 실패한 작업 이름을 남긴다.
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

**2T-05 (2026-10-27, 커밋 39b91295·61105a21).** 변이체 여섯(`v-mul-as-add`, `v-spec-lt-as-le`, `v-lt-as-le`, `v-ge-as-gt`, `c-bounds-check-ule`, `c-bounds-check-removed`)이 2T-02 뒤 소스에 더는 적용되지 않았다(비교가 `ord_term` 하나로 모였고, 스칼라·데이터 경계 검사가 두 벌이 됨). 아무도 몰랐으므로 게이트에 `power` 단계(`gate_power.py check`)를 더하고 여섯을 현재 코드로 다시 겨눴다. `v-spec-lt-as-le`는 명세와 코드가 같은 인코딩을 쓰게 되어 타입 매개변수 순서(`Ord` 경계)의 `<`를 겨눈다. trait 변이체 넷(`v-refine-post-dropped`, `v-refine-pre-dropped`, `v-default-spec-leaks`, `c-trait-call-undispatched`)을 더했다. 첫 실행에서 `v-default-spec-leaks`를 놓쳤는데 동치였다: 바꾼 `has_param` 검사 뒤의 `dispatch` 검사가 같은 경우를 막았다. 그 검사는 오히려 지나치게 보수적이어서(`Self = Wrap<T>`의 기본 본문도 막음) 빼고(`tests/run/trait_defaults`의 `wrapped`), 변이체는 `dispatch` 조건을 겨누게 고쳐 다시 돌렸다. 열 개 모두 잡음. 새 trait 퍼저(`test soundness_traits`)가 `v-refine-post-dropped`·`c-trait-call-undispatched`를 잡는다.

**2O-05·2O-06 (2026-10-27, 커밋 0b6cd9ad).** 소유 타입 변이체 넷을 더했다: `c-box-free-dropped`(해제되는 `Box`를 풀지 않음: 누수), `v-replace-returns-new`(`mem::replace`가 새 값을 돌려준다고 봄), `v-decrease-any-term`(구조적 `decreases`가 호출자 값의 부분이 아닌 인자에도 성립), `c-mut-match-write-back-dropped`(`&mut` `match` 바인딩으로 쓴 스칼라를 페이로드에 되쓰지 않음). 넷 모두 잡음. 소유 퍼저(`test soundness_owned`)가 `c-box-free-dropped`(valgrind)와 `v-replace-returns-new`(반례 재현)를 잡는다. `v-decrease-any-term`은 `test verify`만, `c-mut-match-write-back-dropped`는 `test codegen`만 잡는다(한 작업만 잡는 결함 10개).

**2O-03d (2026-10-27, 커밋 e40ca564).** 변이체 둘: `v-struct-field-write-dropped`(재귀 구조체의 필드 쓰기가 값을 바꾸지 않음), `v-struct-facts-from-definitions`(필드 사실을 값 정의까지 훑음: 소유 퍼저가 구현 중 찾은 결함의 되돌림). 둘 다 잡음. 첫 실행은 `docs/diagnostics.md`가 낡아 게이트가 시험 전 `docs` 단계에서 멈췄으므로(잡은 작업 없음, 12~36초) 버리고 고친 뒤 다시 돌렸다. `v-struct-facts-from-definitions`는 `test verify`(회귀 `built`)만 잡는다: 게이트의 소유 퍼저 3배치는 그 결함을 내는 프로그램을 만들지 않는다(찾은 것은 80배치 중 씨앗 138).

**2K (2026-10-28, 커밋 a843816a·6f607ada).** 제어 흐름 변이체 열둘: 반환(`v-return-post-dropped`, `v-return-rest-wrong-side`, `c-return-exit-drops-skipped`), 루프 havoc(`v-havoc-match-arms-skipped`: 2K-01a에서 고친 건전성 결함의 되돌림), 루프 탈출(`v-break-state-dropped`, `v-continue-head-skipped`, `c-jump-exit-drops-skipped`), 범위 `for`(`v-for-continue-no-step`, `v-for-var-not-havocked`, `c-for-range-ends-swapped`), `?`(`v-try-return-post-skipped`, `c-try-exit-drops-skipped`). `v-ensures-dropped`는 사후조건 검사가 `post_check` 하나로 모여 다시 겨눴다. 열셋 모두 잡음. 첫 실행(a843816a)에서 루프 퍼저는 `c-for-range-ends-swapped`만 잡았다: 탈출 갈래가 제 상태를 바꾸지 않아 이어지는 갈래와 상태가 같았으므로 건너뛴 의무가 드러나지 않았다. 생성기를 고치고(2K-05 후속, 6f607ada: 탈출 갈래의 쓰기, 배치마다 탈출 종류를 씨앗으로 돌리는 아슬아슬한 합) 여섯을 다시 돌려, 루프 퍼저가 `v-return-post-dropped`·`v-break-state-dropped`·`v-continue-head-skipped`·`v-try-return-post-skipped`를 잡는다. `v-for-continue-no-step`은 증가가 없으면 측정이 줄지 않아 그런 루프가 하나도 증명되지 않으므로 완전성 결함으로 고쳐 분류했다. `v-return-rest-wrong-side` 실행에서는 `health` 작업의 z3 질의 하나가 끝나지 않아(16분) 그 z3를 손으로 끝냈다. 다른 작업(`test verify`·`test codegen`·`test stability`)이 따로 잡았다. 한 작업만 잡는 것: `v-havoc-match-arms-skipped`·`v-for-var-not-havocked`(`test verify`), `c-return-exit-drops-skipped`·`c-jump-exit-drops-skipped`·`c-try-exit-drops-skipped`(`test codegen`의 valgrind).

| 변이체 | 종류 | 결함 | 게이트 |
| --- | --- | --- | --- |
| `v-read-index-le` | unsound | reading a[len] is allowed | 잡음: bench bcb, test verify |
| `v-store-index-le` | unsound | writing a[len] is allowed | 잡음: bench bcb, test soundness_loops, test verify |
| `v-div-zero-dropped` | unsound | division by zero is never an obligation | 잡음: bench bcb, health, test verify, test codegen |
| `v-min-rem-dropped` | unsound | i64::MIN % -1 is never an obligation | 잡음: test verify |
| `v-neg-overflow-dropped` | unsound | -i64::MIN is never an obligation | 잡음: test verify |
| `v-sub-swapped` | unsound | a - b is encoded as b - a | 잡음: bench bcb, test soundness, bench brb weak, bench brb oracle, test incremental, bench m1b oracle, health, bench bvb weak, bench m1 harness tests, test lint, test stability, bench bvb oracle, test impact, test soundness_rec, test verify, bench m1 oracle, test codegen, bench m3 generator, bench bvb tests, test gate, test map |
| `v-mul-as-add` | unsound | a * b is encoded as a + b | 잡음: test verify, test lint, test stability, test soundness_data, test soundness_traits, test soundness, bench bvb oracle, bench bvb weak, bench bvb tests, health |
| `v-spec-lt-as-le` | unsound | for a type parameter bounded by Ord, a < b means a <= b (contracts and code share `ord_term` since 2T-02) | 잡음: test verify |
| `v-lt-as-le` | unsound | a < b is encoded as a <= b | 잡음: bench bcb, test incremental, health, test stability, test verify, test soundness_loops, test lint, bench brb weak, bench m1b oracle, bench brb oracle, bench bvb weak, test soundness, bench m1 harness tests, test impact, test soundness_data, test codegen, bench bvb oracle, test soundness_rec, test soundness_specs, bench m1 oracle, bench m3 generator, bench bvb tests, test gate, test map, test cli |
| `v-ge-as-gt` | unsound | a >= b is encoded as a > b | 잡음: test incremental, bench m1b oracle, bench m1 harness tests, bench brb oracle, bench bvb weak, bench m1 oracle, bench bvb oracle, health, test verify, bench brb weak, test lint, test stability, test codegen, test soundness_unsigned, test impact, bench bcb, bench m3 generator, bench bvb tests, test soundness_specs |
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
| `v-ensures-dropped` | unsound | postconditions are never obligations | 잡음: test incremental, bench bcb, test verify, bench m1b weak, test soundness_loops, test lint, test codegen, test soundness, bench m1 harness tests, bench bvb weak, test uses_fill, test soundness_traits, test soundness_data, test soundness_unsigned, test soundness_rec, test soundness_specs, test soundness_owned, test soundness_calls, test soundness_vec, bench m1 weak |
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
| `v-refine-post-dropped` | unsound | a trait impl's postconditions need not imply the trait's (E0817) | 잡음: test verify, test soundness_traits |
| `v-refine-pre-dropped` | unsound | a trait impl may require more than the trait (E0817) | 잡음: test verify |
| `v-default-spec-leaks` | unsound | a default spec body is revealed for a bounded `Self`, whose impl may override it | 잡음: test verify |
| `c-trait-call-undispatched` | miscompile | a trait call with a known `Self` is not lowered to its impl's function | 잡음: test incremental, test soundness_traits, test codegen |
| `c-box-free-dropped` | miscompile | a dropped `Box` is never freed (a leak at every drop) | 잡음: test codegen, test soundness_owned |
| `v-replace-returns-new` | unsound | `std::mem::replace` is taken to return the new value | 잡음: bench bcb, test verify, health, test stability, test soundness_owned |
| `v-decrease-any-term` | unsound | a structural `decreases` holds for any argument, not only a part of the caller's value | 잡음: test verify |
| `c-mut-match-write-back-dropped` | miscompile | a scalar written through a `&mut` match binding is not stored back to the payload | 잡음: test codegen |
| `v-struct-field-write-dropped` | unsound | a field write in a recursive struct leaves the value unchanged | 잡음: health, test verify, test stability, test codegen |
| `v-struct-facts-from-definitions` | unsound | the field facts of recursive struct values also cover values made later or on another path | 잡음: test verify |
| `v-return-post-dropped` | unsound | a `return` does not check the postconditions | 잡음: test verify, test soundness_loops |
| `v-return-rest-wrong-side` | unsound | after a branch that returned, the rest runs under that branch's condition | 잡음: test stability, health, test verify, test codegen |
| `v-havoc-match-arms-skipped` | unsound | a loop keeps the entry value of a local assigned in a `match` arm of its body | 잡음: test verify |
| `c-return-exit-drops-skipped` | miscompile | the owned locals still held at a `return` are not freed (a leak) | 잡음: test codegen |
| `v-break-state-dropped` | unsound | a `while` is left only with its condition false: the `break` states are not among the exits | 잡음: test verify, test soundness_loops |
| `v-continue-head-skipped` | unsound | a `continue` goes back to the head without its invariant or measure obligations | 잡음: test verify, test soundness_loops |
| `v-for-continue-no-step` | incomplete | a `continue` in a `for` goes back to the head without stepping the variable (its measure does not decrease, so no such loop proves) | 잡음: health, test stability, test verify, test codegen |
| `v-for-var-not-havocked` | unsound | a `for` variable keeps its entry value in every iteration | 잡음: test verify |
| `c-jump-exit-drops-skipped` | miscompile | the owned locals of the loop are not freed at a `break` or `continue` (a leak) | 잡음: test codegen |
| `c-for-range-ends-swapped` | miscompile | `lo..hi` runs its end too and `lo..=hi` skips it | 잡음: test soundness_loops, test codegen |
| `v-try-return-post-skipped` | unsound | the `None`/`Err` a `?` returns is not checked against the postconditions | 잡음: test verify, test soundness_loops |
| `c-try-exit-drops-skipped` | miscompile | the owned locals alive at a `?` are not freed when it returns (a leak) | 잡음: test codegen |

검출: 53/53
- unsound: 37/37
- incomplete: 2/2
- miscompile: 14/14

## 관찰

- **한 작업만 잡는 결함 8개(2O 뒤 11개, 2K 뒤 16개).** `v-min-rem-dropped`, `v-neg-overflow-dropped`, `v-decreases-unbounded`, `v-havoc-skipped`, `v-mut-param-havoc-skipped`, `v-recursion-measure-dropped`는 `test verify`(손으로 쓴 코퍼스)만, `c-bounds-check-*`는 `test codegen`만 잡는다. 건전성 퍼저(`soundness*`)는 이 여섯을 못 잡는다: 퍼저가 `%`의 `i64::MIN % -1`, 부정 오버플로, 루프 측도의 하한, 루프 havoc을 만들지 않는다는 뜻이다. 코퍼스 한 줄이 지워지면 결함이 들어와도 게이트가 통과한다.
- **변이체는 손으로 쓴 31개다.** 실제 결함 분포를 대표한다는 근거는 없다. 2B(데이터 타입) 인코딩이 들어오면 같은 방식으로 변이체를 더한다.
- 비용: 31개 한 바퀴 약 2.9시간(게이트 한 번 약 5분). CI에는 넣지 않는다.
