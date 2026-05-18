# Cycle 2937: `break <expr>` Native Codegen MIR 지원
Date: 2026-05-19

## Re-plan
Cycle 2936 Carry-Forward: Native codegen에서 `break <value>` 값이 무시됨 (MIR 미지원). 이번 사이클에서 즉시 수정.

## Scope & Implementation

### 문제 분석

Cycle 2936에서 `break <value>` grammar 추가 → 인터프리터 정상 동작, native codegen 오작동.

Native 빌드 결과:
```
first_square_gt(20) → 0  (expected: 5)
find_divisor(35)    → 0  (expected: 5)
find_divisor(37)    → 0  (expected: -1)
```

원인: `Expr::Break { value: Some(v) }` MIR 처리:
```rust
// Before (lower.rs:2964)
if let Some(v) = value {
    let _ = lower_expr(v, ctx);  // 값 평가 후 즉시 버림
}
```

And `Expr::Loop { body }` returns `Operand::Constant(Constant::Unit)` — break value 전혀 전달 안 됨.

### 수정 전략: Result Slot Pattern

PHI 노드 방식 대신 mutable slot 방식 선택:
1. `loop` 진입 전: `%loop_result_N = 0` (i64 temp local 초기화)
2. `break <value>`: value 평가 → slot에 저장 → exit label로 goto
3. `break` (no value): slot 초기화값(0) 유지 → exit label로 goto
4. exit block: `Operand::Place(result_slot)` 반환

이 방식의 장점:
- 단순한 SSA 확장 (PHI 노드 불필요)
- `break` (no value)도 slot 0 유지로 자연 처리
- while/for 루프는 result slot 없음 (None) → 기존 동작 유지

### 변경 파일

**`bmb/src/mir/mod.rs`**:
```rust
// Before:
pub loop_context_stack: Vec<(String, String)>,

// After:
pub loop_context_stack: Vec<(String, String, Option<String>)>,
//  (continue_label, break_label, result_slot_name)
```

**`bmb/src/mir/lower.rs`**:

1. `Expr::Loop`: result slot 할당 + 스택에 슬롯 포함 + exit에서 슬롯 반환
2. `Expr::Break { value }`: value → slot 저장 후 goto
3. `while-let` 및 `while` 루프: `push(..., None)` (result slot 없음)
4. `Expr::Continue`: 패턴 매칭 `(cond, _, _)` 업데이트

## Verification & Defect Resolution

### 인터프리터 + Native 비교

| 케이스 | 인터프리터 | Native (before) | Native (after) |
|--------|-----------|-----------------|----------------|
| `first_square_gt(20)` | 5 | 0 | 5 ✅ |
| `find_in_range(1,20,49)` | 7 | 0 | 7 ✅ |
| `find_in_range(1,5,999)` | -1 | 0 | -1 ✅ |
| `break` (no value) | unit | unit | unit ✅ |
| `while` loop | 10 | 10 | 10 ✅ |

### 테스트 결과

```
cargo test --release -p bmb: 2388 passed, 0 failed ✅
골든 테스트: break_with_value.bmb (interpreter + native 모두 통과)
```

## Reflection

### Scope fit
- ✅ Native codegen `break <value>` 완전 지원
- ✅ while/for 루프 기존 동작 유지
- ✅ 2388 테스트 통과

### 설계 의의
Result slot 패턴은 SSA를 엄격히 따르지 않고 mutable local을 사용하지만, LLVM IR 레벨에서 이를 `alloca → store → load` 로 자연스럽게 처리함. `loop { break 42 }` 가 native에서 42를 반환하는 것이 확인됨.

### `break <value>` 기능 완성 상태

| 단계 | 상태 |
|------|------|
| Parser (grammar.lalrpop) | ✅ Cycle 2936 |
| 인터프리터 | ✅ 기존 지원 |
| MIR lowering | ✅ Cycle 2937 |
| LLVM IR (native codegen) | ✅ Cycle 2937 (result slot → alloca) |
| 타입 체커 (loop 반환 타입 추론) | ⚠️ 현재 Never → 실용적 영향 낮음 |

## Carry-Forward

- Actionable: 없음 (break <value> 완전 구현됨)
- Structural Improvement Proposals:
  1. **Loop 타입 추론 개선**: `loop { break i: i64 }` → 타입 체커가 `i64`로 추론 (현재 `Never`). 실용적 영향 낮음 — loop 결과를 `let x = loop { ... }` 패턴으로 사용 시 별도 타입 annotation 불필요 (인터프리터/codegen 모두 값 정확히 반환).
  2. **Closure HOF unified representation** (Cycle 2935 유지)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2938 — 추가 언어 갭 또는 B축 개선
