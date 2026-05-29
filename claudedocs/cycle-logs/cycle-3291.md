# Cycle 3291: M12 Phase 6d — @pure fn Z3 UNSAT 공식 검증
Date: 2026-05-29

## Re-plan
Effect lattice 모델링 → 구체적으로 @pure fn을 Z3 SMT 모델에 추가하여 UNSAT 검증.

## Scope & Implementation
**문제**: Cycle 3287에서 @pure violation은 heuristic scan으로만 탐지, Z3는 "sat" 반환.

**신규 함수**:
1. `eff_z3_gen_pure_decls(entries, eff_map, pos, sb) -> i64`
   - entries에서 @pure/@const fn 식별
   - effectful callee가 있는 경우만 처리 (via eff_pure_has_eff_callee)
   - fn_io=false, fn_net=false 등 모든 효과 false로 선언
   - `eff_z3_gen_pure_edges` 호출

2. `eff_z3_gen_pure_edge(caller, callee, eff_map, sb) -> i64`
   - caller_eff 체크 없이 직접 implication 생성
   - `(assert (=> callee_io caller_io))` + 나머지 3종
   - caller_io=false, callee_io=true → (=> true false) = false → UNSAT

3. `eff_z3_gen_pure_edges(caller, calls, eff_map, pos, sb) -> i64`
   - calls를 순회하며 각 callee에 pure_edge 생성

4. `eff_pure_has_eff_callee(calls, eff_map, pos) -> bool`
   - calls 중 eff_map에 있는 callee 존재 여부 (최적화: 없으면 선언 불필요)

**수정**:
- `eff_z3_gen_smt`: `eff_z3_gen_pure_decls` 추가 (선언 후 엣지 전)

**버그 발견/수정**:
- 초기 구현에서 `eff_z3_gen_calls` 재사용 시도 → caller_eff=="" 체크로 skip됨
- 전용 `eff_z3_gen_pure_edges` 구현으로 해결

**검증**:
- @pure bad_fn calls io_fn → z3:"unsat" ✅ (기존 z3:"sat")
- @pure good_fn calls plain fn → z3:"sat" ✅
- cargo test 3800+2390+23 PASS ✅

## Verification & Defect Resolution
모든 테스트 통과.

## Reflection
- **Scope fit**: M12 Phase 6d 완성. @pure violation이 이제 완전히 Z3 UNSAT로 형식화.
- **Effect lattice**: @pure = 빈 effect set 모델링 완성. IO/Net/File/Sys는 독립 boolean flag.
- **Improvement**: pure fn의 Z3 검증이 heuristic scan보다 더 강함.

## Carry-Forward
- Actionable: git commit (10 사이클 완료 전 커밋)
- Structural Improvement Proposals: missing_annotation도 Z3에 추가 (향후)
- Pending Human Decisions: 없음
- Roadmap Revisions: M12 Phase 6d 추가 필요
- Next Recommendation: 최종 커밋 + HANDOFF/ROADMAP 갱신
