# Cycle 1865: Bootstrap Codegen — TBAA Metadata + Branch Weights
Date: 2026-03-12

## Inherited → Addressed
From 1863-1864: tower_of_hanoi/max_consecutive_ones LLVM-level issues (not actionable). Bootstrap String nonnull needs MIR type propagation.

## Scope & Implementation
Analyzed codegen gap between Rust compiler and bootstrap compiler. Found 8 significant differences:
1. **No TBAA metadata** on load/store — prevents alias analysis
2. **No branch weights** on loop conditions — hurts code layout
3. **No GEP-based addressing** — uses inttoptr (destroys provenance)
4. Missing `nofree`/`norecurse` on main function
5. No loop metadata (vectorize hints)
6. No ptr-provenance (dual-alloca for malloc)
7. No load_f64/store_f64 inline codegen

**Implemented in this cycle:**
- **TBAA metadata**: Added full C-compatible TBAA hierarchy (!900-!906) to module footer; `!tbaa !903` on all `load_i64`/`store_i64` operations (both `llvm_gen_call` and `llvm_gen_call_reg`)
- **Branch weights**: Added `!prof !907` with 2000:1 body:exit ratio on loop conditional branches (for_body, while_body, loop_body patterns)
- **Main function attributes**: Added `norecurse nounwind` to `bmb_user_main`

Files changed: `bootstrap/compiler.bmb`

## Review & Resolution
- `cargo test --release`: 6,186 tests PASS
- **3-Stage Bootstrap**: Fixed Point VERIFIED (S2 == S3 at 107,708 lines)
- Golden test (accumulator): PASS — correct output
- IR verification: TBAA metadata at module footer, branch weights on loop branches

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope:
  - GEP-based addressing (replace inttoptr) — HIGH IMPACT, next priority
  - load_f64/store_f64 inline codegen missing in bootstrap
  - Loop metadata (vectorize.enable/width) not yet added
  - Ptr-provenance (dual-alloca for malloc results) not yet implemented
- Next Recommendation: Implement GEP-based addressing to replace inttoptr in load/store codegen
