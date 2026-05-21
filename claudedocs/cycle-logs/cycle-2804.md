# Cycle 2804: Playground WASM Integration — App.tsx + Build Fix
Date: 2026-05-13

## Re-plan
Plan valid. Continuing directly from Cycle 2803 (WASM build infrastructure). Scope: wire WASM into App.tsx UI, fix the `env` import blocker discovered during dev server startup.

## Scope & Implementation

### Problem Solved: `env` WASM imports
The original bundler-target WASM (`--target bundler`) imported `free`, `calloc`, `malloc`, `realloc` from the WASM `env` module. Vite's module system tried to resolve `"env"` as an npm package and failed with HTTP 403.

Root cause: `bmb/src/interp/eval.rs` had an `unsafe extern "C"` block declaring `malloc`/`free`/`calloc`/`realloc`. On wasm32-unknown-unknown these become WASM imports from `env`. On native they link to libc.

Fix in `bmb/src/interp/eval.rs` (lines 7054-7155):
- Gated the libc `extern "C"` block with `#[cfg(not(target_arch = "wasm32"))]`
- Added `#[cfg(target_arch = "wasm32")] mod wasm_heap` with Rust `std::alloc` implementations of all four functions, using a `thread_local! HashMap<usize, usize>` to track allocation sizes (needed for `std::alloc::realloc` which requires the original `Layout`)
- Used `use wasm_heap::{calloc, free, malloc, realloc}` on wasm32 so the rest of the file is unchanged

### WASM rebuild: `--target web`
Rebuilt with `wasm-pack build --target web --out-dir pkg`. The `--target web` output:
- Uses `new URL('bmb_wasm_bg.wasm', import.meta.url)` + `fetch()` to load WASM at runtime
- Exports an async `default` init function; the WASM binary's imports are provided internally via `__wbg_get_imports()`
- Result: zero `env` imports in the WASM binary (verified via Node.js `WebAssembly.Module.imports()`)

### Vite config fix
The WASM file is fetched from `bmb-wasm/pkg/` via `@fs` — Vite restricted access because `bmb-wasm/pkg` is outside the playground's project root. Added `server.fs.allow: ['.', '../bmb-wasm/pkg']` to `vite.config.ts`. Removed `vite-plugin-wasm` (not needed for the web target).

### `compiler-wasm.ts` rewrite
Updated for the `--target web` API pattern:
- Dynamic `import('bmb-wasm')` brings in the module (named exports: `check`, `run`, `version`)
- `await mod.default()` initializes the WASM (async default export = init function)
- `wasmApi` stores the bound `check`/`run`/`version` references
- Lazy singleton: `wasmLoading: Promise<void> | null` prevents double-init on concurrent calls

### App.tsx + Header.tsx integration
- Added `wasmVersion: string | null` state; `initWasm()` called on mount via `useEffect`
- `handleRun` dispatches to `compileAndRunWasm(code)` when `isWasmAvailable()`, else falls back to JS interpreter
- `Header.tsx` shows a `WASM` badge (green) or `JS` badge (yellow) based on `wasmVersion` prop

### Example code corrections
All example code (App.tsx default + ExampleGallery.tsx) used `if cond then expr else expr` — not valid in the real BMB parser. Updated all to `if cond { expr } else { expr }`. Binary search example refactored to use block form with `let` bindings.

## Verification & Defect Resolution

**WASM size:** 1.54 MB (under the 5 MB ISSUE target)

**Execution times:**
- First run (includes WASM init + `fetch()`): ~9ms
- Subsequent runs: ~1ms

**Console:** Zero errors, zero warnings in browser console.

**Native tests:** `cargo test --release` 2354/2354 pass after the `wasm_heap` change.

**Functional test:** Default factorial example runs correctly:
- `factorial(5)` → `120`
- `factorial(10)` → `3628800`

## Reflection

**Scope fit:** Complete. WASM runs in browser, integrated with existing UI, fallback to JS if WASM unavailable.

**Latent defects:**
- The `server.fs.allow` fix only applies to `npm run dev`. For a production deployment the WASM file needs to be in the same domain as the playground (copy to `public/` or adjust deployment). This is a deployment concern, not a dev concern.
- `wasm_heap` uses `Layout::from_size_align(size, 8).expect("bad layout")` — panics if alignment is wrong. A zero size is guarded, but technically `align` must be a power of two ≥ 1, which `8` satisfies. No real risk.

**Structural improvement proposals:**
- The `wasm_heap` module's `SIZES` map can grow without bound if BMB code allocates memory but never frees (common in playground demos). A sweep mechanism could help for long-running sessions.
- The WASM binary could be reduced further with `wasm-opt -O3 --strip-debug` — wasm-pack already runs `wasm-opt`, but size could potentially drop more with aggressive dead-code elimination if the bmb crate is trimmed of non-interpreter paths.

**Philosophy drift:** None. No Rust features added; wasm32 platform support is enabling infrastructure.

**Roadmap impact:** ISSUE-20260413 playground-wasm Phase 1 (build infrastructure) AND Phase 2 (UI integration) are both complete in one session.

## Carry-Forward
- Actionable: Verify production build (`npm run build`) copies WASM correctly for deployment
- Structural Improvement Proposals: wasm_heap SIZES map growth; wasm-opt tuning
- Pending Human Decisions: How should the playground be deployed? (GitHub Pages with `/playground/` base? The WASM file needs to be served from the same origin)
- Roadmap Revisions: ISSUE-20260413 playground-wasm → close or mark DONE in ROADMAP.md
- Next Recommendation: Cycle 2805 — production build verification + ISSUE-20260413 close
