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

### parser_ast.bmb (48KB) - v0.30.2
Parser that produces S-expression AST representation.

**Features (v0.22):**
- Struct definition parsing: `struct Point { x: i64, y: i64 }`
- Struct initialization: `new Point { x: 10, y: 20 }`
- Field access: `p.x`, `p.inner.z` (chained)
- Enum definition parsing: `enum Option { Some(i64), None }`
- Match expression: `match x { Some(v) -> v, None -> 0 }`

**Features (v0.30.1):**
- Generic type application: `Vec<i64>`, `Map<String, i64>`
- Nested generics: `Container<Vec<i64>>`
- Generic types in parameters: `fn foo(data: Vec<i64>)`
- Generic types in return: `fn bar() -> Option<i64>`
- Generic types in struct fields: `struct Foo { items: Vec<i64> }`

**Features (v0.30.2):**
- Type parameter declarations: `struct Box<T> { value: T }`
- Multi-param generics: `struct Pair<K, V> { key: K, val: V }`
- Generic enum definitions: `enum Option<T> { Some(T), None }`
- Generic function definitions: `fn identity<T>(x: T) -> T = x;`
- 39 tests (6 new for type parameter parsing)

**AST Format:**
```lisp
(program
  (fn <name> (params (p <param> type)...) return-type body)
  (struct <name> (fields (field <fname> type)...))
  (enum <name> (variants (variant <vname>) (variant <vname> type)...))
  (type_app <TypeName> (Arg1 Arg2 ...)))  ; v0.30.1: Generic type application
  (type_params <T> <U> ...))              ; v0.30.2: Type parameter declarations

; Examples:
(fn <add> (params (p <x> i64) (p <y> i64)) i64 (op + (var <x>) (var <y>)))
(if (condition) (then-expr) (else-expr))
(let <name> (value) (body))
(call <name> (arg1) (arg2)...)
(struct <Point> (fields (field <x> i64) (field <y> i64)))
(struct <Box> (type_params <T>) (fields (field <value> T)))  ; v0.30.2
(enum <Option> (variants (variant <Some> i64) (variant <None>)))
(enum <Option> (type_params <T>) (variants (variant <Some> T) (variant <None>)))  ; v0.30.2
(fn <identity> (type_params <T>) (params (p <x> T)) T (var <x>))  ; v0.30.2
(new <Point> (x (int 10)) (y (int 20)))
(field (var <p>) <x>)
(match (var <x>) (arms (arm (pattern <Some> <v>) (var <v>)) (arm (pattern <None>) (int 0))))
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

### types.bmb (209KB) - v0.30.22
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
- Named types: struct/enum support (v0.22.2)

**Features (v0.30.3):**
- Type parameter encoding: `kind=10` (TypeParam)
- Type parameter environment: comma-separated names ("T,U,V")
- `tparam_add`, `tparam_lookup`, `tparam_resolve` functions
- Scope tracking for generic type parameters

**Features (v0.30.4):**
- Type name resolution: `resolve_type_name(tenv, name)`
- Primitive type detection: `is_primitive_type`, `primitive_type`
- Type parameter name detection: `is_type_param_name` (A-Z)
- Named type hashing: `name_hash` for struct/enum

**Features (v0.30.5):**
- Generic type application encoding: `kind=11` (GenericApp)
- `type_generic_app(base_hash)` → 11000 + base_hash
- `is_generic_app`, `generic_base_hash` functions
- Common generic type constructors: `type_vec`, `type_option`, `type_result`, `type_map`
- Mod-safe hash function: `name_hash_mod` for base type names

**Features (v0.30.6):**
- Type argument tracking: String-based full type info
- Format: `"Base:Arg1,Arg2,..."` (e.g., `"Vec:i64"`, `"Map:String,i64"`)
- `gen_type_pack`, `gen_type_base`, `gen_type_args`: Pack/unpack functions
- `gen_type_arg_count`, `gen_type_arg_at`: Argument access functions
- `gen_vec_info`, `gen_option_info`, `gen_result_info`, `gen_map_info`: Convenience constructors

**Features (v0.30.7):**
- Type substitution: Maps type parameters to concrete types
- Format: `"T=i64,U=String"` (comma-separated param=type pairs)
- `subst_new`, `subst_add`: Create and extend substitutions
- `subst_lookup`, `subst_has`: Query substitution mappings
- `subst_apply`: Apply substitution to simple type names
- `subst_apply_gen`: Apply substitution to generic type info (e.g., `Vec:T` → `Vec:i64`)
- `subst_from_params_args`: Build substitution from type params and args

**Features (v0.30.8):**
- Generic instantiation integration: Connect substitution to type checking
- `instantiate_generic`: Create instantiated type info from base, params, args
- `get_instantiation_subst`: Build substitution for generic instantiation
- `resolve_field_type`: Resolve field types using substitution (T → i64, Vec:T → Vec:i64)
- `check_arity`: Validate type argument count matches parameters
- `validate_type_app`: Check well-formedness of type applications
- `instantiate_type`: Full instantiation with validation and encoding

**Features (v0.30.9):**
- Generic function type checking: Signature representation and instantiation
- Format: `"name|tparams|params|return"` (e.g., `"identity|T|T|T"`, `"map|K,V|K,V|V"`)
- `gen_fn_pack`, `gen_fn_name`, `gen_fn_tparams`, `gen_fn_params`, `gen_fn_return`: Pack/unpack functions
- `gen_fn_instantiate`: Instantiate generic function with concrete type arguments
- `gen_fn_param_count`, `gen_fn_param_at`: Parameter access functions
- `gen_fn_check_call`: Validate generic function call (arity, type matching)
- `gen_fn_match_params`: Match expected and actual parameter types

**Features (v0.30.10):**
- Generic type inference: Infer type arguments from actual argument types
- `is_single_tparam`: Check if string is a single type parameter (A-Z)
- `infer_single`: Infer binding from single param/actual pair (T, i64 → T=i64)
- `infer_merge`: Merge substitutions with conflict detection
- `infer_from_pair_list`: Infer all type params from param/actual lists
- `infer_all_bound`, `infer_build_targs`: Validate and construct type args
- `gen_fn_infer_call`: Full inference and checking for generic function calls

**Features (v0.30.11):**
- Generic struct definition: Represent and resolve generic structs
- Format: `"StructName|tparams|field1:type1,field2:type2"` (e.g., `"Box|T|value:T"`)
- `gen_struct_pack`, `gen_struct_name`, `gen_struct_tparams`, `gen_struct_fields`: Pack/unpack
- `gen_struct_field_type`: Get field type string by name
- `gen_struct_resolve_field`: Resolve field type with type arguments (Box<i64>.value → i64)
- `gen_struct_is_generic`, `gen_struct_field_count`, `gen_struct_field_name_at`: Utilities

**Features (v0.30.12):**
- Struct registry: Global registry for managing struct definitions
- Format: `"Name1=def1;Name2=def2;..."` (semicolon-separated name=definition pairs)
- `struct_reg_new`, `struct_reg_add`: Create and populate registry
- `struct_reg_lookup`, `struct_reg_has`: Query registry for definitions
- `struct_reg_field_type`: Resolve field type with type arguments through registry lookup
- `struct_reg_count`, `struct_reg_is_generic`: Registry utilities

**Features (v0.30.13):**
- Generic enum definitions: Represent and resolve generic enums
- Format: `"EnumName|tparams|variant1:type1,variant2,variant3:type3"` (e.g., `"Option|T|Some:T,None"`)
- `gen_enum_pack`, `gen_enum_name`, `gen_enum_tparams`, `gen_enum_variants`: Pack/unpack
- `gen_enum_variant_type`, `gen_enum_has_variant`: Variant type lookup
- `gen_enum_resolve_variant`: Resolve variant type with type arguments (Option<i64>.Some → i64)
- `gen_enum_is_generic`, `gen_enum_variant_count`, `gen_enum_variant_name_at`: Utilities

**Features (v0.30.14):**
- Enum registry: Global registry for managing enum definitions
- Format: `"Name1=def1;Name2=def2;..."` (semicolon-separated name=definition pairs)
- `enum_reg_new`, `enum_reg_add`: Create and populate registry
- `enum_reg_lookup`, `enum_reg_has`: Query registry for definitions
- `enum_reg_variant_type`: Resolve variant type with type arguments through registry lookup
- `enum_reg_count`, `enum_reg_is_generic`: Registry utilities

**Features (v0.30.15):**
- Function registry: Global registry for managing function signatures
- Format: `"Name1=sig1;Name2=sig2;..."` (semicolon-separated name=signature pairs)
- `fn_reg_new`, `fn_reg_add`: Create and populate registry
- `fn_reg_lookup`, `fn_reg_has`: Query registry for function signatures
- `fn_reg_return_type`: Get return type with type arguments applied
- `fn_reg_param_type_at`: Get parameter type at index with type arguments applied
- `fn_reg_count`, `fn_reg_is_generic`, `fn_reg_param_count`: Registry utilities

**Features (v0.30.16):**
- Type environment: Unified container for all type information
- Format: `"P:tparams#S:struct_reg#E:enum_reg#F:fn_reg"` (hash-separated sections)
- `tenv_new`, `tenv_get_tparams`, `tenv_get_struct_reg`, `tenv_get_enum_reg`, `tenv_get_fn_reg`: Accessors
- `tenv_with_tparams`, `tenv_with_struct_reg`, `tenv_with_enum_reg`, `tenv_with_fn_reg`: Setters
- `tenv_add_struct`, `tenv_add_enum`, `tenv_add_fn`, `tenv_add_tparam`: Registry integration
- `tenv_struct_field_type`, `tenv_enum_variant_type`, `tenv_fn_return_type`: Type resolution
- `tenv_has_tparam`, `tenv_resolve_tparam`: Type parameter operations

**Features (v0.30.17):**
- Generic call site type checking through type environment
- `tenv_check_fn_call`: Check generic function call with explicit type args
- `tenv_infer_fn_call`: Infer and check generic function call
- `tenv_check_field_access`: Check struct field access with type args
- `tenv_check_match_variant`: Check enum pattern in match expression
- `tenv_extract_field_type`: Extract field type from packed field:type string

**Features (v0.30.18):**
- AST-Type integration: Connect parser_ast.bmb output to tenv system
- AST navigation utilities: `ast_find_close_paren`, `ast_skip_ws`, `ast_find_pattern`
- AST name extraction: `ast_extract_angle_name`, `ast_extract_def_name`
- Type parameter extraction: `ast_extract_type_params` - e.g., `(type_params <T> <U>)` → `"T,U"`
- Fields extraction: `ast_extract_fields` - e.g., `(fields (field <x> i64))` → `"x:i64"`
- Variants extraction: `ast_extract_variants` - e.g., `(variants (variant <Some> T))` → `"Some:T"`
- Function signature extraction: `ast_extract_param_types`, `ast_extract_return_type`
- AST to registry converters: `ast_struct_to_def`, `ast_enum_to_def`, `ast_fn_to_sig`
- tenv registration from AST: `register_struct_from_ast`, `register_enum_from_ast`, `register_fn_from_ast`

**Features (v0.30.19):**
- Program AST traversal: Navigate `(program ...)` S-expressions from parser_ast.bmb
- Item kind detection: `ITEM_FN`, `ITEM_STRUCT`, `ITEM_ENUM` constants
- `ast_item_kind`: Detect item type from AST prefix `(fn `, `(struct `, `(enum `
- `ast_program_start`: Find position after `(program ` prefix (returns 9)
- `ast_extract_item_at`: Extract complete item S-expression at position
- `ast_next_item_pos`: Get position of next item after current one
- `ast_program_item_count`, `ast_program_item_at`: Count and access items by index
- `register_item`: Route item registration based on kind
- `tenv_from_program_ast`: Main entry point - build complete tenv from program AST

**Features (v0.30.20):**
- Expression type checking: `type_of_expr` for S-expression AST inference
- Local variable environment: `locals_new`, `locals_add`, `locals_lookup`
- Expression kind constants: `EXPR_INT`, `EXPR_BOOL`, `EXPR_VAR`, `EXPR_OP`, `EXPR_IF`, `EXPR_LET`, `EXPR_CALL`, `EXPR_NEW`, `EXPR_FIELD`, `EXPR_MATCH`
- Literal type checking: `(int n)` → i64, `(bool n)` → bool
- Variable type checking: `(var <name>)` → lookup in locals
- Operator type checking: `type_of_unop`, `type_of_binop`, `binop_result_type`
- Control flow type checking: `type_of_if` (condition bool, branches match)
- Let binding type checking: `type_of_let` with scope extension
- Function call type checking: `type_of_call` with argument type collection
- Struct construction/field access: `type_of_new`, `type_of_field`
- Error propagation: `is_error_str` for String-based type error detection

**Features (v0.30.21):**
- Function body type checking: Complete program-wide type validation pipeline
- `ast_extract_fn_body`: Extract function body expression from AST
- `ast_extract_param_name`, `ast_extract_param_type`: Parameter parsing
- `ast_extract_params_section`, `ast_count_params`, `ast_get_param_at`: Params section utilities
- `ast_params_to_locals`: Convert function params to locals environment
- `check_fn_body`: Validate function body type matches declared return type
- `check_program_functions`: Check all functions in a program
- `typecheck_program`: Full pipeline - build tenv, then validate all functions

**Features (v0.30.22):**
- Generic function body type checking: Type parameter scope for function bodies
- Modified `check_fn_body` to extract and set type parameters in tenv
- Uses `ast_extract_type_params` to get function's type parameters
- Uses `tenv_with_tparams` to create function-scoped type environment
- Supports `fn identity<T>(x: T) -> T = x` pattern
- Correctly validates generic return types match body types (T == T)
- Detects type mismatches in generic functions (T vs U)

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
4  (struct type tests)
4  (match type tests)
5  (type param encoding tests)
6  (type param env tests)
6  (type param lookup tests)
4  (type param resolve tests)
7  (primitive type tests)
8  (type param name tests)
8  (resolve type name tests)
6  (generic app encoding tests)
7  (common generic types tests)
7  (name hash mod tests)
6  (gen type pack tests)
4  (gen type arg count tests)
7  (gen type arg at tests)
9  (gen convenience tests)
6  (subst basic tests)
5  (subst lookup tests)
5  (subst apply tests)
6  (subst apply gen tests)
6  (subst from params tests)
5  (instantiate generic tests)
6  (resolve field type tests)
6  (check arity tests)
5  (instantiate type tests)
8  (gen fn pack tests)
5  (gen fn instantiate tests)
4  (gen fn param count tests)
6  (gen fn check call tests)
6  (is single tparam tests)
4  (infer single tests)
5  (infer merge tests)
4  (infer from pair list tests)
5  (infer all bound tests)
3  (infer build targs tests)
5  (gen fn infer call tests)
6  (gen struct pack tests)
5  (gen struct field type tests)
4  (gen struct field count tests)
4  (gen struct resolve field tests)
2  (gen struct is generic tests)
4  (gen struct field name at tests)
4  (struct reg add tests)
5  (struct reg lookup tests)
2  (struct reg has tests)
6  (struct reg field type tests)
3  (struct reg is generic tests)
6  (gen enum pack tests)
5  (gen enum variant type tests)
5  (gen enum has variant tests)
5  (gen enum resolve variant tests)
2  (gen enum is generic tests)
3  (gen enum variant count tests)
5  (gen enum variant name at tests)
3  (enum reg add tests)
5  (enum reg lookup tests)
2  (enum reg has tests)
6  (enum reg variant type tests)
3  (enum reg is generic tests)
4  (fn reg add tests)
5  (fn reg lookup tests)
2  (fn reg has tests)
5  (fn reg return type tests)
4  (fn reg param type at tests)
3  (fn reg is generic tests)
4  (fn reg param count tests)
4  (tenv new tests)
4  (tenv with tests)
5  (tenv add tests)
3  (tenv struct field tests)
3  (tenv enum variant tests)
3  (tenv fn return tests)
5  (tenv tparam ops tests)
5  (tenv fn call tests)
4  (tenv field access tests)
4  (tenv match variant tests)
4  (tenv extract field tests)
4  (AST extract name tests)           ; v0.30.18
5  (AST extract type params tests)    ; v0.30.18
4  (AST extract fields tests)         ; v0.30.18
4  (AST extract variants tests)       ; v0.30.18
4  (AST fn params tests)              ; v0.30.18
5  (AST struct to def tests)          ; v0.30.18
5  (AST enum to def tests)            ; v0.30.18
5  (AST fn to sig tests)              ; v0.30.18
4  (pack/unpack tests)                ; v0.30.18
3  (register struct tests)            ; v0.30.18
2  (register enum tests)              ; v0.30.18
4  (item kind tests)                  ; v0.30.19
3  (program start tests)              ; v0.30.19
4  (program item count tests)         ; v0.30.19
3  (program item at tests)            ; v0.30.19
3  (tenv from program tests)          ; v0.30.19
2  (simple program tests)             ; v0.30.19
3  (locals tests)                     ; v0.30.20
6  (expr kind tests)                  ; v0.30.20
2  (type literals tests)              ; v0.30.20
2  (type var tests)                   ; v0.30.20
3  (type binop tests)                 ; v0.30.20
3  (type if tests)                    ; v0.30.20
2  (type let tests)                   ; v0.30.20
2  (fn body extract tests)            ; v0.30.21
4  (param extraction tests)           ; v0.30.21
4  (params section tests)             ; v0.30.21
2  (params to locals tests)           ; v0.30.21
3  (check fn body tests)              ; v0.30.21
3  (typecheck program tests)          ; v0.30.21
5  (check fn body generic tests)      ; v0.30.22
3  (typecheck program generic tests)  ; v0.30.22
888 (separator)
517 (total passed)
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

### optimize.bmb (18KB) - v0.29.1
MIR optimization passes for the bootstrap compiler.

**Features:**
- Constant folding: Evaluate constant expressions at compile time
- Dead code elimination: Remove unreachable code (infrastructure)
- Branch simplification: Optimize conditional branches with constant conditions
- Copy propagation: Replace copies with original values (infrastructure)
- String-based constant and copy tables for tracking values
- Modular pass architecture for extensibility

**Optimization Passes:**
```
1. ConstantFolding:
   %_t0 = const I:10
   %_t1 = const I:20
   %_t2 = + %_t0, %_t1  →  %_t2 = const I:30

2. SimplifyBranches:
   branch %const_true, then, else  →  goto then

3. CopyPropagation (infrastructure):
   %_t0 = copy %x
   ... use %_t0 ...  →  ... use %x ...
```

**Test output:**
```
777 (start marker)
1  (const_table tests)
1  (extract_dest tests)
1  (extract_const tests)
1  (extract_binop tests)
1  (eval_binop tests)
0  (const_folding integration - WIP)
1  (is_checks tests)
0  (optimize_mir integration - WIP)
1  (copy_table tests)
888 (separator)
7 (total passed)
999 (end marker)
```

### lowering.bmb (50KB) - v0.21.1
AST to MIR lowering (transformation) module.

**Features:**
- S-expression AST parsing (from parser_ast.bmb output)
- Expression lowering: int, bool, var, binop, unary, if, let, call
- **Struct lowering (v0.21.0):** struct-init, field-access, field-store
- **Enum lowering (v0.21.1):** enum-variant with discriminant
- **Match lowering (v0.21.1):** switch instruction with cases
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

; Struct support (v0.21.0)
(new Point (x (int 10)) (y (int 20))) →  %_t0 = struct-init Point { x: I:10, y: I:20 }
(field (var <p>) x)                   →  %_t0 = field-access %p.x

; Enum support (v0.21.1)
(Status::None)                        →  %_t0 = enum-variant Status::None 0
(Status::Active (int 42))             →  %_t0 = enum-variant Status::Active 1 I:42
(match (var <s>) ...)                 →  switch %s, 0 -> label1, 1 -> label2
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
4  (struct node detection - v0.21.0)
3  (struct value extraction - v0.21.0)
3  (struct lowering - v0.21.0)
3  (enum node detection - v0.21.1)
4  (enum extraction - v0.21.1)
3  (enum variant lowering - v0.21.1)
3  (match lowering - v0.21.1)
888 (separator)
67 (total passed)
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

### llvm_ir.bmb (58KB) - v0.21.1
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
- **Struct codegen (v0.21.0):**
  - struct-init → insertvalue chain
  - field-access → extractvalue
- **Enum codegen (v0.21.1):**
  - enum-variant → insertvalue (discriminant + payload)
  - switch → LLVM switch instruction
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

; Struct codegen (v0.21.0)
%_t0 = struct-init Point { x: %x, y: %y }
  → %_t0_0 = insertvalue %Point %Point zeroinitializer, i64 %x, 0
  → %_t0 = insertvalue %Point %_t0_0, i64 %y, 1
%_t0 = field-access %p.x    →  %_t0 = extractvalue %Point %p, 0

; Enum codegen (v0.21.1)
%_t0 = enum-variant Status::None 0       →  %_t0 = add i64 0, 0
%_t0 = enum-variant Status::Active 1 %v  →  %_t0_d = insertvalue %EnumData ..., 1
                                         →  %_t0 = insertvalue %EnumData ..., %v
switch %s, [0 -> arm0, 1 -> arm1], merge
  → switch i64 %s, label %merge [i64 0, label %arm0 i64 1, label %arm1]

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
4  (struct line detection - v0.21.0)
3  (insertvalue generation - v0.21.0)
2  (extractvalue generation - v0.21.0)
3  (field name to index - v0.21.0)
3  (field access IR - v0.21.0)
4  (enum line detection - v0.21.1)
3  (enum variant IR - v0.21.1)
4  (switch IR - v0.21.1)
888 (separator)
119 (total passed)
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

## Self-Hosting Tests (v0.23)

Self-hosting verification tests for the Bootstrap compiler.

### selfhost_test.bmb - Stage 1 Parser Verification

Tests the Bootstrap parser's ability to parse BMB source code correctly.

**Features:**
- Token encoding and lexer functions
- Parser for functions, expressions, operators
- 8 test cases covering core language features

**Test coverage:**
1. Constant function: `fn one() -> i64 = 1;`
2. Parameter function: `fn id(x: i64) -> i64 = x;`
3. Binary operations: `fn add(a: i64, b: i64) -> i64 = a + b;`
4. If expressions: `fn max(a: i64, b: i64) -> i64 = if a > b then a else b;`
5. Let expressions: `fn double(x: i64) -> i64 = let y = x + x; y;`
6. Function calls: `fn double_inc(x: i64) -> i64 = inc(inc(x));`
7. Comparison operators: `fn test(a: i64, b: i64) -> bool = a == b;`
8. Boolean expressions: `fn test(a: bool, b: bool) -> bool = a and b or not a;`

**Test output:**
```
777 (start marker)
1  (section marker)
1  (8 parser tests...)
888 (separator)
8 (total passed)
999 (end marker)
```

### selfhost_equiv.bmb - Stage 2 Equivalence Tests

Tests equivalence between Rust compiler output and Bootstrap compiler patterns.

**Features:**
- MIR pattern matching (entry, binop, return, branch)
- LLVM IR pattern matching (define, add, ret, icmp, br, phi)
- Bootstrap lowering pattern verification
- Bootstrap LLVM codegen pattern verification

**Test categories:**
1. MIR patterns: 5 tests (entry, binop, return, cmp, branch)
2. LLVM IR patterns: 6 tests (define, add, ret, icmp, br, phi)
3. Bootstrap MIR patterns: 3 tests (const, binop, call)
4. Bootstrap LLVM patterns: 5 tests (const, binop, cmp, branch, phi)

**Test output:**
```
777 (start marker)
2  (MIR section)
5  (5 MIR tests...)
3  (LLVM section)
6  (6 LLVM tests...)
4  (Bootstrap MIR section)
3  (3 lowering tests...)
5  (Bootstrap LLVM section)
5  (5 codegen tests...)
888 (separator)
19 (total passed)
999 (end marker)
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

### utils.bmb (21KB) - v0.29.6
Shared utility functions for all bootstrap modules.

**Purpose:**
Consolidates common string operations to reduce code duplication across bootstrap files.
Since BMB doesn't yet support imports, this file serves as the canonical reference implementation.

**Sections:**
1. **Character Classification**: `is_digit`, `is_alpha`, `is_alnum`, `is_ident_start`, `is_whitespace`
2. **Character Conversion**: `digit_to_int`, `digit_char`, `int_to_string`, `char_to_string`
3. **Integer Parsing**: `parse_int`, `parse_int_signed`, `parse_int_end`
4. **String Searching**: `find_char`, `find_pipe`, `skip_ws`, `find_ident_end`, `find_number_end`
5. **String Matching**: `starts_with`, `starts_with_at`, `find_pattern`
6. **String Extraction**: `read_until_ws`, `strip_trailing_colon`
7. **Comment Handling**: `skip_comment`, `skip_to_eol`, `skip_all`
8. **Error Handling**: `is_error`, `make_error`, `get_error_msg`, `is_error_loose`
9. **Result Packing**: `pack_result`, `unpack_pos`, `unpack_ast`, `pack_values`, `unpack_first`, `unpack_rest`
10. **Token Encoding**: `tok_encode`, `tok_kind`, `tok_end`, `tok_is_kind`, `tok_is_eof`

**Duplication Reduction:**
| Function | Previously Duplicated In |
|----------|-------------------------|
| `is_digit` | 11 files |
| `is_alpha` | 10 files |
| `parse_int` | 10 files |
| `pack_result` | 8 files |
| `unpack_pos/ast` | 7 files |
| `is_error` | 6 files |
| `tok_kind/tok_end` | 6 files |
| `find_ident_end` | 6 files |
| `starts_with` | 6 files |

**Test output:**
```
✅ 33 tests passed
```

---

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
- [x] Native executable compilation: Text LLVM IR → clang/lld-link (v0.10.12) ✅
- [x] Struct/Enum MIR lowering support (v0.21.0/v0.21.1) ✅
- [x] Struct/Enum LLVM IR codegen (v0.21.0/v0.21.1) ✅
- [x] MIR text output (`--emit-mir` CLI option) (v0.21.2) ✅
- [x] Struct/Enum parsing in parser_ast.bmb (v0.22.0/v0.22.1) ✅
- [x] Struct/Enum type checking in types.bmb (v0.22.2) ✅
- [x] Parser integration tests (v0.22.3) ✅
- [x] Self-hosting Stage 1 verification (v0.23.0) ✅
- [x] Self-hosting Stage 2 equivalence tests (v0.23.1-2) ✅
- [x] MIR optimization passes in BMB (v0.29.1) ✅
- [x] Shared utilities module: utils.bmb (v0.29.3-v0.29.5) ✅
  - Character/String utilities (v0.29.3)
  - Integer parsing (v0.29.4)
  - Error handling & Result packing (v0.29.5)
- [ ] Self-hosting Stage 3 full bootstrap compilation (v0.30+)
