# Cycle 3307: P4 — .bmb-contracts max_params 구현
Date: 2026-05-29

## Re-plan
HANDOFF P4 그대로. bc_check_max_params_scan 구현.

## Scope & Implementation
- `count_top_commas(s, pos, depth)`: `<>` depth-aware 콤마 카운터
- `count_sig_params(sig)`: sig 문자열에서 파라미터 수 추출
- `bc_check_max_params_scan(entries, max_n, sb, isfirst, pos)`: entries 스캔하여 max_n 초과 함수 위반 추가
- `cc_build_json`: `max_params_str` 파싱 + f5 체크 통합 + `has_viol = f5 == 0`

## Verification & Defect Resolution
- cargo test: 3800+47+22+2390+23 PASS (0 FAILED)
- Stage 1 재빌드 필요 (P4 변경 반영): 27초 컴파일 + 12초 링크
- max_params=3 테스트: `too_many_params(5 params)` → violation 정확히 감지
- `ok_fn(2 params)`: 위반 없음 ✅

## Reflection
- 초기 테스트 실패 원인: `compiler_s1.exe`가 P4 변경 전 빌드된 것
- 재빌드 후 즉시 정상 동작
- depth-aware 콤마 카운팅 → `Array<K, V>` 타입도 올바르게 처리

## Carry-Forward
- Actionable: Fixed Point 검증 후 커밋, P2 M12 Z3 lattice 또는 P3 cross-gen FP
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3308 — within-gen Fixed Point 검증 + 커밋
