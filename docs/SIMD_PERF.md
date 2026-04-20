# BMB SIMD Performance Guide

**Audience:** BMB users deciding when to write `f64x4`/`f32x16`/`i32x8`/etc. by hand vs. let the auto-vectorizer do its job.

**TL;DR:** Manual SIMD wins ONLY in cases the LLVM auto-vectorizer can't handle. For straight-line element-wise loops, scalar code is the right answer — `-O2` already produces packed SIMD.

---

## What BMB SIMD provides

### First-class types
- Float: `f64x4`, `f64x8`
- f32: `f32x4`, `f32x8`, `f32x16` — opens AVX-512 hot path
- Integer: `i32x4`, `i32x8`, `i64x2`, `i64x4`, `u32x*`, `u64x*`
- Masks: `mask2`, `mask4`, `mask8`, `mask16` — boolean per-lane

### Operations (`@include "stdlib/simd/mod.bmb"` — auto-loaded)

| Family | Functions | Lowers to |
|--------|-----------|-----------|
| Element-wise arithmetic | `+`, `-`, `*`, `/`, `%` (operators on `VxN`) | `fadd <N x T>` / `add <N x T>` / etc. |
| Load/Store | `load_VxN(base, idx)`, `store_VxN(base, idx, v)` | typed `load`/`store <N x T>` |
| Splat | `splat_VxN(scalar)` | `insertelement` + `shufflevector` |
| Horizontal sum | `hsum_VxN(v) -> scalar` | `llvm.vector.reduce.{fadd,add}.vNT` |
| FMA | `fma_VxN(a, b, c) -> a*b + c` | `llvm.fma.vNT` |
| Min/Max | `{min,max}_VxN(a, b)` | `llvm.{minnum,maxnum,smin,smax}.vNT` |
| Comparison | `cmp_{eq,ne,lt,le,gt,ge}_VxN(a, b) -> maskN` | `fcmp o*` / `icmp s*` |
| Blend | `blend_VxN(m: maskN, a, b) -> VxN` | `select <N x i1>, <N x T>, <N x T>` |
| Mask reductions | `mask_{any,all}_N(m: maskN) -> bool` | `llvm.vector.reduce.{or,and}.vNi1` |
| Dot product | `dot_VxN(a, b) -> scalar` | `*` + `hsum` (auto-FMA at `-O2`) |

### Shuffle (single-vector)
`reverse_VxN`, `broadcast_lane_VxN(v, lane)`, `slide_left_VxN(v, n)`, `slide_right_VxN(v, n)`. The lane / shift argument must be a compile-time integer literal.

### Shuffle (2-source)
`slide_left2_VxN(a, b, n)`, `slide_right2_VxN(a, b, n)`, `concat_lo_hi_VxN(a, b)`, `concat_hi_lo_VxN(a, b)` — for cross-block windowing such as N-point stencils.

Both backends — text (`llvm_text.rs`) and inkwell (`llvm.rs`, `--features llvm`) — emit identical IR (verified by runtime equality on the SIMD bench suite).

---

## When manual SIMD WINS, TIES, or LOSES vs auto-vec

Measurements taken on Windows MinGW + Clang `-O2`, AVX2 host (`-march=native`).

### Wins — write SIMD by hand

| Pattern | Why auto-vec fails | Use |
|---------|-------------------|-----|
| Data-dependent control flow per lane | Vectorizer can't prove safety to merge branches | `cmp_*` + `blend_*` |
| Unstructured cross-lane reductions | Vectorizer needs known reduction shape | `hsum_*`, `mask_any/all_*` |
| Forced FMA in non-`-ffast-math` builds | Plain `a*b+c` doesn't fuse without `fast` flag | `fma_*` |
| Aligned wide stores in dense loops | Cost model may pick narrower lanes | Explicit `f64x8` |

### Ties — either approach is fine; prefer scalar for readability

| Pattern | Result | Note |
|---------|--------|------|
| SAXPY (`y = a*x + y`) | manual ≈ scalar | `-O2` already AVX2 + FMA in scalar |
| matvec (matrix × vector) | manual ~10% faster | ILP unroll helps; small win, not 2× |
| dot product | manual ~7% faster | Reduction tree slightly tighter than auto |

### Loses — auto-vec is better, DON'T write SIMD

| Pattern | Result | Why |
|---------|--------|------|
| 1D 5-point stencil (v1, 5 unaligned loads) | scalar ~25 ms vs SIMD ~30 ms | 5 unaligned vector loads/iter |
| 1D 5-point stencil (v2, slide_left2 + 3 aligned loads) | scalar ~25 ms vs SIMD ~27 ms | 3 aligned loads + 4 shuffles; competitive (variance within noise) |
| Element-wise `x = a + b` over arrays | scalar matches SIMD | Trivially vectorized; no overhead win |
| Reduction over contiguous data | `hsum + sum` ties | Auto-vec uses optimal reduction tree |

---

## Decision flow for BMB users

```
Need to compute over a contiguous array of doubles/ints?
│
├─ Branchless, predictable stride 1?
│  └─→ Write scalar BMB. `-O2` will vectorize. Don't bother with manual SIMD.
│
├─ Per-lane data-dependent (e.g., "if v[i] > 0 then ... else ...")?
│  └─→ Use cmp_*_VxN + blend_VxN. This is where masks shine.
│
├─ Need cross-lane shuffles (slide, broadcast lane, reverse) WITHIN a single vector?
│  └─→ Use reverse_*_VxN / broadcast_lane_*_VxN(v, lane) / slide_{left,right}_*_VxN(v, shift).
│      Constants only: lane/shift must be compile-time integer literals.
│
├─ Need cross-block shuffle (e.g., 5-point stencil combining adjacent 4-wide windows)?
│  └─→ Use slide_left2_* / slide_right2_* / concat_lo_hi_* / concat_hi_lo_*.
│      Stencil recovery verified competitive with scalar — choose based on readability.
│
├─ Need aggregate reduction across lanes (sum / any / all)?
│  └─→ Use hsum_*_VxN / mask_any_N / mask_all_N — these are the win cases.
│
└─ Tight FMA chain that must run as `vfmadd*` even without -ffast-math?
   └─→ Use fma_VxN explicitly.
```

---

## f32 SIMD specifics

All f32 SIMD operations parallel the f64 suite:
- `load_f32x{4,8,16}` / `store_f32x{4,8,16}` — typed vector memory
- `splat_f32x{4,8,16}(x: f32)` — scalar broadcast
- `hsum_f32x{4,8,16}(v) -> f32` — horizontal sum
- `dot_f32x{4,8,16}(a, b) -> f32` — inline composition
- `fma_f32x{4,8,16}(a, b, c)` — single AVX `vfmadd*ps` / AVX-512 `vfmadd*ps` on zmm
- `min_f32x{4,8,16}` / `max_f32x{4,8,16}` — IEEE 754 minNum/maxNum
- `cmp_{eq,ne,lt,le,gt,ge}_f32x{4,8,16}` → `maskN`
- `blend_f32x{4,8,16}` — per-lane select
- `mask_{any,all}_16` — f32x16 mask reductions

### Cast support
Full f32 ↔ scalar type conversions via `as`:
- `i64/i32/u64/u32/Char as f32` → `sitofp`/`uitofp`
- `f32 as i64/i32/u64/u32/Char` → `fptosi`/`fptoui`
- `f32 as f64` → `fpext`
- `f64 as f32` → `fptrunc`

Literal coercion: `let x: f32 = 3.14;` works (F64 literal auto-coerces to F32, mirroring integer `let x: u32 = 10;` precedent).

### Use cases
- AVX-512 hot paths: 16-lane f32 FMA at 2× f64 throughput per register
- ML inference: f32 is the default precision for quantized neural nets
- Image / audio DSP: f32 is native pixel/sample format for most DSP

### Precision caveat
f32 is not associative. Parallel reductions (`hsum_f32x*`) and long accumulator chains can produce bit-different outputs across backends/compilers even for correct code. Use integer checksums for determinism-critical validation.

---

## Validation

| Workload | Outcome |
|----------|---------|
| 51 runtime correctness checks (hsum/splat/load/store/fma/min-max/mask/shuffle) | All pass on both backends |
| SAXPY (4096×5000) — f64 and f32 | Output match scalar; SIMD ~10% wins on matvec |
| 1D 5-point stencil v2 (slide_left2) | Competitive with scalar (~27 ms vs ~25 ms) |
| `tests/bench/simd_f32_correctness.bmb` | 12 checks across f32x4/f32x8/f32x16, exit 0 |
| `tests/bench/stencil_v2_min.bmb` | Minimal stencil repro for regression |
| 3-Stage bootstrap Fixed Point | S2==S3 (~65s) |

**No claim is "SIMD beats C/Rust by 4×" — those headlines come from cherry-picking.** The BMB story is: SIMD types exist as first-class, two backends are bit-identical, and you should reach for them when scalar can't express what you need.

---

## Status & limitations

| Area | Status |
|------|--------|
| f64xN, f32xN, i32xN, i64xN | ✅ |
| Mask types + cmp/blend/any/all | ✅ |
| Shuffle Phase 1 (single-vector) | ✅ |
| Shuffle Phase 2 (2-source) | ✅ |
| Scalar 32-bit helpers (`store_i32`/`load_i32`/`store_f32`/`load_f32`) | ✅ |
| Auto-import (`@include "stdlib/simd"`) | ✅ |
| Both codegen backends (text + inkwell) | ✅ Rule 7 parity |
| Cross-platform AVX2 verification (Linux/macOS) | Pending |
| Gather/scatter intrinsics | Not yet |

For internal development notes and historical context, see `claudedocs/SIMD_PERF_NOTES.md`.
