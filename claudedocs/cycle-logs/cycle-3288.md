# Cycle 3288: M12 Phase 6c — missing_effect_annotation Z3 통합
Date: 2026-05-29

## Re-plan
Carry-Forward: M12 Phase 6c. 계획 유효.

## Scope & Implementation
**목표**: `[missing_effect_annotation]` 탐지를 `effect_verify` 결과에 통합

**신규 함수**:
- `eff_collect_missing_annot(entries, direct_map, transitive_map, pos, sb, isfirst) -> i64`
  - transitive_map에 효과가 있으나 direct_map에 없는 함수 탐지
  - @pure/@const fn 제외 (pure violation에서 별도 처리)
  - main 제외 (entry point 노이즈)
  - "caller_effect":"missing" 타입으로 JSON 방출

**effect_verify_run 수정**:
- transitive_map 생성 추가 (`build_full_transitive_effect_map(entries, eff_map, 5)`)
- `eff_collect_missing_annot` 호출 추가

**3가지 위반 유형**:
1. `"missing"`: 미선언 함수가 transitive IO/Net/File/Sys 사용
2. `"pure"`: @pure/@const fn이 효과 함수 호출
3. Z3 UNSAT: 선언된 효과 간 불일치 (기존)

**검증**:
- wrapper calls io_fn (no annotation) → [missing IO] ✅
- @pure bad_fn calls io_fn → [pure IO], no duplicate missing ✅
- bad_caller (IO) calls safe_net (Net) → Z3 UNSAT + [IO vs Net] ✅
- cargo test 3800+2390+23 PASS ✅

## Verification & Defect Resolution
모든 테스트 통과. 세 가지 위반 유형 모두 올바르게 구분.

## Reflection
- **Scope fit**: Phase 6c 완성. lint와 effect_verify 통합 강화.
- **Design**: main 제외 + pure/const 제외로 노이즈 감소. 필요 시 옵션화 가능.
- **Roadmap impact**: M12 deeper Z3 통합 방향 계속 진행 가능.

## Carry-Forward
- Actionable: Effect lattice 모델링 (IO ⊆ IO+Net partial order) 또는 M15 Phase 5
- Structural Improvement Proposals: main 제외 옵션화 (--include-main 플래그)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: M15 Phase 5 (platform → module capability 자동 연계) 또는 contracts-check 개선
