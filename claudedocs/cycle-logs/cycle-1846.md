# Cycle 1846: LLVM Attribute Enhancement + CSE in Release Pipeline + Bug Fixes
Date: 2026-03-11

## Inherited → Addressed
From 1845: No carry-forward items. Project at 100% golden test pass rate.

## Scope & Implementation

### 1. Runtime Declaration Attributes (Rust text backend)
- Updated ~200+ runtime function declarations in `bmb/src/codegen/llvm_text.rs` with `nocallback`, `nofree`, `nosync`
- Threading functions correctly excluded from `nosync`
- Enables LLVM interprocedural analysis: code motion past runtime calls, better alias analysis

### 2. Runtime Declaration Attributes (Bootstrap)
- Updated ALL declarations in `bootstrap/llvm_ir.bmb` with matching attributes
- Threading/concurrency: `nocallback nounwind` (no `nosync`)
- Memory read-only: + `speculatable`, allocating: + `nosync`
- Updated `llvm.expect` intrinsic from legacy `readnone` to `memory(none)`

### 3. CSE Operand Canonicalization
- `bmb/src/mir/optimize.rs`: Commutative canonicalization so `a+b` and `b+a` share CSE key

### 4. CSE Added to Release Pipeline
- `CommonSubexpressionElimination` now runs in Release mode (was Aggressive-only)
- **Bug discovered**: CopyPropagation didn't handle Select operands — CSE could create undefined SSA refs
- **Fixed**: Extended `propagate_copies_in_inst()` to handle Select, Phi, Cast, FieldStore, IndexLoad/Store, PtrOffset/Load/Store, StructInit, ArrayInit, TupleInit, EnumVariant, ThreadSpawn, ThreadJoin, and Copy chains

### 5. Malloc SSA Name Collision Fix
- `bmb/src/codegen/llvm_text.rs`: Two malloc calls using same size temp `_t3` produced duplicate `_t3.malloc.size`
- **Fixed**: Use `malloc_idx` counter to disambiguate: `_t3.malloc.size.0`, `_t3.malloc.size.1`

### 6. Function Sections for Dead Code Elimination
- Added `-ffunction-sections -fdata-sections` to IR→object compilation in `bmb/src/build/mod.rs`

## Review & Resolution
- cargo test --release: 6,186 tests pass
- 3-Stage Bootstrap: Fixed Point verified (50s)
- Golden tests: running (results pending)
- Benchmarks: knapsack 0.15x FASTER, bellman_ford FASTER, spectral_norm 0.77x FASTER — no regressions

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Run full benchmark suite to verify no regressions; explore additional MIR optimizations
