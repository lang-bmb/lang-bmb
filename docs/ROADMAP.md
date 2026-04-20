# BMB Roadmap

BMB (Bare-Metal-Banter) is an AI-native, contract-verified systems programming language. This document summarizes where BMB is today and what's next. For the detailed per-cycle development log, see `claudedocs/cycle-logs/`.

---

## Current Status — v0.98 (2026-04-21)

### Progress

```
Bootstrap   ██████████████████░░ 98%   3-Stage Fixed Point ✅ (S2 == S3, re-verified post-runtime changes)
Self-Host   ████████████████████ 99%   41 CLI commands, 9-feature LSP, REPL, fmt, lint
Benchmark   ████████████████████ 100%  309 builds, 16+ FASTER vs C, 0 FAIL
Ecosystem   ████████████████░░░░ 82%   5 binding libraries (140 @export), 1,017 pytest
SIMD        ████████████████████ 100%  f64/f32/i32/i64 ×N, masks, shuffle Phase 1+2
Tooling     ████████████████░░░░ 80%   @bench native + --compare ✅, doctor script, Z3 verify
```

### Headline numbers

| Metric | Value |
|--------|-------|
| Self-hosted compiler | 19,818 LOC in BMB (Stage 2 == Stage 3) |
| Golden tests | 2,815 / 2,815 passing (100%) |
| Rust test suite | 6,201 tests passing |
| Benchmark suite | 309 builds, 0 FAIL, BMB > C+Rust in 16 benchmarks |
| Binding ecosystem | 5 libraries, 140 @export functions, 1,017 pytest integration tests |
| Standard library | 15 / 15 modules (core, string, array, io, json, math, time, fs, ...) |

---

## Recently completed

### Cycles 2341-2351 (this session)

**`bmb bench --compare` regression-gate CLI.** Diffs two NDJSON bench outputs by name, classifies each bench into OK / REGRESSION / IMPROVEMENT / MISSING / NEW against a `--threshold` (default 2%), and exits 1 on any regression — CI-ready. Human and machine output modes. `scripts/test-bench-compare.sh` covers 10 scenarios (status categories + error paths). See [BENCHMARK.md](BENCHMARK.md#regression-detection-via---compare).

**Runtime source divergence fixed.** `runtime/bmb_runtime.c` had drifted from `bmb/runtime/bmb_runtime.c` (v0.95 legacy vs v0.98 canonical — notably `bmb_delete_file` return convention flipped from 1/0 to 0/-1). Root caused by the build system compiling from `bmb/runtime/` but mirroring only the `.a` to `runtime/`. Fixed by syncing sources and teaching `scripts/bootstrap.sh` to auto-copy `.c`/`.h` alongside the `.a`, preventing future drift.

**Golden test `test_golden_file_io_extras` repaired.** The failure that the previous handoff attributed to `getcwd` type-registration was actually the `bmb_delete_file` API flip above; test was checking `result == 1` against a function now returning `0` on success. Fixed the expectation; golden test now passes (2,815 / 2,815).

**3-Stage Fixed Point re-verified.** `S2 == S3` (108,574 lines identical, 74 s) after the `bmb_black_box` and runtime-source changes of the previous two sessions — the regression risk flagged in that handoff is now closed.

### Cycles 2326-2339 (previous session)

**`@bench` native mode.** `bmb bench --native` compiles each bench file with a synthesized timing harness. Measured 340× speedup vs interpreter on a real workload (LCG hash: 1.4 μs native vs 473 μs interp, CoV 1.9%). Uses `bmb_black_box` (volatile sink) to defeat LLVM DCE; constant folding remains a known limitation for pure bench bodies.

**SIMD performance guide.** `docs/SIMD_PERF.md` — when to reach for manual SIMD vs trust the auto-vectorizer, based on measured WIN/TIE/LOSE patterns across SAXPY, matvec, dot, stencil.

**Developer environment.** `scripts/doctor.ps1` (877 LOC) checks & auto-installs the Windows toolchain. `docs/DEV_ENVIRONMENT_SETUP.md` covers Windows / Linux / macOS / WSL2.

**Phase C (native ptr) — deferred indefinitely.** Evidence: `opt -O2` eliminates 100% of `inttoptr` instructions in both SAXPY (5→0) and stencil (17→0) hot paths. LLVM's alias analysis + SROA handles the conversion automatically. No measurable benefit justifies the 25–39-cycle multi-session migration.

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
| `@bench --native` mode | ✅ Cycles 2330-2334 |
| `bmb bench --compare` regression-gate CLI | ✅ Cycles 2344-2347 |
| Windows dev environment doctor | ✅ |
| Runtime source auto-sync (`runtime/` ↔ `bmb/runtime/`) | ✅ Cycle 2348 |
| Cross-platform SIMD verification (Linux/macOS) | Pending (needs Linux/macOS env) |
| `bench --compare` CI workflow integration | Pending (CLI ready) |
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
| `bench --compare` CI workflow integration | 1–2 cycles | LOW | Wire `test-bench-compare.sh` + baseline-artifact strategy into `.github/workflows/ci.yml`; closes the "2% regression gate" CI goal |
| Cross-platform SIMD verification (Linux/macOS) | 2–3 cycles | LOW-MEDIUM | Needs Linux/macOS shell or GH Actions runner |
| `^` (XOR) operator in language spec | 3–5 cycles | MEDIUM | Bootstrap only (Rule 6 — Rust compiler frozen); touches parser / AST / types / MIR / codegen |
| Runtime stack trace support (DWARF) | 4–6 cycles | MEDIUM | Debugging ergonomics |
| `stdlib/net` module (TCP/UDP) | 10+ cycles | MEDIUM-HIGH | Brand new external surface |
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
