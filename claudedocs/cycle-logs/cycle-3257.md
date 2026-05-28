# Cycle 3257: M12 Phase 2c — Effect Pure Violation Lint
Date: 2026-05-29

## Re-plan

Plan valid. M12 Phase 2c: callee eff ⊆ caller eff 검증 (lint rule).

## Scope & Implementation

### 변경

1. `calls_has_io_fn(calls)` — IO 함수 목록 체크 (println/read_file/system 등 14종)
2. `lint_check_effect_violations(entries, pos, count)` — @pure/@const fn이 IO 호출 시 경고
3. `lint_file`: `w7 = lint_check_effect_violations` + `total += w7`

### 경고 규칙

`[effect_pure_violation] fname: pure function calls IO function`

- `@pure fn` 또는 `@const fn` → sig에 `@pure fn` 포함
- 호출 목록에 IO 함수 포함 → 경고

### 테스트

- `@pure fn bad_pure` calling println → `[effect_pure_violation] bad_pure` ✅
- compiler.bmb → 0 `[effect_pure_violation]` 경고 ✅ (기존 @pure fn들 모두 IO 미호출)

## Verification

- Stage 1 빌드: ✅
- lint compiler.bmb: 177 warnings (변화 없음) ✅
- Fixed Point S2 == S3: ✅

## Carry-Forward

- **Actionable**: M12 Phase 3 (Z3 effect constraint) — effect가 pre/post와 연동
- **Next Recommendation**: 남은 사이클 (3258-3260): M15 Phase 1 시작 또는 기존 기능 안정화
