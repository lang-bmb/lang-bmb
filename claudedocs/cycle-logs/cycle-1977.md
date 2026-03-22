# Cycle 1977-1980: Bootstrap @export analysis + compiler improvements
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1973: No carry-forward items

## Scope & Implementation

### Bootstrap @export Analysis
Thorough investigation of bootstrap compiler architecture for @export porting:

**Current state**: Bootstrap compiler has NO @export support
- parser_ast.bmb: Only @likely/@unlikely attributes on expressions
- mir.bmb: No is_export field in function metadata
- lowering.bmb: No attribute propagation for functions
- llvm_ir.bmb: Fixed "private" linkage for all non-main functions

**Porting plan documented** (for future cycles):
1. Parser: Add @export recognition before `fn` keyword
2. MIR: Add is_export metadata to function representation
3. Lowering: Extract @export from AST, propagate to codegen
4. Codegen: Conditional linkage (dllexport on Windows, global on Unix)

**Why deferred**: Bootstrap porting requires changes across 4+ files (parser, parser_ast, mir, lowering, llvm_ir, compiler) with 3-stage verification. This is a dedicated multi-session effort requiring careful analysis of the S-expression AST encoding used by the bootstrap.

### Compiler Bug Fix Applied (from Cycle 1969)
- **@export pre-condition i32 type mismatch**: Already fixed in `llvm_text.rs`
- Parameters narrowed by ConstantPropagationNarrowing now get `sext i32 to i64` before precondition checks

### Dogfooding Discoveries Summary (Cycles 1965-1976)
| Discovery | Where Found | Fix Applied |
|-----------|------------|-------------|
| Loop metadata ID collision | bmb-algo (30+ while loops) | Cycle 1964 |
| @export pre-condition i32 mismatch | bmb-algo (merge_sort, binary_search) | Cycle 1969 |
| Base32 4-byte padding error | bmb-crypto (Base32 encoding) | Cycle 1965 |
| Windows DLL loading needs MSYS2 path | Python bindings | Cycle 1965 |

## Review & Resolution
- Bootstrap @export: Analysis complete, porting plan documented
- All prior compiler fixes verified via cargo test (6,186 pass)
- No new defects found

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Bootstrap @export porting (multi-session dedicated effort)
- Next Recommendation: Quality, packaging, docs
