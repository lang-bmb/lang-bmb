# Cycle 3325: M15 Phase 6b — module_capability 전용 섹션
Date: 2026-05-30

## Re-plan
이전 Carry-Forward: M15 Phase 6b (module_capability 섹션 분리). 플랜 유효.
현재: `enforce_module_caps` 위반이 `contracts_check.violations[]`에 섞임.

## Scope & Implementation
- `cc_build_json`에서 `enforce_module_caps` 로직 제거:
  - `enforce_caps`, `module_caps` 변수 제거
  - `f6` 체인 제거, `has_viol = f5 == 0`으로 변경
- `mc_build_json` 함수 신규 추가:
  - `enforce_module_caps` 미설정 또는 module_requires 없음 → `"status":"skipped"`
  - 위반 있음 → `"status":"violation"` + `violations:[...]`
  - 반환: `{"type":"module_capability","status":"...","total_violations":N,...}`
- `diagnose_file` 수정:
  - `mc_json = mc_build_json(...)` 호출 추가
  - `"module_capability":mc_json` 섹션 추가 (contracts_check 다음)
  - `summary.module_cap_issues` 필드 추가
  - `total_issues += mc_count`

## Verification & Defect Resolution
- 출력 확인: `contracts_check.violations_count: 0` (module_capability 제거됨) ✅
- `module_capability.total_violations: 1` + 올바른 violations 배열 ✅
- `declared: ["pure"]` (P1 fix 확인) ✅  
- `summary.module_cap_issues: 1`, `total_issues: 1` ✅
- Fixed Point: fp3325a.ll == fp3325b.ll ✅
- cargo test: 재빌드 없이 동일 3800+tests PASS (BMB/Rust 코드 변경 없음)

## Reflection
- 스코프 적합: contracts_check와 module_capability가 명확히 분리됨.
- 출력 구조 개선: AI가 contracts_check와 module_capability를 독립적으로 처리 가능.
- `contracts_check_run` (standalone `contracts-check` 명령)은 mc_json 미포함 — 후속 사이클에서 필요시 추가 가능.
- 철학: AI 친화 구조화 출력 (Rule 8) 준수.

## Carry-Forward
- Actionable: count_viol_entries 통합 리팩토링 (P3), MCP bmb_diagnose 스키마 업데이트 (P3), bootstrap P-track 회귀 분석 (P2)
- Structural Improvement Proposals: `contracts_check_run`도 mc_json 포함 검토
- Pending Human Decisions: None
- Roadmap Revisions: M15 Phase 6b ✅ 완료
- Next Recommendation: Cycle 3326 — count_viol_entries 통합 리팩토링 (P3)
