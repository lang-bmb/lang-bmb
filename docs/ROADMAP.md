# BMB Roadmap

BMB (Bare-Metal-Banter) is an AI-native, contract-verified systems programming language. This document summarizes where BMB is today and what's next. For the detailed per-cycle development log, see `claudedocs/cycle-logs/`.

---

## Current Status — v0.98 (2026-04-21, post-Cycles 2391-2396)

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

### Cycles 2391-2396 (this session)

**Ephemeral-port discovery for stdlib/net** (Cycles 2391-2392). Runtime
now captures the OS-assigned port via `getsockname()` after
`tcp_listen(0)` / `udp_bind(0)` (previously `sock->port` stored the
user-supplied 0). New `bmb_async_socket_port` + `bmb_async_socket_host`
runtime accessors exposed through stdlib/net as `tcp_listen_port`,
`udp_bind_port`, `tcp_peer_port`, `tcp_peer_host`. Round-trip validated
via `tests/bench/net_port_discovery_smoke.bmb` +
`net_stdlib_port_smoke.bmb`. 3-Stage Fixed Point re-verified.

**Bootstrap `@annotation pub fn` silent parse failure fixed** (Cycle
2394). A hardcoded `121` at `bootstrap/compiler.bmb:2502` (where
`TK_PUB()` is actually `2_000_000_170`) caused every
`@<anything> pub fn ...` combination to silently fail with the
`"expected 'fn' after @X, got integer literal"` fallback. Fix: literal
→ `TK_PUB()`; plus `"fn-trust"` added to `is_fn_node` so the resulting
`(fn-trust ...)` AST reaches MIR lowering. Impact: `@include "stdlib/
time/mod.bmb"` / `stdlib/fs` / `stdlib/io` / `stdlib/process` (27
public functions) now compile via bootstrap. 3-Stage Fixed Point
re-verified after the fix.

**New latent bug identified — Defect 3** (Cycles 2394-2395). Under
narrow conditions, adding a helper fn to `bootstrap/compiler.bmb`
corrupts Stage 2 self-compilation (misplaced parse errors or 16 GB
arena exhaustion). Minimal repro in Cycle 2395: a 5-line
`skip_contract_body_tokens` helper with `or`-chained `tok_kind`
comparisons. Multi-line comments containing `{...}` also trigger a
similar failure class. Blocks a tolerant `skip_contracts` fix that
would otherwise unblock stdlib/string / stdlib/array `@include` via
bootstrap (contracts use `implies`, unsupported by bootstrap parser).
Dedicated investigation deferred. **Workaround**: keep bootstrap
helper fns minimal; prefer inlining over extracting.

### Cycles 2375-2381 (earlier session)

**Bootstrap SIMD stub-compile-safe.** `@include "stdlib/simd/mod.bmb"` via bootstrap previously emitted `ret double %todo` (placeholder `= todo` body in a typed return slot → undefined reference). Two-layer fix: parser now recognises bare `todo` as `(unit)` matching the Rust compiler's `Expr::Todo → Constant::Unit` path; a new post-IR pass `fix_typed_ret_placeholders_ir` rewrites residual `ret double 0` / `ret float 0` / `ret ptr 0` (artifacts of unit-constant propagation through the identity-copy eliminator) to type-appropriate literals. 3-Stage Fixed Point re-verified. SIMD intrinsic CALL-site dispatch (vector types, splat/hsum intrinsic emission from bootstrap) remains a separate, larger work item.

**`BMB_STDLIB_PATH` env-var override restored.** The `@include` preprocessor's 3-tier resolution now includes `$BMB_STDLIB_PATH/<rel_path>` between the source-dir and CWD-fallback lookups. A stale Cycle 2362 comment claimed `getenv` was not String-typed in bootstrap; verification showed it already is. The only wrinkle: an unrelated Rust-compiler triple-concat codegen bug (`env + "/" + rel`) bites at the Rust-build stage — sidestepped with a 2-step helper function.

**`@bench native` corpus made trustworthy.** Added three memory-touching / runtime-seeded benchmarks (`bench_fnv1a_hash`, `bench_mixed_int_ops`, with `bench_lcg_prng` and heap variants evaluated and dropped for noise). Initial baseline had sub-μs benchmarks with 40-100% run-to-run variance; scaled workloads to ≥ 50 μs now produce 0-4% natural variance against the 10% nightly threshold. Committed `.bench-native-baseline.ndjson` and extended the nightly workflow to consume both `bench_smoke.bmb` and `bench_memory.bmb`.

**Orphan `runtime/*.c` / `*.h` removal.** `runtime/bmb_runtime.c`, `runtime/bmb_event_loop.c`, `runtime/bmb_event_loop.h` were sync'd copies of `bmb/runtime/*` that nobody actually read — the Rust compiler's linker lookup only consumes `runtime/libbmb_runtime.a`. Dropped the sync step from `scripts/bootstrap.sh` and removed the files.

**stdlib/net raw-buffer helpers.** `tcp_write_raw(socket, buf)` and `udp_sendto_raw(socket, host_buf, port, data_buf)` wrappers for callers who already hold extracted pointers (from `string_as_cstr` or manual allocation) — skip the String wrapping round-trip.

### Cycles 2359-2373 (earlier session)

**`stdlib/net` full E2E + UDP primitive.** Extended TCP with a Python-backed echo server round-trip (`scripts/test-net-echo.sh`, 2000-byte payload, CI gate on ubuntu-latest via `net-echo-smoke` job). Added UDP primitive (`udp_bind/sendto/recv/close`) with runtime (Win32 + POSIX), bootstrap wiring (types/dispatch/extern), and stdlib wrappers. Full bidirectional UDP echo validated. TCP loopback via `tcp_connect("127.0.0.1", ...)` also working — closes HANDOFF §4 "host: String as i64 cast 경로 미완".

**`@include` directive in bootstrap.** Users can now write `@include "stdlib/net/mod.bmb"` in BMB source and have the bootstrap compiler (Stage 1+) expand it before parsing. Line-based preprocessor with source-dir-relative + CWD-fallback resolution, max-depth-16 recursion safeguard. Wired into all compile pipeline entry points (build, check, run, test, emit-ir, compile-file-to). Introspection tools (fmt, lint, index, query) intentionally unchanged — they should see raw source. 3-Stage Fixed Point (S2 == S3) re-verified.

**Nightly `@bench --native` regression gate.** Added `@bench native baseline diff` step to `.github/workflows/nightly-bench.yml`: fetches `.bench-native-baseline.ndjson` from main, runs `bmb bench --native tests/bench/bench_smoke.bmb`, compares with `--threshold 10`. Baseline-storage strategy chosen (Option A: repo-committed NDJSON) for git-history auditability consistent with existing `.baseline.json` pattern. First-run tolerant — missing baseline emits notice without failing.

**`string_as_cstr` builtin (new v0.98 conversion).** Runtime `bmb_string_as_cstr(const BmbString* s) -> i64` returns `s->data`. Wired into bootstrap as `string_as_cstr`. Unblocks passing BMB string literals to runtime functions that expect `const char*` — previously broken because `String as i64` cast gave BmbString struct pointer, not the underlying `data` field. stdlib/net wrappers (`tcp_connect`, `tcp_write`, `udp_sendto`) updated to route through it. 3-Stage Fixed Point re-verified after bootstrap changes.

### Cycles 2353-2358 (previous session)

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
| `bench --compare` nightly baseline diff | ✅ Cycle 2365 (`.bench-native-baseline.ndjson` + nightly-bench.yml step, threshold 10%) |
| `@include` in bootstrap | ✅ Cycles 2362-2364 (build/check/run/test/emit-ir entries, Fixed Point preserved) |
| stdlib/net UDP primitive | ✅ Cycles 2367-2372 (udp_bind/sendto/recv/close, full echo E2E) |
| `string_as_cstr` builtin (String → char*) | ✅ Cycle 2371 (unblocks host: String in stdlib/net wrappers) |
| TCP loopback via stdlib/net | ✅ Cycle 2372 (HANDOFF §4 closed) |
| XOR `^` operator (bootstrap) | ✅ Cycle 2354 |
| `stdlib/net` TCP primitive (listen/accept/connect/read/write/close) | ✅ Cycles 2355-2357 (wrappers + Stage 1 smoke; E2E echo server pending) |
| `stdlib/net` ephemeral-port + peer-address accessors | ✅ Cycles 2391-2392 (`tcp_listen_port`, `udp_bind_port`, `tcp_peer_port`, `tcp_peer_host` — getsockname() capture + BmbAsyncSocket accessors) |
| Bootstrap `@annotation pub fn` parse (stdlib/time/fs/io/process @include path) | ✅ Cycle 2394 (hardcoded `121` → `TK_PUB()`, `fn-trust` added to `is_fn_node` — 27 public stdlib fns restored) |
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
| Cross-platform SIMD + net verification (Linux/macOS) | 3-5 cycles | LOW-MEDIUM | Needs push to trigger CI; 144+ local commits ahead of origin. First observation on merge covers `net-echo-smoke` (ubuntu-latest), UDP echo + SIMD still Windows-only |
| **Bootstrap self-parse fragility (Defect 3)** | 2-3 cycles | HIGH | Discovered Cycle 2394-2395. Adding certain new helper fns (esp. with `or`-chained `tok_kind` comparisons) or multi-line comments with `{...}` corrupts Stage 2 self-compilation. Blocks tolerant `skip_contracts` (needed for stdlib/string / array @include) and any non-trivial bootstrap refactor. Root-cause: unknown — hex/token-dump investigation needed. |
| stdlib/string, stdlib/array bootstrap `@include` | 1-2 cycles | LOW | Blocked on Defect 3. Requires `implies` operator support in bootstrap parser OR a tolerant `skip_contracts` recovery path. Rust driver already handles these. |
| Bootstrap SIMD intrinsic CALL-site dispatch | 10+ cycles | HIGH | Stub compile safe (Cycle 2375); Cycle 2387 reconnaissance showed full dispatch requires vector-type awareness in the bootstrap type checker (211 intrinsics × vec-type alloca + call replacement). Silent-correctness limitation documented in `stdlib/simd/mod.bmb` header — bootstrap calls return 0. Workaround: use Rust driver for SIMD. Not a v0.98 blocker. |
| `stdlib/net` TLS extension (`tcp_tls_connect`, `accept_tls`) | 6-10 cycles | MEDIUM-HIGH | Needs OpenSSL binding — new external dependency |
| ~~`stdlib/net` `udp_recvfrom` (peer address exposure)~~ | ~~2-4 cycles~~ | ✅ **완료 (Cycles 2385-2386)** | Runtime `BmbUdpPacket` + 5 accessor 심볼 추가, bootstrap extern 매핑 + stdlib wrapper + smoke 테스트. Multi-client UDP server 가능. |
| Runtime stack trace support (DWARF) | 4-6 cycles | MEDIUM | MIR currently lacks span info — gains limited to function-level unless MIR refactored; reconsider vs ROI |
| ~~`.bit_count()` / `.leading_zeros()` codegen (bootstrap)~~ | ~~1-2 cycles~~ | ✅ **완료 (Cycle 2384)** | `method_to_runtime_fn` + `llvm_gen_call` dispatch에 popcount/clz/ctz/bit_reverse/bswap/bit_not/bit_and/bit_or/bit_xor/bit_shift_left/bit_shift_right 전체 추가. Latent 6건 동시 해소 (bit_and/or/xor/shift_*/bit_not). Fixed Point ✅. |
| ~~CHANGELOG.md reconstruction (v0.67 → v0.98)~~ | ~~3-5 cycles~~ | ✅ **완료 (Cycle 2389)** | Summary blocks added for v0.96.20-v0.96.46, v0.97.0-v0.97.5, v0.98.0; v0.96.1-v0.96.19 per-cycle detail preserved under group header. |
| PyPI wheel publish pipeline | 2-4 cycles | MEDIUM | Packaging ready, needs CI job + secret management |
| ~~Legacy `runtime/runtime.c` removal~~ | ~~1 cycle~~ | ✅ **완료 (Cycle 2383)** | 1088-LOC dead C + 2 orphan scripts (`build_test.ps1`, `validate_llvm_ir.sh`) removed. `find_runtime_c` fallback simplified to `bmb_runtime.c`-only (legacy `bmb_init_argv` API was already incompatible with codegen-emitted `bmb_init_runtime`). |

---

## Structural limits (not planned to change)

| Item | Reason |
|------|--------|
| Z3 verify self-hosting | External SMT solver — IPC-only integration |
| Complete Rust retirement | Maintained as regression gate only |
| LLVM-inherent benchmark gaps (insertion_sort, running_median, max_consecutive_ones) | Identical IR; ISel heuristic differences |

---

For granular history (per-cycle logs, decisions, rejected alternatives), see the internal `claudedocs/cycle-logs/` directory.
