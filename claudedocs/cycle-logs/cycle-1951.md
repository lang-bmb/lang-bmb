# Cycles 1951-1952: @export attribute + SharedLib output
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1950 clean

## Scope & Implementation

### @export Attribute (Cycle 1951)
- Added `is_export()` method to `Attribute` (ast/mod.rs)
- Added `is_export: bool` to `MirFunction` (mir/mod.rs)
- Propagated in mir/lower.rs: `has_attribute(&fn_def.attributes, "export")`
- Updated codegen: `@export` functions get global visibility (no `private` linkage)
- Auto-patched 150+ MirFunction constructions across optimize.rs, wasm_text.rs, llvm_text.rs

### SharedLib Output (Cycle 1952)
- Added `OutputType::SharedLib` to build/mod.rs
- Added `--shared` CLI flag (main.rs)
- SharedLib build passes `-shared` to clang linker
- Auto-sets output extension: `.dll` (Windows), `.so` (Linux), `.dylib` (macOS)

### E2E Validation
```
BMB source (@export functions)
    ↓ bmb build --shared
.dll with exported symbols
    ↓ C caller via -lexport_test
add(3, 4) = 7 ✅
multiply(5, 6) = 30 ✅
```

## Review & Resolution
- `cargo test --release`: 6,186 pass, 0 fail ✅
- `@export` functions: global visibility (no private linkage) ✅
- Non-@export functions: private linkage preserved ✅
- SharedLib build: .dll generated with correct export table ✅
- C interop: direct function calls work ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: bmb-algo library with @export + Python binding
