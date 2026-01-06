# Bootstrap Feature Gap Analysis

> Version: v0.30.221
> Date: 2025-01-06
> Purpose: Document gaps between Rust compiler and BMB bootstrap implementation

## Executive Summary

The BMB bootstrap implements the **complete core compilation pipeline** (lexer → parser → type checker → MIR → LLVM IR) with **914 test functions** across 14 files. All P0 features for self-hosting are complete. Remaining gaps are **interpreter** (P1), **verification** (P2), and **tooling** (P3).

## Module Comparison Matrix

| Component | Rust Module | Bootstrap File | Status | Test Count |
|-----------|-------------|----------------|--------|------------|
| Lexer | `lexer/mod.rs`, `lexer/token.rs` | `lexer.bmb` | ✅ Complete | 40 |
| Parser | `parser/mod.rs` | `parser.bmb`, `parser_ast.bmb`, `parser_test.bmb` | ✅ Complete | 216 |
| AST Types | `ast/*.rs` | `parser_ast.bmb` | ✅ Partial | (included above) |
| Type Checker | `types/mod.rs` | `types.bmb` | ✅ Generics+Tuples (v0.30.217) | 173 |
| MIR | `mir/mod.rs` | `mir.bmb` | ✅ Complete | 59 |
| Lowering | `mir/lower.rs` | `lowering.bmb` | ✅ Complete | 4 (stack limited) |
| Optimizer | `mir/optimize.rs` | `optimize.bmb` | ✅ Complete | 56 |
| LLVM Codegen | `codegen/llvm.rs`, `codegen/llvm_text.rs` | `llvm_ir.bmb` | ✅ Complete | 80 |
| Pipeline | (main.rs) | `pipeline.bmb`, `compiler.bmb` | ✅ Complete | 117 |
| SMT Solver | `smt/*.rs` | ❌ Not Implemented | Gap (P2) | - |
| Verifier | `verify/*.rs` | ❌ Not Implemented | Gap (P2) | - |
| Interpreter | `interp/*.rs` | ❌ Not Implemented | Gap (P1) | - |
| REPL | `repl/mod.rs` | ❌ Not Implemented | Gap (P3) | - |
| LSP | `lsp/mod.rs` | ❌ Not Implemented | Gap (P3) | - |
| Resolver | `resolver/mod.rs` | ❌ Not Implemented | Gap (P3) | - |
| Derive | `derive/mod.rs` | ❌ Not Implemented | Gap (P3) | - |
| CFG | `cfg/mod.rs` | ❌ Not Implemented | Gap (P3) | - |
| Query/Index | `query/mod.rs`, `index/mod.rs` | ❌ Not Implemented | Gap (P3) | - |
| Build | `build/mod.rs` | ❌ Not Implemented | Gap (P3) | - |
| Utils | - | `utils.bmb` | ✅ Complete | 74 |
| Self-host Tests | - | `selfhost_test.bmb`, `selfhost_equiv.bmb` | ✅ Complete | 95 |

**Total Bootstrap Tests: 914**

## Priority Feature Gaps

### P0 (Critical for Self-Hosting) - ✅ ALL COMPLETE

#### 1. Trait Support in Bootstrap Type Checker
**Status**: ✅ Complete (v0.30.211+)

**Bootstrap Implementation** (`types.bmb`):
- `trait_reg_*` - Trait registry with method signatures
- `impl_reg_*` - Implementation registry with type mapping
- `type_satisfies_trait()` - Trait satisfaction checking
- `lookup_trait_for_method()` - Method dispatch resolution
- `type_of_trait_call()` - Trait call type inference
- `check_trait_call()` - Trait call validation
- Tests: `test_trait_pack`, `test_trait_reg_add`, `test_impl_reg_add`, etc.

#### 2. Complete Generics Type Checker
**Status**: ✅ Complete (v0.30.217)

**Bootstrap Implementation** (`types.bmb` - 173 tests, 821 assertions):
- Type parameter tracking ✅ (v0.30.3-v0.30.12)
- Generic type application encoding ✅ (Vec<T>, Option<T>, Map<K,V>)
- Type substitution ✅ (single/multi params)
- Type argument inference ✅ (basic patterns)
- Generic struct/enum/fn instantiation ✅
- Trait bounds checking ✅ (type_satisfies_bounds)
- Nested generic types ✅ (packing/unpacking)
- Nested generic substitution ✅ (recursive, v0.30.213)
- Tuple type substitution ✅ (`(A,B)` → `(i64,String)`, v0.30.217)

#### 3. Closure Codegen in Bootstrap
**Status**: ✅ Complete (v0.30.108)

**Bootstrap Implementation**:
- `lowering.bmb`: Closure MIR generation ✅ (v0.30.34), Environment capture ✅ (v0.30.99)
- `llvm_ir.bmb`: Full closure IR support ✅
  - `gen_instr_closure()` - Basic closure representation (v0.30.52)
  - `gen_closure_env_alloc()` - Environment allocation (v0.30.97)
  - `gen_closure_with_captures()` - Closure struct creation (v0.30.97)
  - `gen_instr_call_closure()` - Closure invocation (v0.30.108)
  - Tests: `test_closure_ir`, `test_closure_capture_ir`

### P1 (Important for Complete Toolchain)

#### 4. Bootstrap Interpreter
**Status**: Not Implemented (ROADMAP 30.1.4)

**Rust Implementation** (`interp/*.rs`):
- `eval.rs`: Expression evaluation
- `env.rs`: Environment management
- `value.rs`: Runtime value representation
- `error.rs`: Runtime errors

**Bootstrap Gap**:
- No interpreter in bootstrap
- Tests run via Rust interpreter currently

**Required Work**:
1. Create `interp.bmb` with value encoding
2. Implement expression evaluator
3. Add environment/scope management
4. Enable self-testing without Rust

### P2 (Verification System)

#### 5. SMT Integration
**Status**: Not Implemented

**Rust Implementation** (`smt/*.rs`):
- `translator.rs`: AST → SMT-LIB2
- `solver.rs`: Z3 process communication

**Bootstrap Gap**:
- Contract verification not in bootstrap
- Would require external process calls

#### 6. Contract Verifier
**Status**: Not Implemented

**Rust Implementation** (`verify/*.rs`):
- `mod.rs`: Verification orchestration
- `contract.rs`: Contract checking logic

### P3 (Tooling - Post Self-Hosting)

| Component | Priority | Reason |
|-----------|----------|--------|
| LSP Server | P3 | IDE integration (can use Rust LSP initially) |
| REPL | P3 | Interactive development (Rust REPL works) |
| Module Resolver | P3 | Multi-file projects (basic in pipeline.bmb) |
| Derive Macros | P3 | Code generation convenience |
| CFG Builder | P3 | Advanced optimization |
| Query System | P3 | AI tooling (RFC-0001 implemented in Rust) |

## Test Coverage Analysis

### High Coverage (>50 tests)
| File | Tests | Key Functions |
|------|-------|---------------|
| types.bmb | 173 | Type checking, generics, traits, tuples (v0.30.217) |
| parser_ast.bmb | 119 | S-expression AST |
| llvm_ir.bmb | 80 | LLVM IR generation, closures (v0.30.108) |
| utils.bmb | 74 | String utilities |
| compiler.bmb | 63 | Compilation coordination |
| selfhost_test.bmb | 62 | Self-hosting verification |
| mir.bmb | 59 | MIR representation |
| optimize.bmb | 56 | MIR optimization |
| pipeline.bmb | 54 | End-to-end pipeline |
| parser_test.bmb | 54 | Parser validation |

### Medium Coverage (20-50 tests)
| File | Tests | Notes |
|------|-------|-------|
| parser.bmb | 43 | Grammar parsing |
| lexer.bmb | 40 | Tokenization |
| selfhost_equiv.bmb | 33 | Equivalence testing |

### Low Coverage (<20 tests)
| File | Tests | Reason |
|------|-------|--------|
| lowering.bmb | 4 | Stack overflow limitation |

## Recommendations

### Next Priority (v0.30.221+)

1. **Bootstrap Interpreter** (P1)
   - Create `interp.bmb` with value encoding
   - Enable running bootstrap tests without Rust
   - Self-verification capability for true self-hosting

2. **Lowering Test Coverage**
   - Increase lowering.bmb tests (currently 4)
   - Address stack overflow limitations
   - Better MIR generation coverage

### Future Work (Post Self-Hosting)

3. **Verification System** (P2)
   - SMT-LIB2 translation for contracts
   - Z3 integration for verification

4. **Tooling** (P3)
   - LSP server for IDE integration
   - REPL for interactive development
   - Module resolver for multi-file projects

## Appendix: Bootstrap File Dependencies

```
utils.bmb (no deps)
    │
lexer.bmb ← parser.bmb ← parser_ast.bmb
    │                          │
    └──────────────────────────┼──────────> types.bmb
                               │                │
                               └──> lowering.bmb ←──> mir.bmb
                                          │
                                          └──> llvm_ir.bmb
                                                   │
                                          optimize.bmb
                                                   │
                               pipeline.bmb ← compiler.bmb
                                          │
                               selfhost_test.bmb
                               selfhost_equiv.bmb
                               parser_test.bmb
```

## Conclusion

The bootstrap implementation covers **100% of the core compilation pipeline** (P0 complete as of v0.30.221):

✅ **Completed**:
1. **Trait support** - Full trait/impl registry and dispatch (v0.30.211+)
2. **Complete generics** - Type inference, substitution, tuple types (v0.30.217)
3. **Closure codegen** - MIR lowering + LLVM IR emission (v0.30.108)

🔲 **Remaining (P1+)**:
1. **Bootstrap interpreter** (P1) - Enable self-testing without Rust
2. **Verification system** (P2) - SMT integration for contracts
3. **Tooling** (P3) - LSP, REPL, multi-file resolver

The bootstrap is ready for Stage 3 self-hosting verification.
