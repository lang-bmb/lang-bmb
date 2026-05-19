# Cycle 2964: 잔여 3문제 problem.md 근본 수정 (01/30/86)
Date: 2026-05-19

## Re-plan
Cycle 2963 Carry-Forward: 잔여 3문제(01_binary_search, 30_contract_chain, 86_heap_sort) 수정.
GPUStack 결과 JSON 분석으로 근본 원인 파악.

## Scope & Implementation

각 문제의 실제 실패 원인 분석 및 수정:

### 01_binary_search
- **실패 원인**: 모델이 leftmost search (`hi = mid - 1 when found`) 전략을 반복 생성.
  테스트는 "첫 mid 비교에서 발견 즉시 반환"을 기대 (all-1s array: lo=0,hi=4,mid=2 → expected 2).
- **수정**: 
  1. 문제 설명 "any valid index" → "first index found by mid-point comparison"
  2. Notes에 `lo + (hi - lo) / 2` (overflow-safe) 도입
  3. CRITICAL 경고: "When found: set ans=mid and lo=hi+1 — do NOT change hi"
  4. 완전한 `fn main()` 블록 코드 예시 제공

### 30_contract_chain
- **실패 원인**: Z3 counterexample `limit = 0` — `bound` 함수 `pre limit >= 0`만으로는
  `ret >= 0` 증명 불가 (x < 0이면 ret = x < 0 위반). 모든 10회 시도가 동일 코드로 동일 Z3 실패.
- **수정**: `bound` 함수에 `pre x >= 0 and limit >= 0` 추가 (파이프라인상 scale 출력이 >= 0 보장).
  Contract Requirement 섹션과 Notes 코드 동기화.

### 86_heap_sort
- **실패 원인**: `&&` 연산자가 BMB에서 short-circuit 미지원 (또는 OOB 접근 유발).
  `while swapped == 1 && finished == 0`, `left < n && vec_get(v, left) > k_val` 등에서
  `left >= n`일 때 `vec_get` 호출 → 메모리 오염 → 전 원소 garbage 값 출력.
  기존 Notes의 insertion sort 예시도 `&&` 사용 (`while j >= 0 && vec_get(v, j) > key`).
- **수정**: insertion sort 예시 → bubble sort (중첩 for 루프, `&&` 불필요).
  CRITICAL 경고: "BMB does NOT support `&&`/`||` — use nested `if`"

## Verification & Defect Resolution

세 수정안 모두 `bmb build` + 테스트 케이스 검증:
- 01: `1 5 1 1 1 1 1` → `2` ✓
- 30: `0 100 2 150 3 0 50 100` → `0 100 150`, Z3 check ✓
- 86: `5 5 3 1 4 2` → `1 2 3 4 5` ✓

`cargo test --release`: **6258 tests, 0 failed** ✓

## Reflection

- 각 문제의 실패는 모두 정확히 진단 가능한 패턴이었음 (테스트 결과 JSON 분석)
- 01: 모델이 leftmost/rightmost search를 일관되게 선호하는 경향 확인
- 30: Z3 formal verification의 엄밀성 — "논리적으로 맞다"고 사람이 생각해도 pre 조건 부재 시 counterexample 존재
- 86: `&&` OOB 접근으로 메모리 오염 패턴 — 같은 garbage 값이 여러 원소에 출력되는 특징
- **Roadmap impact**: `&&`/`||` 미지원은 B축 뿐 아니라 BMB 언어 표현력 한계. 다음 사이클 즉시 구현.

## Carry-Forward
- Actionable: `&&`/`||` BMB 언어 지원 구현 (언어 스펙 변경 — 근본 해결)
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정 (수정 반영 확인)
- Roadmap Revisions: None
- Next Recommendation: `&&`/`||` 구현 (parser → AST → types → codegen → interp → bootstrap)
