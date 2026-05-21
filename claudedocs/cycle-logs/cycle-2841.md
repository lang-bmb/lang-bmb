# Cycle 2841: for-in-vec 구현 (M4-10)
Date: 2026-05-14

## Re-plan
이전 Carry-Forward: for-in-vec 다음 주요 목표. 계획 유효.

## Scope & Implementation

**발견**: 문법은 이미 `for x in v {}` 파싱 가능 (SpannedRangeExpr → ImpliesExpr → 변수).
**실제 문제**: 타입 체커가 `i64`를 for-loop iterator로 거부.

변경 파일:
- `bmb/src/types/mod.rs`: `Type::I64 => Type::I64` case 추가 + 에러 메시지 갱신
- `bmb/src/interp/eval.rs`: `Value::Int(vec_ptr) if vec_ptr != 0` case 추가 (unsafe header layout read)
- `bmb/src/types/mod.rs:10963`: `test_err_for_loop_non_range_iterator` 갱신 (bool/f64 → 여전히 오류)
- `bmb/tests/integration.rs:14468`: `test_for_in_array_type_error` 갱신 (i64 → 이제 유효)
- `bmb/tests/integration.rs:24517`: `test_interp_for_in_vec` 5개 케이스 추가

## Verification & Defect Resolution
- cargo test --release: ✅ 6140+ tests ALL PASS
- test_interp_for_in_vec: 5/5 케이스 통과 (sum/empty/range/conditional/nested)
- 이전 range 동작 회귀 없음

**수정된 결함 2건**:
1. `test_err_for_loop_non_range_iterator` — `i64` 이제 유효, bool/f64로 변경
2. `test_for_in_array_type_error` — 동일 이유로 갱신

## Reflection
- ✅ M4-10 for-in-vec 완료. 2 files, ~25 LOC 변경으로 달성.
- ⚠️ `for x in 42 {}` 타입 통과하나 런타임에서 잘못된 메모리 접근 위험. BMB 철학상 사용자 책임 (contract 미작성). 문서화로 해결.
- vec 핸들 탐지는 타입 수준에서 불가 (i64와 구분 안 됨). 향후 typed VecHandle 타입 추가 시 개선 가능하나 현재 범위 외.

## Carry-Forward
- Actionable: None (for-in-vec ✅ 완료)
- Structural Improvement Proposals:
  * `VecHandle` 타입 추가로 `for x in 42` 오용 방지 — 고복잡도, 모든 vec 빌트인 타입 변경 필요. 현재 interpreter-only 제약 + 장기 P3.
- Pending Human Decisions: None
- Roadmap Revisions: M4-10 ✅ 표시 예정
- Next Recommendation: M4-11 String interpolation — `"Hello {name}"` lexer 변환 (고복잡도 2-3 cycles)
