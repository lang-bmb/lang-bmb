# Roadmap: Cycles 452-471

> Previous 20 cycles (432-451): Bootstrap 3-stage fixed point + codegen optimization
> This batch: Bootstrap self-hosting advancement — toward Rust dependency elimination
>
> **Key directive from user**: Rust test additions are unnecessary once self-hosting is complete.
> Focus ALL effort on bootstrap/self-hosting development, not Rust-side testing.

## Current State (v0.90.83, Cycle 453)
- Tests: 5,229 ALL PASSING
- Bootstrap: 3-Stage Fixed Point achieved (67,142 lines)
- All 20 bootstrap .bmb files parse with Stage 1
- 7 bootstrap files compile to native executables
- Performance: bootstrap ~6x slower than Rust
- Infrastructure: str_hashmap + reg_cached_lookup in C runtime
- Outstanding: while loops, struct assignment, enum variants, closure capture

## Strategy: Self-Hosting Priority

The path to eliminating Rust dependency:
```
Current:  Rust compiler → compiler.bmb → Stage 1 → Stage 2 (fixed point)
Goal:     Golden binary → compiler.bmb → compiler (no Rust needed)
```

### Key Architectural Insight (Cycle 453)
- `compiler.bmb` does parsing + lowering + codegen (NO type checking)
- `types.bmb` is a standalone type checker, not integrated into the bootstrap pipeline
- Performance bottleneck is in the Rust compiler's type checker, not the bootstrap
- Phase A (type checker perf) reprioritized: infrastructure built but deferred until types.bmb integration

## Phase A: Type Checker Infrastructure (Cycles 452-453) ✅ COMPLETED
- ✅ Cycle 452: str_hashmap runtime + bottleneck analysis
- ✅ Cycle 453: reg_cached_lookup + Rust compiler integration
- Remaining type checker optimization deferred to future types.bmb integration

## Phase B: Bootstrap Feature Gaps (Cycles 454-462)
Focus: Enable bootstrap to compile more general BMB programs

- Cycle 454: while loop codegen in bootstrap
- Cycle 455: break/continue in bootstrap codegen
- Cycle 456: Struct field assignment codegen
- Cycle 457: Enum variant construction/matching
- Cycle 458: Trait/impl dispatch codegen
- Cycle 459: Nullable T? codegen (match Some/None)
- Cycle 460: Closure capture in native codegen
- Cycle 461: for-in loop codegen
- Cycle 462: Comprehensive program compilation test

## Phase C: BMB Self-Test Infrastructure (Cycles 463-467)
Focus: Build test runner in BMB that replaces need for Rust tests

- Cycle 463: Design BMB test runner architecture
- Cycle 464: Implement test discovery + execution in BMB
- Cycle 465: Port core compiler tests to BMB self-tests
- Cycle 466: Port optimization tests to BMB self-tests
- Cycle 467: Integration: bootstrap compiles + runs its own tests

## Phase D: Verification + Golden Binary (Cycles 468-471)
Focus: Verify end-to-end, update golden binary

- Cycle 468: Full 3-stage bootstrap re-verification
- Cycle 469: Performance regression check vs Cycle 446 baseline
- Cycle 470: Golden binary update attempt
- Cycle 471: Final review + summary
