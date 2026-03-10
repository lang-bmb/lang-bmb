# Cycle 1828: Noalias Metadata for Ptr-Provenance GEP Bases
Date: 2026-03-10

## Inherited → Addressed
From 1827: wave_equation 1.06x PASS — missing noalias metadata on ptr-provenance paths.

## Scope & Implementation

### Root Cause: Noalias Post-Pass Only Detected inttoptr Patterns
The `add_noalias_metadata` post-pass in `llvm_text.rs` only detected inttoptr-based GEP bases (`%base = inttoptr i64 %VAR to ptr`). The ptr-provenance optimization (v0.96.36) uses a different pattern: `%base = load ptr, ptr %VAR.ptr.addr`. This pattern was invisible to the noalias pass, so functions using ptr-provenance got no `!alias.scope` / `!noalias` metadata.

### Fix: Extended Pattern Detection
Added detection for ptr-provenance GEP base patterns in Phase 2 of `add_noalias_metadata`:
- Pattern: `%*_gep_base.N = load ptr, ptr %VAR.ptr.addr`
- Extracts variable name by stripping `.ptr.addr` suffix
- Same downstream processing: finds memory access, maps to base variable

### Result
- **wave_equation**: 0 → 12 noalias annotations on loads/stores
- **dijkstra**: 32 annotations maintained (no regression)
- All 3 distinct array bases (u_prev, u_curr, u_next) correctly scoped

### Files Changed
- `bmb/src/codegen/llvm_text.rs` — Extended `add_noalias_metadata` Phase 2

## Review & Resolution
- All 6,186 tests pass
- No regressions in existing noalias coverage

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Phase 3 — further codegen optimizations, then version bump
