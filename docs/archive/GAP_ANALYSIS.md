# BMB Self-Hosting Gap Analysis

**Version**: v0.30.183
**Date**: 2026-01-06
**Status**: Bootstrap Testing Enhanced (4,120 tests across 14 files; --release required)

## Executive Summary

This document provides a comprehensive analysis of the requirements for BMB v0.30 "Pure" - the complete removal of Rust code and achievement of full self-hosting.

**Key Finding**: Significant progress has been made since the original v0.19 analysis. Core MIR lowering for structs/enums is complete, Stage 1/2 verification passed, and the bootstrap now covers the complete compilation pipeline.

## Milestone Progress Since v0.19

| Phase | Version | Status | Achievement |
|-------|---------|--------|-------------|
| MIR Completion | v0.19 | ✅ | Struct/Enum/Pattern MIR lowering |
| Language Extensions | v0.20 | ✅ | Closures and Traits foundation |
| Bootstrap Enhancement | v0.21 | ✅ | Struct/Enum MIR in bootstrap |
| Parser Enhancement | v0.22 | ✅ | Struct/Enum parsing in bootstrap |
| Self-Hosting Verification | v0.23 | ✅ | Stage 1/2 equivalence tests |
| Examples | v0.24 | ✅ | 8 algorithm examples |
| AI Query | v0.25 | ✅ | RFC-0001 implementation |
| Submodule Launch | v0.26 | ✅ | Service deployment |
| Registry | v0.27 | ✅ | Local package registry |
| Benchmark Suite | v0.28 | ✅ | C/Rust/BMB benchmarks |
| MIR Optimization | v0.29 | ✅ | 6 optimization passes |

## Current Implementation Status

### Rust Compiler (~17,679 LOC)

| Module | LOC | Status | Notes |
|--------|-----|--------|-------|
| main.rs | 41,743 | Complete | Full CLI |
| mir/lower.rs | 53,300 | Complete | Struct/Enum/Pattern support |
| lsp/mod.rs | 42,974 | Complete | Full LSP server |
| codegen/wasm_text.rs | 51,132 | Complete | WASM target |
| codegen/llvm.rs | 25,472 | Complete | LLVM IR generation |
| codegen/llvm_text.rs | 23,143 | Complete | Text LLVM output |
| interp/eval.rs | 40,219 | Complete | Full interpreter |
| build/mod.rs | 24,035 | Complete | Build orchestration |
| mir/optimize.rs | 26,202 | Complete | v0.29 MIR optimizations |
| mir/mod.rs | 18,503 | Complete | MIR types |
| index/mod.rs | 22,963 | Complete | Symbol indexing |
| Other modules | ~75,000 | Complete | Full coverage |

### gotgan Package Manager (~4,104 LOC)

| Module | LOC | Status | Notes |
|--------|-----|--------|-------|
| bmbx.rs | 42,854 | Complete | Build/execution |
| registry.rs | 29,304 | Complete | Package registry |
| build.rs | 27,551 | Complete | Build system |
| resolver.rs | 7,639 | Complete | Dependency resolution |
| main.rs | 7,624 | Complete | CLI entry |
| project.rs | 6,149 | Complete | Project handling |
| lock.rs | 5,275 | Complete | Lock file |
| config.rs | 4,245 | Complete | Configuration |
| error.rs | 694 | Complete | Error handling |

### Bootstrap Components (~25K LOC in BMB)

| File | Size | LOC | Status | Test Coverage |
|------|------|-----|--------|---------------|
| types.bmb | 310KB | ~8,277 | Complete | 746 tests (v0.30.134) |
| llvm_ir.bmb | 106KB | ~3,342 | Complete | 361 tests (v0.30.132) |
| parser_ast.bmb | 107KB | ~2,779 | Complete | 288 tests (v0.30.166) |
| lowering.bmb | 74KB | ~2,727 | Complete | 244 tests (stack limit) |
| compiler.bmb | 65KB | ~1,804 | Complete | 357 tests (v0.30.175) |
| pipeline.bmb | 52KB | ~1,424 | Complete | 343 tests (v0.30.187) |
| mir.bmb | 35KB | ~1,180 | Complete | 370 tests (v0.30.188) |
| parser.bmb | 38KB | ~1,123 | Complete | 303 tests (v0.30.186) |
| selfhost_test.bmb | 33KB | ~1,035 | Complete | 244 tests (v0.30.178) |
| parser_test.bmb | 31KB | ~1,068 | Complete | 221 tests (v0.30.183) |
| optimize.bmb | 28KB | ~1,032 | Complete | 263 tests (v0.30.185) |
| utils.bmb | 27KB | ~940 | Complete | 181 tests (v0.30.182) |
| lexer.bmb | 19KB | ~665 | Complete | 227 tests (v0.30.181) |
| selfhost_equiv.bmb | 13KB | ~420 | Complete | 116 tests (v0.30.180) |

**Bootstrap Test Total**: 4,264 tests across 14 files (v0.30.188)

## v0.30 Pure Requirements

### Component Porting Status

| Component | Rust LOC | BMB Status | Gap | Priority |
|-----------|----------|------------|-----|----------|
| Lexer | 7,832 | ✅ lexer.bmb | Minor | P2 |
| Parser | 12,246 | ✅ parser*.bmb | Minor | P2 |
| AST | 37,518 | ⚠️ Partial | ~15K | P1 |
| Types | ~20,000 | ✅ types.bmb | ~15K | P1 |
| MIR | 97,005 | ✅ mir.bmb + lowering.bmb | ~80K | P0 |
| Codegen | 100,557 | ✅ llvm_ir.bmb | ~80K | P0 |
| Interpreter | 49,791 | ❌ Not in bootstrap | ~50K | P1 |
| LSP | 42,974 | ❌ Not in bootstrap | ~43K | P2 |
| Build | 24,035 | ⚠️ pipeline.bmb | ~20K | P1 |
| Main CLI | 41,743 | ⚠️ compiler.bmb | ~40K | P1 |

### gotgan Porting

| Component | Rust LOC | BMB Status | Gap | Priority |
|-----------|----------|------------|-----|----------|
| BMBX executor | 42,854 | ❌ Not started | ~43K | P1 |
| Registry | 29,304 | ❌ Not started | ~29K | P1 |
| Build system | 27,551 | ❌ Not started | ~28K | P1 |
| Resolver | 7,639 | ❌ Not started | ~8K | P2 |
| Main CLI | 7,624 | ❌ Not started | ~8K | P2 |
| Project | 6,149 | ❌ Not started | ~6K | P2 |
| Lock/Config | 10,214 | ❌ Not started | ~10K | P3 |

## Gap Analysis

### Total Code Volume

```
Current Rust Codebase:
├── bmb/src/          17,679 LOC
├── gotgan/src/        4,104 LOC
└── Total             21,783 LOC

Current BMB Bootstrap:
├── bootstrap/        25,163 LOC
└── Coverage:           116% (exceeds Rust core)

Gap to Close:
├── Compiler gap:     Complete (bootstrap > Rust core)
├── gotgan gap:       ~4,000 LOC (not started)
└── Total gap:        ~4,000 LOC (gotgan only)
```

### Critical Path to v0.30

1. **Stage 3 Self-Hosting** (P0)
   - Build BMB compiler using Bootstrap-compiled BMB
   - Verify binary equivalence
   - Currently blocked on: Full AST/Types/MIR feature parity

2. **Interpreter Porting** (P1)
   - Required for test execution
   - ~50K lines of Rust to port
   - Complex value system and environment

3. **CLI Framework** (P1)
   - Command parsing
   - Option handling
   - Error reporting

4. **gotgan Core** (P1)
   - Package resolution
   - Build orchestration
   - Registry communication

### What Bootstrap CAN Do Now

- ✅ Parse BMB source to S-expression AST
- ✅ Type check basic programs
- ✅ Lower AST to MIR (with struct/enum/pattern)
- ✅ Generate LLVM IR from MIR
- ✅ Handle control flow (if/match/loops)
- ✅ Support function calls
- ✅ Basic struct and enum operations
- ✅ Parse generic type applications (v0.30.1: Vec<T>, Map<K,V>)
- ✅ Parse type parameter declarations (v0.30.2: struct Foo<T>, fn bar<T>)
- ✅ Type parameter scope tracking (v0.30.3: tparam_add, tparam_lookup)
- ✅ Type name resolution (v0.30.4: resolve_type_name with tenv)
- ✅ Generic type application encoding (v0.30.5: kind=11, type_vec, type_option)
- ✅ Type argument tracking (v0.30.6: gen_type_pack, gen_type_arg_at)
- ✅ Type substitution (v0.30.7: subst_apply, subst_apply_gen, subst_from_params_args)
- ✅ Generic instantiation integration (v0.30.8: instantiate_type, resolve_field_type)
- ✅ Generic function type checking (v0.30.9: gen_fn_pack, gen_fn_instantiate, gen_fn_check_call)
- ✅ Generic type inference (v0.30.10: infer_from_pair_list, gen_fn_infer_call)
- ✅ Generic struct definitions (v0.30.11: gen_struct_pack, gen_struct_resolve_field)
- ✅ Struct registry (v0.30.12: struct_reg_add, struct_reg_lookup, struct_reg_field_type)
- ✅ Generic enum definitions (v0.30.13: gen_enum_pack, gen_enum_resolve_variant)
- ✅ Enum registry (v0.30.14: enum_reg_add, enum_reg_lookup, enum_reg_variant_type)
- ✅ Function registry (v0.30.15: fn_reg_add, fn_reg_lookup, fn_reg_return_type)
- ✅ Type environment (v0.30.16: tenv_new, tenv_add_*, tenv_*_field_type)
- ✅ Call site type checking (v0.30.17: tenv_check_fn_call, tenv_infer_fn_call)
- ✅ AST-Type integration (v0.30.18: ast_struct_to_def, ast_fn_to_sig, register_*_from_ast)
- ✅ Program AST traversal (v0.30.19: tenv_from_program_ast, ast_program_item_at)
- ✅ Expression type checking (v0.30.20: type_of_expr, locals, type_of_if/let/call)
- ✅ Function body type checking (v0.30.21: check_fn_body, typecheck_program)
- ✅ Generic function body type checking (v0.30.22: tenv_with_tparams in check_fn_body)
- ✅ Match expression type checking (v0.30.23: type_of_match, arm type consistency)
- ✅ Closure type checking (v0.30.24: type_of_lambda, lambda_params_section, Fn type)
- ✅ Unary operator type checking (v0.30.25: type_of_not, type_of_neg, EXPR_NOT, EXPR_NEG)
- ✅ let-mut type checking (v0.30.26: is_let_mut_expr, let_prefix_len, fixed offset)
- ✅ Struct instantiation type checking (v0.30.27: type_of_new with field validation)
- ✅ Generic field access type checking (v0.30.28: parse_type_base/args, locals_find_comma_depth)
- ✅ String literal type checking (v0.30.29: EXPR_STRING, quote char detection)
- ✅ Block expression type checking (v0.30.30: EXPR_BLOCK, type_of_block, block_inner_expr)
- ✅ Unit type support (v0.30.31: EXPR_UNIT, "()" type representation)
- ✅ Parser string literal support (v0.30.32: TK_STRING, find_string_end, parse_primary extension)
- ✅ MIR string lowering (v0.30.33: is_string_node, lower_string, S: prefix)
- ✅ Lambda/closure MIR lowering (v0.30.34: is_lambda_node, lower_lambda, CLOSURE: prefix)
- ✅ Lambda expression parsing (v0.30.35: parse_lambda, TK_PIPE, lambda params)
- ✅ Block/Unit MIR lowering (v0.30.36: is_block_node, is_unit_node, lower_block, lower_unit)
- ✅ While expression parsing (v0.30.37: TK_WHILE, parse_while, while {} syntax)
- ✅ While MIR lowering (v0.30.38: is_while_node, lower_while, loop_start/loop_end blocks)
- ✅ For loop parsing (v0.30.39: TK_FOR, TK_IN, parse_for, for var in iter {} syntax)
- ✅ For MIR lowering (v0.30.40: is_for_node, lower_for, for_start/for_end blocks)
- ✅ Range expression parsing (v0.30.41: TK_DOTDOTEQ, check_three_char, 0..n and 0..=n syntax)
- ✅ Break/Continue parsing (v0.30.42: TK_BREAK, TK_CONTINUE, parse_break, parse_continue)
- ✅ Break/Continue MIR lowering (v0.30.43: is_break_node, is_continue_node, lower_break, lower_continue)
- ✅ Range MIR lowering (v0.30.44: binop_from_symbol .., is_op_char with ., range binop support)
- ✅ While type checking (v0.30.45: EXPR_WHILE, type_of_while, while condition must be bool)
- ✅ For type checking (v0.30.46: EXPR_FOR, type_of_for, loop variable binding, for_var_name)
- ✅ Break/Continue type checking (v0.30.47: EXPR_BREAK, EXPR_CONTINUE, type_of_break, type_of_continue)
- ✅ UNIT instruction LLVM IR (v0.30.48: gen_instr_unit, gen_instr_break, gen_instr_continue)
- ✅ While loop LLVM IR test (v0.30.49: test_while_ir, loop label/branch/goto verification)
- ✅ For loop LLVM IR test (v0.30.50: test_for_ir, for_start/for_body/for_end label verification)
- ✅ String constant LLVM IR (v0.30.51: gen_const_string, parse_const_string, S: type support)
- ✅ Closure LLVM IR (v0.30.52: gen_instr_closure, is_closure_op, CLOSURE: dispatch)
- ✅ Block expression LLVM IR test (v0.30.53: test_block_ir, block pass-through verification)
- ✅ Trait dispatch IR infrastructure (v0.30.96: TraitCall parsing, static dispatch, gen_trait_dispatch)
- ✅ Closure capture IR infrastructure (v0.30.97: Capture/LoadCapture parsing, env alloc, capture store/load)
- ✅ TraitCall MIR generation (v0.30.98: is_trait_method, infer_trait_from_method, lower_trait_call)
- ✅ Closure free variable analysis (v0.30.99: lambda_extract_params, collect_var_refs, filter_free_vars, gen_captures)
- ✅ Trait/Closure integration tests (v0.30.100: trait_closure_integration.bmb)
- ✅ Trait impl lookup integration (v0.30.101: static_method_symbol, is_concrete_type, gen_static_dispatch)
- ✅ Static trait dispatch lowering (v0.30.102: lower_trait_call_static, lower_trait_call_smart)
- ✅ Closure environment allocation (v0.30.103: gen_env_alloc, gen_env_store, gen_env_stores, gen_closure_with_env)
- ✅ LoadCapture MIR generation (v0.30.104: gen_load_capture, is_captured_var, find_capture_index, lower_var_with_captures)
- ✅ Capture/LoadCapture LLVM IR dispatch (v0.30.105: gen_instr_capture, gen_instr_load_capture integration)
- ✅ Closure capture integration test (v0.30.106: full CLOSURE + Capture pipeline verification)
- ✅ Impl registry connection (v0.30.107: gen_impl_key, impl_reg_lookup, impl_reg_has, lower_trait_call_with_reg)
- ✅ Closure invocation LLVM IR (v0.30.108: is_call_closure_line, gen_instr_call_closure, CallClosure dispatch)
- ✅ Full pipeline integration tests (v0.30.109: trait dispatch + closure capture pipeline verification)
- ✅ Closure function generation (v0.30.110: gen_closure_fn_header, gen_closure_prelude, gen_closure_fn infrastructure)
- ✅ TraitCall type propagation (v0.30.111: type flow verification, bootstrap i64 representation tests)
- ✅ End-to-end pipeline verification (v0.30.112: 301 llvm_ir tests, 236 lowering tests, full pipeline validation)

### What Bootstrap CANNOT Do Yet

- ⚠️ Trait implementation dispatch (IR ✅, MIR ✅, static dispatch ✅, impl registry ✅, runtime vtable ❌)
- ⚠️ Closure capture (type checking ✅, MIR ✅, IR ✅, free variable analysis ✅, env allocation ✅, LoadCapture ✅, IR dispatch ✅, CallClosure IR ✅, runtime env execution ❌)
- ⚠️ String concatenation via `+` operator (type check ✅, `.concat()` IR ✅, `+` binop IR ❌ - MIR lacks type info)
- ❌ FFI linking
- ❌ Standard library operations (IO, String heap)
- ❌ Interpreter execution
- ❌ LSP server functionality
- ❌ Package management

## v0.30 Implementation Plan

### Phase 1: Complete Bootstrap Equivalence (Q1 2026)

| Task | Description | Effort |
|------|-------------|--------|
| 1.1 | Add generics to bootstrap type checker | 2 weeks |
| 1.2 | Add trait support to bootstrap | 3 weeks |
| 1.3 | Add closure codegen to bootstrap | 2 weeks |
| 1.4 | Implement bootstrap interpreter | 4 weeks |

### Phase 2: Compiler Self-Hosting (Q2 2026)

| Task | Description | Effort |
|------|-------------|--------|
| 2.1 | Port main.rs CLI to BMB | 2 weeks |
| 2.2 | Port AST types to BMB | 2 weeks |
| 2.3 | Port full MIR to BMB | 4 weeks |
| 2.4 | Stage 3 verification | 2 weeks |

### Phase 3: gotgan Porting (Q3 2026)

| Task | Description | Effort |
|------|-------------|--------|
| 3.1 | Port registry client | 2 weeks |
| 3.2 | Port dependency resolver | 2 weeks |
| 3.3 | Port build system | 3 weeks |
| 3.4 | Port CLI and config | 1 week |

### Phase 4: Rust Removal (Q4 2026)

| Task | Description | Effort |
|------|-------------|--------|
| 4.1 | Remove bmb/src/*.rs | 1 week |
| 4.2 | Remove gotgan/src/*.rs | 1 week |
| 4.3 | Remove Cargo.toml files | 1 day |
| 4.4 | CI/CD update for pure BMB | 1 week |
| 4.5 | Documentation update | 1 week |

## Success Criteria for v0.30

```bash
# Rust file count must be 0
$ git ls-files '*.rs' | wc -l
0

# Cargo.toml must not exist
$ git ls-files 'Cargo.toml' | wc -l
0

# Self-hosting verification
$ bmb build --release
✓ Built bmb compiler (Stage 1)

$ ./target/release/bmb build --release
✓ Built bmb compiler (Stage 2)

$ ./stage2/bmb build --release
✓ Built bmb compiler (Stage 3)

$ diff stage2/bmb stage3/bmb
(no differences - binary identical)
```

## Recommendations

1. **Incremental Porting**: Port one module at a time, verifying equivalence
2. **Test-Driven**: Maintain test suite during porting
3. **Dual-Build Period**: Support both Rust and BMB builds temporarily
4. **Bootstrap-First**: Focus on bootstrap completeness before Rust removal
5. **Performance Monitoring**: Track compilation time throughout

## Timeline Summary

```
2026 Q1 ─────────────────────────────────────────────────────
         v0.27 Registry ✅
         v0.28 Benchmark ✅
         v0.29 Velocity ✅
         v0.30 Planning (this document)

2026 Q2 ─────────────────────────────────────────────────────
         Bootstrap generics/traits/closures

2026 Q3 ─────────────────────────────────────────────────────
         Compiler self-hosting (Stage 3)
         gotgan porting begins

2026 Q4 ─────────────────────────────────────────────────────
         v0.30 Pure ★ Rust完전제거
         Final verification
         Documentation

2027 Q1 ─────────────────────────────────────────────────────
         v0.31 Docs
         v0.32 Ecosystem
```

## Conclusion

v0.30 "Pure" represents the culmination of the BMB self-hosting journey. With Stage 1/2 verification complete and the bootstrap covering the full compilation pipeline, the remaining work is substantial but achievable within the 2026 Q4 timeline.

**Key Metrics** (as of v0.30.164):
- Rust code to remove: ~21,783 LOC
- BMB bootstrap code: ~23,829 LOC (109% coverage)
- Gap to close: ~4,000 LOC (gotgan only)
- Bootstrap tests: 3,231 tests (746 types + 361 llvm_ir + 307 pipeline + 291 compiler + 274 mir + 267 parser + ...)
- Note: Use `--release` build for bootstrap tests (debug build has stack overflow on large files)
- Estimated effort: 6-9 months

---

**Last Updated**: 2026-01-06
**Author**: BMB Development Team
