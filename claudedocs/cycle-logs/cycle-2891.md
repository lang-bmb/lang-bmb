# Cycle 2891: Inkwell Backend Parity — 40+ Functions (CLAUDE.md Rule 7 Fix)
Date: 2026-05-15

## Re-plan
Inherited defect from Cycles 2871-2876: those cycles added 40+ functions to `llvm_text.rs` but NOT to `llvm.rs` (inkwell backend), violating CLAUDE.md Rule 7. Also discovered pre-existing str_hashmap API mismatch: inkwell had old ptr-handle API while text backend had new i64-handle/BmbString*-key API. This cycle fixes both defects before any new roadmap work.

## Scope & Implementation
**Files changed**: `bmb/src/codegen/llvm.rs`

Three categories of fixes applied to inkwell backend:

1. **String query functions** (i64 return): `str_is_empty`, `str_contains`, `str_starts_with`, `str_ends_with`, `str_find`, `str_to_int`, `str_count`, `str_to_f64` — registered via `self.module.add_function("bmb_string_*", ...)`.

2. **String transform functions** (ptr/String return): `str_trim`, `str_to_upper`, `str_to_lower`, `str_repeat`, `str_trim_left`, `str_trim_right`, `str_reverse`, `str_char_at`, `str_substr`, `str_pad_left`, `str_pad_right`, `int_to_hex`, `int_to_bin` — registered via `reg_str_fn!` macro that adds `nonnull` attribute and entries to both `self.functions` and `self.function_return_types`.

3. **str_replace alias fix**: `bmb_string_replace` was already registered as `"replace"`. Re-adding caused LLVM to create `bmb_string_replace.1` (linker failure). Fix: alias `"str_replace"` → existing `"replace"` function entry.

4. **Vec aggregate functions**: `vec_sum`, `vec_min`, `vec_max`, `vec_sort`, `vec_reverse`, `vec_contains`, `vec_index_of`, `vec_remove`, `vec_fill` — correct void/i64 return types.

5. **Integer math**: `pow_i64`, `gcd_i64`, `clamp_i64`, `popcount` — via `bmb_*` C runtime functions.

6. **Float math**: `log`, `log2`, `log10`, `exp`, `round` via LLVM intrinsics (`llvm.log.f64` etc.); `min_f64`, `max_f64` via `llvm.minnum/maxnum.f64`; `tan`, `atan`, `atan2` via C library. `clamp_f64` special-cased in `gen_inst` (no single LLVM intrinsic — emits two calls inline).

7. **str_hashmap API fix**: Old inkwell registration used ptr handles + int64 keys. Runtime uses i64 handles + BmbString* keys. Replaced entire block with correct `bmb_str_hashmap_*` registration (8 functions: new, insert, get, contains, len, remove/delete, free, inc).

8. **Unused variable cleanup**: Removed `s3_ptr` (declared but unused after str_replace alias fix).

## Verification & Defect Resolution
- `cargo build --release --features llvm --target x86_64-pc-windows-gnu` → ✅ (1 unused variable warning fixed)
- `cargo test --release -p bmb` → 2388 passed, 0 failed ✅
- All 19 `tests/native_*.bmb` files compiled and ran correctly via inkwell binary ✅
- Specific function verification:
  - str_hashmap: insert/get/contains/len/delete/free → result 11 ✅
  - str_contains/starts_with/ends_with/find/count → correct results ✅
  - str_to_upper/to_lower/trim/reverse → correct via println_str ✅
  - vec_sum/min/max/sort/reverse → correct ✅
  - log(e)→1.0, pow_i64(2,10)→1024, clamp_f64(15,0,10)→10.0, popcount(255)→8 ✅

## Reflection
- **Scope fit**: Fully addresses CLAUDE.md Rule 7 violation from Cycles 2871-2876. Both backends now in parity.
- **Latent defects**: Pre-existing str_hashmap API mismatch (old ptr-handle vs new i64-handle) was silently lurking in inkwell. Fixed proactively.
- **Philosophy drift**: None. This is pure parity maintenance.
- **Roadmap impact**: Inkwell backend now supports all 40+ functions added in Cycles 2871-2876. No new interpreter-only functions identified in this cycle.

## Carry-Forward
- Actionable: None from this cycle — all defects resolved
- Structural Improvement Proposals: Consider adding a compile-time assertion or test that verifies `llvm.rs` and `llvm_text.rs` have the same function set (prevent future Rule 7 violations)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2892 — audit remaining interpreter-only features (svec_*, format with > 3 args, str_hashmap_keys, str_split) for native parity; update bmb_reference.md accordingly
