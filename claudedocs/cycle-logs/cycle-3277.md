# Cycle 3277: M15 Phase 4 — Full Transitive Module Capability Check
Date: 2026-05-29

## Re-plan
M14 Phase 4 (SemanticDuplicate) 재평가 → false positives 위험 높음.
M15 Phase 4 (Full Transitive) 우선 — Cycle 3275에서 발견한 알려진 한계 직접 해결. 계획 변경.

## Scope & Implementation
**목표**: process: <IO> + helper→fetch_data: <Net> → process에서 [module_capability] <Net> 미검출 문제 해결

**추가 함수**:
- `build_full_transitive_pass(entries, old_transmap, pos, acc)` — 모든 함수 확장 (명시 선언도 포함)
- `build_full_transitive_map_iter(entries, transmap, iter)` — 반복 수렴
- `build_full_transitive_effect_map(entries, direct_map, iter)` — 진입점

**lint_file 변경**:
- `let full_trans_map = if module_caps != "" { build_full_transitive_effect_map(entries, eff_map, 5) } else { "" };`
- `let w10 = lint_check_module_capabilities(entries, full_trans_map, module_caps, 0, 0);`
- (module 선언 없으면 full_trans_map 빌드 생략으로 성능 보호)

**결과**:
```
[module_capability] process: uses <Net> not declared in module requires [IO]
[module_capability] main: uses <Net> not declared in module requires [IO]
```
process와 main도 올바르게 경고 발생 ✅

## Verification & Defect Resolution
- cargo test: 8259/0 ✅
- Stage 1 (compiler_3277.exe) ✅
- lint warnings: 178 (변경 없음) ✅

## Reflection
- Scope fit: ✅ M15 Phase 4 알려진 한계 해결
- full_trans_map은 module_caps가 있을 때만 빌드 → 성능 안전

## Carry-Forward
- Actionable: Fixed Point S2==S3 검증 + 최종 커밋 + 메모리 업데이트
- Next Recommendation: Cycle 3278 — Fixed Point + HANDOFF 최종 업데이트
