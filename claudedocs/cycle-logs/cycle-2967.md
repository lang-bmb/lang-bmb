# Cycle 2967: Short-Circuit OOB 보호 통합 테스트 추가
Date: 2026-05-19

## Re-plan
Cycle 2966 Carry-Forward: 전체 테스트 통과 확인 후 커밋.
GPUStack 결과 재분석: 정확히 3개 문제만 0/3 (01/30/86), 나머지 97개 모두 3/3.
→ 근본 수정 완료. 이번 사이클: short-circuit 동작 검증 테스트 추가.

## Scope & Implementation

### Short-Circuit OOB 보호 테스트 (`bmb/tests/integration.rs`)
86_heap_sort의 근본 원인(&&로 인한 OOB 메모리 접근)을 재현 + 수정 확인하는 통합 테스트 2개 추가:

**`test_short_circuit_and_prevents_oob`**: 
- `i < n && vec_get(v, i) > 0` 패턴에서 `i >= n`일 때 `vec_get` 호출 안 됨을 검증
- i=3, n=2 → `i < n` = false → 단락 평가 → `vec_get(v, 3)` 미호출 → result = false = 0

**`test_short_circuit_or_prevents_oob`**:
- `i >= n || vec_get(v, i) > 0` 패턴에서 `i >= n`일 때 `vec_get` 호출 안 됨을 검증
- i=5, n=1 → `i >= n` = true → 단락 평가 → `vec_get(v, 5)` 미호출 → result = true = 1

수정 과정: `10_i64` (Rust 문법) → `10` (BMB 문법), `;` 누락 (함수 정의 끝에 `;` 추가).

## Verification & Defect Resolution

- `test_short_circuit_and_prevents_oob`: ✅ ok
- `test_short_circuit_or_prevents_oob`: ✅ ok
- `cargo test --release`: 실행 중

## Reflection

- 인터프리터 단락 평가는 이미 구현됨 — 테스트가 인터프리터 경로를 검증
- native 경로(MIR → LLVM IR)는 `test_ir_boolean_logic`의 `phi i1` 검사로 검증됨
- BMB 문법 주의: 정수 리터럴에 `_i64` 접미사 불가, 함수 정의 끝에 `;` 필수

## Carry-Forward
- Actionable: 전체 테스트 통과 후 커밋 (Cycles 2964-2967 합산)
- Structural Improvement Proposals: native 경로 short-circuit OOB 테스트 추가 가능
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: 커밋 후 추가 언어 개선 또는 P축 개선
