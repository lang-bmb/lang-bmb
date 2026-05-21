# Cycle 2865: f64 수학 free function 완성 (log/exp/round/tan/atan/atan2)
Date: 2026-05-15

## Re-plan
Carry-Forward 없음. `log(x)`, `exp(x)`, `round(x)`, `tan(x)`, `atan(x)` 등이 f64 메서드로만 있고 free function이 없는 갭 발견.
bmb_reference.md에서 `log(2.718)` 형태로 예시하면서 실제로는 작동 안 하는 일관성 버그.

## Scope & Implementation
- `bmb/src/interp/eval.rs`:
  - `f64_unary_builtin!` 매크로로 7종 구현: `log(ln)`, `log2`, `log10`, `exp`, `round`, `tan`, `atan`
  - `builtin_atan2(y, x) -> f64` (2-argument variant)
  - 8종 등록: `log`, `log2`, `log10`, `exp`, `round`, `tan`, `atan`, `atan2`
- `bmb/src/types/mod.rs`:
  - 단항 `f64 -> f64` 7종 + 이항 `(f64, f64) -> f64` 1종 (`atan2`) 추가
- `bmb/tests/integration.rs`: `test_interp_math_free_functions` 신규 (9개 assert)
  - log(1.0) ≈ 0, exp(0.0) ≈ 1, round(3.7) = 4.0, round(3.2) = 3.0,
    log10(100.0) ≈ 2.0, log2(8.0) ≈ 3.0, tan(0.0) = 0.0, atan(0.0) = 0.0, atan2(0.0, 1.0) = 0.0

## Verification & Defect Resolution
- `cargo test --release -p bmb`: **2385 PASS** ✅ (단위 3778 + 통합 2385)
- defect 없음

## Reflection
- Scope fit: ✅ 수학 free function이 메서드와 동등하게 동작
- bmb_reference의 `log(x)` 예시가 실제로 3사이클 동안 broken 상태였음 — 문서 예시도 실제 테스트 필요
- `f64_unary_builtin!` 매크로 활용으로 반복 코드 최소화
- Roadmap: M4 ① 언어 갭 계속 해소 중

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: bmb_reference 예시를 통합 테스트로 검증하는 체계 (장기 아이디어)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2866 — bmb_reference의 Math Builtins 섹션 업데이트 (round/tan/atan/atan2 추가) + 새 언어 갭 탐색
