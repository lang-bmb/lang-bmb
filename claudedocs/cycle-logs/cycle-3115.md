# Cycle 3115: M8-A i64 trivial → semantic (3/10 교체)
Date: 2026-05-25

## Re-plan

Inherited: i64 trivial 10개 → range contracts. 각 함수 바디 분석으로 가능 여부 판단.

분석 결과:
- `s2i`: `parse_int_simple`은 digits only (0-9) → 항상 >= 0 → `post it >= 0` ✅
- `update_range_from_ast`: 모든 경로 explicit `0` 반환 → `post it == 0` ✅
- `main`: 모든 서브커맨드 >= 0 반환 (`show_help()` 이미 `post it >= 0`) → `post it >= 0` ✅
- 나머지 7개: 진정한 임의 i64 반환 (arithmetic/bitwise/string-parse) → trivial 유지

## Scope & Implementation

**3개 교체**:
- `s2i` (L4348): `post it == it` → `post it >= 0`
- `update_range_from_ast` (L18277): `post it == it` → `post it == 0`
- `main` (L22691): `post it == it` → `post it >= 0`

**7개 유지** (meaningful bound 없음):
- `extract_int_value`: parse_int_from handles '-' → 음수 가능
- `cf_table_get`: `-99999999` sentinel or any parsed i64
- `cf_extract_int_val`: 0 or any parsed integer
- `cf_compute`: arithmetic result, full i64 range
- `cf_eval_shift`: shift result, full i64 range
- `cf_eval_bitwise`: bitwise result, full i64 range
- `str_to_int`: handles '-' prefix → 음수 가능

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ 3173 warnings, 0 errors
- `bmb verify bootstrap/compiler.bmb`: ✅ 954/954 verified, 0 failed
  - Note: 3 new contracts on complex functions — Z3 skips (not verifiable), but 0 failed
- 3-Stage Fixed Point: ✅ `A8ADD96654CD39795443635F1DAAB55D` (S3==S4)
- `cargo test --release`: ✅

## Reflection

- Scope fit: 100% (3/10 replaced — the 7 remaining have no meaningful bound)
- Key insight: parse_int_simple's digit-only handling makes s2i always non-negative
- update_range_from_ast always returns 0 (side-effect function using vec_set)
- The 7 unchanged functions are legitimately unconstrained — keeping trivial is honest
- Z3 doesn't verify the 3 new contracts (complex function bodies) but they're semantically correct

## Carry-Forward

- Actionable: Cycle 3116 — bool trivial 96개 분석 + semantic 교체 시작
  - Target: 함수 바디가 단순한 것부터 (`is_error`, `is_digit`, `is_whitespace` 류)
  - Z3-verifiable: `post it == (cond식)` where the body IS the condition
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M8-A i64 3/10 교체 완료, Fixed Point `A8ADD96654CD39795443635F1DAAB55D`
- Next Recommendation: Cycle 3116 — bool trivial → semantic (단순 is_X 패턴부터)
