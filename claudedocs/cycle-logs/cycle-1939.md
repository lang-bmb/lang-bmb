# Cycle 1939: gotgan E2E — dependency resolution + include path fix
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1938 clean (EARLY TERMINATION from previous run)

## Scope & Implementation

### Test Fixtures Created
- `ecosystem/gotgan/tests/fixtures/dep-chain/` — 3-tier dependency chain:
  - `pkg-base` (no deps): double, triple, is_positive
  - `pkg-mid` (depends on pkg-base): six_times, quadruple
  - `pkg-top` (depends on pkg-mid → transitively on pkg-base): twenty_four_times
- `ecosystem/gotgan/tests/fixtures/circular/` — circular dependency pair (pkg-a ↔ pkg-b)

### Core Fix: Module Resolution for Dependencies
**Problem**: `gotgan check` passed dependency source files but BMB compiler couldn't resolve `use` imports because:
1. gotgan didn't pass `-I` include paths to `bmb check`/`bmb build`
2. BMB's resolver didn't try `pkg-name/src/lib.bmb` pattern (gotgan convention)
3. BMB's resolver didn't convert underscore→hyphen for package names

**Fixed in 4 files:**
- `ecosystem/gotgan/src/build.rs`: `run_check()` and `run_build()` now pass `-I parent_dir` for each dependency
- `bmb/src/resolver/mod.rs`:
  - Added `add_search_path()` public method
  - `resolve_module_path()` now tries `name/src/lib.bmb` pattern
  - `resolve_module_path()` now tries underscore→hyphen conversion (`pkg_base` → `pkg-base`)
  - Same for `resolve_module_path_with_span()`
- `bmb/src/main.rs`: `check_file_with_includes()` adds include paths to resolver via `add_search_path()`

## Review & Resolution
- `gotgan check` on pkg-mid (direct dep): ✅
- `gotgan check` on pkg-top (transitive dep): ✅
- Circular dependency detection (pkg-a ↔ pkg-b): ✅ "Circular dependency detected: pkg-b"
- `cargo test --release`: 6,186 pass, 0 fail ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: gotgan `tree` shows only 1 dep (not transitive) — cosmetic issue
- Next Recommendation: Cycle 1940 — gotgan dependency tree display, lock file E2E
