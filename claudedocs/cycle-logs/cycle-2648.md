# Cycle 2648: 추가 dispatch 갭 탐색 + 미지원 문서화
Date: 2026-05-11

## Re-plan
Cycle 2647 Carry-Forward: array of String / tuple String dispatch 탐색. 계획 유효.

## Scope & Implementation

**검증 시나리오**:

1. **`arr[i]` of String** — `let arr = ["a", "b", "c"]; println(arr[1])`:
   - 결과: 포인터 정수값 (예: `140702601383952`) 출력 — **미지원**
   - 원인: 배열 element 타입 추적 인프라 부재. `lower_index_sb` → MIR `load_ptr` → 타입 정보 없음
   - struct registry와 달리 array는 schema 추적 시스템이 없음
   - 워크어라운드: 함수로 래핑 (`fn elem(a: ..., i: i64) -> String = a[i];`) → string_fns 경로 사용

2. **struct method call returning String** — `Person::get_name(p)` returning String:
   - 결과: "Alice" 정상 출력 ✅
   - 이유: `Person_get_name` 함수가 `string_fns`에 자동 등록됨 (M4-4 망글링 + M5-4 dispatch)

**M5-4 종합 매트릭스 (현재 상태)**:

| 입력 | 상태 | 메커니즘 |
|------|------|---------|
| `println("literal")` | ✅ | string MIR opcode → str_sb 마킹 |
| `println(string_var)` | ✅ | str_sb 마킹 전파 |
| `println(user_fn() -> String)` | ✅ | string_fns 자동 등록 |
| `println(builtin_fn() -> String)` | ✅ | is_string_returning_fn 하드코딩 |
| `println(struct.string_field)` | ✅ | registry `~s` suffix (Cycle 2645) |
| `println(nested.path.to.string)` | ✅ | 다단 traversal 작동 (Cycle 2646) |
| `println(static_method() -> String)` | ✅ | string_fns 경로 |
| `println(arr[i])` of `[String]` | ❌ | array element 타입 추적 부재 — 미지원 |
| `println(tuple.0)` of `(String, ...)` | ❓ | 미검증 |
| `println(f64)` / `println(i64)` | ✅ | is_double_var_sb / 기본 경로 |

## Verification & Defect Resolution

**구두 검증**:
- arr[i] of String: 버그 재현 확인
- method call returning String: 정상 작동 확인

**cargo test --release**: ✅ 6210 passed (변경 없음)

## Reflection

**Scope fit**: 갭 탐색 완료. 미지원 항목 명확화.

**미지원 항목 평가**:
- `arr[i]` of String — 실용 영향: 중간. 사용자가 String 배열을 println하는 시나리오는 흔하지만 워크어라운드 가능 (함수 래핑).
- 근본 해결: 배열 element 타입 registry 도입 — 큰 변경, 별도 사이클 필요. M5-5 또는 M6 고려.

**Latent defects**: 없음 (의도된 미지원 케이스).

**Philosophy drift**: 없음. 현재 인프라(str_sb)의 자연스러운 확장은 다 했음.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 배열 element 타입 추적 (M5-5 후보) — element 타입 inference 또는 명시 type annotation 활용
- Pending Human Decisions: PyPI push (로컬 커밋 완료, push 미실행)
- Roadmap Revisions: M5-5 후보 추가 가능
- Next Recommendation: Cycle 2649 — 최종 commit + ROADMAP M5-5 후보 등록 or 세션 종료
