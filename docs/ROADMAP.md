# BMB Roadmap

BMB (Bare-Metal-Banter) is an AI-native, contract-verified systems programming language. This document summarizes where BMB is today and what's next. For the detailed per-cycle development log, see `claudedocs/cycle-logs/`.

---

## Current Status — v0.98 (2026-04-22, post-Cycles 2460-2464)

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

### Cycles 2460-2464 (this session) — ✅ CI downstream jobs unblocked (8/9)

Follow-up session after Cycles 2441-2459 confirmed Build & Test
green but exposed the 6th blocker (downstream jobs missing
`submodules: recursive`) and a family of smaller CI-infra defects. All
seven commits this session are CI/infra only — no Rust compiler
behavior changes.

**Commits (chronological)**:

1. `0f6f1c28` **P1-ci-sub**: `ci.yml` 6 downstream checkout sites get
   `submodules: recursive`. Unblocks `code-quality`, `bootstrap-check`,
   `performance-regression`, `bench-compare-smoke`, `net-echo-smoke`,
   `gate-verification` (all had `cargo build` failing at workspace
   manifest load because `ecosystem/gotgan/Cargo.toml` was not checked
   out).

2. `e843a373` **P1-pin**: `rust-toolchain.toml` pins toolchain to
   1.95.0. Prevents `@stable` drift — Cycle 2453's 5th blocker (16
   clippy errors surfaced on CI-only 1.95) was caused by this drift.
   Both rustup (local) and `dtolnay/rust-toolchain` action read the
   pinned channel, so local and CI agree automatically.

3. `f443011e` **Docs**: `docs/ROADMAP.md` updated for prior session
   (Cycles 2427-2459).

4. `a7cd1a00` **Runtime `<unistd.h>` include**: strict clang on
   ubuntu-latest rejects implicit `getcwd`/`rmdir` declarations. POSIX
   branch of `bmb/runtime/bmb_runtime.c` include block gets
   `<unistd.h>` (1 line).

5. `80736235` **Rule 5 workflow sweep**: 5 additional workflow files
   (bootstrap-benchmark, golden-release, nightly-bench, pypi-publish,
   release) had 7 more `actions/checkout@v4` sites without
   `submodules: recursive`. Direct failure observed: `3-Stage
   Bootstrap` in bootstrap-benchmark.yml failed at cargo manifest load.
   Post-fix audit: 27/27 checkout sites across all 10 workflows now
   have submodules: recursive.

6. `57b061d5` **Bootstrap self-compile check rewrite**: `ci.yml`
   `Measure bootstrap compile time` previously looped `for f in
   bootstrap/*.bmb; do bmb build "$f"; done`. Per-file compilation
   reports spurious undefined-symbol errors because bootstrap modules
   reference symbols defined in siblings (e.g. `lowering.bmb` uses
   `unpack_place` from `mir.bmb`). Fix: build `bootstrap/compiler.bmb`
   only — the entry point resolves the full graph. Local verify:
   `bmb check compiler.bmb` → success (3148 warnings, 0 errors);
   `bmb build --emit-mir` runs in 0.56s vs 60s Gate #4.1 threshold.

7. `6fa06e9e` **LLVM_SYS env var correction**: 4 workflow files +
   `scripts/ci/setup-env.sh` (19 refs) had `LLVM_SYS_210_PREFIX`
   (LLVM 21.0), but `bmb/Cargo.toml` uses `inkwell` with feature
   `llvm21-1` → llvm-sys-211 (LLVM 21.1). Mass rename
   `210 → 211`. Fixes the "No suitable version of LLVM" error in
   `bootstrap-benchmark.yml` `3-Stage Bootstrap`.

**Empirical CI (6fa06e9e)**:

BMB CI 8/9 jobs pass — `Build & Test (3 OS)`, `Code Quality`,
`bmb bench --compare smoke test`, `Bootstrap Self-Compile Check`,
`Gate #4.1 Verification` all ✅. Remaining:
- `stdlib/net TCP echo E2E smoke test` — BMB LLVM IR generator emits
  invalid attribute ordering (`define private noundef range(i64 ...)
  i64 @sign`). Compiler internal bug, deferred to next session.

Bootstrap+Benchmark 4/7 jobs pass — `Build & Test (3 OS)` + `CI
Summary` ✅. Remaining:
- `3-Stage Bootstrap` — `could not find native static library
  'Polly'`. LLVM installation via `llvm.sh 21` does not include
  `llvm-21-polly-dev`. Separate apt install needed.

Golden Binary CI failure across 3 runners: `golden/linux-x86_64/bmb`
and `golden/darwin-universal/bmb` missing from repo — pre-existing
Windows-only golden-binary setup. Documented as P1-golden.

### Cycles 2441-2459 (prev session) — ✅ CI 3-platform Build+Clippy empirical validation

Second session after PyPI wheel infrastructure landing. Previous session
(Cycles 2427-2440) surfaced "3 platform blocker" (submodule + Ubuntu +
macOS + Windows). This session empirically validated all four via actual
CI runs, then found and resolved two more blockers (4th and 5th).

**(1) 4th blocker — gotgan Clippy extra-unused-lifetime (Cycles 2441-2442).**
After submodule + platform fixes unblocked CI checkout/build, `Clippy`
step failed on all 3 platforms with `clippy::extra-unused-lifetimes` on
`ecosystem/gotgan/src/resolver.rs:223` (`fn topo_visit<'a>`). User had
the 1-line fix locally but unstaged; previous session incorrectly treated
gotgan dirty as "untouched local work". Committed `15ab20c` upstream,
parent pointer bumped `dff30558`.

**(2) `bindings-ci.yml` workflow_dispatch (Cycle 2443).** Submodule
pointer bumps don't match bindings-ci path filter → auto-trigger fails.
Added `workflow_dispatch:` trigger for manual re-run flexibility.

**(3) 3-platform Build empirical validation (Cycles 2444-2452).** CI
`dff30558` BMB CI step-level observation confirmed all 3 platforms pass
`Build (release)`:
- Ubuntu `LLVM_SYS_211_PREFIX` (Cycle 2434) — validated
- macOS matrix split `macos-13`/`macos-latest aarch64` (Cycle 2434) — validated
- Windows `egor-tensin/setup-mingw@v2 static: 0` (Cycle 2435) — validated

**(4) 5th blocker — Rust 1.95 lint drift (Cycles 2453-2456).** Ubuntu
Clippy still failed post-Build with 16 errors in `bmb` lib. Root cause:
`dtolnay/rust-toolchain@stable` gave CI Rust 1.95.0 (2026-04-14) while
local was 1.94.0 (2026-03-02). Rust 1.95 strengthens
`clippy::collapsible_match` and `clippy::useless_conversion` lints.
Resolution: local `rustup update`, case-by-case fix of 16 sites — 10
proper guard-collapse, 6 narrow `#[allow(clippy::collapsible_match)]` on
`propagate_copies_in_inst` (pattern-guard bindings are immutable but
`propagate_operand` requires `&mut Operand`, language-level constraint),
1 `.into_iter()` removal. Commit `50f1c607`. Rule 6 judgment:
distribution blocker, semantic-preserving, 3,764 tests pass.

**(5) Empirical CI final — `50f1c607` BMB CI Build+Test result.**
- Build & Test (ubuntu-latest) — ✅ **success** (all 10 steps)
- Build & Test (macos-latest) — ✅ **success**
- Build & Test (windows-latest) — ✅ **success**

Clippy **passes** on all three platforms after the 16-site fix. The
session's core objective — unblock cross-platform CI — is achieved.

**(6) Remaining downstream failures (6th blocker — defer).** Build
success revealed three downstream jobs still fail because their own
`actions/checkout@v4` steps lack `submodules: recursive`:
- `Code Quality (fmt + lint)` — `failed to load manifest for workspace
  member 'ecosystem/gotgan'`
- `bmb bench --compare smoke test` — same
- `stdlib/net TCP echo E2E smoke test` — same

Previously masked by Clippy step failure. Now visible because
`build` passes. Simple fix (6 checkout sites + 1 line each) deferred to
next session.

Session commits: 4 (`15ab20c`, `dff30558`, `9bed6089`, `50f1c607`).
Local verification: `cargo test --release --lib` 3,764 pass / 0 fail,
`cargo clippy --all-targets -- -D warnings` clean on Rust 1.95.

### Cycles 2427-2440 (prev session) — ✅ Submodule blocker + 3-platform CI fix

Resolved the CI checkout blocker discovered in previous-previous session.
Previous HANDOFF had identified a single submodule (`benchmark-bmb`) as
problematic; Rule 5 full audit found **4 submodules** with fast-forward
ahead commits never pushed upstream (`benchmark-bmb`, `gotgan`,
`tree-sitter-bmb`, `vscode-bmb`). All four pushed to upstream
(Cycles 2427-2428). Added `scripts/verify-submodules.sh` as pre-push
regression check (Cycle 2433).

After submodule checkout succeeded, first full CI run surfaced 3 real
platform blockers in `bindings-ci.yml`:
- Ubuntu: `LLVM_SYS_211_PREFIX` env missing (macOS had it) — added
- macOS: arch mismatch (`macos-latest` is ARM, target was x86_64) —
  matrix split into `macos-13` + `macos-latest aarch64` mirroring
  pypi-publish.yml's Cycle 2416 layout
- Windows: `egor-tensin/setup-mingw@v2` default `static: 1` fails on
  missing `libpthread.dll.a` — set `static: 0` (our Cycle 2423 linker
  flags already remove MinGW runtime)

Both `bindings-ci.yml` and `pypi-publish.yml` received the fixes.
Commit `d059dbc7`. Empirical validation landed in the next session
(Cycles 2444-2452 above).

### Cycle 2426 (prev session) — ✅ CI action reference fixed

Pre-existing bug surfaced by Cycle 2425's push. All 15 occurrences of
`dtolnay/rust-action@stable` (non-existent action) renamed to
`dtolnay/rust-toolchain@stable` across 8 workflow files. Unblocks CI
observation that Cycle 2425 enabled. New push-triggered runs now
reach build/test phase instead of failing at "Set up job" in 5s.

### Cycle 2425 (this session) — ✅ 154 commits pushed to origin/main

Maintainer approval obtained. Cross-platform CI now receives changes
from Cycles 2411-2425 inclusive. First normal push-triggered runs
surfaced the Cycle 2426 action-name bug (see above); subsequent runs
exercise actual build/test.

### Cycle 2423 (this session) — ✅ P3-T3a: MinGW runtime statically linked

Adding `-static -static-libgcc` to both link paths in
`bmb/src/build/mod.rs` eliminates `libgcc_s_seh-1.dll` and
`libwinpthread-1.dll` from every `bmb build` output. Remaining DLL deps
are Windows-system UCRT forwarders (`api-ms-win-crt-*`), KERNEL32, and
WS2_32 — all guaranteed on Windows 10+. This was the last distribution
gap after Cycles 2419-2420 fixed Defect 5; the P3 track (T3b wheel
bundling, T3c MSVC toolchain switch) is now fully obviated.

Binary size delta: +30-60 KB per .dll (bmb-algo 305 → 341 KB, +12%).
Acceptable for removing all MinGW runtime dependencies.

Verification: isolated-venv install + functional calls across all 5
bindings; `cargo test --release --lib` 3,764 pass; `cargo clippy` clean;
3-Stage Fixed Point S2 == S3 (69s). Rule 6 judgment: direct continuation
of Cycle 2420's distribution-blocker work, ~10 LOC linker-flag change.

### Cycles 2419-2420 (prev session) — ✅ Defect 5 resolved

Three fixes landed together; `bmb build --shared` now produces correct
platform shared libraries under `--features llvm` (inkwell backend) and
without (text backend).

1. **Runtime ↔ `@export` symbol rename** (Cycle 2419, user-side only).
   `bmb-compute` `bmb_is_power_of_two` / `bmb_next_power_of_two` → `bmb_c_*`
   (consistent with existing `bmb_c_abs/min/max/clamp` prefix);
   `bmb-algo` `bmb_is_prime` → `bmb_algo_is_prime`. No compiler or
   runtime change. Python public APIs unchanged.
2. **Inkwell SharedLib link path** (Cycle 2420). `bmb/src/build/mod.rs`:
   `link_executable` parameterised to `link_native(obj, output, verbose,
   is_shared)`, now called for both `Executable` and `SharedLib` output
   types. Adds `-shared` and skips `-no-pie` on Linux for shared libs.
3. **`@export` dllexport + linkage-priority** (Cycle 2420).
   `bmb/src/codegen/llvm.rs`: `@export` functions now get
   `DLLStorageClass::Export` and override the `always_inline` →
   `Linkage::Private` decision. Without this second fix, inlined
   `@export` functions got `define private dllexport` in IR — private
   wins over dllexport and the symbol never appears in the DLL.

End-to-end verification (Cycle 2420): fresh rebuild of all five binding
libraries succeeds in 1.5s; `./scripts/build-wheel.sh --verify` installs
and imports 5/5 wheels with correct public-function counts
(algo=56, compute=33, crypto=15, json=13, text=24). `cargo test
--release --lib` → 3,764 pass / 0 fail maintained. 3-Stage Fixed Point
unaffected (the inkwell codepath changes only fire on
`@export`/`SharedLib`, neither of which appears in bootstrap build).

**P0-inf now unblocked**: `pypi-publish.yml` and the `bindings-ci.yml`
wheel gate will produce correct wheels on their first CI run. Cross-
platform push remains gated on user approval.

### Cycle 2418 — 🔴 Defect 5 discovered: `bmb build --shared` broken

Audit of the wheel pipeline's foundation revealed a systemic bug. The
infrastructure built in Cycles 2411-2417 is structurally correct but the
underlying `bmb build --shared` command does not produce working `.dll`
files from fresh builds. Three compiler paths all fail:

- **Inkwell backend** skips linking entirely for `OutputType::SharedLib`
  (emits `.o`, prints `build_success`, never calls linker).
- **Text backend** links but hits runtime ↔ `@export` symbol collisions
  (`bmb_is_power_of_two` defined by both `bmb_runtime.c` and
  `ecosystem/bmb-compute/src/lib.bmb`).
- **Bootstrap Stage 1** fails with "lowering produced empty MIR" on the
  same binding source.

Every successful wheel build this session copied a **stale `.dll`** from
prior sessions (`bmb_algo.dll` dated 2026-03-23). Functionally the wheels
install and import correctly, but the `.dll` is frozen months behind
current source. **Fresh CI runners will fail**: no pre-built `.dll` →
`ecosystem/build_all.py` silent no-op → `shutil.copy2` FileNotFoundError
→ job aborts.

**Scope**: not fixable within this session's remaining budget. See
CHANGELOG.md "Discovered (Cycle 2418)" for full detail. Next session
should treat Defect 5 as a blocker above P1 (Defect 3) — Defect 3 is an
improvement path, Defect 5 is a prerequisite for the P0 work to reach
users.

### Cycles 2411-2412 (this session) — PyPI wheel CI pipeline

**P0 from previous handoff — Defect 3 safe zone (`compiler.bmb` untouched).**
Two-cycle scope: unblock PyPI publication of the five binding libraries.

**Cycle 2411 — Platform wheel tagging fix.** Survey uncovered two defects:

1. `pip wheel .` produced `py3-none-any` pure-Python wheels despite each
   package bundling a platform-specific `.dll` / `.so` / `.dylib` in
   `package_data`. A Linux user pip-installing would receive a Windows
   DLL. Fix: `setup.py` shim with `BinaryDistribution(has_ext_modules=
   True)` **and** a custom `bdist_wheel.get_tag()` returning
   `("py3", "none", plat)` — platform-specific, Python-version-independent,
   ABI-independent. Resulting tag: `py3-none-win_amd64` (and the
   corresponding Linux / macOS tags when built on those runners).
2. Version drift — all five `setup.py` files hardcoded `version='0.2.0'`
   while `pyproject.toml` had bmb-algo and bmb-crypto at `0.3.0`.
   Dual source-of-truth collapsed: `setup.py` is now a 30-line shim,
   every metadata field lives in `pyproject.toml`.

Install + import smoke test passed in a clean venv for bmb-algo (56
public functions exposed). All five libraries build wheels with the
correct tag.

**Cycle 2412 — `scripts/build-wheel.sh` + `pypi-publish.yml`.**

- `scripts/build-wheel.sh` (150 LOC) — locates or rebuilds the BMB
  compiler, runs `ecosystem/build_all.py`, then `pip wheel . --no-deps`
  for each library into `dist/wheels/`. Options `--dry-run`, `--lib`,
  `--skip-compiler`, `--skip-libs`. Validation gate exits non-zero if
  any `py3-none-any` wheel slips through.
- `.github/workflows/pypi-publish.yml` — manual-dispatch only
  (`workflow_dispatch`). Matrix Windows + Ubuntu + macOS, each builds
  its own BMB compiler, runs `build-wheel.sh`, validates wheel tags,
  uploads per-platform artifacts. Separate `publish` job (opt-in via
  `inputs.publish=true`, `inputs.repository=testpypi|pypi`) with
  trusted-publishing OIDC + token fallback.
- `.gitignore` extended with `/dist/`, `**/*.egg-info/`, `**/bmb_*.egg-info/`.

Pending human actions (gated):
- Configure `PYPI_API_TOKEN` / `TEST_PYPI_API_TOKEN` repo secrets.
- Create `testpypi` + `pypi` deployment environments.
- Trigger `workflow_dispatch` with `publish=false` once to validate
  cross-platform builds on GitHub-hosted runners.

Full per-cycle detail: `claudedocs/cycle-logs/cycle-2411.md`,
`cycle-2412.md`.

### Cycles 2406-2410 — Defect 4 user-side workaround

**Compiler-side Defect 4 fix blocked by Defect 3 re-trigger.** Two
in-place modifications to `inject_post_assumes_in_fn_scan`
(`bootstrap/compiler.bmb:15702`) — one adding 6 lines of safety
check, the minimal second attempt adding only 3 lines — **both
re-triggered Stage 2 corruption** (parse error at line 1:1 and arena
16 GB exhaustion respectively). Cycle 2402's 1-line `implies` tweak
was therefore not a generic "existing fn body edits are safe"
escape hatch — Defect 3 is sensitive to AST complexity inside
existing fn bodies too. Full quantitative trace:
`claudedocs/cycle-logs/cycle-2407.md`.

**Pivot: user-side stdlib contract weakening** (Cycles 2408-2409).
Instead of fixing the compiler's post-injection substitution, weaken
stdlib posts so the post-assume IR never contains a callee-param
reference to leak. Eight functions now build + run via bootstrap:

- `stdlib/string/mod.bmb`: `find_trim_start_from`,
  `find_trim_end_from` — `ret >= pos` / `ret <= pos` clauses removed
  or replaced with constants.
- `stdlib/array/mod.bmb`: `index_of_i64`, `index_of_i64_from`,
  `count_i64`, `min_i64_from`, `max_i64_from`, `clamp_index`,
  `wrap_index` — `ret < len` / `ret <=/>= current_*` clauses
  dropped or replaced with array-size constants.

Regression guards committed: `tests/bench/defect4_trim_smoke.bmb`
(trim build+run), `tests/bench/defect4_array_all_smoke.bmb` (6-fn
coverage). Both exit 0 via bootstrap. 3-Stage Fixed Point
unchanged (compiler.bmb untouched). `cargo test`: 3,764 pass.

**Deferred**: `stdlib/parse/mod.bmb` has 10+ `ret >= pos` posts but
**zero** current `@include "stdlib/parse"` consumers in the repo —
cleanup postponed until a real user appears.

**Trade-off documented in CHANGELOG**: contracts are strictly
weaker (tighter bounds dropped or replaced with constants); the
stronger forms can be restored once Defect 3 is root-caused and a
proper AST-level param substitution becomes possible in the
bootstrap.

### Cycles 2391-2396 (earlier session)

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
| Lexer-tolerant `implies` keyword (stdlib/string/array `@include` check) | ✅ Cycle 2402 (`keyword_len7` maps `implies` → `TK_OR`; contract bodies discarded by `skip_contracts` so semantics unchanged. Build still blocked by Defect 4). |
| PyPI wheel build + publish | Packaging + CI pipeline ✅ (Cycles 2411-2412), publish gated on repo-secret setup |
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

## Next-session recommended priority (2026-04-22, post-Cycle 2464)

> **Update**: CI infrastructure is broadly green (BMB CI 8/9 jobs
> pass). Remaining work organized into **4 Tracks** with merged scope
> instead of many small tickets. Each Track has a single recommended
> path chosen for alignment with project philosophy:
> **Performance > Everything**, **No Workaround**, **Rule 5 전수
> 검색**, and **Rule 7 백엔드 parity**.

### Track A — CI Green Baseline Completion (AUTONOMOUS, 4-6 cycles + external)

**Goal**: every CI job green, so future regressions are caught
immediately. Merges P1-polly + P1-net-ir + P1-wheel + P1-bind into a
single coherent track.

| Step | Scope | Effort |
|------|-------|--------|
| A.1 | **LLVM Polly dev install (전수)** — add `llvm-21-polly-dev` in `bootstrap-benchmark.yml`'s `llvm.sh 21` block, then Rule 5 audit all other LLVM install scripts (bindings-ci, pypi-publish, benchmark-baseline, nightly-bench, benchmark) for the same omission. Trivial. | 1 cycle |
| A.2 | **BMB LLVM IR `range(...)` attribute position (근본)** — `bmb-stage1.ll` emits `define private noundef range(i64 ...) i64 @sign` which LLVM 21 rejects. Locate emitter in `bmb/src/codegen/llvm.rs` + `llvm_text.rs` + bootstrap `llvm_ir.bmb`, fix emission order, add golden test exercising `range` on function return. **Rule 6 exception** (distribution blocker) + **Rule 7 parity** across both backends. | 2-4 cycles |
| A.3 | **Distribution dispatch + validation (external)** — `gh workflow run pypi-publish.yml -f publish=false` for 4-platform wheel artifacts + `gh workflow run bindings-ci.yml` for end-to-end bindings pytest. Wait 20-40m. Local cycle only for triggering + result inspection. | 1-2 cycles + external |

**Total**: 4-7 cycles + 1× external wait. Track A fully unblocks
distribution pipeline.

### Track B — Distribution Validation (HUMAN DECISION REQUIRED)

**Goal**: real-world `pip install` rehearsal, not just artifact-only.

| Decision | Options | **Recommended** |
|----------|---------|-----------------|
| TestPyPI rehearsal | (a) Register `TEST_PYPI_API_TOKEN` org secret + full publish to TestPyPI + clean-VM `pip install` test. (b) Skip TestPyPI, go straight to prod PyPI once Track A lands. | **(a) — TestPyPI first**. Philosophy: distribution is a user-facing contract; the only faithful validation is real upload → real `pip install` on a clean machine. Artifact-only (`publish=false`) rehearsal cannot detect upload-time issues (wheel metadata conflicts, filename collisions, size limits). Register the token once, reuse forever. |

**Action required**: maintainer creates TestPyPI token at
https://test.pypi.org/manage/account/token/ and registers as
`TEST_PYPI_API_TOKEN` org secret. After that Track A.3 becomes `gh
workflow run pypi-publish.yml -f publish=true -f repository=testpypi`.

### Track C — Compiler Quality — Defect 3/4 (HUMAN DECISION REQUIRED)

**Goal**: root-cause Defect 3 → unblock Defect 4 fix → unblock P5
bootstrap SIMD intrinsic dispatch.

| Decision | Options | **Recommended** |
|----------|---------|-----------------|
| Debug environment for Defect 3 | (a) Install DrMemory on Windows. (b) WSL + gdb on Linux-built Stage 1. (c) Remote Linux VM + gdb. (d) IR diff (probe vs no-probe) — debugger-free. | **(b) WSL + gdb**. Philosophy: match production (Linux is primary distribution target), use native debugger (stronger than DrMemory heuristics), Stage 1 is deterministic on both OSes so findings transfer. WSL is zero-cost on existing Windows dev box. Fallback (d) IR diff in parallel if gdb inconclusive. |

**Action required**: maintainer installs WSL + Ubuntu + `apt install
gdb build-essential`. After that Cycle runs `P2 Defect 3 dedicated`
with 2-cycle HARD limit per roadmap rule.

### Track D — Infrastructure Cleanup (AUTONOMOUS, 1-2 cycles)

**Goal**: remove or right-size duplicated / obsolete CI paths.

| Decision | Options | **Recommended** |
|----------|---------|-----------------|
| Golden Binary CI (Linux/macOS binaries missing) | (a) Conditional matrix skip on missing-binary OS. (b) Generate + commit 10-50 MB binaries (git-lfs?). (c) **Deprecate golden-ci.yml + golden-release.yml**. | **(c) Deprecate**. Philosophy — **No Workaround**: option (a) is a workaround (mask failure with `if:`), (b) adds dependency (LFS or binary bloat) and still tracks three moving targets. **Rule 5 audit rationale**: `pypi-publish.yml` and `bindings-ci.yml` already build fresh on 3 platforms every dispatch — golden CI's "pre-built binary consistency" check is redundant to "CI rebuilds + tests pass". Windows-only golden is scope-inconsistent with cross-platform distribution. Removal is the cleanest path: delete 2 workflow files + `golden/` directory; distribution verification is fully covered by Track A.3. |

### Track E — Later (v0.99+, post-v1.0)

Unchanged from previous handoff. Not next-session scope:

| Item | Effort | Risk | Notes |
|------|--------|------|-------|
| P4 `stdlib/net` TLS (OpenSSL) | 6-10 cycles | MEDIUM-HIGH | Post-v1.0 advanced-users. |
| P5 Bootstrap SIMD intrinsic CALL dispatch | 10+ cycles | HIGH | Defect 3 의존. |
| P6 DWARF stack trace | 4-6 cycles | MEDIUM | ROI-capped. |
| P7 `stdlib/parse` post weakening | 1-2 cycles | LOW | Zero consumers, defer. |

### Completed in prior/current sessions (no action)

| # | Item | Status |
|---|------|--------|
| P0-new / P0-inf | Defect 5 + PyPI CI pipeline | ✅ Cycles 2419-2420 / 2411-2417 |
| P3-T3a | MinGW runtime static-link | ✅ Cycle 2423 |
| P1-new-push / -ci / -sub / -3plat / -clippy1/2 | Various CI unblocks | ✅ Cycles 2425-2456 |
| P1-ci-sub / -pin / -bootstrap-check / -llvm-sys | This session's cleanups | ✅ Cycles 2460-2464 |
| P4 | `stdlib/net` TLS (`tcp_tls_connect`, `accept_tls`) | 6-10 cycles | MEDIUM-HIGH | MEDIUM | OpenSSL external dependency. Post-v1.0 advanced-users target. |
| P5 | Bootstrap SIMD intrinsic dispatch | 10+ cycles | HIGH | MEDIUM | Defect 3-adjacent risk. |
| P6 | DWARF stack trace | 4-6 cycles | MEDIUM | LOW | MIR lacks span info; gains limited. |
| P7 | stdlib/parse post weakening | 1-2 cycles | LOW | LOW | Zero current consumers. Defer. |
| P3 | `stdlib/net` TLS (`tcp_tls_connect`, `accept_tls`) | 6-10 cycles | MEDIUM-HIGH | MEDIUM | OpenSSL external dependency. Post-v1.0 advanced-users target. |
| P4 | Bootstrap SIMD intrinsic CALL-site dispatch | 10+ cycles | HIGH | MEDIUM | 211 intrinsics × vec-type alloca/call rewrite. **Likely re-triggers Defect 3** given scope — gate on P1 outcome. |
| P5 | DWARF stack trace | 4-6 cycles | MEDIUM | LOW | MIR lacks span info; gains limited to function granularity. ROI-capped. |
| P6 | stdlib/parse post weakening | 1-2 cycles | LOW | LOW | Currently zero `@include "stdlib/parse"` consumers. Defer until a real user appears. |

**Decision tree (post-Cycle 2464)**: CI downstream unblocked (8/9).
Next session pursues **Track A** (4-7 cycles autonomous, fully green
CI + wheel rehearsal) in parallel with maintainer-side **Track B**
(TestPyPI token) and **Track C** (WSL environment). **Track D**
(deprecate golden CI) autonomous fill-in (1-2 cycles) if budget
remains. Track A complete → distribution pipeline production-ready.

---

## Next-session options (full menu)

| Option | Effort | Risk | Notes |
|--------|--------|------|-------|
| Cross-platform SIMD + net verification (Linux/macOS) | 3-5 cycles | LOW-MEDIUM | Needs push to trigger CI; 144+ local commits ahead of origin. First observation on merge covers `net-echo-smoke` (ubuntu-latest), UDP echo + SIMD still Windows-only |
| **Bootstrap self-parse fragility (Defect 3)** | 2-3 cycles | HIGH | Trigger narrowed in Cycles 2399-2401 (20-probe matrix): any new top-level fn whose body references a param via expression *or* whose two param names are both long (`source`+`position` etc.) causes either 16 GB arena exhaustion or a misplaced EOF parse error. Failure is deterministic per input. Stage 1 (Rust-built) and Stage 2 (BMB-built) binaries fail identically — bug is inside `compiler.bmb`, not Rust codegen. Root cause still unknown. Blocks Defect 4 fix + any major bootstrap refactor. Hex/token-dump investigation still needed. |
| **Bootstrap overload post-injection (Defect 4)** | 2-4 cycles | HIGH | Discovered Cycle 2403. `inject_post_assumes_in_fn_scan` at `compiler.bmb:15702` replaces `%ret` → `result_reg` at call site injection but leaves callee parameters (e.g. `%pos` from `find_trim_start_from`'s `post ret >= pos`) dangling. Generated IR fails `opt` with "use of undefined value". Correct fix requires AST-level arg→param substitution + at least one new helper fn — blocked by Defect 3. Rust driver unaffected. **Cycles 2406-2409 user-side workaround**: weakened stdlib/string (2 fns) + stdlib/array (6 fns) posts to remove param refs; smoke tests `defect4_trim_smoke.bmb` + `defect4_array_all_smoke.bmb` now build+run via bootstrap. **Cycle 2407 added evidence** that Defect 3 is sensitive to AST complexity inside existing fn bodies too — Cycle 2402's 1-line tolerance was not a general escape hatch. |
| ~~stdlib/string, stdlib/array bootstrap `@include` check~~ | ~~1-2 cycles~~ | ✅ **완료 (Cycle 2402)** | `keyword_len7` lexer-tolerant `implies → TK_OR` mapping. Check passes; build still blocked by Defect 4. |
| Bootstrap SIMD intrinsic CALL-site dispatch | 10+ cycles | HIGH | Stub compile safe (Cycle 2375); Cycle 2387 reconnaissance showed full dispatch requires vector-type awareness in the bootstrap type checker (211 intrinsics × vec-type alloca + call replacement). Silent-correctness limitation documented in `stdlib/simd/mod.bmb` header — bootstrap calls return 0. Workaround: use Rust driver for SIMD. Not a v0.98 blocker. |
| `stdlib/net` TLS extension (`tcp_tls_connect`, `accept_tls`) | 6-10 cycles | MEDIUM-HIGH | Needs OpenSSL binding — new external dependency |
| ~~`stdlib/net` `udp_recvfrom` (peer address exposure)~~ | ~~2-4 cycles~~ | ✅ **완료 (Cycles 2385-2386)** | Runtime `BmbUdpPacket` + 5 accessor 심볼 추가, bootstrap extern 매핑 + stdlib wrapper + smoke 테스트. Multi-client UDP server 가능. |
| Runtime stack trace support (DWARF) | 4-6 cycles | MEDIUM | MIR currently lacks span info — gains limited to function-level unless MIR refactored; reconsider vs ROI |
| ~~`.bit_count()` / `.leading_zeros()` codegen (bootstrap)~~ | ~~1-2 cycles~~ | ✅ **완료 (Cycle 2384)** | `method_to_runtime_fn` + `llvm_gen_call` dispatch에 popcount/clz/ctz/bit_reverse/bswap/bit_not/bit_and/bit_or/bit_xor/bit_shift_left/bit_shift_right 전체 추가. Latent 6건 동시 해소 (bit_and/or/xor/shift_*/bit_not). Fixed Point ✅. |
| ~~CHANGELOG.md reconstruction (v0.67 → v0.98)~~ | ~~3-5 cycles~~ | ✅ **완료 (Cycle 2389)** | Summary blocks added for v0.96.20-v0.96.46, v0.97.0-v0.97.5, v0.98.0; v0.96.1-v0.96.19 per-cycle detail preserved under group header. |
| ~~PyPI wheel publish pipeline~~ | ~~2-4 cycles~~ | ✅ **pipeline wired (Cycles 2411-2412)** | `scripts/build-wheel.sh` + `.github/workflows/pypi-publish.yml` (manual-dispatch, 3-OS matrix); platform-wheel tagging fixed via `setup.py` shim (py3-none-&lt;platform&gt;). Verification hardened Cycle 2414 (`twine check` + install-import). Maintainer guide: [`docs/PACKAGING.md`](PACKAGING.md). Publish itself gated on `PYPI_API_TOKEN` secret registration (user action). |
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
