# Cycle 3271: M12 Phase 4 — Transitive Effect Propagation
Date: 2026-05-29

## Re-plan
Carry-forward: M13 Phase 4, M12 Phase 4. M12 Phase 4 먼저 진행 (인프라 선행). 계획 유효.

## Scope & Implementation
**목표**: A calls B (undeclared) calls C (<Net>) → A must declare <Net>

**추가 함수** (bootstrap/compiler.bmb 42419 이후 삽입):
- `eff_add_if_new` — 효과 집합에 새 효과 추가
- `eff_merge_effects_pos` / `eff_merge_effects` — 두 효과 집합 병합
- `expand_calls_effects_trans` — 호출자 효과를 callee 효과로 확장
- `build_transitive_pass(entries, direct_map, old_transmap, pos, acc)` — 1 pass
  - 명시적 효과 선언 함수: direct_map 값 그대로 유지
  - 효과 선언 없는 함수: callee 전이 효과 확장
- `build_transitive_effect_map_iter` / `build_transitive_effect_map` — 수렴 반복

**수정** (lint_file):
- `let transitive_map = build_transitive_effect_map(entries, eff_map, 5);`
- `let w8 = lint_check_effect_propagation(entries, transitive_map, 0, 0);`

**골든 테스트**: `tests/golden/test_golden_effect_transitive.bmb`
- a_handler: <IO>, calls b_helper (undeclared), b_helper calls c_fetch: <Net>
- 결과: `[effect_propagation] a_handler: declares <IO> but calls b_helper which uses <Net>` ✅

**버그 수정**: 초기 설계에서 transitive_map이 caller에도 적용되어 check가 trivially pass
→ explicit 선언 함수는 direct effect만 유지, 미선언 함수만 전이 확장으로 수정

## Verification & Defect Resolution
- cargo test --release: 3800+47+22+2390 = 8259 tests, 0 FAILED ✅
- Stage 1 (compiler_3271.exe) 빌드 성공 ✅
- test_golden_effect_transitive.bmb lint: 2 warnings (unused + effect_propagation) ✅
- test_golden_effect_propagation.bmb lint: 1 warning (unused only, no regression) ✅

## Reflection
- Scope fit: ✅ M12 Phase 4 목표 달성
- Latent: `build_transitive_pass`가 5-param 함수. BMB lint [complex] 경고 가능성.
- Fixed Point 미완 (다음 사이클)

## Carry-Forward
- Actionable: Fixed Point S2==S3 검증 + bmb lint warnings 체크
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3272 — Fixed Point + lint warnings 확인
