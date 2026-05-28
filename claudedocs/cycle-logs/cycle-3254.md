# Cycle 3254: M12 Phase 2a — Effect Row MIR 전파 + 인프라
Date: 2026-05-29

## Re-plan

Plan valid. M12 Phase 2 (effect propagation) 시작. Phase 2 전체는 3-4 사이클이므로, Cycle 3254에서 인프라 구축.

## Scope & Implementation

### 설계

Phase 2a = effect row를 MIR 함수 헤더에 `@effect:IO Net` 형식으로 포함.
Phase 2b = 타입 체커에서 callee eff ⊆ caller eff 검증 (별도 사이클).

**동작 방식**:
- `fn add_io(x: i64, y: i64): <IO> -> i64` → MIR: `fn add_io(...) -> i64 @effect:IO {...}`
- LLVM codegen: `find_mir_annotation`이 `@effect`를 인식하지 못하므로 무시 (LLVM 속성 없음)
- MIR에는 보존되어 있으므로 미래 분석 패스에서 사용 가능

### 변경

**bootstrap/compiler.bmb**:
1. `get_fn_effect_row(ast)` (신규) — fn AST에서 `(eff ...)` 노드 추출 → effect names string
2. `get_fn_eff_scan(content, pos)` (신규) — paren content 스캔으로 eff 노드 탐지
3. `lower_function_sb` 수정 — eff_row 추출 + `@effect:eff_row` MIR 어노테이션 추가
4. `@exec_with_stdin` LLVM extern declare 추가 (Cycle 3253에서 필요 발견)

### Effect → MIR 매핑

```
fn foo(x: i64): <IO, Net> -> i64 = body
  → fn-AST: (fn <foo> (param <x> i64) (eff IO Net) i64 body)
  → MIR: fn foo(x: i64) -> i64 @effect:IO Net { ... }
  → LLVM IR: define private noundef i64 @foo(...) { ... }  (effect 무시, 안전)
```

## Verification & Defect Resolution

- Stage 1 빌드: ✅
- effect row 테스트 17 ✅ (기능 정상)
- Fixed Point S2 == S3: ✅
- bmb lint: 177 warnings (변화 없음) ✅

## Reflection

**Scope fit**: Phase 2a 인프라 완성. effect row가 MIR에 보존됨.

**Latent defects**: LLVM에서 `@effect`가 무시됨 (의도적, Phase 2b에서 속성 매핑 추가).

**Roadmap impact**: M12 Phase 2a ✅. Phase 2b (callee eff ⊆ caller eff 검증)은 별도.

## Carry-Forward

- **Actionable**: M12 Phase 2b — LLVM에 `@effect` 속성 매핑 (e.g., `@effect:IO Net` → `"effect"="IO Net"` LLVM string attr)
- **Structural Improvement Proposals**: 
  - compiler.bmb 함수들에 effect 어노테이션 추가 (dogging M12)
  - `fn_has_effect_annotation(ast)` → M13 Phase 3에서 유용
- **Pending Human Decisions**: None
- **Roadmap Revisions**: M12 Phase 2a ✅
- **Next Recommendation**: M12 Phase 2b 또는 M14 Phase 2 (gotgan build --locked 검증)
