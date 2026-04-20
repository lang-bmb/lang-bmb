# BMB Roadmap

BMB (Bare-Metal-Banter) is an AI-native, contract-verified systems programming language. This document summarizes where BMB is today and what's next. For the detailed per-cycle development log, see `claudedocs/cycle-logs/`.

---

## Current Status — v0.98 (2026-04-21, post-Cycles 2353-2358)

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

### Cycles 2353-2358 (this session)

**CI smoke gate for `bmb bench --compare`.** Added `bench-compare-smoke` job to `.github/workflows/ci.yml` that runs `scripts/test-bench-compare.sh` (10/10 CLI scenarios) on every PR. Closes the "2% regression threshold CI Requirement" basic gate. Full nightly baseline-diff remains a follow-up.

**XOR `^` operator.** Added `TK_CARET` lexer token and taught `parse_bitxor_rest` to accept `^` as a synonym of the existing `bxor` keyword. Bootstrap-only per Rule 6 — the Rust compiler stays frozen. Completed in 1 cycle (budgeted 3-5). 3-Stage Fixed Point preserved (S2 == S3).

**`stdlib/net` TCP primitive landing.** Added `tcp_listen` + `tcp_accept` to `bmb/runtime/bmb_runtime.c` (Win32 + POSIX). Wired them into the bootstrap compiler (types, dispatch, extern declare). New `stdlib/net/mod.bmb` provides `tcp_connect / listen / accept / read / write / close` wrappers. Smoke test `tests/bench/net_listen_smoke.bmb` passes (listen on ephemeral port 0 + close, exit 0 via Stage 1). Echo server E2E (needs external client) deferred.

**Latent bug: `gen_runtime_decls()` missing async_socket declares.** Discovered while running the net smoke test: the compiler's runtime declaration emitter never emitted `declare` lines for `bmb_async_socket_*`, so user code calling those would fail `opt -O2` verification. No prior user code exercised this path, hiding the bug. Added all six (`connect / read / write / close / listen / accept`) — fix verified end-to-end.

### Cycles 2341-2351 (previous session)

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
| `bench --compare` CI smoke gate | ✅ Cycle 2353 (scripts/test-bench-compare.sh 10/10 on every PR) |
| `bench --compare` nightly baseline diff | Pending (needs baseline-storage strategy) |
| XOR `^` operator (bootstrap) | ✅ Cycle 2354 |
| `stdlib/net` TCP primitive (listen/accept/connect/read/write/close) | ✅ Cycles 2355-2357 (wrappers + Stage 1 smoke; E2E echo server pending) |
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
| `stdlib/net` echo-server E2E smoke | 2-4 cycles | MEDIUM | Needs external client (Python) or multi-threaded server; exercises `accept` + read/write round-trip |
| `bench --compare` nightly baseline diff | 2-3 cycles | MEDIUM | Decide baseline storage (repo-commit vs CI artifact); wire into `nightly-bench.yml` |
| Cross-platform SIMD + net verification (Linux/macOS) | 3-5 cycles | LOW-MEDIUM | Needs Linux/macOS shell or GH Actions runner; v0.97 SIMD + v0.98 net never run outside Windows |
| Runtime stack trace support (DWARF) | 4-6 cycles | MEDIUM | MIR currently lacks span info — gains limited to function-level unless MIR refactored; reconsider vs ROI |
| UDP + TLS in `stdlib/net` | 6-10 cycles | MEDIUM-HIGH | Extends skeleton; TLS needs OpenSSL binding |
| CHANGELOG.md reconstruction (v0.67 → v0.98) | 3-5 cycles | LOW | Retroactive; could be partial |
| PyPI wheel publish pipeline | 2-4 cycles | MEDIUM | Packaging ready, needs CI job + secret management |

---

## Structural limits (not planned to change)

| Item | Reason |
|------|--------|
| Z3 verify self-hosting | External SMT solver — IPC-only integration |
| Complete Rust retirement | Maintained as regression gate only |
| LLVM-inherent benchmark gaps (insertion_sort, running_median, max_consecutive_ones) | Identical IR; ISel heuristic differences |

---

For granular history (per-cycle logs, decisions, rejected alternatives), see the internal `claudedocs/cycle-logs/` directory.
