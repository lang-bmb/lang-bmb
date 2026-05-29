# Cycle 3296: index/query platform 블록 스킵 버그 수정
Date: 2026-05-29

## Re-plan
Plan valid, HANDOFF P2 계승.

## Scope & Implementation
**P2 버그 수정**: `index_source`와 `query_source`의 platform 블록 swallow 버그 수정.
- `bootstrap/compiler.bmb:36738-36748` — `index_source`에 TK_IDENT "platform" 분기 추가 → `skip_platform_block` 호출
- `bootstrap/compiler.bmb:36917-36927` — `query_source`에 동일 패턴 적용
- `callers_collect_source` (Cycle 3289 수정)와 동일한 `skip_nested_brace`/`skip_platform_block` 인프라 재사용

**검증**: `compiler_s1.exe index tests/tmp_platform_index_test.bmb` → 2 functions, 0 structs (플랫폼 내부 fn 배제 ✅)

## Verification & Defect Resolution
- Stage 1: `target/tmp/compiler_s1.exe` 빌드 성공 ✅
- cargo test: 3800+2390+22 PASS, 0 FAILED ✅
- index/query 기능 수동 검증 ✅

## Reflection
- 수정이 `callers_collect_source` 패치와 동일한 패턴 — 일관성 유지
- annotation 분기에는 TK_IDENT 추가 불필요 (platform은 항상 비-annotation 컨텍스트)
- 로드맵 영향: 없음

## Carry-Forward
- Actionable: P3 module-suggest set-equality 비교 (다음 사이클)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3297 — module-suggest eff_set_equals 구현
