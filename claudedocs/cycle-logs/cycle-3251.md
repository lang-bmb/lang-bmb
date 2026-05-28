# Cycle 3251: M12 Phase 1 — Effect Row 문법 파싱
Date: 2026-05-29

## Re-plan

**Rule 6 충돌 해소**: HANDOFF의 Rust 파일 참조(grammar.lalrpop, ast/, types/)는 stale — Rule 6("모든 새 기능은 BMB에서 직접 구현")와 충돌. M11-C 선례(모든 stack array 기능을 bootstrap 단독으로 구현)에 따라 bootstrap/compiler.bmb에서만 작업.

Plan: M12 Phase 1 bootstrap 구현 (Rust 파일 변경 없음).

## Scope & Implementation

### 설계 결정

- `fn foo(): <pure> -> T` → `fn-pure` AST 노드 (기존 `@pure fn` 동등)
- `fn foo(): <IO, Net> -> T` → `(fn <foo> (param...) (eff IO Net) ret body)` — `(eff ...)` 자식 노드
- `fn foo(): <*> -> T` → 기존 `fn` 노드 (효과 제한 없음, eff 노드 없음)
- `fn foo() -> T` → 기존 `fn` 노드 (변경 없음)

Phase 1 = 파싱 전용 (효과 검사는 Phase 2에서 추가).

### 핵심 AST 구조

```
(fn <add_io> (param <x> i64) (param <y> i64) (eff IO) i64 body)
```

- `get_fn_return_scan`: `<name>`, `(param...)`, `(eff...)` 모두 skip → `i64` 정확히 추출 ✅
- `get_fn_body_scan`: `param`과 동일하게 `eff` 노드 skip → body만 반환 ✅

### 파일 변경

**bootstrap/compiler.bmb**:

1. **`parse_fn_after_params`** (신규): params 파싱 후 `:`/`->` 분기 처리. parse_fn complexity 감소.
2. **`parse_fn_effect_tail`** (신규): `: <effects> -> ret = body` 처리. `<pure>` → `fn-pure`, 기타 → `fn + (eff ...)`.
3. **`parse_effect_row`** (신규): `<effect1, effect2, ...>` 파싱 wrapper.
4. **`parse_effect_names`** (신규): 재귀 파서 — IDENT, `*`, `>` 처리.
5. **`parse_fn`** 수정: params 후 `parse_fn_after_params` 위임 (complexity 유지).
6. **`get_fn_body_scan`** 수정: `ntype == "eff"` → skip (param과 동일).

**tests/golden/test_golden_effect_row.bmb** (신규): `<pure>`, `<IO>`, `<*>` 3종 테스트.
**tests/golden/test_golden_effect_row.bmb.out** (신규): 기대 출력 `17`.

### 검증

- `double(3): <pure>` → IR: `memory(none) speculatable nofree` (pure fn 속성) ✅
- `add_io(4,5): <IO>` → 정상 컴파일, 실행 ✅
- `ident(2): <*>` → 정상 컴파일, 실행 ✅
- 실행 출력: `17` (= 6+9+2) ✅

## Verification & Defect Resolution

### 발견 및 해결: `[complex]` 경고

- 초기 구현(parse_fn 내 직접 TK_COLON 분기)으로 `parse_fn: 22 calls (max 20)` 경고 발생
- 원인: TK_COLON() + parse_fn_effect_tail = 2 새 unique 함수 호출 추가
- 해결: `parse_fn_after_params` helper로 params 이후 전체 로직 분리 → `parse_fn` 단순화
- 결과: 경고 177개 (이전과 동일, 새 경고 없음) ✅

### 빌드/검증

- Stage 1 (Rust bmb → S1c.exe): ✅
- effect row 테스트 → 17 ✅
- cargo test --release integration: 2390 PASS ✅
- 3-Stage Fixed Point S2c == S3c (0 diff): ✅
- bmb lint: 177 warnings (이전 177과 동일, 새 경고 없음) ✅

## Reflection

**Scope fit**: M12 Phase 1 목표 달성. `fn foo(): <pure/IO/File/Net/Sys/*> -> T` 문법 파싱.

**Latent defects**: Phase 1은 파싱 전용 — 효과 타입 체킹 없음(의도적, Phase 2 범위). `(eff IO Net)` 노드는 현재 lowering에서 무시됨.

**Structural improvements**:
- `parse_fn_after_params` 분리로 `parse_fn`이 더 단순해짐 (향후 parse_fn 확장 시 유리)
- Effect 시스템 AST 표현(`(eff ...)` 노드) Phase 2 type checking 에 자연스럽게 연결됨

**Philosophy drift**: 없음. Rule 6 준수 (bootstrap 전용).

**Roadmap impact**: M12 Phase 1 ✅ COMPLETE. Phase 2 (타입 체커 effect propagation)는 별도 사이클.

## Carry-Forward

- **Actionable**: M12 Phase 2 — 타입 체커에 effect propagation 추가 (callee eff ⊆ caller eff 검증)
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: None
- **Roadmap Revisions**:
  - claudedocs/ROADMAP.md § M12: Phase 1 ✅ COMPLETE 마킹 필요 (다음 사이클에서 일괄)
  - claudedocs/cycle-logs/ROADMAP.md 업데이트 필요
- **Next Recommendation**: M14 Phase 1 (gotgan SHA-256 lockfile) 또는 M12 Phase 2 (effect 타입 체커). M14가 더 독립적이고 범위가 명확.
