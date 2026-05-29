# Cycle 3286: sim_count_shared 버그 수정 + semdp 통합
Date: 2026-05-29

## Re-plan
Carry-Forward: sim_count_shared P1 버그. 계획 유효.

## Scope & Implementation
**근본 원인 분석**:
- `sim_find_start_rev`에 `pre pos >= 0` 계약 → bootstrap LLVM이 `if pos < 0 { 0 }` 분기를 dead code로 제거
- `sim_find_start`가 `sim_find_start_rev(s, pos - 1)`을 호출 시 pos = 0이면 pos-1 = -1 → `byte_at(-1)` UB
- 결과: 첫 번째 call name 추출 실패 → N개 call 중 N-1 match 반환

**수정**:
- `sim_find_start_rev`: `if pos < 0 { 0 }` → `if pos <= 0 { 0 }`
  - `pos <= 0`은 LLVM에서 `pos >= 0` (pre-condition)에 의해 `pos == 0`으로 최적화
  - pos == 0 시 즉시 0 반환 → UB 방지

**통합**:
- `semdp_count_shared` + `semdp_name_start` 제거 (sim_count_shared가 고쳐졌으므로 중복)
- `semdp_check_pair`에서 `semdp_count_shared` → `sim_count_shared`로 교체

**검증 결과**:
- 1-call: [1 shared] ✅ (기존 segfault)
- 2-call: [2 shared] ✅ (기존 [1 shared])
- 3-call: [3 shared] ✅ (기존 [2 shared])
- cargo test: 3800+2390+23 PASS ✅
- Within-generation Fixed Point: S3==S3b (deterministic) ✅

## Verification & Defect Resolution
- All tests pass
- Cross-generation IR diff (18446744073709551615 vs -1): 기존 알려진 표현 차이, 기능 동일

## Reflection
- **Scope fit**: P1 버그 완전 해결 + 코드 정리 (semdp 제거)
- **Metacircular contract**: pre pos >= 0 + if pos < 0 패턴이 부트스트랩에서 dead code로 제거됨. 이는 CLAUDE.md 패턴 "post it >= 0"과 동일한 메카니즘.
- **Philosophy**: workaround(semdp 우회) 제거, 근본 수정.
- **Roadmap impact**: 없음. Fixed Point 유지.

## Carry-Forward
- Actionable: M12 Z3 더 깊은 통합 (Phase 6b: @pure fn violation Z3)
- Structural Improvement Proposals: sim_find_start_rev의 pre를 `post it >= 0 and it >= -1` 로 정정 가능 (저우선순위)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: M12 Phase 6b (@pure fn violation Z3 verification)
