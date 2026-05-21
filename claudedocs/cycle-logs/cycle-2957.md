# Cycle 2957: 오해 유발 진단 패턴 수정 + 추가 problem.md 개선
Date: 2026-05-19

## Re-plan

Cycle 2956 완료. 실제 테스트로 확인: `_` wildcard와 tuple destructuring 모두 BMB에서 동작함. 이를 잘못 안내하는 진단 패턴 2개 발견 → 즉시 수정.

## Scope & Implementation

**오해 유발 진단 패턴 수정** (diagnostics/patterns.rs):
1. `tuple_destruct`: "BMB does not support tuple destructuring" → 블록 컨텍스트에서는 동작함을 명시
   - 수정: "Tuple destructuring `let (a, b) = expr;` WORKS inside a block { ... }"
   - `_` wildcard trigger 제거 (중복 위험)
2. `match_wildcard`: "BMB does not support underscore patterns in match" → `_ =>` 동작함!
   - 수정: `_` wildcard 동작함 명시, `_ ->` (잘못된 화살표)만 trigger로 남김

**추가 problem.md 개선** (7개 문제):
- 55_token_count: parallel arrays 대신 linear search 패턴
- 58_spiral_order: flat matrix + print/print_str 패턴
- 78_event_loop: 양쪽 type+value 읽기 필수
- 76_multi_function: abs() 빌트인 + sign 패턴

## Verification & Defect Resolution

테스트 실행 중 (백그라운드).

실제 언어 검증:
- `match x { 1 => a, _ => b }` → `99` ✅ (wildcard 동작)
- `let (a, b) = pair; a + b` → `7` ✅ (tuple 블록 컨텍스트 동작)

## Reflection

- 핵심 발견: 잘못된 진단이 B-loop를 야기할 수 있음 — LLM이 동작하는 코드를 포기하게 만듦
- bool_operators (Cycle 2954) + tuple_destruct + match_wildcard = 3개 잘못된 진단 수정
- 각 수정이 LLM 성공률 직접 향상 가능

## Carry-Forward

- Actionable: diagnostics 테스트 결과 확인 필요
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: Cycle 2958 → 전체 테스트 확인 + 나머지 개선 탐색
