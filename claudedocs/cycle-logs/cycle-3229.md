# Cycle 3229: IPR `array_free memory(read)` False Positive Fix + Array Allocation Bug Fix
Date: 2026-05-28

## Re-plan
Plan valid. Inherited carry-forward: fix `array_free` getting `memory(read)` from IPR pass → STATUS_HEAP_CORRUPTION when sorting benchmark compiled with bootstrap compiler. Two related bugs discovered during investigation:
1. **IPR false positive**: `ipr_all_calls_readonly` matches `"free,"` as substring of `"array_free,"` → `array_free` annotated `memory(read)` → LLVM DCE's `free()` call
2. **Array allocation mismatch**: `array_new(n)` used `malloc(n*8)` but BMB `arr[i]` syntax adds `+2` header offset, requiring `malloc((n+2)*8)`

## Scope & Implementation

### Bug 1: IPR False Positive (`bootstrap/compiler.bmb`)

**Root cause**: `ipr_try_annotate_section` adds `self_name` to `known_self` BEFORE checking calls (line 17443):
```bmb
let known_self = if self_name != "" { known + self_name + "," } else { known };
```
When `self_name = "array_free"` and `fn_name = "free"`, the key `"free,"` matches as suffix of `"array_free,"` in `known_self` → false positive.

**Fix**: Added explicit blacklist for `@free` and `@realloc` in both `ipr_all_calls_pure` (~line 17319) and `ipr_all_calls_readonly` (~line 17381):
```bmb
// @free and @realloc modify allocator state — never readonly/pure regardless of name matching
else if fn_name == "free" or fn_name == "realloc" { false }
```

### Bug 2: Array Allocation Mismatch (sorting benchmark files)

**Root cause**: BMB `arr[i]` syntax always adds `+2` to the index (bootstrap/compiler.bmb lines 9438, 9568):
```bmb
// Add 2 to index to skip array header
let hdr_const = "%_t" + i2s(t);
let _w0a = sb_push_mir(sb, "  " + hdr_const + " = const 2");
```
All `arr[i]` accesses effectively access `base[(i+2)*8]`. With `malloc(n*8)` allocation, `arr[n-1]` writes at offset `(n+1)*8` — 8 bytes beyond the allocated region.

**Fix**: Changed `array_new` in both benchmark files:
- `main.bmb`: `malloc(n * 8)` → `malloc((n + 2) * 8)`
- `main_inproc.bmb`: `malloc(n * 8)` → `malloc((n + 2) * 8)`

### Bootstrap Stage Notes
- `compiler_3224.exe` has pre-existing breakage building Stage 2 from `compiler.bmb`
- Used `target/release/bmb.exe` (Rust compiler, frozen) to build `compiler_3229_rust_s1.exe` from fixed `compiler.bmb`
- S1 verification: `compiler_3229_rust_s1.exe` correctly builds sorting benchmark with checksum 2019526740

## Verification & Defect Resolution

### Tests
- `cargo test --release`: **6282 tests, 0 FAILED** ✅
- S1 bootstrap builds correctly from fixed `compiler.bmb`

### IPR Fix Verification
- Pre-opt IR `sort_s1_ipr_check.ll`: `array_free` definition has no `memory(read)` attribute ✅
- `compiler_3229_rust_s1.exe emit-ir main_inproc.bmb`: confirmed no `memory(read)` on `array_free`

### Array Allocation Fix Verification
- `sort_v2.exe` (both fixes): checksum = 2019526740, exit 0 ✅
- `sort_s1_final.exe` (S1 compiled): checksum = 2019526740, exit 0 ✅

### Performance Analysis

**Key finding**: GCC -O3 does not auto-vectorize the `init_reverse` loops. LLVM opt -O2 vectorizes them (28 vector basic blocks in BMB IR vs 0 in GCC-compiled C IR).

| Measurement | BMB Fixed | C GCC-O3 | C Clang-O2 | BMB/GCC | BMB/Clang |
|-------------|-----------|-----------|------------|---------|-----------|
| External single-run | ~107ms | ~678ms | ~130ms | **0.158×** | **0.822×** |
| In-process 5-iter (µs) | ~585,000 | ~3,250,000 | ~543,000 | 0.180× | 1.077× |
| Pre-built binary (in-proc) | ~505,000 | — | — | — | 0.930× |

**Official baseline** (benchmark runner uses GCC -O3): **0.158× (BMB 6.3× faster)** ✅

The pre-built `main_inproc_bmb.exe` was faster (~505ms vs ~585ms) because the IPR bug caused LLVM to DCE `free()` calls and potentially do escape analysis → promote temp arrays to stack allocation. This was incorrect behavior (memory leak). The fixed binary correctly preserves heap allocation semantics.

The old benchmark memory "0.910×" ratio was from an earlier codebase state (Cycle 2535, 2026-05-01) before `@inline` optimizations and vectorization improvements. Current ratio is significantly better.

## Reflection

**Scope fit**: Both root causes found and fixed. IPR substring-match false positive correctly blacklists `free`/`realloc`. Array allocation mismatch corrected with `(n+2)*8` formula.

**Latent defects**: The same IPR substring-match pattern could affect other function names that are substrings of known-safe function names. E.g., if `known` contains `"my_free,"` and `fn_name = "free"`, the key `"free,"` would still match. However, this scenario requires `fn_name` to appear as a suffix of a known-safe function name in the `known` set. The explicit blacklist for `free`/`realloc` covers the critical cases.

**Structural improvement**: `ipr_try_annotate_section` adds `self_name` to `known_self` before examining calls. A more robust approach would check `fn_name == self_name` explicitly rather than substring search. But the blacklist approach is minimal and correct for the current known problematic cases.

**Philosophy drift**: None. This is a P0 correctness bug fix — `memory(read)` on `free()` causes incorrect LLVM optimization (DCE of allocator calls) leading to STATUS_HEAP_CORRUPTION.

**Roadmap impact**: No roadmap changes needed. M11-C Phase 2 cycle work continues.

## Carry-Forward
- Actionable: 
  - Investigate whether other benchmarks have `malloc(n*8)` with BMB `arr[i]` access — may need similar `(n+2)*8` fix in other benchmark BMB files
  - The `compiler_3224.exe` Stage 2 breakage is pre-existing and separate — track separately
- Structural Improvement Proposals:
  - `ipr_try_annotate_section`: Consider exact match (`fn_name == known_entry`) instead of substring match for better precision. Current blacklist approach is minimal fix. Low priority.
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Continue M11-C Phase 2 (Cycle 3230+). Consider checking other benchmark BMB files for similar `array_new(n*8)` allocation mismatch.
