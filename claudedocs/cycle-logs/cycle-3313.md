# Cycle 3313: semantic_duplicate → diagnose 통합
Date: 2026-05-29

## Re-plan
diagnose 4번째 섹션으로 semantic_duplicate 추가.

## Scope & Implementation
- `semdp_build_json(input, entries)`: 기존 semdp_run에서 JSON 빌더 분리 (print 없이)
  - `pairs_count` 필드 추가 (count_fn_a_entries 사용)
- `count_fn_a_entries(s, pos)`: `{"fn_a":` 패턴 카운터 (semdp 형식)
- `semantic_duplicate_run`: semdp_build_json 호출로 단순화
- `diagnose_file`: `sd_json = semdp_build_json(input, entries)` + 4번째 섹션 출력

## Verification & Defect Resolution
- cargo test: 3800+47+22+2390+23 PASS (0 FAILED)
- Stage 1 재빌드 ✅
- 테스트 (3 shared calls): pairs_count:1, fn_a:"dup1", fn_b:"dup2" ✅

## Reflection
- diagnose가 완전한 품질 통합 대시보드로 발전 (4섹션)
- entries 계산 재사용 (diagnose_file에서 한 번만 callers_collect_source 호출)
- violations 형식 불일치 ({"caller": vs {"rule": vs {"fn_a":}) 는 기술부채로 남음
  → 추후 통합 기회 있으나 현재는 breaking change

## Carry-Forward
- Actionable: Fixed Point 검증 후 커밋 + HANDOFF 갱신
- Structural Improvement Proposals: violations 형식 통일 제안 (모두 {"type":"..."} 형식으로)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3314 — Fixed Point + 커밋 + HANDOFF 갱신
