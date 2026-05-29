# Cycle 3301: diagnose lint_effects 섹션 통합
Date: 2026-05-29

## Re-plan
Carry-Forward: diagnose에 lint effect 경고 JSON 섹션 추가.

## Scope & Implementation
**신규 JSON 빌더 함수 (bootstrap/compiler.bmb)**:
- `lint_eff_pure_viol_sb` — effect_pure_violation 경고 수집 (sb 기반)
- `check_callee_missing_effects_sb` — effect_propagation 위반 수집 (sb 기반)
- `check_one_callee_effect_sb`, `check_calls_for_effects_sb` — 중간 헬퍼
- `lint_eff_propagation_sb` — 전체 effect_propagation 스캔
- `lint_missing_eff_sb` — missing_effect_annotation 경고 수집
- `lint_effects_build_json(input, entries, eff_map, transitive_map) -> String`
  - 3종 경고 통합 → `{"type":"lint_effects","file":"...","warnings":[...]}`

**diagnose_file 갱신**: `lint_effects` 섹션 추가
- `{"type":"diagnose",...,"effect_verify":{...},"contracts_check":{...},"lint_effects":{...}}`

## Verification & Defect Resolution
- Stage 1: 성공 ✅
- cargo test: 3800+2390+23 PASS, 0 FAILED ✅
- diagnose JSON: 3섹션 통합 출력 검증 ✅
  - effect_verify: violation 탐지 ✅
  - lint_effects.warnings: missing_effect_annotation 탐지 ✅

## Reflection
- 기존 3종 lint effect 함수(`lint_check_effect_violations`, `lint_check_effect_propagation`, `lint_check_missing_effect_annotations`)와 동일 로직 — 일관성 유지
- 빌더 패턴(sb 기반) 완전히 정착 (Cycle 3298 패턴 재사용)
- 로드맵 영향: diagnose CLI 완전체 달성

## Carry-Forward
- Actionable: Fixed Point 검증 + 커밋
- Structural Improvement Proposals: diagnose count 필드 추가 (총 경고 수)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3302 — Fixed Point 검증 + 커밋
