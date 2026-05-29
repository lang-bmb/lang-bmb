# Cycle 3287: M12 Phase 6b — @pure fn 위반 Z3 통합
Date: 2026-05-29

## Re-plan
Carry-Forward: M12 Z3 더 깊은 통합 (Phase 6b). 계획 유효.

## Scope & Implementation
**목표**: `@pure fn`이 IO/Net/File/Sys 효과 함수를 호출 시 `effect-verify`에서 탐지

**신규 함수**:
1. `eff_collect_pure_calls(fn_name, calls, eff_map, pos, sb, isfirst) -> i64`
   - 단일 함수의 call set 스캔
   - callee에 선언된 effect가 있으면 pure violation 방출
2. `eff_collect_pure_violations(entries, eff_map, pos, sb, isfirst) -> i64`
   - 전체 entries 스캔
   - sig에 "@pure fn" 또는 "@const fn" 포함 시 eff_collect_pure_calls 호출

**effect_verify_run 수정**:
- `eff_collect_pure_violations` 결합 (declared-effect violations 이후)
- status 결정 로직 수정: `z3_result == "unsat"` → violation, `has_violation` → violation, else → safe

**버그 발견/수정**:
- 초기 status 로직이 z3 "sat" 시 has_violation 무시 → 수정

**검증**:
- `@pure fn bad_fn calls io_fn` → violation 탐지 ✅
- `@pure fn good_fn calls helper` (helper = 일반 fn) → safe ✅
- 기존 declared-effect 위반 탐지 유지 ✅
- cargo test 3800+2390+23 PASS ✅

## Verification & Defect Resolution
모든 테스트 통과. Golden test 추가.

## Reflection
- **Scope fit**: Phase 6b 완성. lint+verify 통합으로 @pure violation 단일 진단점.
- **Architecture**: declared-effect Z3 + @pure heuristic scan 이중 레이어로 서로 보완.
- **Latent**: @const fn도 처리 ✅ (is_pure 체크에 포함).
- **Roadmap impact**: M12 deeper integration 진행 중.

## Carry-Forward
- Actionable: M12 Phase 6c - [missing_effect_annotation] 추론 → Z3 assertion (다음 사이클)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: ROADMAP에 Phase 6b 완료 마킹 필요
- Next Recommendation: M12 Phase 6c 또는 M15 Phase 5
