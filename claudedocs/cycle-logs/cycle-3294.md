# Cycle 3294: contracts-check require_effect_annotation JSON 통합
Date: 2026-05-29

## Re-plan
[P3] contracts-check require_effect_annotation 위반을 JSON violations에 포함.

## Scope & Implementation
**신규 함수**: `bc_check_missing_eff_anno(entries, direct_map, transitive_map, pos, sb, isfirst) -> i64`
- eff_collect_missing_annot와 동일한 로직, JSON rule="require_effect_annotation" 형식
- @pure fn, main, 선언있음, platform은 제외

**contracts_check_run 수정**:
- 기존: `lint_check_missing_effect_annotations` count만 확인
- 신규: `bc_check_missing_eff_anno` → JSON violations 직접 방출
- `has_viol` 계산 통합 (f4)

**검증**:
- missing_fn → [require_effect_annotation] 위반 탐지 ✅
- declared_fn, pure_fn, main → 제외 ✅
- cargo test 3800+2390+23 PASS ✅

## Verification & Defect Resolution
모든 테스트 통과.

## Carry-Forward
- Actionable: 최종 커밋 + HANDOFF 갱신 (Cycle 3295)
- Structural: module-suggest declared==suggested set-equality 비교 (low priority)
- Next: 최종 정리 사이클
