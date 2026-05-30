# Cycle 3326: count_viol_entries 통합 리팩토링 (P3)
Date: 2026-05-30

## Re-plan
이전 Carry-Forward: count_viol_entries 통합. count_caller_entries, count_fn_a_entries, count_rule_entries 세 함수가 동일 구현.

## Scope & Implementation
- `count_caller_entries` 본체를 `count_rule_entries(s, pos)` 위임으로 변경
- `count_fn_a_entries` 본체를 `count_rule_entries(s, pos)` 위임으로 변경
- `count_rule_entries`가 유일한 실제 구현; 나머지 두 함수는 순수 alias
- 기존 호출 사이트 변경 없음 (backwards compatibility 유지)

## Verification & Defect Resolution
- Stage 1 재빌드: 40s ✅
- Fixed Point: fp3326a.ll == fp3326b.ll ✅
- 기능 테스트: module_cap_issues 카운팅 정확 (1) ✅

## Reflection
- 순수 리팩토링. 기능 변경 없음.
- 향후 `count_rule_entries` → `count_viol_entries` 이름 변경도 가능하지만 기존 호출 사이트 4곳 변경 필요.
- 현재 alias 방식으로 충분; 더 변경하려면 STEP 4에서 논의.
- 로드맵 영향: P3 리팩토링 완료. P3 MCP 업데이트, P2 bootstrap P-track으로 이동.

## Carry-Forward
- Actionable: MCP bmb_diagnose 스키마 업데이트 (P3), bootstrap P-track 회귀 분석 (P2)
- Structural Improvement Proposals: `count_rule_entries`를 `count_viol_entries`로 이름 변경 (선택 사항)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3327 — MCP bmb_diagnose 스키마 업데이트 (P3)
