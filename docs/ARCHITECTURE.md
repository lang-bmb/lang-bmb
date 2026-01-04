# BMB Compiler Architecture

This document describes the internal architecture of the BMB compiler.

## Compilation Pipeline

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
     │ MIR
     ▼
┌─────────┐
│ CodeGen │  LLVM IR / WASM generation
└────┬────┘
     │
     ▼
  Native Binary
```

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

Contract verification orchestration.

| File | Purpose |
|------|---------|
| `mod.rs` | Verification orchestration |
| `z3.rs` | Z3 process communication |
| `counterexample.rs` | Counterexample parsing |

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

### MIR (`bmb/src/mir/`)

Middle Intermediate Representation for optimization and codegen.

| File | Purpose |
|------|---------|
| `mod.rs` | MIR types and builder |
| `lower.rs` | AST to MIR lowering |

**MIR structure:**
```rust
pub struct MirFunction {
    pub name: String,
    pub params: Vec<MirParam>,
    pub return_type: MirType,
    pub blocks: Vec<BasicBlock>,
}

pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<MirInstr>,
    pub terminator: Terminator,
}
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

```
1. Source file → Lexer → Token stream
2. Token stream → Parser → Untyped AST
3. Untyped AST → Type Checker → Typed AST
4. Typed AST → SMT Generator → SMT-LIB2
5. SMT-LIB2 → Z3 → Verification result
6. Typed AST → MIR Lowering → MIR
7. MIR → CodeGen → LLVM IR / WASM
8. LLVM IR → LLVM → Native binary
```

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

```bash
# Run all tests
cargo test

# Run specific test module
cargo test parser::tests
cargo test types::tests

# Run with verbose output
cargo test -- --nocapture
```

## Performance Considerations

1. **Parallel Verification**: Contract verification can be parallelized per function
2. **Incremental Compilation**: MIR caching for unchanged functions
3. **Lazy Type Inference**: Type inference deferred until needed
4. **SMT Caching**: Verification results cached for unchanged contracts
