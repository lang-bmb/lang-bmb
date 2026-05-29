# Cycle 3262: M12 Phase 3 — Effect Callee Propagation Lint
Date: 2026-05-29

## Re-plan
Carry-Forward에서 M12 Phase 3. Effect callee propagation을 bootstrap lint로 구현.
전략: Z3 대신 bootstrap 레벨에서 직접 호출 effect 전파 검사.

## Scope & Implementation
- `bootstrap/compiler.bmb`: M12 Phase 3 함수군 추가
  - `eff_clean(eff_sexp)`: "(eff IO Net)" → "IO Net" 변환
  - `eff_contains_name(eff, name)`: space-separated 효과 목록에서 이름 검색
  - `eff_map_get(map, name, pos)`: 효과 맵에서 함수 효과 조회
  - `build_fn_effect_map(src, pos, acc)`: 소스 스캔으로 "fn_name\teffect\n" 맵 구성
  - `scan_fn_effect_entry(src, pos)`: fn 뒤에서 `(): <X, Y>` 패턴 파싱
  - `check_callee_missing_effects`: 호출자 효과에 없는 피호출자 효과 검출
  - `lint_check_effect_propagation(entries, eff_map, pos, count)`: 메인 검사 패스
- Stage 1 바이너리 재빌드: `target/release/bmb.exe build bootstrap/compiler.bmb`
- 골든 테스트 추가: `tests/golden/test_golden_effect_propagation.bmb`

## Verification & Defect Resolution
- cargo test 3800+2390+47+22+23 PASS ✅
- 직접 테스트: `process(): <IO>` calls `read_net(): <Net>` → `[effect_propagation]` 경고 ✅
- `send_and_log(): <IO, Net>` calls `fetch_url(): <Net>` → 경고 없음 ✅
- compiler.bmb lint: 177 non-recursive (pre-existing, 새 경고 0개) ✅

## Reflection
- Scope fit: M12 Phase 3 effect propagation 검사 완전 달성
- Latent defects: transitive effect 검사 없음 (직접 호출만) — Phase 3 범위
- Structural improvement: transitive 검사는 Phase 4로 (다중 홉 전파)
- Philosophy drift: Rule 6 완전 준수

## Carry-Forward
- Actionable: M15 Phase 2 (platform capabilities registration)
- Structural Improvement Proposals: M12 Phase 4 (transitive effect propagation)
- Pending Human Decisions: None
- Roadmap Revisions: M12 Phase 3 ✅ COMPLETE
- Next Recommendation: M15 Phase 2
