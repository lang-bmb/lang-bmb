# Cycle 1839: Fix speculatable on Non-Leaf Functions
Date: 2026-03-11

## Inherited → Addressed
From 1838: "Run full golden test suite to measure how many of the 39 failures are resolved" + speculatable bug on recursive memory(none) functions.

## Scope & Implementation

### Root Cause
The `spec_rebuild()` function in `compiler.bmb` searched for `"call @"` to detect non-leaf functions, but LLVM IR format is `call i64 @func(...)` — the return type appears between `call` and `@`. The pattern never matched, so ALL memory(none) functions got `speculatable`, including recursive ones.

For recursive `memory(none)` functions like `lps_simple`, LLVM's `select` transformation would speculate the recursive call past the base case check, creating infinite recursion at runtime.

### Fix
Changed search pattern from `"call @"` to `" call "` in `spec_rebuild()`:
```
let has_call = find_pattern_noa_range(fn_body, " call ", 0, fn_body.len()) >= 0;
```

This correctly matches `call i64 @...`, `tail call i64 @...`, `call void @...` etc.

### Files Changed
- `bootstrap/compiler.bmb` — Fixed pattern in `spec_rebuild()` (line 8068)

### Verification
- **Golden tests**: 2782 → 2794 PASS (39 → 27 FAIL, +12 tests fixed)
- **Fixed tests**: lps_length, ackermann, scc_kosaraju, tree_diameter, lcs_three, coin_change, matrix_chain, rod_cutting, tower_hanoi, nqueens, power_set, and more
- **Remaining 27 failures**: 18 compile (closures/generics unsupported), 6 file not found, 3 opt (missing runtime declarations)
- **Zero wrong-output or runtime failures**
- **Rust tests**: 6,186 pass (no regression)
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (S2 == S3, 53s)

## Review & Resolution
- The pattern bug was subtle: `"call @"` looks correct for LLVM IR but misses the mandatory return type token
- Combined with Cycle 1838's TRL fix, all 12 runtime-failure golden tests are now resolved
- ackermann (previously thought to be stack overflow) was actually a speculatable infinite recursion
- All remaining failures are pre-existing infrastructure issues (unsupported features, missing files/declarations)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: 3 opt failures from missing runtime declarations (bmb_clamp, int methods, method_chain) — bootstrap doesn't declare these newer builtins
- Next Recommendation: Run benchmarks to verify speculatable removal from non-leaf functions doesn't cause performance regression
