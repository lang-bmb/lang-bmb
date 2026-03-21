# Cycle 1935: inttoptr analysis
Date: 2026-03-21

## Inherited → Addressed
- Cycle 1934 clean

## Scope & Implementation
- Analyzed inttoptr counts across all bootstrap IR files:
  - Stage 1 (Rust-compiled): 18 inttoptr — near zero ✅
  - Stage 2 (bootstrap-compiled): 5,638 pre-opt → 2,901 post-opt (49% reduction by LLVM)
  - Stage 3: 5,659 pre-opt → 2,926 post-opt
- Categorized inttoptr by suffix pattern:
  - `_r`: 2,070 (string method right-hand side)
  - `_lp`: 1,336 (load pointer)
  - `_str`: 1,135 (string parameter access)
  - `_p0`: 631 (pointer base for GEP)
- Top sources: `fn_name` (440), `line` (319), `s` (304) — all string parameters passed as i64
- Existing roundtrip elimination only handles ptrtoint→inttoptr pairs (~1-3% coverage)

## Review & Resolution
- **Conclusion**: inttoptr reduction beyond LLVM opt requires Phase C-1 (native ptr type system in MIR)
  - All remaining inttoptr come from the fundamental design: pointers represented as i64
  - Load/store/GEP patterns each generate 1 inttoptr per operation
  - No low-hanging fruit remaining — LLVM already eliminates 49%
- Redirecting remaining cycles to stdlib README update, clippy, and verification

## Carry-Forward
- Pending Human Decisions: Phase C-1 (native ptr type system) is a 6-8 week project requiring separate cycle run
- Discovered out-of-scope: Current inttoptr count is 5,638 pre-opt (not 7,947 as in roadmap — roadmap was outdated)
- Next Recommendation: Update stdlib README, run clippy, full verification
