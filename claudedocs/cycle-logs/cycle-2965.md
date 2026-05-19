# Cycle 2965: &&/|| Short-Circuit MIR Lowering + Test Fix
Date: 2026-05-19

## Re-plan
Cycle 2964 Carry-Forward: `&&`/`||` BMB 언어 지원 구현 (parser → AST → types → codegen → interp → bootstrap).
Cycle 2964에서 B축 3문제 problem.md 수정 완료. 이번 사이클에서 근본 해결: short-circuit lowering 구현.

## Scope & Implementation

### &&/|| Short-Circuit MIR Lowering (`bmb/src/mir/lower.rs`)
`Expr::Binary` match arm 상단에 `BinOp::And` / `BinOp::Or` 특수 케이스 추가.
Phi 노드 기반 short-circuit (같은 패턴의 `Expr::If` lowering):

- `a && b` → `if a { b } else { false }` (phi 노드)
- `a || b` → `if a { true } else { b }` (phi 노드)

생성되는 LLVM IR 구조:
```
; a && b
%lhs = ...
br i1 %lhs, label %and_rhs, label %and_false
and_rhs:  %rhs = ...; br label %and_merge
and_false: br label %and_merge
and_merge: %result = phi i1 [ %rhs, %and_rhs ], [ false, %and_false ]
```

### Test Fix (`bmb/tests/integration.rs`)
`test_ir_boolean_logic` 테스트가 기존 `source_to_ir_unopt` + `" and "` 검사로 실패.
Short-circuit은 phi 노드를 생성하므로 검사 방식 변경:
- `source_to_ir_unopt` 유지
- `assert!(ir.contains(" and "))` 제거 → `assert!(ir.contains("phi i1"))` 추가
- `assert!(ir.contains(" or "))` 제거 (phi i1로 통합 검사)
- `bxor` → `xor i64` 검사는 유지

## Verification & Defect Resolution

- `test_ir_boolean_logic`: ✅ 통과 (phi i1 패턴 확인)
- `cargo test --release`: **전체 통과, 0 실패** ✅

## Reflection

- Short-circuit lowering은 `Expr::If`와 동일한 MIR 패턴 — 언어 일관성 유지
- LLVM `-O2`는 순수 인수(`a and b`)에 대해 phi → `and i1` 접기를 수행하지 않음 → LLVM 정책 확인
- `bxor`은 여전히 직접 `xor i64` — bitwise op는 short-circuit 불필요
- B축 86_heap_sort의 근본 원인(OOB 메모리 오염) 해소

## Carry-Forward
- Actionable: Bootstrap compiler `and`/`or` codegen 검증 (bootstrap/compiler.bmb)
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정 (수정 반영 확인)
- Roadmap Revisions: None
- Next Recommendation: Bootstrap Stage 1 검증 + 추가 언어 갭 해소
