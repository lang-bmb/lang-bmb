# Cycle 3312: violations_count 필드 추가
Date: 2026-05-29

## Re-plan
lint_effects에만 count 필드 있음 → effect_verify, contracts_check에도 추가.

## Scope & Implementation
- `count_caller_entries(s, pos)`: `{"caller":` 패턴 카운터 (effect_verify violations 형식)
- `eff_verify_build_json`: `viol_str` 빌드 후 `count_caller_entries` → `violations_count:N` 추가
- `cc_build_json`: `viol_str` 빌드 후 `count_rule_entries` → `violations_count:N` 추가
- `sb_build(viol_sb)` 중복 호출 → `viol_str` 변수로 재사용 (효율)

## Verification & Defect Resolution
- 초기: `count_rule_entries`를 effect_verify에도 사용 → 0 반환 (violations이 `{"caller":` 형식)
- 수정: `count_caller_entries` 추가 → `violations_count:1` 정확히 표시
- cargo test: 3800+47+22+2390+23 PASS (0 FAILED)
- Stage 1 재빌드 ✅

## Reflection
- 진단 출력의 세 섹션 모두 count 필드 일관성 달성
  - effect_verify: violations_count ({"caller": 패턴)
  - contracts_check: violations_count ({"rule": 패턴)
  - lint_effects: count ({"rule": 패턴)
- AI 에이전트가 violations 배열 파싱 없이 count로 즉시 파악 가능

## Carry-Forward
- Actionable: semantic_duplicate를 diagnose에 통합 또는 새 contracts 규칙
- Structural Improvement Proposals: violations 형식 통일 (모두 {"rule": 사용)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3313 — semantic_duplicate를 diagnose에 통합
