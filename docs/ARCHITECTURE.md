# BMB Compiler Architecture

This document describes the internal architecture of the BMB compiler.

## Compilation Pipeline

### Current Pipeline (v0.51)

```
Source (.bmb)
    │
    ▼
┌─────────┐
│  Lexer  │  logos-based tokenizer
└────┬────┘
     │ Token stream
     ▼
┌─────────┐
│ Parser  │  lalrpop LR(1) parser
└────┬────┘
     │ AST
     ▼
┌─────────┐
│  Types  │  Type inference and checking
└────┬────┘
     │ Typed AST
     ▼
┌─────────┐
│   SMT   │  Contract verification (Z3)
└────┬────┘
     │ Verified AST
     ▼
┌─────────┐
│   MIR   │  Middle Intermediate Representation
└────┬────┘
     │ MIR (with optimizations)
     ▼
┌─────────┐
│ CodeGen │  LLVM IR / WASM generation
└────┬────┘
     │
     ▼
  Native Binary
```

### Target Pipeline (v0.55+)

```
Source (.bmb)
    │
    ▼
┌─────────┐
│  Lexer  │  logos-based tokenizer
└────┬────┘
     │ Token stream
     ▼
┌─────────┐
│ Parser  │  lalrpop LR(1) parser
└────┬────┘
     │ AST
     ▼
┌─────────┐
│  Types  │  Type inference and checking
└────┬────┘
     │ Typed AST
     ▼
┌─────────┐
│   CIR   │  Contract IR - contracts as logical propositions
└────┬────┘   (bmb/src/cir/ - implemented, not integrated)
     │ CIR
     ▼
┌─────────┐
│ Verify  │  SMT verification + proof generation
└────┬────┘   (bmb/src/verify/ - ProofDatabase implemented)
     │ Verified CIR + ProofFacts
     ▼
┌─────────┐
│   PIR   │  Proof-Indexed IR - every expr carries proven facts
└────┬────┘   (bmb/src/pir/ - implemented, not integrated)
     │ PIR
     ▼
┌─────────┐
│   MIR   │  Middle IR with proof-guided optimization
└────┬────┘
     │ Optimized MIR
     ▼
┌─────────┐
│ CodeGen │  LLVM IR / WASM generation
└────┬────┘
     │
     ▼
  Native Binary
```

### Integration Status (v0.55)

| IR | Module | Status | Integration |
|----|--------|--------|-------------|
| AST | `bmb/src/ast/` | ✅ Complete | ✅ Integrated |
| CIR | `bmb/src/cir/` | ✅ Complete | ✅ **Integrated (v0.52)** |
| CIR→MIR Facts | `bmb/src/cir/to_mir_facts.rs` | ✅ Complete | ✅ **Integrated (v0.52)** |
| PIR | `bmb/src/pir/` | ✅ Complete | ✅ **Integrated (v0.55)** |
| PIR→MIR Facts | `bmb/src/pir/to_mir_facts.rs` | ✅ Complete | ✅ **Integrated (v0.55)** |
| MIR | `bmb/src/mir/` | ✅ Complete | ✅ Integrated |
| ProofDB | `bmb/src/verify/proof_db.rs` | ✅ Complete | ✅ **Integrated (v0.55)** |

**v0.55 Full Pipeline Integration**:

1. **CIR Integration (v0.52)**: Extracts contract propositions from AST
2. **PIR Integration (v0.55)**: Propagates proofs through control flow
   - Branch conditions (if/else)
   - Loop invariants (while/for)
   - Postconditions from function calls
3. **ProofDatabase (v0.55)**: Caches proofs for incremental compilation
4. **Fact Extraction**: Both CIR and PIR facts merged with MIR's `ContractFact`
5. **Proof-Guided Optimizations**: BCE, NCE, DCE, PUE use augmented facts

## Module Overview

### Lexer (`bmb/src/lexer/`)

Token generation using the `logos` crate with derive macros.

| File | Purpose |
|------|---------|
| `mod.rs` | Token enum definition with logos attributes |
| `tests.rs` | Token stream tests |

**Key tokens:**
- Keywords: `fn`, `let`, `var`, `if`, `then`, `else`, `match`, `pre`, `post`
- Types: `i32`, `i64`, `f64`, `bool`, `String`
- Operators: `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`
- Delimiters: `(`, `)`, `{`, `}`, `[`, `]`, `:`, `;`, `,`

### Parser (`bmb/src/parser/`)

LR(1) parser using `lalrpop` with grammar definition.

| File | Purpose |
|------|---------|
| `mod.rs` | Parser entry point and error handling |
| `grammar.lalrpop` | Complete grammar definition |
| `tests.rs` | 85+ parser tests covering all constructs |

**Grammar highlights:**
- Expression-based language (everything is an expression)
- Contract clauses: `pre`, `post`, `modifies`
- Generics: `<T>`, `<T: Trait>`
- Pattern matching: `match`, `is`
- Ownership: `own`, `&`, `&mut`

### AST (`bmb/src/ast/`)

Abstract Syntax Tree definitions with span information.

| File | Purpose |
|------|---------|
| `mod.rs` | Module exports |
| `types.rs` | Type AST nodes |
| `expr.rs` | Expression and statement nodes |
| `span.rs` | Source location tracking |
| `output.rs` | S-expression output formatter |

**Core types:**
```rust
pub struct Program {
    pub items: Vec<Item>,
}

pub enum Item {
    Function(FnDef),
    TypeDef(TypeDef),
    EnumDef(EnumDef),
    StructDef(StructDef),
    Use(UseStmt),
}

pub struct FnDef {
    pub name: Spanned<String>,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<Type>>,
    pub contracts: Vec<Contract>,
    pub body: Spanned<Expr>,
}
```

### Type System (`bmb/src/types/`)

Hindley-Milner type inference with contract-aware checking.

| File | Purpose |
|------|---------|
| `mod.rs` | Type checker entry point |
| `infer.rs` | Type inference algorithm |
| `unify.rs` | Type unification |
| `env.rs` | Type environment |
| `generics.rs` | Generic type handling |

**Features:**
- Bidirectional type inference
- Generic type instantiation
- Refinement type validation
- Contract type checking
- Option/Result method resolution

### SMT Verification (`bmb/src/smt/`)

SMT-LIB2 generation and Z3 integration.

| File | Purpose |
|------|---------|
| `mod.rs` | SMT generation entry point |
| `expr.rs` | Expression to SMT translation |
| `types.rs` | Type to SMT-LIB sort mapping |

**Verification modes:**
| Annotation | Behavior |
|------------|----------|
| (none) | Full SMT verification required |
| `@trust` | Skip verification (programmer guarantee) |
| `@check` | Runtime assertion on verification timeout |

### Verifier (`bmb/src/verify/`)

Contract verification orchestration with proof caching.

| File | Purpose |
|------|---------|
| `mod.rs` | Verification orchestration |
| `contract.rs` | ContractVerifier implementation |
| `proof_db.rs` | ProofDatabase for caching (v0.53) |
| `summary.rs` | FunctionSummary extraction |
| `incremental.rs` | IncrementalVerifier (v0.53) |

**ProofDatabase structure:**
```rust
pub struct ProofDatabase {
    function_proofs: HashMap<String, FunctionProofResult>,
    file_hashes: HashMap<String, u64>,  // For incremental compilation
    stats: ProofDbStats,
}

pub struct ProofFact {
    pub proposition: Proposition,
    pub scope: ProofScope,
    pub evidence: ProofEvidence,
}

pub enum ProofEvidence {
    SmtProof { query_hash, solver },
    Precondition,
    TypeInvariant(String),
    FunctionCall { callee, postcondition_index },
    ControlFlow,
}
```

### Interpreter (`bmb/src/interp/`)

Tree-walking interpreter for direct execution.

| File | Purpose |
|------|---------|
| `mod.rs` | Interpreter entry point |
| `eval.rs` | Expression evaluation |
| `env.rs` | Runtime environment |
| `value.rs` | Runtime value types |

### REPL (`bmb/src/repl/`)

Interactive Read-Eval-Print Loop using `rustyline`.

| File | Purpose |
|------|---------|
| `mod.rs` | REPL loop and commands |

**Commands:**
- `:help` - Show help
- `:type <expr>` - Show expression type
- `:quit` - Exit REPL

### CIR (`bmb/src/cir/`) - v0.52

Contract Intermediate Representation - contracts as first-class logical propositions.

| File | Purpose |
|------|---------|
| `mod.rs` | CIR types: CirProgram, CirFunction, Proposition |
| `lower.rs` | AST to CIR lowering |
| `output.rs` | CIR text output |
| `smt.rs` | CIR to SMT-LIB2 translation |
| `verify.rs` | CIR-based verification |
| `to_mir_facts.rs` | **v0.52**: CIR → MIR ContractFact conversion |

**CIR structure:**
```rust
pub struct CirProgram {
    pub functions: Vec<CirFunction>,
    pub structs: HashMap<String, CirStruct>,
    pub type_invariants: HashMap<String, Vec<Proposition>>,
}

pub enum Proposition {
    True, False,
    Compare { lhs, op, rhs },
    Not(Box<Proposition>),
    And(Vec<Proposition>),
    Or(Vec<Proposition>),
    InBounds { index, array },
    NonNull(Box<CirExpr>),
    // ... more variants
}
```

### PIR (`bmb/src/pir/`) - v0.55

Proof-Indexed IR - every expression carries proven facts.

| File | Purpose |
|------|---------|
| `mod.rs` | PIR types: PirProgram, PirExpr, ProvenFact |
| `propagate.rs` | Proof propagation through the program |
| `to_mir_facts.rs` | **v0.55**: PIR → MIR ContractFact extraction |
| `lower_to_mir.rs` | PIR to MIR lowering (stub - not used) |

**PIR structure:**
```rust
pub struct PirExpr {
    pub kind: PirExprKind,
    pub proven: Vec<ProvenFact>,       // Facts available at this point
    pub result_facts: Vec<ProvenFact>, // Facts about the result
    pub ty: PirType,
}

// Example: Index with bounds proof
PirExprKind::Index {
    array: Box<PirExpr>,
    index: Box<PirExpr>,
    bounds_proof: Option<ProvenFact>, // If Some, bounds check eliminated
}
```

### MIR (`bmb/src/mir/`)

Middle Intermediate Representation for optimization and codegen.

| File | Purpose |
|------|---------|
| `mod.rs` | MIR types and builder |
| `lower.rs` | AST to MIR lowering |
| `optimize.rs` | 15+ MIR optimization passes |

**MIR structure:**
```rust
pub struct MirFunction {
    pub name: String,
    pub params: Vec<MirParam>,
    pub return_type: MirType,
    pub blocks: Vec<BasicBlock>,
    pub is_memory_free: bool,  // For memory(none) attribute
    pub inline_hint: bool,     // For inlinehint attribute
}

pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<MirInstr>,
    pub terminator: Terminator,
}
```

**Optimization Passes:**

| Pass | Description | Status |
|------|-------------|--------|
| **LICM** | Loop Invariant Code Motion | ✅ v0.51.16 |
| **TCO** | Tail Call Optimization | ✅ v0.50.66 |
| **Contract-Based Opt** | Pre/post elimination | ✅ v0.50.76 |
| **Pure Function CSE** | Common subexpression elimination for `pure fn` | ✅ |
| **Constant Propagation** | Including narrowing (i64→i32) | ✅ v0.50.80 |
| **Semantic DCE** | Contract-based dead code elimination | 📋 CDO Phase |

**CDO (Contract-Driven Optimization) Pipeline** — [RFC-0001](rfcs/RFC-0008-contract-driven-optimization.md):

```
Typed AST
    │
    ▼
┌──────────────────┐
│  Contract IR     │  Contract intermediate representation
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌──────────────┐
│Semantic│ │  Contract    │
│  DCE   │ │Specialization│
└────┬───┘ └──────┬───────┘
     │            │
     └────┬───────┘
          ▼
    ┌─────────┐
    │   MIR   │  Optimized MIR
    └────┬────┘
         │
         ▼
┌─────────────────┐
│ Cross-Module    │  Link-time CDO
│ Optimization    │
└─────────────────┘
```

### Code Generation (`bmb/src/codegen/`)

Backend code generation.

| File | Purpose |
|------|---------|
| `mod.rs` | CodeGen trait and dispatch |
| `llvm.rs` | LLVM IR generation (inkwell) |
| `llvm_text.rs` | LLVM IR text output |
| `wasm.rs` | WASM generation |
| `wasm_text.rs` | WAT text output |

**LLVM integration:**
- Requires `llvm` feature flag
- Uses `inkwell` for LLVM bindings
- Supports optimization levels: O0, O1, O2, O3

**SIMD codegen path (v0.97):**
- `Type::Vector { elem, lanes }` → `MirType::Vector { elem, lanes }` via `ast_type_to_mir*`
- `llvm_text::mir_type_to_llvm_owned` emits `<lanes x elem>`; text backend is bootstrap-critical.
- `infer_place_mir_type` / `operand_vector_type` bypass the `&'static str` inference when operands are vectors — emit `fadd fast <N x T>` / `add <N x T>` / `sdiv <N x T>` directly.
- Vector locals: `alloca <N x T>, align (N*elem_size)` — natural alignment (`f64x4` → 32).
- Vector params/returns: `<N x T> noundef` by value (no nonnull/deref).
- `codegen/llvm.rs` (inkwell) has type mapping parity but BinOp emission pending full parity (Rule 7 follow-up).

**SIMD intrinsic dispatch (Cycles 2246-2256):**
- Name-based recognition in the `MirInst::Call` handler (like `sqrt` / `pow_f64`).
- Families: `hsum_*`, `splat_*`, `load_*`, `store_*`, `fma_*`, `min_*`, `max_*` — each emits a single LLVM vector intrinsic call or lowered insertelement/shufflevector sequence.
- `Copy`-for-Vector, Call-arg Vector passing, and Return `Constant::Unit` all use Vector-aware paths to preserve `<N x T>` type instead of collapsing to `"ptr"`.
- `fmt_f64_lit` helper centralizes NaN/Inf/finite LLVM-literal formatting — prevents `{:e}` → invalid `1e0` regression.
- Native build passes `-march=native` in Release for AVX2/FMA codegen parity with the inkwell path.

### LSP (`bmb/src/lsp/`)

Language Server Protocol implementation.

| File | Purpose |
|------|---------|
| `mod.rs` | LSP server implementation |

**Capabilities:**
- Diagnostics (errors, warnings)
- Hover information
- Go to definition
- Symbol outline

### Error Reporting (`bmb/src/error/`)

Rich error messages using `ariadne`.

| File | Purpose |
|------|---------|
| `mod.rs` | Error types and formatting |

**Error categories:**
- Lexer errors (invalid tokens)
- Parser errors (syntax errors)
- Type errors (type mismatches)
- Verification errors (contract violations)

## Build System (`bmb/src/build/`)

Build orchestration and caching.

| File | Purpose |
|------|---------|
| `mod.rs` | Build pipeline coordination |

## Data Flow

### Current Flow (v0.51)

```
1. Source file → Lexer → Token stream
2. Token stream → Parser → Untyped AST
3. Untyped AST → Type Checker → Typed AST
4. Typed AST → SMT Generator → SMT-LIB2
5. SMT-LIB2 → Z3 → Verification result
6. Typed AST → MIR Lowering → MIR
7. MIR → Optimizer → Optimized MIR
8. Optimized MIR → CodeGen → LLVM IR / WASM
9. LLVM IR → clang → Native binary
```

### Target Flow (v0.55+)

```
1. Source file → Lexer → Token stream
2. Token stream → Parser → Untyped AST
3. Untyped AST → Type Checker → Typed AST
4. Typed AST → CIR Lowering → CIR [cir/lower.rs - implemented]
5. CIR → SMT Generator → SMT-LIB2 [cir/smt.rs - implemented]
6. SMT-LIB2 → Z3 → ProofFacts [verify/proof_db.rs - implemented]
7. CIR + ProofFacts → PIR Lowering → PIR [pir/ - implemented]
8. PIR → Proof Propagation → PIR with facts [pir/propagate.rs - implemented]
9. PIR → MIR Lowering → MIR [pir/lower_to_mir.rs - implemented]
10. MIR → Contract-Based Optimizer → Optimized MIR [NOT IMPLEMENTED]
11. Optimized MIR → CodeGen → LLVM IR / WASM
12. LLVM IR → clang → Native binary
```

### Gap Analysis

Steps 4-10 have code implemented but are **not integrated** into the main pipeline.
The main pipeline still uses: AST → MIR → CodeGen (skipping CIR/PIR).

## Key Design Decisions

### Expression-Based Language

Everything is an expression, including control flow:

```bmb
let x = if condition then 1 else 2;
let y = match opt {
  Some(v) => v,
  None => 0
};
```

### Contract-First Verification

Contracts are integral to the type system:

```bmb
fn divide(a: i32, b: i32) -> i32
  pre b != 0
  post ret * b == a
= a / b;
```

### Ownership Model

Rust-inspired ownership with explicit annotations:

```bmb
fn consume(x: own String) { ... }
fn borrow(x: &String) { ... }
fn mutate(x: &mut String) { ... }
```

### Generic Type System

Full generics with trait bounds:

```bmb
fn max<T: Ord>(a: T, b: T) -> T
  post ret >= a and ret >= b
= if a > b then a else b;
```

## Testing

### Compiler Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test parser::tests
cargo test types::tests

# Run with verbose output
cargo test -- --nocapture
```

### BMB Test Framework (`ecosystem/bmb-test`)

The bmb-test framework provides advanced testing capabilities:

| Feature | Description |
|---------|-------------|
| **Property-Based Testing** | Generate thousands of inputs automatically |
| **Contract-Aware Generation** | Inputs respect preconditions |
| **Fuzz Testing** | Find edge cases through randomization |

```bmb
#[property]
fn sort_is_idempotent(arr: [i32; 100]) {
    assert(sort(sort(arr)) == sort(arr));
}

#[test]
fn test_binary_search() {
    let arr = [1, 3, 5, 7, 9];
    assert(binary_search(&arr, 5) == Some(2));
}
```

**Philosophy**: Tests define intent, AI implements. Contracts are the specification.

### Test Categories

| Category | Purpose | Location |
|----------|---------|----------|
| Unit Tests | Individual module testing | `bmb/src/*/tests.rs` |
| Integration Tests | Cross-module behavior | `bmb/tests/` |
| Bootstrap Tests | Self-hosted compiler | `bootstrap/*.bmb` |
| Benchmark Tests | Performance validation | `ecosystem/benchmark-bmb/` |
| Contract Tests | Verification coverage | `ecosystem/bmb-test/` |

## Ecosystem Integration

The compiler integrates with several ecosystem tools:

```
┌─────────────────────────────────────────────────────────────────┐
│                        BMB Ecosystem                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │  bmb-mcp    │    │  bmb-test   │    │  bmb-query  │         │
│  │  (Chatter)  │    │  (Testing)  │    │  (Query)    │         │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘         │
│         │                  │                  │                 │
│         └──────────────────┼──────────────────┘                 │
│                            │                                    │
│                     ┌──────▼──────┐                            │
│                     │ BMB Compiler │                            │
│                     │   (bmb/)     │                            │
│                     └──────┬──────┘                            │
│                            │                                    │
│              ┌─────────────┼─────────────┐                     │
│              │             │             │                     │
│       ┌──────▼──────┐ ┌────▼────┐ ┌─────▼─────┐               │
│       │   gotgan    │ │ vscode  │ │ benchmark │               │
│       │ (packages)  │ │  -bmb   │ │   -bmb    │               │
│       └─────────────┘ └─────────┘ └───────────┘               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

| Tool | Compiler Integration |
|------|---------------------|
| **bmb-mcp** | Invokes `bmb check`, `bmb verify` for AI feedback |
| **bmb-test** | Uses compiler's contract system for test generation |
| **bmb-query** | Parses contracts for natural language queries |
| **gotgan** | Orchestrates builds, CDO-aware dependency resolution |
| **vscode-bmb** | LSP integration via `bmb lsp` |

## Performance Considerations

1. **Parallel Verification**: Contract verification can be parallelized per function
2. **Incremental Compilation**: MIR caching for unchanged functions
3. **Lazy Type Inference**: Type inference deferred until needed
4. **SMT Caching**: Verification results cached for unchanged contracts
5. **CDO Caching**: Contract analysis results cached for unchanged modules
