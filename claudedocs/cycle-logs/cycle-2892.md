# Cycle 2892: Remaining Interpreter-Only → Native Porting (svec/hashmap)
Date: 2026-05-15

## Re-plan
Carry-Forward from Cycle 2891: audit remaining interpreter-only features. Found 4 functions with no C runtime implementation (svec_sort, svec_remove, svec_clear, str_hashmap_update) and 1 missing from inkwell only (read_f64). All implemented this cycle.

## Scope & Implementation
**Files changed**: `bmb/runtime/bmb_runtime.c`, `bmb/src/codegen/llvm_text.rs`, `bmb/src/codegen/llvm.rs`, `runtime/libbmb_runtime.a`, `bmb/runtime/libbmb_runtime.a`, 2 new test files

**New C functions** (`bmb_runtime.c`):
- `bmb_svec_sort(i64 handle)` — `qsort` the BmbSvec data array using `bmbstring_cmp` (lexicographic memcmp + length tiebreak)
- `bmb_svec_remove(i64 handle, i64 index)` — shift-remove element at index, return 1 on success, 0 on bounds failure
- `bmb_svec_clear(i64 handle)` — sets `sv->len = 0` (keeps backing array)
- `bmb_str_hashmap_update(i64 handle, BmbString* key, i64 val)` — calls existing `str_hashmap_insert` (which overwrites)

**Text backend** (`llvm_text.rs`):
- Added `declare` statements for all 4 new C functions + str_hashmap_update
- Added dispatch entries in the function name → LLVM name table
- Added `"i64"` return type annotations

**Inkwell backend** (`llvm.rs`):
- Added registration for all 4 svec functions using `svec_h_fn` / `svec_remove_type`
- Added `str_hashmap_update` registration
- Added `read_f64` registration (was missing from inkwell, existed in text backend since Cycle 2875)

**Runtime libraries rebuilt**:
- `bmb/runtime/libbmb_runtime.a` (for text backend `bmb build`)
- `runtime/libbmb_runtime.a` (for inkwell `bmb build --features llvm`)

**Key insight**: Two separate `libbmb_runtime.a` locations serve two different build paths. The verbose output shows `"Using runtime: D:\data\lang-bmb\runtime\libbmb_runtime.a"` for inkwell builds — this is the top-level `runtime/` copy.

## Verification & Defect Resolution
- `cargo test --release -p bmb` → 2388 passed, 0 failed ✅
- All 21 `tests/native_*.bmb` pass with inkwell binary ✅ (19 existing + 2 new)
- svec_sort test: ["cherry","apple","banana"] → first element before="cherry", after="apple" ✅
- svec_remove: len 3→2 ✅
- svec_clear: len 0 ✅
- str_hashmap_update: 10 → 99 ✅

## Reflection
- **Scope fit**: Completes the interpreter-only → native porting audit for this session
- **Latent defects**: None found
- **Structural improvement**: The two-runtime-library problem (text vs inkwell use different `.a` files) should ideally be unified or documented. Carry to Derive-Next.
- **Roadmap impact**: All builtins from Cycles 2823-2876 are now natively supported on both backends.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: 
  - Unify runtime library search (both backends should use same `libbmb_runtime.a`) — currently requires rebuilding 2 files when runtime C changes
  - Consider a CI step that rebuilds runtime libraries when `bmb_runtime.c` changes
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2893 — update `bmb_reference.md` to reflect all newly native functions; update HANDOFF.md; run comprehensive native test battery
