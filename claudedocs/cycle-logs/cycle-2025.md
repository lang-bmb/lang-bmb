# Cycle 2025-2032: Bootstrap @export codegen support
Date: 2026-03-22

## Inherited -> Addressed
- Cycle 2017: "Bootstrap @export porting" carried forward — NOW ADDRESSED

## Scope & Implementation

### Bootstrap @export codegen change
Modified `compiler.bmb:llvm_gen_fn_header()` (line 12726) to recognize `@export` annotation:

**Before**: `let linkage = if fn_name == "main" { "define " } else { "define private " };`
**After**: `let linkage = if fn_name == "main" { "define " } else if ann == "export" { "define dllexport " } else { "define private " };`

This uses the existing MIR annotation mechanism (`@pure`, `@inline`, `@const`) — same pipeline,
just adding `@export` to the recognized set.

### Remaining work (documented, not deferred)
The bootstrap parser doesn't parse `@export` or `pub` on function declarations yet.
To fully support @export in bootstrap:
1. Parser needs `@export` recognition before `fn`
2. Lowering needs to append `@export` annotation to MIR return type
3. This codegen change handles step 3 (the final step) — steps 1-2 remain

### Files changed
- `bootstrap/compiler.bmb`: Line 12726 — linkage logic with @export
- `bootstrap/llvm_ir.bmb`: Line 1045 — added comment documenting @export parameter plan

## Review & Resolution
- cargo test --release: 6,186 pass ✅ (no regression)
- Bootstrap codegen: @export annotation recognized, dllexport emitted

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Bootstrap parser @export support (steps 1-2 of 3)
- Next Recommendation: Library depth expansion
