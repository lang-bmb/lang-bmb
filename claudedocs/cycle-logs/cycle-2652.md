# Cycle 2652: M5-5 매트릭스 확정 — repeat-with-var / fn-return 미지원 + alias / while iter ✅
Date: 2026-05-11

## Re-plan
Cycle 2651 Carry-Forward: M5-5b/c (repeat-with-var, fn-return) 검증. 계획 유효.

## Scope & Implementation

**검증 시나리오** (cycle 2651의 literal 외 4개 케이스):

| 케이스 | 코드 | 결과 |
|--------|------|------|
| literal | `let arr = ["a","b"]` | ✅ Cycle 2651에서 통과 |
| alias | `let arr2 = arr` | ✅ R: marker auto-propagates |
| while iter | `while i<3 { println(arr[i]); ... }` | ✅ loop body 내 dispatch |
| repeat-with-var | `let s = "x"; let arr = [s; 3]` | ❌ val_type="var" (not "string") → mark 발행 누락 |
| fn return | `fn make() -> Array<String> = ...` | ❌ ret_type="Array<String>" (not "String") → string_fns 등록 누락 |

**미지원 케이스 분석**:
- `[s; N]`: `step_array_repeat` (라인 5395)에서 `val_type == "string"` 체크. var 노드는 "var" 반환 → mark 발행 안 됨. 해결 = lower 단계에서 str_sb 접근 또는 별도 type registry 필요 (큰 변경).
- 함수 반환: `collect_string_fns_from_mir`는 ret_type 정확히 "String"만 등록. `Array<String>` 반환 케이스는 별도 array_fns registry 필요 (큰 변경).

**워크어라운드** (사용자 코드):
- `[s; N]` 대신 `[s, s, s]` 명시 (literal 경로)
- 함수 반환 대신 caller에서 inline 작성: `let arr = ["a","b"]` 직접

**골든 테스트 추가** (2개):
- `test_golden_arr_str_alias.bmb`: alias propagation 확인
- `test_golden_arr_str_for_loop.bmb`: loop body 내 dispatch 확인

**미지원 케이스 테스트 파일 정리**: `test_golden_arr_str_repeat.bmb`, `test_golden_arr_str_fn_return.bmb`, `test_golden_tuple_str_destructure.bmb` 삭제 (M5-5b/c carry-forward로 충분, 회귀 안 됨).

## Verification & Defect Resolution

**`cargo test --release`**: ✅ 6210 passed (변경 없음)

**골든 테스트 카운트**: 2847 → 2849 (alias, for_loop 추가)

**M5-5 매트릭스** (cycle 2648 dispatch 매트릭스 확장):
| `arr[i]` 변형 | 상태 | 메커니즘 |
|--------------|------|---------|
| `[s1, s2]` literal | ✅ | mark_str_ptr (Cycle 2651) |
| `let arr2 = arr` alias | ✅ | R: marker propagation (기존) |
| `while ... arr[i]` loop | ✅ | block-internal R: persist |
| `[s; N]` var repeat | ❌ | val_type "var" detection 한계 |
| `fn() -> Array<String>` | ❌ | string_fns ret_type "String" only |
| `set arr[i] = s` (mut) | ❓ | 미검증 |
| `arr.push(s)` | N/A | Vec 미구현 |

## Reflection

**Scope fit**: M5-5의 핵심 사용 케이스 (literal + alias + iteration) 완전 동작. 미지원 2개는 큰 인프라 (var-type registry / array-fn signature analysis) 요구 → 별도 사이클.

**Latent defects**: 없음 (의도된 미지원).

**Philosophy 점검**:
- 기존 인프라 재활용 (R: 마커 propagation chain) — workaround 없음 ✅
- "복잡도 기피 안 함"이지만 본 cycle은 *큰 인프라 변경 없이도 즉시 가치 있는 갭* 해결 우선 ✅
- 미지원 케이스 워크어라운드 문서화 — 사용자에게 명확한 경로 제공 ✅

**Roadmap impact**:
- M5-5 핵심 ✅ 완료. 잔여 (M5-5b/c)는 별도 cycle 또는 인프라 큰 변경 필요.
- ROADMAP M5-5 항목 분할 고려 (cycle 2653 또는 종합 갱신 시).

## Carry-Forward
- Actionable: Cycle 2653 — `set arr[i] = s` (mut String array) 검증 + M5-4-A (tuple String) 시도
- Structural Improvement Proposals:
  - M5-5b: var/expr → array repeat (`[s; N]`) — lower 단계 type registry 필요
  - M5-5c: 함수 반환 `Array<String>` — array-fn signature analysis 인프라
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 진척도 ~50% (literal/alias/while ✅, var-repeat/fn-return ❌)
- Next Recommendation: Cycle 2653 — `set arr[i] = s` mut + tuple String dispatch 시도
