# Cycle 1831: Stack Overflow Fix + Build Pipeline Restoration
Date: 2026-03-10

## Inherited → Addressed
From 1830: "Address bootstrap segfault or continue with other development areas"
Addressed: Fixed stack overflow (thread spawn), fixed 3 codegen type mismatches, fixed build pipeline.

## Scope & Implementation

### 1. Stack Overflow Fix (main.rs)
- Wrapped all command dispatch in spawned thread with 64MB stack (`COMPILER_STACK_SIZE`)
- Root cause: `infer()` (1,625 lines, 57 match arms) creates huge stack frames; 27-30 level nested if-else in compiler.bmb overflows default 1-2MB stack
- `bmb check bootstrap/compiler.bmb` now succeeds

### 2. Codegen Type Fixes (llvm_text.rs)
- **Select type registration**: Added `MirInst::Select` to `build_place_type_map()` — was defaulting to i64 for ptr results
- **Narrowed i32 param sext in Select**: Added sext from i32→i64 for narrowed params used in Select condition comparisons
- **Copy ptr propagation**: Allow ptr/double types to override declared local types (was blocked by v0.51.48 guard)
- **Second-pass type resolution**: Added second pass in `build_place_type_map()` for phi/copy/select that reference not-yet-typed back-edge variables

### 3. Build Pipeline Fix (build/mod.rs)
- **Linker**: Replaced `lld-link` (MSVC) with `clang` (works in MinGW/MSYS2 environments)
- **MinGW target**: Added `--target=x86_64-pc-windows-gnu` to avoid MSVC header conflicts
- **Runtime discovery**: `find_runtime_c()` now searches for `bmb_runtime.c` (full runtime) before `runtime.c` (legacy); handles `BMB_RUNTIME_PATH` as directory
- **Event loop**: Added compilation of `bmb_event_loop.c` alongside main runtime
- **UCRT64 clang path**: Added `C:\msys64\ucrt64\bin\clang.exe` to candidates

### Files Changed
- `bmb/src/main.rs` — Thread spawn with 64MB stack
- `bmb/src/codegen/llvm_text.rs` — Select type, narrowed param sext, Copy ptr propagation, second-pass types
- `bmb/src/build/mod.rs` — Linker, MinGW target, runtime discovery, event loop

## Review & Resolution
- All 6,186 tests pass
- `bmb build` works for simple programs (exit code 42 ✓)
- Stage 1 bootstrap succeeds: `compiler.bmb` → `bmb-stage1.exe` (12s)
- Stage 2 succeeds: stage1 compiles `compiler.bmb` → 107,729 lines IR
- No regressions

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Inkwell backend segfaults on Windows with `--fast-compile` (pre-existing v0.50.54)
- Next Recommendation: Run full 3-stage bootstrap to verify fixed point; continue with Cycle 1832
