# Cycle 1871: Bootstrap Inline Main Wrapper — Runtime Init Fix

Date: 2026-03-12

## Inherited → Addressed
Cycle 1870: max_consecutive_ones 1.13x confirmed as LLVM SLP vectorizer limitation. Bootstrap inline main wrapper added but Stage 3 binary showed help (g_argc/g_argv not initialized).

## Scope & Implementation

### Runtime Init Fix
- **Root cause**: Inline `@main()` wrapper in bootstrap IR called `bmb_user_main()` directly without initializing runtime globals `g_argc`/`g_argv` or arena
- **Fix**: Added `bmb_init_runtime(int argc, char** argv)` function to `bmb_runtime.c` that sets g_argc, g_argv, and calls bmb_arena_mode(1)
- **Wrapper updated**: Now calls `bmb_init_runtime` before `bmb_user_main`, and `bmb_arena_destroy` after

### Files Changed
- `bmb/runtime/bmb_runtime.c` — Added `bmb_init_runtime()` function
- `bootstrap/compiler.bmb` — Updated inline main wrapper with init/destroy calls + declarations
- `bmb/src/build/mod.rs` — Updated Rust-side emit-ir wrapper with init/destroy calls + declarations

## Review & Resolution

### Verification
- **Rust tests**: 6,186/6,186 PASS (3,762 + 47 + 2,354 + 23)
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (108,519 lines IR, S2 == S3)
- **Stage timing**: S1 20s, S2 24s, S3 33s (78s total)
- **Benchmark**: Suite not available in working tree (submodule not initialized)

## Carry-Forward
- Pending Human Decisions: Accept max_consecutive_ones as LLVM SLP vectorizer limitation (from cycle 1870)
- Discovered out-of-scope: None
- Next Recommendation: Phase C-E per roadmap — investigate max_consecutive_ones alternate IR patterns, runtime splitting
