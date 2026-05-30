# Cycle 3321: M15 Phase 6a — enforce_module_caps contracts 규칙
Date: 2026-05-30

## Re-plan
P3 M15 Phase 6 시작. Phase 6a: `enforce_module_caps = true` 규칙 구현.

## Scope & Implementation
- `bc_check_module_cap_fn`: 함수의 transitive effect가 module_caps를 초과하면 violation 방출
- `bc_check_module_caps_scan`: entries 전체 스캔 (lint_check_module_capabilities의 contracts 버전)
- `cc_build_json` 수정: `enforce_caps`, `module_caps` 변수 추가 + f6 단계에서 bc_check_module_caps_scan 호출

## Verification & Defect Resolution
- cargo test: 3800+47+22+2390+23 = 6282 PASS, 0 FAILED ✅
- Stage 1 build (compiler_s1f.exe): 성공 ✅
- enforce_module_caps 동작 확인: module requires [IO] + Net 사용 → 3 violations 정확 ✅
- Within-gen Fixed Point: fp3321a.ll == fp3321b.ll ✅

## Reflection
- M15 Phase 6a 완성: lint 경고 → contracts 위반 으로 격상 가능
- 프로젝트가 .bmb-contracts에 enforce_module_caps = true 추가하면 모듈 capability가 실질적으로 강제됨
- declared 필드에 효과 목록이 JSON 배열 형식이나 문자열로 들어감 (개선 여지 있음)

## Carry-Forward
- Actionable: Cross-gen FP 검증 (S2==S3) + 커밋
- Structural Improvement Proposals: declared 필드를 JSON 배열 `["IO","Net"]`로 개선 (현재는 문자열)
- Pending Human Decisions: None
- Roadmap Revisions: M15 Phase 6a 완료 마킹 예정
- Next Recommendation: Cycle 3322 — Cross-gen FP + 커밋 + HANDOFF 갱신
