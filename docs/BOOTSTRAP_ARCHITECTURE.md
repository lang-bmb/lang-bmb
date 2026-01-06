# Bootstrap Architecture Documentation

> Version: v0.30.206
> Purpose: Document the BMB self-hosting compiler architecture
> Related: [Feature Gap Analysis](BOOTSTRAP_FEATURE_GAP.md) | [ROADMAP](ROADMAP.md)

## Overview

The BMB bootstrap implements a complete compiler pipeline written in BMB itself, enabling self-hosting compilation. This document describes the architecture, data flow, and encoding schemes used throughout the bootstrap components.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BMB Bootstrap Compiler                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌───────────┐     ┌───────────┐     ┌───────────┐     ┌───────────┐       │
│  │           │     │           │     │           │     │           │       │
│  │  Source   │────▶│   Lexer   │────▶│  Parser   │────▶│    AST    │       │
│  │  (.bmb)   │     │           │     │           │     │           │       │
│  │           │     │ lexer.bmb │     │parser.bmb │     │parser_ast │       │
│  └───────────┘     └───────────┘     └───────────┘     └─────┬─────┘       │
│                                                              │              │
│                                                              ▼              │
│  ┌───────────┐     ┌───────────┐     ┌───────────┐     ┌───────────┐       │
│  │           │     │           │     │           │     │           │       │
│  │  Output   │◀────│  LLVM IR  │◀────│    MIR    │◀────│   Types   │       │
│  │   (.ll)   │     │           │     │           │     │           │       │
│  │           │     │llvm_ir.bmb│     │  mir.bmb  │     │ types.bmb │       │
│  └───────────┘     └───────────┘     └───────────┘     └───────────┘       │
│                           ▲                │                                │
│                           │                ▼                                │
│                    ┌───────────┐     ┌───────────┐                         │
│                    │           │     │           │                         │
│                    │ Optimize  │◀────│ Lowering  │                         │
│                    │           │     │           │                         │
│                    │optimize.bmb│    │lowering.bmb│                        │
│                    └───────────┘     └───────────┘                         │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         Coordination Layer                             │  │
│  │  ┌────────────────┐  ┌────────────────┐  ┌─────────────────────────┐  │  │
│  │  │  pipeline.bmb  │  │  compiler.bmb  │  │  selfhost_test/equiv.bmb│  │  │
│  │  │ (E2E pipeline) │  │ (orchestration)│  │ (verification)          │  │  │
│  │  └────────────────┘  └────────────────┘  └─────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                          Utilities Layer                               │  │
│  │  ┌────────────────┐  ┌──────────────────────────────────────────────┐  │  │
│  │  │   utils.bmb    │  │  parser_test.bmb (parser validation)         │  │  │
│  │  │ (string utils) │  └──────────────────────────────────────────────┘  │  │
│  │  └────────────────┘                                                    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Descriptions

### 1. Lexer (`lexer.bmb`)

**Purpose**: Tokenize BMB source code into a stream of tokens.

**Key Functions**:
```bmb
fn is_keyword(word: String) -> bool      -- Check if word is a reserved keyword
fn lookup_keyword(word: String) -> i64   -- Get keyword token ID
fn char_to_digit(c: i64) -> i64          -- Convert digit char to value
fn token_kind(tok: i64) -> i64           -- Extract token kind from encoding
fn token_value(tok: i64) -> i64          -- Extract token value from encoding
```

**Token Encoding** (single i64):
```
token = kind * 1000 + value
  kind: 1=INT, 2=IDENT, 3=STRING, 4=OP, 5=PUNCT, 6=KEYWORD, etc.
  value: numeric or index reference
```

**Keywords Supported**:
`fn`, `let`, `if`, `then`, `else`, `match`, `for`, `in`, `struct`, `enum`, `type`, `trait`, `impl`, `where`, `pre`, `post`, `true`, `false`, `and`, `or`, `not`, `own`, `ref`, `mut`, `move`, `copy`, `drop`, `linear`, `forall`, `exists`, `old`, `rec`

**Test Coverage**: 40 test functions

---

### 2. Parser (`parser.bmb`, `parser_ast.bmb`)

**Purpose**: Transform token stream into Abstract Syntax Tree.

**Parser Core** (`parser.bmb`):
```bmb
fn parse_expr(src: String) -> String     -- Parse expression to S-expr
fn parse_stmt(src: String) -> String     -- Parse statement to S-expr
fn parse_fn(src: String) -> String       -- Parse function definition
fn parse_program(src: String) -> String  -- Parse complete program
```

**AST Builder** (`parser_ast.bmb`):
```bmb
fn ast_int(value: i64) -> String         -- "(int VALUE)"
fn ast_var(name: String) -> String       -- "(var <NAME>)"
fn ast_binop(op: String, left: String, right: String) -> String
fn ast_if(cond: String, then_: String, else_: String) -> String
fn ast_let(name: String, value: String, body: String) -> String
fn ast_fn(name: String, params: String, ret_ty: String, body: String) -> String
```

**AST Format** (S-expression):
```
(program
  (fn <name> (params (p <x> i64) (p <y> i64)) (ret i64)
    (binop + (var <x>) (var <y>))))
```

**Test Coverage**: 216 test functions (parser: 43, parser_ast: 119, parser_test: 54)

---

### 3. Type Checker (`types.bmb`)

**Purpose**: Validate type correctness and infer types.

**Type Encoding** (single i64):
```
type = kind * 1000 + extra_info
  kind: 1=i32, 2=i64, 3=f64, 4=bool, 5=String, 6=Unit, 7=Named, 9=Error, 10=TypeParam, 11=GenericApp
  extra_info: varies by kind (name_id, param_index, base_hash)
```

**Key Functions**:
```bmb
fn type_i64() -> i64                     -- Get i64 type encoding
fn type_bool() -> i64                    -- Get bool type encoding
fn type_param(idx: i64) -> i64           -- Get type parameter encoding
fn type_generic_app(hash: i64) -> i64    -- Get generic application encoding
fn type_kind(ty: i64) -> i64             -- Extract kind from encoding
fn is_type_param(ty: i64) -> bool        -- Check if type parameter
fn check_binop(op: String, left_ty: i64, right_ty: i64) -> i64  -- Check binary op
fn unify(expected: i64, actual: i64) -> bool  -- Type unification
```

**Environment**:
- Fixed-size arrays for name-type pairs (max 64 variables)
- Function registry (max 32 functions)
- Struct registry with field tracking
- Type parameter scope tracking

**Generics Support** (v0.30.3-v0.30.12):
- Type parameter declaration and scope
- Generic type application encoding (Vec<T>, Option<T>)
- Type argument tracking (string-based)
- Basic type substitution

**Test Coverage**: 167 test functions

---

### 4. MIR Representation (`mir.bmb`)

**Purpose**: Define Mid-level Intermediate Representation.

**Instruction Encoding** (single i64):
```
instruction = kind * 1000 + extra_info
  kind: 1=CONST, 2=COPY, 3=BINOP, 4=UNARY, 5=CALL
```

**Terminator Encoding**:
```
terminator = kind * 1000
  kind: 10=RETURN, 11=GOTO, 12=BRANCH
```

**Binary Operator Encoding**:
```
1=Add, 2=Sub, 3=Mul, 4=Div, 5=Mod, 6=Eq, 7=Ne, 8=Lt, 9=Gt, 10=Le, 11=Ge, 12=And, 13=Or
```

**Key Functions**:
```bmb
fn mir_const() -> i64                    -- CONST instruction kind
fn mir_binop() -> i64                    -- BINOP instruction kind
fn mir_return() -> i64                   -- RETURN terminator kind
fn binop_add() -> i64                    -- Add operator encoding
fn mir_inst_kind(inst: i64) -> i64       -- Extract instruction kind
fn is_terminator(kind: i64) -> bool      -- Check if terminator
```

**MIR Text Format**:
```
fn @add(%a: i64, %b: i64) -> i64 {
bb0:
  %_t0 = const I:5
  %_t1 = add i64 %a, %b
  ret %_t1
}
```

**Test Coverage**: 59 test functions

---

### 5. Lowering (`lowering.bmb`)

**Purpose**: Transform AST to MIR.

**Key Functions**:
```bmb
fn lower_expr(ast: String, dest: String) -> String  -- Lower expression
fn lower_binop(op: String, left: String, right: String, dest: String) -> String
fn lower_if(cond: String, then_: String, else_: String, dest: String) -> String
fn lower_let(name: String, value: String, body: String, dest: String) -> String
fn lower_fn(ast: String) -> String       -- Lower function definition
```

**Closure Support** (v0.30.34):
```bmb
fn gen_closure_with_env(closure: String, env: String, fn_id: i64) -> String
fn gen_closure_fn_header(fn_id: i64, params: String) -> String
fn gen_closure_prelude(free_vars: String, count: i64, idx: i64) -> String
fn gen_load_capture(dest: String, closure: String, idx: i64) -> String
```

**Test Coverage**: 4 test functions (limited by stack overflow)

---

### 6. Optimizer (`optimize.bmb`)

**Purpose**: Optimize MIR for better code generation.

**Optimizations**:
- Constant folding
- Dead code elimination
- Copy propagation
- Basic block merging

**Key Functions**:
```bmb
fn optimize_mir(mir: String) -> String   -- Apply all optimizations
fn fold_constants(mir: String) -> String -- Constant folding pass
fn eliminate_dead_code(mir: String) -> String  -- DCE pass
fn propagate_copies(mir: String) -> String     -- Copy propagation
```

**Test Coverage**: 56 test functions

---

### 7. LLVM IR Generator (`llvm_ir.bmb`)

**Purpose**: Generate LLVM IR text from MIR.

**Key Functions**:
```bmb
fn gen_llvm_module(mir: String) -> String    -- Generate complete module
fn gen_llvm_fn(mir_fn: String) -> String     -- Generate function
fn gen_llvm_block(block: String) -> String   -- Generate basic block
fn gen_binop_arith(dest: String, op: String, left: String, right: String, ty: String) -> String
fn gen_binop_cmp(dest: String, op: String, left: String, right: String, ty: String) -> String
fn gen_binop_logic(dest: String, op: String, left: String, right: String, ty: String) -> String
fn gen_unary_not(dest: String, operand: String) -> String
fn gen_call(dest: String, fn_name: String, args: String, ret_ty: String) -> String
```

**LLVM IR Format**:
```llvm
define i64 @add(i64 %a, i64 %b) {
entry:
  %_t0 = add i64 %a, %b
  ret i64 %_t0
}
```

**Supported Operations**:
- Arithmetic: add, sub, mul, sdiv, srem
- Comparison: icmp eq/ne/slt/sgt/sle/sge
- Logical: and, or
- Control: br, ret, call

**Test Coverage**: 80 test functions

---

### 8. Pipeline Coordination (`pipeline.bmb`, `compiler.bmb`)

**Purpose**: Orchestrate complete compilation pipeline.

**Pipeline** (`pipeline.bmb`):
```bmb
fn compile(src: String) -> String        -- Source → LLVM IR
fn parse_and_lower(src: String) -> String  -- Source → MIR
fn test_pipeline(src: String, expected_pattern: String) -> i64
fn test_parse(src: String, expected_pattern: String) -> i64
```

**Compiler** (`compiler.bmb`):
```bmb
fn compile_program(src: String) -> String   -- Complete compilation
fn compile_function(src: String) -> String  -- Single function
```

**Test Coverage**: 111 test functions (pipeline: 48, compiler: 63)

---

### 9. Self-Hosting Verification (`selfhost_test.bmb`, `selfhost_equiv.bmb`)

**Purpose**: Verify bootstrap produces correct output equivalent to Rust compiler.

**Test Functions** (`selfhost_test.bmb`):
```bmb
fn test_selfhost_arithmetic() -> i64     -- Verify arithmetic compilation
fn test_selfhost_comparison() -> i64     -- Verify comparison ops
fn test_selfhost_logical() -> i64        -- Verify logical ops
fn test_selfhost_control_flow() -> i64   -- Verify if/match
```

**Equivalence Testing** (`selfhost_equiv.bmb`):
```bmb
fn equiv_test(bmb_output: String, rust_output: String) -> bool
fn normalize_llvm(llvm: String) -> String   -- Normalize for comparison
```

**Test Coverage**: 95 test functions (selfhost_test: 62, selfhost_equiv: 33)

---

### 10. Utilities (`utils.bmb`)

**Purpose**: Shared string and numeric utilities.

**Key Functions**:
```bmb
fn int_to_string(n: i64) -> String       -- Convert int to string
fn digit_char(d: i64) -> String          -- Single digit to char
fn str_contains(haystack: String, needle: String) -> bool  -- Substring search
fn str_split(s: String, delim: String) -> String  -- Split string
fn str_trim(s: String) -> String         -- Trim whitespace
fn char_at(s: String, idx: i64) -> i64   -- Get char code at index
fn str_len(s: String) -> i64             -- String length
```

**Test Coverage**: 74 test functions

---

## Data Flow

### Complete Compilation Pipeline

```
1. Source Code (.bmb)
   │
   ▼
2. Lexer (lexer.bmb)
   │ Token stream (i64-encoded tokens)
   ▼
3. Parser (parser.bmb + parser_ast.bmb)
   │ S-expression AST (String)
   ▼
4. Type Checker (types.bmb)
   │ Typed AST (verified types, i64-encoded)
   ▼
5. Lowering (lowering.bmb)
   │ MIR text representation
   ▼
6. Optimizer (optimize.bmb)
   │ Optimized MIR
   ▼
7. LLVM IR Generator (llvm_ir.bmb)
   │ LLVM IR text (.ll)
   ▼
8. LLVM/Clang (external)
   │ Native executable
   ▼
9. Execution
```

### Example: `fn add(a: i64, b: i64) -> i64 = a + b;`

**Source**:
```bmb
fn add(a: i64, b: i64) -> i64 = a + b;
```

**Lexer Output** (token stream):
```
[FN, IDENT(add), LPAREN, IDENT(a), COLON, IDENT(i64), COMMA,
 IDENT(b), COLON, IDENT(i64), RPAREN, ARROW, IDENT(i64), EQ,
 IDENT(a), PLUS, IDENT(b), SEMI]
```

**Parser Output** (S-expression):
```
(fn <add> (params (p <a> i64) (p <b> i64)) (ret i64)
  (binop + (var <a>) (var <b>)))
```

**Type Checker**:
- Verify `a` and `b` are i64
- Verify `+` operator accepts i64 operands
- Verify return type matches i64

**MIR Output**:
```
fn @add(%a: i64, %b: i64) -> i64 {
bb0:
  %_t0 = add i64 %a, %b
  ret %_t0
}
```

**LLVM IR Output**:
```llvm
define i64 @add(i64 %a, i64 %b) {
entry:
  %_t0 = add i64 %a, %b
  ret i64 %_t0
}
```

---

## Encoding Schemes

### Type Encoding (types.bmb)

| Kind | Value | Description | Extra Info |
|------|-------|-------------|------------|
| 1 | 1000 | i32 | - |
| 2 | 2000 | i64 | - |
| 3 | 3000 | f64 | - |
| 4 | 4000 | bool | - |
| 5 | 5000 | String | - |
| 6 | 6000 | Unit | - |
| 7 | 7000+ | Named | name_id (0-999) |
| 9 | 9000 | Error | - |
| 10 | 10000+ | TypeParam | param_idx (0-999) |
| 11 | 11000+ | GenericApp | base_hash (0-999) |

### Token Encoding (lexer.bmb)

| Kind | Value Range | Description |
|------|-------------|-------------|
| 1 | 1000+ | Integer literal |
| 2 | 2000+ | Identifier |
| 3 | 3000+ | String literal |
| 4 | 4000+ | Operator |
| 5 | 5000+ | Punctuation |
| 6 | 6000+ | Keyword |

### MIR Instruction Encoding (mir.bmb)

| Kind | Value | Description |
|------|-------|-------------|
| 1 | 1000 | CONST (load constant) |
| 2 | 2000 | COPY (copy value) |
| 3 | 3000 | BINOP (binary operation) |
| 4 | 4000 | UNARY (unary operation) |
| 5 | 5000 | CALL (function call) |
| 10 | 10000 | RETURN (terminator) |
| 11 | 11000 | GOTO (terminator) |
| 12 | 12000 | BRANCH (terminator) |

---

## Design Rationale

### Why Single i64 Encoding?

1. **No Structs/Enums**: Bootstrap uses only primitive types available in BMB core
2. **Efficient Storage**: Compact representation for all metadata
3. **Simple Operations**: Arithmetic operations for encoding/decoding
4. **Recursive Pattern**: Same pattern used across lexer, parser, types, MIR

### Why String-Based AST?

1. **S-expression Format**: Easy to parse, generate, and manipulate
2. **No Custom Data Structures**: Avoids complex memory management
3. **Human Readable**: Debugging and testing friendly
4. **Pattern Matching**: `str_contains` enables simple pattern validation

### Why Text-Based MIR/LLVM?

1. **No Binary Format**: Simplifies generation and verification
2. **Direct LLVM Input**: Output can be directly passed to `llc`
3. **Debugging**: Human-readable intermediate stages
4. **Equivalence Testing**: Text comparison with Rust compiler output

---

## Test Organization

| File | Purpose | Tests |
|------|---------|-------|
| `*_test.bmb` | Unit tests for component | Validates individual functions |
| `selfhost_*.bmb` | Integration tests | Validates complete pipeline |
| Functions named `test_*` | Test functions | Return count of passed assertions |

**Test Pattern**:
```bmb
fn test_example() -> i64 =
    let t1 = if condition1 then 1 else 0;
    let t2 = if condition2 then 1 else 0;
    let t3 = if condition3 then 1 else 0;
    t1 + t2 + t3;  -- Returns count of passed tests
```

**Running Tests**:
```bash
bmb run bootstrap/types.bmb  # Returns total test count (167)
bmb run bootstrap/lexer.bmb  # Returns total test count (40)
```

---

## Future Architecture Considerations

### Trait System Integration

When trait support is added (30.1.2):
```
types.bmb additions:
- trait_info encoding (kind=12?)
- impl registry
- lookup_trait_method function
- method dispatch logic
```

### Interpreter Integration

When interpreter is added (30.1.4):
```
New file: interp.bmb
- Value encoding (similar to type encoding)
- Expression evaluator
- Environment management
- Built-in function table
```

### Stage 3 Bootstrap

For complete self-hosting:
```
Bootstrap compiles itself:
1. Rust compiler builds bootstrap (Stage 1)
2. Bootstrap compiles bootstrap (Stage 2)
3. Stage 2 output compiles bootstrap (Stage 3)
4. Stage 2 output == Stage 3 output (verification)
```

---

## Conclusion

The BMB bootstrap architecture follows a consistent design philosophy:
- **Single i64 encoding** for all metadata
- **String-based representations** for AST and intermediate formats
- **Functional/recursive style** matching BMB idioms
- **Comprehensive testing** with 902 test functions

This architecture enables self-hosting while maintaining simplicity and debuggability.
