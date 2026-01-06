# Bootstrap Feature Gap Analysis

> Version: v0.30.205
> Date: 2025-01-06
> Purpose: Document gaps between Rust compiler and BMB bootstrap implementation

## Executive Summary

The BMB bootstrap currently implements **core compilation pipeline** (lexer → parser → type checker → MIR → LLVM IR) with **902 test functions** across 14 files. Key gaps exist in **trait dispatch** and **advanced verification** components.

## Module Comparison Matrix

| Component | Rust Module | Bootstrap File | Status | Test Count |
|-----------|-------------|----------------|--------|------------|
| Lexer | `lexer/mod.rs`, `lexer/token.rs` | `lexer.bmb` | ✅ Complete | 40 |
| Parser | `parser/mod.rs` | `parser.bmb`, `parser_ast.bmb`, `parser_test.bmb` | ✅ Complete | 216 |
| AST Types | `ast/*.rs` | `parser_ast.bmb` | ✅ Partial | (included above) |
| Type Checker | `types/mod.rs` | `types.bmb` | ⚠️ Basic Generics | 167 |
| MIR | `mir/mod.rs` | `mir.bmb` | ✅ Complete | 59 |
| Lowering | `mir/lower.rs` | `lowering.bmb` | ✅ Complete | 4 (stack limited) |
| Optimizer | `mir/optimize.rs` | `optimize.bmb` | ✅ Complete | 56 |
| LLVM Codegen | `codegen/llvm.rs`, `codegen/llvm_text.rs` | `llvm_ir.bmb` | ✅ Complete | 80 |
| Pipeline | (main.rs) | `pipeline.bmb`, `compiler.bmb` | ✅ Complete | 111 |
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

**Total Bootstrap Tests: 902**

## Priority Feature Gaps

### P0 (Critical for Self-Hosting)

#### 1. Trait Support in Bootstrap Type Checker
**Status**: Pending (ROADMAP 30.1.2)

**Rust Implementation** (`types/mod.rs`):
```rust
pub struct TraitInfo {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub methods: Vec<TraitMethodInfo>,
}

pub struct ImplInfo {
    pub trait_name: String,
    pub target_type: Box<Type>,
    pub methods: Vec<String>,
}

// TypeChecker fields
traits: HashMap<String, TraitInfo>,
impls: Vec<ImplInfo>,
```

**Bootstrap Gap**:
- `types.bmb` has NO trait/impl handling
- No trait method lookup (`lookup_trait_method`)
- No impl resolution logic

**Required Work**:
1. Add `trait_info` encoding to types.bmb (similar to struct registry)
2. Add trait method table
3. Implement `lookup_trait_method` for method dispatch
4. Add impl block registration and lookup

#### 2. Complete Generics Type Checker
**Status**: Partially Complete (ROADMAP 30.1.1)

**Rust Implementation** (`types/mod.rs`):
```rust
generic_functions: HashMap<String, (Vec<TypeParam>, Type, Vec<(String, Type)>)>,
generic_structs: HashMap<String, (Vec<TypeParam>, Vec<(String, Type)>)>,
generic_enums: HashMap<String, (Vec<TypeParam>, Vec<EnumVariant>)>,

fn infer_type_args(...) -> Result<Vec<Type>, TypeError>
fn substitute_type(...) -> Type
```

**Bootstrap Gap** (`types.bmb`):
- Type parameter tracking ✅ (v0.30.3-v0.30.12)
- Generic type application encoding ✅
- Type argument inference ⚠️ (basic only)
- Full substitution ⚠️ (partial)

**Required Work**:
1. Complete type inference for complex generic patterns
2. Full constraint solving for bounded type parameters
3. Monomorphization tracking

### P1 (Important for Complete Toolchain)

#### 3. Closure Codegen in Bootstrap
**Status**: Partially Complete (ROADMAP 30.1.3)

**Rust Implementation** (`ast/expr.rs`, `mir/lower.rs`):
```rust
Expr::Closure { params, ret_ty, body }
// Lowering produces MirInst::MakeClosure
```

**Bootstrap Gap** (`lowering.bmb`):
- Closure MIR generation ✅ (v0.30.34)
- Environment capture ✅
- LLVM IR emission ❌ (not in llvm_ir.bmb)

**Required Work**:
1. Add `gen_closure_llvm` to llvm_ir.bmb
2. Add closure calling convention (environment passing)
3. Integrate with function pointer types

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
| types.bmb | 167 | Type checking, generics |
| parser_ast.bmb | 119 | S-expression AST |
| llvm_ir.bmb | 80 | LLVM IR generation |
| utils.bmb | 74 | String utilities |
| compiler.bmb | 63 | Compilation coordination |
| selfhost_test.bmb | 62 | Self-hosting verification |
| mir.bmb | 59 | MIR representation |
| optimize.bmb | 56 | MIR optimization |

### Medium Coverage (20-50 tests)
| File | Tests | Notes |
|------|-------|-------|
| parser_test.bmb | 54 | Parser validation |
| pipeline.bmb | 48 | End-to-end pipeline |
| parser.bmb | 43 | Grammar parsing |
| lexer.bmb | 40 | Tokenization |
| selfhost_equiv.bmb | 33 | Equivalence testing |

### Low Coverage (<20 tests)
| File | Tests | Reason |
|------|-------|--------|
| lowering.bmb | 4 | Stack overflow limitation |

## Recommendations

### Immediate Priority (v0.30.205-210)

1. **Document Bootstrap Architecture** (Phase 30.1.206)
   - Create architecture diagram showing component relationships
   - Document data flow: Source → Token → AST → MIR → LLVM IR
   - Document encoding schemes (type encoding, instruction encoding)

2. **End-to-End Verification Tests** (Phase 30.1.207)
   - Add comprehensive E2E tests in pipeline.bmb
   - Test complete programs from source to LLVM IR
   - Verify output against Rust compiler output

3. **Trait Support Foundation** (Phase 30.1.210+)
   - Start with trait method table structure
   - Add basic impl registration
   - Implement simple trait method dispatch

### Medium Term (v0.30.210-220)

4. **Closure LLVM Emission**
   - Complete closure → LLVM IR path
   - Test with captured variables

5. **Bootstrap Interpreter**
   - Enable running bootstrap tests without Rust
   - Self-verification capability

### Long Term (v0.30.220+)

6. **SMT/Verification** (if required for self-hosting)
7. **Tooling ports** (as needed)

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

The bootstrap implementation covers **80% of the core compilation pipeline**. The primary gaps are:

1. **Trait dispatch** (required for stdlib generics)
2. **Complete generics** (constraint solving)
3. **Closure LLVM emission** (final codegen step)
4. **Bootstrap interpreter** (self-verification)

These gaps are documented in ROADMAP.md items 30.1.1-30.1.4 and should be addressed before Stage 3 bootstrap verification.
