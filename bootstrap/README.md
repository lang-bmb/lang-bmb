# BMB Bootstrap System

Self-hosting components for the BMB language, written in BMB itself.

## Philosophy

Following the BMB LAWS principle of 부트스트랩 (self-compilation):
- **자기 작성**: The compiler is written in the language it compiles
- **자기 검증**: The bootstrap validates the language semantics
- **자기 학습**: AI-driven improvement through self-reflection

## Files

### lexer.bmb (8KB)
Core lexical analyzer that tokenizes BMB source code.

**Features:**
- Token encoding: `kind * 1000000 + end_position`
- Comment skipping (`--` style)
- All BMB keywords and operators
- Unicode-safe identifier handling

**Test output:**
```
777 (start marker)
<token kinds for each token>
888 (separator)
<token count>
999 (end marker)
```

### parser.bmb (22KB)
Recursive descent parser that validates BMB syntax.

**Features:**
- Full BMB grammar support
- Expression parsing with operator precedence
- Function definition parsing
- Let binding and if-then-else
- Contract clause handling (pre/post)

**Test output:**
```
777 (start marker)
<1 for each successful parse>
888 (separator)
<total passed>
999 (end marker)
```

### parser_ast.bmb (21KB)
Parser that produces S-expression AST representation.

**AST Format:**
```lisp
(program
  (fn <name> (params (p <param> type)...) return-type body))

; Examples:
(fn <add> (params (p <x> i64) (p <y> i64)) i64 (op + (var <x>) (var <y>)))
(if (condition) (then-expr) (else-expr))
(let <name> (value) (body))
(call <name> (arg1) (arg2)...)
```

**Design decisions:**
- Angle brackets `<name>` instead of quotes (BMB string limitation)
- Result packing: `"pos:ast"` format for position+AST returns
- Error format: `"ERR:message"`

### parser_test.bmb (25KB)
Comprehensive test suite with 15 test categories.

**Test coverage:**
1. Multiple functions in program
2. Nested if expressions
3. Complex operator chains
4. All comparison operators
5. Let binding chains
6. Mutable let bindings
7. Multi-argument function calls
8. Boolean expressions (and/or/not)
9. Parenthesized expressions
10. Negation operations
11. Mixed types (i32, i64, bool)
12. Empty parameter lists
13. Range operator (..)
14. Deep nesting
15. Nested function calls

### types.bmb (15KB) - v0.10.0
Type checker foundation for BMB.

**Features:**
- Type encoding: `kind * 1000` (i64=2000, bool=4000, etc.)
- Environment: String-based name:type pairs with linear lookup
- Built-in function signatures (println, abs, min, max, etc.)
- Binary operator type checking (+, -, *, /, %, ==, <, etc.)
- Unary operator type checking (-, not)
- If-then-else type checking (condition bool, branches match)
- Let binding type checking
- Function call type checking (arity and arg types)

**Test output:**
```
777 (start marker)
5  (type encoding tests)
5  (binary operator tests)
4  (unary operator tests)
3  (environment tests)
5  (builtin lookup tests)
4  (if-then-else tests)
3  (let binding tests)
8  (function call tests)
888 (separator)
37 (total passed)
999 (end marker)
```

### mir.bmb (18KB) - v0.10.1
Middle IR (MIR) foundation for code generation.

**Features:**
- Instruction encoding: `kind * 1000` (CONST=1000, COPY=2000, BINOP=3000, etc.)
- Terminator encoding: (RETURN=10000, GOTO=11000, BRANCH=12000)
- Binary/unary operator encoding with symbol output
- Constant encoding: `I:42`, `B:1`, `S:hello`, `U`
- Place (variable) encoding: `%name`, `%_t0` (temporaries)
- Text-based MIR output format
- Example lowering functions (add, max with if)

**MIR Text Format:**
```
fn add(a: i64, b: i64) -> i64 {
entry:
  %_t0 = + %a, %b
  return %_t0
}
```

**Test output:**
```
777 (start marker)
5  (instruction encoding tests)
5  (terminator encoding tests)
5  (binop symbol tests)
7  (constant encoding tests)
6  (place encoding tests)
5  (mir text instruction tests)
4  (mir text terminator tests)
4  (type name tests)
3  (result packing tests)
2  (example function tests)
888 (separator)
46 (total passed)
999 (end marker)
```

### lowering.bmb (25KB) - v0.10.2
AST to MIR lowering (transformation) module.

**Features:**
- S-expression AST parsing (from parser_ast.bmb output)
- Expression lowering: int, bool, var, binop, unary, if, let, call
- Function lowering with basic block generation
- Program lowering (multiple functions)
- Pack/unpack result format: `temp:block:place:text`

**Supported Transformations:**
```lisp
; AST → MIR examples
(int 42)              →  %_t0 = const I:42
(var <x>)             →  %x (no instruction, just reference)
(op + (var <a>) (var <b>)) →  %_t0 = + %a, %b
(if (var <c>) (int 1) (int 2)) →  branch %c, then_0, else_0 ...
(let <x> (int 5) (var <x>))   →  %_t0 = const I:5 | %x = copy %_t0
(call <foo> (var <a>))        →  %_t0 = call foo(%a)
```

**Test output:**
```
777 (start marker)
6  (node type detection)
5  (value extraction)
5  (child extraction)
3  (integer lowering)
2  (boolean lowering)
2  (variable lowering)
3  (binop lowering)
2  (unary lowering)
3  (if lowering)
2  (let lowering)
3  (call lowering)
3  (function lowering)
2  (program lowering)
888 (separator)
41 (total passed)
999 (end marker)
```

### pipeline.bmb (25KB) - v0.10.3
End-to-end compilation pipeline demonstrating Source → AST → MIR.

**Features:**
- Integrated parsing and lowering from single source
- S-expression AST generation (from parser_ast.bmb patterns)
- MIR text generation (from lowering.bmb patterns)
- Expression-level compilation: `compile_expr(src) -> MIR text`
- Full pipeline test suite

**Architecture:**
```
Source (BMB) → Lexer (Tokens) → Parser (S-expr AST) → Lowering (MIR Text)
```

**Compilation Examples:**
```bmb
-- Integer literal
compile_expr("42")      →  "%_t0 = const I:42"

-- Binary operation
compile_expr("a + b")   →  "%_t0 = + %a, %b"

-- Nested operations
compile_expr("a * b + c")  →  "%_t0 = * %a, %b|%_t1 = + %_t0, %c"

-- Unary operations
compile_expr("-x")      →  "%_t0 = neg %x"
compile_expr("not b")   →  "%_t0 = not %b"
```

**Test output:**
```
777 (start marker)
5  (parsing tests)
5  (expression pipeline tests)
4  (complex expression tests)
888 (separator)
14 (total passed)
999 (end marker)
```

### codegen.bmb (18KB) - v0.10.4
MIR to C code generation module.

**Features:**
- Type mapping: i64 → int64_t, bool → int, unit → void
- MIR instruction parsing and C statement generation
- Binary operators: +, -, *, /, %, ==, !=, <, >, <=, >=, and, or
- Unary operators: neg (negation), not (logical not)
- Control flow: return, goto, branch (conditional)
- Label generation for basic blocks
- Function generation with signature and body
- Parameter conversion: "a: i64, b: i64" → "int64_t a, int64_t b"

**C Code Generation:**
```c
// MIR → C examples
%_t0 = const I:42      →  int64_t _t0 = 42;
%_t0 = + %a, %b        →  int64_t _t0 = a + b;
%_t0 = == %x, %y       →  int64_t _t0 = x == y;
%_t0 = neg %x          →  int64_t _t0 = -x;
%_t0 = not %b          →  int _t0 = !b;
return %_t0            →  return _t0;
goto entry             →  goto entry;
branch %c, t, e        →  if (c) goto t; else goto e;
entry:                 →  entry:;
```

**Function Generation:**
```c
// Input: name="add", params="a: i64, b: i64", ret="i64"
// Body: "entry:|%_t0 = + %a, %b|return %_t0"
// Output:
int64_t add(int64_t a, int64_t b) {
entry:;
    int64_t _t0 = a + b;
    return _t0;
}
```

**Test output:**
```
777 (start marker)
3  (constant generation tests)
4  (binary operation tests)
2  (unary operation tests)
3  (control flow tests)
1  (branch tests)
1  (label tests)
2  (multi-line tests)
2  (function generation tests)
888 (separator)
18 (total passed)
999 (end marker)
```

## Token Encoding

Tokens are encoded as a single i64 value:
```
encoded = kind * 1000000 + end_position
```

Decoding:
```bmb
fn tok_kind(tok: i64) -> i64 = tok / 1000000;
fn tok_end(tok: i64) -> i64 = tok - (tok / 1000000) * 1000000;
```

This allows passing both token type and position in a single return value.

## Result Packing

Since BMB functions return single values, we pack multiple results:
```bmb
fn pack_result(pos: i64, ast: String) -> String =
    int_to_string(pos) + ":" + ast;

fn unpack_pos(result: String) -> i64 =
    parse_int_prefix(result, 0, 0);

fn unpack_ast(result: String) -> String =
    result.slice(find_colon(result, 0) + 1, result.len());
```

## Running Tests

```bash
# Check syntax
cargo run --release --bin bmb -- check bootstrap/lexer.bmb
cargo run --release --bin bmb -- check bootstrap/parser.bmb
cargo run --release --bin bmb -- check bootstrap/parser_ast.bmb
cargo run --release --bin bmb -- check bootstrap/parser_test.bmb
cargo run --release --bin bmb -- check bootstrap/types.bmb
cargo run --release --bin bmb -- check bootstrap/mir.bmb
cargo run --release --bin bmb -- check bootstrap/lowering.bmb
cargo run --release --bin bmb -- check bootstrap/pipeline.bmb
cargo run --release --bin bmb -- check bootstrap/codegen.bmb

# Run tests
cargo run --release --bin bmb -- run bootstrap/lexer.bmb
cargo run --release --bin bmb -- run bootstrap/parser.bmb
cargo run --release --bin bmb -- run bootstrap/parser_ast.bmb
cargo run --release --bin bmb -- run bootstrap/parser_test.bmb
cargo run --release --bin bmb -- run bootstrap/types.bmb
cargo run --release --bin bmb -- run bootstrap/mir.bmb
cargo run --release --bin bmb -- run bootstrap/lowering.bmb
cargo run --release --bin bmb -- run bootstrap/pipeline.bmb
cargo run --release --bin bmb -- run bootstrap/codegen.bmb
```

## Limitations

1. **No imports**: Each file must include all needed functions
2. **No string escapes**: Can't use `\"` in strings, use alternative notation
3. **No newlines in strings**: Use separate test cases instead
4. **println only i64**: String output not available in type system

## Future Work

- [ ] String output support for debugging
- [ ] Import system for code sharing
- [ ] Full compiler pipeline in BMB
- [ ] Self-compilation of the bootstrap
- [x] MIR foundation (v0.10.1) ✅
- [x] AST → MIR lowering (v0.10.2) ✅
- [x] End-to-end pipeline: source → AST → MIR → text output (v0.10.3) ✅
- [x] MIR → C code generation (v0.10.4) ✅
- [ ] Struct/Enum lowering support (v0.11+)
- [ ] Optimization passes in BMB (v0.11+)
