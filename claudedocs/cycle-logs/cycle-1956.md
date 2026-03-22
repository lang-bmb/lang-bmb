# Cycles 1956-1958: FFI Safety Infrastructure — EARLY TERMINATION
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1955: String FFI needed for edit_distance/lcs

## Scope & Implementation

### 1. setjmp/longjmp Error Handling (bmb_runtime.c)
- Added `bmb_ffi_begin()` / `bmb_ffi_end()` / `bmb_ffi_has_error()` / `bmb_ffi_error_message()`
- Thread-local state: `__thread` (GCC/Clang) / `__declspec(thread)` (MSVC)
- `bmb_panic()` → `longjmp` when `g_ffi_active` (instead of `exit(1)`)
- `bmb_panic_bounds()`, `bmb_panic_divzero()`, `bmb_assert()` — all FFI-safe
- `BMB_EXPORT` macro for `__declspec(dllexport)` on Windows

### 2. String FFI (bmb_runtime.c)
- `bmb_ffi_cstr_to_string()` — C string → BmbString (uses malloc, not arena)
- `bmb_ffi_string_data()` — BmbString → C string pointer
- `bmb_ffi_string_len()` — get string length
- `bmb_ffi_free_string()` — free BmbString (malloc-based)

### 3. DLL Export Fix (llvm_text.rs)
- `@export` functions now get `dllexport` linkage on Windows
- Ensures functions appear in DLL export table
- Fixed gc-sections stripping exported functions in SharedLib mode

### 4. Runtime Path Fix
- Synced `/d/data/lang-bmb/runtime/bmb_runtime.c` with `bmb/runtime/bmb_runtime.c`

### E2E Verified
```
C caller:
  knapsack = 9, LCS = 4, edit_distance = 3 ✅
  String FFI: cstr_to_bmb ↔ bmb_to_cstr works ✅
  No crash on errors (longjmp instead of exit) ✅

Python:
  All 8 algorithms working including string-based (edit_distance, lcs) ✅
```

## Review & Resolution
- cargo test --release: 6,186 pass ✅
- C FFI test: all pass ✅
- Python binding: all 8 algorithms correct ✅

### CRITICAL Issues Status

| Issue | Status |
|-------|--------|
| bmb_panic → exit(1) | ✅ FIXED (setjmp/longjmp) |
| String FFI 부재 | ✅ FIXED (bmb_ffi_cstr_to_string) |
| 계약 런타임 미검증 | ⚠️ PARTIAL (panic functions FFI-safe, but pre not auto-checked) |
| 전역 상태 비안전 | ⚠️ FFI state is TLS, other globals remain |

## Carry-Forward
- Pending Human Decisions: None
- EARLY TERMINATION: Core FFI safety infrastructure complete, all 8 algorithms working from Python
