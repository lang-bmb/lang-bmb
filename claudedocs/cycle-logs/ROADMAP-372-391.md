# Roadmap: Cycles 372-391

## Theme: Lint Rules + Testing Depth + Compiler Quality

Previous batch (352-371) completed DX improvements, error diagnostics, and method gaps. Audit reveals interpreter is at near-complete parity with type checker (107 string, 54 float, 110 array methods). Focus shifts to new lint rules, comprehensive testing, and compiler robustness.

### Key Gaps Identified
- Only 22 lint rules — room for many more static analysis warnings
- 4 explicit TODOs (closure MIR, checked arithmetic, type deps, proven facts)
- Comprehensive testing can be deepened across all feature areas
- 80 panic! calls in CIR lowering

## Phase 1: New Lint Rules (372-378)
- Cycle 372: Constant condition detection (if true/false, while true/false literals)
- Cycle 373: Self-comparison detection (x == x, x != x always true/false)
- Cycle 374: Redundant boolean comparison (x == true → x, x == false → !x)
- Cycle 375: Duplicate match arm detection
- Cycle 376: Integer division truncation warning
- Cycle 377: Unused function return value detection
- Cycle 378: Lint rule integration tests

## Phase 2: Compiler Robustness (379-383)
- Cycle 379: Replace panic! with Result in CIR lower (batch 1 — pattern matching)
- Cycle 380: Replace panic! with Result in CIR lower (batch 2 — expressions)
- Cycle 381: Replace panic! with Result in CIR lower (batch 3 — remaining)
- Cycle 382: CIR error handling tests
- Cycle 383: Compiler graceful degradation tests

## Phase 3: Testing Depth (384-388)
- Cycle 384: Comprehensive trait + impl tests
- Cycle 385: Ownership + borrowing edge cases
- Cycle 386: Complex generic type tests
- Cycle 387: Concurrency type checking tests
- Cycle 388: Full pipeline regression tests

## Phase 4: Quality Gate (389-391)
- Cycle 389: Code quality sweep + clippy pedantic check
- Cycle 390: Documentation audit + missing test coverage
- Cycle 391: Final quality review + summary
