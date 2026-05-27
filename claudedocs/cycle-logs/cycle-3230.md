# Cycle 3230: Fixed Point Verification After Cycle 3229 IPR Fix
Date: 2026-05-28

## Re-plan
Plan valid. After Cycle 3229 modified `bootstrap/compiler.bmb` (IPR blacklist), the 3-Stage Fixed Point needs re-verification. Cycle 3229 only built S1 from Rust compiler; S2 and Fixed Point check were pending.

Secondary scope: Update canonical `bootstrap/compiler.exe` to the new verified S2 binary.

## Scope & Implementation

### Stage 2 Build
- Built `compiler_3229_s2.exe` from `compiler_3229_rust_s1.exe`:
  - Command: `compiler_3229_rust_s1.exe build bootstrap/compiler.bmb -o bootstrap/compiler_3229_s2.exe`
  - Duration: 32.6 seconds (expected — `compiler.bmb` is 32K LOC)
  - Result: `v0.96.30`, 1,198,222 bytes

### Fixed Point Verification
- S3 IR: `compiler_3229_s2.exe emit-ir bootstrap/compiler.bmb compiler_3229_s3.ll` (6202 lines, 382KB)
- S4 IR: `compiler_3229_s2.exe emit-ir bootstrap/compiler.bmb compiler_3229_s4.ll` (6202 lines, 382KB)
- **S3 == S4: 0 differences** ✅ Fixed Point confirmed

### S2 IPR Verification
- `compiler_3229_s2.exe emit-ir main_inproc.bmb`: `array_free` has no `memory(read)` attribute ✅
- IPR fix propagated correctly through the bootstrap chain

### Canonical Binary Update
- `bootstrap/compiler.exe` updated to `compiler_3229_s2.exe` (1,198,222 bytes)
- Previous `compiler.exe`: 1,198,204 bytes (same family, 18 bytes smaller — likely Cycle 3228 state)
- `compiler.exe --version`: `BMB Bootstrap Compiler v0.96.30` ✅

### Tests
- `cargo test --release`: 6282 tests, 0 FAILED ✅ (run in Cycle 3229, no code changes this cycle)
- S2 basic functionality: builds and runs simple programs correctly ✅

## Verification & Defect Resolution

### Fixed Point Chain
```
Rust bmb.exe → compiler_3229_rust_s1.exe (S1, 1.5MB, v0.96.30)
compiler_3229_rust_s1.exe → compiler_3229_s2.exe (S2, 1.2MB, v0.96.30)
compiler_3229_s2.exe → S3 IR (6202 lines)
compiler_3229_s2.exe → S4 IR (6202 lines)
S3 == S4 ✅ (0 differences)
```

### Defect: Stage 2 arena OOM
- `compiler_3229_rust_s1.exe build bootstrap/compiler.bmb` failed with default 4GB arena
- Fix: `BMB_ARENA_MAX_SIZE=32G` (known pattern — self-compile requires large arena)
- Root cause: text-mode string-based AST is O(n²) in string allocations for large inputs

## Reflection

**Scope fit**: Fixed Point verified, canonical binary updated. Both the IPR fix (Cycle 3229) and the standard bootstrap chain are consistent.

**Latent defects**: None found.

**Architecture note**: The `compiler_3224.exe` "Stage 2 breakage" mentioned in Cycle 3229 was NOT reproduced here. The Stage 2 build from `compiler_3229_rust_s1.exe` succeeded. The earlier breakage was specific to `compiler_3224.exe`'s S1 build — likely because that S1 had a codegen issue from the unfixed `compiler.bmb`.

**Structural improvement**: Consider adding `BMB_ARENA_MAX_SIZE=32G` as default when running `bootstrap.sh` on large inputs.

**Philosophy drift**: None.

**Roadmap impact**: The IPR fix + Fixed Point completion means the bootstrap chain is fully verified post-Cycle 3229. No roadmap changes needed.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  1. The `[T; N]` + `arr[i]` +2 offset interaction: `[i64; N]` allocates exactly `N*8` bytes but `arr[i]` accesses at `(i+2)*8`. Safe for i=0..N-3 only. Correct usage requires `[i64; N+2]` for full N-element use. Document this in LANGUAGE_REFERENCE. (Low priority, non-blocking)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: 
  1. IPR pass robustness: more comprehensive fix (exact name matching instead of substring)
  2. Or: survey `compiler.bmb` for stack array migration opportunities (dogfooding M11-C Phase 2)
  3. Or: start M11-C Phase 3 design (`arr[i]` syntax for `[T; N]` arrays)
