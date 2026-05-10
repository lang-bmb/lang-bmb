# Cycle 2650: Re-plan + tuple String dispatch 갭 발견
Date: 2026-05-11

## Re-plan
직전 세션 종료(2649) Carry-Forward: 액션 없음. 후보 = M5-5 (array element), M3-2 (bmb-algo 벤치). 새 세션 시작 — `cargo test` 6210 PASS, HEAD `d9855b04`.

## Scope & Implementation

**활동**: 직전 cycle 2648에서 ❓로 남은 `println(tuple.0)` of `(String, ...)` 검증.

테스트 작성: `tests/bootstrap/test_golden_tuple_str_destructure.bmb`
```bmb
fn make() -> (String, i64) = ("hello", 42);
fn main() -> i64 = {
    let (s, n) = make();
    println(s);
    println(n);
    n
};
```

**결과**: ❌ `s`가 포인터 정수(`140698014584848`) 출력. tuple destructuring 경로에서 String dispatch 미작동.

**구조 분석** (`bootstrap/compiler.bmb` 1118-1166, 1930-1945):
- `let (a, b) = expr` → `let __tup_X = expr in let a = (field __tup_X 0) in let b = (field __tup_X 1) in body`
- 각 component는 `(field tup idx)` AST 노드로 추출 — struct field와 동일 노드
- struct field는 registry(`~s` suffix, Cycle 2645)로 String 추적 → tuple은 schema 없음

## Verification & Defect Resolution

**검증**: 갭 재현 확인.

**복잡도 평가**:
- 인라인 tuple literal (`("a", 42)`): RHS 컴파일타임 분석 가능 (간단)
- 함수 반환 (`make() -> (String, i64)`): 반환 타입 추적 필요 (큰 변경)
- 두 lowering 경로 (recursive `lower_expr_sb` + iterative `step_expr`) 양쪽 수정 필요 (Rule 3)

**테스트 파일은 그대로 유지** (다음 cycle 회귀 검증용으로 활용).

## Reflection

**Scope fit**: 갭 명확화 완료. M5-4 매트릭스 ❓ → ❌ 확정.

**우선순위 비교** (M5-5 vs M5-4-A tuple String):
| 항목 | 사용 빈도 | 구현 난이도 | 인프라 |
|------|---------|-----------|--------|
| `arr[i]` of String (M5-5) | 중-고 | 큰 (registry 신규) | array element type registry |
| `let (s, n) = ...` String component | 낮 | 중 (tuple-only inline 분석) | 인라인 RHS 추론 |

→ **M5-5 우선**: 사용 빈도 높음 + 인프라 가치(다른 dispatch에 재활용 가능). Tuple String은 M5-4-A로 carry.

**Latent defects**: 없음 (의도된 갭).

**Roadmap impact**:
- M5-4-A 신규 후보 등록 (tuple destructuring + String component)
- M5-5 다음 cycle부터 본격 시작

## Carry-Forward
- Actionable: 다음 cycle (2651) — M5-5 array element 타입 추적 인프라 설계 시작
- Structural Improvement Proposals:
  - M5-4-A: tuple destructuring + String component dispatch (인라인 literal부터 점진 확대)
  - M5-5: `[String; N]` array element 타입 추적 (element-type registry 또는 array decl 기반 추적)
- Pending Human Decisions: 변경 없음 (PyPI push, NuGet, M3-1, M4-1)
- Roadmap Revisions: M5-4-A 후보 등록 예정 (cycle 2651 또는 이후 종합 갱신)
- Next Recommendation: Cycle 2651 — `[String; N]` array decl 추적 → element 타입 dispatch 1차 인프라 설계
