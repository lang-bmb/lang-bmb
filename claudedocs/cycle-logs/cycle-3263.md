# Cycle 3263: M15 Phase 2 — Platform Capabilities 등록
Date: 2026-05-29

## Re-plan
Carry-Forward에서 M15 Phase 2. Platform 블록 함수 선언 effect 등록.

## Scope & Implementation
- `bootstrap/compiler.bmb`: M15 Phase 2 구현
  - `scan_platform_effects(src, pos, acc)`: platform { } 내부 fn 선언 스캔
  - Platform 함수는 "PLAT:" prefix로 표시 → callee 조회에 활용하되 본인 검사 제외
  - `build_fn_effect_map`에 platform 블록 감지 추가 (TK_IDENT + "platform")
  - `eff_map_get_raw(map, name, pos)`: raw (PLAT: prefix 포함) 조회
  - `lint_check_effect_propagation`: `is_platform` 검사로 platform 선언 건너뜀

주요 버그 발견 및 수정:
- `callers_collect_source`가 platform 내부 fn을 일반 함수처럼 처리 → "calls" 필드에 인접 함수 포함
- 해결: platform 선언에 "PLAT:" 마커 → `lint_check_effect_propagation`에서 건너뜀

## Verification & Defect Resolution
- cargo test 3800+2390+47+22+23 PASS ✅
- platform 블록 내 Net 함수를 IO만 선언한 일반 함수가 호출 → 경고 ✅
- platform 선언 자체는 검사 대상에서 제외 ✅
- compiler.bmb lint: 177 non-recursive (pre-existing, 새 경고 0개) ✅

## Reflection
- Scope fit: M15 Phase 2 완전 달성
- Latent defects: callers_collect_source가 platform 블록을 인식하지 못함 → platform 함수의 "calls" 부정확. 해결책: "PLAT:" 마커로 충분히 처리됨.
- Philosophy drift: Rule 6 준수

## Carry-Forward
- Actionable: M13 Phase 3 (structured repair signal JSON)
- Structural Improvement Proposals: callers_collect_source가 platform 블록을 skip하도록 수정
- Pending Human Decisions: None
- Roadmap Revisions: M15 Phase 2 ✅ COMPLETE
- Next Recommendation: M13 Phase 3
