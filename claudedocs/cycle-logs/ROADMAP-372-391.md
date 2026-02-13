# Roadmap: Cycles 372-391

## Theme: Lint Rules + Testing Depth + Compiler Quality

Previous batch (352-371) completed DX improvements, error diagnostics, and method gaps. Audit reveals interpreter is at near-complete parity with type checker (107 string, 54 float, 110 array methods). Focus shifts to new lint rules, comprehensive testing, and compiler robustness.

### Key Gaps Identified
- Only 22 lint rules — room for many more static analysis warnings
- 4 explicit TODOs (closure MIR, checked arithmetic, type deps, proven facts)
- Comprehensive testing can be deepened across all feature areas
- ~~80 panic! calls in CIR lowering~~ **RESOLVED: only 2 production panics, both intentional**

## Phase 1: New Lint Rules (372-378) — COMPLETE
- ✅ Cycle 372: Constant condition detection (if true/false, while true/false literals)
- ✅ Cycle 373: Self-comparison detection (x == x, x != x always true/false)
- ✅ Cycle 374: Redundant boolean comparison (x == true → x, x == false → !x)
- ✅ Cycle 375: Duplicate match arm detection
- ✅ Cycle 376: Integer division truncation warning
- ✅ Cycle 377: Unused function return value detection
- ✅ Cycle 378: Lint rule integration tests

## Phase 2: Testing Depth (379-385) — COMPLETE
> CIR audit showed only 2 production panics (both intentional guards). Phase 2 redirected to testing depth.
- ✅ Cycle 379: Comprehensive trait + impl tests
- ✅ Cycle 380: Ownership + borrowing edge cases
- ✅ Cycle 381: Complex generic type tests
- ✅ Cycle 382: Pattern matching edge cases
- ✅ Cycle 383: Error recovery + edge case tests
- ✅ Cycle 384: Method chaining + type interaction tests
- ✅ Cycle 385: Full pipeline regression tests

## Phase 3: Additional Lint Rules + Quality (386-389) — COMPLETE
> Pivots: Cycle 386 from "unnecessary parentheses" (AST doesn't preserve parens) to identity operation detection. Cycle 387 from "empty block" to negated if-condition. Cycle 388 from "unreachable else-branch" (BMB if-else is expression-based) to absorbing element detection.
- ✅ Cycle 386: Identity operation detection lint (x + 0, x * 1, etc.)
- ✅ Cycle 387: Negated if-condition detection lint (if not x → swap)
- ✅ Cycle 388: Absorbing element detection lint (x * 0, x % 1)
- ✅ Cycle 389: Lint rule comprehensive test suite

## Phase 4: Quality Gate (390-391)
- ✅ Cycle 390: Code quality sweep — DRY refactor of lint detection helpers
- Cycle 391: Final quality review + summary
