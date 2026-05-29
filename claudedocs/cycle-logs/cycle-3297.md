# Cycle 3297: module-suggest set-equality 비교 구현
Date: 2026-05-29

## Re-plan
Carry-Forward P3 계승.

## Scope & Implementation
**P3 수정**: module-suggest에서 declared_caps vs used_caps 비교를 순서-무관 set-equality로 교체.

**추가 함수 (bootstrap/compiler.bmb)**:
- `eff_subset(a, b, pos)`: a의 모든 cap이 b에 포함되는지 재귀 체크
- `eff_set_equals(a, b)`: `eff_subset(a,b,0) and eff_subset(b,a,0)` 양방향 포함

**수정 위치**: `module_suggest_file`의 `declared_caps == used_caps` → `eff_set_equals(declared_caps, used_caps)`

## Verification & Defect Resolution
- Stage 1: 성공 ✅
- cargo test: 3800+2390+23 PASS, 0 FAILED ✅
- 기능 검증: "File IO" declared + "IO File" suggested → status:"ok" ✅
- 불일치 케이스: declared:["IO"] vs suggested:["File","IO"] → status:"mismatch" ✅

## Reflection
- `eff_contains_name` + `eff_find_space_or_end` 기존 인프라를 그대로 재사용 — 일관성 유지
- `eff_subset`에 `pre pos >= 0` + `post it or not it` 계약 추가로 M9/Track B 스타일 유지
- 로드맵 영향: HANDOFF P3 완료

## Carry-Forward
- Actionable: P1 diagnose CLI 통합 (다음 사이클)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3298 — diagnose CLI (`compiler.exe diagnose`) 설계 및 구현 시작
