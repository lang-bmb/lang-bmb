# Cycle 3231: IPR Structural Fix — Comma-Bounded Substring Matching
Date: 2026-05-28

## Re-plan
Plan valid. Carry-forward from Cycle 3230: implement structural fix to IPR substring-match false positive.
Change `let key = fn_name + ","` → `let key = "," + fn_name + ","` in both `ipr_all_calls_pure` and
`ipr_all_calls_readonly` to prevent any function name from matching as a suffix of a longer name.

## Scope & Implementation

### Root Cause (recap)
`ipr_try_annotate_section` prepends `self_name` to `known_self` BEFORE checking callees:
```bmb
let known_self = if self_name != "" { known + self_name + "," } else { known };
```
`known_self` = `"...,array_free,"`. When checking callee `fn_name="free"`, old key `"free,"` matches as
suffix of `"array_free,"` → false positive. Fixed in Cycle 3229 with a blacklist for `free`/`realloc`.

### Structural Fix (Cycle 3231)
Changed key format in **two places** in `bootstrap/compiler.bmb`:

**`ipr_all_calls_pure` (~line 17327)**:
```bmb
// Old (buggy):
let key = fn_name + ",";

// New (correct):
// Use comma-bounded matching: ",fn_name," to prevent substring false positives
// e.g. "free" must NOT match inside "array_free" in the known set
let key = "," + fn_name + ",";
```

**`ipr_all_calls_readonly` (~line 17393)**:
```bmb
// Old (buggy):
let key = fn_name + ",";

// New (correct):
// Use comma-bounded matching: ",fn_name," to prevent substring false positives
let key = "," + fn_name + ",";
```

**Why this works**: `known` and `known_self` are comma-prefixed lists: `",fn1,fn2,...,"`.
Searching `",free,"` in `"...,array_free,"` returns NOT FOUND (because `_free` precedes `free`, not `,`).
With old key `"free,"`, it matched as suffix of `"array_free,"` — this is prevented by the new format.

### Stage 1 Build
- `compiler_3231_rust_s1.exe` (from Rust compiler): 1,500,069 bytes, v0.96.30, 36.8s ✅

### IPR Verification
- `emit-ir main_inproc.bmb`: `array_free` has NO `memory(read)` attribute ✅
- `@free()` call IS present in `array_free` body ✅

### Stage 2 Build
- `compiler_3231_s2.exe` (from Rust S1): 1,198,734 bytes, v0.96.30, 32.7s ✅

### Fixed Point Verification
- S3 IR: 6202 lines | S4 IR: 6202 lines | **Differences: 0** ✅
- Fixed Point confirmed: S3 == S4

### Canonical Binary Update
- `bootstrap/compiler.exe` → `compiler_3231_s2.exe` (1,198,734 bytes)

### Tests
- `cargo test --release`: **6282 tests, 0 FAILED** ✅

## Verification & Defect Resolution

### Latent Defect Discovered (Pre-existing, Not Introduced by Cycle 3231)
**S2 bootstrap IR truncation for recursive functions with certain patterns**:

`compiler_3231_s2.exe` (and also `compiler_3229_s2.exe`) generates TRUNCATED function bodies for
`merge_sort_helper` and `quick_sort_helper` in the sorting benchmark:

```ll
; S2-generated (WRONG):
define private noundef i64 @merge_sort_helper(...) norecurse ... memory(none) speculatable {
entry:
  %_t2_cmp = icmp slt i64 %l, %r
[MISSING: branches, recursive calls, phi nodes, ret, closing }]

; Rust S1-generated (CORRECT):
define private noundef i64 @merge_sort_helper(...) nofree mustprogress nounwind willreturn ... {
entry:
  [full body with branches, calls, ret]
```

- `opt.exe` rejects the truncated IR: "expected instruction opcode" at the next `define` line
- **Confirmed pre-existing**: `compiler_3229_s2.exe` exhibits the exact same truncation
- Both functions: recursive, take `*i64` parameters, `merge_sort_helper` calls `merge` (which uses tuples)
- `quick_sort_helper` calls `partition` which returns a tuple `(i64, i64)` → `result.0`, `result.1`
- Hypothesis: tuple return value destructuring interacts with IR generation to cause premature exit

Additionally, S2 adds incorrect `norecurse` and `alwaysinline` attributes to these recursive functions.
Rust S1 does not. This suggests a second codegen divergence.

The Rust S1 correctly builds and runs the sorting benchmark (checksum 2019526740 ✅).

## Reflection

**Scope fit**: Structural IPR fix implemented correctly and fully verified. Core scope complete.

**Latent defects**: Pre-existing S2 bootstrap issue discovered (truncated IR bodies for recursive
*i64-pointer functions with tuple returns). Not introduced by Cycle 3231. Needs dedicated cycle.

**Structural improvement**: The blacklist for `free`/`realloc` added in Cycle 3229 is now redundant
(the comma-bounded matching would prevent the false positive anyway). However it provides defense-in-depth
and documents intent — keep it.

**Philosophy drift**: None. This is correctness work on the IPR pass.

**Roadmap impact**: The pre-existing S2 truncation bug suggests the bootstrap compiler has a codegen
issue with certain patterns that isn't caught by `compiler.bmb` self-compilation (since those patterns
may not appear in `compiler.bmb` itself). Fixed Point S3==S4 verifies `compiler.bmb` compiles correctly
but does NOT catch bugs in patterns outside that file.

## Carry-Forward
- Actionable:
  - **[P1] Investigate S2 IR truncation for recursive *i64-pointer functions** — specifically
    `merge_sort_helper` and `quick_sort_helper` patterns. Suspected cause: tuple return destructuring
    (`result.0`, `result.1`) interacting with IR generation. Test hypothesis with minimal repro case.
  - The Cycle 3229 `free`/`realloc` blacklist is now redundant (comma-bounded matching prevents the
    false positive regardless) but provides defense-in-depth — leave as-is.
- Structural Improvement Proposals:
  - Consider adding a bootstrap self-test that compiles the sorting benchmark with the S2 binary to
    catch future regressions. The Fixed Point check alone misses S2 bugs for non-self-hosted patterns.
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation:
  1. **[P1] Investigate S2 IR truncation** — diagnose with minimal repro, fix the bootstrap compiler's
     handling of the offending pattern (recursive functions / tuple return destructuring / *i64 pointer)
  2. Or: survey `compiler.bmb` for stack array migration opportunities (M11-C Phase 2 dogfooding)
  3. Or: start M11-C Phase 3 design (`arr[i]` subscript syntax for `[T; N]` arrays)
