# Cycle 3315: diagnose summary 섹션 추가 (P2)
Date: 2026-05-30

## Re-plan
Cycle 3314 Carry-Forward: P2(summary) 또는 P4(forbid_function). P2를 먼저 실행 — AI 통합성을 즉시 개선하는 1-사이클 태스크.

## Scope & Implementation
- `json_extract_int_field(json, field)` 헬퍼 함수 추가 (compiler.bmb 라인 46772 앞)
- `diagnose_file()` 수정: ev_count/cc_count/le_count/sd_count 추출 → summary JSON 생성
- 출력 형식: `"summary":{"total_issues":N,"effect_issues":N,"contract_issues":N,"lint_issues":N,"duplicate_pairs":N}`

## Verification & Defect Resolution
- cargo test: 3800+47+22+2390+23 = 6282 PASS, 0 FAILED ✅
- Stage 1 build: 성공 (26s) ✅
- diagnose 동작 확인: safe 파일 → all 0, violation 파일 → 정확한 카운트 ✅
- Within-gen Fixed Point: fp3315a.ll == fp3315b.ll ✅

## Reflection
- 구현이 간결하고 정확: `find_substr_pos` + `parse_int_from` 조합으로 JSON에서 값 추출
- summary 섹션은 AI가 파일 품질을 한 필드로 파악하게 해줌 (total_issues가 0이면 안전)
- 비파괴적 추가: 기존 4섹션 구조 유지, 마지막에 summary 추가

## Carry-Forward
- Actionable: P4 forbid_function rule 구현 (1 사이클)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3316 — P4 forbid_function contracts-check 규칙
