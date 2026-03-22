# Cycles 1959-1961: TLS globals + bmb-crypto + inline main fix — EARLY TERMINATION
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1956: TLS globals partial, crypto not started

## Scope & Implementation

### TLS Global State (Cycle 1959)
- `bmb_rng_state` → `__thread` (xorshift64 RNG)
- `g_argc/g_argv` → `__thread` (CLI arguments)
- `g_arena_*` (6 vars) → `__thread` (arena allocator: head, current, total, max, save state)
- All use `__thread` (GCC/Clang) or `__declspec(thread)` (MSVC)

### Inline Main Fix (Cycle 1960)
- **Found**: non-emit-ir builds on MinGW failed with `undefined reference to WinMain`
  - Root cause: `__attribute__((weak)) main()` in runtime doesn't work on MinGW
- **Fixed**: Always include inline `@main()` wrapper in LLVM IR (both emit-ir and non-emit-ir modes)
- **Verified**: SHA-256 standalone build works after fix

### bmb-crypto Library (Cycle 1961)
- Created from gotgan-packages `bmb-sha256` (372 LOC)
- Added `@export pub fn bmb_sha256(input: String) -> String`
- Built as .dll — `bmb_sha256` in export table
- Python binding: matches `hashlib.sha256` exactly on 3 test vectors
  - empty string, "hello", "abc" — all PASS

## Review & Resolution
- cargo test --release: 6,186 pass ✅
- Stage 1 bootstrap: 11.8s ✅
- bmb-algo Python: all 8 algorithms ✅
- bmb-crypto Python: SHA-256 matches hashlib ✅

### CRITICAL Issues Status

| Issue | Status |
|-------|--------|
| bmb_panic → exit(1) | ✅ FIXED |
| String FFI 부재 | ✅ FIXED |
| 전역 상태 비안전 | ✅ FIXED (arena + rng + argc all TLS) |
| 계약 런타임 미검증 | ⚠️ PARTIAL (panic FFI-safe, pre auto-check 미구현) |
| MinGW weak main | ✅ FIXED (inline main always in IR) |

## Carry-Forward
- Pending Human Decisions: None
- EARLY TERMINATION: All CRITICAL issues resolved except pre auto-check (codegen change needed)
