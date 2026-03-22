# Cycle 2065-2072: Bootstrap @export full pipeline (parser + lowering + codegen)
Date: 2026-03-22

## Inherited -> Addressed
- Cycle 2051: "Bootstrap parser @export" — NOW FULLY ADDRESSED

## Scope & Implementation

### Bootstrap @export: Complete 3-file implementation

**parser.bmb** (validation path):
- `parse_program`: Now handles `@export [pub] fn` by skipping @, identifier, optional pub

**pipeline.bmb** (AST builder):
- `parse_program`: Handles `@export [pub] fn` with annotation injection
- `annotate_fn_export`: Appends `@export` to return type in AST
- Helper functions: `find_params_end`, `find_ret_end`

**compiler.bmb** (codegen):
- `parse_program_sb`: Handles `@annotation [pub] fn` (skips optional pub between annotation and fn)
- `llvm_gen_fn_header`: Recognizes `@export` annotation → emits `dllexport` linkage (from Cycle 2025)

### Full @export pipeline
```
@export pub fn foo(x: i64) -> i64 = ...
   ↓ parser.bmb: validates @export pub fn syntax
   ↓ pipeline.bmb: builds AST "(fn <foo> (params) i64 @export (body))"
   ↓ lowering.bmb: MIR includes @export in return type annotation
   ↓ compiler.bmb: find_mir_annotation detects "export"
   ↓ llvm_gen_fn_header: emits "define dllexport ..." instead of "define private ..."
```

## Review & Resolution
- cargo test --release: 6,186 pass ✅ (zero regression across all 3 file changes)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: selfhost_test.bmb has duplicate parse_program (not updated, tests-only)
- Next Recommendation: More library expansion + edge case testing
