# Cycle 3298: diagnose CLI 통합 구현
Date: 2026-05-29

## Re-plan
Carry-Forward P1 계승.

## Scope & Implementation
**P1**: `diagnose` 명령 — effect-verify + contracts-check 통합 JSON.

**리팩터링**:
- `eff_verify_build_json(input, entries, eff_map, transitive_map) -> String` 추가
  - `effect_verify_run`의 JSON 빌딩 로직 추출 (println 제거)
- `cc_build_json(input, src, entries, eff_map, transitive_map, contracts) -> String` 추가
  - `contracts_check_run`의 JSON 빌딩 로직 추출
- `effect_verify_run`, `contracts_check_run` → 빌더 호출 + println_str 패턴으로 단순화

**신규 함수**: `diagnose_file(input: String) -> i64`
- entries/eff_map/transitive_map 한 번 계산
- `eff_verify_build_json` + `cc_build_json` 호출
- 통합 JSON 출력: `{"type":"diagnose","file":"...","effect_verify":{...},"contracts_check":{...}}`

**Command dispatch**: `diagnose` 분기 추가 (`effect-verify` 앞에)

## Verification & Defect Resolution
- Stage 1: 성공 ✅
- cargo test: 3800+2390+23 PASS, 0 FAILED ✅
- diagnose safe case: `status:"safe"` 양쪽 ✅
- diagnose violation: `effect_verify.status:"violation"` + violations 배열 ✅
- 기존 effect-verify/contracts-check 명령 동작 유지 ✅

## Reflection
- 빌더 분리로 중복 계산 제거 (entries/eff_map/transitive_map 1회)
- 기존 `effect_verify_run`/`contracts_check_run` API 불변 — 기존 코드 무영향
- 로드맵 영향: HANDOFF P1/P2/P3 전부 완료

## Carry-Forward
- Actionable: Fixed Point 검증 (Stage 1→2, within-gen FP)
- Structural Improvement Proposals: diagnose에 lint effect rules 섹션 추가 가능 (P4, 장기)
- Pending Human Decisions: None
- Roadmap Revisions: ROADMAP § 6 다음 세션 태스크 P1/P2/P3 완료 마킹 필요
- Next Recommendation: Cycle 3299 — Fixed Point within-gen 검증 + 커밋
