# BMB Language Roadmap: v0.1 → v1.0.0-rc

> Progressive difficulty progression • Complete ecosystem • Self-hosting completion • Rust removal • C/Rust performance parity

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Maturity Milestones](#programming-language-maturity-milestones)
3. [Version Overview](#version-overview)
4. [Completed Phases (v0.1-v0.29)](#completed-phases-v01-v029)
5. [Remaining Phases (v0.30-v1.0.0-rc)](#remaining-phases-v030-v100-rc)
6. [Ecosystem Repositories](#ecosystem-repositories)
7. [Success Criteria](#success-criteria)

---

## Design Principles

| Principle | Description | Reference |
|-----------|-------------|-----------|
| **Gradual Progression** | Minimize difficulty gaps between versions | Gleam's 5-year 0.x journey |
| **Built-in Tooling** | `bmb fmt`, `bmb lsp` work without separate installation | Gleam pattern |
| **Small Releases** | Split large features across minor versions | Zig pattern |
| **0.x = Experimental** | Breaking changes allowed; 1.0 = stability promise | Common practice |
| **Package-First** | All reusable code registered in gotgan | Ecosystem growth |
| **Performance Proof** | Benchmark verification against C/Rust | Contract-based optimization |
| **Self-Hosting** | Complete Rust removal, BMB-only composition | Rust (OCaml→Rust 2011) |

### Non-Negotiable Priorities

| Priority | Principle | Description |
|----------|-----------|-------------|
| **Performance** | Maximum Performance Syntax | Syntax must enable maximum performance without constraints |
| **Correctness** | Compile-Time Verification | If compile-time checking is possible, it MUST be in the language spec |
| **Self-Hosting** | Bootstrap Completion | BMB compiler must compile itself. No Rust dependency after v0.30 |

### Versioning Scheme

```
v0.MAJOR.MINOR
  │      │
  │      └── Small improvements, bug fixes, feature additions
  └────────── Major milestones (Seed, Sprout, Root, ...)
```

---

## Programming Language Maturity Milestones

> References: [Wikipedia - Self-hosting compilers](https://en.wikipedia.org/wiki/Self-hosting_(compilers)), [Earthly - Programming Language Tooling](https://earthly.dev/blog/programming-language-improvements/)

### Required Milestones

| Stage | Component | Description | BMB Status | Target |
|-------|-----------|-------------|------------|--------|
| **1. Compiler** | Lexer + Parser | Source code parsing | ✅ Complete | v0.1 |
| **2. Type System** | Type Checker | Static type checking | ✅ Complete | v0.2 |
| **3. Code Generation** | Code Generator | Native/WASM output | ✅ Complete | v0.4/v0.12 |
| **4. Standard Library** | stdlib | "Batteries Included" | ✅ Complete | v0.6 |
| **5. Package Manager** | Package Manager | Dependency management | ✅ Complete | v0.8 |
| **6. Toolchain** | Tooling | fmt, lsp, test, lint | ✅ Complete | v0.7 |
| **7. IDE Support** | LSP + Extensions | VS Code, IntelliJ, etc. | ✅ Complete | v0.9 |
| **8. Self-Hosting** | Bootstrap | Compile itself | 🔄 In Progress | v0.30 |
| **9. Benchmarks** | Performance Suite | C/Rust performance proof | ✅ Complete | v0.28 |
| **10. Documentation** | Documentation | Reference, tutorials | 📋 Planned | v0.31 |
| **11. Playground** | Online Editor | Browser execution environment | ✅ Complete | v0.24 |
| **12. Community** | Ecosystem | Packages, contributors, users | 📋 Planned | v1.0 |

### Self-Hosting Definition (Bootstrap Completion Criteria)

> "A self-hosting compiler is a compiler capable of compiling its own source code." - Wikipedia

| Condition | Description | Status |
|-----------|-------------|--------|
| **Stage 1** | Build BMB compiler with Rust compiler | ✅ Complete |
| **Stage 2** | Build BMB compiler with BMB compiler | 🔄 In Progress |
| **Stage 3** | Rebuild with Stage 2 output (identical binary) | 📋 Planned |
| **Rust Removal** | Remove all Rust code, BMB-only composition | 📋 Planned (v0.30) |

**Historical References**:
- Rust: Started with OCaml → First self-compile April 2011 (1 hour)
- Go: Bootstrapped 1.5 with Go 1.4 (GCC-Go also possible)
- Lisp: First self-hosting compiler at MIT 1962

---

## Version Overview

| Version | Codename | Goal | Status |
|---------|----------|------|--------|
| v0.1-v0.9 | Foundation | Compiler, tools, ecosystem (Rust) | ✅ Complete |
| v0.10-v0.18 | Language | Generics, modules, methods | ✅ Complete |
| v0.19-v0.23 | Bootstrap | MIR completion, Stage 1/2 verification | ✅ Complete |
| v0.24-v0.29 | Polish | Examples, AI Query, benchmarks, optimization | ✅ Complete |
| **v0.30** | **Pure** | **Rust code complete removal (Self-Hosting)** | 📋 Planned |
| **v0.31** | **Docs** | **Language refinements (RFCs) + Documentation + website** | 📋 Planned |
| **v0.32** | **Ecosystem** | **100+ packages + community** | 📋 Planned |
| **v1.0.0-rc** | **Golden** | **Final verification + stability promise** | 📋 Planned |

---

## Completed Phases (v0.1-v0.29)

> Summary of 29 completed versions representing the foundation-building phase

### Phase 1: Compiler Foundation (v0.1-v0.4)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.1 | Seed | Minimal parser + type checker | Lexer (logos), Parser (lalrpop), AST, CLI (clap) |
| v0.2 | Sprout | SMT integration + basic verification | Type checker, SMT-LIB generation, Z3 integration, Error reporting (ariadne) |
| v0.3 | Root | Interpreter + REPL | Tree-walking interpreter, REPL (rustyline), Stack trace |
| v0.4 | Stem | Code generation (LLVM) | MIR (CFG-based IR), LLVM IR generation, Native build |

### Phase 2: Language & Tooling (v0.5-v0.9)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.5 | Branch | Language extensions + Bootstrap start | Pattern matching, Generic basics, Module system, Attributes |
| v0.6 | Leaf | Standard library foundation (100+ functions) | core (50+), string (25+), math (30+), io (10+) |
| v0.7 | Bloom | Tooling foundation | bmb fmt, bmb lsp, bmb test, action-bmb GitHub Action |
| v0.8 | Fruit | Package manager (gotgan) | gotgan init/build/add, Dependency resolution (SAT solver) |
| v0.9 | Harvest | Ecosystem | tree-sitter-bmb, vscode-bmb, playground, lang-bmb-site |

### Phase 3: Component Packaging & WASM (v0.10-v0.12)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.10 | Sunrise | Component packaging | bmb-lexer, bmb-parser, bmb-types, bmb-smt packages |
| v0.11 | Dawn | AI-Native gotgan | BMBX bundle format, Contract-based dependency check, AI package exploration |
| v0.12 | Horizon | WASM dual target | MIR→WASM converter, WASI runtime bindings, Browser runtime, Conditional compilation (@cfg), Dual target build |

### Phase 4: Language Completion (v0.13-v0.18)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.13 | Forge | Language completion | extern fn support, Generic basics, Error handling (? operator + try blocks), @derive attribute macro |
| v0.14 | Foundation | Generic stdlib + package standardization | Package structure standard, Option<T> generics, Result<T,E> generics |
| v0.15 | Generics | Generic type system completion | Where clauses, Generic constraints, Associated types |
| v0.16 | Consolidate | Generic enum/struct type checker | Complete generic instantiation, Type inference improvements |
| v0.17 | Module | Module system + cross-package type reference | Module resolution, Import/export, Type visibility |
| v0.18 | Methods | Option/Result method call syntax | Method chaining, Self type, Trait method resolution |

### Phase 5: Bootstrap & Verification (v0.19-v0.24)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.19 | Complete | MIR Completion (Struct/Enum/Pattern) | Struct MIR lowering, Enum MIR lowering, Pattern matching MIR |
| v0.20 | Extend | Language Extensions | Closures, Traits foundation |
| v0.21 | Bootstrap | Bootstrap Enhancement | Struct/Enum MIR in bootstrap compiler |
| v0.22 | Mirror | Parser Struct/Enum + Type Checker | Bootstrap parser enhancement, Type checker for structs/enums |
| v0.23 | Verify | Self-hosting Stage 1/2 Verification | Stage 1/2 equivalence tests (19 tests) |
| v0.24 | Examples | Bootstrap Examples | 8 algorithm examples in BMB |

### Phase 6: Polish & Performance (v0.25-v0.29)

| Version | Codename | Achievement | Key Deliverables |
|---------|----------|-------------|------------------|
| v0.25 | Query | AI Query System (RFC-0001) | Natural language code queries, Semantic search |
| v0.26 | Launch | Submodule completion + service launch | Production-ready submodules, Service deployment |
| v0.27 | Registry | gotgan local registry | Local package publishing, Version management |
| v0.28 | Benchmark | C/Rust/BMB benchmark suite | Compute-intensive benchmarks, Contract-optimized benchmarks, Real-world workloads |
| v0.29 | Velocity | C/Rust performance sprint | MIR optimization framework (6 passes), Contract-based optimization, Bootstrap optimization module |

### Bootstrap Statistics (as of v0.30.183)

| Metric | Value |
|--------|-------|
| Rust Codebase | ~21,783 LOC |
| BMB Bootstrap | ~26,023 LOC |
| Coverage | 119% |
| Stage 1/2 Tests | 48 tests passing |
| Bootstrap Tests | 4,514 tests (746 types + 397 llvm_ir + 393 compiler + 334 mir + 343 pipeline + 324 parser_ast + 303 parser + 244 lowering + 280 selfhost_test + 263 optimize + 263 lexer + 255 parser_test + 217 utils + 152 selfhost_equiv) |
| Build Mode | Use `--release` for bootstrap tests (debug build stack overflow on large files) |
| Stack-Limited Files | lowering.bmb (structural depth issue) |

---

## Remaining Phases (v0.30-v1.0.0-rc)

> Detailed task breakdown with gradual difficulty progression

### v0.30 Pure - Self-Hosting Completion

**Goal**: Complete Rust code removal, achieve full self-hosting

**Difficulty**: ⭐⭐⭐⭐⭐ (Highest - Core milestone)

**Duration Estimate**: 8-12 weeks

#### Phase 30.1: Bootstrap Compiler Enhancement

| Task | Description | Priority | Status |
|------|-------------|----------|--------|
| 30.1.0 | Generic type parsing (Vec<T>, Map<K,V>) | P0 | ✅ Complete |
| 30.1.5 | Type parameter declaration parsing | P0 | ✅ Complete |
| 30.1.6 | Type parameter scope tracking | P0 | ✅ Complete |
| 30.1.7 | Type name resolution | P0 | ✅ Complete |
| 30.1.1 | Add generics to bootstrap type checker | P0 | ✅ Complete (v0.30.217) |
| 30.1.2 | Add trait support to bootstrap | P0 | ✅ Complete (v0.30.211) |
| 30.1.3 | Add closure codegen to bootstrap | P0 | ✅ Complete (v0.30.108) |
| 30.1.4 | Implement bootstrap interpreter | P1 | Pending |

**v0.30.1 Completed (2026-01-04)**:
- `parse_type_args`: Comma-separated type arguments inside `<...>`
- `parse_type`: Extended to support generic types in function params/returns
- `parse_type_or_ident`: Extended to support generic types in struct fields
- 6 new tests for generic type parsing

**v0.30.2 Completed (2026-01-04)**:
- `parse_type_param_name`: Parse single type parameter name
- `parse_type_params_inner`: Comma-separated type parameters inside `<...>`
- `try_parse_type_params`: Optional type parameter block after name
- Extended `parse_struct_def`, `parse_enum_def`, `parse_fn` for generics
- 6 new tests for type parameter declarations (39 total)

**v0.30.3 Completed (2026-01-04)**:
- Type parameter encoding: `kind=10` for TypeParam in types.bmb
- `tparam_add`, `tparam_count`: Type parameter environment management
- `tparam_lookup`, `tparam_in_scope`: Scope checking functions
- `tparam_resolve`: Convert name to type_param(idx) or type_error()
- 4 new test functions, 21 assertions (66 total in types.bmb)

**v0.30.4 Completed (2026-01-04)**:
- `is_primitive_type`, `primitive_type`: Detect and resolve primitive types
- `is_type_param_name`: Detect single uppercase letters (A-Z)
- `resolve_type_name`: Unified resolution (primitives → type params → named)
- `name_hash`: Simple hash for named types (struct/enum)
- 3 new test functions, 23 assertions (89 total in types.bmb)

**v0.30.5 Completed (2026-01-04)**:
- `type_generic_app(base_hash)`: GenericApp type encoding (kind=11)
- `is_generic_app`, `generic_base_hash`: Detection and extraction functions
- `type_vec`, `type_option`, `type_result`, `type_map`: Common generic constructors
- `name_hash_mod`, `name_hash_base`: Mod-safe hash for base type names
- 3 new test functions, 20 assertions (109 total in types.bmb)

**v0.30.6 Completed (2026-01-04)**:
- `gen_type_pack(base, args)`: Pack generic type info as string "Base:Arg1,Arg2"
- `gen_type_base`, `gen_type_args`: Unpack base and arguments
- `gen_type_arg_count`, `gen_type_arg_at`: Access individual type arguments
- `gen_vec_info`, `gen_option_info`, `gen_result_info`, `gen_map_info`: Convenience constructors
- 4 new test functions, 26 assertions (135 total in types.bmb)

**v0.30.7 Completed (2026-01-04)**:
- `subst_new`, `subst_add`: Create and extend type substitutions
- `subst_lookup`, `subst_has`: Query substitution mappings
- `subst_apply`: Apply substitution to simple type names
- `subst_apply_gen`: Apply substitution to generic type info (Vec:T → Vec:i64)
- `subst_from_params_args`: Build substitution from type params and args
- 5 new test functions, 28 assertions (163 total in types.bmb)

**v0.30.8 Completed (2026-01-04)**:
- `instantiate_generic`: Create instantiated type info (Box, T, i64 → Box:i64)
- `get_instantiation_subst`: Build substitution for instantiation
- `resolve_field_type`: Resolve field types using substitution (T → i64, Vec:T → Vec:i64)
- `check_arity`: Validate type argument count matches parameters
- `validate_type_app`: Check well-formedness of type applications
- `instantiate_type`: Full instantiation with validation and encoding
- 4 new test functions, 22 assertions (185 total in types.bmb)

**v0.30.9 Completed (2026-01-04)**:
- `gen_fn_pack`, `gen_fn_name`, `gen_fn_tparams`, `gen_fn_params`, `gen_fn_return`: Signature pack/unpack
- `gen_fn_instantiate`: Instantiate generic function with concrete type arguments
- `gen_fn_param_count`, `gen_fn_param_at`: Parameter access functions
- `gen_fn_check_call`: Validate generic function call (arity, type matching)
- `gen_fn_match_params`: Match expected and actual parameter types
- 4 new test functions, 23 assertions (208 total in types.bmb)

**v0.30.10 Completed (2026-01-04)**:
- `is_single_tparam`: Check if string is a single type parameter (A-Z)
- `infer_single`, `infer_merge`: Infer and merge type param bindings
- `infer_from_pair_list`: Infer all type params from param/actual type lists
- `infer_all_bound`, `infer_build_targs`: Validate and construct type args
- `gen_fn_infer_call`: Full inference and checking for generic function calls
- 7 new test functions, 32 assertions (240 total in types.bmb)

**v0.30.11 Completed (2026-01-04)**:
- `gen_struct_pack`, `gen_struct_name`, `gen_struct_tparams`, `gen_struct_fields`: Pack/unpack
- `gen_struct_field_type`: Get field type string by name
- `gen_struct_resolve_field`: Resolve field type with type arguments (Box<i64>.value → i64)
- `gen_struct_is_generic`, `gen_struct_field_count`, `gen_struct_field_name_at`: Utilities
- 6 new test functions, 25 assertions (265 total in types.bmb)

**v0.30.12 Completed (2026-01-04)**:
- `struct_reg_new`, `struct_reg_add`: Create and populate struct registry
- `struct_reg_lookup`, `struct_reg_has`: Query registry for struct definitions
- `struct_reg_field_type`: Resolve field type with type args through registry lookup
- `struct_reg_count`, `struct_reg_is_generic`: Registry utilities
- 5 new test functions, 20 assertions (285 total in types.bmb)

**v0.30.13 Completed (2026-01-04)**:
- `gen_enum_pack`, `gen_enum_name`, `gen_enum_tparams`, `gen_enum_variants`: Pack/unpack
- `gen_enum_variant_type`, `gen_enum_has_variant`: Variant type lookup
- `gen_enum_resolve_variant`: Resolve variant type with type arguments (Option<i64>.Some → i64)
- `gen_enum_is_generic`, `gen_enum_variant_count`, `gen_enum_variant_name_at`: Utilities
- 7 new test functions, 31 assertions (316 total in types.bmb)

**v0.30.14 Completed (2026-01-04)**:
- `enum_reg_new`, `enum_reg_add`: Create and populate enum registry
- `enum_reg_lookup`, `enum_reg_has`: Query registry for enum definitions
- `enum_reg_variant_type`: Resolve variant type with type args through registry lookup
- `enum_reg_count`, `enum_reg_is_generic`: Registry utilities
- 5 new test functions, 19 assertions (335 total in types.bmb)

**v0.30.88 Completed (2026-01-05)**:
- Trait method dispatch type checking: type_of_trait_call, lookup_trait_for_method
- Method parameter extraction: trait_method_params, extract_method_params, check_trait_call
- Find trait with method: find_trait_with_method, get_method_params_from
- 2 new test functions (8 assertions)
- types.bmb total: 712 tests (156 test functions)

**v0.30.89 Completed (2026-01-06)**:
- String concatenation type checking: binop_result_type extended for String + String
- Arithmetic (+) now supports both i64 and String operands
- Mixed type errors: i64+String and String+i64 properly detected
- 1 new test function (5 assertions)
- types.bmb total: 717 tests (157 test functions)

**v0.30.90 Completed (2026-01-06)**:
- Trait method dispatch IR testing: test_method_dispatch_edge in llvm_ir.bmb (10 tests)
- Edge cases for method dispatch: unsupported methods, various argument counts, method call parsing
- Trait dispatch edge cases: test_trait_dispatch_edge in types.bmb (8 tests)
- Multiple traits with overlapping method names, lookup resolution
- llvm_ir.bmb total: 239 tests (52 test functions)
- types.bmb total: 725 tests (158 test functions)

**v0.30.91 Completed (2026-01-06)**:
- Return type annotation edge cases: test_return_type_edge in types.bmb (8 tests)
- Tests for if-then-else, let bindings, nested let, bool returns, type mismatch detection
- types.bmb total: 733 tests (159 test functions)

**v0.30.92 Completed (2026-01-06)**:
- Pipeline integration verification: 3 new test groups in pipeline.bmb (12 tests)
- Multi-operand expressions, mixed operations, combined expressions
- pipeline.bmb total: 42 tests (10 test groups)

**v0.30.290 Completed (2026-01-07)**:
- Phase 30.1.290-292: Code Quality Analysis & Cleanup
- **Analysis (30.1.290)**: Scanned TODO/FIXME (7), Clippy warnings (147)
- **Cleanup (30.1.291)**: Fixed `push_str` single char → `push` (8 instances)
- **Result**: Clippy warnings reduced 147 → 139 (5% reduction)
- **Files modified**: `lsp/mod.rs`, `main.rs`
- **Remaining work**: 139 warnings (mostly style), 7 TODOs (future work)

**v0.30.287 Completed (2026-01-07)**:
- Phase 30.1.287-289: Final Stage 3 Analysis & Documentation
- **Investigation (30.1.287)**: Confirmed ~1MB failure is heap allocation, not stacker fiber limit
- **Evaluation (30.1.288)**: Additional optimization deemed infeasible (architectural constraint)
- **Documentation (30.1.289)**: Updated BOOTSTRAP_FEATURE_GAP.md with final analysis
- **Root cause**: Bootstrap's `lower_let` recursive MIR generation exceeds heap memory limits
- **Conclusion**: 6/7 (86%) Stage 3 tests pass - represents practical limit of incremental optimization
- **Next steps**: Full resolution requires Bootstrap compiler architecture redesign

**v0.30.286 Completed (2026-01-07)**:
- Phase 30.1.283-286: StringRope Optimization
- **Design (30.1.283)**: StringRope variant for lazy string concatenation
- **Implementation (30.1.284)**: Value::StringRope, concat_strings(), materialize_string()
- **Integration (30.1.285)**: eval_method_call, match_pattern, main.rs StringRope handling
- **Memory reduction**: ~28% (1.4MB → 1MB) for bootstrap compilation
- **Stage 3 status**: 6/7 tests pass (let binding still ~1MB, architectural limit)
- **Interpreter tests**: 25/25 pass with StringRope

**v0.30.280 Completed (2026-01-07)**:
- Phase 30.1.278-280: ScopeStack Memory Optimization
- **Investigation (30.1.278)**: Trampolining feasibility analysis - determined infeasible for bootstrap
- **Design (30.1.279)**: ScopeStack prototype - `Vec<HashMap<String, Value>>` replacing Rc chains
- **Implementation (30.1.280)**: Full interpreter integration with eval_fast path
- **Architecture**: Immediate scope deallocation on exit instead of waiting for Rc drop
- **Result**: Environment chain memory issue RESOLVED
- **Stage 3 status**: 6/7 tests pass (let binding failure now due to string concatenation, not env chains)
- **Interpreter tests**: 20/20 pass with ScopeStack

**v0.30.277 Completed (2026-01-07)**:
- Phase 30.1.273-277: Bootstrap Bottleneck Analysis
- **String operation census**: lowering.bmb contains 221 pack/unpack calls, 253 string concatenations
- **Optimization evaluation**: 4 options assessed (Bootstrap rewrite, interpreter opt, tuple repr, document)
- **Decision**: Document as architectural limitation (lowest risk, highest certainty)
- **Conclusion**: Let binding failure is Environment chain lifetime issue, not string operations
- **Stage 3 status**: 6/7 tests pass (practical success, let bindings require trampolining)

**v0.30.269 Completed (2026-01-07)**:
- Phase 30.1.269-272: Let Binding Root Cause Analysis
- **Investigation**: Analyzed why even single let bindings fail in Stage 3
- **Root cause identified**: Self-referential complexity
  - Bootstrap's `lower_let` uses recursive `lower_expr` calls
  - Compiling let-binding code triggers bootstrap's own let-binding implementation
  - Exponential growth in call depth and memory allocation
- **Stacker tuning tested**: Reduced 4MB→1MB→2MB segments all fail
- **Conclusion**: Architectural limitation requiring trampolining or CPS transformation
- **Status**: 6/7 tests pass (let bindings remain unsupported)

**v0.30.268 Completed (2026-01-07)**:
- Phase 30.1.264-268: Value::Str Rc<String> Optimization
- **Memory optimization**: Changed `Value::Str(String)` to `Value::Str(Rc<String>)`
- **Benefits**: Reduced clone overhead for string values in interpreter
- **Files modified**: `interp/value.rs`, `interp/eval.rs`, `main.rs`
- **Stage 3 status**: 6/7 tests pass (unchanged, fiber allocation remains for let bindings)
- **Interpreter tests**: 14/14 pass with Rc<String> implementation

**v0.30.263 Completed (2026-01-07)**:
- Phase 30.1.259-263: Stage 3 Test Expansion
- **Test coverage expanded**: 3/4 → 6/7 tests pass
- **New passing tests**:
  - `stage3_nested_cond.bmb`: Nested conditional expressions
  - `stage3_call.bmb`: Function composition (`f(g(x))`)
  - `stage3_arith.bmb`: Complex arithmetic expressions
- **New limitations discovered**:
  - Boolean return types cause memory allocation failures
  - Recursive functions cause fiber allocation failures on Windows
- **Supported patterns documented**: Binary ops, conditionals, function composition
- **Unsupported patterns documented**: Let bindings, bool returns, recursion

**v0.30.258 Completed (2026-01-07)**:
- Phase 30.1.254-258: Bootstrap String Optimization
- **Interpreter optimization**: String concatenation with pre-allocated capacity (`String::with_capacity` + `push_str`)
- **Results**: Memory usage reduced from ~2MB to ~1.1MB (~44% reduction)
- **Stage 3 status**: 3/4 tests pass (let binding still fails due to memory lifetime)
- **Root cause remains**: Rc<RefCell<Environment>> chain keeps all scopes alive until stack unwinds
- Future improvements: Arena allocator (P1), tail-call optimization (P2), Cow<str> (P3)

**v0.30.253 Completed (2026-01-07)**:
- Phase 30.1.250-253: Bootstrap Memory Analysis
- **ROOT CAUSE IDENTIFIED**: Let binding memory failure in `compile_program`
  - Bootstrap compiler (2035 lines) creates deep call graphs when interpreting
  - Each function call creates new Environment (Rc<RefCell>)
  - String operations in `pack_lower_result` accumulate ~2MB before stack unwinds
  - Test gap: `compile_program` + let bindings never tested in selfhost_equiv.bmb
- **Decision**: Accept current limitation (3/4 tests pass) as Stage 3 baseline
- Future priority: Optimize bootstrap string operations for full let binding support

**v0.30.248 Completed (2026-01-07)**:
- Phase 30.1.245-249: Stage 3 Verification Harness
- **NEW COMMAND**: `bmb verify-stage3 <file.bmb>` compares Rust vs Bootstrap IR output
- Implementation:
  - `verify-stage3` CLI command with verbose and output options
  - `call_function_with_args` method in interpreter for direct function invocation
  - Bootstrap compiler runs in 64MB stack thread (same as `bmb run`)
  - IR normalization filters comments, declarations, module info
  - Semantic matching compares function signatures
- Increased `MAX_RECURSION_DEPTH` from 10,000 to 100,000 for bootstrap complexity
- **Test Results**: 3/4 tests pass (simple functions, conditionals, multiple functions)
- Known limitation: Let bindings cause ~2MB memory allocation failure in bootstrap
- Future: Optimize `lowering.bmb` memory efficiency for let binding support

**v0.30.241 Completed (2026-01-07)**:
- Phase 30.1.241-244: Stack overflow fix for bootstrap execution
- **CRITICAL FIX**: Interpreter now runs in 64MB stack thread
  - Problem: Windows default 1MB stack overflowed with deep bootstrap recursion
  - Solution: `thread::Builder::stack_size(64MB)` in `run_file()`
  - All bootstrap files now execute successfully
- Test results: utils(217), compiler(393), selfhost_equiv(152), types(821), llvm_ir(433)
- Updated BOOTSTRAP_FEATURE_GAP.md: Stack overflow marked as FIXED
- **Stage 3 unblocked**: Bootstrap execution now works, only verification harness needed

**v0.30.236 Completed (2026-01-07)**:
- Phase 30.1.236-240: Stage 3 readiness analysis and blocker identification
- **STAGE 3 BLOCKERS IDENTIFIED**:
  - ~~Stack overflow: Bootstrap .bmb files overflow interpreter stack~~ → Fixed in v0.30.241
  - No File I/O: Bootstrap can't read source files or write output
  - No Process Exec: Can't invoke LLVM toolchain (llvm-as, llc, linker)
  - No Verification Harness: Need tool to compare stage outputs
- E2E test results: Rust compiler build works (173KB exe), bootstrap execution fails → Fixed
- Updated BOOTSTRAP_FEATURE_GAP.md: Stage 3 Blockers section, updated recommendations
- **Recommended path**: Create Rust harness to wrap bootstrap execution

**v0.30.232 Completed (2026-01-07)**:
- Phase 30.1.232-235: lowering.bmb test coverage analysis
- **CORRECTED**: lowering.bmb has 4 helper functions + 79 test groups (244 assertions), NOT "4 tests"
- Inline test pattern: tests in main() due to stack overflow with many separate test functions
- Updated BOOTSTRAP_FEATURE_GAP.md: corrected test counts, added "Inline Test Pattern" section
- Total bootstrap tests: 914 functions with proper coverage documentation

**v0.30.228 Completed (2026-01-07)**:
- Phase 30.1.228-231: Stage 2 verification analysis and documentation
- **STAGE 2 VERIFIED**: 152 equivalence assertions passing (selfhost_equiv.bmb)
- Stage 2 tests: MIR equivalence, LLVM IR equivalence, lowering patterns, LLVM patterns
- Bootstrap design: minimal BMB subset (no closures/structs/enums) enables self-compilation
- All bootstrap tests verified: types(821), llvm_ir(433), pipeline(415), mir(406), compiler(393)
- Updated BOOTSTRAP_FEATURE_GAP.md with Stage status table and Stage 2 verification details

**v0.30.221 Completed (2026-01-07)**:
- Phase 30.1.221-223: Bootstrap documentation actualization
- **P0 ALL COMPLETE**: Trait support (v0.30.211), Generics (v0.30.217), Closures (v0.30.108)
- Updated BOOTSTRAP_FEATURE_GAP.md to reflect actual implementation status
- Test count: 914 tests across 14 files (verified counts)
- Updated recommendations: P1 interpreter as next priority
- Stage 2 self-hosting: Pipeline complete, verification in progress

**v0.30.217 Completed (2026-01-07)**:
- Phase 30.1.217-220: Tuple type substitution in bootstrap type checker
- **FIXED P0 GAP**: Tuple types `(A,B)` now correctly substitute to `(i64,String)`
- Added is_tuple_type(), subst_find_tuple_elem_end(), subst_apply_tuple(), subst_apply_tuple_elems()
- Modified subst_apply() priority: direct lookup → tuple types → generic types
- tests: test_subst_tuple (10 assertions), test_subst_tuple_fn (5 assertions)
- types.bmb: 173 tests, 821 assertions (tuple substitution complete)
- Generics system now handles: primitives, type params, generic types, tuple types

**v0.30.216 Completed (2026-01-06)**:
- Phase 30.1.213-216: Recursive nested generic substitution implementation
- **FIXED P0 GAP**: Nested generic substitution now works recursively
- Modified subst_apply() to detect generic type packs (contains ':') and recursively process
- Verified substitutions: Option<List<T>>→Option<List<i64>>, Map<K,List<V>>→Map<String,List<i64>>
- Deep nesting verified: Option<Result<List<T>,E>>→Option<Result<List<i64>,String>>
- Updated test_nested_generic_subst to verify correct recursive behavior
- BOOTSTRAP_FEATURE_GAP.md: nested substitution status changed ⚠️→✅
- Remaining generics gaps: tuple return types, monomorphization tracking

**v0.30.212 Completed (2026-01-06)**:
- Phase 30.1.209-212: Generics implementation analysis and advanced generic tests
- types.bmb: 782 → 806 tests (+24) - nested generic types, nested substitution, bounded type params, generic fn bounds
- Added test_nested_generic_types: Option<List<T>>, Map<K,V>, Result<Option<T>,E>
- Added test_nested_generic_subst: documented top-level-only substitution limitation
- Added test_bounded_type_params: Clone+Display multiple bounds verification
- Added test_generic_fn_bounds: generic function instantiation with multi-params
- Updated BOOTSTRAP_FEATURE_GAP.md: generics status, test counts, remaining nested subst gap
- types.bmb test functions: 167 → 171 (+4), assertions: 782 → 806 (+24)
- Bootstrap test total: 4,718 tests across 14 files (+24)

**v0.30.207 Completed (2026-01-06)**:
- Phase 30.1.205-208: Bootstrap feature gap analysis, architecture documentation, E2E verification
- Created docs/BOOTSTRAP_FEATURE_GAP.md: Comprehensive Rust vs BMB capability analysis
- Created docs/BOOTSTRAP_ARCHITECTURE.md: Bootstrap architecture, data flow, encoding schemes
- pipeline.bmb: 379 → 415 tests (+36) - E2E simple functions, arithmetic/comparison/logical ops, nested expressions, function signatures
- Documented P0 gaps: trait support, complete generics
- Documented P1 gaps: closure LLVM emission, bootstrap interpreter
- Bootstrap test total: 4,694 tests across 14 files (+36)

**v0.30.203 Completed (2026-01-06)**:
- Phase 30.1.200-203: Bootstrap integration test enhancement
- types.bmb: 746 → 782 tests (+36) - trait method dispatch, impl pack, binop result type integration tests
- mir.bmb: 370 → 406 tests (+36) - binop symbols, MIR text, call variations, const encoding, fn header tests
- pipeline.bmb: 343 → 379 tests (+36) - multi-function parsing, arithmetic, nested let, logical, compound expression tests
- llvm_ir.bmb: 397 → 433 tests (+36) - binop i32, const edge cases, cmp full, unary full, logic full tests
- Bootstrap test total: 4,658 tests across 14 files (+144)

**v0.30.198 Completed (2026-01-06)**:
- Phase 30.1.195-198: Bootstrap test coverage enhancement
- lexer.bmb: 227 → 263 tests (+36) - token decode, whitespace boundary, ident start, two-char ops tests
- utils.bmb: 181 → 217 tests (+36) - whitespace, digit, alpha boundaries, token encoding tests
- parser_test.bmb: 221 → 255 tests (+34) - token constants, bool tokens, find_substring, is_alpha/is_digit tests
- selfhost_equiv.bmb: 116 → 152 tests (+36) - pattern prefix/suffix, MIR/LLVM control flow, global patterns tests
- Bootstrap test total: 4,514 tests across 14 files

**v0.30.193 Completed (2026-01-06)**:
- Phase 30.1.190-193: Bootstrap test coverage enhancement
- compiler.bmb: 357 → 393 tests (+36) - has_pattern, make_tok, skip_comment, keyword_or_ident tests
- selfhost_test.bmb: 244 → 280 tests (+36) - keyword_kind, next_token_raw, parse_type, op_symbol tests
- parser_ast.bmb: 288 → 324 tests (+36) - skip_all, symbol_code, lookup_keyword, is_predicates tests
- llvm_ir.bmb: 361 → 397 tests (+36) - skip_ws, find_char, starts_with, char_conversion, const_parsing tests
- Bootstrap test total: 4,372 tests across 14 files

**v0.30.188 Completed (2026-01-06)**:
- Phase 30.1.185-188: Bootstrap test coverage enhancement
- optimize.bmb: 227 → 263 tests (+36) - optimization level, const/copy table tests
- parser.bmb: 267 → 303 tests (+36) - token memory, result categories, binop coverage
- pipeline.bmb: 307 → 343 tests (+36) - make_tok, lookup_keyword, node funcs tests
- mir.bmb: 334 → 370 tests (+36) - binop range, instruction/terminator encoding tests
- Bootstrap test total: 4,264 tests across 14 files

**v0.30.160 Completed (2026-01-06)**:
- Documentation update: GAP_ANALYSIS.md, ROADMAP.md
- Bootstrap test total: 2,565 tests across 14 files
- Phase 30.1.156-160 comprehensive test enhancement complete

**v0.30.159 Completed (2026-01-06)**:
- optimize.bmb test enhancement: 63 → 155 tests (+92)
- 19 new unit test functions covering:
  * test_int_to_string_ext, test_starts_with_ext, test_find_char_ext
  * test_find_pattern_ext, test_parse_int_at_ext, test_extract_dest_ext
  * test_extract_const_ext, test_extract_binop_ext, test_extract_operands
  * test_eval_binop_ext, test_more_inst_checks, test_const_table_ext
  * test_copy_table_ext, test_is_label_ext, test_branch_cond
  * test_fold_const_inst, test_make_stats, test_digit_char, test_skip_ws_ext
- Comprehensive MIR optimization utility coverage

**v0.30.158 Completed (2026-01-06)**:
- selfhost_test.bmb test enhancement: 56 → 152 tests (+96)
- 23 new unit test functions covering:
  * Lexer helpers: skip_ws, skip_to_eol, skip_comment, skip_all
  * Scanner functions: scan_ident, scan_int
  * Token generation: next_token_raw
  * Parser utilities: find_colon, parse_int_prefix, parse_type
  * Expression parsing: parse_primary, parse_mul, parse_add, parse_cmp_ops
  * Logical parsing: parse_and, parse_or, parse_negation
  * Token constants, keywords, op_symbol, is_binop
- Full self-hosting lexer/parser verification

**v0.30.157 Completed (2026-01-06)**:
- parser_test.bmb test enhancement: 15 → 99 tests (+84)
- 12 new unit test functions covering:
  * test_token_constants: TK_FN, TK_LET, TK_IF, etc. (8 tests)
  * test_char_class: is_digit, is_alpha, is_ident_char (8 tests)
  * test_skip_funcs: skip_ws, skip_to_eol, skip_comment (6 tests)
  * test_lookup_kw: keyword lookup with fallback (12 tests)
  * test_symbol_codes: +, -, *, /, etc. symbol encoding (8 tests)
  * test_two_char: ==, !=, <=, >=, ->, etc. (8 tests)
  * test_result_pack: position:ast format (6 tests)
  * test_op_str: operator string mapping (6 tests)
  * test_type_str: type name mapping (6 tests)
  * test_is_binop: binary operator detection (6 tests)
  * test_tok_encoding: kind * 1000000 + pos encoding (4 tests)
  * test_helpers: string utilities (6 tests)
- Comprehensive parser utility coverage

**v0.30.156 Completed (2026-01-06)**:
- utils.bmb test enhancement: 38 → 47 tests (+9)
- Extended unit test functions covering:
  * Edge case testing for string utilities
  * Boundary condition validation
  * Additional helper function coverage
- utils.bmb: 47 tests

**v0.30.140 Completed (2026-01-06)**:
- Bootstrap coverage analysis: 1,732 tests across 14 files
- Test coverage summary documented

**v0.30.139 Completed (2026-01-06)**:
- utils.bmb edge case tests (+5 tests)
- test_char_boundary_cases: digit/alpha boundary values
- test_int_to_string_edge: large numbers, negatives
- test_string_match_edge: empty strings, position edge cases
- test_result_edge_cases: special characters, pipe handling
- test_skip_edge_cases: skip functions boundary conditions
- utils.bmb: 38 tests

**v0.30.138 Completed (2026-01-06)**:
- parser_ast.bmb operator precedence and complex expression tests (+9 tests)
- test_operator_precedence: multiplication before addition, parentheses override
- test_complex_expressions: nested if-then-else, nested let, method chains
- parser_ast.bmb: 113 tests

**v0.30.137 Completed (2026-01-06)**:
- optimize.bmb additional tests (+14 tests)
- test_var_usage: variable usage detection (5 tests)
- test_side_effects: side effects detection (5 tests)
- test_opt_levels: optimization level validation (5 tests)
- optimize.bmb: 29 tests

**v0.30.135 Completed (2026-01-06)**:
- MIR generation tests in mir.bmb (+9 tests)
- test_unop_and_blocks: unary operators, block labels, void calls
- test_function_headers: function start/end, parameter types
- mir.bmb: 62 tests (13 test functions)
- Total bootstrap tests: 1,437

**v0.30.134 Completed (2026-01-06)**:
- Type checking tests in types.bmb (+13 tests)
- test_generic_inference_edge: generic type inference (infer_single, infer_merge, infer_all_bound)
- test_call_arity_edge: function call arity validation
- types.bmb: 746 tests (161 test functions)

**v0.30.133 Completed (2026-01-06)**:
- Investigated lowering.bmb stack overflow issue
- Root cause: 79 test groups with recursive lower_expr in single main()
- Decision: Skip refactoring (high cost), proceed with other bootstrap files

**v0.30.132 Completed (2026-01-06)**:
- Advanced IR pattern tests in llvm_ir.bmb (6 tests)
- test_advanced_ir_patterns: integer/bool constants, sub/sdiv/icmp slt instructions
- llvm_ir.bmb: 361 tests (68 test functions)
- Stack overflow fix: use `--release` build for bootstrap tests
- Total bootstrap tests: 1,415

**v0.30.131 Completed (2026-01-06)**:
- Pipeline conditional expression tests in pipeline.bmb (8 tests)
- Test 15: Conditional expressions (==, !=, >=, <=)
- Test 16: Chained logic operators (and, or, not)
- pipeline.bmb: 64 tests (16 test groups)

**v0.30.130 Completed (2026-01-06)**:
- MIR optimization edge case tests in optimize.bmb (6 tests)
- test_opt_edge_cases: subtraction/multiplication/division/modulo folding, statistics format, no-opt level 0
- optimize.bmb: 15 tests (10 test groups)

**v0.30.129 Completed (2026-01-06)**:
- MIR encoding edge case tests in mir.bmb (7 tests)
- test_encoding_edge_cases: unary NOT, comparison operators, large temps, empty call, terminator detection
- mir.bmb: 53 tests (11 test groups)

**v0.30.128 Completed (2026-01-06)**:
- Function boundary IR tests in llvm_ir.bmb (6 tests)
- test_function_boundary_ir: fn header/footer, params, bool return, call, return
- llvm_ir.bmb: 355 tests (67 test functions)
- Total bootstrap tests: 1,388

**v0.30.127 Completed (2026-01-06)**:
- Type system edge case tests in llvm_ir.bmb (6 tests)
- test_type_system_edges: String/bool/unit type mapping, comparison, inttoptr
- llvm_ir.bmb: 349 tests (66 test functions)

**v0.30.126 Completed (2026-01-06)**:
- Complex MIR generation patterns in pipeline.bmb (4 tests)
- Test 14: Nested operations, logical combinations, grouped expressions
- pipeline.bmb: 56 tests (14 test groups)

**v0.30.125 Completed (2026-01-06)**:
- Multi-param and return type verification in pipeline.bmb (4 tests)
- Test 13: Triple-param, bool return, String return, zero-param functions
- pipeline.bmb: 52 tests (13 test groups)

**v0.30.124 Completed (2026-01-06)**:
- Complex pattern matching MIR tests in lowering.bmb (4 tests)
- test_pattern_match_tests: Match arm blocks, merge blocks, switch dispatch, copy
- lowering.bmb: 244 tests (80 test groups) - stack limit reached
- Total bootstrap tests: 1,368

**v0.30.123 Completed (2026-01-06)**:
- Control flow edge case lowering tests in lowering.bmb (4 tests)
- Break/continue in while loops, if-then-else branches, nested control flow
- Tests: BREAK instruction, CONTINUE instruction, branch pattern, loop_start

**v0.30.122 Completed (2026-01-06)**:
- Cross-module and extern call IR tests in llvm_ir.bmb (6 tests)
- test_cross_module_ir: Module header, extern declarations, runtime patterns
- Tests: target triple, declare patterns, i8* return types
- llvm_ir.bmb total: 343 tests (66 test functions)
- Total bootstrap tests: 1,360

**v0.30.121 Completed (2026-01-06)**:
- Edge case and boundary condition tests in llvm_ir.bmb (6 tests)
- test_edge_cases_ir: Empty strings, zero constants, boolean values
- Tests: inttoptr patterns, add i64 patterns, i1 patterns

**v0.30.120 Completed (2026-01-06)**:
- Deeply nested expression IR tests in llvm_ir.bmb (6 tests)
- test_nested_expression_ir: Nested calls, deep arithmetic, complex logic
- Tests: f(g(h(x))) patterns, register chaining, comparison nesting

**v0.30.119 Completed (2026-01-06)**:
- Operator precedence IR tests in llvm_ir.bmb (6 tests)
- test_operator_precedence_ir: mul before add, left associativity, logical precedence
- Tests: binop ordering, div/mod patterns, and-before-or

**v0.30.118 Completed (2026-01-06)**:
- Bootstrap self-parse integration tests in pipeline.bmb (8 new tests, 2 test groups)
- Test 11: Bootstrap-style function parsing (4 tests) - fn signatures with params
- Test 12: Bootstrap-style complex expressions (4 tests) - nested binops, and-chains
- pipeline.bmb total: 48 tests (12 test groups)
- Total bootstrap tests: 1,336

**v0.30.117 Completed (2026-01-06)**:
- Method call chaining IR generation tests in llvm_ir.bmb (6 tests)
- test_method_chaining: Verifies chain dispatch, register flow, type consistency
- Tests: slice→len chain, concat preservation, char_at on chain results
- llvm_ir.bmb total: 319 tests (62 test functions)

**v0.30.116 Completed (2026-01-06)**:
- String concatenation LLVM IR verification and gap documentation
- Verified `.concat()` method IR generation works correctly
- Documented `+` operator gap: MIR lacks type info for String binop dispatch
- Gap added to GAP_ANALYSIS.md for Stage 2 self-hosting consideration

**v0.30.115 Completed (2026-01-06)**:
- Bootstrap completeness verification: 1,324 total bootstrap tests passing
- llvm_ir.bmb: 313 tests, lowering.bmb: 236 tests, pipeline.bmb: 42 tests, types.bmb: 733 tests
- All cargo tests passing (19 tests)
- Full integration validation complete

**v0.30.114 Completed (2026-01-06)**:
- Full MIR → LLVM IR pipeline verification
- test_full_mir_pipeline: 6 tests for TraitCall, MethodCall, LoadCapture, CallClosure
- Verified gen_mir_line_typed dispatches all MIR instruction types correctly
- llvm_ir.bmb total: 313 tests (61 test functions)

**v0.30.113 Completed (2026-01-06)**:
- TraitCall dispatch integration in gen_mir_line_typed
- Added `is_trait_call_line(line) then gen_instr_trait_call(line)` dispatch
- Fixed parse_trait_call_args bounds checking for edge cases
- test_trait_call_dispatch: 6 tests verifying MIR→LLVM dispatch
- llvm_ir.bmb: 307 tests

**v0.30.112 Completed (2026-01-06)**:
- End-to-end pipeline verification
- Verified all trait dispatch and closure capture systems work together
- llvm_ir.bmb: 301 tests, lowering.bmb: 236 tests, pipeline.bmb: 42 tests
- Full bootstrap validation passed

**v0.30.111 Completed (2026-01-06)**:
- TraitCall type propagation tests in llvm_ir.bmb (6 new tests)
- test_trait_type_propagation_ir: Return type verification, dispatch generation tests
- Verified i64 bootstrap representation for all trait methods
- llvm_ir.bmb total: 301 tests

**v0.30.110 Completed (2026-01-06)**:
- Closure function generation infrastructure in lowering.bmb
- gen_closure_fn_header: Generate fn @closure_N(%env: i64*, params...) headers
- gen_closure_prelude: Generate LoadCapture instructions for captured variables
- gen_closure_fn: Combine header, prelude, body, footer into complete function
- closure_fn_name: Generate @closure_N function names

**v0.30.109 Completed (2026-01-06)**:
- Full trait/closure pipeline integration tests in lowering.bmb (12 new tests)
- Test 75: Trait dispatch pipeline (4 tests) - static dispatch with impl registry
- Test 76: Closure capture pipeline (4 tests) - complete CLOSURE + Capture flow
- Test 77: Combined trait + closure pipeline (4 tests) - both systems working together
- lowering.bmb total: 236 tests

**v0.30.108 Completed (2026-01-06)**:
- Closure invocation LLVM IR in llvm_ir.bmb (6 new tests)
- is_call_closure_line, parse_call_closure_dest/closure/args: CallClosure parsing
- gen_instr_call_closure: Generate indirect call through closure struct {fn_ptr, env}
- Extract function pointer (GEP + load + inttoptr), extract env, call with env as first arg
- llvm_ir.bmb total: 289 tests (at v0.30.108)

**v0.30.107 Completed (2026-01-06)**:
- Impl registry connection in lowering.bmb (14 new tests)
- gen_impl_key: Generate "Trait:Type" key format for impl lookup
- impl_reg_lookup, impl_reg_has: Registry lookup and existence check
- lower_trait_call_with_reg: Smart dispatch using impl registry
- lowering.bmb total: 224 tests

**v0.30.106 Completed (2026-01-06)**:
- Closure capture integration test in lowering.bmb (4 new tests)
- Full pipeline verification: CLOSURE marker + Capture instruction + captured var name
- Validates complete closure capture MIR generation with free variable analysis
- lowering.bmb total: 210 tests

**v0.30.105 Completed (2026-01-06)**:
- Capture/LoadCapture LLVM IR dispatch in llvm_ir.bmb (6 new tests)
- gen_instr_capture: Generate LLVM IR for Capture MIR instruction
- gen_instr_load_capture: Generate LLVM IR for LoadCapture MIR instruction
- Connected dispatch in gen_mir_line_typed for closure environment operations
- llvm_ir.bmb total: 289 tests

**v0.30.104 Completed (2026-01-06)**:
- LoadCapture MIR generation in lowering.bmb (18 new tests)
- gen_load_capture: Generate LoadCapture instruction for captured variable access
- is_captured_var, find_capture_index: Captured variable detection and index lookup
- gen_captured_var_access, lower_var_with_captures: Variable access with capture context
- lowering.bmb total: 206 tests

**v0.30.103 Completed (2026-01-06)**:
- Closure environment allocation in lowering.bmb (12 new tests)
- gen_env_alloc: Generate EnvAlloc instruction for closure environments
- gen_env_store, gen_env_stores: Generate EnvStore instructions for captured variables
- gen_closure_with_env: Generate ClosureEnv instruction with function reference
- lowering.bmb total: 188 tests

**v0.30.102 Completed (2026-01-06)**:
- Static trait dispatch lowering in lowering.bmb (8 new tests)
- lower_trait_call_static: Generate call @Trait_Type_method for concrete types
- lower_trait_call_smart: Auto-select between static dispatch and TraitCall
- Enables direct function calls when receiver type is known

**v0.30.101 Completed (2026-01-06)**:
- Trait impl lookup integration in lowering.bmb (19 new tests)
- static_method_symbol: Generate @Trait_Type_method format symbols
- is_concrete_type: Check if type is concrete for static dispatch
- gen_static_dispatch: Generate static dispatch call instructions
- Expanded infer_trait_from_method: 19 additional trait-method mappings

**v0.30.100 Completed (2026-01-06)**:
- End-to-end integration tests: trait_closure_integration.bmb
- Tests trait dispatch and closure capture together
- Validates complete TraitCall and Capture MIR generation pipeline

**v0.30.99 Completed (2026-01-06)**:
- Closure free variable analysis in lowering.bmb (23 new tests)
- lambda_extract_params, strip_param_parens: extract lambda parameters
- param_extract_name, collect_param_names: process parameter list
- collect_var_refs, collect_var_refs_at, find_var_pattern: scan for variable references
- filter_free_vars, is_in_names: filter out bound parameters
- count_names, name_at_index: utility functions for comma-separated name lists
- gen_captures: generate Capture MIR instructions
- Updated lower_lambda: free variable analysis → Capture instruction generation
- lowering.bmb total: 149 tests

**v0.30.98 Completed (2026-01-06)**:
- TraitCall MIR generation in lowering.bmb (18 new tests)
- is_builtin_method: detect String/Array built-in methods (len, slice, push, etc.)
- is_trait_method: negation of builtin check
- infer_trait_from_method: map method names to trait names (show→Display, clone→Clone)
- lower_trait_call: generate TraitCall MIR with Trait::method format
- Updated lower_method_call: route based on method type
- lowering.bmb total: 126 tests

**v0.30.97 Completed (2026-01-06)**:
- Closure capture IR infrastructure: test_closure_capture_ir (12 tests)
- is_capture_line, is_load_capture_line: detect capture MIR instructions
- parse_capture_closure, parse_capture_var, parse_capture_idx: extract capture info
- gen_closure_env_alloc, gen_capture_store, gen_capture_load: environment operations
- gen_closure_with_captures: full closure struct generation
- llvm_ir.bmb total: 283 tests (57 test functions)

**v0.30.96 Completed (2026-01-06)**:
- Trait dispatch IR infrastructure: test_trait_dispatch_ir (8 tests)
- is_trait_call_line: detect TraitCall MIR instructions
- parse_trait_call_trait, parse_trait_call_method: extract trait/method names
- gen_trait_dispatch: static dispatch IR generation (@Trait_Type_method)
- gen_instr_trait_call: full trait call IR conversion
- llvm_ir.bmb total: 271 tests (56 test functions)

**v0.30.95 Completed (2026-01-06)**:
- Struct/Enum IR edge cases: test_struct_enum_edge (8 tests)
- Multiple field struct chaining, extractvalue at index 2
- Enum with large discriminants (99), non-sequential discriminants
- llvm_ir.bmb total: 263 tests (55 test functions)

**v0.30.94 Completed (2026-01-06)**:
- If-then-else control flow completeness: test_if_control_flow (8 tests)
- then/else/merge label generation, conditional branch with block indices
- phi node with multiple sources, terminator line detection
- llvm_ir.bmb total: 255 tests (54 test functions)

**v0.30.93 Completed (2026-01-06)**:
- Match expression IR testing: test_match_ir (8 tests)
- match_arm/match_merge label generation, switch with multiple arms
- goto match_merge, switch_line detection for match
- llvm_ir.bmb total: 247 tests (53 test functions)

**v0.30.87 Completed (2026-01-05)**:
- Trait constraint checking: type_satisfies_trait, trait_methods_match, impl_has_method
- Type bounds checking: type_satisfies_bounds, type_satisfies_bounds_from
- Impl query functions: tenv_get_impls_for_type, tenv_impl_count_for_type
- 2 new test functions (11 assertions)

**v0.30.86 Completed (2026-01-05)**:
- Impl registry: gen_impl_pack, gen_impl_trait_name, gen_impl_target_type, gen_impl_methods
- impl_reg_new, impl_reg_add, impl_reg_lookup, impl_reg_has, impl_reg_method_return
- tenv impl integration: tenv_add_impl, tenv_has_impl, tenv_impl_method_return
- 4 new test functions (22 assertions)

**v0.30.85 Completed (2026-01-05)**:
- Trait registry: gen_trait_pack, gen_trait_name, gen_trait_tparams, gen_trait_methods
- Trait method storage: gen_trait_method_count, gen_trait_has_method, gen_trait_method_return
- trait_reg_new, trait_reg_add, trait_reg_lookup, trait_reg_has, trait_reg_is_generic
- tenv trait integration: tenv_add_trait, tenv_has_trait, tenv_trait_method_return
- Updated tenv format: "P:#S:#E:#F:#T:#I:" for trait and impl registries
- Helper functions: find_char, count_commas for string processing
- 4 new test functions (21 assertions)

**v0.30.84 Completed (2026-01-05)**:
- String eq method IR: gen_method_eq for string equality comparison
- Runtime call to @bmb_string_eq(i8*, i8*) -> i64

**v0.30.83 Completed (2026-01-05)**:
- String concat/eq method IR: gen_method_concat, gen_method_eq
- gen_method_dispatch updated for concat and eq routing
- Runtime calls to @bmb_string_concat, @bmb_string_eq
- 6 new tests in test_method_call_ir (now 18 tests)
- llvm_ir total: 229 tests (51 test functions)

**v0.30.82 Completed (2026-01-05)**:
- Pipeline integration tests: test_pipeline_integration with 8 tests
- Tests MIR→LLVM IR pipeline for functions, control flow, closures
- Verified gen_function, gen_control_flow, gen_closure_ir pipeline

**v0.30.81 Completed (2026-01-05)**:
- Enhanced runtime declarations: gen_extern_string_concat, gen_extern_string_eq
- Memory runtime: gen_extern_alloc, gen_extern_free
- gen_runtime_decls combines basic, string, array, array_mut, memory decls
- 6 new tests in test_runtime_decls (now 22 tests)

**v0.30.80 Completed (2026-01-05)**:
- Array mutation methods: gen_method_array_push, gen_method_array_pop, gen_method_array_clear
- Runtime declarations: gen_extern_array_push, gen_extern_array_pop, gen_extern_array_clear
- gen_method_dispatch updated for array mutation routing
- 6 new tests in test_array_method_ir (now 16 tests)

**v0.30.79 Completed (2026-01-05)**:
- Array method IR tests: 10 tests for array_len, array_get, array_set
- test_array_method_ir: Comprehensive tests for all array method IR
- gen_extern_array_* declarations verified in tests
- llvm_ir total: 203 tests (49 test functions)

**v0.30.78 Completed (2026-01-05)**:
- Array method IR generation: gen_method_array_len, gen_method_array_get, gen_method_array_set
- gen_method_dispatch updated for array_len, array_get, array_set routing
- Runtime calls to @bmb_array_len, @bmb_array_get, @bmb_array_set

**v0.30.77 Completed (2026-01-05)**:
- Array runtime declarations: gen_extern_array_len, gen_extern_array_get, gen_extern_array_set
- Runtime declarations for @bmb_array_len(i8*), @bmb_array_get(i8*, i64), @bmb_array_set(i8*, i64, i64)
- gen_runtime_decls updated to include all array runtime functions

**v0.30.76 Completed (2026-01-05)**:
- String.slice IR: `MethodCall %recv.slice(%start, %end)` → external call
- gen_method_slice: Call @bmb_string_slice(i8*, i64, i64)
- extract_slice_arg, extract_first_arg, extract_second_arg: Argument parsing
- gen_extern_string_slice: Runtime declaration for string slice
- Complete method call pipeline for all String methods

**v0.30.75 Completed (2026-01-05)**:
- String.char_at IR: `MethodCall %recv.char_at(%idx)` → GEP + load + sext
- gen_method_char_at: Generate getelementptr i8 + load i8 + sext to i64
- Character access via pointer arithmetic on i8* string

**v0.30.74 Completed (2026-01-05)**:
- String.len IR: `MethodCall %recv.len()` → external call @bmb_string_len
- gen_method_len: Generate call to @bmb_string_len(i8*)
- gen_extern_string_len: Runtime declaration for string length

**v0.30.73 Completed (2026-01-05)**:
- MethodCall MIR detection and dispatch in gen_mir_line_typed
- is_method_call_line, has_method_call_keyword: Line detection
- parse_method_call_dest, parse_method_call_recv: MIR parsing
- parse_method_call_method, parse_method_call_args: Method/args extraction
- gen_method_dispatch: Route to specific method implementations
- 12 method call IR tests, llvm_ir total: 193 tests

**v0.30.72 Completed (2026-01-05)**:
- Index expression LLVM IR: `%dest = Index %base[%idx]` → GEP + load
- gen_instr_index: Generate getelementptr and load instructions
- read_until_bracket, extract_index_expr: Parsing helpers
- extract_until_close_bracket: Extract index from bracket syntax
- 6 index IR tests, llvm_ir total: 181 tests

**v0.30.71 Completed (2026-01-05)**:
- Array literal LLVM IR: `%dest = Array [%e1, %e2]` → alloca + GEP + stores
- gen_instr_array: Generate array allocation and element stores
- count_array_elems, count_commas: Element counting utilities
- gen_array_stores, extract_array_elem: Store instruction generation
- trim_ws, trim_end_ws: Whitespace handling for element extraction
- 6 array IR tests

**v0.30.70 Completed (2026-01-05)**:
- Range operator LLVM IR: `..` and `..=` → insertvalue {i64, i64}
- is_range_op: Detect range operators in binop dispatch
- gen_binop_range: Generate insertvalue sequence for Range<i64> struct
- LLVM IR output: `{i64, i64}` tuple with start and end values
- 6 range IR tests

**v0.30.69 Completed (2026-01-05)**:
- Range operator type checking: `..` and `..=` operators in binop_result_type
- Range type handling: Returns `Range<i64>` for valid range expressions
- Range error detection: Reports type error for non-i64 operands
- 6 range type checking tests, types total: 650 tests

**v0.30.68 Completed (2026-01-05)**:
- Method call type checking: `(method_call receiver <method> args)` expressions
- tenv_method_lookup: Built-in method type lookup (String.len, String.slice)
- type_of_method_call: Determine return type based on receiver and method
- method_call_receiver, method_call_name: AST extraction helpers
- EXPR_METHOD_CALL constant (23) for expr_kind detection
- 8 method call type checking tests, types total: 644 tests

**v0.30.67 Completed (2026-01-05)**:
- Index expression type checking: `(index base idx)` expressions
- type_of_index: Extract element type from array type
- index_base_expr, index_index_expr: AST component extraction
- array_element_type: Strip `[` and `]` from array type notation
- EXPR_INDEX constant (22) for expr_kind detection
- 7 index type checking tests

**v0.30.66 Completed (2026-01-05)**:
- Array literal type checking: `(array elem1 elem2 ...)` expressions
- type_of_array: Infer element type from first element or unit for empty
- array_element_at, array_element_count: Element access and counting
- EXPR_ARRAY constant (21) for expr_kind detection
- 8 array type checking tests

**v0.30.65 Completed (2026-01-05)**:
- Index expression MIR lowering: `(index expr idx)` → MIR Index instruction
- lower_index: Lower base and index expressions, generate Index MIR
- MIR format: `%dest = Index %base[%idx]`
- is_index_node: Node type detection for index expressions
- 4 index lowering tests, lowering total: 108 tests

**v0.30.64 Completed (2026-01-05)**:
- Array literal MIR lowering: `(array elem1 elem2)` → MIR Array instruction
- lower_array, lower_array_elements: Recursive element lowering
- MIR format: `%dest = Array [%elem1, %elem2, ...]`
- is_array_node: Node type detection for array literals
- 5 array lowering tests

**v0.30.63 Completed (2026-01-05)**:
- Method call parsing: `obj.method(args)` syntax support
- parse_postfix extended: Detect `(` after field name for method calls
- parse_method_args, parse_method_args_more: Argument list handling
- AST format: `(method_call receiver <method> arg1 arg2 ...)`
- Method call MIR lowering: lower_method_call, lower_method_args
- MIR format: `%dest = MethodCall %recv.method(args)`
- is_method_call_node, get_method_name: Node detection and extraction
- 5 parser tests (104 total), 4 lowering tests

**v0.30.62 Completed (2026-01-05)**:
- Array type parsing: `[T]` type syntax in params and return types
- parse_type: Delegated to parse_type_or_ident for unified array support
- Nested arrays: `[[i64]]` for 2D array types
- AST format: `(array_type T)` for array types
- 6 array type tests, parser total: 99 tests

**v0.30.61 Completed (2026-01-05)**:
- Index expression parsing: `arr[i]` syntax for array element access
- parse_postfix extended: Handle TK_LBRACKET for index operations
- Chained indexing: `arr[i][j]` for multi-dimensional arrays
- AST format: `(index expr index_expr)`
- 5 index expression tests

**v0.30.60 Completed (2026-01-05)**:
- Array literal parsing: `[1, 2, 3]` syntax for array construction
- parse_array_literal, parse_array_elements: Array element parsing
- TK_LBRACKET (313), TK_RBRACKET (314): Bracket token support
- AST format: `(array expr1 expr2 ...)`
- 5 array literal tests

**v0.30.59 Completed (2026-01-05)**:
- Impl block parsing: parse_impl_block function with generic support
- Generic impl: `impl<T> TraitName<T> for Type { ... }`
- Self parameter: Updated parse_params to handle 'self' without type annotation
- AST format: `(impl [type_params] trait_name target (methods (fn ...)))`
- 5 impl parsing tests, parser total: 83 tests

**v0.30.58 Completed (2026-01-05)**:
- Trait definition parsing: parse_trait_def with generic support
- Trait methods: parse_trait_methods, parse_trait_method_sig, parse_trait_params
- Self parameter handling: `(param <self>)` for methods
- AST format: `(trait <Name> [type_params] (methods (method-sig ...)))`
- 5 trait parsing tests

**v0.30.57 Completed (2026-01-05)**:
- Trait token support: TK_TRAIT (127) and TK_IMPL (128) tokens
- Keyword recognition: 'trait' and 'impl' in lookup_keyword
- Token tests: test_trait_token, test_impl_token

**v0.30.56 Completed (2026-01-05)**:
- End-to-end LLVM IR tests: Complete MIR to LLVM IR function generation tests
- Return type tracking: gen_function now extracts return type for proper terminator generation
- extract_return_type: Parse return type from MIR function header
- gen_terminator_typed: Pass return type to terminator for correct `ret i1`/`ret i64`
- test_full_compare_function: Validates comparison operators generate `icmp sle`
- test_full_logic_function: Validates logic operators generate `and i1`/`or i1`/`xor i1`
- Total: 163 tests passing in llvm_ir.bmb (156 + 7 new)

**v0.30.55 Completed (2026-01-05)**:
- Pipeline associativity tests: Verify left-to-right operator chaining
- Pipeline unary tests: Verify nested unary operator handling
- test_pipeline extended: 8 new test cases for operator precedence
- Tests: `a + b + c`, `a * b / c`, `not not a`, `-(-x)`, `-(a + b)`
- Total: 30 tests passing in pipeline.bmb (22 + 8 new)

**v0.30.54 Completed (2026-01-05)**:
- Pipeline comparison/logic tests: End-to-end verification of comparison and logic operators
- find_child_end bug fix: Distinguish `<` operator from `<name>` pattern
- low_is_ident_char check: Verify next char before treating `<` as name delimiter
- test_pipeline extended: 8 new test cases for `<`, `>`, `<=`, `>=`, `and`, `or`, `!=`, `==`
- Total: 22 tests passing in pipeline.bmb (14 + 8 new)

**v0.30.53 Completed (2026-01-05)**:
- Block expression LLVM IR test: Verification of block pass-through behavior
- test_block_ir: Tests block expressions with various inner expressions
- Integer/boolean/arithmetic/UNIT/string/closure inner expressions verified
- Block expressions correctly pass through to inner expression codegen
- Total: 156 tests passing in llvm_ir.bmb (150 + 6 new)

**v0.30.52 Completed (2026-01-05)**:
- Closure LLVM IR generation: CLOSURE instruction codegen support
- gen_instr_closure: Generate closure as i8* pointer (simplified representation)
- is_closure_op: Detect CLOSURE prefix in instruction
- parse_closure_id: Extract closure ID from CLOSURE:N format
- gen_instr_dispatch extended: Handle CLOSURE instructions
- Total: 150 tests passing in llvm_ir.bmb (143 + 7 new)

**v0.30.51 Completed (2026-01-05)**:
- String constant LLVM IR: S: type constant codegen support
- parse_const_type extended: Recognize 'S' (ASCII 83) as string type
- parse_const_string: Extract string content after S: prefix
- gen_const_string: Generate string as comment + i8* inttoptr
- gen_instr_const extended: Handle string type constants
- Total: 143 tests passing in llvm_ir.bmb (137 + 6 new)

**v0.30.50 Completed (2026-01-05)**:
- For loop LLVM IR test: End-to-end verification of for loop MIR to LLVM IR conversion
- test_for_ir: Tests for_start/for_body/for_end label generation
- Label generation verified: for_start_0, for_body_0, for_end_0
- Branch instruction verified: br i1 %cond, label %for_body, label %for_end
- Goto instruction verified: br label %for_start_0
- UNIT at loop end verified: add i64 0, 0
- Total: 137 tests passing in llvm_ir.bmb (131 + 6 new)

**v0.30.49 Completed (2026-01-05)**:
- While loop LLVM IR test: End-to-end verification of while loop MIR to LLVM IR conversion
- test_while_ir: Tests loop_start/body/loop_end label generation
- Label generation verified: loop_start_0, loop_end_0
- Branch instruction verified: br i1 %_t0 pattern
- Goto instruction verified: br label %loop_start_0
- UNIT at loop end verified: add i64 0, 0
- Terminator detection verified: branch pattern recognition
- Total: 131 tests passing in llvm_ir.bmb (125 + 6 new)

**v0.30.48 Completed (2026-01-05)**:
- UNIT instruction LLVM IR: Support for unit value in LLVM IR generation
- gen_instr_unit: Generate unit value as `add i64 0, 0`
- gen_instr_break: Generate BREAK placeholder with comment
- gen_instr_continue: Generate CONTINUE placeholder with comment
- gen_instr_dispatch extended: Handle UNIT, BREAK, CONTINUE operations
- test_unit_break_continue: 6 tests for unit/break/continue instructions
- Total: 125 tests passing in llvm_ir.bmb (119 + 6 new)

**v0.30.47 Completed (2026-01-05)**:
- Break/Continue type checking: Type system support for break and continue expressions
- EXPR_BREAK (19): Expression kind constant for break
- EXPR_CONTINUE (20): Expression kind constant for continue
- expr_kind extended: Detect (break) and (continue) expressions
- type_of_break: Type check break expression (returns unit)
- type_of_continue: Type check continue expression (returns unit)
- type_of_expr extended: Dispatch to type_of_break/type_of_continue
- Total: 621 tests passing in types.bmb (613 + 8 new)

**v0.30.46 Completed (2026-01-05)**:
- For type checking: Type system support for for loop expressions
- EXPR_FOR (18): Expression kind constant for for loops
- expr_kind extended: Detect (for <var> range body) expressions
- for_var_name: Extract loop variable name from for expression
- for_range_expr: Extract range expression from for AST
- for_body_expr: Extract body expression from for AST
- type_of_for: Type check for expression (binds loop var to i64, returns unit)
- type_of_expr extended: Dispatch to type_of_for
- Total: 613 tests passing in types.bmb (606 + 7 new)

**v0.30.45 Completed (2026-01-05)**:
- While type checking: Type system support for while loop expressions
- EXPR_WHILE (17): Expression kind constant for while loops
- expr_kind extended: Detect (while cond body) expressions
- type_of_while: Type check while expression (condition must be bool, returns unit)
- type_of_expr extended: Dispatch to type_of_while
- Total: 606 tests passing in types.bmb (600 + 6 new)

**v0.30.44 Completed (2026-01-05)**:
- Range MIR lowering: MIR generation for range expressions
- binop_from_symbol extended: Map ".." to 14, "..=" to 15
- binop_symbol extended: Return ".." and "..=" for range operators
- is_op_char extended: Include '.' (ASCII 46) for range operators
- Range expressions lowered via existing lower_binop infrastructure
- Total: 95 tests passing in lowering.bmb (91 + 4 new)

**v0.30.43 Completed (2026-01-05)**:
- Break/Continue MIR lowering: MIR generation for break and continue
- is_break_node: Detect break expressions "(break)"
- is_continue_node: Detect continue expressions "(continue)"
- lower_break: Generate BREAK instruction (placeholder for loop exit)
- lower_continue: Generate CONTINUE instruction (placeholder for loop restart)
- lower_expr extended: Handle break/continue nodes
- Total: 91 tests passing in lowering.bmb (85 + 6 new)

**v0.30.42 Completed (2026-01-05)**:
- Break/Continue parsing: Full support for break and continue statements
- TK_BREAK (125): Token for break keyword
- TK_CONTINUE (126): Token for continue keyword
- parse_break: Parser function generating (break) AST node
- parse_continue: Parser function generating (continue) AST node
- parse_primary extended: Handle TK_BREAK and TK_CONTINUE
- Total: 71 tests passing in parser_ast.bmb (66 + 5 new)

**v0.30.41 Completed (2026-01-05)**:
- Range expression parsing: Exclusive (..) and inclusive (..=) range support
- TK_DOTDOTEQ (324): Token for inclusive range operator ..=
- check_three_char: Three-character token detection for ..=
- is_binop extended: Include TK_DOTDOTEQ as binary operator
- op_str extended: Map TK_DOTDOTEQ to "..=" string
- next_token_raw extended: Check three-char before two-char tokens
- Total: 66 tests passing in parser_ast.bmb (61 + 5 new)

**v0.30.40 Completed (2026-01-05)**:
- For MIR lowering: For loop MIR generation in lowering.bmb
- is_for_node: Detect for expressions "(for <var> iter body)"
- get_for_var/get_for_iter/get_for_body: Helper functions for AST extraction
- extract_for_varname: Extract variable name from "<varname>" format
- lower_for: Generate loop MIR structure (for_start, for_body, for_end blocks)
- lower_expr extended: Handle for nodes via lower_for call
- Total: 85 tests passing in lowering.bmb (79 + 6 new)

**v0.30.39 Completed (2026-01-05)**:
- For loop parsing: For loop syntax support in parser_ast.bmb
- TK_FOR (123): Token for for keyword
- TK_IN (124): Token for in keyword
- lookup_keyword extended: Map "for" and "in" to tokens
- parse_for: Parse "for var in iter { body }" to "(for <var> iter body)"
- parse_primary extended: Handle TK_FOR via parse_for call
- Total: 61 tests passing in parser_ast.bmb (56 + 5 new)

**v0.30.38 Completed (2026-01-05)**:
- While MIR lowering: While loop MIR generation in lowering.bmb
- is_while_node: Detect while expressions "(while cond body)"
- get_while_cond/get_while_body: Helper functions for AST extraction
- lower_while: Generate loop MIR structure (loop_start, body, loop_end blocks)
- lower_expr extended: Handle while nodes via lower_while call
- Total: 79 tests passing in lowering.bmb (73 + 6 new)

**v0.30.37 Completed (2026-01-05)**:
- While expression parsing: While loop syntax support in parser_ast.bmb
- TK_WHILE (122): Token for while keyword
- lookup_keyword extended: Map "while" to TK_WHILE
- parse_while: Parse "while condition { body }" to "(while cond body)"
- parse_primary extended: Handle TK_WHILE via parse_while call
- Total: 56 tests passing in parser_ast.bmb (51 + 5 new)

**v0.30.36 Completed (2026-01-05)**:
- Block/Unit MIR lowering: Block and unit expression support in lowering.bmb
- is_block_node: Detect block expressions "(block inner_expr)"
- is_unit_node: Detect unit expressions "()" exactly
- block_inner_expr: Extract inner expression from block AST
- lower_block: Lower block by delegating to inner expression
- lower_unit: Generate MIR UNIT constant for unit expressions
- lower_expr extended: Handle block and unit nodes
- Total: 73 tests passing in lowering.bmb (64 + 9 new)

**v0.30.35 Completed (2026-01-05)**:
- Lambda expression parsing: Full lambda syntax support in parser_ast.bmb
- TK_PIPE (309): Token for | pipe character
- symbol_code extended: Map | (ASCII 124) to TK_PIPE
- parse_lambda: Parse "fn |params| body" or "fn |params| -> type body"
- parse_lambda_params, parse_lambda_params_more: Lambda parameter parsing
- parse_primary extended: Handle TK_FN followed by TK_PIPE as lambda
- Total: 51 tests passing in parser_ast.bmb (42 + 9 new)

**v0.30.34 Completed (2026-01-05)**:
- Lambda/closure MIR lowering: Closure support in lowering.bmb
- is_lambda_node: Detect lambda expressions "(fn |...| body)"
- lambda_find_pipe, lambda_body_start, lambda_extract_body: Body extraction helpers
- lower_lambda: Generate MIR CLOSURE: prefix for closure references
- lower_expr extended: Handle lambda nodes via is_lambda_node check
- Total: 64 tests passing in lowering.bmb (55 + 9 new)

**v0.30.33 Completed (2026-01-05)**:
- MIR string lowering: String literal support in lowering.bmb
- is_string_node: Detect string literals starting with char 34 (quote)
- lower_string: Generate MIR S: prefix for string constants
- lower_expr extended: Handle string nodes via is_string_node check
- Total: 55 tests passing in lowering.bmb (52 + 3 new)

**v0.30.32 Completed (2026-01-05)**:
- Parser string literal support: Full string tokenization and AST generation
- TK_STRING constant (202): New token type for string literals
- find_string_end: Find closing quote position for string scanning
- next_token_raw extended: Detect strings starting with char 34 (quote)
- parse_primary extended: Handle TK_STRING tokens, keep raw form with quotes
- Total: 42 tests passing in parser_ast.bmb (39 + 3 new)

**v0.30.31 Completed (2026-01-05)**:
- Unit type support: EXPR_UNIT constant = 16
- expr_kind extended: Detects "()" exactly (len==2, chars 40 and 41)
- type_of_expr: Returns "()" type for unit expressions
- Total: 600 tests passing (combined with v0.30.30)

**v0.30.30 Completed (2026-01-05)**:
- Block expression type checking: EXPR_BLOCK constant = 15
- expr_kind extended: Detects "(block" pattern via 'b','l','o' chars
- type_of_block: Returns type of inner expression
- block_inner_expr: Extracts inner expression from "(block expr)"
- Total: 600 tests passing (8 new tests with v0.30.31)

**v0.30.29 Completed (2026-01-05)**:
- String literal type checking: EXPR_STRING constant and quote char detection
- expr_kind extended: Detects strings starting with char 34 (quote) before checking '('
- type_of_expr: Returns "String" type for EXPR_STRING expressions
- Total: 592 tests passing (5 new tests)

**v0.30.28 Completed (2026-01-05)**:
- Generic field access type checking: Fixed type_of_field to handle generic types
- parse_type_base: Extract base type from generic (e.g., "Vec<i64>" → "Vec")
- parse_type_args: Extract type arguments (e.g., "Vec<i64>" → "i64")
- type_str_find_angle: Find '<' position in type string
- type_str_find_close_angle: Find matching '>' with depth tracking
- type_has_args: Check if type has generic arguments
- locals_find_comma_depth: Handle commas inside generic type args
- Total: 587 tests passing (11 new tests)

**v0.30.27 Completed (2026-01-05)**:
- Struct instantiation type checking: Enhanced type_of_new with field validation
- expr_new_field_count: Count field initializers in new expression
- expr_new_field_at: Get field at index (returns "(fieldname expr)")
- new_field_name: Extract field name from field initializer
- new_field_expr: Extract field expression from field initializer
- check_new_fields: Recursive field type validation against struct definition
- Total: 576 tests passing (9 new tests)

**v0.30.26 Completed (2026-01-05)**:
- let-mut type checking: Fixed offset calculation for "(let-mut" expressions
- is_let_mut_expr helper: Detects let-mut via char_at(4) == '-'
- let_prefix_len helper: Returns 8 for let-mut, 4 for let
- expr_let_name fix: Uses dynamic prefix length for name extraction
- type_of_let fix: Uses dynamic prefix for value/body position calculation
- Total: 567 tests passing (9 new tests)

**v0.30.25 Completed (2026-01-05)**:
- Unary operator type checking: type_of_not and type_of_neg functions
- EXPR_NOT constant (kind=12): Logical not expression "(not expr)"
- EXPR_NEG constant (kind=13): Unary negation expression "(neg expr)"
- Expression detection: expr_kind extended for (not and (neg patterns
- Type validation: bool for not, i64 for neg
- Inner expression extraction: not_inner_expr, neg_inner_expr
- Type checking integration: type_of_expr routing for new kinds
- Total: 558 tests passing (9 new tests)

**v0.30.24 Completed (2026-01-05)**:
- Closure type checking: type_of_lambda for lambda expression type inference
- EXPR_LAMBDA constant (kind=11): Lambda expression detection in expr_kind
- Lambda detection: expr_kind_check_lambda for "(fn |..." pattern
- Parameter section parsing: lambda_params_section extracting "|params|"
- Parameter extraction: lambda_param_count, lambda_param_at, lambda_param_name, lambda_param_type
- Lambda body parsing: lambda_body, lambda_body_start, lambda_find_pipe_end
- Return type handling: lambda_has_arrow_prefix, lambda_return_type
- Local scope building: lambda_build_locals, lambda_build_param_types
- Fn type construction: "Fn(T1,T2,...)->R" format generation
- Type checking integration: type_of_expr → type_of_lambda routing
- Condition order fix in ast_find_close_paren_depth: Check depth==0 before pos>=len
- Total: 549 tests passing (17 new tests)

**v0.30.23 Completed (2026-01-05)**:
- Match expression type checking: type_of_match for pattern matching
- Match scrutinee extraction: match_scrutinee from (match expr (arms ...))
- Arms section parsing: match_arms_section, match_arm_count, match_arm_at
- Single arm type checking: type_of_match_arm with pattern/body extraction
- Pattern extraction: arm_pattern, arm_body from (arm (pattern ...) body)
- Variant/binding extraction: pattern_variant, pattern_binding
- Binding scope extension: extend_locals_with_binding for pattern variables
- Type consistency checking: Validates all match arms return same type
- Error detection: ERR:match arm types differ for mismatched branches
- Total: 532 tests passing (15 new tests)

**v0.30.22 Completed (2026-01-05)**:
- Generic function body type checking: Type parameter scope for function bodies
- Modified check_fn_body to extract and set type parameters in tenv
- Uses ast_extract_type_params to get function's type parameters
- Uses tenv_with_tparams to create function-scoped type environment
- Supports fn identity<T>(x: T) -> T = x pattern
- Correctly validates generic return types match body types (T == T)
- Detects type mismatches in generic functions (T vs U)
- Total: 517 tests passing (8 new tests)

**v0.30.21 Completed (2026-01-05)**:
- Function body type checking: Complete program-wide type validation pipeline
- ast_extract_fn_body: Extract function body expression from AST
- ast_extract_param_name, ast_extract_param_type: Parameter parsing
- ast_extract_params_section, ast_count_params, ast_get_param_at: Params section utilities
- ast_params_to_locals: Convert function params to locals environment
- check_fn_body: Validate function body type matches declared return type
- check_program_functions: Check all functions in a program
- typecheck_program: Full pipeline - build tenv, then validate all functions
- Total: 509 tests passing (18 new tests)

**v0.30.20 Completed (2026-01-05)**:
- Expression type checking: type_of_expr for S-expression AST inference
- Local variable environment: locals_new, locals_add, locals_lookup
- Expression kind detection: EXPR_INT, EXPR_BOOL, EXPR_VAR, EXPR_OP, EXPR_IF, EXPR_LET, EXPR_CALL, EXPR_NEW, EXPR_FIELD, EXPR_MATCH
- Literal type checking: (int n) → i64, (bool n) → bool
- Variable type checking: (var <name>) → lookup in locals
- Operator type checking: type_of_unop, type_of_binop, binop_result_type
- Control flow type checking: type_of_if (condition bool, branches match)
- Let binding type checking: type_of_let with scope extension
- Function call type checking: type_of_call with argument type collection
- Struct construction/field access: type_of_new, type_of_field
- Error propagation: is_error_str for String-based type error detection
- Total: 491 tests passing (21 new tests)

**v0.30.19 Completed (2026-01-05)**:
- Program AST Traversal: Navigate `(program ...)` S-expressions from parser_ast.bmb
- Item kind detection: ITEM_FN, ITEM_STRUCT, ITEM_ENUM constants
- `ast_item_kind`: Detect item type from AST prefix (fn, struct, enum)
- `ast_program_start`: Find position after "(program " prefix
- `ast_extract_item_at`: Extract complete item S-expression at position
- `ast_next_item_pos`: Get position of next item
- `ast_program_item_count`, `ast_program_item_at`: Count and access items by index
- `register_item`: Route item registration based on kind
- `tenv_from_program_ast`: Main entry point - build complete tenv from program AST
- Total: 470 tests passing (19 new tests)

**v0.30.18 Completed (2026-01-05)**:
- AST-Type Integration: Connect parser_ast.bmb output to types.bmb tenv system
- AST navigation utilities: ast_find_close_paren, ast_skip_ws, ast_find_pattern
- AST name extraction: ast_extract_angle_name, ast_extract_def_name
- Type parameter extraction: ast_extract_type_params (e.g., `(type_params <T> <U>)` → "T,U")
- Fields extraction: ast_extract_fields (e.g., `(fields (field <x> i64))` → "x:i64")
- Variants extraction: ast_extract_variants (e.g., `(variants (variant <Some> T))` → "Some:T")
- Function signature extraction: ast_extract_param_types, ast_extract_return_type
- AST to registry converters: ast_struct_to_def, ast_enum_to_def, ast_fn_to_sig
- tenv registration from AST: register_struct_from_ast, register_enum_from_ast, register_fn_from_ast
- Total: 451 tests passing (45 new tests)

**v0.30.17 Completed (2026-01-04)**:
- Generic call site type checking through type environment
- tenv_check_fn_call, tenv_infer_fn_call for function calls
- tenv_check_field_access, tenv_check_match_variant for data types
- Total: 406 tests passing

**v0.30.16 Completed (2026-01-04)**:
- Unified type environment for all registries
- Type parameter, struct, enum, function registry integration
- `tenv_*` family of functions (27 tests)
- Total: 389 tests passing

**v0.30.15 Completed (2026-01-04)**:
- `fn_reg_new`, `fn_reg_add`: Create and populate function registry
- `fn_reg_lookup`, `fn_reg_has`: Query registry for function signatures
- `fn_reg_return_type`: Get return type with type arguments applied
- `fn_reg_param_type_at`: Get parameter type at index with type arguments applied
- `fn_reg_count`, `fn_reg_is_generic`, `fn_reg_param_count`: Registry utilities
- 7 new test functions, 27 assertions (362 total in types.bmb)

**Deliverables**:
- Bootstrap compiler can type-check generic code
- Trait dispatch works in bootstrap
- Closure capture and codegen functional

#### Phase 30.2: Compiler Porting (lang-bmb)

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 30.2.1 | Port main.rs CLI to BMB | P0 | 2 weeks |
| 30.2.2 | Port AST types to BMB | P0 | 2 weeks |
| 30.2.3 | Port full MIR module to BMB | P0 | 4 weeks |
| 30.2.4 | Port codegen module to BMB | P0 | 3 weeks |
| 30.2.5 | Stage 3 verification | P0 | 2 weeks |

**Deliverables**:
- Complete BMB compiler written in BMB
- Stage 3 binary identical to Stage 2

#### Phase 30.3: Package Manager Porting (gotgan)

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 30.3.1 | Port registry client to BMB | P1 | 2 weeks |
| 30.3.2 | Port dependency resolver to BMB | P1 | 2 weeks |
| 30.3.3 | Port build system to BMB | P1 | 3 weeks |
| 30.3.4 | Port CLI and config to BMB | P1 | 1 week |

**Deliverables**:
- gotgan package manager written in BMB
- Full feature parity with Rust version

#### Phase 30.4: Rust Removal

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 30.4.1 | Remove bmb/src/*.rs | P0 | 1 week |
| 30.4.2 | Remove gotgan/src/*.rs | P0 | 1 week |
| 30.4.3 | Remove Cargo.toml files | P0 | 1 day |
| 30.4.4 | Update CI/CD for pure BMB | P0 | 1 week |

**Success Criteria**:
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

---

### v0.31 Docs - Documentation Completion

**Goal**: Comprehensive documentation, language refinement, and website launch

**Difficulty**: ⭐⭐⭐ (Medium)

**Duration Estimate**: 6-8 weeks

#### Phase 31.0: Language Refinements (RFCs)

| Task | Description | Priority | Effort | RFC |
|------|-------------|----------|--------|-----|
| 31.0.1 | Remove @check annotation | P0 | 1 day | RFC-0003 |
| 31.0.2 | Add @trust "reason" mandatory reason | P0 | 1 day | RFC-0003 |
| 31.0.3 | Add `todo` keyword | P0 | 2 days | RFC-0004 |
| 31.0.4 | Add module header system | P0 | 1 week | RFC-0002 |
| 31.0.5 | Add contract hash duplicate detection | P1 | 3 days | - |
| 31.0.6 | Update SPECIFICATION.md | P0 | 1 day | - |

**Deliverables**:
- @check removed, @trust requires reason
- `todo` keyword for incremental development
- Module headers for AI-friendly navigation
- Contract duplicate warning in build

**Philosophy Alignment**:
- P0 Performance: No runtime contract checks (@check removed)
- AI-Native: Module headers enable fast navigation
- Incremental Development: `todo` supports contract-first workflow

#### Phase 31.1: Language Reference

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.1.1 | Complete language syntax reference | P0 | 1 week |
| 31.1.2 | Document type system and generics | P0 | 1 week |
| 31.1.3 | Document contract system (pre/post/invariant) | P0 | 1 week |
| 31.1.4 | Document memory model (ownership/borrowing) | P0 | 1 week |

**Deliverables**:
- Complete language reference document
- Interactive examples for each feature

#### Phase 31.2: Standard Library Documentation

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.2.1 | Generate API documentation for stdlib | P0 | 1 week |
| 31.2.2 | Add usage examples for each module | P1 | 1 week |
| 31.2.3 | Document contract specifications | P1 | 1 week |

**Deliverables**:
- Searchable API documentation
- Code examples for all public functions

#### Phase 31.3: Tutorials and Guides

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.3.1 | Write "Getting Started" tutorial | P0 | 1 week |
| 31.3.2 | Write "By Example" guide | P0 | 2 weeks |
| 31.3.3 | Write "From Rust" migration guide | P1 | 1 week |
| 31.3.4 | Write "Contract Programming" guide | P1 | 1 week |

**Deliverables**:
- Step-by-step getting started guide
- Comprehensive example collection
- Migration guides for Rust/C developers

#### Phase 31.4: Website Launch (bmb.dev)

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 31.4.1 | Deploy documentation site | P0 | 1 week |
| 31.4.2 | Integrate playground (play.bmb.dev) | P0 | 1 week |
| 31.4.3 | Set up package registry UI (gotgan.bmb.dev) | P1 | 1 week |
| 31.4.4 | Set up benchmark dashboard (bench.bmb.dev) | P1 | 1 week |

**Deliverables**:
- Live documentation website
- Integrated playground
- Package registry with search

---

### v0.32 Ecosystem - Package Ecosystem Growth

**Goal**: 100+ packages and active community

**Difficulty**: ⭐⭐⭐ (Medium - Ongoing effort)

**Duration Estimate**: 6-8 weeks

#### Phase 32.1: Core Package Development

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 32.1.1 | Develop bmb-json (JSON serialization) | P0 | 1 week |
| 32.1.2 | Develop bmb-http (HTTP client) | P0 | 2 weeks |
| 32.1.3 | Develop bmb-regex (Regular expressions) | P0 | 2 weeks |
| 32.1.4 | Develop bmb-crypto (Cryptography) | P1 | 2 weeks |

**Target Package Categories**:

| Category | Count | Key Packages |
|----------|-------|--------------|
| Core/Foundation | 20 | bmb-core, bmb-iter, bmb-hash, bmb-fmt |
| Collections | 15 | bmb-vec, bmb-hashmap, bmb-btreemap |
| IO/Filesystem | 10 | bmb-io, bmb-fs, bmb-path, bmb-tar |
| Networking | 15 | bmb-http, bmb-websocket, bmb-grpc |
| Serialization | 10 | bmb-serde, bmb-json, bmb-toml, bmb-yaml |
| Async | 10 | bmb-async, bmb-future, bmb-channel |
| Crypto/Security | 10 | bmb-crypto, bmb-sha, bmb-aes |
| Database | 10 | bmb-sql, bmb-postgres, bmb-redis |
| **Total** | **100+** | |

#### Phase 32.2: Rust Library Migration

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 32.2.1 | Port serde patterns to BMB | P0 | 2 weeks |
| 32.2.2 | Port regex patterns to BMB | P0 | 2 weeks |
| 32.2.3 | Port clap patterns to BMB | P1 | 1 week |

**Migration Principles**:
- API compatibility maintained (Rust user familiarity)
- Active use of BMB contract system (enhanced type safety)
- Performance parity or improvement goal

#### Phase 32.3: Community Building

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 32.3.1 | Set up contribution guidelines | P0 | 1 week |
| 32.3.2 | Create package submission process | P0 | 1 week |
| 32.3.3 | Establish quality standards | P1 | 1 week |
| 32.3.4 | Set up community forum/Discord | P2 | 1 week |

**Deliverables**:
- CONTRIBUTING.md with clear guidelines
- Package quality checklist
- Community communication channels

---

### v1.0.0-rc Golden - Release Candidate

**Goal**: Final verification and stability promise

**Difficulty**: ⭐⭐⭐⭐ (High - Quality gate)

**Duration Estimate**: 4-6 weeks

#### Phase 1.0.1: Stability Verification

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 1.0.1.1 | Run complete test suite | P0 | 1 week |
| 1.0.1.2 | Verify all benchmarks pass thresholds | P0 | 1 week |
| 1.0.1.3 | Security audit | P0 | 2 weeks |
| 1.0.1.4 | Performance regression testing | P0 | 1 week |

#### Phase 1.0.2: API Freeze

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 1.0.2.1 | Document public API stability guarantees | P0 | 1 week |
| 1.0.2.2 | Mark experimental features | P0 | 1 week |
| 1.0.2.3 | Create deprecation policy | P0 | 1 week |

#### Phase 1.0.3: Release Preparation

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| 1.0.3.1 | Write release notes | P0 | 1 week |
| 1.0.3.2 | Update all documentation | P0 | 1 week |
| 1.0.3.3 | Prepare binary distributions | P0 | 1 week |
| 1.0.3.4 | Set up release automation | P1 | 1 week |

**v1.0.0-rc Checklist**:

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Self-Hosting | ✅ 0 lines of Rust, BMB-only build | Pending |
| Performance | ✅ All benchmarks >= C -O3 | Pending |
| Documentation | ✅ Complete language reference + tutorials | Pending |
| Ecosystem | ✅ 100+ packages, active community | Pending |
| Tooling | ✅ fmt, lsp, test, lint, doc complete | Pending |
| Stability | ✅ No breaking changes after 1.0 | Promise |

---

## Ecosystem Repositories

| Repository | Purpose | Current Language | BMB Porting | Service |
|------------|---------|------------------|-------------|---------|
| **lang-bmb** | Main compiler | Rust | v0.30 ★ | - |
| **gotgan** | Package manager | Rust | v0.30 ★ | gotgan.bmb.dev |
| **gotgan-packages** | Additional packages | BMB | v0.26 ✅ | gotgan.bmb.dev |
| **action-bmb** | GitHub Action | YAML/Shell | Maintain | - |
| **bmb-samples** | Example programs | BMB | v0.26 ✅ | - |
| **benchmark-bmb** | Standard benchmarks | C/Rust/BMB | v0.28 ✅ | bench.bmb.dev |
| **playground** | Online playground | TypeScript | Maintain (WASM) | play.bmb.dev |
| **lang-bmb-site** | Official website | Astro/TS | Maintain | bmb.dev |
| **vscode-bmb** | VS Code extension | TypeScript | Maintain | Marketplace |
| **tree-sitter-bmb** | Grammar definition | JavaScript | Maintain | - |

★ = Self-Hosting target (Complete Rust code removal)

### Repository Classification

| Classification | Repositories | Reason |
|----------------|--------------|--------|
| **BMB Porting** | lang-bmb, gotgan | Self-Hosting required |
| **BMB Written** | gotgan-packages, bmb-samples | BMB code examples/libraries |
| **Maintain Current** | playground, lang-bmb-site | Web frontend (WASM integration) |
| **Maintain Current** | vscode-bmb, tree-sitter-bmb | Editor plugins (standard language) |
| **Maintain Current** | action-bmb | GitHub Action (YAML standard) |

---

## Benchmark Goals

> Reference: [Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/), [Is Rust C++-fast? (arXiv)](https://arxiv.org/abs/2209.09127)

| Metric | Target | Description |
|--------|--------|-------------|
| **Runtime Performance** | BMB >= C -O3 | Equal or better on all benchmarks |
| **Contract Optimization** | BMB > C -O3 | Contract-based optimization exceeds C |
| **Memory Usage** | BMB <= Rust | Equal or better than Rust |
| **Compile Speed** | BMB >= Rust | Equal or better than Rust |
| **Binary Size** | BMB <= Rust | Equal or better than Rust |

### Benchmark Categories

#### Category 1: Compute-Intensive (Benchmarks Game Standard)

| Benchmark | Description | Status |
|-----------|-------------|--------|
| fibonacci | Recursive function calls | ✅ Complete |
| n-body | N-body simulation (FP, SIMD) | 📋 Planned |
| mandelbrot | Fractal generation (parallel) | 📋 Planned |
| spectral-norm | Matrix operations | 📋 Planned |
| binary-trees | GC/Memory management | 📋 Planned |

#### Category 2: Contract-Optimized (BMB Unique)

| Benchmark | Contract Benefit | Expected Improvement |
|-----------|------------------|---------------------|
| bounds-check | `pre i < len(arr)` → bounds check elimination | 10-30% |
| null-check | `NonNull<T>` type → null check elimination | 5-15% |
| purity-opt | `pure` function → memoization/inlining | 20-50% |
| aliasing | Ownership-based → LLVM noalias hint | 10-25% |
| invariant-hoist | `invariant` → loop invariant extraction | 15-40% |

---

## Success Criteria

### v1.0.0-rc Release Requirements

```bash
# 1. No Rust code
$ git ls-files '*.rs' | wc -l
0

# 2. Self-hosting verification
$ bmb build --release
✓ Built bmb compiler (Stage 1)

$ ./target/release/bmb build --release
✓ Built bmb compiler (Stage 2)

$ ./stage2/bmb build --release
✓ Built bmb compiler (Stage 3)

$ diff stage2/bmb stage3/bmb
(no differences - binary identical)

# 3. Performance verification
$ bmb bench --all
✓ All benchmarks >= C -O3 threshold

# 4. Test suite
$ bmb test --all
✓ All tests passing (1000+ tests)

# 5. Documentation
$ bmb doc --check
✓ All public items documented
```

### Timeline Summary

```
2025 Q4 ─────────────────────────────────────────────────────
         v0.27 Registry ✅
         v0.28 Benchmark ✅
         v0.29 Velocity ✅

2026 Q1-Q2 ──────────────────────────────────────────────────
         v0.30 Pure (Self-Hosting Completion)
         - Bootstrap generics/traits/closures
         - Compiler/gotgan porting
         - Rust removal

2026 Q3 ─────────────────────────────────────────────────────
         v0.31 Docs (Documentation)
         - Language reference
         - API documentation
         - Tutorials and guides
         - Website launch

2026 Q4 ─────────────────────────────────────────────────────
         v0.32 Ecosystem (Package Ecosystem)
         - 100+ packages
         - Community building
         - Rust library migration

2027 Q1 ─────────────────────────────────────────────────────
         v1.0.0-rc Golden ★
         - Final verification
         - Stability promise
         - Official release
```

---

## Gap Analysis Reference

For detailed analysis of the remaining work, see [GAP_ANALYSIS.md](./GAP_ANALYSIS.md).

**Key Metrics (as of v0.30.140)**:
- Rust code to remove: ~21,783 LOC
- BMB bootstrap code: ~16,000 LOC
- Bootstrap tests: 1,732 tests across 14 files
- Gap to close: ~5,800 LOC additional BMB (reduced from ~7,900)
- Build mode: Use `--release` for bootstrap tests
- Note: lowering.bmb at stack limit (244 tests max)

---

**Last Updated**: 2026-01-06
**Version**: v0.30.140 → v1.0.0-rc Planning Document
