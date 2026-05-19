//! Pattern definitions — AI mistake patterns derived from pilot data + CLAUDE.md.
//!
//! IMPORTANT: Triggers must match ACTUAL compiler error messages, not source code.
//! lalrpop errors use: "Unrecognized token `X`", "Expected one of ..."
//! Type errors use: "expected T, got U", "unknown function `f`"
//! Last verified: 2026-03-26

use super::DiagPattern;

pub static PATTERNS: &[DiagPattern] = &[
    // ===== Parser errors (lalrpop format) =====

    DiagPattern {
        id: "option_type",
        kind: "",
        // Matches both source-level (Option<) and error messages (unknown type `option`, token `Some`)
        // Note: "None" omitted — too generic, matches LLVM IR "memory(none)" and causes false positives
        triggers: &["Option<", "Option::", "Some(", "token `some`", "token `none`", "unknown type `option`"],
        suggestion: "BMB uses T? for nullable types, not Option<T>.",
        example_wrong: "let x: Option<i64> = Some(42);",
        example_correct: "let x: i64? = 42;",
    },
    DiagPattern {
        id: "function_name_reserved",
        kind: "",
        // Linker error when user defines a function with the same name as a BMB built-in
        triggers: &["invalid redefinition of function", "redefinition of '"],
        suggestion: "This function name conflicts with a BMB built-in. Use a different name (e.g., 'clamp_val' instead of 'clamp', which is reserved).",
        example_wrong: "fn clamp(x: i64, lo: i64, hi: i64) -> i64 = ...;",
        example_correct: "fn clamp_val(x: i64, lo: i64, hi: i64) -> i64 = ...;",
    },
    DiagPattern {
        id: "vec_generic",
        kind: "",
        // lalrpop tokenizes Vec::new() as identifier Vec, then :: then token `new`
        triggers: &["Vec<", "Vec::new", "vec!", "token `new`", "unknown type `vec`"],
        suggestion: "BMB vectors use free functions: vec_new(), vec_push(v, val), vec_get(v, idx), vec_len(v), vec_free(v). Handle type is i64.",
        example_wrong: "let v: Vec<i64> = Vec::new();\nv.push(42);",
        example_correct: "let v: i64 = vec_new();\nvec_push(v, 42);",
    },
    DiagPattern {
        id: "method_call",
        kind: "",
        triggers: &[".len()", ".push(", ".pop(", ".get(", ".set(", ".contains("],
        suggestion: "BMB uses free functions, not method calls. Use func(obj, args) instead of obj.func(args).",
        example_wrong: "arr.len()",
        example_correct: "vec_len(arr)",
    },
    DiagPattern {
        id: "println_macro",
        kind: "",
        // lalrpop sees `!` after println and produces "Unrecognized token `!`"
        triggers: &["println!", "print!", "eprintln!", "format!"],
        suggestion: "BMB uses println(num) or println_str(\"text\"), not Rust macros.",
        example_wrong: "println!(\"value: {}\", x);",
        example_correct: "println(x);\nprintln_str(\"text\");",
    },
    DiagPattern {
        id: "string_type",
        kind: "",
        triggers: &["String::", "String::from", "&String", ".to_string()", ".as_str()"],
        suggestion: "BMB uses &str for strings. No String type.",
        example_wrong: "let s: String = String::from(\"hello\");",
        example_correct: "let s: &str = \"hello\";",
    },
    DiagPattern {
        id: "type_annotation",
        kind: "type",
        triggers: &["cannot infer", "type mismatch"],
        suggestion: "BMB supports type inference but some complex expressions may need explicit annotations. Add ': Type' to the let binding.",
        example_wrong: "let x = complex_expr();  // type ambiguous",
        example_correct: "let x: i64 = complex_expr();",
    },
    DiagPattern {
        id: "fn_return_expr",
        kind: "parser",
        triggers: &["expected `=`", "expected `{`", "function body"],
        suggestion: "BMB functions use '= expr;' for expression bodies or '= { ... };' for block bodies.",
        example_wrong: "fn add(a: i64, b: i64) -> i64 { a + b }",
        example_correct: "fn add(a: i64, b: i64) -> i64 = a + b;",
    },
    DiagPattern {
        id: "bitwise_ops",
        kind: "",
        triggers: &["bitwise", "bit and", "bit or"],
        suggestion: "BMB uses 'band', 'bor', 'bxor' for bitwise operations, not &, |, ^.",
        example_wrong: "let x: i64 = a & b;",
        example_correct: "let x: i64 = a band b;",
    },
    DiagPattern {
        id: "impl_block",
        kind: "parser",
        // lalrpop produces "Unrecognized token `impl`" when encountering impl
        triggers: &["`impl`", "impl block", "token `impl`"],
        suggestion: "BMB has no impl blocks. Define free functions instead.",
        example_wrong: "impl Foo {\n    fn bar(&self) -> i64 { ... }\n}",
        example_correct: "fn foo_bar(self: &Foo) -> i64 = ...;",
    },
    DiagPattern {
        id: "trait_def",
        kind: "parser",
        triggers: &["`trait`", "trait definition", "token `trait`"],
        suggestion: "BMB traits use 'trait Name { fn method(self: &Self) -> Type; }' syntax.",
        example_wrong: "trait Foo {\n    fn bar(&self) -> i64;\n}",
        example_correct: "trait Foo {\n    fn bar(self: &Self) -> i64;\n}",
    },
    DiagPattern {
        id: "tuple_destruct",
        kind: "",
        triggers: &["let (", "destructur", "tuple"],
        // `let (a, b) = expr` WORKS inside a block `{ ... }`.
        // It fails only in expression context (LALR conflict). Keep code in a block.
        suggestion: "Tuple destructuring `let (a, b) = expr;` WORKS inside a block { ... }.\nIf it fails, ensure it is inside a function block body, not in expression position.",
        example_wrong: "fn f() -> i64 = let (a, b) = pair;  // expression position — fails",
        example_correct: "fn f() -> i64 = { let (a, b) = pair; a + b };  // block — works",
    },
    DiagPattern {
        id: "match_wildcard",
        kind: "",
        triggers: &["_ ->", "wildcard"],
        // `_ =>` in match WORKS in BMB. Only `_ ->` (wrong arrow) is an issue.
        suggestion: "BMB match uses `=>` arrow (not `->`). The `_` wildcard works: `match x { 1 => a, _ => b }`.",
        example_wrong: "match x { 1 -> a, _ -> b }",
        example_correct: "match x { 1 => a, _ => b }",
    },
    DiagPattern {
        id: "static_method",
        kind: "",
        // lalrpop tokenizes Type::method() — "new", "from", etc. appear as unrecognized tokens
        triggers: &["::new(", "::from(", "::default(", "::with_capacity("],
        suggestion: "BMB has no static methods or associated functions. Use free functions.",
        example_wrong: "let v = Vec::new();",
        example_correct: "let v: i64 = vec_new();",
    },
    DiagPattern {
        id: "io_functions",
        kind: "",
        triggers: &["io::stdin", "read_line", "std::io", "BufRead"],
        suggestion: "BMB uses read_int() to read from stdin. No std::io.",
        example_wrong: "let mut input = String::new();\nstd::io::stdin().read_line(&mut input);",
        example_correct: "let n: i64 = read_int();",
    },
    DiagPattern {
        id: "array_syntax",
        kind: "",
        triggers: &["[i64;", "&[i64]", "array literal"],
        suggestion: "BMB uses vec_new()/vec_push()/vec_get() for dynamic arrays.",
        example_wrong: "let arr: [i64; 5] = [1, 2, 3, 4, 5];",
        example_correct: "let arr: i64 = vec_new();\nvec_push(arr, 1);\nvec_push(arr, 2);",
    },
    DiagPattern {
        id: "use_import",
        kind: "parser",
        triggers: &["`use`", "use std", "use crate", "token `use`"],
        suggestion: "BMB uses 'import' instead of Rust's 'use'. Standard functions are built-in.",
        example_wrong: "use std::collections::HashMap;",
        example_correct: "// No import needed - vec_new(), println(), read_int() are built-in",
    },

    // ===== Type errors =====

    DiagPattern {
        id: "void_return_used",
        kind: "type",
        triggers: &["expected i64, got ()", "expected f64, got ()", "expected bool, got ()"],
        suggestion: "println/print/vec_push/vec_set/vec_free return () not i64. Wrap in a block: { println(x); 0 }",
        example_wrong: "fn main() -> i64 = println(42);",
        example_correct: "fn main() -> i64 = { println(42); 0 };",
    },
    DiagPattern {
        id: "unit_to_value",
        kind: "type",
        triggers: &["expected (), got i64", "expected (), got f64"],
        suggestion: "Block body returns () by default. If you need to return a value, use = expr; syntax or ensure the last expression matches the return type.",
        example_wrong: "fn foo() { 42 }",
        example_correct: "fn foo() -> i64 = 42;",
    },
    DiagPattern {
        id: "nullable_type_mismatch",
        kind: "type",
        // NEW: matches "expected i64, got i64?" — AI creates nullable but context needs concrete type
        triggers: &["got i64?", "got f64?", "got bool?", "expected i64, got i64?", "expected f64, got f64?"],
        suggestion: "Cannot use T? where T is expected. Use plain T if value is always present, or match to extract the value.",
        example_wrong: "let x: i64? = 42;\nprintln(x);  // error: expected i64, got i64?",
        example_correct: "let x: i64 = 42;\nprintln(x);  // or: match x { some(v) => println(v), none => () };",
    },
    DiagPattern {
        id: "underscore_pattern",
        kind: "parser",
        triggers: &["`_`", "Unrecognized token `_`", "token `_`"],
        suggestion: "BMB does not support underscore _ patterns. Use a named variable or else clause.",
        example_wrong: "let _ = foo();",
        example_correct: "let _unused: i64 = foo();",
    },
    DiagPattern {
        id: "missing_semicolon_eof",
        kind: "parser",
        // NEW: matches the actual lalrpop EOF error format
        triggers: &["Unrecognized EOF", "Expected one of \";\""],
        suggestion: "BMB top-level definitions (fn, struct) must end with ';'. Block expressions end with '};'.",
        example_wrong: "fn main() -> i64 = { 0 }",
        example_correct: "fn main() -> i64 = { 0 };",
    },
    DiagPattern {
        id: "missing_semicolon_block",
        kind: "parser",
        triggers: &["expected `}`", "expected `;`"],
        suggestion: "BMB blocks must end with a semicolon after while/if/for. Use: while cond { body; };",
        example_wrong: "while i < n { set i = i + 1; }",
        example_correct: "while i < n { set i = i + 1; };",
    },
    DiagPattern {
        id: "missing_else",
        kind: "parser",
        triggers: &["Expected one of \"else\""],
        suggestion: "BMB if-expressions used as values require an else branch.",
        example_wrong: "let x: i64 = if cond { 1 };",
        example_correct: "let x: i64 = if cond { 1 } else { 0 };",
    },
    DiagPattern {
        id: "bool_operators",
        kind: "parser",
        // Single `|` (bitwise OR) and single `&` (bitwise AND) are unrecognized tokens.
        // Note: `||` and `&&` DO work in BMB (they are valid boolean operators).
        // The error fires only for single `|` or `&` used as bitwise operators.
        triggers: &["Unrecognized token `|`", "Unrecognized token `&`"],
        suggestion: "BMB does not support single '|' or '&' as operators.\nFor BITWISE operators: 'a | b' → 'a bor b',  'a & b' → 'a band b',  'a ^ b' → 'a bxor b'\nNote: '||' and '&&' DO work in BMB for boolean OR/AND (no change needed for those).",
        example_wrong: "let bit: i64 = n & 1;\nlet masked: i64 = a | b;",
        example_correct: "let bit: i64 = n band 1;\nlet masked: i64 = a bor b;",
    },
    DiagPattern {
        id: "closure_lambda",
        kind: "parser",
        triggers: &["`|`", "closure", "lambda", "token `|`"],
        suggestion: "BMB has no closures or lambdas. Use named functions instead.",
        example_wrong: "let f = |x| x + 1;",
        example_correct: "fn add_one(x: i64) -> i64 = x + 1;",
    },
    DiagPattern {
        id: "mutable_param",
        kind: "",
        triggers: &["&mut"],
        suggestion: "BMB function parameters are immutable. Copy to a local mut variable.",
        example_wrong: "fn foo(mut x: i64) -> i64 = { x = x + 1; x };",
        example_correct: "fn foo(x: i64) -> i64 = { let mut local: i64 = x; set local = local + 1; local };",
    },
    DiagPattern {
        id: "print_string_fn",
        kind: "type",
        triggers: &["expected &str, got i64", "expected i64, got &str"],
        suggestion: "BMB has separate print functions: println(i64) for numbers, println_str(&str) for strings.",
        example_wrong: "println(\"hello\");",
        example_correct: "println_str(\"hello\");\nprintln(42);",
    },
    DiagPattern {
        id: "if_without_else_unit",
        kind: "type",
        triggers: &["if expression without else", "branch types do not match"],
        suggestion: "BMB `if cond { body }` without else returns (). This works fine for statements.\nIf used as a VALUE (assigned to a variable), add else: `if cond { expr } else { default }`.",
        example_wrong: "let x: i64 = if cond { 42 };  // if-as-value requires else",
        example_correct: "let x: i64 = if cond { 42 } else { 0 };\nif cond { do_side_effect() };  // side-effect only — no else needed",
    },
    DiagPattern {
        id: "iterator_methods",
        kind: "",
        triggers: &[".iter()", ".map(", ".filter(", ".collect(", ".enumerate(", ".zip("],
        suggestion: "BMB has no iterators or functional methods. Use while/for loops with vec_get/vec_len.",
        example_wrong: "let sum: i64 = arr.iter().sum();",
        example_correct: "let mut sum: i64 = 0;\nfor i in 0..vec_len(arr) {\n    sum = sum + vec_get(arr, i);\n};",
    },
    DiagPattern {
        id: "type_cast",
        kind: "",
        triggers: &[" as ", "as usize", "as i64", "as i32"],
        suggestion: "BMB has no type casting with 'as'. All integers are i64.",
        example_wrong: "let idx = n as usize;",
        example_correct: "let idx: i64 = n;  // All integers are i64",
    },
    // ===== Patterns from Cycle 1+2 testing =====
    DiagPattern {
        id: "missing_return_type",
        kind: "parser",
        // AI tries void function: fn foo() = { ... } — BMB requires explicit return type
        triggers: &["Expected one of \"->\"", "expected `->`"],
        suggestion: "BMB requires explicit return type for all functions. For void-like functions, return i64 and return 0.",
        example_wrong: "fn do_stuff(x: i64) = { println(x); };",
        example_correct: "fn do_stuff(x: i64) -> i64 = { println(x); 0 };",
    },
    // =====
    DiagPattern {
        id: "unknown_function",
        kind: "type",
        // Matches common AI hallucinated function names
        triggers: &["unknown function"],
        suggestion: "Check function name. BMB built-ins:\n  I/O: println(i64), println_str(&str), print(i64), print_str(&str), read_int(), read_line()\n  Vec: vec_new(), vec_push(v,x), vec_get(v,i), vec_set(v,i,x), vec_len(v), vec_pop(v), vec_clear(v), vec_free(v)\n  Math: abs(x)->i64/f64, min(a,b), max(a,b), f64_sqrt(x)\n  String: str_concat(a,b), str_len(s), str_substr(s,start,len), str_to_int(s)",
        example_wrong: "let x: i64 = input();",
        example_correct: "let x: i64 = read_int();",
    },
    DiagPattern {
        id: "unwrap_bang",
        kind: "parser",
        // AI tries Rust-style ! for macros, unwrap, or boolean negation
        triggers: &["Unrecognized token `!`"],
        suggestion: "BMB has no ! operator.\n- For boolean negation: use 'not x' (NOT '!x')\n- For macros (println!, format!): use plain functions: println(x), format(\"{}\", x)\n- For unwrap: use match or change type to plain T",
        example_wrong: "if !found { ... }\nprintln!(\"hello\");",
        example_correct: "if not found { ... }\nprintln_str(\"hello\");",
    },
    DiagPattern {
        id: "if_stmt_no_semicolon",
        kind: "parser",
        // lalrpop produces "Unrecognized token `if`" when an if-expression is used as a statement
        // inside a block without a trailing ';', and another statement follows immediately.
        // Also catches "Expected one of \"else\", \";\" or \"}\"" when any identifier follows an if-block.
        triggers: &["Unrecognized token `if`", "token `if` found", "Expected one of \"else\", \";\" or \"}\""],
        suggestion: "In BMB, if-expressions used as statements inside blocks must be followed by ';' before the next statement. Add ';' after each if-statement that is not the last expression in a block.",
        example_wrong: "fn f(n: i64) -> i64 = {\n    if n < 2 { return 0 }\n    if n == 2 { return 1 }\n    n - 1\n};",
        example_correct: "fn f(n: i64) -> i64 = {\n    if n < 2 { return 0 };\n    if n == 2 { return 1 };\n    n - 1\n};",
    },
    DiagPattern {
        id: "contract_param_undefined",
        kind: "type",
        // Fires when "undefined variable" appears — most commonly when the model puts a contract
        // (pre/post) on fn main() which has no parameters, causing "undefined variable: `n`".
        // BMB contract variables must be function parameters or the special `ret` (postcondition).
        triggers: &["undefined variable"],
        suggestion: "In BMB, pre/post contracts can only reference function parameters (and `ret` in post). If you see 'undefined variable' inside a contract, the variable is not a parameter of that function.\nCOMMON MISTAKE: Do NOT put contracts on fn main() — main() has no parameters, so any name in main's pre/post will be undefined.\nFix: Move the contract to the helper function that receives the parameter.\nExample: fn factorial(n: i64) -> i64 pre n >= 0 and n <= 20 post ret >= 1 = ...;",
        example_wrong: "fn main() -> i64\n    pre n >= 0\n= { ... };  // n is not a parameter of main",
        example_correct: "fn factorial(n: i64) -> i64\n    pre n >= 0\n= ...;  // n is a parameter here",
    },
];
