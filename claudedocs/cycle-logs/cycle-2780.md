# Cycle 2780: D2 — Bootstrap linker stack flag (64MB)
Date: 2026-05-12

## Re-plan

Carry-forward from Cycle 2779: D2 bootstrap parser stack fix — `-Wl,--stack=64M` linker flag.
Plan valid. ⚪ NONE.

## Scope & Implementation

**Target**: prevent STATUS_STACK_OVERFLOW in BMB-compiled binaries on Windows.
Default Windows thread stack = 1MB; bootstrap parser recursion depth exceeds this on
moderately complex files.

**Fix location** — `bmb/src/build/mod.rs` text backend link path (~line 1143):
```rust
#[cfg(target_os = "windows")]
if clang_is_mingw {
    cmd.args(["-static", "-static-libgcc"]);
    // D2 (Cycle 2780): 64MB stack — prevents STATUS_STACK_OVERFLOW on deeply
    // nested ASTs (bootstrap parser recursion depth). Default Windows stack is 1MB.
    cmd.arg("-Wl,--stack,67108864");
}
```

**Diagnostic note**: prior session had added the flag to `link_native()` (the
`#[cfg(feature = "llvm")]`-gated path), which is NOT the active code path for standard
builds. The text backend link path at line 1088–1200 is the active path — flag correctly
placed here.

**Verification** (via `objdump -p bootstrap/stage1.exe`):
```
SizeOfStackReserve 0000000004000000  (= 64 MB)
```

**Regression check**: fibonacci ✅ `{"type":"build_success",...}`, mandelbrot ✅.

## Verification & Defect Resolution

### Stack flag: ✅ complete

Flag confirmed in PE header. Simple and moderately complex files compile correctly.

### Deeper discovery: hash_table still crashes at 64MB

`ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb` (226 LOC) causes
STATUS_STACK_OVERFLOW in stage1.exe even with 64MB stack. Pattern: `while` loop +
`if/else if/else` chain + `@inline` attribute. The Rust compiler handles it without issue.

Root cause: bootstrap parser uses unbounded recursive descent; recursion depth grows with
nesting depth. 64MB is not enough for this specific pattern.

**Resolution**: D2 linker flag scope is complete (1MB→64MB, a strict improvement). The
deeper recursion issue is documented in
`claudedocs/issues/ISSUE-20260512-bootstrap-stack-depth-hash_table.md` (P1, separate
investigation). `ulimit -s unlimited` does not help — PE stack reserve must be set at link
time.

## Reflection

Scope fit: ✅ D2 flag objective met.
Philosophy drift: none — minimal-patch, no workaround (Rule 6 exception applies per D3 context).
Roadmap impact: ISSUE created for deeper recursion (new P1); D2 flag delivers immediate
improvement even without resolving hash_table.
User-facing quality: Windows BMB-compiled binaries now have 64MB stack; crashes on simple
files are eliminated. hash_table-class programs still problematic — documented.

## Carry-Forward

- Actionable: D3 — CLAUDE.md Rule 6 P0-exception clause (Cycle 2781)
- Structural Improvement Proposals: None
- Pending Human Decisions:
  - D5-A workflow push final approval (CI change)
  - D7 (npm + PyPI publish)
  - D8 (M4-1 B baseline with BMB_BENCH_API_KEY)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2781 — D3 Rule 6 P0-exception documentation
