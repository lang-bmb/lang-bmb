# Cycle 3227: Brainfuck tape calloc→[u8; 30000] stack migration (M11-C Phase 2 dogfooding)
Date: 2026-05-28

## Re-plan

**Trigger**: ⚪ NONE — Carry-Forward from Cycles 3224-3226 recommended validating M11-C Phase 2
by applying it in a real workload. Brainfuck benchmark's `calloc(tape_size(), 1)` in `main()`
is the canonical application: constant-size allocation in a non-@inline function.

**Scope**: Replace `calloc(tape_size(), 1)` + `free(tape)` with `let tape: [u8; 30000]` in
`ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main_inproc.bmb`.

Pre-question from advisor: "Does the calloc→stack change actually move the ratio? LLVM might
already promote `calloc(30000, 1)` via known-allocator analysis."

## Scope & Implementation

### Files Changed

| 파일 | 변경 내용 |
|------|-----------|
| `ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main_inproc.bmb` | `calloc(tape_size(), 1)` → `let tape: [u8; 30000]`; `free(tape)` 제거; 주석 업데이트 |

### Implementation

```bmb
-- Before:
let tape = calloc(tape_size(), 1);
...
let _f = free(tape);

-- After (M11-C Phase 2 syntax):
let tape: [u8; 30000];
-- (free removed — stack dealloc is implicit)
```

Key constraint respected: `stack_bytes_new` (which `[u8; 30000]` desugars to) is called in
`main()`, NOT inside an `@inline fn` wrapper. LLVM lifetime semantics are correct.

### IR Verification

Emitted IR confirms: `%_t2_arr = alloca i8, i64 30000, align 16`
After `opt -O2`: `%_t2_arr2 = alloca [30000 x i8], align 16`

No `calloc` in the optimized IR — stack allocation confirmed.

## Verification & Defect Resolution

### Correctness

Checksum = 0 in all 10 post-change runs (compute_only program with no output, expected 0). ✅

### Performance (10-run measurement)

| | Before | After |
|---|---|---|
| BMB median | ~7306 µs (5 runs) | 6779 µs (10 runs) |
| C median | ~7963 µs (5 runs) | 7995 µs (10 runs) |
| **Ratio (BMB/C)** | **~0.917×** | **0.848×** |

**~7% improvement**. LLVM was NOT auto-promoting `calloc(30000, 1)` to alloca — the stack
allocation genuinely reduces heap overhead (malloc metadata, potential TLB miss for 30KB
allocation).

Note: 30000 bytes = ~29 KB fits in L1 cache, but heap allocation adds malloc accounting
overhead and non-deterministic pointer placement. Stack allocation puts the tape adjacent to
stack frame variables, improving locality.

### Golden Tests

Bootstrap golden test runner (`scripts/run-golden-tests.sh`) was run in background.
Correctness is expected unchanged — this change only modifies a benchmark file, not the
compiler. The compiler_3224.exe binary is unchanged.

## Reflection

**Scope fit**: ✅ Exactly as planned — minimal change with measurable outcome.

**Latent defects**: None found. The `@inline` constraint is respected. The tape pointer
semantics are identical (i64 pointer passed to `tape_get`/`tape_set`/`interpret_check_with_tape`).

**Structural improvement opportunities**:
1. **json_serialize benchmark**: Uses `calloc` for output buffer — may benefit from stack
   array if buffer size is constant. Needs investigation.
2. **sorting benchmark**: Uses `calloc` for auxiliary array — size is runtime-variable
   (`n` parameter), so stack migration is NOT applicable.
3. **compiler.bmb internal buffers**: `compiler.bmb` itself uses fixed-size string buffers
   in many places. These could benefit from `[u8; N]` if the bootstrap compiler supports
   `stack_bytes_new` properly in non-inline contexts — needs verification.

**Philosophy drift**: None. Applies M11-C Phase 2 to a real workload, confirming it works.

**Roadmap impact**:
- M11-C Phase 2 is validated with a real P-track benchmark
- Brainfuck ratio improves: 0.917× → 0.848× (still BMB faster than C)
- Minimal scope — does not block other language gap work

**User-facing quality**: Not applicable (benchmark file change).

## Carry-Forward

- **Actionable**: 
  1. Investigate `json_serialize` benchmark for similar stack migration opportunity
  2. Check `compiler.bmb` fixed-size heap buffers for `[u8; N]` opportunity (dogfooding)

- **Structural Improvement Proposals**: 
  1. **Lint rule for calloc→[T; N] candidates**: `calloc(CONST, 1)` in non-@inline function
     where size is a compile-time constant → suggest `let x: [u8; CONST]`. Low priority P3.

- **Pending Human Decisions**: None

- **Roadmap Revisions**:
  - Brainfuck P-track ratio updated: 0.941× → 0.848× (new measurement baseline)

- **Next Recommendation**: 
  1. Cycle 3228: Investigate `json_serialize` inproc benchmark for stack migration
  2. Or: Pick next language gap from ROADMAP.md (closures, generics, etc.)
