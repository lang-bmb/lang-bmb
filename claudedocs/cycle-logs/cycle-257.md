# Cycle 257: Nullable T? MIR Lowering — Research & Design

## Date
2026-02-12

## Scope
Research current T? support across all compiler layers and design the MIR lowering strategy for proper nullable type representation.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary

### Current Status by Layer

| Layer | Status | Notes |
|-------|--------|-------|
| Parser | ✅ Complete | `T?` syntax for all types, `null` literal |
| AST | ✅ Complete | `Type::Nullable(Box<Type>)`, `Expr::Null` |
| Type Checker | ✅ Mostly | Auto-wrapping, unification, method checking |
| Interpreter | ⚠️ Partial | `null = 0`, nullable methods on Int |
| MIR Lowering | ❌ Broken | `Type::Nullable(T)` → `MirType::T` (strips nullable) |
| Codegen | ⚠️ N/A | No nullable-specific handling (relies on MIR) |

### Critical Gap: MIR Lowering (lower.rs:2720-2721)
```rust
Type::Nullable(inner) => ast_type_to_mir(inner), // Discards nullable info!
```
- Null literal: `Expr::Null => Constant::Int(0)` (just integer 0)
- Methods: Hardcoded `is_some()` = `value != 0`, `is_none()` = `value == 0`
- Problem: `Some(0)` and `None` are indistinguishable for `i64?`

### Design Decision: Desugar T? to Option<T> Enum

Per specification, `T?` is syntactic sugar for `Option<T>`. The proper approach:

1. **MIR Type**: `Nullable(T)` → `MirType::Enum { name: "Option", variants: [("Some", [T]), ("None", [])] }`
2. **Null Literal**: `Expr::Null` → `EnumVariant { enum_name: "Option", variant: "None", args: [] }`
3. **Value Wrapping**: `T` assigned to `T?` → `EnumVariant { ..., variant: "Some", args: [value] }`
4. **Methods**: Replace hardcoded methods with enum discriminant checks

### Performance Considerations
- Tagged union: 2 words (discriminant + value) vs 1 word for null-as-0
- Contract-based optimization should eliminate tag checks (`pre x != null` → `NonNull` fact)
- Null pointer optimization possible for `*T?` (pointer types)

### Estimated Effort: 4 cycles
- Cycle A: MIR type lowering + null literal
- Cycle B: Value wrapping (auto T→T? conversion)
- Cycle C: Method call lowering
- Cycle D: Codegen verification + tests

### Files to Modify
- `bmb/src/mir/lower.rs` (critical: type lowering, null literal, methods)
- `bmb/src/mir/mod.rs` (optional: MirType alias)
- `bmb/src/codegen/llvm.rs` (verify enum codegen)
- `bmb/src/interp/eval.rs` (remove hardcoded nullable methods)
- Integration tests

## Implementation
Research-only cycle. No code changes.

## Test Results
- N/A (research cycle)

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | N/A | Research cycle |
| Architecture | 10/10 | Comprehensive multi-layer analysis |
| Philosophy Alignment | 10/10 | Identifies root cause at correct level (MIR) |
| Test Quality | N/A | Research cycle |
| Code Quality | N/A | Research cycle |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | MIR lowering discards Nullable info | Implementation needed (4 cycles) |
| I-02 | M | null=0 ambiguity for i64? | Fix with tagged union |
| I-03 | L | Bootstrap doesn't support T? | Roadmap v0.92 item |

## Next Cycle Recommendation
- Defer full Nullable implementation to dedicated cycle batch
- Proceed with WASM backend improvements or checked arithmetic
- Or: Additional compiler quality improvements (error messages, diagnostics)
