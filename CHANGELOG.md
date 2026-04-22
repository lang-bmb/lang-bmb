# Changelog

All notable changes to BMB (Bare-Metal-Banter) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Work on `v0.98.x` — see cycle logs under `claudedocs/cycle-logs/cycle-2383.md`
and later for per-cycle detail.

### Discovered (Cycle 2418) — Defect 5: `bmb build --shared` fundamentally broken

During an audit of `ecosystem/build_all.py` the wheel pipeline's foundation
turned out to be non-functional for fresh builds. Multiple compiler paths
all fail to produce a working `.dll`:

1. **Inkwell backend** (`cargo build --release --features llvm`):
   `bmb/src/build/mod.rs` line 868 only calls `link_executable` when
   `output_type == Executable`. For `OutputType::SharedLib` the code emits
   the `.o` file, prints `build_success` JSON, and returns — **no link
   step executes**, so no `.dll` is written. bmb.exe's exit code is non-
   zero (2) but the JSON successfully says success — ambiguous reporting
   that let the bug hide.
2. **Text backend** (`cargo build --release` without `--features llvm`):
   has a shared-library link path (`cmd.arg("-shared")`, etc.) but the
   link fails with `multiple definition of 'bmb_is_power_of_two'`.
   The runtime C file `bmb/runtime/bmb_runtime.c` defines
   `bmb_is_power_of_two` / `bmb_next_power_of_two`, and
   `ecosystem/bmb-compute/src/lib.bmb` exports functions with the same
   names via `@export pub fn bmb_is_power_of_two`. When the runtime object
   and the library object are linked into a single `.dll`, the symbols
   collide.
3. **Bootstrap** (`target/bootstrap/bmb-stage1.exe`): fails with
   `lowering produced empty MIR` on `ecosystem/bmb-compute/src/lib.bmb` —
   presumably an `@export`-pipeline incompatibility in the bootstrap
   compiler.

**How this bug stayed hidden**: the five `bmb_*.dll` files in
`ecosystem/bmb-*/` were built months ago (bmb_algo.dll is dated
2026-03-23) and have persisted in the working tree across sessions.
Every wheel build this session (Cycles 2411-2417) copied these **stale**
binaries into the wheels. `pip install` + `import` smoke tests all
passed because the stale `.dll` is still functionally correct — just not
built from current source.

**Impact on wheel CI**: a fresh CI runner has no pre-built `.dll`.
`ecosystem/build_all.py` calls `bmb build --shared`, which silently
produces nothing, and then `shutil.copy2` fails with `FileNotFoundError`.
Consequently the `pypi-publish.yml` workflow introduced in Cycle 2412 and
the wheel gate in `bindings-ci.yml` introduced in Cycle 2417 **will both
fail on the first CI run** until Defect 5 is fixed.

**Scope on current session**: not fixable within remaining budget — a
single-cycle fix attempt in Cycle 2418 (`-static-libgcc` for MinGW
runtime) made the inkwell path regress further and was reverted. The
correct fix likely requires either adding a `SharedLib` link path to
the inkwell backend, resolving the runtime ↔ `@export` symbol collision,
or a combined approach. Planned investigation: dedicated session.

**Related finding**: even when the shared `.dll` builds, it depends on
`libgcc_s_seh-1.dll` (MinGW runtime), which end-user machines without
MSYS2 installed do not have. Either `-static-libgcc` (not yet working
with lld), static linking of libstdc++ (not currently needed, not yet
tested), or bundling the MinGW runtime DLLs in the wheel's
`package_data` would be required for reliable `pip install` on
unmodified Windows.

### Added (Cycles 2411-2412) — PyPI wheel CI pipeline

- **`scripts/build-wheel.sh`** — cross-platform wheel build orchestrator.
  Locates or rebuilds the BMB compiler, runs `ecosystem/build_all.py` to
  produce `.dll` / `.so` / `.dylib`, then `pip wheel . --no-deps` for
  each of the five binding libraries into `dist/wheels/`. Options:
  `--dry-run`, `--lib <name>`, `--skip-compiler`, `--skip-libs`. Exits
  non-zero if any wheel gets tagged `py3-none-any` (would break
  cross-platform install).
- **`.github/workflows/pypi-publish.yml`** — manual-dispatch workflow
  (`workflow_dispatch` only). Matrix Windows + Ubuntu + macOS, each
  builds its own BMB compiler, invokes `build-wheel.sh`, validates wheel
  tags, uploads per-platform artifacts (`wheels-win_amd64`,
  `wheels-linux_x86_64`, `wheels-macosx_10_9_x86_64`). Separate `publish`
  job gated on `inputs.publish=true` + repository choice
  (testpypi/pypi); supports trusted-publishing OIDC + token fallback.

### Fixed (Cycles 2411-2412) — Python packaging correctness

- **Wheel mis-tagging** (Cycle 2411). All five binding libraries
  (`bmb-algo`, `bmb-compute`, `bmb-crypto`, `bmb-text`, `bmb-json`)
  previously produced `py3-none-any` pure-Python wheels despite
  bundling native `.dll` / `.so` / `.dylib` binaries in `package_data`.
  A Linux user pip-installing would have received a Windows DLL.
  Fix: each `setup.py` is now a 30-line shim with:
  - `BinaryDistribution(has_ext_modules=True)` — marks the wheel as
    platform-specific.
  - Custom `bdist_wheel.get_tag()` returning `("py3", "none", plat)` —
    overrides the default `cp3XX-cp3XX-<plat>` tag so any Python 3.x on
    the matching OS can install (the binary is loaded via `ctypes`,
    not CPython ABI).
  Resulting tag per runner: `py3-none-win_amd64`, `py3-none-linux_x86_64`,
  `py3-none-macosx_10_9_x86_64`.
- **`setup.py` / `pyproject.toml` version drift** (Cycle 2411). All five
  `setup.py` files hardcoded `version='0.2.0'` while `pyproject.toml`
  already moved `bmb-algo` and `bmb-crypto` to `0.3.0`. Collapsed to a
  single source of truth: metadata (version, description, classifiers,
  URLs, keywords) now lives entirely in `pyproject.toml`; `setup.py` is
  a minimal shim that only sets the distclass/cmdclass hooks above.

### Added (Cycles 2391-2394)
- **Ephemeral-port discovery for stdlib/net** (Cycle 2391): runtime now
  calls `getsockname()` after `tcp_listen(0)` / `udp_bind(0)` so the
  OS-assigned port is recoverable. New `bmb_async_socket_port(handle)`
  accessor exposed as `tcp_listen_port` / `udp_bind_port` in stdlib/net.
- **TCP peer host/port accessors** (Cycle 2392): `bmb_async_socket_host`
  runtime fn + `tcp_peer_port` / `tcp_peer_host` stdlib wrappers. Reuse
  `BmbAsyncSocket->{host,port}` fields already populated by `tcp_accept`
  from the remote sockaddr.
- **stdlib wrapper smoke coverage** (Cycle 2393):
  `tests/bench/net_stdlib_port_smoke.bmb` validates the full
  `@include → tcp_listen → tcp_listen_port` path at the user API layer.

### Fixed (Cycle 2394)
- **Bootstrap `@annotation pub fn` silently broken** — a hardcoded
  literal `121` at `bootstrap/compiler.bmb:2502` was being compared
  against `tok_kind(tok2)` where `TK_PUB()` is actually `2_000_000_170`.
  Consequence: every `@<anything> pub fn ...` combination fell through
  to the fallback error *"expected 'fn' after @X, got integer literal"*.
  Fix: literal → `TK_PUB()`. Paired fix: added `"fn-trust"` to
  `is_fn_node` so the AST produced by `@trust pub fn ...` is accepted by
  MIR lowering (was silently dropped, yielding *"lowering produced empty
  MIR"*).
  - **Impact**: `@include "stdlib/time/mod.bmb"` / `stdlib/fs` /
    `stdlib/io` / `stdlib/process` now compile via bootstrap. Previously
    27 public stdlib functions could only be reached through the Rust
    driver.
  - **3-Stage Fixed Point** re-verified (S2 == S3) after both fixes.

### Known limitations (Cycle 2394-2395 discovery)
- **Bootstrap self-parse fragility (Defect 3)**: under narrow, not-yet-
  characterised conditions, adding a new helper fn to
  `bootstrap/compiler.bmb` can corrupt Stage 2 self-compilation — either
  producing a misplaced parse error (e.g. the parser decides the file
  header is inside an `if` clause) or exhausting a 16 GB arena allocation.
  Minimal reproduction during Cycle 2395: a 5-line
  `skip_contract_body_tokens` helper with `or`-chained `tok_kind`
  comparisons broke the bootstrap even though the same code was syntactic-
  ally valid, the Rust-built Stage 1 binary compiled cleanly, and a
  trivial arithmetic `fn` at the same location was harmless. Multi-line
  comments containing `{...}` have also triggered the same class of
  failure. Root-cause analysis deferred to a dedicated investigation
  session. **Workaround**: keep bootstrap helper fns minimal and
  single-purpose; prefer inlining over extracting.
- **stdlib/string / stdlib/array via bootstrap `@include`**: blocked on
  the absence of an `implies` operator in the bootstrap parser — contracts
  like `post (len == 0) implies not ret` currently fail `parse_expr`
  inside `skip_contracts`. A token-scan recovery was attempted in Cycle
  2395 but blocked by the fragility above.

### Added (Cycles 2399-2404)
- **Lexer-tolerant `implies` keyword** (Cycle 2402):
  `bootstrap/compiler.bmb:593` now maps the 7-char keyword `implies` to
  `TK_OR` inside `keyword_len7`. The bootstrap parser discards contract
  bodies via `skip_contracts` rather than evaluating them, so parsing
  `implies` as `or` has no effect on compiled output. The Rust driver
  continues to reject `implies` entirely.
  - **Impact**: `@include "stdlib/string/mod.bmb"` and
    `@include "stdlib/array/mod.bmb"` now pass `bmb check` under the
    bootstrap-built binary. The `build` path remains blocked by a
    separate overload-post-injection bug (Defect 4).
- **`tests/bench/stdlib_string_array_include_smoke.bmb`** regression
  guard for the lexer change.

### Cycle 2399-2403 investigation — Defect 3 trigger narrowed
- **20-probe matrix on `bootstrap/compiler.bmb`** established that the
  "bootstrap self-parse fragility" is **deterministic per input**, not
  intermittent. Two independent triggers:
  1. Function body references a parameter via an expression (e.g.
     `fn f(src: String, pos: i64) -> i64 = pos + 1;` exhausts the
     arena; the same signature with body `42` succeeds).
  2. Both parameter names are "long" (e.g.
     `(source: String, position: i64)` fails; changing either to a
     short name succeeds).
- Stage 1 (Rust-built) and Stage 2 (BMB-built) binaries fail
  identically, so the fault is inside `compiler.bmb` itself, not the
  Rust codegen path.
- Full probe log: `claudedocs/cycle-logs/cycle-2399.md` and
  `cycle-2400.md`. The Cycle 2402 lexer tweak succeeded specifically
  because it modifies an existing `if-elif-else` chain inside
  `keyword_len7` rather than introducing a new top-level fn.

### Known limitations (Cycle 2403 discovery)
- **Defect 4 — Overload post-injection substitutes `%ret` but not
  callee parameters**: `inject_post_assumes_in_fn_scan`
  (`bootstrap/compiler.bmb:15702`) runs
  `replace_all_str(raw_ir, "%ret", result_reg)` when emitting a post
  condition at a call site, but callee-parameter references (e.g.
  `%pos` inside `find_trim_start_from`'s `post ret >= pos …`) are left
  dangling. The generated IR then contains `%pos` at a caller scope
  that has no such SSA value, and `opt` rejects it with *"use of
  undefined value '%pos'"*. A correct fix requires AST-level
  substitution of callee params with the actual call-site args, which
  in turn needs at least one new helper fn — blocked by Defect 3.

### Fixed — Defect 4 user-side workaround (Cycles 2406-2409)

Compiler-side Defect 4 fix remains blocked by Defect 3 (both in-place
attempts at `inject_post_assumes_in_fn_scan` re-triggered Stage 2
corruption — see `cycle-2407.md` for quantitative evidence that the
trigger is sensitive to AST complexity, not just new fn additions).
Pivot: weaken problematic post contracts in stdlib sources so the
post-injection never has a param reference to leak.

- **stdlib/string/mod.bmb** (Cycle 2408):
  - `find_trim_start_from.post ret >= pos and ret <= s.len()` →
    `post ret >= 0 and ret <= s.len()`
  - `find_trim_end_from.post ret >= 0 and ret <= pos` →
    `post ret >= 0`
- **stdlib/array/mod.bmb** (Cycle 2409) — 5 fns weakened to remove
  `%len`, `%current_min`, `%current_max` param references from
  post-assume IR:
  - `index_of_i64`, `index_of_i64_from`: `ret < len` → `ret < 8`
    (pre `len <= 8` makes the constant bound valid)
  - `count_i64`: `ret <= len` → `ret <= 8`
  - `min_i64_from`, `max_i64_from`: drop `ret <=/>= current_*` clause
  - `clamp_index`, `wrap_index`: drop `ret < len` clause
- **Regression guards**:
  - `tests/bench/defect4_trim_smoke.bmb` — `find_trim_start` /
    `find_trim_end` build + run (exit 0)
  - `tests/bench/defect4_array_all_smoke.bmb` — 6 fn coverage
    (sum/count/index_of/min/max/clamp_index) build + run (exit 0)
- **Trade-off documented**: contract expressiveness reduced (weaker
  upper/lower bounds); SMT verification still works with the
  weakened forms. Stronger contracts can be restored once Defect 3 is
  root-caused and the proper AST-level param substitution can be
  implemented in the bootstrap.
- **Still unaddressed**: `stdlib/parse/mod.bmb` has 10+ param-ref
  posts. No current `@include "stdlib/parse"` consumers exist, so
  the cleanup is deferred to the point where a test or binding
  actually depends on parse.

### Added — bootstrap + user workaround discovery (Cycles 2406-2409)
- `claudedocs/cycle-logs/cycle-2406.md` — design of in-place Defect 4
  safety check.
- `cycle-2407.md` — **quantitative Defect 3 escalation**: even
  modifying an existing fn body with 3 extra `let`s + 1 nested `if`
  re-triggers Stage 2 corruption. Cycle 2402's 1-line `implies`
  addition was not a generic escape hatch. Implication: Defect 3
  blocks most non-trivial compiler.bmb edits, not only new fn
  additions.
- `cycle-2408.md`, `cycle-2409.md` — user-side workaround execution
  logs.

---

## [v0.98.0] — 2026-04 (Cycles 2300-2388, stdlib/net + tooling)

### Added
- **`stdlib/net` module** (Cycles 2353-2374): TCP + UDP primitives on top of
  `bmb_async_socket_*` + `bmb_async_udp_*` runtime. E2E loopback-verified
  via `scripts/test-net-echo.sh` / `scripts/test-net-udp-echo.sh`.
  Cross-platform smoke via `.github/workflows/ci.yml::net-echo-smoke` on
  ubuntu-latest.
- **`@bench --native` mode** (Cycles 2326-2339): synthesises a bench
  harness, compiles via the standard pipeline, parses stdout NDJSON.
  `bmb_black_box` helper defeats DCE for micro-workloads.
- **`bmb bench --compare`** (Cycles 2341-2351): CI regression gate with
  5-way classification (OK/REG/IMP/MISSING/NEW), threshold default 2%.
  Nightly `@bench` baseline (`.bench-native-baseline.ndjson`) committed on
  `main`; `.github/workflows/nightly-bench.yml` runs the gate on live
  benches at threshold 10%.
- **`@include "path"` via bootstrap** (Cycles 2362-2364): multi-tier
  resolver (source-dir → `BMB_STDLIB_PATH` env → cwd). `string_as_cstr`
  builtin fills the `String → char*` gap for runtime calls taking
  `const char*`.
- **XOR (`^`) operator** (Cycle 2338) via bootstrap parser/AST/MIR/codegen
  (Rule 5 full sweep). Rust compiler frozen per Rule 6.
- **Runtime source auto-sync** (Cycle 2348): `scripts/bootstrap.sh` copies
  `bmb/runtime/bmb_runtime.c` / `.h` into `runtime/` alongside the `.a`,
  preventing the v0.95/v0.98 divergence that had silently broken golden
  tests.
- **i64 bit-op method dispatch in bootstrap** (Cycles 2384-2388):
  `bit_count`, `leading_zeros`, `trailing_zeros`, `reverse_bits`, `bswap`,
  `bit_not`, `bit_and`, `bit_or`, `bit_xor`, `bit_shift_left`,
  `bit_shift_right`, `is_power_of_two`, `next_power_of_two`, `is_prime`
  routed to LLVM intrinsics / native instructions / runtime helpers.
  Previously emitted undefined `@bmb_bit_count` etc.
- **`BmbUdpPacket` API** (Cycles 2385-2386): `udp_recvfrom(sock)` + 5
  accessor wrappers (`udp_packet_payload`/`host`/`port`/`len`/`free`)
  expose peer address for multi-client UDP servers.
- **Bootstrap `todo` / unit-body correctness fix** (Cycle 2375): parser
  routes bare `todo` to `(unit)` + new `fix_typed_ret_placeholders_ir`
  pass rewrites `ret double 0` → `ret double 0.0` etc. after
  identity-copy propagation. Was silent-LINK-FAIL on f64/float/ptr-returning
  stubs.

### Fixed
- `test_golden_file_io_extras` (Cycle 2342): `bmb_delete_file` return
  convention flipped from 1/0 to 0/-1 in v0.98; golden tests realigned.
- Runtime v0.95 ↔ v0.98 divergence (Cycle 2348): see Added above.

### Removed
- Legacy `runtime/runtime.c` + `runtime/build_test.ps1` +
  `runtime/validate_llvm_ir.sh` (Cycle 2383): 1088-LOC dead C referenced
  via a `find_runtime_c` fallback that could never produce a working
  binary — the legacy `bmb_init_argv` API was incompatible with the
  codegen-emitted `bmb_init_runtime`. Fallback search simplified to
  `bmb_runtime.c`-only.

### Verification
- 3-Stage Fixed Point (S2 == S3): 108,609 lines identical, re-verified
  after each bootstrap change (Cycles 2376, 2377, 2382, 2386, 2388).
- `cargo test --release --lib`: 3,764 pass / 0 fail.
- Nightly `@bench --compare` gate: 5/5 OK at threshold 10%.

### Known limitations
- Bootstrap SIMD call-site dispatch is not implemented (Cycle 2387
  reconnaissance). `@include "stdlib/simd/mod.bmb"` compiles but calling
  a SIMD intrinsic through the bootstrap silently returns 0 because the
  bootstrap treats vector types as `i64`. Full parity requires vector-type
  awareness throughout the bootstrap type checker and codegen (tracked in
  `docs/ROADMAP.md`). Workaround: use the Rust driver
  (`cargo build --release --features llvm`) for SIMD code.

---

## [v0.97.0] through [v0.97.5] — 2026-02 to 2026-04 (Cycles ~1900-2300)

### Added — SIMD 1st-class types (Cycles 2215-2330)
- Vector type tokens in the grammar: `f64x{4,8}`, `i32x{4,8}`,
  `i64x{2,4}`, `u32xN`, `u64xN`, `f32x{4,8,16}`, `maskN`.
- Text + inkwell codegen parity (Rule 7) for BinOp / Copy / Call / Return
  on vector types (Cycles 2266-2272).
- `stdlib/simd` module with 211 intrinsic wrappers: splat/hsum/load/store/
  dot/fma/min/max/cmp/blend/shuffle/broadcast_lane (Cycles 2246-2316).
- 2-source shuffle (`slide_left2`/`slide_right2`/`concat_{lo_hi,hi_lo}`)
  — 36 new fns × 11 runtime checks (Cycles 2313-2316).
- `@bench`-attribute driven microbenchmarks + `bmb bench` CLI; `@test`
  attribute-driven discovery unified (Cycle 2237).
- `SIMD_PERF_NOTES.md` user guide (Cycle 2289) — when manual SIMD WINS /
  TIES / LOSES vs auto-vectorization.

### Added — Bindings ecosystem (Cycles 1951-2185)
- `@export` attribute + `--shared` builds producing `.dll` / `.so`, with
  FFI safety via `setjmp` / `longjmp` + TLS.
- 5 binding libraries totalling 140 `@export` functions:
  `bmb-algo` (55), `bmb-compute` (33), `bmb-text` (24), `bmb-crypto` (15),
  `bmb-json` (13). 1,017 pytest + 137 integration + 127 stress + 81 edge.
- C headers × 5 + WASM × 5 (62-289 KB each).
- Packaging infra: `pyproject.toml`, `.pyi` stubs, `__all__`,
  `MANIFEST.in`, CI.

### Added — Generic monomorphization (Cycles 241-360, v0.97.3)
- `fn<T>` / `struct<T>` monomorphization — end-to-end golden tests.
- Nested generics, generic stdlib, `Vec<T>` + `HashMap<K,V>` with
  auto-grow.
- Turbofish call syntax; skip-codegen for unmonomorphized bodies.
- Native generic enum codegen (8 correctness fixes in Cycles 261-280).

### Added — LSP + ecosystem (Cycles ~1700-2100)
- LSP 9 features: diagnostics, hover, completion, definition, document
  symbol, references, rename, formatting, workspace symbol.
- tree-sitter-bmb v0.3.0 (16 new grammar features).
- `gotgan` package manager: dependency resolver, topological build order,
  circular-dep detection; 102+ packages.

---

## [v0.96.20] through [v0.96.46] — 2026-01 to 2026-02 (Cycles 1629-1874)

### Added — Performance sprint (Cycles 1809-1874)
- **v0.96.36-37** (Cycles 1809-1823): for-loop branch weights,
  `gc-sections` dead code elimination, MIR algebraic simplification
  (bitwise / float / shift identities + `PartialEq`).
- **v0.96.38-39** (Cycles 1824-1830): ptr-provenance `noalias` metadata,
  narrowing GEP flow fix.
- **v0.96.40** (Cycles 1831-1834): bootstrap restoration — stack overflow
  fix + build pipeline.
- **v0.96.41** (Cycles 1838-1842): TRL false positive + `speculatable`
  on recursive functions — golden tests 39 → 18 failures.
- **v0.96.42** (Cycles 1843-1844): lambda string-builder encoding fix —
  golden tests 100% (2815/2815).
- **v0.96.43** (Cycles 1846-1849): LLVM attribute enhancement + CSE in
  release + `CopyPropagation` fix.
- **v0.96.44** (Cycles 1854-1859): `CopyPropagation` invalidation +
  malloc narrowing + pipeline fix + `lld gc-sections`.
- **v0.96.45** (Cycles 1860-1864): MIR pattern expansion + bootstrap
  `noundef` + Store-Load Forwarding.
- **v0.96.46** (Cycles 1865-1874): TBAA metadata + inline load/store +
  LLVM 21 `nocapture` fix + bootstrap GEP `inbounds nuw` + `nonnull`
  + inline `main` wrapper. 13-17% ThinLTO overhead removed.

### Verification
- 6,186 Rust tests pass + 2,815/2,815 golden tests.

---

## [v0.96.1] through [v0.96.19] — 2025-12 to 2026-01 (Cycles 1449-1628)

These earlier v0.96.x releases are recorded at per-cycle granularity below.
Topically they cover gotgan + CLI improvements, 72 golden-test additions
for algorithmic coverage, LLVM attribute hardening (`nonnull`, `noalias`,
`dereferenceable(24)`, `nosync`, `memory(read)`, `range()`), and
interprocedural analysis that propagates memory effects across call chains.

### Added (Cycles 1609-1628: v0.96.19)

#### v0.96.19: Interprocedural Analysis + memory(read) + 72 Golden Tests (Cycles 1609-1628)

**Compiler Improvements (Cycles 1609, 1612)**:
- **memory(read) LLVM attribute** (Cycle 1609): Both backends
  - Three-tier hierarchy: `memory(none)` > `memory(read)` > no annotation
  - Functions that read but don't write memory get `memory(read)`
  - Enables dead store elimination across calls, better alias analysis, LICM
- **Interprocedural memory effect analysis** (Cycle 1612): Both backends
  - Two-phase analysis: intraprocedural pass + interprocedural fixpoint loop
  - Functions calling only pure functions now correctly get `memory(none)`
  - Functions reading memory and calling only non-writing functions get `memory(read)`

**Golden Tests**: +72 new tests (352 → 415 total, 9 remaining = 8 test + 1 verification)
- Cycles 1610-1611: 16 tests (kadane, histogram, jump game, matrix rotate, catalan paths, RGB pack, pancake sort, matrix minor, digit DP, bit permute, tower hanoi, binary search, prefix sum, window stats, longest plateau)
- Cycles 1613-1616: 16 tests (coin change, Josephus, insertion sort, sqrt decomp, trap water, LIS, matrix chain, string period, max subseq sum, radix convert, merge intervals, game of life, Chinese remainder, convex hull, topological sort, sparse table)
- Cycles 1617-1620: 16 tests (majority element, stock profit, cycle detect, string compress, bucket sort, median find, longest common, matrix spiral, xorshift RNG, magic square, binary heap, interval scheduling, nim game, polynomial, Knuth shuffle, prime factor)
- Cycles 1621-1622: 8 tests (binomial coefficients, day of week, matrix exponentiation, inversion count, combination sum, integer sqrt, array rotate, histogram area)
- Cycles 1623-1626: 16 tests (derangement, matrix LU, ring buffer, color mix, eight queens, Huffman frequency, Bellman-Ford, fast power, max flow, string hash, bigint add, Karatsuba, bridges, stable matching, prefix function, SCC)

**Verification**: 6,186 Rust tests PASS, 72/72 new golden tests PASS

### Added (Cycles 1589-1608: v0.96.18)

#### v0.96.18: LLVM Attributes + range() + 56 Golden Tests (Cycles 1589-1608)

**E-4: LLVM Attribute Enhancements (Cycles 1589-1590, 1598, 1601)**:
- **dereferenceable(24) on String params/returns** (Cycle 1589): Both backends
  - BmbString = { ptr, i64, i64 } = 24 bytes, always valid
  - Enables speculative loads, dead store elimination
- **align(8) on String params** (Cycle 1590): Both backends
  - BmbString fields all 8-byte aligned, malloc returns 8-byte aligned
  - Enables aligned vector loads
- **nosync on all user functions** (Cycle 1598): Both backends
  - BMB user functions never use atomic operations or synchronization
  - Enables LICM, better alias analysis, aggressive reordering
- **range() return attribute from postconditions** (Cycle 1601): Text backend
  - `post ret >= 0` → `range(i64 0, MIN)` (non-negative)
  - `post ret >= 0 and ret <= 1` → `range(i64 0, 2)` (boolean-like)
  - Enables vectorization trip count proof, nsw inference, branch elimination

**Golden Tests**: +56 new tests (296 → 352 total)
- Cycles 1591-1597: 28 tests (various algorithms)
- Cycles 1599-1607: 28 tests (bit manipulation, number theory, data structures, sorting)

**Verification**: 6,186 Rust tests PASS, all golden tests PASS

### Added (Cycles 1579-1588: v0.96.17)

#### v0.96.17: LLVM Attributes + 31 Golden Tests (Cycles 1579-1588)

**E-4: LLVM Attribute Enhancements (Cycles 1583-1584)**:
- **noalias+nocapture+readonly on String params** (Cycle 1583): Both backends
  - BMB strings are immutable — safe for noalias
  - Enables: LICM, GVN, load/store forwarding optimizations
- **nonnull return for String functions** (Cycle 1584): Both backends
  - BMB String functions always return valid (non-null) pointers
  - Enables LLVM null pointer elimination

**Golden Tests**: +31 new tests (264 → 295 total)
- Cycle 1579: contract_chain, range_proof, sliding_max (3)
- Cycle 1580: rle_codec, prefix_query, char_ops, rpn_calc (4)
- Cycle 1581: matrix_transform, poly_arith, ring_buffer, hashmap_sim (4)
- Cycle 1582: register_vm, cellular_1d, sparse_vec, expr_tree (4)
- Cycle 1585: coord_math, roman_conv, statistics, bit_field (4)
- Cycle 1586: interval_merge, base_convert, run_length, newton_sqrt (4)
- Cycle 1587: dutch_flag, matrix_det, stack_machine, median_find (4)
- Cycle 1588: version bump + 20-cycle summary (4)

**Verification**: 6,186 Rust tests PASS, all golden tests PASS
- **Version**: v0.96.16 → v0.96.17

### Added (Cycles 1569-1578: v0.96.16)

#### v0.96.16: Contract→Performance Pipeline (Cycles 1569-1578)

**EXISTENTIAL Priority**: Closing the gap where contracts exist but don't improve performance.

**E-1: llvm.assume Generation (Cycles 1569-1571)**:
- **Text backend** (Cycle 1569): `pre` conditions → `call void @llvm.assume(i1)` at function entry
  - VarCmp, VarVarCmp, NonNull facts converted to icmp + assume
- **Inkwell backend** (Cycle 1570): Same assume generation via LLVM C API
- **Impact measurement** (Cycle 1571): Verified assumes enable LLVM range analysis

**E-2: Runtime Overhead Elimination (Cycles 1572-1577)**:
- **nonnull attributes** (Cycles 1572-1573): String pointer parameters guaranteed non-null
  - Text backend: 20+ functions with `nonnull` on string params/returns
  - Inkwell backend: 35 nonnull annotations across 20 string functions
- **Saturating arithmetic elimination** (Cycle 1574): SaturatingArithmeticElimination pass
  - Constant folding: saturating_add/sub/mul at compile time
  - Algebraic simplification: x +| 0 → x, x *| 1 → x, etc.
  - Range-based proof: `pre x >= 0 and x <= 1000` → `+|` becomes `add nsw`
  - 3/5 test functions optimized (bounded ranges proven safe)
- **noundef attribute** (Cycle 1575): All function parameters marked `noundef` (both backends)
  - BMB guarantees all values are initialized — no poison/undef
- **Division range proof** (Cycle 1575): DivisionCheckElimination enhanced
  - Range bounds prove non-zero: `pre b >= 1` → zero excluded → division safe
- **Postcondition propagation** (Cycle 1576): Interprocedural `post` condition facts
  - `post ret >= 0` in callee → `VarCmp >= 0` in caller for result variable
  - Enables downstream MIR passes to use callee return value guarantees
- **Postcondition llvm.assume** (Cycle 1577): `llvm.assume` emitted after call sites
  - `abs(x) post ret >= 0` → caller gets `llvm.assume(result >= 0)` after each call
  - Full contract→LLVM pipeline for both pre and post conditions

**E-4: LLVM Attributes (Cycle 1575)**:
- `noundef` on all parameters (enables LLVM poison analysis)

**Golden Tests**: +3 contract→performance tests
- `test_golden_sat_contract.bmb` — saturating arithmetic with bounded contracts
- `test_golden_div_contract.bmb` — division safety with range contracts
- `test_golden_post_prop.bmb` — postcondition propagation across calls

**Verification**: 6,186 Rust tests PASS, all golden tests PASS
- **Version**: v0.96.15 → v0.96.16

### Added (Cycles 1549-1568: v0.96.15)

#### v0.96.15: LLVM Optimization + 62 Golden Tests (Cycles 1549-1568)

**Compiler Optimizations (Cycles 1549-1555)**:
- **Private linkage** (Cycle 1549): All 1,277 non-main functions → `define private`
  - After opt -O2: 1,284 → 362 functions (72% eliminated by LLVM)
- **Aggressive inlining** (Cycle 1550): 285 `alwaysinline` + 568 `inlinehint`
- **nofree attribute** (Cycle 1551): 184 memory-annotated functions
- **Tail call annotation** (Cycle 1555): 650 self-recursive calls → `tail call`
  - LLVM promotes to `fastcc` with tail call elimination

**Golden Tests (Cycles 1553-1567)**: 202 → 264 total (+62 tests)
- Cycles 1553-1554: fibonacci, prime sieve, binary search, GCD, sorting, power, roman numerals, RLE, Pascal's triangle, Josephus, matrix determinant, Catalan, Hanoi, digit sum
- Cycles 1557-1558: prime factorization, Fibonacci matrix, perfect numbers, base conversion, merge/heap/counting sort, string distance, modular inverse, interval scheduling, LIS, matrix multiply, convex hull, bigint, radix sort, Knuth shuffle
- Cycles 1559-1562: sieve of Eratosthenes, Huffman tree, postfix evaluator, maze solver, polynomial eval, magic square, sparse matrix, Euler totient, topological sort, bit manipulation, Dutch national flag, matrix chain, expression tree, LIS patience, Gray code, sliding window, flood fill, run length
- Cycles 1564-1567: Z-algorithm, segment tree, knapsack, edit distance, Fenwick tree, LCS, counting inversions, next permutation, quicksort, Dijkstra, balanced parens, subset sum, selection sort, longest palindrome, rotate array, majority element

**Verification**: 6,186 Rust tests PASS, 233/264 golden PASS (23 pre-existing failures + 8 new native-only)
- **3-Stage Fixed Point**: 90,153 lines IR
- **Version**: v0.96.12 → v0.96.15

### Previously Released (Cycles 1549-1552: v0.96.13)
- See v0.96.15 above (consolidated)

### Added (Cycles 1546-1548: v0.96.12)

#### v0.96.12: Final Verification + Golden Tests (Cycles 1546-1548)
- **Full verification run** (Cycle 1546): 6186/6186 Rust tests PASS, 177/200 golden PASS (23 pre-existing)
  - Fixed CRLF line endings in all `.bmb.out` files
  - Confirmed all 34 new golden tests from this session: 34/34 PASS
- **2 new golden tests** (Cycle 1547): compiler_stress (deep calls/loops/vec), algorithm_zoo (search/Dutch flag/RLE/sparse matrix)
- **20-cycle run summary** (Cycle 1548): Cycles 1529-1548
  - Compiler improvements: interprocedural memory(read/none), speculatable attributes
  - Golden tests: 166 → 202 (+36 tests, ~900+ new test cases)
  - Version: v0.96.8 → v0.96.12
- **Version**: v0.96.11 → v0.96.12

### Added (Cycles 1539-1545: v0.96.11)

#### v0.96.11: 200 Golden Tests Milestone (Cycles 1539-1545)
- **19 new golden tests** (181→200): cipher_patterns, sorting_advanced, closure_advanced, hash_compute, iterator_sim, graph_basic, probability_sim, stack_machine_adv, geometry_compute, calendar_compute, ring_buffer, automata_sim, number_base, tree_sim, physics_sim, scheduler_sim, expression_eval, memory_patterns, milestone_200
  - Cipher patterns: ROT13, XOR cipher, Atbash, rail fence, hex encoding (Cycle 1539)
  - Advanced sorting: selection/insertion/merge sort, Lomuto partition, counting sort (Cycle 1539)
  - Advanced closures: pipeline, composition, reduce, conditional closures (Cycle 1539)
  - Hash compute: DJB2, FNV-1a, Adler-32, checksums (Cycle 1540)
  - Iterator sim: range/filter/zip/chain/enumerate patterns (Cycle 1540)
  - Graph basic: BFS, DFS, connected components via adjacency list (Cycle 1540)
  - Probability: mean, variance, median, mode, percentile, LCG (Cycle 1541)
  - Stack machine: postfix eval, Fibonacci via stack, palindrome (Cycle 1541)
  - Geometry: distance, shoelace area, cross product, collinearity (Cycle 1541)
  - Calendar: leap year, Sakamoto day-of-week, day-of-year (Cycle 1542)
  - Ring buffer: wrap-around, drain/refill, circular average (Cycle 1542)
  - Automata: DFA/NFA simulation, binary div-by-3, accepting paths (Cycle 1542)
  - Number base: palindrome, narcissistic, digital root (Cycle 1543)
  - Tree sim: BST insert/search/height/min/max via parallel arrays (Cycle 1543)
  - Physics: free fall, collision, kinetic energy, spring PE (Cycle 1543)
  - Scheduler: priority queue, round-robin, FCFS, SJF (Cycle 1544)
  - Expression eval: Horner, Newton, continued fraction, Chebyshev (Cycle 1544)
  - Memory patterns: pool/arena/slab allocator, reference counting (Cycle 1544)
  - **Milestone 200**: Comprehensive showcase of all BMB features (Cycle 1545)
- **~600+ new test cases** across all golden tests
- **200 Golden Tests milestone reached**
- **Version**: v0.96.10 → v0.96.11

### Added (Cycles 1533-1538: v0.96.10)

#### v0.96.10: Golden Tests Expansion (Cycles 1533-1538)
- **15 new golden tests** (166→181): counter_patterns, nested_control, math_compute, type_convert, bit_manipulation, string_basic, simulation_patterns, array_advanced, recursion_patterns, fp_patterns, numeric_algorithms, game_logic, matrix_advanced, string_algorithm, dp_patterns
  - Counter/accumulator patterns: running sum, factorial, Fibonacci (Cycle 1533)
  - Nested control flow: 4-level nested if-else, nested for with break (Cycle 1533)
  - Math computations: GCD, LCM, isqrt, modpow, totient (Cycle 1533)
  - Type conversions: i64↔f64, as cast, precision (Cycle 1534)
  - Bit manipulation: popcount, reversal, rotate, pack/unpack (Cycle 1534)
  - String basics: len, byte_at, slice, concat, find (Cycle 1534)
  - Simulation: LCG random, population, bank account, FSM (Cycle 1535)
  - Array advanced: sort, binary search, prefix sum (Cycle 1535)
  - Recursion: Hanoi, Catalan, McCarthy 91 (Cycle 1535)
  - FP patterns: map, filter, reduce, composition, pipeline (Cycle 1536)
  - Numeric algorithms: Newton, bisection, Taylor series (Cycle 1536)
  - Game logic: tic-tac-toe, nim, RPS (Cycle 1536)
  - Matrix: multiply, determinant, transpose (Cycle 1537)
  - String algorithms: palindrome, Caesar cipher, RLE (Cycle 1537)
  - DP patterns: knapsack, Kadane, LIS, coin change (Cycle 1537)
- **~330 new test cases** across all golden tests
- **Version**: v0.96.9 → v0.96.10

### Added (Cycles 1529-1532: v0.96.9)

#### v0.96.9: Interprocedural Memory & Speculatable Analysis (Cycles 1529-1532)
- **Interprocedural memory(read) annotation** (Cycle 1529): Fixpoint analysis propagating memory(read) across function call boundaries — 35→52 functions (+17, 48.6% increase)
  - 3-phase algorithm: collect known readonly → check candidates → fixpoint iteration (max 5 rounds)
  - Scans both `declare` and `define` lines for known readonly functions
  - 17 newly annotated: compound_op, is_hex_digit, keyword_len2-10, tok_kind, tok_kind_name, etc.
- **Interprocedural memory(none) detection** (Cycle 1530): Pure function detection via fixpoint — 134→141 memory(none) functions (+7)
  - 5 functions upgraded from memory(read) to memory(none) (stricter annotation)
  - Total annotated: 169→189 (+20 functions, 11.8% increase)
- **Speculatable attribute** (Cycle 1531): Added LLVM `speculatable` to 127 pure functions that cannot trap
  - Excludes functions with sdiv/udiv/srem/urem (can trap on div-by-zero)
  - Enables LLVM to speculate execution before branch resolution
- **3-Stage Fixed Point**: 89,177 lines IR
- **Version**: v0.96.8 → v0.96.9

### Added (Cycles 1524-1528: v0.96.8)

#### v0.96.8: Golden Tests Expansion (Cycles 1524-1528)
- **14 new golden tests** (158→165): complex_expr, f64_ops, scope, multi_fn, bitwise_ext, kv_store, newton_approx, linked_list_sim
  - Complex expressions: max3/min3/median3, sorted checks, triangle classification (Cycle 1524)
  - Float operations: f64 arithmetic, conversions, loop accumulation, power (Cycle 1524)
  - Scope/shadowing: flat scoping behavior documented, block scope, for scope (Cycle 1525)
  - Multi-function computation: square/cube, predicates, collatz, digital_root (Cycle 1525)
  - Bitwise extended: popcount, power-of-two, lowest-set-bit, toggle-bit (Cycle 1526)
  - KV store: parallel array key-value storage, lookup, update, existence check (Cycle 1526)
  - Newton's method: isqrt, icbrt, binary search, geometric series, fixed-point iteration (Cycle 1527)
  - Linked list simulation: traverse, insert, delete via parallel arrays (Cycle 1527)
- **Key discovery**: BMB interpreter has flat scoping (inner `let` modifies outer scope)
- **Version**: v0.96.7 → v0.96.8

### Added (Cycles 1514-1523: v0.96.7)

#### v0.96.7: Golden Tests + Compiler Quality (Cycles 1514-1523)
- **f64_min/f64_max codegen fix**: Added dual-name intrinsic support (`@f64_min` + `@bmb_f64_min`) (Cycle 1515)
- **memory(read) expansion**: Removed overly conservative `inttoptr` from write detection — 10→35 memory(read) functions (Cycle 1520)
- **Runtime parse annotations**: Added `memory(argmem: read)` to `bmb_parse_int`/`bmb_parse_f64` (Cycle 1521)
- **14 new golden tests** (150→158): wrapping_arith, match_expr, int_intrinsics, logical_shift, float_intrinsics, math_intrinsics, struct_ops, enum_match, closure, loop_for, array_ops, return_stmt, saturating_edge, string_proc, recursive
- **3-Stage Fixed Point**: 87,821 lines IR
- **Version**: v0.96.6 → v0.96.7

### Added (Cycles 1509-1512: v0.96.6)

#### v0.96.6: Benchmark Improvements + Error Cleanup (Cycles 1509-1512)
- **Fannkuch benchmark 0.78x FASTER**: Stack arrays + correct swap algorithm beats Clang -O3 by 22% (Cycle 1509)
  - Previous heap version was 1.06x OK; stack+swap version is 0.78x FASTER
  - Root cause: heap allocation overhead + wrong algorithm (O(n) rotate vs O(1) swap) in prior versions
- **Parse error position propagation**: Fixed 6 missing `is_error()` checks after `parse_block_stmts()` calls (Cycle 1511)
  - Before: `error[parse]: expected '}' to close block at line 1:1` (wrong position)
  - After: `error[parse]: expected ';' after let binding at line 5:15` (correct position)
- **Error message cleanup**: Removed redundant "PARSE:" prefix from error display (Cycle 1512)
  - `compile_program()` now passes parse errors through as-is
  - `compile_file_to()` uses proper `print_compile_err()` formatting
- **Benchmark status**: 7 FASTER, 12 PASS, 2 OK, 0 WARN/FAIL vs Clang -O3
- **Version**: v0.96.5 → v0.96.6

### Added (Cycles 1489-1501: v0.96.4-v0.96.5)

#### v0.96.4: New Operators + LLVM Intrinsics (Cycles 1489-1499)
- **Saturating arithmetic operators**: `+|`, `-|`, `*|` — clamped to i64 min/max on overflow (Cycle 1490)
  - Uses `@llvm.sadd.sat.i64`, `@llvm.ssub.sat.i64`, and `smul.with.overflow` + select
- **Wrapping arithmetic operators**: `+%`, `-%`, `*%` — two's complement wrap on overflow (Cycle 1491)
  - Generates LLVM `add`/`sub`/`mul` without `nsw` flag
- **Logical right shift operator**: `>>>` — zero-fill right shift for unsigned semantics (Cycle 1492)
  - Generates LLVM `lshr` instead of `ashr`
- **30 LLVM intrinsics** replacing C runtime calls — single hardware instructions, zero function call overhead:
  - Integer: `popcount`, `clz`, `ctz`, `bit_reverse`, `abs`, `min`, `max`, `clamp`, `bswap`, `rotate_left`, `rotate_right` (Cycles 1493-1496)
  - Float: `fabs`, `floor`, `ceil`, `round`, `sqrt`, `f64_min`, `f64_max` (Cycle 1498)
  - Math: `sin`, `cos`, `tan`, `atan`, `atan2`, `log`, `log2`, `log10`, `exp`, `pow_f64` (Cycle 1499)
  - Special: `fmod` → LLVM `frem` instruction (Cycle 1499)
- **Golden tests**: `float_intrinsics` (10 cases), `math_intrinsics` (10 cases), `saturating_arith`, `wrapping_arith`
- **Version**: v0.96.3 → v0.96.5

#### v0.96.5: Error Diagnostics + Compile Stats (Cycles 1497, 1500-1508)
- **Type error messages**: 9 messages improved with actual type information in `types.bmb` (Cycle 1497)
- **`tok_kind_name` helper**: 33 token kinds → human-readable names for error messages (Cycle 1500)
- **93/93 parse error messages** improved to show actual token found (e.g., "expected identifier after 'let', got integer literal"):
  - let/fn/if/for/match/set (12 messages, Cycle 1500)
  - assert/dbg/tuple/while/loop/match arms (15 messages, Cycle 1501)
  - set/lambda/array/arguments/if-else (15 messages, Cycle 1504)
  - for/struct/fn params/match patterns (15 messages, Cycle 1505)
  - Block/const/wildcard/remaining (34 messages, Cycle 1506) — **100% coverage**
- **`--stats` flag**: `bmb build file.bmb --stats` shows function count, declarations, string constants, IR lines (Cycle 1503)
  - Uses while-loop counting (O(1) stack) with large file guard (>500KB → byte count only)

### Added (Cycles 1477-1488: v0.96.3 CLI Improvements)
- **`run` command**: Compile-and-execute workflow (`bmb run <file>`) (Cycle 1477)
- **`test` command**: Compile, run, compare output against `.bmb.out` expected files (Cycles 1482-1483)
  - Single file: `bmb test <file>` — PASS/FAIL/SKIP results
  - Directory batch: `bmb test <dir>` — summary with pass/fail/skip counts + elapsed time
- **Build timing**: Compile and link phase timing displayed after successful builds (Cycles 1478-1479)
- **Error classification**: Errors categorized as parse/resolve/type/compile (Cycle 1480)
- **JSON version**: `bmb --version --json` for tooling integration (Cycle 1481)
- **Directory `check`**: `bmb check <dir>` type-checks all .bmb files with summary (Cycle 1486)
- **7 new golden tests**: early_return, chained_ops, bool_logic, string_concat, recursive_math, fmt_contracts.out, lint_target.out (Cycle 1485)
- **Version**: v0.96.2 → v0.96.3

### Fixed (Cycles 1477-1488)
- **Clang warnings**: Added `-w` flag to all 10 clang invocations to suppress "overriding module target triple" warnings (Cycle 1484)
- **Temp file cleanup**: `run` command now cleans up temporary `.ll` and `.exe` files after execution (Cycle 1479)

### Added (Cycles 1469-1476: v0.96.2 Dev Tools)
- **Linter**: `bmb lint <file|dir>` — 5 static analysis checks (unused/params/naming/complexity/recursive) (Cycle 1469)
- **Formatter**: `bmb fmt <file|dir> [--check]` — source-level code formatting, 3-state machine (Cycles 1470-1471)
- **REPL**: `bmb repl` — interactive expression evaluator with String support (Cycles 1472-1473)
- **Golden test formatting**: 59 files reformatted, idempotency verified (Cycle 1474)

### Fixed (Cycles 1469-1476)
- **Runtime name collision**: User-defined functions (abs, min, max, parse_int) now correctly shadow builtins in interpreter (3 paths) and text codegen (Cycle 1475)

### Added (Cycles 1449-1468: v0.96.1 gotgan-bmb)
- **gotgan-bmb**: Complete BMB package manager (847 LOC, 15 commands, 256KB native binary)
  - Commands: add, build, check, clean, deps, fmt, init, lint, new, remove, run, test, verify, version, help
  - TOML parser (zero-copy, position-based), dependency management, package lifecycle
- **Runtime filesystem APIs**: `is_dir()`, `make_dir()` (recursive mkdir -p), `list_dir()`, `remove_file()`, `remove_dir()`
- **Golden tests**: 6 new test files (42 tests) — dir_ops, gotgan_deps, gotgan_init, gotgan_new, gotgan_test, gotgan_add, fs_ops
- **Native compilation demo**: Multi-feature program (fibonacci, primes, gcd, collatz, string ops, file I/O)

### Fixed (Cycles 1449-1468)
- **MIR ConstantFolding**: Critical stale constant propagation bug — variables reassigned to non-constant values retained old constant in propagation map (Cycle 1466-1467)
  - Affected 10 instruction types: BinOp, UnaryOp, Copy, Call, IndexLoad, StructInit, FieldAccess, EnumVariant, ArrayInit, Phi
  - Caused incorrect native compilation of mutable variable patterns
- **Runtime `_mkdir`**: MinGW compatibility — use `mkdir()` from `<direct.h>` instead of `_mkdir` (Cycle 1462)
- **gotgan `cmd_new`**: Create parent directory before subdirectory (Cycle 1463)

### Fixed (Cycles 1429-1439)
- **ProofUnreachableElimination**: Unsound bool folding when variable has multiple defs (Cycle 1430, 1434)
- **AlgebraicSimplification**: Unsound sdiv/srem power-of-2 optimization for negative dividends (Cycle 1431)
- **TailCallOptimization**: Phi-based tail call skipping intervening instructions (Cycle 1432)
- **IfElseToSelect**: Live variable corruption when branches assign different loop variables (Cycle 1435)
- **LoopBoundedNarrowing**: Shl not detected as multiplication, causing i32 overflow (Cycle 1437)

### Improved
- **Golden tests**: 66 → 123 (100% pass rate on compiled execution)
- **3-Stage Fixed Point**: Verified after all fixes (23.5s)
- **G-5 milestone**: 102/102 ecosystem packages compile and execute correctly
- **GEP optimizations**: sieve 1.37x→0.96x, matrix_multiply 1.09x→0.97x (Cycles 1443-1445)
- **Interprocedural index-param narrowing guard**: Prevents shl+ashr patterns in array-indexed loops (Cycle 1448)

## [0.67.0] - 2026-02-05 (Release Candidate)

### Added

- **Dogfooding V Complete (v0.66)**: All BMB tools and packages verified through circular build
  - bmb-test: 821 tests passing (parser, selfhost, lexer, codegen, error)
  - 7 tools type-check successfully (bmb-test, bmb-bench, bmb-fmt, bmb-lint, bmb-doc, bmb-check, gotgan-bmb)
  - 11/13 packages compile (bmb-core, bmb-string, bmb-array, bmb-io, bmb-test, bmb-process, bmb-json, bmb-http, bmb-regex, bmb-iter, bmb-traits)

- **New packages (v0.66)**:
  - `bmb-json`: JSON parser and serializer (587 LOC)
  - `bmb-http`: HTTP client using curl backend (370 LOC)
  - `bmb-regex`: Backtracking regex engine with quantifiers, character classes, anchors (444 LOC)

- **Package registry (v0.65)**: 14 official packages in packages/INDEX.toml
  - Core: bmb-core, bmb-traits, bmb-option, bmb-result
  - Data: bmb-string, bmb-array, bmb-iter, bmb-json
  - System: bmb-io, bmb-process, bmb-runtime
  - Testing: bmb-test
  - Network: bmb-http, bmb-regex

- **Updated documentation**:
  - API reference for all packages (docs/api/)
  - Updated GETTING_STARTED.md with package-based stdlib usage
  - Practical tutorial example (config processor)

### Changed

- **Stdlib separation (v0.65)**: Standard library reorganized into separate packages
  - Each package has its own gotgan.toml manifest
  - Packages can be used independently

### Fixed

- **Bootstrap compiler**: compiler.bmb type-checks with 606 warnings
- **Performance**: All Tier 1 benchmarks within 2% threshold

### Known Issues

- `bmb-option` and `bmb-result` packages use nullable type syntax (`T?`) not yet supported by bootstrap compiler
- These packages will be fixed in v1.0 release

## [0.51.1] - 2026-01-21

### Added

- **Phi-based tail call optimization** (57.P1): Detect `Call → Goto → Phi → Return` patterns in MIR
  - TCO now works for tail calls in conditional branches (BMB's if-else expression pattern)
  - fannkuch improved from 2.12x → 1.59x C

- **Compile-time string constant folding** (57.P2): Evaluate string operations at compile time
  - `"Hello" + " " + "World"` → single `"Hello World"` constant
  - `chr(65)` → `"A"`, `chr(13) + chr(10)` → `"\r\n"` at compile time
  - Reduces runtime allocations in string-heavy code

- **Runtime hashmap implementation** (v0.50.64): Native hashmap with runtime functions
  - `hashmap_new()`, `hashmap_get()`, `hashmap_set()`, `hashmap_contains()`, `hashmap_remove()`

### Changed

- **Runtime compiled with matching optimization level** (v0.51): `runtime.c` now uses same `-O` flag as BMB code
  - Previous: runtime always compiled with `-O0` → 30% FFI overhead
  - Now: `--aggressive` → `-O3` for both BMB and runtime
  - FFI-heavy benchmarks improved significantly

- **Loop grammar improvement** (57.P12): `while`/`for`/`loop` body now accepts direct assignment
  - Previous: required nested block `while x < 10 { { x = x + 1; () } }`
  - Now: `while x < 10 { x = x + 1 }` works directly

### Fixed

- **vec_push PHI node bug** (57.P9): Replace inline codegen with runtime call
  - Inline blocks broke PHI predecessor tracking
  - brainfuck improved from 2.92x (interpreter fallback) → 1.24x C

- **file_exists type mismatch** (57.P10): Return type is `i64`, not `bool`
  - Caused segfault when return value used in conditionals

- **Loop-safe MIR optimization** (v0.50.72): Fix constant folding/copy propagation in loop contexts
  - Prevents incorrect optimization of loop variables

- **malloc/free pointer type handling** (v0.50.72): Proper LLVM IR type emission for memory operations

### Performance

- **37/48 (77%) benchmarks ≤1.10x C** (v0.51)
- **26 benchmarks FASTER than C** (hash_table 0.45x, n_body 0.22x)
- 11 SLOW benchmarks have documented root causes (language design decisions, not bugs)

## [0.50.27] - 2026-01-17

### Changed

- **Bootstrap parser StringBuilder optimization**: `parse_program_sb` now uses StringBuilder for O(n) program AST accumulation
  - Previously O(n²) due to string concatenation in recursive `parse_program`
  - Stage 1 build time: ~0.9s (unchanged, Rust compiler is already optimized)
  - Stage 2 still blocked by remaining O(n²) patterns in `parse_fn`, `parse_args`, `parse_params`

### Known Issues

- **Stage 2 Bootstrap**: Self-compilation still takes >5 minutes due to:
  1. O(n²) string concatenation in individual function parsing (`parse_fn`)
  2. O(n²) string concatenation in argument/parameter parsing
  3. Deep recursion causing stack issues on default stack size (requires `ulimit -s unlimited`)
  4. Segfault on default stack, timeout on unlimited stack

## [0.50.26] - 2026-01-17

### Added

- **Array reference indexing**: Support indexing through references to arrays (`&[T; N]`)
  - Type checker now accepts `arr[idx]` where `arr: &[T; N]`
  - Interpreter automatically dereferences before indexing
  - Also supports string reference indexing (`&String`)
  - Resolves ISSUE-20260117-array-reference-indexing

### Changed

- Type checker match on `Type::Ref(inner)` for index expressions
- Interpreter `eval` and `eval_fast` dereference `Value::Ref` before indexing
- Added 3 integration tests: `test_array_ref_index`, `test_string_ref_index`, `test_invalid_ref_index`

## [0.50.25] - 2026-01-17

### Added

- **LSP local variable support**: Language server now provides hover and completion for local variables
  - `LocalVar` struct tracks local variable name, type, definition span, and scope span
  - `collect_locals()` recursively collects let bindings, for loop variables, and closure parameters from AST
  - `get_locals_at_offset()` finds visible locals at cursor position using scope-based filtering
  - Hover shows local variable types with "(local)" annotation
  - Completion includes local variables with higher sort priority than keywords

### Changed

- `DocumentState.locals` field added to cache collected local variables
- `collect_symbols()` now returns locals as third tuple element
- Completion items use sort text prefixes (`!0`, `!1`, `!2`) for priority ordering

## [0.50.24] - 2026-01-17

### Added

- **Proof verification query** (Task 47.7-47.8): `bmb q proof` queries verification results
  - Filters: `--unverified`, `--failed`, `--timeout`
  - Shows Z3 availability and version
  - Summary stats: total, verified, failed, timeout, unknown, pending
  - Counterexample display for failed verifications
- **Proof index generation**: `bmb verify` now saves results to `.bmb/index/proofs.json`
  - Tracks pre/post verification status per function
  - Records verification time and timestamp
  - Stores counterexamples for failed verifications

### Changed

- `ProofIndex`, `ProofEntry`, `ProofStatus` types added to `bmb::index` module
- `query_proofs()` function added to `bmb::query` module

## [0.50.23] - 2026-01-17

### Added

- **Cross-compilation target flag** (Task 48.3-48.4): `bmb build --target <triple>` generates LLVM IR for specified target
  - Supports target triples: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`
  - LLVM IR can be compiled on target platform with clang
  - WASM backend verified working on Windows

### Changed

- `BuildConfig` now supports `target_triple` field for cross-compilation
- `TextCodeGen` uses custom target triple when specified

## [0.50.22] - 2026-01-17

### Added

- **HTTP Query Server** (Task 50.7): `bmb q serve` starts an HTTP server for AI tools
  - Endpoints: `GET /health`, `GET /metrics`, `POST /query`
  - Supports all query types: sym, fn, type, metrics, deps, contract, impact
  - JSON request/response format
  - Default: `127.0.0.1:3000`, configurable with `--host` and `--port`
  - No additional dependencies (uses std::net::TcpListener)

## [0.50.21] - 2026-01-17

### Added

- **Index watch mode** (v0.50.8): `bmb index --watch` now monitors .bmb files for changes and re-indexes automatically
  - Uses `notify` crate for cross-platform file system watching
  - 500ms debouncing to avoid rapid re-indexing during saves
  - Only re-indexes when .bmb files change (ignores other file types)
  - Graceful error handling during re-indexing

### Dependencies

- Added `notify = "8"` for file system event watching
- Added `notify-debouncer-mini = "0.6"` for event debouncing

## [0.50.20] - 2026-01-17

### Added

- **Formatter comment preservation** (v0.45.5): `bmb fmt` now preserves comments in source files
  - File-level comments (before first item)
  - Function-level comments (before each function/struct/enum)
  - Both `//` and `--` (legacy) comment styles supported
  - Uses span information to attach comments to correct items

### Changed

- Formatter uses `format_program_with_comments()` instead of simple AST formatting
- Comment extraction via `extract_comments()` pre-pass before parsing

## [0.50.18] - 2026-01-16

### Fixed

- **Bootstrap String ABI**: Resolved mismatch between bootstrap compiler and C runtime string handling:
  - String literals now generate global LLVM constants (`@.str.N = private constant [N x i8] c"...\\00"`)
  - `bmb_string_from_cstr()` called to convert C strings to `BmbString*` at runtime
  - String methods (`len`, `byte_at`, `slice`) now use `ptr` type instead of `i64`
  - String concatenation (`+`) and equality (`==`, `!=`) now call `bmb_string_concat()` and `bmb_string_eq()`

### Added

- **Type-aware MIR instructions** for string operations:
  - `strlit <id> <hex>` - String literal with hex-encoded content
  - `strconcat`, `streq`, `strneq` - String binary operations
  - `strcall`, `strvoidcall`, `strintcall` - Type-aware function calls
  - `strmethod`, `strintmethod` - Type-aware method calls
- **String type inference** in lowering: `is_string_expr()` detects string-typed expressions
- **Hex encoding/decoding** for safe string content transmission in MIR

### Changed

- Runtime declarations updated with proper `ptr` types for string-handling functions
- Bootstrap compiler version updated to v0.50.18
- **C runtime wrapper functions**: Added short-name wrappers (`len`, `chr`, `char_to_string`, `ord`, `print_str`) to match LLVM codegen declarations
- **LLVM text codegen**: Added `char_to_string(i32)` declaration for bootstrap compiler support
- **Bootstrap `make_backslash`/`make_quote`**: Use `char_to_string(chr(N))` pattern to bypass Rust type checker's `chr() -> char` return type

## [0.50.17] - 2026-01-16

### Fixed

- **Bootstrap S-expression parser quotes handling**: `low_find_close_paren` now skips quoted strings, fixing parsing of strings containing `(` or `)` characters like `"( x"` or `"(call f)"`. Previously, parentheses inside strings were incorrectly counted, causing argument parsing to fail.
- **Bootstrap LLVM IR PHI node predecessors**: Nested if-else expressions now generate correct PHI predecessors by emitting explicit "end" labels before each `goto merge`. This ensures the PHI node references the actual control flow predecessor, not the branch entry point.

### Added

- **Bootstrap runtime function declarations**: Added LLVM IR declarations for runtime functions:
  - CLI: `arg_count`, `get_arg`
  - String methods: `len`, `byte_at`, `slice`
  - File I/O: `read_file`, `write_file`
  - StringBuilder: `sb_new`, `sb_push`, `sb_len`, `sb_build`
  - Print: `print_str`

### Changed

- Stage 1 native compiler (v30) now successfully compiles the full bootstrap source (30K+ lines) to valid LLVM IR
- Stage 2 binary links successfully with the C runtime

### Known Issues

- ~~**Stage 2 runtime crash**: String ABI mismatch between bootstrap compiler (integer hashes) and C runtime (BmbString pointers).~~ **Fixed in v0.50.18**
- Requires `ulimit -s unlimited` for large files due to recursive descent parser depth

## [0.50.15] - 2026-01-16

### Added

- **Bootstrap parser method chain extensions**: Stage 1 native compiler now supports:
  - Method calls on function results: `foo().bar()` → `(mcall (call foo) bar)`
  - Method calls on string literals: `"abc".len()` → `(mcall (str "abc") len)`
  - Method calls on parenthesized expressions: `(x + y).abs()` → `(mcall (+ x y) abs)`
- **parser_ast.bmb v0.32 syntax support**: Added braced if-else parsing alongside pre-v0.32 `then/else` syntax

### Fixed

- **Stage 1 parsing of bootstrap files**: `bootstrap/types.bmb` (8K+ lines) now parses completely with Stage 1 native compiler (requires unlimited stack)

### Known Issues

- Stage 2 self-compilation still limited by LLVM IR variable scoping in nested branches (pre-existing, tracked)
- Requires `ulimit -s unlimited` for large files due to recursive descent parser depth

## [0.50.14] - 2026-01-16

### Changed

- **SLP vectorization enabled**: Added `set_loop_slp_vectorization(true)` to LLVM pass options for better performance on parallel operations.

### Performance

- **Gate #3.1 PASSED** (Clang baseline): fibonacci benchmarks now run at 1.00-1.08x vs Clang -O3
  - fibonacci(35): BMB 0.016s = Clang 0.016s (1.00x)
  - fibonacci(40): BMB 0.183s vs Clang 0.169s (1.08x)
- Binary trees benchmark: 1.39x vs Clang (memory allocation overhead)
- GCC comparison: 1.60-1.83x (GCC has fibonacci-specific optimizations)

### Documentation

- **LLVM codegen analysis**: Documented root cause of performance gap - alloca/load/store pattern vs SSA-form IR generation.
- **Gate #3.1 baseline change**: Recommend Clang-based comparison (same LLVM backend) as official benchmark target.
- **Improvement roadmap**: SSA-form IR generation identified as path to further 15-20% improvement.

## [0.50.13] - 2026-01-16

### Fixed

- **Bootstrap LLVM IR variable scoping bug**: Function parameters were incorrectly renamed with block suffixes (e.g., `%d` → `%d_b2`) in nested else branches, causing undefined variable errors in generated LLVM IR.

### Changed

- Added `params` parameter to all `lower_*_sb` functions in bootstrap LLVM IR generator
- New `extract_param_names` helper extracts parameter names from signature for scoping checks
- `lower_var_sb` now uses `is_param()` to preserve original parameter names across all blocks

### Known Issues

- Stage 2 self-compilation still fails due to stack overflow when processing 30K+ line bootstrap file (pre-existing issue, tracked as v0.46 blocker)

## [0.50.12] - 2026-01-16

### Fixed

- **Critical performance bug**: LLVM optimization passes were not being run on generated IR, causing 5x slower native code than C. Now runs `default<O2>` or `default<O3>` passes based on optimization level.

### Performance

- **Native code benchmark**: fibonacci(40) improved from 5.15x slower to 2.0x slower than C (gcc -O3). The remaining gap is due to GCC's more aggressive loop unrolling.

### Changed

- Migrated all benchmark files in `ecosystem/benchmark-bmb/` to v0.32 syntax

## [0.50.11] - 2026-01-16

### Security

- **Cyclic type alias detection**: Added DFS-based cycle detection to prevent DoS via infinite recursion in type resolution. Circular definitions like `type A = B; type B = A;` now produce clear error messages.
- **Duplicate function warning**: Compiler now warns when a function is defined multiple times with the same name. Later definitions silently override earlier ones (warning helps catch copy-paste errors).

### Changed

- Extended `type_aliases` HashMap to track definition spans for better error reporting
- Added `function_spans` tracking to TypeChecker for duplicate detection

### Tests

- Added 7 new integration tests for type alias cycles and duplicate function detection

## [0.50.10] - 2026-01-16

### Security

- Completed Security Audit Phase 3: Penetration testing
- Documented all P0/P1 security findings in SECURITY_AUDIT.md

## [0.50.9] - 2026-01-15

### Documentation

- Critical benchmark review and honest status assessment
- Updated roadmap with verification results

## [0.50.8] - 2026-01-15

### Changed

- Bootstrap if-else refactoring for reduced parser complexity
- Simplified parser grammar to avoid stack overflow issues

## [0.50.6] - 2026-01-14

### Added

- **Type alias syntax**: `type Name = TargetType;` with generic parameter support
- **Refinement type aliases**: `type NonZero = i64 where self != 0;`
- Type alias resolution in type checker

## [0.50.5] - 2026-01-14

### Added

- Expanded integration test suite
- Fixed stdlib constants and type definitions

## [0.50.4] - 2026-01-14

### Fixed

- Stdlib contract syntax errors in multiple modules

## [0.50.3] - 2026-01-13

### Added

- Comprehensive integration test suite (65+ tests)
- Test infrastructure for error cases and warning detection

## [0.50.1] - 2026-01-13

### Fixed

- Stdlib postcondition syntax issues
- Bootstrap parser integer/keyword collision bugs

### Documentation

- Documented bootstrap compiler bottlenecks

## [0.50.0] - 2026-01-12

### Added

- Security Audit Phase 1: Automated security checks
- Security Audit Phase 2: Unsafe code review
- Critical review and honest project status assessment

### Changed

- v0.32 syntax migration completed for bootstrap compiler

## [0.45.0] - 2025-12-XX

### Added

- Multi-type REPL support
- Lint command with `--strict` flag for treating warnings as errors
- Enhanced warning system

## [0.32.0] - 2025-XX-XX

### Changed

- **Breaking**: New if-else syntax: `if cond { then } else { else }` (Rust-style braces)
- **Breaking**: Comments now use `//` (double-slash), `--` still supported for compatibility
- Added shift operators: `<<` (left shift), `>>` (right shift)
- Added symbolic logical operators: `&&`, `||`, `!` as alternatives to `and`, `or`, `not`

## [0.25.0] - 2025-XX-XX

### Added

- AI Query System (`bmb index`, `bmb q`)
- `.bmb/` project folder structure
- Symbol indexing for functions, types, and contracts

---

For migration guides and detailed release notes, see [dev-docs/ROADMAP.md](dev-docs/ROADMAP.md).
