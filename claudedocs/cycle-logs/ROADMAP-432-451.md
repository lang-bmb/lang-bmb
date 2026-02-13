# Roadmap: Cycles 432-451

> Previous 40 cycles (392-431): Test coverage expansion (4562 → 5172 tests)
> This batch: Feature development toward v0.92-v0.93 (bootstrap completion + optimization)

## Current State (v0.90.61, Cycle 431)
- Tests: 5172 ALL PASSING
- i32: ✅ Already implemented in Rust compiler (lexer, parser, types, MIR, codegen)
- Open issues: 1 (phi-type-inference)
- Bootstrap: 3-Stage Fixed Point, but lacks nullable T? + closure capture for arbitrary programs

## Phase A: Bug Fixes + Integration Tests (Cycles 432-435)
Focus: Fix the last open issue, add integration tests for i32 type, verify benchmarks

- Cycle 432: Fix phi type inference issue (ISSUE-20260209-phi-type-inference.md)
- Cycle 433: i32 integration tests — verify i32 works end-to-end through type checker
- Cycle 434: i32 MIR + codegen integration tests — verify i32 LLVM IR generation
- Cycle 435: Benchmark verification — run benchmarks with i32, measure improvement

## Phase B: v0.92 Bootstrap Completion (Cycles 436-441)
Focus: Enable bootstrap compiler to compile arbitrary BMB programs

- Cycle 436: Nullable T? — analysis of what's needed in bootstrap for nullable types
- Cycle 437: Nullable T? — bootstrap lexer/parser support
- Cycle 438: Nullable T? — bootstrap type checking + MIR lowering
- Cycle 439: Closure capture — analysis + bootstrap capture analysis
- Cycle 440: Closure capture — bootstrap codegen for captured variables
- Cycle 441: Bootstrap arbitrary program test suite — verify with real BMB programs

## Phase C: v0.93 Bootstrap Codegen Optimization (Cycles 442-447)
Focus: Improve code quality of bootstrap-generated LLVM IR

- Cycle 442: LLVM function attributes (memory(none), willreturn, norecurse)
- Cycle 443: byte_at inlining (runtime call → GEP+load)
- Cycle 444: Identity copy removal (add nsw i64 0, X patterns)
- Cycle 445: select direct generation for simple if/else
- Cycle 446: Dominator tree CSE (cross-block common subexpression elimination)
- Cycle 447: Copy propagation completion (optimize.bmb TODO)

## Phase D: Verification + Review (Cycles 448-451)
Focus: Performance verification, integration testing, documentation

- Cycle 448: Bootstrap performance benchmarking (target: ≤1.10x vs Rust)
- Cycle 449: 4-Stage bootstrap verification attempt
- Cycle 450: Integration test sweep for all new features
- Cycle 451: Final review + summary
