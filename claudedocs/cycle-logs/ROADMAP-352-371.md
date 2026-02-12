# Roadmap: Cycles 352-371

## Theme: Developer Experience + Error Diagnostics + Method Gaps

Previous batches (292-351) completed comprehensive stdlib method expansion (400+ methods across all types). This batch shifts focus to developer experience, better error messages, and filling remaining type method gaps.

## Phase 1: Error Message & Diagnostic Improvements (352-356)
- Cycle 352: Method-not-found "did you mean?" suggestions for all types
- Cycle 353: Better type mismatch messages with expected/actual context
- Cycle 354: Argument count mismatch improvements (show signature)
- Cycle 355: Chained method error context (show receiver type in chain)
- Cycle 356: Integration tests for error message quality

## Phase 2: Linter Rule Expansion (357-361)
- Cycle 357: Naming convention lint rules (snake_case functions, PascalCase types)
- Cycle 358: Unused parameter detection
- Cycle 359: Single-arm match detection (suggest if-let)
- Cycle 360: Redundant type cast detection
- Cycle 361: Linter dedicated test suite

## Phase 3: Remaining Type Methods (362-367)
- Cycle 362: Tuple methods — len, swap, to_array, contains
- Cycle 363: String regex-like methods — is_match, find_pattern, replace_pattern
- Cycle 364: Array window/slide methods — windows, sliding_pairs, pairwise
- Cycle 365: Integer binary methods — popcount, bit_length, to_binary_string
- Cycle 366: Float formatting methods — to_fixed, to_exponential, to_precision
- Cycle 367: Cross-type method chaining tests

## Phase 4: Quality & Integration (368-371)
- Cycle 368: Comprehensive edge case tests
- Cycle 369: Error recovery stress tests
- Cycle 370: Clippy + code quality sweep
- Cycle 371: Final quality review + summary
