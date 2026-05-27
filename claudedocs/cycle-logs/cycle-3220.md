# Cycle 3220: M11-C Phase 2 — IPR memset Bug Fix + stack_bytes_new Correctness
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3219 Carry-Forward — implement `parse_block_let_array_type_aware` + `lower_stack_array_sb`.

**Trigger**: 🟠 RE-PLAN — On attempting to test `stack_bytes_new` in the brainfuck benchmark, discovered
a CRITICAL correctness bug: brainfuck "Nested loops" outputs garbage ('€' instead of 'X'). Root cause
investigation took priority over new parser features.

**Adjusted scope**: Root cause analysis + fix for the `stack_bytes_new` correctness bug.

## Scope & Implementation

### Root Cause Analysis

The brainfuck benchmark uses `stack_bytes_new` via `@inline fn tape_new()`. Two bugs discovered:

#### Bug 1: `ipr_all_calls_readonly` treats ALL `@llvm.*` intrinsics as "readonly"

File: `bootstrap/compiler.bmb` line 17261-17263:
```bmb
// BEFORE (BUG):
// LLVM intrinsics are always readonly
if find_pattern_at(fn_name, "llvm.", 0) >= 0 { true }
```

This caused `annotate_memory_read_interprocedural` (the module-level IPR pass) to annotate `tape_new()`
with `memory(read)` — because the function:
- Has no `store ` instructions (`ipr_has_store` → false)
- Has only one call (`@llvm.memset.p0.i64`) which was classified as "readonly" (Bug 1)

Result: `tape_new()` got `memory(read)` in the emitted LLVM IR.

#### Bug 2: LLVM inliner adds premature `lifetime.end`

When LLVM's inliner inlines `tape_new()` into `interpret()`, it adds:
```llvm
call void @llvm.lifetime.start.p0(i64 30000, ptr nonnull %_t1_arr1.i)
%_t1.i = ptrtoint ptr %_t1_arr1.i to i64
call void @llvm.lifetime.end.p0(i64 30000, ptr nonnull %_t1_arr1.i)  ← PREMATURE
```

The `lifetime.end` is placed at the "exit point" of the inlined function (where `ret` was). With
`memory(read)` on `tape_new()`, the optimizer then:
1. Sees alloca with lifetime.end before any reads → memset is a "dead store" → ELIMINATED
2. Subsequent loads from the tape read uninitialized stack data → garbage output

Even after fixing Bug 1 (removing `memory(read)`), the `lifetime.end` placement problem persists
because `ptrtoint` breaks LLVM's pointer provenance tracking — LLVM can't see that `%_t1.i + offset`
accesses the alloca's memory.

**The fix for Bug 2**: Move `stack_bytes_new` call DIRECTLY into `interpret()` (not via an `@inline`
wrapper function). When the alloca is in `interpret()`'s own body, the LLVM inliner is NOT involved,
so no `lifetime.start/end` markers are added.

### Changes

#### 1. `bootstrap/compiler.bmb` — Fix `ipr_all_calls_readonly`

```bmb
// AFTER (FIX):
// LLVM intrinsics are readonly EXCEPT memory-writing ones (memset/memcpy/memmove)
if find_pattern_at(fn_name, "llvm.", 0) >= 0 {
    find_pattern_at(fn_name, "llvm.memset", 0) < 0
    and find_pattern_at(fn_name, "llvm.memcpy", 0) < 0
    and find_pattern_at(fn_name, "llvm.memmove", 0) < 0
}
```

#### 2. `bootstrap/compiler.bmb` — Fix `ipr_has_store`

Added detection of `@llvm.memset/memcpy/memmove` calls as write operations (defense-in-depth).

#### 3. `main.bmb` — Move `stack_bytes_new` to `interpret()` directly

```bmb
// tape_new() reverted to safe calloc abstraction:
@inline fn tape_new() -> i64 = calloc(tape_size(), 1);
@inline fn tape_free(tape: i64) -> i64 = { free(tape); 0 };

// interpret() uses stack_bytes_new directly:
fn interpret(prog: String) -> i64 = {
    let tape = stack_bytes_new(tape_size());  // alloca directly — no lifetime.end issue
    ...
    0  -- stack tape: no free needed
};
```

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":141,"failed":0}
```

Golden tests: **2862/2862 ✅** (no regressions)

Fixed Point: **S2 IR == S3 IR ✅**

Brainfuck correctness: "Nested loops" now correctly outputs 'X' ✅

Performance (stable runs):
- C (GCC -O2): ~17ms
- BMB (stack tape): ~17ms → **BMB ≈ C ✅**

## Reflection

**Scope fit**: RE-PLAN was correct. The `stack_bytes_new` correctness bug blocked all further
development. The root cause was subtle: a two-layer interaction between (1) the IPR pass incorrectly
treating `@llvm.memset` as readonly and (2) LLVM's inliner adding premature `lifetime.end` markers.

**Latent defects**: The `ptrtoint` escape issue means that CORRECT use of `stack_bytes_new` requires
the alloca to be in the CALLING function's body — NOT inside an `@inline fn` wrapper. This is now
documented in `main.bmb` with a comment. Future users of `stack_bytes_new` need to be aware.

**Structural improvement opportunities**:
- The `stack_bytes_new` + `@inline fn` pattern is dangerous and should be explicitly documented
  or the compiler should emit a warning when `stack_bytes_new` is used in an `@inline fn`
- `ipr_all_calls_readonly` assumed all LLVM intrinsics are readonly — this was an overly broad
  assumption; other future intrinsics that write (e.g., `llvm.memcpy.inline`) need similar treatment

**Philosophy drift**: None — fixing correctness bugs is core to Performance > Everything.

**Roadmap impact**: M11-C Phase 1 complete (stack_bytes_new working correctly). Next: parser
extension for `[u8; N]` type syntax (Cycle 3221+).

## Carry-Forward

- **Actionable**: Cycle 3221 — resume M11-C Phase 2: `parse_block_let_array_type_aware` + `lower_stack_array_sb`
- **Structural Improvement Proposals**:
  1. Add compiler warning when `stack_bytes_new` is used inside `@inline fn` (lifetime.end risk)
  2. Consider adding a validation pass that checks `@llvm.memcpy`, `@llvm.memmove` similarly to memset
- **Pending Human Decisions**: None
- **Roadmap Revisions**: M11-C Phase 1 (stack_bytes_new builtin) ✅ COMPLETE
- **Next Recommendation**: Cycle 3221 — `[u8; N]` type annotation parser support + Stage 2 build
