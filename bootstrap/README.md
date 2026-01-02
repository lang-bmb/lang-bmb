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

### llvm_ir.bmb (35KB) - v0.10.8
Complete LLVM IR generation module with full pipeline integration.

**Features:**
- Type mapping: i64 → i64, i32 → i32, bool → i1, unit → void
- MIR instruction parsing and LLVM IR generation
- Arithmetic operators: add, sub, mul, sdiv, srem
- Comparison operators: icmp eq/ne/slt/sgt/sle/sge
- Logical operators: and, or, xor
- Unary operators: neg (sub 0), not (xor 1)
- **Control flow (v0.10.6):**
  - Labels: `entry:`, `then_0:`, `else_0:`
  - Unconditional branch: `br label %target`
  - Conditional branch: `br i1 %cond, label %then, label %else`
  - Return: `ret i64 %value`, `ret void`
  - PHI nodes: `%result = phi i64 [ %a, %then ], [ %b, %else ]`
- **Function generation (v0.10.7):**
  - Function headers: `define i64 @add(i64 %a, i64 %b) {`
  - Parameter conversion: MIR → LLVM parameter format
  - Function calls: `%r = call i64 @func(i64 %a)`
  - Complete function transformation: MIR → LLVM IR
- **Full pipeline integration (v0.10.8):**
  - Program generation: Multiple functions with `||` separator
  - Module headers: ModuleID and target triple
  - Runtime declarations: println, abs, min, max
  - End-to-end validation: MIR function → LLVM IR function

**Complete Pipeline Architecture:**
```
Source (BMB) → Lexer → Parser → AST → MIR → LLVM IR Text
```

**LLVM IR Generation:**
```llvm
; MIR → LLVM IR examples
%_t0 = const I:42      →  %_t0 = add i64 0, 42
%_t0 = + %a, %b        →  %_t0 = add i64 %a, %b
%_t0 = - %a, %b        →  %_t0 = sub i64 %a, %b
%_t0 = * %a, %b        →  %_t0 = mul i64 %a, %b
%_t0 = / %a, %b        →  %_t0 = sdiv i64 %a, %b
%_t0 = == %x, %y       →  %_t0 = icmp eq i64 %x, %y
%_t0 = < %x, %y        →  %_t0 = icmp slt i64 %x, %y
%_t0 = neg %x          →  %_t0 = sub i64 0, %x
%_t0 = not %b          →  %_t0 = xor i1 %b, 1
%_t0 = and %a, %b      →  %_t0 = and i1 %a, %b

; Control flow (v0.10.6)
entry:                 →  entry:
br label %done         →  br label %done
br i1 %c, label %t, label %e
ret i64 %x             →  ret i64 %x
%r = phi i64 [ %a, %then ], [ %b, %else ]

; Function generation (v0.10.7)
fn add(a: i64, b: i64) -> i64 {  →  define i64 @add(i64 %a, i64 %b) {
%_t0 = call foo(%a, %b)         →  %_t0 = call i64 @foo(i64 %a, i64 %b)

; Runtime declarations (v0.10.8)
declare void @println(i64)
declare i64 @abs(i64)
declare i64 @min(i64, i64)
declare i64 @max(i64, i64)
```

**Test output:**
```
777 (start marker)
5  (type mapping tests)
3  (constant generation tests)
5  (arithmetic operation tests)
4  (comparison operation tests)
2  (logical operation tests)
2  (unary operation tests)
5  (instruction parsing tests)
5  (const parsing tests)
3  (label tests)
3  (branch tests)
2  (return tests)
2  (phi tests)
3  (terminator tests)
7  (line detection tests)
3  (function header tests)
3  (parameter generation tests)
3  (call generation tests)
3  (parameter conversion tests)
3  (field extraction tests)
3  (call args conversion tests)
3  (function generation tests)
3  (call line detection tests)
2  (module header tests)
3  (extern declaration tests)
4  (full add function tests)
4  (full max function tests)
2  (double pipe tests)
3  (has pattern tests)
888 (separator)
93 (total passed)
999 (end marker)
```

### compiler.bmb (42KB) - v0.10.9
Unified compiler entry point providing complete Source → LLVM IR compilation.

**Features:**
- Unified compilation pipeline in single file
- Source → AST (parse_source)
- AST → MIR (lower_program)
- MIR → LLVM IR (gen_program)
- Module assembly: header + runtime declarations + functions
- `compile_program(source)` → complete LLVM IR module

**Architecture:**
```
Source (BMB) → Parser → S-expr AST → Lowering → MIR Text → LLVM Gen → LLVM IR
                                                              ↓
                                           Module Header + Runtime Decls + Functions
```

**API Functions:**
```bmb
-- Compile BMB source to complete LLVM IR module
fn compile_program(source: String) -> String

-- Compile single function source to LLVM IR
fn compile_function(source: String) -> String

-- Error handling
fn is_compile_error(result: String) -> bool
fn get_error_type(result: String) -> String

-- Module generation
fn gen_module_header() -> String
fn gen_runtime_decls() -> String
```

**Compilation Example:**
```bmb
-- Input: BMB source
"fn add(a: i64, b: i64) -> i64 = a + b;"

-- Output: LLVM IR module
; ModuleID = bmb_bootstrap
target triple = x86_64-unknown-linux-gnu

declare void @println(i64)
declare i64 @abs(i64)
declare i64 @min(i64, i64)
declare i64 @max(i64, i64)

define i64 @add(i64 %a, i64 %b) {
entry:
  %_t0 = add i64 %a, %b
  ret i64 %_t0
}
```

**Note:** Due to interpreter stack limits, tests use pre-computed AST inputs
rather than parsing BMB source strings within the test file.

**Test output:**
```
777 (start marker)
1  (module header test)
1  (runtime declarations test)
1  (lower simple AST test)
1  (MIR function signature test)
1  (LLVM generation test)
1  (LLVM return instruction test)
1  (lower binop AST test)
1  (LLVM add instruction test)
888 (separator)
8 (total passed)
```

## Integration Testing (v0.10.10)

The `runtime/` directory contains integration testing infrastructure for validating generated LLVM IR.

### Files

| File | Purpose |
|------|---------|
| `runtime.c` | C runtime library with println, abs, min, max functions |
| `test_add.ll` | Simple LLVM IR test (add function) |
| `test_max.ll` | Complex LLVM IR test (if-then-else with PHI nodes) |
| `validate_llvm_ir.sh` | Shell script for IR validation |
| `build_test.ps1` | PowerShell script for full Windows build |

### Runtime Functions

```c
// Bootstrap runtime functions (matches llvm_ir.bmb declarations)
void println(int64_t x);     // Print i64 with newline
int64_t abs(int64_t x);      // Absolute value
int64_t min(int64_t a, int64_t b);  // Minimum
int64_t max(int64_t a, int64_t b);  // Maximum
```

### Validation Process

```bash
# Validate LLVM IR syntax and compile to object file
cd runtime
bash validate_llvm_ir.sh

# Output:
# [1/3] Validating LLVM IR syntax...
#   ✓ LLVM IR syntax valid
# [2/3] Compiling to object file...
#   ✓ Object file created (724 bytes)
# [3/3] Verifying symbols...
#   ✓ Symbol 'add' found (defined)
#   ✓ Symbol 'main' found (defined)
#   ✓ Symbol 'println' found (external reference)
```

### Full Build (Windows with Visual Studio)

```powershell
# From Developer PowerShell for VS 2022
cd runtime
.\build_test.ps1 -Run

# Creates test_add.exe and runs it
```

## End-to-End Validation (v0.10.11)

The `examples/bootstrap_test/` directory provides comprehensive end-to-end validation comparing interpreter results with natively compiled executables.

### Test Programs

| Program | Algorithm | Expected Output |
|---------|-----------|-----------------|
| `fibonacci.bmb` | Recursive Fibonacci(10) | 55 |
| `factorial.bmb` | Iterative factorial(5) | 120 |

### Hand-Written LLVM IR

Each test program has a corresponding `.ll` file demonstrating the expected LLVM IR output:

**fibonacci.ll** - Recursive with PHI nodes:
```llvm
define i64 @fib(i64 %n) {
entry:
  %cmp = icmp sle i64 %n, 1
  br i1 %cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  %n_minus_1 = sub i64 %n, 1
  %fib_n1 = call i64 @fib(i64 %n_minus_1)
  %n_minus_2 = sub i64 %n, 2
  %fib_n2 = call i64 @fib(i64 %n_minus_2)
  %sum = add i64 %fib_n1, %fib_n2
  br label %merge_0
merge_0:
  %result = phi i64 [ %n, %then_0 ], [ %sum, %else_0 ]
  ret i64 %result
}
```

**factorial.ll** - Tail-recursive with accumulator:
```llvm
define i64 @factorial_iter(i64 %n, i64 %acc) {
entry:
  %cmp = icmp sle i64 %n, 1
  br i1 %cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  %n_minus_1 = sub i64 %n, 1
  %new_acc = mul i64 %acc, %n
  %rec_result = call i64 @factorial_iter(i64 %n_minus_1, i64 %new_acc)
  br label %merge_0
merge_0:
  %result = phi i64 [ %acc, %then_0 ], [ %rec_result, %else_0 ]
  ret i64 %result
}
```

### Validation Scripts

| Script | Platform | Purpose |
|--------|----------|---------|
| `validate_all.sh` | Unix/Git Bash | Compile all .ll files and verify symbols |
| `run_test.sh` | Unix/Git Bash | Full e2e test: interpreter vs native |
| `run_test.ps1` | Windows PowerShell | Full e2e test with Visual Studio |

### Running Validation

```bash
# Quick validation (LLVM IR → object file)
cd examples/bootstrap_test
bash validate_all.sh

# Output:
# === BMB Bootstrap LLVM IR Validation ===
# --- Testing: fibonacci ---
#   ✓ Compiled successfully
#   ✓ 'main' symbol found
#   ✓ 'println' external reference found
#   ✓ fibonacci PASSED

# Full end-to-end test (requires Developer PowerShell on Windows)
.\run_test.ps1

# Output:
# [1/5] Running with BMB interpreter...
#   Interpreter result: 55
# [2/5] Compiling LLVM IR...
# [3/5] Compiling runtime...
# [4/5] Linking...
# [5/5] Running native executable...
#   Native result: 55
# SUCCESS: Results match!
```

### Symbol Verification

The LLVM object files are verified with llvm-nm:
```
00000000 T fib           # T = defined function
00000060 T main          # T = defined function
         U println       # U = external reference (runtime)
```

### Test LLVM IR Examples

**test_add.ll** - Basic function call:
```llvm
define i64 @add(i64 %a, i64 %b) {
entry:
  %_t0 = add i64 %a, %b
  ret i64 %_t0
}
```

**test_max.ll** - Control flow with PHI nodes:
```llvm
define i64 @max_manual(i64 %a, i64 %b) {
entry:
  %cmp = icmp sgt i64 %a, %b
  br i1 %cmp, label %then_0, label %else_0
then_0:
  br label %merge_0
else_0:
  br label %merge_0
merge_0:
  %result = phi i64 [ %a, %then_0 ], [ %b, %else_0 ]
  ret i64 %result
}
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
cargo run --release --bin bmb -- check bootstrap/llvm_ir.bmb
cargo run --release --bin bmb -- check bootstrap/compiler.bmb

# Run tests
cargo run --release --bin bmb -- run bootstrap/lexer.bmb
cargo run --release --bin bmb -- run bootstrap/parser.bmb
cargo run --release --bin bmb -- run bootstrap/parser_ast.bmb
cargo run --release --bin bmb -- run bootstrap/parser_test.bmb
cargo run --release --bin bmb -- run bootstrap/types.bmb
cargo run --release --bin bmb -- run bootstrap/mir.bmb
cargo run --release --bin bmb -- run bootstrap/lowering.bmb
cargo run --release --bin bmb -- run bootstrap/pipeline.bmb
cargo run --release --bin bmb -- run bootstrap/llvm_ir.bmb
cargo run --release --bin bmb -- run bootstrap/compiler.bmb
```

## Limitations

1. **No imports**: Each file must include all needed functions
2. **No string escapes**: Can't use `\"` in strings, use alternative notation
3. **No newlines in strings**: Use separate test cases instead
4. **println only i64**: String output not available in type system

## Future Work

- [ ] String output support for debugging
- [ ] Import system for code sharing
- [ ] Self-compilation of the bootstrap
- [x] MIR foundation (v0.10.1) ✅
- [x] AST → MIR lowering (v0.10.2) ✅
- [x] End-to-end pipeline: source → AST → MIR → text output (v0.10.3) ✅
- [x] MIR → LLVM IR foundation (v0.10.5) ✅
- [x] LLVM IR control flow: branch, label, phi (v0.10.6) ✅
- [x] LLVM IR function generation (v0.10.7) ✅
- [x] Full compiler pipeline integration (v0.10.8) ✅
- [x] Unified compiler entry point (v0.10.9) ✅
- [x] Integration testing with LLVM toolchain (v0.10.10) ✅
- [x] End-to-end program compilation validation (v0.10.11) ✅
- [ ] Struct/Enum lowering support (v0.11+)
- [ ] Optimization passes in BMB (v0.11+)
