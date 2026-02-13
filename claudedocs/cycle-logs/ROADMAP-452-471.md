# Roadmap: Cycles 452-471

> Previous 20 cycles (432-451): Bootstrap 3-stage fixed point + codegen optimization
> This batch: Bootstrap self-hosting advancement — toward Rust dependency elimination
>
> **Key directive from user**: Rust test additions are unnecessary once self-hosting is complete.
> Focus ALL effort on bootstrap/self-hosting development, not Rust-side testing.

## Current State (v0.90.88, Cycle 459)
- Tests: 5,229 ALL PASSING
- Bootstrap: 3-Stage Fixed Point achieved (68,624 lines)
- Stage 1 `build` command fully self-hosting (S1→S2→S3 fixed point via build)
- Golden tests: 10/10 PASS with automated runner
- All 28 AST node types supported in bootstrap lowering
- Infrastructure: str_hashmap, reg_cached_lookup, system() for opt/clang
- Outstanding: closures, traits/impl, generics, enum variants, async

## Strategy: Self-Hosting Priority

The path to eliminating Rust dependency:
```
Current:  Rust compiler → compiler.bmb → Stage 1 → Stage 2 (fixed point)
Goal:     Golden binary → compiler.bmb → compiler (no Rust needed)
Status:   ✅ Stage 1 `build` command works end-to-end (Cycle 459 verified)
          ✅ Full build chain fixed point: S1→S2→S3 via build command
          → Golden binary update is the remaining step
```

## Phase A: Type Checker Infrastructure (Cycles 452-453) ✅ COMPLETED
- ✅ Cycle 452: str_hashmap runtime + bottleneck analysis
- ✅ Cycle 453: reg_cached_lookup + Rust compiler integration

## Phase B: Bootstrap Feature Completeness (Cycles 454-459) ✅ COMPLETED
- ✅ Cycle 454: break/continue/return codegen fix (inline concat extraction)
- ✅ Cycle 455: for-loop continue fix + golden test expansion (for-in, loop)
- ✅ Cycle 456: match expression support (desugar to if-else chains)
- ✅ Cycle 457: struct init support (calloc + field-store)
- ✅ Cycle 458: integration golden test (struct+match+for+if composition)
- ✅ Cycle 459: automated golden test runner + build chain verification

## Phase C: Bootstrap Compiler Advancement (Cycles 460-465)
Focus: Expand bootstrap capabilities, improve reliability, prepare for golden binary update

- Cycle 460: Roadmap update + bootstrap error resilience (error recovery, diagnostics)
- Cycle 461: Expand golden test coverage (recursive algorithms, complex control flow)
- Cycle 462: Bootstrap performance analysis (identify bottlenecks vs Rust)
- Cycle 463: Optimize critical compilation paths in compiler.bmb
- Cycle 464: Cross-platform build verification (Linux/macOS build chain)
- Cycle 465: Bootstrap self-test expansion (add golden tests to bootstrap/tests/)

## Phase D: Golden Binary Preparation (Cycles 466-469)
Focus: Prepare and verify a new golden binary from bootstrap

- Cycle 466: Golden binary candidate generation from Stage 1
- Cycle 467: Comprehensive golden binary validation
- Cycle 468: Golden binary CI workflow integration
- Cycle 469: Golden binary finalization + documentation

## Phase E: Final Verification + Summary (Cycles 470-471)
Focus: End-to-end verification and session summary

- Cycle 470: Full regression check, all tests, all golden tests
- Cycle 471: Session summary, roadmap for next batch
