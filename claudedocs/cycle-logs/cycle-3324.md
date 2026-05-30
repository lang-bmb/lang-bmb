# Cycle 3324: declared 필드 JSON 배열 형식 개선 (P1)
Date: 2026-05-30

## Re-plan
이전 Carry-Forward의 P1 actionable: `module_capability` violation의 `declared` 필드가 무효 JSON이었음.
미커밋 변경(`json_esc(module_caps)` → `ms_caps_to_json(module_caps, 0, 1)`)이 이미 준비된 상태.
범위: 검증 + Stage 1 재빌드 + Fixed Point + cargo test.

## Scope & Implementation
- 미커밋 변경 확인: `bc_check_module_cap_fn` (line 46516)
  - Old: `"declared":[" + json_esc(module_caps) + "]` → `"declared":[pure]` (invalid JSON)
  - New: `"declared":[" + ms_caps_to_json(module_caps, 0, 1) + "]` → `"declared":["pure"]` (valid JSON)
- Stage 1 재빌드: `bootstrap/compiler_s1.exe` (~45초)
- 검증: old compiler → `[pure]` / new compiler_s1 → `["pure"]` 차이 확인

## Verification & Defect Resolution
- cargo test --release: 3800 + 2390 + ... = 6282 tests, 0 FAILED ✅
- Fixed Point: fp3324a.ll == fp3324b.ll ✅
- declared 필드 형식: `["pure"]` (valid JSON array) ✅

## Reflection
- 스코프 적합: P1 declared 형식 수정 완료. 이번 변경은 `ms_caps_to_json` (기존 함수)를 재사용.
- 철학 정렬: AI가 파싱하는 JSON 출력의 정확성 개선 (Rule 8: AI 친화 구조화 출력).
- 로드맵 영향: P1 완료. P2 (module_capability 전용 섹션) 진행 준비.
- 잠재 결함: `ms_caps_to_json`의 마지막 항목 이후 쉼표가 없는지 확인 필요 (다중 cap 테스트).

## Carry-Forward
- Actionable: M15 Phase 6b — module_capability 전용 섹션 분리 (P2), count_viol_entries 리팩토링 (P3)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3325 — M15 Phase 6b (module_capability 섹션 분리)
