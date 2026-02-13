# Roadmap: Cycles 452-471

> Previous 20 cycles (432-451): Bootstrap 3-stage fixed point + codegen optimization
> This batch: Bootstrap self-hosting advancement — toward Rust dependency elimination
>
> **Key directive from user**: Rust test additions are unnecessary once self-hosting is complete.
> Focus ALL effort on bootstrap/self-hosting development, not Rust-side testing.

## Current State (v0.90.81, Cycle 451)
- Tests: 5,229 ALL PASSING
- Bootstrap: 3-Stage Fixed Point achieved
- All 20 bootstrap .bmb files parse with Stage 1
- 7 bootstrap files compile to native executables
- Performance: bootstrap ~6x slower than Rust (type checker = 99.7%)
- Outstanding: Nullable T?, closure capture native, type checker perf

## Strategy: Self-Hosting Priority

The path to eliminating Rust dependency:
```
Current:  Rust compiler → compiler.bmb → Stage 1 → Stage 2 (fixed point)
Goal:     Golden binary → compiler.bmb → compiler (no Rust needed)
```

Remaining gaps for arbitrary program compilation:
1. **Type checker performance** — 6x slower, need hash-based lookup
2. **while loop codegen** — bootstrap handles recursive style, while loops needed for general programs
3. **Struct/enum codegen** — partially done in bootstrap
4. **BMB-level test suite** — replace Rust tests with BMB self-tests

## Phase A: Type Checker Performance (Cycles 452-457)
Focus: Replace string-based O(n²) lookup with hash-based O(1) lookup in types.bmb

- Cycle 452: Research — analyze env_lookup bottleneck, design hashmap strategy
- Cycle 453: Implement hashmap-based env_lookup in types.bmb
- Cycle 454: Migrate struct/fn/enum/trait registries to hashmap
- Cycle 455: Benchmark — measure improvement, verify fixed point
- Cycle 456: Arena memory optimization — reduce >4GB requirement
- Cycle 457: Performance parity verification — target ≤2x vs Rust

## Phase B: BMB Self-Test Infrastructure (Cycles 458-462)
Focus: Build test runner in BMB that replaces need for Rust tests

- Cycle 458: Design BMB test runner architecture
- Cycle 459: Implement test discovery + execution in BMB
- Cycle 460: Port core compiler tests to BMB self-tests
- Cycle 461: Port optimization tests to BMB self-tests
- Cycle 462: Integration: bootstrap compiles + runs its own tests

## Phase C: Bootstrap Feature Gaps (Cycles 463-467)
Focus: Enable bootstrap to compile more general BMB programs

- Cycle 463: while loop codegen in bootstrap
- Cycle 464: break/continue in bootstrap codegen
- Cycle 465: Struct field assignment codegen
- Cycle 466: Enum variant construction/matching
- Cycle 467: Comprehensive program compilation test

## Phase D: Verification + Golden Binary (Cycles 468-471)
Focus: Verify end-to-end, update golden binary

- Cycle 468: Full 3-stage bootstrap re-verification
- Cycle 469: Performance regression check vs Cycle 446 baseline
- Cycle 470: Golden binary update attempt
- Cycle 471: Final review + summary
