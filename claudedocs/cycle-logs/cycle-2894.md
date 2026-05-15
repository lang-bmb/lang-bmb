# Cycle 2894: str_hashmap_values Native Implementation
Date: 2026-05-15

## Re-plan
HANDOFF.md listed `str_hashmap_values` as the sole remaining interpreter-only function. Investigation revealed it was not even implemented in the interpreter — only documented as planned. Implementing it now completes the full native porting effort.

## Scope & Implementation
**Files changed**: `bmb/runtime/bmb_runtime.c`, `bmb/src/types/mod.rs`, `bmb/src/interp/eval.rs`, `bmb/src/codegen/llvm_text.rs`, `bmb/src/codegen/llvm.rs`, runtime libs, 1 test file

**Internal function already existed**: `str_hashmap_values(handle)` was already implemented in C as a helper (collecting i64 values via `bmb_vec_push`). Only needed a `bmb_str_hashmap_values` public wrapper.

**Changes**:
- C runtime: `bmb_str_hashmap_values(i64) -> i64` wrapper calling existing `str_hashmap_values`
- Type checker: `str_hashmap_values: (i64) -> i64` (vec handle)  
- Interpreter: `builtin_str_hashmap_values` — uses `builtin_vec_new` + `builtin_vec_push` per value
- Text backend: `declare i64 @bmb_str_hashmap_values(i64)`, dispatch, return type
- Inkwell: `shm_vals_t = i64_type.fn_type(&[i64_type.into()], false)` registration
- Runtime libs: both `bmb/runtime/libbmb_runtime.a` and `runtime/libbmb_runtime.a` rebuilt

## Verification & Defect Resolution
- `cargo test --release -p bmb` → 2388 passed, 0 failed ✅
- Text backend: n=3, sum=60 ✅
- Inkwell backend: n=3, sum=60 ✅
- Interpreter (`bmb run`): also works via builtin ✅
- All 22 `tests/native_*.bmb` pass ✅

## Reflection
- **Scope fit**: Completes interpreter-only → native porting campaign started in Cycle 2871
- **Residual**: The reference documentation still has some "interpreter-only" comments in example code (lines 176, 505, 556, 647, etc.) that weren't updated in Cycle 2893. These are examples, not API docs — lower priority.
- **Session-level achievement**: ALL builtins from Cycles 2823-2876 are now natively supported on both text and inkwell backends. Zero interpreter-only builtins remain.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - Runtime library unification (text + inkwell use different .a files — manual dual rebuild required)
  - CI auto-rebuild of runtime libs on bmb_runtime.c change
  - Parity test for inkwell vs text backend function registration
- Pending Human Decisions: B축 재측정, tier3-spawn-overhead, runtime lib unification
- Roadmap Revisions: None
- Next Recommendation: Cycle 2895 — either next ROADMAP language gap (M4/M5 items) OR start new language feature (see ROADMAP.md for P-track/M-track priorities)
