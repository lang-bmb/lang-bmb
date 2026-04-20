# BMB Roadmap

BMB (Bare-Metal-Banter) is an AI-native, contract-verified systems programming language. This document summarizes where BMB is today and what's next. For the detailed per-cycle development log, see `claudedocs/cycle-logs/`.

---

## Current Status — v0.98 (2026-04-20)

### Progress

```
Bootstrap   ██████████████████░░ 98%   3-Stage Fixed Point ✅ (S2 == S3)
Self-Host   ████████████████████ 99%   41 CLI commands, 9-feature LSP, REPL, fmt, lint
Benchmark   ████████████████████ 100%  309 builds, 16+ FASTER vs C, 0 FAIL
Ecosystem   ████████████████░░░░ 82%   5 binding libraries (140 @export), 1,017 pytest
SIMD        ████████████████████ 100%  f64/f32/i32/i64 ×N, masks, shuffle Phase 1+2
Tooling     ███████████████░░░░░ 75%   @bench native mode ✅, doctor script, Z3 verify
```

### Headline numbers

| Metric | Value |
|--------|-------|
| Self-hosted compiler | 19,818 LOC in BMB (Stage 2 == Stage 3) |
| Golden tests | 2,814 / 2,815 passing (99.96%) |
| Rust test suite | 6,201 tests passing |
| Benchmark suite | 309 builds, 0 FAIL, BMB > C+Rust in 16 benchmarks |
| Binding ecosystem | 5 libraries, 140 @export functions, 1,017 pytest integration tests |
| Standard library | 15 / 15 modules (core, string, array, io, json, math, time, fs, ...) |

---

## Recently completed (Cycles 2326-2339, this session)

### `@bench` native mode (v0.98)
`bmb bench --native` compiles each bench file with a synthesized timing harness and runs the binary natively. Measured 340× speedup over the interpreter on a real workload (LCG hash: 1.4 μs native vs 473 μs interp, CoV 1.9%). Uses `bmb_black_box` (volatile sink) to defeat LLVM DCE; constant folding remains a known limitation for pure bench bodies. See [BENCHMARK.md](BENCHMARK.md) for usage and caveats.

### SIMD performance guide
`docs/SIMD_PERF.md` published — when to reach for manual SIMD types vs trust the auto-vectorizer, based on measured WIN/TIE/LOSE patterns across SAXPY, matvec, dot, stencil workloads.

### Developer environment
`scripts/doctor.ps1` (877 LOC) checks & auto-installs the Windows toolchain (MSYS2, LLVM 21, Rust GNU host, runtime library). `docs/DEV_ENVIRONMENT_SETUP.md` covers Windows / Linux / macOS / WSL2 setup.

### Phase C (native ptr) — deferred indefinitely
Evidence: `opt -O2` eliminates 100% of `inttoptr` instructions in both SAXPY (5→0) and stencil (17→0) hot paths. LLVM's alias analysis + SROA handles the conversion automatically. No measurable benefit justifies the 25–39-cycle multi-session migration. Revisit only if a workload surfaces where inttoptr is actually preserved through `-O2`.

---

## Phase overview

### v0.97 — SIMD + bindings (✅ complete)
- `f64xN`, `f32xN`, `i32xN`, `i64xN`, `u32xN`, `u64xN`, `maskN` first-class types
- `stdlib/simd` — 219 functions including shuffle Phase 1 + 2 (2-source cross-block)
- f32 primitive + AVX-512 f32x16 hot path
- Both codegen backends (text + inkwell) bit-identical
- `@bench` microbenchmark attribute + `bmb bench` interpreter mode
- 5 binding libraries (bmb-algo, bmb-compute, bmb-text, bmb-crypto, bmb-json)

### v0.98 — tooling + distribution (in progress)
| Task | Status |
|------|--------|
| `@bench --native` mode | ✅ this session |
| Windows dev environment doctor | ✅ this session |
| Cross-platform SIMD verification (Linux/macOS) | Pending |
| PyPI wheel build + publish | Packaging ✅, publish pending |
| Node.js WASM bindings | Not started |
| ~~Native Ptr type system (inttoptr removal)~~ | Deferred (evidence: auto-handled by `opt -O2`) |

### v0.99 — generics + ecosystem
- Full `Vec<T>` / `HashMap<K,V>` generics (bootstrap currently partial)
- Playground WASM deployment
- Cross-platform CI (Linux / macOS / ARM64)
- Language specification final draft

### v1.0 — release + community
- AI-native code-generation empirical study (30 problems, 34 patterns, 388 tests infrastructure ready)
- HN / Reddit announcement
- Community building

---

## Next-session options

| Option | Effort | Risk | Notes |
|--------|--------|------|-------|
| Cross-platform SIMD verification (Linux/macOS) | 2–3 cycles | LOW | Needs Linux/macOS shell access |
| `bmb bench --compare a.json b.json` CLI for regression gate | 2–4 cycles | MEDIUM | Builds on `@bench --native` output |
| 3-Stage Fixed Point re-verify after runtime.c changes | 1–3 cycles | LOW | `bmb_black_box` was added; S2==S3 not re-confirmed in session |
| `stdlib/net` module (TCP/UDP) | 10+ cycles | MEDIUM-HIGH | Brand new external surface |
| Runtime stack trace support | 4–6 cycles | MEDIUM | DWARF integration |
| CHANGELOG.md reconstruction (v0.67 → v0.98) | 3–5 cycles | LOW | Retroactive; could be partial |

---

## Structural limits (not planned to change)

| Item | Reason |
|------|--------|
| Z3 verify self-hosting | External SMT solver — IPC-only integration |
| Complete Rust retirement | Maintained as regression gate only |
| LLVM-inherent benchmark gaps (insertion_sort, running_median, max_consecutive_ones) | Identical IR; ISel heuristic differences |

---

For granular history (per-cycle logs, decisions, rejected alternatives), see the internal `claudedocs/cycle-logs/` directory.
