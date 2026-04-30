# Cycle 2500: B'.1 windows-latest Bindings CI — MSVC clang POSIX gap fix
Date: 2026-04-30

## Re-plan
Plan valid, inherited scope from HANDOFF.md priority 1 (B'.1 windows-latest
verification). HEAD `25998ad6` CI results: BMB CI ✅, Bootstrap+Benchmark ✅,
Update Benchmark Baseline ✅; **Bindings CI windows-latest ❌**. Job
`73608010107` failed at "Build all binding libraries" step (5/5 libraries
fail). `gh run list` showed Bindings CI as "queued" only because macos-13
is still queued — the 3 main matrix jobs completed. ubuntu-latest ✅
(4m48s), macos-latest ✅ (7m23s), windows-latest ❌ (10m4s).

## Scope & Implementation

**Root cause** (from log lines 1380-1538):
```
fatal error: 'dirent.h' file not found
 2748 | #include <dirent.h>
```

The runtime `bmb_runtime.c` unconditionally included POSIX `<dirent.h>`.
- MinGW UCRT64 (local + Cycle 2492 path): provides `dirent.h` via
  `_mingw_unicode.h`-adjacent headers.
- KyleMayes LLVM 21 on `windows-latest` (CI): MSVC clang ABI, links against
  Microsoft UCRT — `dirent.h` not available.

Cycle 2492 made the `--target=x86_64-pc-windows-gnu` flag conditional on
detected MinGW ABI. With MSVC clang, that flag is now correctly absent —
which exposed the latent POSIX dependency in the runtime that was
previously masked by always forcing the MinGW target.

**Decision Framework analysis** — Level 5 (Runtime). Runtime needed to
support both MSVC and MinGW clang ABIs without forcing a specific target.

**Fix** (`bmb/runtime/bmb_runtime.c`):

1. Made `<dirent.h>` POSIX-only:
   ```c
   #ifdef _WIN32
   #include <direct.h>
   #ifndef S_ISDIR
   #define S_ISDIR(m) (((m) & _S_IFMT) == _S_IFDIR)  // MSVC fallback
   #endif
   #else
   #include <dirent.h>
   #endif
   ```

2. Rewrote `bmb_readdir` with Win32 `FindFirstFileA`/`FindNextFileA`/
   `FindClose` for `_WIN32`, kept POSIX `opendir`/`readdir`/`closedir`
   for non-Windows. Same external behavior (newline-separated, skip
   `.`/`..`). Also fixed a latent realloc-NULL leak in the POSIX path
   while there.

3. Made `_mkdir` (line 2778) and `_rmdir` (line 2854) explicit on
   `_WIN32` instead of relying on MSVC's deprecated `mkdir`/`rmdir`
   wrappers in `<direct.h>` (which compile but emit warnings under
   `_CRT_NONSTDC_DEPRECATE`; safer to use the underscore-prefixed names).

**Files changed**:
- `bmb/runtime/bmb_runtime.c` — 3 hunks (lines 2747-2751, 2778-2782,
  2796-2835, 2851-2859)

**Rule 5 (전수 검색) sweep**: Other POSIX-only sites checked:
- `getcwd` (line 2585): already in `#else` POSIX block ✅
- `usleep` (line 4616): already in `#else` POSIX block ✅
- `<unistd.h>` (line 26, 4615): already in `#else` POSIX blocks ✅
- `<pthread.h>` (line 23): already in `#else` POSIX block ✅
- `stat` / `S_ISDIR`: stat is in MSVC `<sys/stat.h>`; S_ISDIR fallback added.

## Verification & Defect Resolution

| Check | Result |
|-------|--------|
| `cargo build --release` (text backend, MinGW UCRT) | ✅ 1m 8s |
| `cargo test --release --lib` | ✅ **3,772 pass** / 0 fail |
| `cargo test --release --lib --features llvm --target x86_64-pc-windows-gnu` | ✅ **3,953 pass** / 0 fail |
| `cargo clippy --all-targets -- -D warnings` | ✅ clean |
| `python ecosystem/build_all.py` (MinGW UCRT, local) | ✅ **5/5 OK** in 4.3s |

**MSVC clang validation** is CI-only (no local MSVC available). The Win32
API substitution (`FindFirstFileA`, `_mkdir`, `_rmdir`) is standard
documented Windows API available on both MinGW and MSVC, so failure is
unexpected.

No defects found in Reflect step.

## Reflection

**Scope fit**: ✅ Direct and minimal — runtime POSIX→Win32 substitution
only. No compiler changes required (the codegen flag gating from Cycle
2492 was correct; this fix complements it on the runtime side).

**Latent defects discovered**: 1 minor (and fixed) — the original POSIX
`bmb_readdir` realloc leaked the original buffer if `realloc` returned
NULL. Fixed in the rewrite by storing the new pointer in a temp before
overwriting.

**Philosophy drift**: None. This is a Level 5 runtime fix that resolves a
distribution blocker (MSVC clang ABI in CI). No language-spec or
compiler-structure compromise.

**Roadmap impact**: B'.1 was the headline blocker for the next phase. If
CI on this commit shows windows-latest Bindings green, B'.1 is complete
and the session can proceed to B'.2 (HUMAN gate — TestPyPI token) or
G.1 (Z3 setup). If CI still red, deeper MSVC ABI gaps may exist —
escalate.

## Carry-Forward
- **Actionable**: Push commit → observe Bindings CI on windows-latest. If
  green, mark B'.1 complete; if red, analyze new failure (likely a
  similarly latent POSIX dependency revealed by Cycle 2492's correct
  ABI gating).
- **Pending Human Decisions**: B'.2 (TestPyPI token) gating is unchanged.
- **Roadmap Revisions**: None (same priorities, just B'.1 implementation
  in progress).
- **Next Recommendation**: Cycle 2501 = commit + push + observe CI; then
  G.1 (Z3 setup) if Z3 is locally available, otherwise G.4 latent dedup
  or H tier C.
