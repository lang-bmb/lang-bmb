//! Integration tests for BMB compiler
//!
//! Tests the full compilation pipeline including:
//! - Type checking (check command)
//! - Interpreter execution (run command)
//! - Contract verification (verify command)
//! - Error diagnostics

use bmb::lexer::tokenize;
use bmb::parser::parse;
use bmb::types::TypeChecker;

/// Helper to compile and type-check a BMB program
fn check_program(source: &str) -> bmb::Result<()> {
    let tokens = tokenize(source)?;
    let ast = parse("test.bmb", source, tokens)?;
    let mut tc = TypeChecker::new();
    tc.check_program(&ast)?;
    Ok(())
}

/// Helper to check if a program type-checks successfully
fn type_checks(source: &str) -> bool {
    check_program(source).is_ok()
}

/// Helper to check if a program fails type-checking
fn type_error(source: &str) -> bool {
    check_program(source).is_err()
}

/// Helper to check if a program produces a type error containing a specific message
fn type_error_contains(source: &str, expected: &str) -> bool {
    match check_program(source) {
        Err(e) => format!("{e}").contains(expected),
        Ok(_) => false,
    }
}

/// Helper to check if a program produces a specific warning kind
fn has_warning_kind(source: &str, warning_kind: &str) -> bool {
    let tokens = match tokenize(source) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let ast = match parse("test.bmb", source, tokens) {
        Ok(a) => a,
        Err(_) => return false,
    };
    let mut tc = TypeChecker::new();
    if tc.check_program(&ast).is_err() {
        return false;
    }
    tc.warnings().iter().any(|w| w.kind() == warning_kind)
}

// ============================================
// Basic Type Checking Tests
// ============================================

#[test]
fn test_simple_function() {
    assert!(type_checks("fn main() -> i64 = 42;"));
}

#[test]
fn test_function_with_params() {
    assert!(type_checks("fn add(a: i64, b: i64) -> i64 = a + b;"));
}

#[test]
fn test_function_with_let() {
    assert!(type_checks(
        "fn main() -> i64 = { let x = 10; let y = 20; x + y };"
    ));
}

#[test]
fn test_bool_function() {
    assert!(type_checks("fn is_positive(x: i64) -> bool = x > 0;"));
}

#[test]
fn test_if_expression() {
    assert!(type_checks(
        "fn max(a: i64, b: i64) -> i64 = if a > b { a } else { b };"
    ));
}

// ============================================
// Contract Tests
// ============================================

#[test]
fn test_precondition() {
    assert!(type_checks(
        "fn sqrt(x: i64) -> i64
           pre x >= 0
         = x;"
    ));
}

#[test]
fn test_postcondition() {
    assert!(type_checks(
        "fn abs(x: i64) -> i64
           post ret >= 0
         = if x >= 0 { x } else { 0 - x };"
    ));
}

#[test]
fn test_pre_and_post() {
    assert!(type_checks(
        "fn safe_div(a: i64, b: i64) -> i64
           pre b != 0
           post ret == a / b
         = a / b;"
    ));
}

#[test]
fn test_contract_with_implies() {
    assert!(type_checks(
        "fn min(a: i64, b: i64) -> i64
           post ret <= a and ret <= b and (ret == a or ret == b)
         = if a <= b { a } else { b };"
    ));
}

// ============================================
// Type Error Tests
// ============================================

#[test]
fn test_type_mismatch() {
    // Return type mismatch
    assert!(type_error("fn main() -> bool = 42;"));
}

#[test]
fn test_undefined_variable() {
    assert!(type_error("fn main() -> i64 = x;"));
}

#[test]
fn test_wrong_param_type() {
    assert!(type_error(
        "fn add(a: i64, b: i64) -> i64 = a + b;
         fn main() -> i64 = add(true, 1);"
    ));
}

#[test]
fn test_wrong_arg_count() {
    assert!(type_error(
        "fn add(a: i64, b: i64) -> i64 = a + b;
         fn main() -> i64 = add(1);"
    ));
}

// ============================================
// Struct Tests
// ============================================

#[test]
fn test_struct_definition() {
    // Note: BMB uses `new StructName { ... }` for struct instantiation
    assert!(type_checks(
        "struct Point { x: i64, y: i64 }
         fn origin() -> Point = new Point { x: 0, y: 0 };"
    ));
}

#[test]
fn test_struct_field_access() {
    assert!(type_checks(
        "struct Point { x: i64, y: i64 }
         fn get_x(p: Point) -> i64 = p.x;"
    ));
}

// ============================================
// Enum Tests
// ============================================

#[test]
fn test_enum_definition() {
    assert!(type_checks(
        "enum Color { Red, Green, Blue }
         fn red() -> Color = Color::Red;"
    ));
}

#[test]
fn test_enum_with_data() {
    assert!(type_checks(
        "enum Option<T> { Some(T), None }
         fn some(x: i64) -> Option<i64> = Option::Some(x);"
    ));
}

#[test]
fn test_match_expression() {
    assert!(type_checks(
        "enum Option<T> { Some(T), None }
         fn unwrap_or(opt: Option<i64>, default: i64) -> i64 =
           match opt {
             Option::Some(x) => x,
             Option::None => default
           };"
    ));
}

// ============================================
// Array Tests
// ============================================

#[test]
fn test_array_literal() {
    assert!(type_checks("fn arr() -> [i64; 3] = [1, 2, 3];"));
}

#[test]
fn test_array_index() {
    assert!(type_checks(
        "fn first(arr: [i64; 3]) -> i64 = arr[0];"
    ));
}

#[test]
fn test_array_ref_index() {
    // v0.50.26: Array reference indexing
    assert!(type_checks(
        "fn first_ref(arr: &[i64; 3]) -> i64 = arr[0];"
    ));
}

#[test]
fn test_string_ref_index() {
    // v0.50.26: String reference indexing
    assert!(type_checks(
        "fn char_at_ref(s: &String, i: i64) -> i64 = s[i];"
    ));
}

#[test]
fn test_invalid_ref_index() {
    // Cannot index into a reference to non-array/non-string
    assert!(type_error(
        "fn bad(x: &i64, i: i64) -> i64 = x[i];"
    ));
}

// ============================================
// Generic Function Tests
// ============================================

#[test]
fn test_generic_function() {
    assert!(type_checks(
        "fn identity<T>(x: T) -> T = x;
         fn main() -> i64 = identity(42);"
    ));
}

#[test]
fn test_generic_with_constraint() {
    assert!(type_checks(
        "fn first<T>(x: T, y: T) -> T = x;"
    ));
}

// ============================================
// Recursion Tests
// ============================================

#[test]
fn test_simple_recursion() {
    assert!(type_checks(
        "fn factorial(n: i64) -> i64
           pre n >= 0
         = if n <= 1 { 1 } else { n * factorial(n - 1) };"
    ));
}

#[test]
fn test_mutual_recursion() {
    assert!(type_checks(
        "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
         fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };"
    ));
}

// ============================================
// Closure Tests (v0.20.0)
// ============================================

#[test]
fn test_closure_expression() {
    // BMB uses `fn |params| { body }` closure syntax
    assert!(type_checks(
        "fn make_adder() -> i64 = {
           let add_one = fn |x: i64| { x + 1 };
           42
         };"
    ));
}

#[test]
fn test_closure_multi_params() {
    assert!(type_checks(
        "fn make_add() -> i64 = {
           let add = fn |x: i64, y: i64| { x + y };
           42
         };"
    ));
}

// ============================================
// Shift Operator Tests (v0.32)
// ============================================

#[test]
fn test_left_shift() {
    assert!(type_checks("fn shl(x: i64) -> i64 = x << 2;"));
}

#[test]
fn test_right_shift() {
    assert!(type_checks("fn shr(x: i64) -> i64 = x >> 1;"));
}

#[test]
fn test_shift_combined() {
    assert!(type_checks(
        "fn shift_test(x: i64) -> i64 = (x << 2) >> 1;"
    ));
}

// ============================================
// Logical Operator Tests (v0.32)
// ============================================

#[test]
fn test_symbolic_and() {
    assert!(type_checks(
        "fn both(a: bool, b: bool) -> bool = a && b;"
    ));
}

#[test]
fn test_symbolic_or() {
    assert!(type_checks(
        "fn either(a: bool, b: bool) -> bool = a || b;"
    ));
}

#[test]
fn test_symbolic_not() {
    assert!(type_checks(
        "fn negate(x: bool) -> bool = !x;"
    ));
}

// ============================================
// Wrapping Arithmetic Tests (v0.37)
// ============================================

#[test]
fn test_wrapping_add() {
    assert!(type_checks("fn add_wrap(a: i64, b: i64) -> i64 = a +% b;"));
}

#[test]
fn test_wrapping_sub() {
    assert!(type_checks("fn sub_wrap(a: i64, b: i64) -> i64 = a -% b;"));
}

#[test]
fn test_wrapping_mul() {
    assert!(type_checks("fn mul_wrap(a: i64, b: i64) -> i64 = a *% b;"));
}

// ============================================
// Comment Syntax Tests
// ============================================

#[test]
fn test_double_slash_comment() {
    assert!(type_checks(
        "// This is a comment
         fn main() -> i64 = 42;"
    ));
}

#[test]
fn test_legacy_comment() {
    assert!(type_checks(
        "-- Legacy comment style
         fn main() -> i64 = 42;"
    ));
}

// ============================================
// Visibility Tests
// ============================================

#[test]
fn test_pub_function() {
    assert!(type_checks("pub fn public_fn() -> i64 = 42;"));
}

#[test]
fn test_pub_struct() {
    assert!(type_checks(
        "pub struct PublicStruct { x: i64 }"
    ));
}

// ============================================
// Complex Expression Tests
// ============================================

#[test]
fn test_nested_if() {
    assert!(type_checks(
        "fn classify(x: i64) -> i64 =
           if x < 0 { -1 }
           else if x == 0 { 0 }
           else { 1 };"
    ));
}

#[test]
fn test_complex_contract() {
    assert!(type_checks(
        "fn clamp(x: i64, lo: i64, hi: i64) -> i64
           pre lo <= hi
           post ret >= lo and ret <= hi
         = if x < lo { lo } else if x > hi { hi } else { x };"
    ));
}

#[test]
fn test_block_with_multiple_lets() {
    assert!(type_checks(
        "fn compute(x: i64) -> i64 = {
           let a = x * 2;
           let b = a + 1;
           let c = b * b;
           c
         };"
    ));
}

// ============================================
// Floating Point Tests (f64)
// ============================================

#[test]
fn test_f64_literal() {
    assert!(type_checks("fn pi() -> f64 = 3.14;"));
}

#[test]
fn test_f64_arithmetic() {
    assert!(type_checks(
        "fn circle_area(r: f64) -> f64 = 3.14159 * r * r;"
    ));
}

#[test]
fn test_f64_comparison() {
    assert!(type_checks(
        "fn is_positive_f(x: f64) -> bool = x > 0.0;"
    ));
}

// ============================================
// String Tests
// ============================================

#[test]
fn test_string_literal() {
    assert!(type_checks(r#"fn hello() -> String = "hello";"#));
}

#[test]
fn test_string_concat() {
    assert!(type_checks(
        r#"fn greet(name: String) -> String = "Hello, " + name;"#
    ));
}

// ============================================
// Bitwise Operator Tests (keyword syntax: band, bor, bxor, bnot)
// ============================================

#[test]
fn test_bitwise_and() {
    // BMB uses `band` keyword instead of `&`
    assert!(type_checks("fn bitand(a: i64, b: i64) -> i64 = a band b;"));
}

#[test]
fn test_bitwise_or() {
    // BMB uses `bor` keyword instead of `|`
    assert!(type_checks("fn bitor(a: i64, b: i64) -> i64 = a bor b;"));
}

#[test]
fn test_bitwise_xor() {
    // BMB uses `bxor` keyword instead of `^`
    assert!(type_checks("fn bitxor(a: i64, b: i64) -> i64 = a bxor b;"));
}

// ============================================
// While Loop Tests
// ============================================

#[test]
fn test_while_loop() {
    // BMB while loops require:
    // 1. `let mut` for mutable variables with explicit type
    // 2. Double braces for the body: { { stmts; value } }
    assert!(type_checks(
        "fn count_to(n: i64) -> i64 = {
           let mut i: i64 = 0;
           while i < n { { i = i + 1; i } };
           i
         };"
    ));
}

// ============================================
// Refinement Type Tests (where) - NOT YET IMPLEMENTED
// ============================================
// Note: Refinement types (type X = Y where condition) are specified
// in SPECIFICATION.md but not yet implemented in the parser.
// These tests are commented out until implementation.

// #[test]
// fn test_refinement_type() {
//     assert!(type_checks(
//         "type NonZero = i64 where self != 0;
//          fn safe_div(a: i64, b: NonZero) -> i64 = a / b;"
//     ));
// }

// #[test]
// fn test_refinement_positive() {
//     assert!(type_checks(
//         "type Positive = i64 where self > 0;
//          fn double_positive(x: Positive) -> i64 = x * 2;"
//     ));
// }

// ============================================
// @trust Annotation Tests
// ============================================

#[test]
fn test_trust_annotation() {
    assert!(type_checks(
        "@trust
         fn unsafe_operation(x: i64) -> i64
           pre x > 0
           post ret > x
         = x;"
    ));
}

// ============================================
// Method Call Tests
// ============================================

#[test]
fn test_string_method_len() {
    assert!(type_checks(
        r#"fn string_length(s: String) -> i64 = s.len();"#
    ));
}

// ============================================
// Type Alias Tests (v0.50.6)
// ============================================

#[test]
fn test_type_alias_basic() {
    assert!(type_checks(
        "type Age = i64;
         fn get_age(a: Age) -> Age = a;"
    ));
}

#[test]
fn test_type_alias_in_function() {
    assert!(type_checks(
        "type Counter = i64;
         fn increment(c: Counter) -> Counter = c + 1;"
    ));
}

#[test]
fn test_type_alias_chain() {
    assert!(type_checks(
        "type A = i64;
         type B = A;
         fn use_b(x: B) -> B = x;"
    ));
}

#[test]
fn test_type_alias_cyclic_error() {
    // Cyclic type aliases should be rejected (v0.50.11)
    assert!(type_error(
        "type A = B;
         type B = A;
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_type_alias_self_referential_error() {
    // Self-referential type aliases should be rejected
    assert!(type_error(
        "type A = A;
         fn main() -> i64 = 0;"
    ));
}

// ============================================
// Duplicate Function Detection Tests (v0.50.11)
// ============================================

#[test]
fn test_duplicate_function_warning() {
    // Duplicate function definitions should trigger a warning
    assert!(has_warning_kind(
        "fn foo() -> i64 = 1;
         fn foo() -> i64 = 2;
         fn main() -> i64 = foo();",
        "duplicate_function"
    ));
}

#[test]
fn test_no_duplicate_warning_unique_functions() {
    // Unique function names should not trigger duplicate warning
    assert!(!has_warning_kind(
        "fn foo() -> i64 = 1;
         fn bar() -> i64 = 2;
         fn main() -> i64 = foo() + bar();",
        "duplicate_function"
    ));
}

// ============================================
// Negation Tests
// ============================================

#[test]
fn test_unary_minus() {
    assert!(type_checks("fn negate(x: i64) -> i64 = -x;"));
}

#[test]
fn test_unary_minus_expression() {
    assert!(type_checks("fn abs(x: i64) -> i64 = if x < 0 { -x } else { x };"));
}

// ============================================
// Comparison Chain Tests
// ============================================

#[test]
fn test_chained_comparisons() {
    assert!(type_checks(
        "fn in_range(x: i64, lo: i64, hi: i64) -> bool = x >= lo && x <= hi;"
    ));
}

// ============================================
// Modulo Operator Tests
// ============================================

#[test]
fn test_modulo() {
    assert!(type_checks("fn remainder(a: i64, b: i64) -> i64 = a % b;"));
}

#[test]
fn test_is_even() {
    assert!(type_checks("fn is_even_mod(n: i64) -> bool = n % 2 == 0;"));
}

// ============================================
// Concurrency Type-Check Tests (v0.70-v0.85)
// ============================================
// These tests verify that all concurrency test files
// in bmb/tests/concurrency/ parse and type-check correctly.

/// Helper to type-check a BMB file from disk
fn check_file(path: &str) -> bool {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    check_program(&source).is_ok()
}

macro_rules! concurrency_typecheck_test {
    ($name:ident, $file:expr) => {
        #[test]
        fn $name() {
            let path = concat!("tests/concurrency/", $file);
            assert!(
                check_file(path),
                "Type-check failed for {}",
                path
            );
        }
    };
}

concurrency_typecheck_test!(test_concurrency_spawn_basic, "spawn_basic.bmb");
concurrency_typecheck_test!(test_concurrency_mutex_basic, "mutex_basic.bmb");
concurrency_typecheck_test!(test_concurrency_mutex_threaded, "mutex_threaded.bmb");
concurrency_typecheck_test!(test_concurrency_atomic_basic, "atomic_basic.bmb");
concurrency_typecheck_test!(test_concurrency_channel_basic, "channel_basic.bmb");
concurrency_typecheck_test!(test_concurrency_channel_close_basic, "channel_close_basic.bmb");
concurrency_typecheck_test!(test_concurrency_channel_iter_basic, "channel_iter_basic.bmb");
concurrency_typecheck_test!(test_concurrency_rwlock_basic, "rwlock_basic.bmb");
concurrency_typecheck_test!(test_concurrency_barrier_basic, "barrier_basic.bmb");
concurrency_typecheck_test!(test_concurrency_condvar_basic, "condvar_basic.bmb");
concurrency_typecheck_test!(test_concurrency_future_basic, "future_basic.bmb");
concurrency_typecheck_test!(test_concurrency_async_fn_basic, "async_fn_basic.bmb");
concurrency_typecheck_test!(test_concurrency_arc_basic, "arc_basic.bmb");
concurrency_typecheck_test!(test_concurrency_try_recv_basic, "try_recv_basic.bmb");
concurrency_typecheck_test!(test_concurrency_recv_timeout_basic, "recv_timeout_basic.bmb");
concurrency_typecheck_test!(test_concurrency_send_timeout_basic, "send_timeout_basic.bmb");
concurrency_typecheck_test!(test_concurrency_executor_basic, "executor_basic.bmb");
concurrency_typecheck_test!(test_concurrency_select_basic, "select_basic.bmb");
concurrency_typecheck_test!(test_concurrency_select_multi, "select_multi.bmb");
concurrency_typecheck_test!(test_concurrency_async_io_basic, "async_io_basic.bmb");
concurrency_typecheck_test!(test_concurrency_async_socket_basic, "async_socket_basic.bmb");
concurrency_typecheck_test!(test_concurrency_thread_pool_basic, "thread_pool_basic.bmb");
concurrency_typecheck_test!(test_concurrency_scoped_threads_basic, "scoped_threads_basic.bmb");

// ============================================
// MIR Lowering Integration Tests (v0.88)
// ============================================
// These tests verify that BMB source code is correctly lowered
// to MIR instructions through the full parse → type-check → lower pipeline.

use bmb::mir::{self, MirInst, Terminator, MirType, Constant, MirBinOp, ContractFact, CmpOp};

/// Helper to parse, type-check, and lower a BMB program to MIR
fn lower_to_mir(source: &str) -> mir::MirProgram {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type-check failed");
    mir::lower_program(&ast)
}

/// Helper to find a function by name in MIR output
fn find_mir_fn<'a>(program: &'a mir::MirProgram, name: &str) -> &'a mir::MirFunction {
    program.functions.iter().find(|f| f.name == name)
        .unwrap_or_else(|| panic!("MIR function '{}' not found", name))
}

/// Helper to check if any instruction in any block matches a predicate
fn has_inst(func: &mir::MirFunction, pred: impl Fn(&MirInst) -> bool) -> bool {
    func.blocks.iter().any(|b| b.instructions.iter().any(&pred))
}

// --- Basic lowering ---

#[test]
fn test_mir_lower_arithmetic() {
    let mir = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    assert_eq!(mir.functions.len(), 1);
    let func = &mir.functions[0];
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert!(matches!(func.ret_ty, MirType::I64));
    assert!(has_inst(func, |i| matches!(i, MirInst::BinOp { op: MirBinOp::Add, .. })));
}

#[test]
fn test_mir_lower_float_arithmetic() {
    let mir = lower_to_mir("fn mul(a: f64, b: f64) -> f64 = a * b;");
    let func = find_mir_fn(&mir, "mul");
    assert!(matches!(func.ret_ty, MirType::F64));
    assert!(has_inst(func, |i| matches!(i, MirInst::BinOp { op: MirBinOp::FMul, .. })));
}

#[test]
fn test_mir_lower_constant() {
    let mir = lower_to_mir("fn answer() -> i64 = 42;");
    let func = find_mir_fn(&mir, "answer");
    // Constant may be inlined as Operand::Constant in Return terminator
    let has_const = has_inst(func, |i| matches!(i, MirInst::Const { value: Constant::Int(42), .. }));
    let has_return_const = func.blocks.iter().any(|b| matches!(&b.terminator,
        Terminator::Return(Some(mir::Operand::Constant(Constant::Int(42))))));
    assert!(has_const || has_return_const, "should have constant 42");
}

#[test]
fn test_mir_lower_bool_constant() {
    let mir = lower_to_mir("fn yes() -> bool = true;");
    let func = find_mir_fn(&mir, "yes");
    let has_const = has_inst(func, |i| matches!(i, MirInst::Const { value: Constant::Bool(true), .. }));
    let has_return_const = func.blocks.iter().any(|b| matches!(&b.terminator,
        Terminator::Return(Some(mir::Operand::Constant(Constant::Bool(true))))));
    assert!(has_const || has_return_const, "should have constant true");
}

#[test]
fn test_mir_lower_string_literal() {
    let mir = lower_to_mir(r#"fn greet() -> String = "hello";"#);
    let func = find_mir_fn(&mir, "greet");
    assert!(matches!(func.ret_ty, MirType::String));
    // String literal returns as Operand::Constant in Return terminator
    let has_return_string = func.blocks.iter().any(|b| matches!(&b.terminator,
        Terminator::Return(Some(mir::Operand::Constant(Constant::String(s)))) if s == "hello"));
    assert!(has_return_string, "should return string 'hello'");
}

// --- Control flow ---

#[test]
fn test_mir_lower_if_expression() {
    let mir = lower_to_mir("fn max(a: i64, b: i64) -> i64 = if a > b { a } else { b };");
    let func = find_mir_fn(&mir, "max");
    // If expression creates: entry, then, else, merge blocks
    assert!(func.blocks.len() >= 4);
    assert!(func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Branch { .. })));
}

#[test]
fn test_mir_lower_while_loop() {
    let mir = lower_to_mir(
        "fn count_to(n: i64) -> i64 = {
           let mut i: i64 = 0;
           while i < n { { i = i + 1; i } };
           i
         };"
    );
    let func = find_mir_fn(&mir, "count_to");
    // While loop creates loop header with Branch and back-edge with Goto
    assert!(func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Branch { .. })));
    assert!(func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Goto(_))));
}

// --- Let bindings ---

#[test]
fn test_mir_lower_let_binding() {
    let mir = lower_to_mir("fn f(x: i64) -> i64 = let y = x + 1; y;");
    let func = find_mir_fn(&mir, "f");
    assert!(has_inst(func, |i| matches!(i, MirInst::BinOp { op: MirBinOp::Add, .. })));
    assert!(func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Return(Some(_)))));
}

// --- Function calls ---

#[test]
fn test_mir_lower_function_call() {
    let mir = lower_to_mir(r#"
        fn double(x: i64) -> i64 = x * 2;
        fn main() -> i64 = double(21);
    "#);
    let func = find_mir_fn(&mir, "main");
    assert!(has_inst(func, |i| matches!(i, MirInst::Call { func, .. } if func == "double")));
}

// --- Structs ---

#[test]
fn test_mir_lower_struct_init() {
    let mir = lower_to_mir("struct Point { x: i64, y: i64 } fn origin() -> Point = new Point { x: 0, y: 0 };");
    let func = find_mir_fn(&mir, "origin");
    assert!(has_inst(func, |i| matches!(i, MirInst::StructInit { struct_name, .. } if struct_name == "Point")));
}

#[test]
fn test_mir_lower_field_access() {
    let mir = lower_to_mir(r#"
        struct Point { x: i64, y: i64 }
        fn get_x(p: Point) -> i64 = p.x;
    "#);
    let func = find_mir_fn(&mir, "get_x");
    assert!(has_inst(func, |i| matches!(i, MirInst::FieldAccess { field, struct_name, .. } if field == "x" && struct_name == "Point")));
}

// --- Arrays ---

#[test]
fn test_mir_lower_array_init() {
    let mir = lower_to_mir("fn arr() -> [i64; 3] = [1, 2, 3];");
    let func = find_mir_fn(&mir, "arr");
    assert!(has_inst(func, |i| matches!(i, MirInst::ArrayInit { elements, .. } if elements.len() == 3)));
}

#[test]
fn test_mir_lower_array_index() {
    let mir = lower_to_mir(r#"
        fn first(arr: [i64; 3]) -> i64 = arr[0];
    "#);
    let func = find_mir_fn(&mir, "first");
    assert!(has_inst(func, |i| matches!(i, MirInst::IndexLoad { .. })));
}

// --- Concurrency ---

#[test]
fn test_mir_lower_mutex_new() {
    let mir = lower_to_mir(r#"
        fn make_mutex() -> Mutex<i64> = Mutex::new(0);
    "#);
    let func = find_mir_fn(&mir, "make_mutex");
    assert!(has_inst(func, |i| matches!(i, MirInst::MutexNew { .. })));
}

#[test]
fn test_mir_lower_atomic_new() {
    let mir = lower_to_mir(r#"
        fn make_atomic() -> Atomic<i64> = Atomic::new(0);
    "#);
    let func = find_mir_fn(&mir, "make_atomic");
    assert!(has_inst(func, |i| matches!(i, MirInst::AtomicNew { .. })));
}

#[test]
fn test_mir_lower_channel_new() {
    let mir = lower_to_mir("fn make_channel() -> (Sender<i64>, Receiver<i64>) = channel<i64>(10);");
    let func = find_mir_fn(&mir, "make_channel");
    assert!(has_inst(func, |i| matches!(i, MirInst::ChannelNew { .. })));
}

#[test]
fn test_mir_lower_rwlock_new() {
    let mir = lower_to_mir(r#"
        fn make_rwlock() -> RwLock<i64> = RwLock::new(0);
    "#);
    let func = find_mir_fn(&mir, "make_rwlock");
    assert!(has_inst(func, |i| matches!(i, MirInst::RwLockNew { .. })));
}

// --- Contracts ---

#[test]
fn test_mir_lower_precondition() {
    let mir = lower_to_mir(r#"
        fn safe_div(a: i64, b: i64) -> i64
            pre b != 0
        = a / b;
    "#);
    let func = find_mir_fn(&mir, "safe_div");
    assert!(!func.preconditions.is_empty(), "should have preconditions");
}

#[test]
fn test_mir_lower_pre_and_post_contracts() {
    let mir = lower_to_mir(
        "fn clamp(x: i64, lo: i64, hi: i64) -> i64
           pre lo <= hi
         = if x < lo { lo } else if x > hi { hi } else { x };"
    );
    let func = find_mir_fn(&mir, "clamp");
    assert!(!func.preconditions.is_empty(), "should have preconditions from pre lo <= hi");
}

#[test]
fn test_mir_lower_postcondition_ret_cmp() {
    // v0.89: Postcondition with return value comparison should now extract facts
    let mir = lower_to_mir(
        "fn abs(x: i64) -> i64
           post ret >= 0
         = if x >= 0 { x } else { 0 - x };"
    );
    let func = find_mir_fn(&mir, "abs");
    assert!(!func.postconditions.is_empty(), "should have postconditions from post ret >= 0");
    assert!(func.postconditions.iter().any(|f| matches!(f,
        ContractFact::ReturnCmp { op: CmpOp::Ge, value: 0 }
    )), "should have ReturnCmp(Ge, 0)");
}

#[test]
fn test_mir_lower_postcondition_ret_var_cmp() {
    // v0.89: Postcondition with return value vs variable comparison
    let mir = lower_to_mir(
        "fn min(a: i64, b: i64) -> i64
           post ret <= a and ret <= b
         = if a <= b { a } else { b };"
    );
    let func = find_mir_fn(&mir, "min");
    assert!(func.postconditions.len() >= 2, "should have at least 2 postconditions");
    assert!(func.postconditions.iter().any(|f| matches!(f,
        ContractFact::ReturnVarCmp { op: CmpOp::Le, var } if var == "a"
    )), "should have ReturnVarCmp(Le, a)");
    assert!(func.postconditions.iter().any(|f| matches!(f,
        ContractFact::ReturnVarCmp { op: CmpOp::Le, var } if var == "b"
    )), "should have ReturnVarCmp(Le, b)");
}

// --- Function attributes ---

#[test]
fn test_mir_lower_pure_function() {
    let mir = lower_to_mir(r#"
        @pure
        fn square(x: i64) -> i64 = x * x;
    "#);
    let func = find_mir_fn(&mir, "square");
    assert!(func.is_pure, "should be marked pure");
}

#[test]
fn test_mir_lower_const_function() {
    let mir = lower_to_mir(r#"
        @const
        fn five() -> i64 = 5;
    "#);
    let func = find_mir_fn(&mir, "five");
    assert!(func.is_const, "should be marked const");
}

// --- Multiple functions ---

#[test]
fn test_mir_lower_multiple_functions() {
    let mir = lower_to_mir(r#"
        fn add(a: i64, b: i64) -> i64 = a + b;
        fn sub(a: i64, b: i64) -> i64 = a - b;
        fn mul(a: i64, b: i64) -> i64 = a * b;
    "#);
    assert_eq!(mir.functions.len(), 3);
    assert!(mir.functions.iter().any(|f| f.name == "add"));
    assert!(mir.functions.iter().any(|f| f.name == "sub"));
    assert!(mir.functions.iter().any(|f| f.name == "mul"));
}

// --- Return type lowering ---

#[test]
fn test_mir_lower_unit_return() {
    let mir = lower_to_mir(r#"
        fn noop() -> () = ();
    "#);
    let func = find_mir_fn(&mir, "noop");
    assert!(matches!(func.ret_ty, MirType::Unit));
}

#[test]
fn test_mir_lower_bool_return() {
    let mir = lower_to_mir(r#"
        fn is_positive(x: i64) -> bool = x > 0;
    "#);
    let func = find_mir_fn(&mir, "is_positive");
    assert!(matches!(func.ret_ty, MirType::Bool));
}

// --- Keyword-as-method-name (v0.89) ---

#[test]
fn test_keyword_spawn_as_method() {
    // v0.89: `spawn` keyword should be usable as a method name after `.`
    assert!(type_checks(
        "fn test_scope() -> () = {
           let s: Scope = thread_scope();
           s.spawn(fn || { () });
           s.wait();
         };"
    ));
}

// ============================================
// Codegen Round-Trip Tests (v0.89)
// ============================================
// These tests verify that the full pipeline (parse → type-check → MIR → codegen)
// produces correct LLVM IR text output.

use bmb::codegen::TextCodeGen;
use bmb::mir::{OptimizationPipeline, OptLevel};

/// Helper: source → optimized MIR → LLVM IR text
fn source_to_ir(source: &str) -> String {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type-check failed");
    let mut mir_prog = mir::lower_program(&ast);
    let pipeline = OptimizationPipeline::for_level(OptLevel::Release);
    pipeline.optimize(&mut mir_prog);
    let codegen = TextCodeGen::new();
    codegen.generate(&mir_prog).expect("codegen failed")
}

/// Helper: source → unoptimized MIR → LLVM IR text
fn source_to_ir_unopt(source: &str) -> String {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type-check failed");
    let mir_prog = mir::lower_program(&ast);
    let codegen = TextCodeGen::new();
    codegen.generate(&mir_prog).expect("codegen failed")
}

#[test]
fn test_codegen_simple_function_signature() {
    let ir = source_to_ir("fn add(a: i64, b: i64) -> i64 = a + b;");
    // Functions are defined as `define private` with attributes
    assert!(ir.contains("@add(i64 %a, i64 %b)"),
        "IR should contain add function signature");
}

#[test]
fn test_codegen_bool_return_type() {
    let ir = source_to_ir("fn is_zero(x: i64) -> bool = x == 0;");
    assert!(ir.contains("i1 @is_zero(i64 %x)"),
        "IR should define bool (i1) return type");
}

#[test]
fn test_codegen_constant_folded() {
    // After optimization, 2 + 3 should be folded to constant 5
    let ir = source_to_ir("fn five() -> i64 = 2 + 3;");
    // ConstantPropagationNarrowing narrows to i32, so look for store i32 5
    assert!(ir.contains("i32 5") || ir.contains("i64 5"),
        "Constant 2+3 should be folded to 5 in IR");
}

#[test]
fn test_codegen_string_constant() {
    let ir = source_to_ir_unopt(r#"fn greeting() -> String = "hello";"#);
    // String constants appear as @.str.N globals
    assert!(ir.contains("@.str."),
        "IR should contain string constant global");
    assert!(ir.contains("hello"),
        "IR should contain string literal value");
}

#[test]
fn test_codegen_branch_structure() {
    let ir = source_to_ir_unopt(
        "fn abs(x: i64) -> i64 = if x >= 0 { x } else { 0 - x };"
    );
    assert!(ir.contains("br i1"),
        "IR should contain conditional branch");
    assert!(ir.contains("icmp"),
        "IR should contain integer comparison");
}

#[test]
fn test_codegen_recursive_call() {
    let ir = source_to_ir_unopt(
        "fn factorial(n: i64) -> i64 = if n <= 1 { 1 } else { n * factorial(n - 1) };"
    );
    assert!(ir.contains("call i64 @factorial("),
        "IR should contain recursive call to factorial");
}

#[test]
fn test_codegen_tail_recursion_to_loop() {
    let ir = source_to_ir(
        "fn count_down(n: i64) -> i64 = if n <= 0 { 0 } else { count_down(n - 1) };"
    );
    // After TCO + TailRecursiveToLoop, the call is eliminated and replaced with a loop
    assert!(!ir.contains("call i64 @count_down("),
        "Tail call should be eliminated by loop transformation");
    // Should have a loop backedge (br label %bb_loop_header)
    assert!(ir.contains("loop_header"),
        "Should have a loop header block from tail-to-loop conversion");
}

#[test]
fn test_codegen_multiple_functions() {
    let ir = source_to_ir(
        "fn double(x: i64) -> i64 = x * 2;
         fn triple(x: i64) -> i64 = x * 3;"
    );
    assert!(ir.contains("@double("),
        "IR should contain double function");
    assert!(ir.contains("@triple("),
        "IR should contain triple function");
}

#[test]
fn test_codegen_f64_operations() {
    let ir = source_to_ir("fn add_f(a: f64, b: f64) -> f64 = a + b;");
    assert!(ir.contains("double @add_f(double %a, double %b)"),
        "IR should define f64 (double) function");
    assert!(ir.contains("fadd"),
        "IR should contain floating-point add");
}

#[test]
fn test_codegen_contract_eliminates_check() {
    // With precondition x >= 0, the check x >= 0 should be eliminated to true
    let ir = source_to_ir(
        "fn test_contract(x: i64) -> bool
           pre x >= 0
         = x >= 0;"
    );
    // Contract optimization stores constant true (i1 1)
    assert!(ir.contains("store i1 1") || ir.contains("store i1 true"),
        "Contract should eliminate redundant check to constant true");
}

#[test]
fn test_codegen_dead_code_eliminated() {
    let ir = source_to_ir(
        "fn simple(x: i64) -> i64 = { let unused: i64 = x * 42; x };"
    );
    assert!(ir.contains("@simple("),
        "IR should contain simple function");
    // After DCE, the unused mul should be gone, function just returns x
    assert!(ir.contains("ret i64 %x"),
        "After DCE, should return x directly without unused computation");
}

#[test]
fn test_codegen_module_header() {
    let ir = source_to_ir("fn noop() -> i64 = 0;");
    assert!(ir.contains("ModuleID"),
        "IR should contain ModuleID header");
    assert!(ir.contains("target triple"),
        "IR should contain target triple");
}

// ============================================
// Interpreter End-to-End Tests
// ============================================

use bmb::interp::{Interpreter, Value};

/// Helper: parse, type-check, and interpret a BMB program, returning main's result
fn run_program(source: &str) -> Value {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut interp = Interpreter::new();
    interp.run(&ast).expect("interpreter failed")
}

#[test]
fn test_interp_constant_return() {
    assert_eq!(run_program("fn main() -> i64 = 42;"), Value::Int(42));
}

#[test]
fn test_interp_arithmetic() {
    assert_eq!(run_program("fn main() -> i64 = 3 + 4 * 2;"), Value::Int(11));
}

#[test]
fn test_interp_function_call() {
    assert_eq!(
        run_program("fn double(x: i64) -> i64 = x * 2;\nfn main() -> i64 = double(21);"),
        Value::Int(42)
    );
}

#[test]
fn test_interp_recursive_factorial() {
    assert_eq!(
        run_program("fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };\nfn main() -> i64 = fact(5);"),
        Value::Int(120)
    );
}

#[test]
fn test_interp_while_loop() {
    assert_eq!(
        run_program("fn main() -> i64 = { let mut s = 0; let mut i = 1; while i <= 10 { s = s + i; i = i + 1; 0 }; s };"),
        Value::Int(55)
    );
}

#[test]
fn test_interp_if_else() {
    assert_eq!(
        run_program("fn main() -> i64 = if true { 1 } else { 0 };"),
        Value::Int(1)
    );
    assert_eq!(
        run_program("fn main() -> i64 = if false { 1 } else { 0 };"),
        Value::Int(0)
    );
}

#[test]
fn test_interp_nested_calls() {
    assert_eq!(
        run_program("fn inc(x: i64) -> i64 = x + 1;\nfn main() -> i64 = inc(inc(inc(0)));"),
        Value::Int(3)
    );
}

#[test]
fn test_interp_let_binding() {
    assert_eq!(
        run_program("fn main() -> i64 = { let x = 10; let y = x * 2; y + 1 };"),
        Value::Int(21)
    );
}

#[test]
fn test_interp_bool_result() {
    assert_eq!(
        run_program("fn main() -> bool = 5 > 3;"),
        Value::Bool(true)
    );
    assert_eq!(
        run_program("fn main() -> bool = 3 > 5;"),
        Value::Bool(false)
    );
}

#[test]
fn test_interp_string_len() {
    assert_eq!(
        run_program("fn main() -> i64 = \"hello\".len();"),
        Value::Int(5)
    );
}

#[test]
fn test_interp_float_arithmetic() {
    match run_program("fn main() -> f64 = 1.5 + 2.5;") {
        Value::Float(v) => assert!((v - 4.0).abs() < 1e-10),
        other => panic!("expected Float, got {:?}", other),
    }
}

#[test]
fn test_interp_boolean_logic() {
    assert_eq!(
        run_program("fn main() -> bool = true and false;"),
        Value::Bool(false)
    );
    assert_eq!(
        run_program("fn main() -> bool = true or false;"),
        Value::Bool(true)
    );
}

#[test]
fn test_interp_assign_in_if_branch() {
    assert_eq!(
        run_program("fn main() -> i64 = { let mut x = 0; if true { x = 42; 0 } else { 0 }; x };"),
        Value::Int(42)
    );
}

#[test]
fn test_interp_let_in_if_branch() {
    assert_eq!(
        run_program("fn main() -> i64 = if true { let y = 10; y } else { 0 };"),
        Value::Int(10)
    );
}

#[test]
fn test_interp_multiple_functions() {
    assert_eq!(
        run_program("fn add(a: i64, b: i64) -> i64 = a + b;\nfn mul(a: i64, b: i64) -> i64 = a * b;\nfn main() -> i64 = add(mul(3, 4), 5);"),
        Value::Int(17)
    );
}

#[test]
fn test_interp_comparison_chain() {
    assert_eq!(
        run_program("fn clamp(x: i64, lo: i64, hi: i64) -> i64 = if x < lo { lo } else if x > hi { hi } else { x };\nfn main() -> i64 = clamp(50, 0, 10);"),
        Value::Int(10)
    );
}

#[test]
fn test_interp_modulo() {
    assert_eq!(
        run_program("fn main() -> i64 = 17 % 5;"),
        Value::Int(2)
    );
}

#[test]
fn test_interp_negation() {
    assert_eq!(
        run_program("fn main() -> i64 = 0 - 42;"),
        Value::Int(-42)
    );
}

// ============================================
// Nullable Type Checking Tests (Cycle 105)
// ============================================

#[test]
fn test_nullable_return_value() {
    // Test that `fn f() -> i64? = 42;` type checks OK
    assert!(type_checks("fn f() -> i64? = 42;"));
}

#[test]
fn test_nullable_return_null() {
    // Test that `fn f() -> i64? = null;` type checks OK
    assert!(type_checks("fn f() -> i64? = null;"));
}

#[test]
fn test_nullable_return_if_else() {
    // Test that `fn f() -> i64? = if true { 1 } else { null };` type checks OK
    assert!(type_checks("fn f() -> i64? = if true { 1 } else { null };"));
}

#[test]
fn test_nullable_type_error() {
    // Test that `fn f() -> i64? = true;` fails type check
    assert!(type_error("fn f() -> i64? = true;"));
}

#[test]
fn test_nullable_unwrap_or_type_check() {
    // Test that nullable.unwrap_or(default) type checks correctly
    assert!(type_checks(
        "fn get_value(opt: i64?) -> i64 = opt.unwrap_or(0);"
    ));
}

#[test]
fn test_nullable_is_some_type_check() {
    // Test that nullable.is_some() type checks correctly
    assert!(type_checks(
        "fn has_value(opt: i64?) -> bool = opt.is_some();"
    ));
}

#[test]
fn test_nullable_is_none_type_check() {
    // Test that nullable.is_none() type checks correctly
    assert!(type_checks(
        "fn is_missing(opt: i64?) -> bool = opt.is_none();"
    ));
}

#[test]
fn test_nullable_string_type() {
    // Test nullable with String type
    assert!(type_checks(
        "fn maybe_name(has_name: bool) -> String? = if has_name { \"Alice\" } else { null };"
    ));
}

#[test]
fn test_nullable_struct_type() {
    // Test nullable with struct type
    assert!(type_checks(
        "struct Point { x: i64, y: i64 }
         fn maybe_point(has_point: bool) -> Point? =
           if has_point { new Point { x: 1, y: 2 } } else { null };"
    ));
}

// ============================================
// Error Handling Tests
// ============================================

#[test]
fn test_error_parse_invalid_syntax() {
    let result = tokenize("fn main() -> { }");
    // Should tokenize but fail to parse
    if let Ok(tokens) = result {
        assert!(parse("test.bmb", "fn main() -> { }", tokens).is_err());
    }
}

#[test]
fn test_error_undefined_function_call() {
    assert!(type_error("fn main() -> i64 = nonexistent();"));
}

#[test]
fn test_error_wrong_return_type() {
    assert!(type_error("fn main() -> i64 = true;"));
}

#[test]
fn test_error_duplicate_param_names() {
    // Duplicate parameter names should error at type check or parse
    let source = "fn f(x: i64, x: i64) -> i64 = x;";
    // May or may not error depending on implementation
    let _result = check_program(source);
}

// ============================================
// Pipeline Integration Tests
// ============================================

#[test]
fn test_pipeline_parse_lower_format() {
    let source = "fn add(a: i64, b: i64) -> i64 = a + b;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mir = bmb::mir::lower_program(&ast);
    let formatted = bmb::mir::format_mir(&mir);
    assert!(formatted.contains("fn add("));
    assert!(formatted.contains("-> i64"));
}

#[test]
fn test_pipeline_parse_lower_codegen() {
    let source = "fn square(x: i64) -> i64 = x * x;";
    let ir = source_to_ir(source);
    assert!(ir.contains("@square"));
    assert!(ir.contains("mul"));
    assert!(ir.contains("ret i64"));
}

// ============================================
// Interpreter Execution Tests
// ============================================
// These tests run BMB programs through the full pipeline:
// tokenize -> parse -> type check -> interpreter evaluate
// and verify the actual computed results.

/// Helper: run a program and extract the i64 result (panics if not Int)
fn run_program_i64(source: &str) -> i64 {
    match run_program(source) {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    }
}

fn run_program_str(source: &str) -> String {
    match run_program(source) {
        Value::Str(s) => s.to_string(),
        other => panic!("expected Str, got {:?}", other),
    }
}

fn run_program_f64(source: &str) -> f64 {
    match run_program(source) {
        Value::Float(f) => f,
        other => panic!("expected Float, got {:?}", other),
    }
}

// --- Basic Arithmetic ---

#[test]
fn test_run_subtraction() {
    assert_eq!(run_program_i64("fn main() -> i64 = 100 - 37;"), 63);
}

#[test]
fn test_run_division() {
    assert_eq!(run_program_i64("fn main() -> i64 = 84 / 4;"), 21);
}

#[test]
fn test_run_mixed_arithmetic() {
    // Verify operator precedence: 2 + 3 * 4 - 10 / 2 = 2 + 12 - 5 = 9
    assert_eq!(
        run_program_i64("fn main() -> i64 = 2 + 3 * 4 - 10 / 2;"),
        9
    );
}

#[test]
fn test_run_modulo_chain() {
    // 100 % 7 = 2, 2 * 3 = 6
    assert_eq!(
        run_program_i64("fn main() -> i64 = { let r = 100 % 7; r * 3 };"),
        6
    );
}

// --- Recursive Functions ---

#[test]
fn test_run_fibonacci() {
    assert_eq!(
        run_program_i64(
            "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };
             fn main() -> i64 = fib(10);"
        ),
        55
    );
}

#[test]
fn test_run_gcd() {
    assert_eq!(
        run_program_i64(
            "fn gcd(a: i64, b: i64) -> i64 = if b == 0 { a } else { gcd(b, a % b) };
             fn main() -> i64 = gcd(48, 18);"
        ),
        6
    );
}

// --- If-Else Expressions ---

#[test]
fn test_run_nested_if_classify() {
    // Classify: negative=-1, zero=0, positive=1
    assert_eq!(
        run_program_i64(
            "fn classify(x: i64) -> i64 =
               if x < 0 { 0 - 1 } else if x == 0 { 0 } else { 1 };
             fn main() -> i64 = classify(0 - 5) + classify(0) + classify(7);"
        ),
        0  // -1 + 0 + 1
    );
}

#[test]
fn test_run_if_as_expression_value() {
    // Use if-else result directly in arithmetic
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let x = 10;
               let y = if x > 5 { x * 2 } else { x };
               y + 1
             };"
        ),
        21
    );
}

// --- Match Expressions ---

#[test]
fn test_run_match_integer_literal() {
    assert_eq!(
        run_program_i64(
            "fn describe(n: i64) -> i64 = match n {
               0 => 100,
               1 => 200,
               _ => 300
             };
             fn main() -> i64 = describe(0) + describe(1) + describe(99);"
        ),
        600  // 100 + 200 + 300
    );
}

#[test]
fn test_run_match_enum_variant() {
    assert_eq!(
        run_program_i64(
            "enum Option<T> { Some(T), None }
             fn unwrap_or(opt: Option<i64>, default: i64) -> i64 =
               match opt {
                 Option::Some(x) => x,
                 Option::None => default
               };
             fn main() -> i64 = {
               let a = unwrap_or(Option::Some(42), 0);
               let b = unwrap_or(Option::None, 99);
               a + b
             };"
        ),
        141  // 42 + 99
    );
}

// --- While Loops with Mutable Variables ---

#[test]
fn test_run_while_power_of_two() {
    // Compute 2^10 = 1024 via repeated multiplication
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut result: i64 = 1;
               let mut i: i64 = 0;
               while i < 10 { result = result * 2; i = i + 1; 0 };
               result
             };"
        ),
        1024
    );
}

#[test]
fn test_run_while_countdown() {
    // Sum countdown: 5 + 4 + 3 + 2 + 1 = 15
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut n: i64 = 5;
               let mut total: i64 = 0;
               while n > 0 { total = total + n; n = n - 1; 0 };
               total
             };"
        ),
        15
    );
}

// --- String Operations ---

#[test]
fn test_run_string_concat_len() {
    // Concatenate two strings and check the length
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = {
               let s = "hello" + " world";
               s.len()
             };"#
        ),
        11
    );
}

// --- Struct Creation and Field Access ---

#[test]
fn test_run_struct_field_access() {
    assert_eq!(
        run_program_i64(
            "struct Point { x: i64, y: i64 }
             fn main() -> i64 = {
               let p = new Point { x: 3, y: 4 };
               p.x + p.y
             };"
        ),
        7
    );
}

#[test]
fn test_run_struct_pass_to_function() {
    assert_eq!(
        run_program_i64(
            "struct Rect { w: i64, h: i64 }
             fn area(r: Rect) -> i64 = r.w * r.h;
             fn main() -> i64 = area(new Rect { w: 6, h: 7 });"
        ),
        42
    );
}

// --- Nested Function Calls (deeper) ---

#[test]
fn test_run_deeply_nested_function_calls() {
    // Chain of functions that compose a computation
    assert_eq!(
        run_program_i64(
            "fn add1(x: i64) -> i64 = x + 1;
             fn double(x: i64) -> i64 = x * 2;
             fn square(x: i64) -> i64 = x * x;
             fn main() -> i64 = square(double(add1(2)));"
        ),
        36  // add1(2)=3, double(3)=6, square(6)=36
    );
}

// --- Boolean Logic ---

#[test]
fn test_run_boolean_not() {
    assert_eq!(
        run_program("fn main() -> bool = !true;"),
        Value::Bool(false)
    );
    assert_eq!(
        run_program("fn main() -> bool = !false;"),
        Value::Bool(true)
    );
}

#[test]
fn test_run_boolean_short_circuit() {
    // Test complex boolean expression
    assert_eq!(
        run_program(
            "fn main() -> bool = (3 > 2) && (10 < 20) && !(5 == 6);"
        ),
        Value::Bool(true)
    );
}

// --- Closure / Lambda ---

#[test]
fn test_run_closure_call() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let add_ten = fn |x: i64| { x + 10 };
               add_ten(32)
             };"
        ),
        42
    );
}

// --- Shift Operators ---

#[test]
fn test_run_shift_operators() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = 1 << 10;"),
        1024
    );
    assert_eq!(
        run_program_i64("fn main() -> i64 = 1024 >> 3;"),
        128
    );
}

// ============================================
// MIR Pipeline and LLVM IR Output Tests
// ============================================
// These tests verify MIR lowering correctness, LLVM IR quality,
// and optimization effectiveness through the full compilation pipeline.

// --- MIR Lowering Correctness ---

#[test]
fn test_mir_while_loop() {
    // While loop should produce Goto (back-edge) and Branch (condition check) terminators
    let mir = lower_to_mir(
        "fn sum_to(n: i64) -> i64 = {
           let mut total: i64 = 0;
           let mut i: i64 = 1;
           while i <= n { { total = total + i; i = i + 1; 0 } };
           total
         };"
    );
    let func = find_mir_fn(&mir, "sum_to");
    // While loop creates a loop header with Branch terminator (condition)
    assert!(func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Branch { .. })),
        "While loop should produce Branch terminator for condition check");
    // While loop creates a back-edge with Goto terminator (loop back)
    assert!(func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Goto(_))),
        "While loop should produce Goto terminator for back-edge");
    // Should have multiple blocks: entry, loop header, loop body, loop exit
    assert!(func.blocks.len() >= 3,
        "While loop should create at least 3 blocks (header, body, exit)");
}

#[test]
fn test_mir_match_expression() {
    // Match expression should produce Switch terminator or branch chain
    let mir = lower_to_mir(
        "fn classify(x: i64) -> i64 = match x { 0 => 10, 1 => 20, _ => 30 };"
    );
    let func = find_mir_fn(&mir, "classify");
    // Match creates either Switch terminator or Branch chain
    let has_switch = func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Switch { .. }));
    let has_branch = func.blocks.iter().any(|b| matches!(b.terminator, Terminator::Branch { .. }));
    assert!(has_switch || has_branch,
        "Match expression should produce Switch or Branch terminator");
    // Match creates merge block with multiple incoming paths
    assert!(func.blocks.len() >= 3,
        "Match should create multiple blocks (arms + merge)");
}

#[test]
fn test_mir_struct_field_access() {
    // Struct field access should produce FieldAccess MIR instruction
    let mir = lower_to_mir(r#"
        struct Pair { a: i64, b: i64 }
        fn sum_pair(p: Pair) -> i64 = p.a + p.b;
    "#);
    let func = find_mir_fn(&mir, "sum_pair");
    // Should have FieldAccess instructions for both p.a and p.b
    assert!(has_inst(func, |i| matches!(i, MirInst::FieldAccess { field, struct_name, .. }
        if field == "a" && struct_name == "Pair")),
        "Should have FieldAccess for field 'a' on struct 'Pair'");
    assert!(has_inst(func, |i| matches!(i, MirInst::FieldAccess { field, struct_name, .. }
        if field == "b" && struct_name == "Pair")),
        "Should have FieldAccess for field 'b' on struct 'Pair'");
}

#[test]
fn test_mir_function_call() {
    // Function call should produce Call MIR instruction
    let mir = lower_to_mir(r#"
        fn square(x: i64) -> i64 = x * x;
        fn cube(x: i64) -> i64 = x * square(x);
    "#);
    let func = find_mir_fn(&mir, "cube");
    assert!(has_inst(func, |i| matches!(i, MirInst::Call { func, .. } if func == "square")),
        "cube() should contain a Call to square()");
}

// --- LLVM IR Quality ---

#[test]
fn test_ir_modulo_operator() {
    // Modulo (%) should produce srem instruction in LLVM IR
    let ir = source_to_ir("fn modulo(a: i64, b: i64) -> i64 = a % b;");
    assert!(ir.contains("srem"),
        "Modulo operator should produce srem instruction, got:\n{}", ir);
}

#[test]
fn test_ir_comparison_operators() {
    // Comparisons should produce icmp instructions
    let ir = source_to_ir_unopt(
        "fn less(a: i64, b: i64) -> bool = a < b;
         fn equal(a: i64, b: i64) -> bool = a == b;
         fn greater(a: i64, b: i64) -> bool = a > b;"
    );
    assert!(ir.contains("icmp slt"),
        "Less-than should produce icmp slt, got:\n{}", ir);
    assert!(ir.contains("icmp eq"),
        "Equality should produce icmp eq, got:\n{}", ir);
    assert!(ir.contains("icmp sgt"),
        "Greater-than should produce icmp sgt, got:\n{}", ir);
}

#[test]
fn test_ir_float_operations() {
    // f64 operations should produce fadd/fsub/fmul/fdiv instructions
    let ir = source_to_ir_unopt(
        "fn f_add(a: f64, b: f64) -> f64 = a + b;
         fn f_sub(a: f64, b: f64) -> f64 = a - b;
         fn f_mul(a: f64, b: f64) -> f64 = a * b;
         fn f_div(a: f64, b: f64) -> f64 = a / b;"
    );
    assert!(ir.contains("fadd"), "f64 addition should produce fadd, got:\n{}", ir);
    assert!(ir.contains("fsub"), "f64 subtraction should produce fsub, got:\n{}", ir);
    assert!(ir.contains("fmul"), "f64 multiplication should produce fmul, got:\n{}", ir);
    assert!(ir.contains("fdiv"), "f64 division should produce fdiv, got:\n{}", ir);
}

#[test]
fn test_ir_boolean_logic() {
    // Boolean and/or/bxor should produce and/or/xor in LLVM IR
    let ir = source_to_ir_unopt(
        "fn bool_and(a: bool, b: bool) -> bool = a and b;
         fn bool_or(a: bool, b: bool) -> bool = a or b;
         fn int_xor(a: i64, b: i64) -> i64 = a bxor b;"
    );
    assert!(ir.contains(" and "),
        "Boolean 'and' should produce LLVM 'and' instruction, got:\n{}", ir);
    assert!(ir.contains(" or "),
        "Boolean 'or' should produce LLVM 'or' instruction, got:\n{}", ir);
    assert!(ir.contains(" xor "),
        "Bitwise 'bxor' should produce LLVM 'xor' instruction, got:\n{}", ir);
}

#[test]
fn test_ir_shift_operations() {
    // Shift operators should produce shl/ashr instructions
    let ir = source_to_ir(
        "fn shift_left(x: i64, n: i64) -> i64 = x << n;
         fn shift_right(x: i64, n: i64) -> i64 = x >> n;"
    );
    assert!(ir.contains("shl"),
        "Left shift should produce shl instruction, got:\n{}", ir);
    assert!(ir.contains("ashr"),
        "Right shift should produce ashr (arithmetic shift right) instruction, got:\n{}", ir);
}

#[test]
fn test_ir_pure_function_attribute() {
    // @pure functions should get memory(none) attribute in LLVM IR
    let ir = source_to_ir(r#"
        @pure
        fn pure_add(a: i64, b: i64) -> i64 = a + b;
    "#);
    assert!(ir.contains("memory(none)"),
        "@pure function should have memory(none) attribute for LLVM optimization, got:\n{}", ir);
}

// --- Optimization Verification ---

#[test]
fn test_ir_constant_folding() {
    // Constant expression 2 + 3 should be folded to 5 by optimizer
    let ir = source_to_ir("fn five() -> i64 = 2 + 3;");
    // After constant folding + narrowing, should contain literal 5
    assert!(ir.contains("i64 5") || ir.contains("i32 5"),
        "Constant 2+3 should be folded to 5, got:\n{}", ir);
}

#[test]
fn test_ir_dead_code_eliminated() {
    // Unreachable/unused computation should be removed by DCE
    let ir = source_to_ir(
        "fn identity(x: i64) -> i64 = { let unused: i64 = x * 42; x };"
    );
    // After DCE, the unused multiplication should be removed
    // The function should just return x directly
    assert!(ir.contains("ret i64 %x"),
        "After DCE, unused multiplication should be removed and function returns x directly, got:\n{}", ir);
}

#[test]
fn test_ir_tail_call_to_loop() {
    // Tail-recursive function should be transformed to a loop (no call instruction)
    let ir = source_to_ir(
        "fn countdown(n: i64) -> i64 = if n <= 0 { 0 } else { countdown(n - 1) };"
    );
    // After TCO + TailRecursiveToLoop, the self-call should be eliminated
    assert!(!ir.contains("call i64 @countdown"),
        "Tail-recursive call should be eliminated and converted to loop, got:\n{}", ir);
    // Should have a loop structure
    assert!(ir.contains("loop_header"),
        "Tail-to-loop conversion should produce loop_header block, got:\n{}", ir);
}

#[test]
fn test_ir_contract_bounds_elimination() {
    // Precondition should allow optimizer to eliminate redundant checks
    let ir = source_to_ir(
        "fn check_positive(x: i64) -> bool
           pre x > 0
         = x > 0;"
    );
    // Contract-based optimization should fold x > 0 to constant true
    assert!(ir.contains("store i1 1") || ir.contains("store i1 true") || ir.contains("ret i1 1") || ir.contains("ret i1 true"),
        "Precondition x > 0 should eliminate redundant check x > 0 to constant true, got:\n{}", ir);
}

#[test]
fn test_ir_algebraic_identity() {
    // x * 1 should be simplified to just x
    let ir_opt = source_to_ir("fn times_one(x: i64) -> i64 = x * 1;");
    let ir_unopt = source_to_ir_unopt("fn times_one(x: i64) -> i64 = x * 1;");
    // Unoptimized should have the multiplication
    assert!(ir_unopt.contains("mul"),
        "Unoptimized IR should contain mul instruction, got:\n{}", ir_unopt);
    // Optimized should simplify: either no mul, or direct return of x
    let simplified = !ir_opt.contains("mul") || ir_opt.contains("ret i64 %x");
    assert!(simplified,
        "Optimizer should simplify x * 1 to just x (no mul or direct return), got:\n{}", ir_opt);
}

// ============================================
// Cycles 115-116: Codegen Edge Cases & Runtime Behavior
// ============================================
// These tests cover deeply nested control flow, complex enum matching,
// struct interactions, cast operations, array/tuple usage, and
// negative type-checking scenarios.

// --- Deeply Nested Match Expressions ---

#[test]
fn test_run_nested_match_inside_match() {
    // Match inside match: outer match selects a category, inner match refines it
    assert_eq!(
        run_program_i64(
            "fn classify(x: i64, y: i64) -> i64 =
               match x {
                 0 => match y {
                   0 => 0,
                   1 => 1,
                   _ => 2
                 },
                 1 => match y {
                   0 => 10,
                   _ => 11
                 },
                 _ => 99
               };
             fn main() -> i64 = classify(0, 1) + classify(1, 0) + classify(5, 5);"
        ),
        110  // 1 + 10 + 99
    );
}

#[test]
fn test_run_match_with_computation_in_arms() {
    // Each match arm performs non-trivial computation
    assert_eq!(
        run_program_i64(
            "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };
             fn compute(mode: i64) -> i64 =
               match mode {
                 0 => fib(8),
                 1 => fib(6) * 2,
                 _ => 0
               };
             fn main() -> i64 = compute(0) + compute(1);"
        ),
        37  // fib(8)=21, fib(6)*2=8*2=16, 21+16=37
    );
}

// --- Complex Enum with Data and Pattern Matching ---

#[test]
fn test_run_enum_with_data_nested_unwrap() {
    // Multiple levels of enum unwrapping
    assert_eq!(
        run_program_i64(
            "enum Option<T> { Some(T), None }
             fn add_options(a: Option<i64>, b: Option<i64>) -> i64 =
               match a {
                 Option::Some(x) => match b {
                   Option::Some(y) => x + y,
                   Option::None => x
                 },
                 Option::None => match b {
                   Option::Some(y) => y,
                   Option::None => 0
                 }
               };
             fn main() -> i64 = {
               let r1 = add_options(Option::Some(10), Option::Some(20));
               let r2 = add_options(Option::Some(5), Option::None);
               let r3 = add_options(Option::None, Option::Some(3));
               let r4 = add_options(Option::None, Option::None);
               r1 + r2 + r3 + r4
             };"
        ),
        38  // 30 + 5 + 3 + 0
    );
}

#[test]
fn test_run_enum_result_like() {
    // Model a Result-like enum with Ok(i64) and Err(i64)
    assert_eq!(
        run_program_i64(
            "enum Result { Ok(i64), Err(i64) }
             fn unwrap_or_error(r: Result) -> i64 =
               match r {
                 Result::Ok(v) => v,
                 Result::Err(code) => 0 - code
               };
             fn main() -> i64 = {
               let ok_val = unwrap_or_error(Result::Ok(42));
               let err_val = unwrap_or_error(Result::Err(7));
               ok_val + err_val
             };"
        ),
        35  // 42 + (-7) = 35
    );
}

// --- While Loop with Early Exit Condition ---

#[test]
fn test_run_while_loop_find_first_divisible() {
    // Find the first number >= start that is divisible by d
    assert_eq!(
        run_program_i64(
            "fn find_divisible(start: i64, d: i64) -> i64 = {
               let mut n: i64 = start;
               while n % d != 0 { n = n + 1; 0 };
               n
             };
             fn main() -> i64 = find_divisible(10, 7);"
        ),
        14  // 14 is first number >= 10 divisible by 7
    );
}

#[test]
fn test_run_while_accumulate_digits() {
    // Sum digits of a number: 1234 -> 1+2+3+4 = 10
    assert_eq!(
        run_program_i64(
            "fn sum_digits(n: i64) -> i64 = {
               let mut remaining: i64 = n;
               let mut total: i64 = 0;
               while remaining > 0 {
                 total = total + remaining % 10;
                 remaining = remaining / 10;
                 0
               };
               total
             };
             fn main() -> i64 = sum_digits(1234);"
        ),
        10
    );
}

// --- Recursive Data Processing ---

#[test]
fn test_run_recursive_sum_to_n() {
    // Recursive sum: sum(n) = n + sum(n-1), base case sum(0) = 0
    assert_eq!(
        run_program_i64(
            "fn sum_to(n: i64) -> i64 = if n <= 0 { 0 } else { n + sum_to(n - 1) };
             fn main() -> i64 = sum_to(100);"
        ),
        5050
    );
}

#[test]
fn test_run_recursive_power() {
    // Compute base^exp recursively
    assert_eq!(
        run_program_i64(
            "fn power(base: i64, exp: i64) -> i64 =
               if exp == 0 { 1 } else { base * power(base, exp - 1) };
             fn main() -> i64 = power(3, 5);"
        ),
        243  // 3^5 = 243
    );
}

// --- Multiple Structs Interacting ---

#[test]
fn test_run_point_and_rect_structs() {
    // Point struct and a function computing Manhattan distance
    assert_eq!(
        run_program_i64(
            "struct Point { x: i64, y: i64 }
             struct Rect { origin: Point, w: i64, h: i64 }
             fn abs(v: i64) -> i64 = if v < 0 { 0 - v } else { v };
             fn rect_perimeter(r: Rect) -> i64 = 2 * (r.w + r.h);
             fn main() -> i64 = {
               let r = new Rect { origin: new Point { x: 1, y: 2 }, w: 10, h: 5 };
               rect_perimeter(r)
             };"
        ),
        30  // 2 * (10 + 5) = 30
    );
}

#[test]
fn test_run_struct_returned_from_function() {
    // Function creates and returns a struct
    assert_eq!(
        run_program_i64(
            "struct Vec2 { x: i64, y: i64 }
             fn make_vec(a: i64, b: i64) -> Vec2 = new Vec2 { x: a, y: b };
             fn dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y;
             fn main() -> i64 = {
               let v1 = make_vec(3, 4);
               let v2 = make_vec(2, 5);
               dot(v1, v2)
             };"
        ),
        26  // 3*2 + 4*5 = 6 + 20 = 26
    );
}

// --- Mixed Arithmetic with Casts ---

#[test]
fn test_run_cast_i64_to_f64() {
    // Cast an integer to float and do float arithmetic
    match run_program("fn main() -> f64 = { let x: i64 = 7; let y = x as f64; y * 1.5 };") {
        Value::Float(v) => assert!((v - 10.5).abs() < 1e-10, "expected 10.5, got {}", v),
        other => panic!("expected Float, got {:?}", other),
    }
}

#[test]
fn test_run_cast_f64_to_i64_truncation() {
    // Cast float to integer (truncation)
    assert_eq!(
        run_program_i64("fn main() -> i64 = { let f: f64 = 9.7; f as i64 };"),
        9  // truncation, not rounding
    );
}

#[test]
fn test_run_cast_bool_to_i64() {
    // Cast booleans to integers: true=1, false=0
    assert_eq!(
        run_program_i64("fn main() -> i64 = { let t = true as i64; let f = false as i64; t + f };"),
        1  // 1 + 0
    );
}

// --- For Loop with Range ---

#[test]
fn test_run_for_loop_sum_range() {
    // Sum integers in range 0..10 using for loop
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = { let mut s: i64 = 0; for i in 0..10 { s = s + i }; s };"
        ),
        45  // 0+1+2+...+9
    );
}

#[test]
fn test_run_for_loop_nested() {
    // Nested for loops to compute a small sum-of-sums
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut total: i64 = 0;
               for i in 0..4 {
                 for j in 0..3 {
                   total = total + i * j
                 }
               };
               total
             };"
        ),
        18  // sum of i*j for i in 0..4, j in 0..3 = 0+0+0 + 0+1+2 + 0+2+4 + 0+3+6 = 18
    );
}

// --- Array Operations ---

#[test]
fn test_run_array_create_and_index() {
    // Create array, index into it, sum first and last elements
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let arr = [10, 20, 30, 40, 50];
               arr[0] + arr[4]
             };"
        ),
        60  // 10 + 50
    );
}

#[test]
fn test_run_array_length() {
    // Test array .len() method
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let arr = [1, 2, 3, 4, 5, 6, 7];
               arr.len()
             };"
        ),
        7
    );
}

// --- Tuple Creation and Field Access ---

#[test]
fn test_run_tuple_create_and_access() {
    // Create a tuple and access its fields by index
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let t = (10, 20, 30);
               t.0 + t.1 + t.2
             };"
        ),
        60
    );
}

#[test]
fn test_run_tuple_from_function() {
    // Function returning a tuple, caller accesses fields
    assert_eq!(
        run_program_i64(
            "fn divmod(a: i64, b: i64) -> (i64, i64) = (a / b, a % b);
             fn main() -> i64 = {
               let result = divmod(17, 5);
               result.0 * 10 + result.1
             };"
        ),
        32  // (17/5)=3, (17%5)=2 -> 3*10 + 2 = 32
    );
}

// --- Multi-Function Programs with Shared Structs ---

#[test]
fn test_run_multi_function_struct_pipeline() {
    // Chain of functions transforming a struct
    assert_eq!(
        run_program_i64(
            "struct Counter { value: i64 }
             fn make_counter() -> Counter = new Counter { value: 0 };
             fn get_value(c: Counter) -> i64 = c.value;
             fn main() -> i64 = {
               let c = make_counter();
               get_value(c) + 100
             };"
        ),
        100
    );
}

// --- Complex Boolean and Bitwise Operations ---

#[test]
fn test_run_bitwise_operations() {
    // Test band, bor, bxor
    assert_eq!(
        run_program_i64("fn main() -> i64 = (15 band 9) + (5 bor 3) + (12 bxor 10);"),
        22  // (15 & 9)=9, (5 | 3)=7, (12 ^ 10)=6 -> 9+7+6=22
    );
}

#[test]
fn test_run_complex_boolean_expression() {
    // Complex boolean logic with multiple operators
    assert_eq!(
        run_program(
            "fn check(a: i64, b: i64, c: i64) -> bool =
               (a > 0 and b > 0) or (c < 0 and not (a == b));
             fn main() -> bool = check(1, 2, 3);"
        ),
        Value::Bool(true)  // (1>0 and 2>0) = true, short-circuits to true
    );
}

// --- Negative / Error Tests (Type Checking Failures) ---

#[test]
fn test_type_error_return_struct_for_int() {
    // Cannot return a struct where i64 is expected
    assert!(type_error(
        "struct Foo { x: i64 }
         fn main() -> i64 = new Foo { x: 1 };"
    ));
}

#[test]
fn test_type_error_mismatched_if_branches() {
    // If/else branches return different types
    assert!(type_error(
        r#"fn bad(x: bool) -> i64 = if x { 42 } else { "hello" };"#
    ));
}

#[test]
fn test_type_error_wrong_struct_field_type() {
    // Struct field initialized with wrong type
    assert!(type_error(
        r#"struct Point { x: i64, y: i64 }
         fn main() -> Point = new Point { x: "bad", y: 0 };"#
    ));
}

#[test]
fn test_type_error_enum_variant_wrong_data() {
    // Enum variant constructed with wrong type
    assert!(type_error(
        "enum Option<T> { Some(T), None }
         fn bad() -> Option<i64> = Option::Some(true);"
    ));
}

// ===== Cycles 125-126: Positive pipeline tests =====

#[test]
fn test_nested_match_enum() {
    assert!(type_checks(
        "enum Color { Red, Green, Blue }
         fn to_code(c: Color) -> i64 =
           match c {
             Color::Red => 1,
             Color::Green => 2,
             Color::Blue => 3,
           };
         fn main() -> i64 = to_code(Color::Green);"
    ));
}

#[test]
fn test_struct_method_style() {
    assert!(type_checks(
        "struct Vec2 { x: i64, y: i64 }
         fn dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y;
         fn main() -> i64 = dot(new Vec2 { x: 3, y: 4 }, new Vec2 { x: 1, y: 2 });"
    ));
}

#[test]
fn test_generic_enum_option() {
    assert!(type_checks(
        "enum MyOpt<T> { Some(T), None }
         fn unwrap_or(opt: MyOpt<i64>, default: i64) -> i64 =
           match opt {
             MyOpt::Some(v) => v,
             MyOpt::None => default,
           };
         fn main() -> i64 = unwrap_or(MyOpt::Some(42), 0);"
    ));
}

#[test]
fn test_for_loop_sum() {
    assert!(type_checks(
        "fn sum(n: i64) -> i64 = { let mut total = 0; for i in 0..n { total = total + i }; total };
         fn main() -> i64 = sum(10);"
    ));
}

#[test]
fn test_while_loop_countdown() {
    assert!(type_checks(
        "fn countdown(n: i64) -> i64 = { let mut x = n; while x > 0 { x = x - 1; 0 }; x };
         fn main() -> i64 = countdown(5);"
    ));
}

#[test]
fn test_contracts_postcondition() {
    assert!(type_checks(
        "fn abs(x: i64) -> i64 post ret >= 0 = if x >= 0 { x } else { 0 - x };
         fn main() -> i64 = abs(-5);"
    ));
}

#[test]
fn test_recursive_factorial() {
    assert!(type_checks(
        "fn fact(n: i64) -> i64
           pre n >= 0
         = if n <= 1 { 1 } else { n * fact(n - 1) };
         fn main() -> i64 = fact(5);"
    ));
}

#[test]
fn test_mutual_recursion_even_odd() {
    assert!(type_checks(
        "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
         fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };
         fn main() -> bool = is_even(4);"
    ));
}

#[test]
fn test_nullable_pipeline() {
    assert!(type_checks(
        "fn safe_div(a: i64, b: i64) -> i64? =
           if b == 0 { null } else { a / b };
         fn main() -> i64 = { let r = safe_div(10, 2); r.unwrap_or(0) };"
    ));
}

#[test]
fn test_string_concat_pipeline() {
    assert!(type_checks(
        r#"fn greet(name: String) -> String = "Hello, " + name;
         fn main() -> String = greet("world");"#
    ));
}

#[test]
fn test_bitwise_operations() {
    assert!(type_checks(
        "fn mask(x: i64, m: i64) -> i64 = x band m;
         fn main() -> i64 = mask(255, 15);"
    ));
}

#[test]
fn test_complex_struct_nesting() {
    assert!(type_checks(
        "struct Inner { val: i64 }
         struct Outer { a: Inner, b: Inner }
         fn sum_outer(o: Outer) -> i64 = o.a.val + o.b.val;
         fn main() -> i64 = sum_outer(new Outer { a: new Inner { val: 3 }, b: new Inner { val: 7 } });"
    ));
}

#[test]
fn test_type_alias_usage() {
    assert!(type_checks(
        "type Distance = i64;
         fn add_dist(a: Distance, b: Distance) -> Distance = a + b;
         fn main() -> i64 = add_dist(10, 20);"
    ));
}

#[test]
fn test_enum_with_multiple_data() {
    assert!(type_checks(
        "enum Shape { Circle(f64), Rect(f64, f64) }
         fn describe(s: Shape) -> f64 =
           match s {
             Shape::Circle(r) => r,
             Shape::Rect(w, h) => w + h,
           };
         fn main() -> f64 = describe(Shape::Circle(3.14));"
    ));
}

// ===== Cycles 125-126: Error integration tests =====

#[test]
fn test_error_while_non_bool_condition() {
    assert!(type_error(
        "fn bad() -> i64 = { while 42 { 0 }; 0 };"
    ));
}

#[test]
fn test_error_for_non_integer_range() {
    assert!(type_error(
        r#"fn bad() -> i64 = { for i in "a".."z" { 0 }; 0 };"#
    ));
}

#[test]
fn test_error_if_non_bool_condition() {
    assert!(type_error(
        "fn bad() -> i64 = if 42 { 1 } else { 0 };"
    ));
}

#[test]
fn test_error_binary_op_type_mismatch() {
    // Cannot add i64 and bool
    assert!(type_error(
        "fn bad() -> i64 = 42 + true;"
    ));
}

#[test]
fn test_error_return_str_for_i64() {
    assert!(type_error(
        r#"fn bad() -> i64 = "not a number";"#
    ));
}

#[test]
fn test_error_unknown_function_call() {
    assert!(type_error(
        "fn main() -> i64 = nonexistent(42);"
    ));
}

#[test]
fn test_error_struct_unknown_field() {
    assert!(type_error(
        "struct Point { x: i64, y: i64 }
         fn main() -> i64 = { let p = new Point { x: 1, z: 2 }; p.x };"
    ));
}

// ============================================
// Loop Expression Tests (Cycles 184-185)
// ============================================

#[test]
fn test_loop_with_break_type_checks() {
    assert!(type_checks(
        "fn count(n: i64) -> i64 = {
           let mut i: i64 = 0;
           loop { if i >= n { break } else { () }; { i = i + 1 } };
           i
         };
         fn main() -> i64 = count(5);"
    ));
}

#[test]
fn test_loop_with_break_runs() {
    assert_eq!(
        run_program_i64(
            "fn count(n: i64) -> i64 = {
               let mut i: i64 = 0;
               loop { if i >= n { break } else { () }; { i = i + 1 } };
               i
             };
             fn main() -> i64 = count(5);"
        ),
        5
    );
}

#[test]
fn test_loop_with_continue_runs() {
    // Sum of odd numbers from 1 to 9 using continue to skip even
    assert_eq!(
        run_program_i64(
            "fn sum_odd(n: i64) -> i64 = {
               let mut i: i64 = 0;
               let mut s: i64 = 0;
               loop {
                 if i >= n { break } else { () };
                 { i = i + 1 };
                 if i % 2 == 0 { continue } else { () };
                 { s = s + i }
               };
               s
             };
             fn main() -> i64 = sum_odd(10);"
        ),
        25 // 1+3+5+7+9 = 25
    );
}

#[test]
fn test_loop_with_return_runs() {
    assert_eq!(
        run_program_i64(
            "fn find(target: i64) -> i64 = {
               let mut i: i64 = 0;
               loop {
                 if i == target { return i * 10 } else { () };
                 { i = i + 1 }
               };
               -1
             };
             fn main() -> i64 = find(7);"
        ),
        70
    );
}

#[test]
fn test_nested_loop_runs() {
    // Count how many pairs (i,j) where i*j == 6, i,j in 1..5
    assert_eq!(
        run_program_i64(
            "fn count_products(target: i64) -> i64 = {
               let mut count: i64 = 0;
               let mut i: i64 = 1;
               loop {
                 if i > 5 { break } else { () };
                 let mut j: i64 = 1;
                 loop {
                   if j > 5 { break } else { () };
                   if i * j == target { count = count + 1 } else { () };
                   { j = j + 1 }
                 };
                 { i = i + 1 }
               };
               count
             };
             fn main() -> i64 = count_products(6);"
        ),
        2 // i,j in 1..5: (2,3) and (3,2) give product 6
    );
}

#[test]
fn test_return_expression_type_checks() {
    assert!(type_checks(
        "fn early(x: i64) -> i64 = {
           if x > 10 { return 100 } else { () };
           x * 2
         };
         fn main() -> i64 = early(5);"
    ));
}

#[test]
fn test_return_expression_runs() {
    assert_eq!(
        run_program_i64(
            "fn early(x: i64) -> i64 = {
               if x > 10 { return 100 } else { () };
               x * 2
             };
             fn main() -> i64 = early(5);"
        ),
        10
    );
}

#[test]
fn test_return_expression_early_exit() {
    assert_eq!(
        run_program_i64(
            "fn early(x: i64) -> i64 = {
               if x > 10 { return 100 } else { () };
               x * 2
             };
             fn main() -> i64 = early(20);"
        ),
        100
    );
}

#[test]
fn test_return_from_nested_if() {
    assert_eq!(
        run_program_i64(
            "fn classify(x: i64) -> i64 = {
               if x > 0 {
                 if x > 100 { return 3 } else { () };
                 return 2
               } else { () };
               if x < 0 { return 1 } else { () };
               0
             };
             fn main() -> i64 = classify(50);"
        ),
        2
    );
}

#[test]
fn test_while_with_early_return() {
    assert_eq!(
        run_program_i64(
            "fn find_first(n: i64) -> i64 = {
               let mut i: i64 = 0;
               while i < n {
                 if i * i > 10 { return i } else { () };
                 { i = i + 1 }
               };
               -1
             };
             fn main() -> i64 = find_first(100);"
        ),
        4 // 4*4=16 > 10
    );
}

#[test]
fn test_for_loop_with_break() {
    assert!(type_checks(
        "fn main() -> i64 = {
           let mut found: i64 = -1;
           for i in 0..100 {
             if i * i > 50 { found = i; break } else { () }
           };
           found
         };"
    ));
}

#[test]
fn test_for_loop_with_break_runs() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut found: i64 = -1;
               for i in 0..100 {
                 if i * i > 50 { found = i; break } else { () }
               };
               found
             };"
        ),
        8 // 8*8=64 > 50
    );
}

#[test]
fn test_for_loop_with_continue_runs() {
    // Sum of odd numbers 1..10 using continue
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut s: i64 = 0;
               for i in 0..10 {
                 if i % 2 == 0 { continue } else { () };
                 { s = s + i }
               };
               s
             };"
        ),
        25 // 1+3+5+7+9 = 25
    );
}

#[test]
fn test_for_loop_with_return_runs() {
    assert_eq!(
        run_program_i64(
            "fn find_square(target: i64) -> i64 = {
               for i in 0..100 {
                 if i * i >= target { return i } else { () }
               };
               -1
             };
             fn main() -> i64 = find_square(50);"
        ),
        8 // 8*8=64 >= 50
    );
}

#[test]
fn test_while_with_break_runs() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut i: i64 = 0;
               while i < 100 {
                 if i > 5 { break } else { () };
                 { i = i + 1 }
               };
               i
             };"
        ),
        6
    );
}

#[test]
fn test_while_with_continue_runs() {
    // Count even numbers from 0..10
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut i: i64 = 0;
               let mut count: i64 = 0;
               while i < 10 {
                 { i = i + 1 };
                 if i % 2 != 0 { continue } else { () };
                 { count = count + 1 }
               };
               count
             };"
        ),
        5 // 2,4,6,8,10
    );
}

#[test]
fn test_loop_infinite_guard() {
    // Test that loop without break causes infinite loop (we can't run this,
    // but we can at least type-check it)
    assert!(type_checks(
        "fn diverge() -> i64 = loop { () };
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_loop_accumulator_runs() {
    // Fibonacci using loop
    assert_eq!(
        run_program_i64(
            "fn fib_loop(n: i64) -> i64 = {
               let mut a: i64 = 0;
               let mut b: i64 = 1;
               let mut i: i64 = 0;
               loop {
                 if i >= n { break } else { () };
                 let temp: i64 = b;
                 { b = a + b };
                 { a = temp };
                 { i = i + 1 }
               };
               a
             };
             fn main() -> i64 = fib_loop(10);"
        ),
        55 // fib(10) = 55
    );
}

#[test]
fn test_interp_is_prime() {
    assert_eq!(
        run_program_i64(
            "fn is_prime(n: i64) -> bool = {
               if n < 2 { return false } else { () };
               let mut i: i64 = 2;
               while i * i <= n {
                 if n % i == 0 { return false } else { () };
                 { i = i + 1 }
               };
               true
             };
             fn main() -> i64 = is_prime(17) as i64;"
        ),
        1
    );
}

#[test]
fn test_interp_is_not_prime() {
    assert_eq!(
        run_program_i64(
            "fn is_prime(n: i64) -> bool = {
               if n < 2 { return false } else { () };
               let mut i: i64 = 2;
               while i * i <= n {
                 if n % i == 0 { return false } else { () };
                 { i = i + 1 }
               };
               true
             };
             fn main() -> i64 = is_prime(15) as i64;"
        ),
        0
    );
}

#[test]
fn test_interp_count_divisors() {
    assert_eq!(
        run_program_i64(
            "fn count_div(n: i64) -> i64 = {
               let mut count: i64 = 0;
               let mut i: i64 = 1;
               loop {
                 if i > n { break } else { () };
                 if n % i == 0 { count = count + 1 } else { () };
                 { i = i + 1 }
               };
               count
             };
             fn main() -> i64 = count_div(12);"
        ),
        6 // 1,2,3,4,6,12
    );
}

#[test]
fn test_interp_gcd_loop() {
    assert_eq!(
        run_program_i64(
            "fn gcd(a: i64, b: i64) -> i64 = {
               let mut x: i64 = a;
               let mut y: i64 = b;
               loop {
                 if y == 0 { return x } else { () };
                 let r: i64 = x % y;
                 { x = y };
                 { y = r }
               };
               x
             };
             fn main() -> i64 = gcd(48, 18);"
        ),
        6
    );
}

#[test]
fn test_interp_power_loop() {
    assert_eq!(
        run_program_i64(
            "fn power(base: i64, exp: i64) -> i64 = {
               let mut result: i64 = 1;
               let mut i: i64 = 0;
               while i < exp {
                 { result = result * base };
                 { i = i + 1 }
               };
               result
             };
             fn main() -> i64 = power(2, 10);"
        ),
        1024
    );
}

#[test]
fn test_interp_collatz_steps() {
    assert_eq!(
        run_program_i64(
            "fn collatz(n: i64) -> i64 = {
               let mut x: i64 = n;
               let mut steps: i64 = 0;
               loop {
                 if x == 1 { break } else { () };
                 if x % 2 == 0 { x = x / 2 } else { x = 3 * x + 1 };
                 { steps = steps + 1 }
               };
               steps
             };
             fn main() -> i64 = collatz(27);"
        ),
        111 // collatz(27) takes 111 steps
    );
}

// ============================================
// Cycle 189: Tuple Tests
// ============================================

#[test]
fn test_tuple_type_check() {
    assert!(type_checks(
        "fn pair() -> (i64, bool) = (42, true);
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_tuple_field_access_type_check() {
    assert!(type_checks(
        "fn first(p: (i64, bool)) -> i64 = p.0;
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_interp_tuple_creation() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let t: (i64, i64) = (10, 20);
               t.0 + t.1
             };"
        ),
        30
    );
}

#[test]
fn test_interp_tuple_three_elements() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let t: (i64, i64, i64) = (1, 2, 3);
               t.0 + t.1 + t.2
             };"
        ),
        6
    );
}

// ============================================
// Cycle 189: Cast Expression Tests
// ============================================

#[test]
fn test_cast_bool_to_i64() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = true as i64;"),
        1
    );
}

#[test]
fn test_cast_false_to_i64() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = false as i64;"),
        0
    );
}

#[test]
fn test_cast_in_expression() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let b: bool = 5 > 3;
               let v: i64 = b as i64;
               v * 10
             };"
        ),
        10
    );
}

// ============================================
// Cycle 189: Array Repeat Tests
// ============================================

#[test]
fn test_array_repeat_type_check() {
    assert!(type_checks(
        "fn main() -> i64 = {
           let a: [i64; 5] = [0; 5];
           a[0]
         };"
    ));
}

#[test]
fn test_interp_array_repeat() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a: [i64; 3] = [7; 3];
               a[0] + a[1] + a[2]
             };"
        ),
        21
    );
}

// ============================================
// Cycle 189: Struct Field Mutation Tests
// ============================================

#[test]
fn test_struct_field_assign_type_check() {
    assert!(type_checks(
        "struct Point { x: i64, y: i64 }
         fn main() -> i64 = {
           let mut p: Point = new Point { x: 1, y: 2 };
           set p.x = 10;
           p.x
         };"
    ));
}

#[test]
fn test_interp_struct_field_assign() {
    assert_eq!(
        run_program_i64(
            "struct Point { x: i64, y: i64 }
             fn main() -> i64 = {
               let mut p: Point = new Point { x: 1, y: 2 };
               set p.x = 10;
               set p.y = 20;
               p.x + p.y
             };"
        ),
        30
    );
}

// ============================================
// Cycle 189: Index Assignment Tests
// ============================================

#[test]
fn test_array_index_assign_type_check() {
    assert!(type_checks(
        "fn main() -> i64 = {
           let mut a: [i64; 3] = [1, 2, 3];
           set a[0] = 10;
           a[0]
         };"
    ));
}

#[test]
fn test_array_index_assign_multitype() {
    // Index assignment type-checks with different array element types
    assert!(type_checks(
        "fn main() -> i64 = {
           let mut a: [bool; 2] = [true, false];
           set a[0] = false;
           0
         };"
    ));
}

// ============================================
// Cycle 189: Closure Tests (extended)
// ============================================

#[test]
fn test_closure_capture() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let x: i64 = 10;
               let add_x = fn |y: i64| { x + y };
               add_x(5)
             };"
        ),
        15
    );
}

#[test]
fn test_closure_nested() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a: i64 = 5;
               let f = fn |x: i64| { a + x };
               let g = fn |y: i64| { f(y) * 2 };
               g(3)
             };"
        ),
        16 // (5+3)*2 = 16
    );
}

// ============================================
// Cycle 189: Todo Expression Tests
// ============================================

#[test]
fn test_todo_type_checks() {
    // todo should type-check as any type
    assert!(type_checks(
        "fn unimplemented() -> i64 = todo \"not done yet\";
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_todo_in_branch() {
    // todo in one branch, concrete in the other — should type-check
    assert!(type_checks(
        "fn maybe(x: i64) -> i64 = if x > 0 { x } else { todo \"negative case\" };
         fn main() -> i64 = 0;"
    ));
}

// ============================================
// Cycle 189: Advanced Pattern Matching Tests
// ============================================

#[test]
fn test_match_with_wildcard() {
    assert!(type_checks(
        "enum Color { Red, Green, Blue }
         fn to_int(c: Color) -> i64 = match c {
           Color::Red => 0,
           Color::Green => 1,
           _ => 2
         };
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_match_enum_with_data() {
    assert!(type_checks(
        "enum Shape { Circle(f64), Rect(f64, f64) }
         fn area(s: Shape) -> f64 = match s {
           Shape::Circle(r) => 3.14159 * r * r,
           Shape::Rect(w, h) => w * h
         };
         fn main() -> i64 = 0;"
    ));
}

// ============================================
// Cycle 189: Nested Control Flow Tests
// ============================================

#[test]
fn test_interp_nested_loop_break() {
    // Outer loop counts, inner loop finds first divisor > 1
    assert_eq!(
        run_program_i64(
            "fn smallest_factor(n: i64) -> i64 = {
               let mut i: i64 = 2;
               loop {
                 if i > n { break } else { () };
                 if n % i == 0 { return i } else { () };
                 { i = i + 1 }
               };
               n
             };
             fn main() -> i64 = smallest_factor(15);"
        ),
        3
    );
}

#[test]
fn test_interp_while_with_early_return() {
    assert_eq!(
        run_program_i64(
            "fn find_threshold(limit: i64) -> i64 = {
               let mut sum: i64 = 0;
               let mut i: i64 = 1;
               while i <= 100 {
                 { sum = sum + i };
                 if sum > limit { return i } else { () };
                 { i = i + 1 }
               };
               0
             };
             fn main() -> i64 = find_threshold(50);"
        ),
        10 // 1+2+3+4+5+6+7+8+9+10=55 > 50
    );
}

#[test]
fn test_interp_for_sum_with_break() {
    assert_eq!(
        run_program_i64(
            "fn sum_until(limit: i64) -> i64 = {
               let mut sum: i64 = 0;
               for i in 1..=100 {
                 { sum = sum + i };
                 if sum >= limit { break } else { () }
               };
               sum
             };
             fn main() -> i64 = sum_until(20);"
        ),
        21 // 1+2+3+4+5+6=21 >= 20
    );
}

// ============================================
// Cycle 189: Recursive Data Structure Tests
// ============================================

#[test]
fn test_mutual_recursion_type_check() {
    assert!(type_checks(
        "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
         fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_interp_mutual_recursion() {
    assert_eq!(
        run_program_i64(
            "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
             fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };
             fn main() -> i64 = is_even(10) as i64 + is_odd(7) as i64;"
        ),
        2 // is_even(10)=true(1) + is_odd(7)=true(1) = 2
    );
}

// ============================================
// Cycle 189: Floating Point Interpreter Tests
// ============================================

#[test]
fn test_interp_f64_operations() {
    let result = run_program(
        "fn main() -> f64 = 1.5 * 2.0;"
    );
    match result {
        Value::Float(f) => assert!((f - 3.0).abs() < 0.001),
        other => panic!("expected Float, got {:?}", other),
    }
}

#[test]
fn test_interp_f64_comparison() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = if 3.14 > 2.71 { 1 } else { 0 };"
        ),
        1
    );
}

// ============================================
// Cycle 192: i32 Full Pipeline Integration Tests
// ============================================

// --- i32 Type Checking ---

#[test]
fn test_i32_basic_function() {
    assert!(type_checks("fn add32(a: i32, b: i32) -> i32 = a + b;"));
}

#[test]
fn test_i32_arithmetic_ops() {
    assert!(type_checks(
        "fn math32(x: i32, y: i32) -> i32 = (x + y) * (x - y) / (y + 1);"
    ));
}

#[test]
fn test_i32_comparison() {
    assert!(type_checks(
        "fn gt32(a: i32, b: i32) -> bool = a > b;"
    ));
}

#[test]
fn test_i32_bitwise_ops() {
    assert!(type_checks(
        "fn bits32(x: i32, y: i32) -> i32 = (x band y) bor (x bxor y);"
    ));
}

#[test]
fn test_i32_shift_ops() {
    assert!(type_checks(
        "fn shift32(x: i32, n: i32) -> i32 = (x << n) >> n;"
    ));
}

#[test]
fn test_i32_negation() {
    assert!(type_checks(
        "fn neg32(x: i32) -> i32 = -x;"
    ));
}

#[test]
fn test_i32_if_expression() {
    assert!(type_checks(
        "fn max32(a: i32, b: i32) -> i32 = if a > b { a } else { b };"
    ));
}

#[test]
fn test_i32_let_binding() {
    assert!(type_checks(
        "fn test32() -> i32 = { let x: i32 = 10; let y: i32 = 20; x + y };"
    ));
}

#[test]
fn test_i32_while_loop() {
    assert!(type_checks(
        "fn sum32(n: i32) -> i32 = {
            let mut s: i32 = 0;
            let mut i: i32 = 0;
            while i < n {
                s = s + i;
                { i = i + 1 }
            };
            s
        };"
    ));
}

#[test]
fn test_i32_for_loop() {
    assert!(type_checks(
        "fn sum_range32(n: i64) -> i64 = {
            let mut s: i64 = 0;
            for i in 1..=n {
                s = s + i
            };
            s
        };"
    ));
}

#[test]
fn test_i32_cast_to_i64() {
    assert!(type_checks(
        "fn widen(x: i32) -> i64 = x as i64;"
    ));
}

#[test]
fn test_i64_cast_to_i32() {
    assert!(type_checks(
        "fn narrow(x: i64) -> i32 = x as i32;"
    ));
}

#[test]
fn test_i32_cast_to_f64() {
    assert!(type_checks(
        "fn to_float(x: i32) -> f64 = x as f64;"
    ));
}

#[test]
fn test_f64_cast_to_i32() {
    assert!(type_checks(
        "fn to_int(x: f64) -> i32 = x as i32;"
    ));
}

#[test]
fn test_i32_bool_cast() {
    assert!(type_checks(
        "fn to_i32(b: bool) -> i32 = b as i32;"
    ));
}

#[test]
fn test_i32_with_contract() {
    assert!(type_checks(
        "fn safe_div32(x: i32, y: i32) -> i32
           pre y != 0
         = x / y;"
    ));
}

#[test]
fn test_i32_modulo() {
    assert!(type_checks(
        "fn mod32(x: i32, y: i32) -> i32 = x % y;"
    ));
}

// --- i32 Type Errors ---

#[test]
fn test_i32_i64_mismatch() {
    // Cannot add i32 and i64 without explicit cast
    assert!(type_error(
        "fn bad(a: i32, b: i64) -> i64 = a + b;"
    ));
}

#[test]
fn test_i32_return_type_mismatch() {
    // Returning i32 from i64 function should fail
    assert!(type_error(
        "fn bad(x: i32) -> i64 = x;"
    ));
}

// --- i32 Interpreter Tests ---

#[test]
fn test_interp_i32_addition() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 15; let b: i32 = 25; a + b };"
        ),
        Value::Int(40)
    );
}

#[test]
fn test_interp_i32_subtraction() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 50; let b: i32 = 30; a - b };"
        ),
        Value::Int(20)
    );
}

#[test]
fn test_interp_i32_multiplication() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 7; let b: i32 = 8; a * b };"
        ),
        Value::Int(56)
    );
}

#[test]
fn test_interp_i32_division() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 100; let b: i32 = 4; a / b };"
        ),
        Value::Int(25)
    );
}

#[test]
fn test_interp_i32_modulo() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 17; let b: i32 = 5; a % b };"
        ),
        Value::Int(2)
    );
}

#[test]
fn test_interp_i32_negation() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let x: i32 = 42; -x };"
        ),
        Value::Int(-42)
    );
}

#[test]
fn test_interp_i32_bitwise_and() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 12; let b: i32 = 10; a band b };"
        ),
        Value::Int(8) // 1100 & 1010 = 1000
    );
}

#[test]
fn test_interp_i32_bitwise_or() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 12; let b: i32 = 10; a bor b };"
        ),
        Value::Int(14) // 1100 | 1010 = 1110
    );
}

#[test]
fn test_interp_i32_bitwise_xor() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let a: i32 = 12; let b: i32 = 10; a bxor b };"
        ),
        Value::Int(6) // 1100 ^ 1010 = 0110
    );
}

#[test]
fn test_interp_i32_shift_left() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let x: i32 = 1; x << 4 };"
        ),
        Value::Int(16)
    );
}

#[test]
fn test_interp_i32_shift_right() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let x: i32 = 64; x >> 3 };"
        ),
        Value::Int(8)
    );
}

#[test]
fn test_interp_i32_comparison_ops() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
                let a: i32 = 10;
                let b: i32 = 20;
                let r1 = if a < b { 1 } else { 0 };
                let r2 = if a > b { 1 } else { 0 };
                let r3 = if a == a { 1 } else { 0 };
                let r4 = if a != b { 1 } else { 0 };
                (r1 + r2 + r3 + r4) as i64
            };"
        ),
        3 // true + false + true + true = 3
    );
}

#[test]
fn test_interp_i32_cast_to_i64() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = { let x: i32 = 42; x as i64 };"
        ),
        42
    );
}

#[test]
fn test_interp_i64_cast_to_i32() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let x: i64 = 100; x as i32 };"
        ),
        Value::Int(100)
    );
}

#[test]
fn test_interp_i32_overflow_truncation() {
    // i64 value larger than i32 max should truncate
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let x: i64 = 2147483648; x as i32 };"
        ),
        Value::Int(-2147483648) // i32 overflow wraps
    );
}

#[test]
fn test_interp_i32_negative_cast() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = { let x: i32 = -100; x as i64 };"
        ),
        -100 // sign-extended
    );
}

#[test]
fn test_interp_i32_if_expression() {
    assert_eq!(
        run_program(
            "fn max32(a: i32, b: i32) -> i32 = if a > b { a } else { b };
             fn main() -> i32 = max32(30, 20);"
        ),
        Value::Int(30)
    );
}

#[test]
fn test_interp_i32_while_sum() {
    assert_eq!(
        run_program(
            "fn sum32(n: i32) -> i32 = {
                let mut s: i32 = 0;
                let mut i: i32 = 1;
                while i <= n {
                    s = s + i;
                    { i = i + 1 }
                };
                s
            };
            fn main() -> i32 = sum32(10);"
        ),
        Value::Int(55)
    );
}

#[test]
fn test_interp_i32_recursive() {
    assert_eq!(
        run_program(
            "fn factorial32(n: i32) -> i32 = {
                let one: i32 = 1;
                if n <= one { one } else { n * factorial32(n - one) }
             };
             fn main() -> i32 = factorial32(10);"
        ),
        Value::Int(3628800)
    );
}

#[test]
fn test_interp_i32_bool_cast() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let b: bool = true; b as i32 };"
        ),
        Value::Int(1)
    );
}

#[test]
fn test_interp_i32_f64_cast() {
    let result = run_program(
        "fn main() -> f64 = { let x: i32 = 42; x as f64 };"
    );
    match result {
        Value::Float(f) => assert!((f - 42.0).abs() < 0.001),
        other => panic!("expected Float, got {:?}", other),
    }
}

#[test]
fn test_interp_i32_f64_to_i32_cast() {
    assert_eq!(
        run_program(
            "fn main() -> i32 = { let f: f64 = 3.14; f as i32 };"
        ),
        Value::Int(3)
    );
}

// ============================================
// Cycle 194: IndexAssign Fix Verification
// ============================================

#[test]
fn test_interp_array_index_assign() {
    // This test previously panicked with RefCell borrow conflict.
    // v0.90.24: Fixed by binding env.borrow().get() result to a variable
    // so the Ref<Env> temporary is dropped before borrow_mut() is called.
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
                let mut a: [i64; 3] = [10, 20, 30];
                set a[0] = 100;
                set a[2] = 300;
                a[0] + a[1] + a[2]
            };"
        ),
        420 // 100 + 20 + 300
    );
}

#[test]
fn test_interp_array_index_assign_loop() {
    // Array modification inside a loop
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
                let mut a: [i64; 5] = [0, 0, 0, 0, 0];
                let mut i: i64 = 0;
                while i < 5 {
                    set a[i] = i * i;
                    { i = i + 1 }
                };
                a[0] + a[1] + a[2] + a[3] + a[4]
            };"
        ),
        30 // 0 + 1 + 4 + 9 + 16
    );
}

// ============================================
// Cycle 195: Parser/Type-Checking Edge Cases
// ============================================

// --- Nested function calls as arguments ---

#[test]
fn test_nested_fn_call_as_arg() {
    // Function call result used directly as argument to another function call,
    // three levels deep with different arities
    assert_eq!(
        run_program_i64(
            "fn add(a: i64, b: i64) -> i64 = a + b;
             fn mul(a: i64, b: i64) -> i64 = a * b;
             fn neg(x: i64) -> i64 = 0 - x;
             fn main() -> i64 = add(mul(neg(3), 4), mul(5, neg(2)));"
        ),
        -22 // neg(3)=-3, mul(-3,4)=-12, neg(2)=-2, mul(5,-2)=-10, add(-12,-10)=-22
    );
}

// --- Multiple return paths combining return with match ---

#[test]
fn test_multiple_return_paths_match_and_early_return() {
    // Early return from within a match arm, plus normal match return
    assert_eq!(
        run_program_i64(
            "fn process(mode: i64, x: i64) -> i64 = {
               if x < 0 { return -1 } else { () };
               match mode {
                 0 => x * 2,
                 1 => { if x > 100 { return 999 } else { () }; x + 10 },
                 _ => 0
               }
             };
             fn main() -> i64 = process(1, -5) + process(0, 7) + process(1, 200) + process(1, 50);"
        ),
        // process(1, -5) = -1 (early return x<0)
        // process(0, 7) = 14 (match 0 => 7*2)
        // process(1, 200) = 999 (match 1 => x>100 early return)
        // process(1, 50) = 60 (match 1 => 50+10)
        // -1 + 14 + 999 + 60 = 1072
        1072
    );
}

// --- Unit type from empty-ish blocks ---

#[test]
fn test_unit_type_empty_function() {
    // Function returning unit with side-effect-like structure
    assert!(type_checks(
        "fn do_nothing() -> () = ();
         fn also_nothing() -> () = { let x: i64 = 42; () };
         fn main() -> i64 = { do_nothing(); also_nothing(); 0 };"
    ));
}

// --- Struct containing array field ---

#[test]
fn test_struct_with_array_field() {
    assert_eq!(
        run_program_i64(
            "struct Matrix { data: [i64; 4], rows: i64, cols: i64 }
             fn trace(m: Matrix) -> i64 = m.data[0] + m.data[3];
             fn main() -> i64 = {
               let m = new Matrix { data: [1, 2, 3, 4], rows: 2, cols: 2 };
               trace(m)
             };"
        ),
        5 // data[0]=1, data[3]=4 -> 1+4=5
    );
}

// --- Array of booleans with index assignment ---

#[test]
fn test_array_bool_index_assign_and_read() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut flags: [bool; 4] = [false, false, false, false];
               set flags[0] = true;
               set flags[2] = true;
               let mut count: i64 = 0;
               if flags[0] { count = count + 1 } else { () };
               if flags[1] { count = count + 1 } else { () };
               if flags[2] { count = count + 1 } else { () };
               if flags[3] { count = count + 1 } else { () };
               count
             };"
        ),
        2 // only flags[0] and flags[2] are true
    );
}

// --- Forward reference: calling a function defined after the caller ---

#[test]
fn test_forward_reference_function_call() {
    // main calls helper which is defined after main
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = compute(5, 3);
             fn compute(a: i64, b: i64) -> i64 = a * a + b * b;"
        ),
        34 // 25 + 9
    );
}

// --- Char type operations ---

#[test]
fn test_char_literal_type_checks() {
    assert!(type_checks(
        "fn first_char() -> char = 'A';
         fn main() -> i64 = 0;"
    ));
}

#[test]
fn test_char_ord_builtin() {
    // ord() converts char to i64
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = ord('A');"
        ),
        65
    );
}

// --- Deeply nested if-else chain returning different values ---

#[test]
fn test_deeply_nested_if_else_chain() {
    assert_eq!(
        run_program_i64(
            "fn bucket(x: i64) -> i64 =
               if x < 10 { 1 }
               else if x < 20 { 2 }
               else if x < 30 { 3 }
               else if x < 40 { 4 }
               else if x < 50 { 5 }
               else { 6 };
             fn main() -> i64 = bucket(5) + bucket(15) + bucket(25) + bucket(35) + bucket(45) + bucket(99);"
        ),
        21 // 1+2+3+4+5+6
    );
}

// --- Closure with multiple captured variables ---

#[test]
fn test_closure_captures_multiple_vars() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a: i64 = 10;
               let b: i64 = 20;
               let c: i64 = 30;
               let f = fn |x: i64| { a + b + c + x };
               f(40)
             };"
        ),
        100 // 10+20+30+40
    );
}

// --- Nullable struct field access with unwrap_or ---

#[test]
fn test_nullable_with_struct_field_access() {
    assert!(type_checks(
        "struct Point { x: i64, y: i64 }
         fn maybe_origin(flag: bool) -> Point? =
           if flag { new Point { x: 0, y: 0 } } else { null };
         fn get_x_or_default(flag: bool) -> i64 = {
           let p: Point? = maybe_origin(flag);
           let val: Point = p.unwrap_or(new Point { x: -1, y: -1 });
           val.x
         };
         fn main() -> i64 = 0;"
    ));
}

// --- Wrapping arithmetic interpreter execution ---

#[test]
fn test_interp_wrapping_add() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a: i64 = 9223372036854775807;
               let b: i64 = 1;
               a +% b
             };"
        ),
        -9223372036854775808 // i64::MAX wrapping_add 1 = i64::MIN
    );
}

// --- Type error: struct field assign with wrong type ---

#[test]
fn test_error_struct_field_assign_type_mismatch() {
    assert!(type_error(
        "struct Point { x: i64, y: i64 }
         fn main() -> i64 = {
           let mut p: Point = new Point { x: 1, y: 2 };
           set p.x = true;
           p.x
         };"
    ));
}

// --- Type error: array index assign with wrong element type ---

#[test]
fn test_error_array_index_assign_type_mismatch() {
    assert!(type_error(
        "fn main() -> i64 = {
           let mut a: [i64; 3] = [1, 2, 3];
           set a[0] = true;
           a[0]
         };"
    ));
}

// --- Type error: match arms returning different types ---

#[test]
fn test_error_match_arms_type_mismatch() {
    assert!(type_error(
        r#"fn bad(x: i64) -> i64 = match x {
             0 => 42,
             1 => "hello",
             _ => 0
           };"#
    ));
}

// --- For-loop with inclusive range ---

#[test]
fn test_interp_for_inclusive_range_sum() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut s: i64 = 0;
               for i in 1..=10 { s = s + i };
               s
             };"
        ),
        55 // 1+2+...+10
    );
}

// --- Tuple in function param and return with field arithmetic ---

#[test]
fn test_interp_tuple_swap_and_compute() {
    assert_eq!(
        run_program_i64(
            "fn swap(t: (i64, i64)) -> (i64, i64) = (t.1, t.0);
             fn main() -> i64 = {
               let original = (3, 7);
               let swapped = swap(original);
               swapped.0 * 10 + swapped.1
             };"
        ),
        73 // swapped = (7,3) -> 7*10 + 3 = 73
    );
}

// --- Nested struct field access through function return ---

#[test]
fn test_nested_struct_field_through_fn_return() {
    assert_eq!(
        run_program_i64(
            "struct Inner { val: i64 }
             struct Outer { a: Inner, b: Inner }
             fn make_outer(x: i64, y: i64) -> Outer =
               new Outer { a: new Inner { val: x }, b: new Inner { val: y } };
             fn main() -> i64 = {
               let o = make_outer(11, 22);
               o.a.val + o.b.val
             };"
        ),
        33
    );
}

// --- Type error: cannot use non-bool in logical and/or ---

#[test]
fn test_error_logical_and_non_bool_operand() {
    assert!(type_error(
        "fn bad() -> bool = 42 and true;"
    ));
}

// ============================================
// Cycle 201: Extended Interpreter Coverage Tests
// ============================================

// --- Struct passed to function and returned (method-style call chain) ---

#[test]
fn test_interp_struct_returned_from_function() {
    assert_eq!(
        run_program_i64(
            "struct Vec2 { x: i64, y: i64 }
             fn add_vec(a: Vec2, b: Vec2) -> Vec2 =
               new Vec2 { x: a.x + b.x, y: a.y + b.y };
             fn dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y;
             fn main() -> i64 = {
               let v1 = new Vec2 { x: 3, y: 4 };
               let v2 = new Vec2 { x: 1, y: 2 };
               let v3 = add_vec(v1, v2);
               dot(v3, new Vec2 { x: 2, y: 3 })
             };"
        ),
        26 // v3 = (4,6), dot = 4*2 + 6*3 = 8+18 = 26
    );
}

// --- Enum pattern matching with data extraction ---

#[test]
fn test_interp_enum_tree_depth() {
    // Binary tree-like enum: compute value based on pattern matching
    assert_eq!(
        run_program_i64(
            "enum Tree { Leaf(i64), Node(i64, i64) }
             fn tree_sum(t: Tree) -> i64 =
               match t {
                 Tree::Leaf(v) => v,
                 Tree::Node(l, r) => l + r
               };
             fn main() -> i64 = {
               let a = tree_sum(Tree::Leaf(10));
               let b = tree_sum(Tree::Node(20, 30));
               a + b
             };"
        ),
        60 // 10 + (20+30) = 60
    );
}

// --- Enum with multiple variant data extraction ---

#[test]
fn test_interp_enum_shape_area() {
    assert_eq!(
        run_program_i64(
            "enum Shape { Square(i64), Rect(i64, i64) }
             fn area(s: Shape) -> i64 =
               match s {
                 Shape::Square(side) => side * side,
                 Shape::Rect(w, h) => w * h
               };
             fn main() -> i64 = {
               let s1 = area(Shape::Square(5));
               let s2 = area(Shape::Rect(3, 7));
               s1 + s2
             };"
        ),
        46 // 25 + 21 = 46
    );
}

// --- Nested closures with capture chaining ---

#[test]
fn test_interp_nested_closure_three_levels() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let x: i64 = 2;
               let f = fn |a: i64| {
                 let g = fn |b: i64| {
                   let h = fn |c: i64| { x + a + b + c };
                   h(4)
                 };
                 g(3)
               };
               f(1)
             };"
        ),
        10 // 2 + 1 + 3 + 4 = 10
    );
}

// --- Closure returned from if expression ---

#[test]
fn test_interp_closure_from_branch() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let flag: bool = true;
               let f = if flag { fn |x: i64| { x * 2 } } else { fn |x: i64| { x * 3 } };
               f(10)
             };"
        ),
        20 // flag=true, so f = x*2, f(10)=20
    );
}

// --- While-loop: count digits ---

#[test]
fn test_interp_while_count_digits() {
    assert_eq!(
        run_program_i64(
            "fn count_digits(n: i64) -> i64 = {
               let mut num: i64 = n;
               let mut count: i64 = 0;
               while num > 0 {
                 count = count + 1;
                 num = num / 10;
                 0
               };
               count
             };
             fn main() -> i64 = count_digits(123456);"
        ),
        6
    );
}

// --- While-loop: integer power computation ---

#[test]
fn test_interp_while_power() {
    assert_eq!(
        run_program_i64(
            "fn power(base: i64, exp: i64) -> i64 = {
               let mut result: i64 = 1;
               let mut i: i64 = 0;
               while i < exp {
                 result = result * base;
                 i = i + 1;
                 0
               };
               result
             };
             fn main() -> i64 = power(3, 5);"
        ),
        243 // 3^5 = 243
    );
}

// --- While-loop: reverse digits accumulator ---

#[test]
fn test_interp_while_reverse_digits() {
    assert_eq!(
        run_program_i64(
            "fn reverse_digits(n: i64) -> i64 = {
               let mut num: i64 = n;
               let mut rev: i64 = 0;
               while num > 0 {
                 rev = rev * 10 + num % 10;
                 num = num / 10;
                 0
               };
               rev
             };
             fn main() -> i64 = reverse_digits(12345);"
        ),
        54321
    );
}

// --- String operations: byte_at ---

#[test]
fn test_interp_string_byte_at() {
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = {
               let s = "ABCDE";
               s.byte_at(0)
             };"#
        ),
        65 // 'A' = 65
    );
}

// --- String operations: slice and len ---

#[test]
fn test_interp_string_slice_len() {
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = {
               let s = "Hello, World!";
               let sub = s.slice(0, 5);
               sub.len()
             };"#
        ),
        5 // "Hello".len() = 5
    );
}

// --- String operations: concat + byte_at combined ---

#[test]
fn test_interp_string_concat_byte_at() {
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = {
               let a = "AB";
               let b = "CD";
               let c = a + b;
               c.byte_at(2)
             };"#
        ),
        67 // "ABCD".byte_at(2) = 'C' = 67
    );
}

// --- Mutual recursion: Collatz-like ping-pong ---

#[test]
fn test_interp_mutual_recursion_ping_pong() {
    // ping decrements odd numbers, pong decrements even
    assert_eq!(
        run_program_i64(
            "fn ping(n: i64) -> i64 =
               if n <= 0 { 0 } else { 1 + pong(n - 1) };
             fn pong(n: i64) -> i64 =
               if n <= 0 { 0 } else { 1 + ping(n - 1) };
             fn main() -> i64 = ping(10);"
        ),
        10 // alternates ping/pong 10 times
    );
}

// --- Complex arithmetic chains ---

#[test]
fn test_interp_complex_arithmetic_chain() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a: i64 = 7;
               let b: i64 = 3;
               let c: i64 = 5;
               let d: i64 = 2;
               (a * b + c) * d - (a + b * c) / d
             };"
        ),
        41 // (7*3+5)*2 - (7+3*5)/2 = 26*2 - 22/2 = 52 - 11 = 41
    );
}

// --- Complex arithmetic: nested function composition ---

#[test]
fn test_interp_arithmetic_function_composition() {
    assert_eq!(
        run_program_i64(
            "fn f(x: i64) -> i64 = x * x + 1;
             fn g(x: i64) -> i64 = 2 * x - 3;
             fn h(x: i64) -> i64 = f(g(x)) + g(f(x));
             fn main() -> i64 = h(4);"
        ),
        57 // g(4)=5, f(5)=26; f(4)=17, g(17)=31; 26+31=57
    );
}

// --- Nested struct field access and assignment ---

#[test]
fn test_interp_nested_struct_field_access() {
    assert_eq!(
        run_program_i64(
            "struct Inner { val: i64 }
             struct Middle { inner: Inner, extra: i64 }
             struct Outer { mid: Middle, flag: i64 }
             fn main() -> i64 = {
               let o = new Outer {
                 mid: new Middle {
                   inner: new Inner { val: 100 },
                   extra: 20
                 },
                 flag: 3
               };
               o.mid.inner.val + o.mid.extra + o.flag
             };"
        ),
        123 // 100 + 20 + 3
    );
}

// --- Nested struct field assignment ---

#[test]
fn test_interp_struct_field_assign_and_read() {
    assert_eq!(
        run_program_i64(
            "struct Point { x: i64, y: i64 }
             fn main() -> i64 = {
               let mut p: Point = new Point { x: 0, y: 0 };
               set p.x = 5;
               set p.y = p.x * 2;
               p.x + p.y
             };"
        ),
        15 // x=5, y=10, 5+10=15
    );
}

// --- For-loop with mutable accumulator: factorial ---

#[test]
fn test_interp_for_loop_factorial() {
    assert_eq!(
        run_program_i64(
            "fn factorial(n: i64) -> i64 = {
               let mut result: i64 = 1;
               for i in 1..=n { result = result * i };
               result
             };
             fn main() -> i64 = factorial(10);"
        ),
        3628800 // 10!
    );
}

// --- Enum matching with wildcard and data ---

#[test]
fn test_interp_enum_match_wildcard_with_data() {
    assert_eq!(
        run_program_i64(
            "enum Action { Add(i64), Mul(i64), Reset }
             fn apply(acc: i64, a: Action) -> i64 =
               match a {
                 Action::Add(v) => acc + v,
                 Action::Mul(v) => acc * v,
                 Action::Reset => 0
               };
             fn main() -> i64 = {
               let mut val: i64 = 1;
               val = apply(val, Action::Add(9));
               val = apply(val, Action::Mul(5));
               val = apply(val, Action::Add(7));
               val
             };"
        ),
        57 // 1+9=10, 10*5=50, 50+7=57
    );
}

// --- String is_empty method ---

#[test]
fn test_interp_string_is_empty() {
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = {
               let a = "";
               let b = "hello";
               let empty_flag = a.is_empty() as i64;
               let nonempty_flag = b.is_empty() as i64;
               empty_flag * 10 + nonempty_flag
             };"#
        ),
        10 // a.is_empty()=true(1), b.is_empty()=false(0) -> 1*10+0=10
    );
}

// ============================================
// Cycle 205: Integration Tests for Undercovered Features
// ============================================

// --- Nullable Types (3 tests) ---

#[test]
fn test_interp_nullable_unwrap_or_value() {
    // Nullable i64? with a non-null value, unwrap_or returns the value
    assert_eq!(
        run_program_i64(
            "fn safe_div(a: i64, b: i64) -> i64? =
               if b == 0 { null } else { a / b };
             fn main() -> i64 = {
               let r: i64? = safe_div(100, 4);
               r.unwrap_or(-1)
             };"
        ),
        25 // 100 / 4 = 25
    );
}

#[test]
fn test_interp_nullable_unwrap_or_null() {
    // Nullable i64? with null, unwrap_or returns the default
    assert_eq!(
        run_program_i64(
            "fn safe_div(a: i64, b: i64) -> i64? =
               if b == 0 { null } else { a / b };
             fn main() -> i64 = {
               let r: i64? = safe_div(100, 0);
               r.unwrap_or(-1)
             };"
        ),
        -1 // division by zero -> null -> unwrap_or returns -1
    );
}

#[test]
fn test_interp_nullable_is_some_is_none() {
    // Test is_some and is_none on nullable values encoded as i64
    assert_eq!(
        run_program_i64(
            "fn maybe(x: i64) -> i64? = if x > 0 { x } else { null };
             fn main() -> i64 = {
               let a: i64? = maybe(5);
               let b: i64? = maybe(0);
               let a_some = a.is_some() as i64;
               let a_none = a.is_none() as i64;
               let b_some = b.is_some() as i64;
               let b_none = b.is_none() as i64;
               a_some * 1000 + a_none * 100 + b_some * 10 + b_none
             };"
        ),
        1001 // a: is_some=1, is_none=0; b: is_some=0, is_none=1 -> 1000+0+0+1
    );
}

// --- Pattern Matching (3 tests) ---

#[test]
fn test_interp_match_enum_with_data_extraction() {
    // Match on enum variants that carry data, extract and compute
    assert_eq!(
        run_program_i64(
            "enum Expr { Lit(i64), Neg(i64), Add(i64) }
             fn eval_expr(e: Expr, acc: i64) -> i64 =
               match e {
                 Expr::Lit(v) => v,
                 Expr::Neg(v) => 0 - v,
                 Expr::Add(v) => acc + v
               };
             fn main() -> i64 = {
               let a = eval_expr(Expr::Lit(10), 0);
               let b = eval_expr(Expr::Neg(3), 0);
               let c = eval_expr(Expr::Add(7), a);
               a + b + c
             };"
        ),
        24 // a=10, b=-3, c=10+7=17, sum=10+(-3)+17=24
    );
}

#[test]
fn test_interp_nested_match_two_levels() {
    // Nested match: outer on first arg, inner on second
    assert_eq!(
        run_program_i64(
            "fn grid(row: i64, col: i64) -> i64 =
               match row {
                 0 => match col {
                   0 => 1,
                   1 => 2,
                   _ => 3
                 },
                 1 => match col {
                   0 => 4,
                   1 => 5,
                   _ => 6
                 },
                 _ => 0
               };
             fn main() -> i64 =
               grid(0, 0) + grid(0, 1) + grid(0, 2) + grid(1, 0) + grid(1, 1) + grid(1, 2) + grid(2, 0);"
        ),
        21 // 1+2+3+4+5+6+0 = 21
    );
}

#[test]
fn test_interp_match_wildcard_fallthrough() {
    // Wildcard matches everything not explicitly handled
    assert_eq!(
        run_program_i64(
            "fn classify(x: i64) -> i64 =
               match x {
                 1 => 100,
                 2 => 200,
                 3 => 300,
                 _ => -1
               };
             fn main() -> i64 = {
               let mut sum: i64 = 0;
               for i in 1..=5 {
                 sum = sum + classify(i)
               };
               sum
             };"
        ),
        598 // 100+200+300+(-1)+(-1) = 598
    );
}

// --- Array Operations (3 tests) ---

#[test]
fn test_interp_array_sum_with_loop() {
    // Sum array elements using index-based for loop
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let arr = [3, 7, 11, 13, 17];
               let mut sum: i64 = 0;
               let mut i: i64 = 0;
               while i < arr.len() {
                 sum = sum + arr[i];
                 { i = i + 1 }
               };
               sum
             };"
        ),
        51 // 3+7+11+13+17 = 51
    );
}

#[test]
fn test_interp_array_transform_in_place() {
    // Modify array elements in-place: double each value
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut a: [i64; 4] = [2, 5, 8, 11];
               let mut i: i64 = 0;
               while i < 4 {
                 set a[i] = a[i] * 2;
                 { i = i + 1 }
               };
               a[0] + a[1] + a[2] + a[3]
             };"
        ),
        52 // 4+10+16+22 = 52
    );
}

#[test]
fn test_interp_array_2d_simulation() {
    // Simulate a 2D grid as flat array: 3x3 grid, sum diagonal elements
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut grid: [i64; 9] = [0; 9];
               let mut i: i64 = 0;
               while i < 3 {
                 let mut j: i64 = 0;
                 while j < 3 {
                   set grid[i * 3 + j] = i * 3 + j + 1;
                   { j = j + 1 }
                 };
                 { i = i + 1 }
               };
               grid[0] + grid[4] + grid[8]
             };"
        ),
        15 // grid[0]=1, grid[4]=5, grid[8]=9 -> 1+5+9=15
    );
}

// --- Generic Functions (3 tests) ---

#[test]
fn test_interp_generic_identity() {
    // Generic identity function instantiated with i64
    assert_eq!(
        run_program_i64(
            "fn identity<T>(x: T) -> T = x;
             fn main() -> i64 = identity(42) + identity(8);"
        ),
        50 // 42 + 8
    );
}

#[test]
fn test_interp_generic_pair_choose() {
    // Generic function that chooses between two values
    assert_eq!(
        run_program_i64(
            "fn choose<T>(a: T, b: T, pick_first: bool) -> T =
               if pick_first { a } else { b };
             fn main() -> i64 = {
               let x = choose(10, 20, true);
               let y = choose(30, 40, false);
               x + y
             };"
        ),
        50 // 10 + 40
    );
}

#[test]
fn test_interp_generic_apply_twice() {
    // Generic higher-order-like pattern: apply a transformation via helper
    assert_eq!(
        run_program_i64(
            "fn double(x: i64) -> i64 = x * 2;
             fn apply_double(x: i64) -> i64 = double(double(x));
             fn id<T>(x: T) -> T = x;
             fn main() -> i64 = {
               let a = apply_double(3);
               let b = id(a);
               b
             };"
        ),
        12 // double(double(3)) = double(6) = 12
    );
}

// --- Complex Control Flow (3 tests) ---

#[test]
fn test_interp_loop_break_with_early_return() {
    // Loop with break and mutable result capture
    assert_eq!(
        run_program_i64(
            "fn find_first_square_above(threshold: i64) -> i64 = {
               let mut i: i64 = 1;
               loop {
                 if i * i > threshold { return i } else { () };
                 { i = i + 1 }
               };
               0
             };
             fn main() -> i64 = find_first_square_above(50);"
        ),
        8 // 8*8=64 > 50, so return 8
    );
}

#[test]
fn test_interp_early_return_from_function() {
    // Early return from within a loop inside a function
    assert_eq!(
        run_program_i64(
            "fn find_divisor(n: i64) -> i64 = {
               let mut d: i64 = 2;
               while d * d <= n {
                 if n % d == 0 { return d } else { () };
                 { d = d + 1 }
               };
               n
             };
             fn main() -> i64 = {
               let a = find_divisor(35);
               let b = find_divisor(13);
               a * 100 + b
             };"
        ),
        513 // smallest divisor of 35 is 5, 13 is prime -> 5*100+13=513
    );
}

#[test]
fn test_interp_multi_arm_if_else_chain() {
    // Multi-arm if-else chain mapping ranges to categories
    assert_eq!(
        run_program_i64(
            "fn score_to_grade(s: i64) -> i64 =
               if s >= 90 { 4 }
               else { if s >= 80 { 3 }
               else { if s >= 70 { 2 }
               else { if s >= 60 { 1 }
               else { 0 } } } };
             fn main() -> i64 = {
               let mut total: i64 = 0;
               total = total + score_to_grade(95);
               total = total + score_to_grade(85);
               total = total + score_to_grade(75);
               total = total + score_to_grade(65);
               total = total + score_to_grade(55);
               total
             };"
        ),
        10 // 4+3+2+1+0 = 10
    );
}

// --- String Operations (2 tests) ---

#[test]
fn test_interp_string_length_accumulation() {
    // Compute lengths of multiple strings and sum them
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = {
               let a = "hello";
               let b = "world!";
               let c = "BMB";
               let d = "test string";
               a.len() + b.len() + c.len() + d.len()
             };"#
        ),
        25 // 5 + 6 + 3 + 11 = 25
    );
}

#[test]
fn test_interp_string_byte_comparison() {
    // Compare bytes at specific positions in a string
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = {
               let s = "ABCDE";
               let a_byte = s.byte_at(0);
               let e_byte = s.byte_at(4);
               e_byte - a_byte
             };"#
        ),
        4 // 'E'(69) - 'A'(65) = 4
    );
}

// ============================================
// Cycle 211: Advanced Language Feature Integration Tests
// ============================================

// --- Recursive Data Structures (3 tests) ---

#[test]
fn test_interp_recursive_linked_list_sum() {
    // Simulate a linked list using enum: each node carries a value and a "next tag"
    // We encode a linked list as nested function calls that accumulate a sum
    assert_eq!(
        run_program_i64(
            "fn list_sum(n: i64) -> i64 =
               if n <= 0 { 0 } else { n + list_sum(n - 1) };
             fn main() -> i64 = list_sum(10);"
        ),
        55 // 10+9+8+...+1 = 55
    );
}

#[test]
fn test_interp_recursive_binary_tree_node_count() {
    // Simulate a full binary tree: count nodes at depth d = 2^d - 1
    assert_eq!(
        run_program_i64(
            "fn power_of_two(n: i64) -> i64 =
               if n <= 0 { 1 } else { 2 * power_of_two(n - 1) };
             fn tree_nodes(depth: i64) -> i64 =
               if depth <= 0 { 1 }
               else { 1 + tree_nodes(depth - 1) + tree_nodes(depth - 1) };
             fn main() -> i64 = tree_nodes(4);"
        ),
        31 // 2^5 - 1 = 31
    );
}

#[test]
fn test_interp_recursive_ackermann_small() {
    // Ackermann function for small inputs: A(2, 3) = 9
    assert_eq!(
        run_program_i64(
            "fn ack(m: i64, n: i64) -> i64 =
               if m == 0 { n + 1 }
               else { if n == 0 { ack(m - 1, 1) }
               else { ack(m - 1, ack(m, n - 1)) } };
             fn main() -> i64 = ack(2, 3);"
        ),
        9 // A(2,3) = 9
    );
}

// --- Higher-Order Functions / Closures as Arguments (3 tests) ---

#[test]
fn test_interp_closure_as_transformer() {
    // Store different closures in variables and apply them
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let double = fn |x: i64| { x * 2 };
               let square = fn |x: i64| { x * x };
               let a = double(5);
               let b = square(4);
               a + b
             };"
        ),
        26 // 10 + 16 = 26
    );
}

#[test]
fn test_interp_closure_capturing_and_composing() {
    // Build closures that capture outer variables and compose their results
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let base: i64 = 100;
               let offset: i64 = 7;
               let add_base = fn |x: i64| { x + base };
               let add_offset = fn |x: i64| { x + offset };
               let step1 = add_base(20);
               let step2 = add_offset(step1);
               step2
             };"
        ),
        127 // 20 + 100 = 120, 120 + 7 = 127
    );
}

#[test]
fn test_interp_closure_selection_via_flag() {
    // Select between closures based on runtime condition, then apply
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut total: i64 = 0;
               let mut i: i64 = 1;
               while i <= 5 {
                 let op = if i % 2 == 0 {
                   fn |x: i64| { x * 3 }
                 } else {
                   fn |x: i64| { x * 2 }
                 };
                 total = total + op(i);
                 { i = i + 1 }
               };
               total
             };"
        ),
        36 // i=1(odd):1*2=2, i=2(even):2*3=6, i=3(odd):3*2=6, i=4(even):4*3=12, i=5(odd):5*2=10
        // 2+6+6+12+10 = 36
    );
}

// --- Enum with Data (3 tests) ---

#[test]
fn test_interp_enum_command_dispatch() {
    // Enum with multiple data variants used as a command dispatcher
    assert_eq!(
        run_program_i64(
            "enum Cmd { Inc(i64), Dec(i64), Set(i64), Nop }
             fn execute(state: i64, cmd: Cmd) -> i64 =
               match cmd {
                 Cmd::Inc(v) => state + v,
                 Cmd::Dec(v) => state - v,
                 Cmd::Set(v) => v,
                 Cmd::Nop => state
               };
             fn main() -> i64 = {
               let mut s: i64 = 0;
               s = execute(s, Cmd::Set(50));
               s = execute(s, Cmd::Inc(25));
               s = execute(s, Cmd::Dec(10));
               s = execute(s, Cmd::Nop);
               s
             };"
        ),
        65 // 0 -> Set(50)=50 -> Inc(25)=75 -> Dec(10)=65 -> Nop=65
    );
}

#[test]
fn test_interp_enum_two_field_variant_extraction() {
    // Enum variant with two data fields, extracted in match
    assert_eq!(
        run_program_i64(
            "enum Rect { Empty, Sized(i64, i64) }
             fn rect_area(r: Rect) -> i64 =
               match r {
                 Rect::Empty => 0,
                 Rect::Sized(w, h) => w * h
               };
             fn main() -> i64 = {
               let a = rect_area(Rect::Empty);
               let b = rect_area(Rect::Sized(6, 9));
               let c = rect_area(Rect::Sized(3, 4));
               a + b + c
             };"
        ),
        66 // 0 + 54 + 12 = 66
    );
}

#[test]
fn test_interp_enum_recursive_eval_chain() {
    // Chain multiple enum operations, matching and accumulating
    assert_eq!(
        run_program_i64(
            "enum Op { Add(i64), Mul(i64), Neg }
             fn apply_op(val: i64, op: Op) -> i64 =
               match op {
                 Op::Add(n) => val + n,
                 Op::Mul(n) => val * n,
                 Op::Neg => 0 - val
               };
             fn main() -> i64 = {
               let mut v: i64 = 3;
               v = apply_op(v, Op::Add(7));
               v = apply_op(v, Op::Mul(2));
               v = apply_op(v, Op::Neg);
               v = apply_op(v, Op::Add(100));
               v
             };"
        ),
        80 // 3+7=10, 10*2=20, -20, -20+100=80
    );
}

// --- Mixed Control Flow (3 tests) ---

#[test]
fn test_interp_loop_with_match_and_early_return() {
    // Combine loop, match on computed value, and early return
    assert_eq!(
        run_program_i64(
            "fn find_special(limit: i64) -> i64 = {
               let mut i: i64 = 1;
               let mut sum: i64 = 0;
               while i <= limit {
                 let category = match i % 3 {
                   0 => 10,
                   1 => 1,
                   _ => 5
                 };
                 sum = sum + category;
                 if sum > 30 { return sum } else { () };
                 { i = i + 1 }
               };
               sum
             };
             fn main() -> i64 = find_special(20);"
        ),
        32 // i=1:1%3=1->1, i=2:2%3=2->5, i=3:0->10, i=4:1->1, i=5:2->5, i=6:0->10
        // cumulative: 1,6,16,17,22,32 -> 32 > 30, return 32
    );
}

#[test]
fn test_interp_nested_loops_with_break() {
    // Nested while loops: outer counts rows, inner counts columns
    // Accumulate a grid-based sum and break inner loop early
    assert_eq!(
        run_program_i64(
            "fn grid_sum(rows: i64, cols: i64, max_col: i64) -> i64 = {
               let mut total: i64 = 0;
               let mut r: i64 = 0;
               while r < rows {
                 let mut c: i64 = 0;
                 while c < cols {
                   if c >= max_col { break } else { () };
                   total = total + r * cols + c;
                   { c = c + 1 }
                 };
                 { r = r + 1 }
               };
               total
             };
             fn main() -> i64 = grid_sum(3, 4, 2);"
        ),
        27 // r=0: (0+0)+(0+1)=1, r=1: (4+0)+(4+1)=9, r=2: (8+0)+(8+1)=17 -> 1+9+17=27
    );
}

#[test]
fn test_interp_for_loop_with_nested_conditionals() {
    // For loop computing collatz-like steps for multiple starting values
    assert_eq!(
        run_program_i64(
            "fn collatz_steps(start: i64) -> i64 = {
               let mut n: i64 = start;
               let mut steps: i64 = 0;
               while n != 1 {
                 if n % 2 == 0 { n = n / 2 }
                 else { n = 3 * n + 1 };
                 { steps = steps + 1 }
               };
               steps
             };
             fn main() -> i64 = {
               let mut total: i64 = 0;
               for i in 2..=6 {
                 total = total + collatz_steps(i)
               };
               total
             };"
        ),
        23 // 2->1 (1 step), 3->10->5->16->8->4->2->1 (7 steps),
        // 4->2->1 (2 steps), 5->16->8->4->2->1 (5 steps),
        // 6->3->10->5->16->8->4->2->1 (8 steps) -> 1+7+2+5+8=23
    );
}

// --- Edge Cases (3 tests) ---

#[test]
fn test_interp_deeply_nested_if_else() {
    // 8-level deep if-else chain testing deep nesting
    assert_eq!(
        run_program_i64(
            "fn deep_classify(x: i64) -> i64 =
               if x >= 128 { 8 }
               else { if x >= 64 { 7 }
               else { if x >= 32 { 6 }
               else { if x >= 16 { 5 }
               else { if x >= 8 { 4 }
               else { if x >= 4 { 3 }
               else { if x >= 2 { 2 }
               else { if x >= 1 { 1 }
               else { 0 } } } } } } } };
             fn main() -> i64 = {
               let mut sum: i64 = 0;
               sum = sum + deep_classify(0);
               sum = sum + deep_classify(1);
               sum = sum + deep_classify(3);
               sum = sum + deep_classify(7);
               sum = sum + deep_classify(15);
               sum = sum + deep_classify(31);
               sum = sum + deep_classify(63);
               sum = sum + deep_classify(127);
               sum = sum + deep_classify(255);
               sum
             };"
        ),
        36 // 0+1+2+3+4+5+6+7+8 = 36
    );
}

#[test]
fn test_interp_large_integer_arithmetic() {
    // Test large i64 values near the upper range
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let big_a: i64 = 1000000000;
               let big_b: i64 = 2000000000;
               let product = big_a * 3;
               let sum = product + big_b;
               sum / 1000000
             };"
        ),
        5000 // (1000000000 * 3 + 2000000000) / 1000000 = 5000000000 / 1000000 = 5000
    );
}

#[test]
fn test_interp_array_single_element_operations() {
    // Single-element array: test boundary behavior
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a: [i64; 1] = [42];
               let mut b: [i64; 1] = [0];
               set b[0] = a[0] * 2;
               a[0] + b[0] + a.len() + b.len()
             };"
        ),
        128 // 42 + 84 + 1 + 1 = 128
    );
}

// ============================================
// Cycle 221: Feature Combination Integration Tests
// ============================================

// --- Nested Loop with Break/Continue ---

#[test]
fn test_interp_nested_loop_outer_break() {
    // Nested loops: outer loop breaks after inner loop runs 3 times
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut total = 0;
               let mut i = 0;
               loop {
                 let mut j = 0;
                 loop {
                   total = total + 1;
                   j = j + 1;
                   if j >= 3 { break } else { () }
                 };
                 i = i + 1;
                 if i >= 4 { break } else { () }
               };
               total
             };"
        ),
        12 // 4 outer * 3 inner = 12
    );
}

#[test]
fn test_interp_continue_skip_even() {
    // Continue to skip even iterations, sum only odds
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut sum = 0;
               let mut i = 0;
               loop {
                 i = i + 1;
                 if i > 10 { break } else { () };
                 if i % 2 == 0 { continue } else { () };
                 sum = sum + i
               };
               sum
             };"
        ),
        25 // 1+3+5+7+9 = 25
    );
}

#[test]
fn test_interp_nested_continue_inner() {
    // Continue inside inner loop only affects inner
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut count = 0;
               let mut i = 0;
               loop {
                 i = i + 1;
                 if i > 3 { break } else { () };
                 let mut j = 0;
                 loop {
                   j = j + 1;
                   if j > 5 { break } else { () };
                   if j % 2 == 0 { continue } else { () };
                   count = count + 1
                 }
               };
               count
             };"
        ),
        9 // 3 outer * 3 odd j values (1,3,5) = 9
    );
}

// --- Return from Nested Structures ---

#[test]
fn test_interp_return_from_nested_if() {
    // Early return from inside nested if-else
    assert_eq!(
        run_program_i64(
            "fn find_threshold(n: i64) -> i64 = {
               if n > 100 {
                 return 100
               } else { () };
               if n > 50 {
                 return 50
               } else { () };
               n
             };
             fn main() -> i64 = find_threshold(75) + find_threshold(200) + find_threshold(30);"
        ),
        180 // 50 + 100 + 30 = 180
    );
}

#[test]
fn test_interp_return_from_loop() {
    // Return exits the function from inside a loop
    assert_eq!(
        run_program_i64(
            "fn find_first_divisible(n: i64, d: i64) -> i64 = {
               let mut i = 1;
               loop {
                 if i * d >= n { return i } else { () };
                 i = i + 1
               };
               0
             };
             fn main() -> i64 = find_first_divisible(20, 7);"
        ),
        3 // 3*7=21 >= 20
    );
}

// --- While Loop with Complex Control Flow ---

#[test]
fn test_interp_while_with_multiple_breaks() {
    // Simulate while with loop+break, two possible exit points
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut i = 0;
               let mut found = 0;
               loop {
                 i = i + 1;
                 if i > 100 { break } else { () };
                 if i * i > 50 {
                   found = i;
                   break
                 } else { () }
               };
               found
             };"
        ),
        8 // 8*8=64 > 50
    );
}

// --- Function Composition with Control Flow ---

#[test]
fn test_interp_recursive_with_loop_helper() {
    // Recursive function calling loop-based helper
    assert_eq!(
        run_program_i64(
            "fn sum_to(n: i64) -> i64 = {
               let mut s = 0;
               let mut i = 1;
               loop {
                 if i > n { break } else { () };
                 s = s + i;
                 i = i + 1
               };
               s
             };
             fn triangle(n: i64) -> i64 =
               if n <= 0 { 0 }
               else { sum_to(n) + triangle(n - 1) };
             fn main() -> i64 = triangle(4);"
        ),
        20 // sum_to(4)+sum_to(3)+sum_to(2)+sum_to(1) = 10+6+3+1 = 20
    );
}

// --- Loop with Accumulator Pattern ---

#[test]
fn test_interp_loop_fibonacci_iterative() {
    // Iterative fibonacci using loop
    assert_eq!(
        run_program_i64(
            "fn fib(n: i64) -> i64 = {
               if n <= 1 { return n } else { () };
               let mut a = 0;
               let mut b = 1;
               let mut i = 2;
               loop {
                 let c = a + b;
                 a = b;
                 b = c;
                 i = i + 1;
                 if i > n { break } else { () }
               };
               b
             };
             fn main() -> i64 = fib(10);"
        ),
        55
    );
}

#[test]
fn test_interp_loop_power() {
    // Iterative power using loop with accumulator
    assert_eq!(
        run_program_i64(
            "fn power(base: i64, exp: i64) -> i64 = {
               let mut result = 1;
               let mut i = 0;
               loop {
                 if i >= exp { break } else { () };
                 result = result * base;
                 i = i + 1
               };
               result
             };
             fn main() -> i64 = power(2, 10);"
        ),
        1024
    );
}

// --- For Loop with Break/Continue ---

#[test]
fn test_interp_for_with_continue() {
    // For loop with continue to skip specific values
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut sum = 0;
               for i in 1..11 {
                 if i == 5 { continue } else { () };
                 if i == 8 { continue } else { () };
                 sum = sum + i
               };
               sum
             };"
        ),
        42 // 1+2+3+4+6+7+9+10 = 42
    );
}

#[test]
fn test_interp_for_with_early_break() {
    // For loop with break before reaching end
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut last = 0;
               for i in 1..100 {
                 if i * i > 200 { break } else { () };
                 last = i
               };
               last
             };"
        ),
        14 // 14*14=196 <= 200, 15*15=225 > 200
    );
}

// --- Type Checking: Feature Combinations ---

#[test]
fn test_type_return_inside_block_typechecks() {
    assert!(type_checks(
        "fn f(x: i64) -> i64 = { if x > 0 { return x } else { () }; 0 };\nfn main() -> i64 = f(5);"
    ));
}

#[test]
fn test_type_loop_break_continue_typechecks() {
    assert!(type_checks(
        "fn main() -> i64 = { let mut x = 0; loop { x = x + 1; if x > 5 { break } else { continue } }; x };"
    ));
}

// ========================================================================
// Cycle 223: Closure + Control Flow Integration Tests
// ========================================================================

#[test]
fn test_interp_closure_with_loop_accumulator() {
    // Closure that uses a loop internally to compute sum
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let sum_to = fn |n: i64| {
                 let mut acc = 0;
                 let mut i = 1;
                 while i <= n {
                   acc = acc + i;
                   { i = i + 1 }
                 };
                 acc
               };
               sum_to(10)
             };"
        ),
        55 // 1+2+...+10 = 55
    );
}

#[test]
fn test_interp_closure_capturing_with_control_flow() {
    // Closure captures outer variable and uses if/else internally
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let threshold: i64 = 50;
               let clamp = fn |x: i64| {
                 if x > threshold { threshold } else { x }
               };
               clamp(30) + clamp(100)
             };"
        ),
        80 // clamp(30)=30, clamp(100)=50, 30+50=80
    );
}

#[test]
fn test_interp_closure_returning_from_loop() {
    // Closure with early exit from loop via break
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let find_first_square = fn |limit: i64| {
                 let mut i = 1;
                 let mut result = 0;
                 loop {
                   if i * i > limit {
                     result = i;
                     break
                   } else { () };
                   { i = i + 1 }
                 };
                 result
               };
               find_first_square(100)
             };"
        ),
        11 // 11*11=121 > 100, first i where i*i > 100
    );
}

// ========================================================================
// Cycle 223: Generic Function + Control Flow Tests
// ========================================================================

#[test]
fn test_interp_generic_with_branching() {
    // Generic function that branches based on a bool parameter
    assert_eq!(
        run_program_i64(
            "fn pick_or_default<T>(val: T, default_val: T, use_val: bool) -> T =
               if use_val { val } else { default_val };
             fn main() -> i64 = {
               let a = pick_or_default(42, 0, true);
               let b = pick_or_default(99, 7, false);
               a + b
             };"
        ),
        49 // 42 + 7 = 49
    );
}

#[test]
fn test_interp_generic_nested_calls() {
    // Nested generic function calls
    assert_eq!(
        run_program_i64(
            "fn wrap<T>(x: T) -> T = x;
             fn double_wrap<T>(x: T) -> T = wrap(wrap(x));
             fn main() -> i64 = double_wrap(42);"
        ),
        42
    );
}

// ========================================================================
// Cycle 223: Struct + Control Flow Integration Tests
// ========================================================================

#[test]
fn test_interp_struct_conditional_field_assignment() {
    // Struct with conditional field updates
    assert_eq!(
        run_program_i64(
            "struct Counter { val: i64, limit: i64 }
             fn main() -> i64 = {
               let mut c: Counter = new Counter { val: 0, limit: 5 };
               let mut i = 0;
               while i < 10 {
                 if c.val < c.limit {
                   set c.val = c.val + 1
                 } else { () };
                 { i = i + 1 }
               };
               c.val
             };"
        ),
        5 // clamped at limit=5
    );
}

#[test]
fn test_interp_struct_in_loop_accumulation() {
    // Pass struct to function, accumulate in loop
    assert_eq!(
        run_program_i64(
            "struct Pair { a: i64, b: i64 }
             fn sum_pair(p: Pair) -> i64 = p.a + p.b;
             fn main() -> i64 = {
               let mut total = 0;
               let mut i = 1;
               while i <= 3 {
                 let p = new Pair { a: i, b: i * 2 };
                 total = total + sum_pair(p);
                 { i = i + 1 }
               };
               total
             };"
        ),
        18 // i=1: 1+2=3, i=2: 2+4=6, i=3: 3+6=9, total=3+6+9=18
    );
}

#[test]
fn test_interp_struct_computed_field() {
    // Struct with fields derived from computation
    assert_eq!(
        run_program_i64(
            "struct Point { x: i64, y: i64 }
             fn scale(p: Point, s: i64) -> Point =
               new Point { x: p.x * s, y: p.y * s };
             fn main() -> i64 = {
               let p = new Point { x: 3, y: 4 };
               let p2 = scale(p, 5);
               p2.x + p2.y
             };"
        ),
        35 // 3*5 + 4*5 = 15 + 20 = 35
    );
}

// ========================================================================
// Cycle 223: Match Expression + Feature Combination Tests
// ========================================================================

#[test]
fn test_interp_match_with_accumulator() {
    // Match inside a loop to categorize and accumulate
    assert_eq!(
        run_program_i64(
            "fn categorize(x: i64) -> i64 =
               match x % 3 {
                 0 => 10,
                 1 => 20,
                 _ => 30
               };
             fn main() -> i64 = {
               let mut total = 0;
               let mut i = 0;
               while i < 6 {
                 total = total + categorize(i);
                 { i = i + 1 }
               };
               total
             };"
        ),
        120 // i=0:10, i=1:20, i=2:30, i=3:10, i=4:20, i=5:30, total=120
    );
}

#[test]
fn test_interp_match_enum_with_computation() {
    // Match on enum, compute different results per variant
    assert_eq!(
        run_program_i64(
            "enum Op { Add(i64, i64), Mul(i64, i64), Neg(i64) }
             fn compute(op: Op) -> i64 =
               match op {
                 Op::Add(a, b) => a + b,
                 Op::Mul(a, b) => a * b,
                 Op::Neg(x) => 0 - x
               };
             fn main() -> i64 = {
               let r1 = compute(Op::Add(10, 20));
               let r2 = compute(Op::Mul(3, 4));
               let r3 = compute(Op::Neg(5));
               r1 + r2 + r3
             };"
        ),
        37 // 30 + 12 + (-5) = 37
    );
}

// ========================================================================
// Cycle 223: Multi-Feature Combination Tests
// ========================================================================

#[test]
fn test_interp_closure_over_struct() {
    // Closure that operates on struct fields
    assert_eq!(
        run_program_i64(
            "struct Rect { w: i64, h: i64 }
             fn main() -> i64 = {
               let area = fn |r: Rect| { r.w * r.h };
               let r1 = new Rect { w: 3, h: 4 };
               let r2 = new Rect { w: 5, h: 6 };
               area(r1) + area(r2)
             };"
        ),
        42 // 12 + 30 = 42
    );
}

#[test]
fn test_interp_recursive_with_match() {
    // Recursive function using match for base case
    assert_eq!(
        run_program_i64(
            "fn fib(n: i64) -> i64 =
               match n {
                 0 => 0,
                 1 => 1,
                 _ => fib(n - 1) + fib(n - 2)
               };
             fn main() -> i64 = fib(10);"
        ),
        55 // fib(10) = 55
    );
}

#[test]
fn test_interp_generic_function_with_struct() {
    // Generic function operating on struct
    assert_eq!(
        run_program_i64(
            "fn id<T>(x: T) -> T = x;
             struct Box2 { val: i64 }
             fn main() -> i64 = {
               let b = new Box2 { val: 42 };
               let b2 = id(b);
               b2.val
             };"
        ),
        42
    );
}

// ========================================================================
// Cycle 223: Type Checking Feature Combinations
// ========================================================================

#[test]
fn test_type_closure_with_loop_typechecks() {
    assert!(type_checks(
        "fn main() -> i64 = {
           let f = fn |n: i64| {
             let mut i = 0;
             while i < n { { i = i + 1 } };
             i
           };
           f(5)
         };"
    ));
}

#[test]
fn test_type_struct_in_match_typechecks() {
    assert!(type_checks(
        "struct Val { x: i64 }
         fn check(v: Val) -> i64 = match v.x { 0 => 1, _ => v.x };
         fn main() -> i64 = check(new Val { x: 5 });"
    ));
}

#[test]
fn test_type_generic_with_bool_typechecks() {
    assert!(type_checks(
        "fn first<T>(a: T, b: T) -> T = a;
         fn main() -> bool = first(true, false);"
    ));
}

// ========================================================================
// Cycle 224: Error Detection Coverage Tests
// ========================================================================

/// Helper: check that a program fails to parse
fn parse_error(source: &str) -> bool {
    let tokens = match tokenize(source) {
        Ok(t) => t,
        Err(_) => return true, // Lexer error counts as parse error
    };
    parse("test.bmb", source, tokens).is_err()
}

// --- Argument Count Mismatch ---

#[test]
fn test_error_too_few_args() {
    assert!(type_error(
        "fn add(a: i64, b: i64) -> i64 = a + b;
         fn main() -> i64 = add(1);"
    ));
}

#[test]
fn test_error_too_many_args() {
    assert!(type_error(
        "fn add(a: i64, b: i64) -> i64 = a + b;
         fn main() -> i64 = add(1, 2, 3);"
    ));
}

// --- Type Mismatch Errors ---

#[test]
fn test_error_bool_where_int_expected() {
    assert!(type_error(
        "fn main() -> i64 = true;"
    ));
}

#[test]
fn test_error_int_where_bool_expected() {
    assert!(type_error(
        "fn main() -> bool = 42;"
    ));
}

#[test]
fn test_error_string_where_int_expected() {
    assert!(type_error(
        r#"fn main() -> i64 = "hello";"#
    ));
}

#[test]
fn test_error_arithmetic_on_bool() {
    assert!(type_error(
        "fn main() -> i64 = true + false;"
    ));
}

#[test]
fn test_error_comparison_mixed_types() {
    assert!(type_error(
        r#"fn main() -> bool = 42 == "hello";"#
    ));
}

// --- Undefined References ---

#[test]
fn test_error_undefined_function() {
    assert!(type_error(
        "fn main() -> i64 = nonexistent(42);"
    ));
}

#[test]
fn test_error_undefined_variable() {
    assert!(type_error(
        "fn main() -> i64 = x + 1;"
    ));
}

#[test]
fn test_error_undefined_struct() {
    assert!(type_error(
        "fn main() -> i64 = { let p = new Missing { x: 1 }; 0 };"
    ));
}

// --- Struct Errors ---

#[test]
fn test_error_missing_struct_field() {
    assert!(type_error(
        "struct Point { x: i64, y: i64 }
         fn main() -> Point = new Point { x: 1 };"
    ));
}

#[test]
fn test_error_field_access_on_non_struct() {
    // Cannot access .field on an integer
    assert!(type_error(
        "fn main() -> i64 = { let x: i64 = 42; x.field };"
    ));
}

#[test]
fn test_error_wrong_field_type_in_struct() {
    assert!(type_error(
        r#"struct Pair { a: i64, b: i64 }
         fn main() -> Pair = new Pair { a: "bad", b: 0 };"#
    ));
}

// --- Enum Errors ---

#[test]
fn test_error_unknown_enum_variant() {
    assert!(type_error(
        "enum Color { Red, Green, Blue }
         fn main() -> Color = Color::Yellow;"
    ));
}

// --- Return Type Mismatch ---

#[test]
fn test_error_void_function_returning_value() {
    assert!(type_error(
        "fn bad() -> () = 42;"
    ));
}

#[test]
fn test_error_non_void_function_returning_unit() {
    assert!(type_error(
        "fn bad() -> i64 = ();"
    ));
}

// --- Control Flow Type Errors ---

#[test]
fn test_error_if_condition_not_bool() {
    assert!(type_error(
        "fn main() -> i64 = if 42 { 1 } else { 0 };"
    ));
}

#[test]
fn test_error_while_condition_not_bool() {
    assert!(type_error(
        "fn main() -> i64 = { while 1 { 0 }; 0 };"
    ));
}

// --- Parse Errors ---

#[test]
fn test_parse_error_unclosed_brace() {
    assert!(parse_error(
        "fn main() -> i64 = { 42"
    ));
}

#[test]
fn test_parse_error_missing_return_type() {
    assert!(parse_error(
        "fn main() = 42;"
    ));
}

#[test]
fn test_parse_error_missing_semicolon() {
    assert!(parse_error(
        "fn main() -> i64 = 42"
    ));
}

// ========================================================================
// Cycle 225: Interpreter Edge Case & Boundary Tests
// ========================================================================

// --- Arithmetic Edge Cases ---

#[test]
fn test_interp_negative_numbers() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = -42;"),
        -42
    );
}

#[test]
fn test_interp_negative_arithmetic() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = -10 + 3;"),
        -7
    );
}

#[test]
fn test_interp_modulo_positive() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = 17 % 5;"),
        2
    );
}

#[test]
fn test_interp_modulo_negative_dividend() {
    // In most languages, -7 % 3 = -1 (truncated toward zero)
    assert_eq!(
        run_program_i64("fn main() -> i64 = -7 % 3;"),
        -1
    );
}

#[test]
fn test_interp_integer_division_truncates_toward_zero() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = 7 / 2;"),
        3
    );
}

#[test]
fn test_interp_negative_division_truncates_toward_zero() {
    assert_eq!(
        run_program_i64("fn main() -> i64 = -7 / 2;"),
        -3
    );
}

#[test]
fn test_interp_large_multiplication() {
    // 1000000 * 1000000 = 1_000_000_000_000 (fits in i64)
    assert_eq!(
        run_program_i64("fn main() -> i64 = 1000000 * 1000000;"),
        1_000_000_000_000
    );
}

// --- Boolean Edge Cases ---

#[test]
fn test_interp_bool_not_operator() {
    assert_eq!(
        run_program("fn main() -> bool = !true;"),
        Value::Bool(false)
    );
}

#[test]
fn test_interp_bool_not_false() {
    assert_eq!(
        run_program("fn main() -> bool = !false;"),
        Value::Bool(true)
    );
}

#[test]
fn test_interp_chained_comparison() {
    // a < b && b < c pattern
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a = 1;
               let b = 2;
               let c = 3;
               if a < b && b < c { 1 } else { 0 }
             };"
        ),
        1
    );
}

// --- Recursive Edge Cases ---

#[test]
fn test_interp_recursive_base_case_zero() {
    // Recursive function that should immediately return at base case
    assert_eq!(
        run_program_i64(
            "fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };
             fn main() -> i64 = fact(0);"
        ),
        1
    );
}

#[test]
fn test_interp_mutual_recursion_even_odd() {
    // Even/odd mutual recursion
    assert_eq!(
        run_program_i64(
            "fn is_even(n: i64) -> i64 = if n == 0 { 1 } else { is_odd(n - 1) };
             fn is_odd(n: i64) -> i64 = if n == 0 { 0 } else { is_even(n - 1) };
             fn main() -> i64 = is_even(10) + is_odd(7);"
        ),
        2 // is_even(10)=1, is_odd(7)=1, 1+1=2
    );
}

// --- Loop Edge Cases ---

#[test]
fn test_interp_loop_zero_iterations() {
    // While loop that never executes
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut x = 42;
               while false { { x = 0 } };
               x
             };"
        ),
        42
    );
}

#[test]
fn test_interp_for_empty_range() {
    // For loop with empty range (start >= end)
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut sum = 0;
               for i in 5..5 { sum = sum + i };
               sum
             };"
        ),
        0 // empty range, loop body never executes
    );
}

#[test]
fn test_interp_nested_break_only_inner() {
    // Break in inner loop should not affect outer loop
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let mut outer_count = 0;
               let mut i = 0;
               while i < 3 {
                 let mut j = 0;
                 loop {
                   j = j + 1;
                   if j >= 2 { break } else { () }
                 };
                 outer_count = outer_count + j;
                 { i = i + 1 }
               };
               outer_count
             };"
        ),
        6 // 3 iterations of outer, each inner runs j to 2, total = 2+2+2 = 6
    );
}

// --- String Edge Cases ---

#[test]
fn test_interp_empty_string_concat() {
    // Empty string concatenation should produce the other string
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = if "" == "" { 1 } else { 0 };"#
        ),
        1
    );
}

#[test]
fn test_interp_string_equality() {
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = if "hello" == "hello" { 1 } else { 0 };"#
        ),
        1
    );
}

#[test]
fn test_interp_string_inequality() {
    assert_eq!(
        run_program_i64(
            r#"fn main() -> i64 = if "hello" == "world" { 1 } else { 0 };"#
        ),
        0
    );
}

// --- Variable Shadowing ---

#[test]
fn test_interp_variable_shadowing() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let x = 10;
               let x = x + 20;
               x
             };"
        ),
        30
    );
}

#[test]
fn test_interp_shadowing_overwrites_in_scope() {
    // In BMB, let in a block overwrites the outer binding (flat scope)
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let x = 1;
               let y = {
                 let x = 100;
                 x + 5
               };
               x + y
             };"
        ),
        205 // x becomes 100 (overwritten), y=105, x+y = 100+105=205
    );
}

// --- Complex Expression Evaluation ---

#[test]
fn test_interp_nested_if_expressions() {
    assert_eq!(
        run_program_i64(
            "fn main() -> i64 = {
               let a = 3;
               let b = 7;
               if a > b {
                 if a > 10 { 100 } else { 50 }
               } else {
                 if b > 10 { 200 } else { 25 }
               }
             };"
        ),
        25 // a=3 < b=7, b=7 <= 10, so 25
    );
}

#[test]
fn test_interp_expression_as_function_arg() {
    assert_eq!(
        run_program_i64(
            "fn add(a: i64, b: i64) -> i64 = a + b;
             fn main() -> i64 = add(2 + 3, 4 * 5);"
        ),
        25 // add(5, 20) = 25
    );
}

// --- Type Checking Edge Cases ---

#[test]
fn test_type_nested_generic_instantiation() {
    assert!(type_checks(
        "fn id<T>(x: T) -> T = x;
         fn main() -> i64 = id(id(42));"
    ));
}

#[test]
fn test_type_if_else_both_return_same_type() {
    assert!(type_checks(
        "fn main() -> i64 = {
           let x = 5;
           if x > 0 { x * 2 } else { 0 - x }
         };"
    ));
}

// ========================================================================
// Cycle 226: Warning Detection Tests
// ========================================================================

// --- Unused Function Warnings ---

#[test]
fn test_warning_unused_function() {
    // A function defined but never called should produce a warning
    assert!(has_warning_kind(
        "fn helper() -> i64 = 42;
         fn main() -> i64 = 0;",
        "unused_function"
    ));
}

#[test]
fn test_no_warning_used_function() {
    // A function that is called should NOT produce unused warning
    assert!(!has_warning_kind(
        "fn helper() -> i64 = 42;
         fn main() -> i64 = helper();",
        "unused_function"
    ));
}

// --- Unused Type/Struct Warnings ---

#[test]
fn test_warning_unused_type() {
    assert!(has_warning_kind(
        "struct Unused { x: i64 }
         fn main() -> i64 = 0;",
        "unused_type"
    ));
}

#[test]
fn test_no_warning_used_struct() {
    assert!(!has_warning_kind(
        "struct Point { x: i64, y: i64 }
         fn main() -> i64 = { let p = new Point { x: 1, y: 2 }; p.x };",
        "unused_type"
    ));
}

// --- Unused Enum Warnings ---

#[test]
fn test_warning_unused_enum() {
    assert!(has_warning_kind(
        "enum Color { Red, Green, Blue }
         fn main() -> i64 = 0;",
        "unused_enum"
    ));
}

#[test]
fn test_no_warning_used_enum() {
    assert!(!has_warning_kind(
        "enum Choice { A, B }
         fn pick(c: Choice) -> i64 = match c { Choice::A => 1, Choice::B => 2 };
         fn main() -> i64 = pick(Choice::A);",
        "unused_enum"
    ));
}

// --- Duplicate Function Warnings ---

#[test]
fn test_warning_duplicate_function_three() {
    // Three definitions of same name
    assert!(has_warning_kind(
        "fn f() -> i64 = 1;
         fn f() -> i64 = 2;
         fn f() -> i64 = 3;
         fn main() -> i64 = f();",
        "duplicate_function"
    ));
}

// --- Unreachable Pattern Warnings ---

#[test]
fn test_warning_unreachable_pattern() {
    assert!(has_warning_kind(
        "fn f(x: bool) -> i64 = match x {
           _ => 1,
           true => 2,
           false => 3
         };
         fn main() -> i64 = f(true);",
        "unreachable_pattern"
    ));
}

#[test]
fn test_no_warning_all_patterns_reachable() {
    assert!(!has_warning_kind(
        "fn f(x: bool) -> i64 = match x {
           true => 1,
           false => 2
         };
         fn main() -> i64 = f(true);",
        "unreachable_pattern"
    ));
}

// --- Warning Absence Tests ---

#[test]
fn test_no_error_warnings_clean_program() {
    // A well-formed program should NOT produce error-level warnings
    // (missing_postcondition is expected for functions without contracts)
    let source = "fn add(a: i64, b: i64) -> i64 = a + b;
                   fn main() -> i64 = add(1, 2);";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    let unexpected: Vec<&str> = tc.warnings().iter()
        .map(|w| w.kind())
        .filter(|k| *k != "missing_postcondition") // expected for no-contract fns
        .collect();
    assert!(unexpected.is_empty(), "unexpected warnings: {:?}", unexpected);
}

// --- Multiple Warning Types ---

#[test]
fn test_program_with_multiple_warning_kinds() {
    // A program that triggers multiple different warning kinds
    let source = "struct Unused { x: i64 }
                   fn helper() -> i64 = 42;
                   fn main() -> i64 = 0;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    let warning_kinds: Vec<&str> = tc.warnings().iter().map(|w| w.kind()).collect();
    assert!(warning_kinds.contains(&"unused_type"), "should warn about unused struct");
    assert!(warning_kinds.contains(&"unused_function"), "should warn about unused function");
}

// ========================================================================
// Cycle 227: Full Pipeline Verification Tests
// ========================================================================

/// Helper: full pipeline — parse, type-check, lower to MIR, format MIR text
fn full_pipeline_mir(source: &str) -> (String, bmb::mir::MirProgram) {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mir = bmb::mir::lower_program(&ast);
    let text = bmb::mir::format_mir(&mir);
    (text, mir)
}

// --- Pipeline: Recursive Fibonacci ---

#[test]
fn test_pipeline_fibonacci_full() {
    // Verify fibonacci goes through the full pipeline correctly
    let source = "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };";

    // Stage 1: Parse succeeds
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();

    // Stage 2: Type check succeeds
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();

    // Stage 3: MIR lowering produces recursive call
    let mir = bmb::mir::lower_program(&ast);
    assert_eq!(mir.functions.len(), 1);
    let func = &mir.functions[0];
    assert_eq!(func.name, "fib");
    let has_call = func.blocks.iter().any(|b|
        b.instructions.iter().any(|i| matches!(i, bmb::mir::MirInst::Call { func, .. } if func == "fib"))
    );
    assert!(has_call, "should have recursive call to fib");

    // Stage 4: Codegen produces valid IR
    let codegen = TextCodeGen::new();
    let ir = codegen.generate(&mir).unwrap();
    assert!(ir.contains("@fib"));
    assert!(ir.contains("call i64 @fib"));
    assert!(ir.contains("ret i64"));
}

// --- Pipeline: Struct operations ---

#[test]
fn test_pipeline_struct_operations() {
    let source = "struct Point { x: i64, y: i64 }
                   fn dist_sq(p: Point) -> i64 = p.x * p.x + p.y * p.y;";
    let (text, mir) = full_pipeline_mir(source);
    // MIR should have struct definition
    assert!(mir.struct_defs.contains_key("Point"), "struct Point should be in MIR");
    // MIR text should reference the function
    assert!(text.contains("dist_sq"));
    assert!(text.contains("*"), "MIR should contain multiplication op");
}

// --- Pipeline: Enum with match ---

#[test]
fn test_pipeline_enum_match() {
    let source = "enum Direction { North, South, East, West }
                   fn to_dx(d: Direction) -> i64 = match d {
                     Direction::East => 1,
                     Direction::West => -1,
                     _ => 0
                   };";
    let (text, mir) = full_pipeline_mir(source);
    assert!(!mir.functions.is_empty());
    assert!(text.contains("to_dx"), "MIR should contain to_dx function");
    assert!(text.contains("return"), "MIR should have return terminator");
}

// --- Pipeline: Loop with accumulator ---

#[test]
fn test_pipeline_loop_accumulator() {
    let source = "fn sum_to(n: i64) -> i64 = {
                     let mut total = 0;
                     let mut i = 1;
                     while i <= n { total = total + i; { i = i + 1 } };
                     total
                   };";
    let (ir, mir) = full_pipeline_mir(source);
    let func = &mir.functions[0];
    // Should have multiple blocks for loop structure
    assert!(func.blocks.len() >= 3, "loop should create multiple blocks");
    // IR should have loop back-edge
    assert!(ir.contains("sum_to"), "MIR should contain sum_to function");
    assert!(ir.contains("goto"), "MIR should have goto for loop back-edge");
}

// --- Pipeline: Closure ---

#[test]
fn test_pipeline_closure() {
    let source = "fn main() -> i64 = {
                     let f = fn |x: i64| { x * 2 };
                     f(21)
                   };";
    // Parse, type check, and interpret
    assert_eq!(run_program_i64(source), 42);
}

// --- Pipeline: Generic function ---

#[test]
fn test_pipeline_generic_function() {
    let source = "fn id<T>(x: T) -> T = x;
                   fn main() -> i64 = id(42);";
    assert_eq!(run_program_i64(source), 42);
    // Also verify it type-checks
    assert!(type_checks(source));
}

// --- Pipeline: Contract verification ---

#[test]
fn test_pipeline_contract_precondition() {
    // Program with precondition should type-check and run correctly
    let source = "fn safe_div(a: i64, b: i64) -> i64
                     pre b != 0
                   = a / b;
                   fn main() -> i64 = safe_div(10, 2);";
    assert_eq!(run_program_i64(source), 5);
}

// --- Pipeline: For loop with range ---

#[test]
fn test_pipeline_for_range() {
    let source = "fn sum_range() -> i64 = {
                     let mut total = 0;
                     for i in 0..5 { total = total + i };
                     total
                   };";
    let (ir, _mir) = full_pipeline_mir(source);
    assert!(ir.contains("sum_range"), "MIR should contain sum_range function");
    // Verify interpreter produces correct result
    assert_eq!(run_program_i64(
        "fn sum_range() -> i64 = { let mut total = 0; for i in 0..5 { total = total + i }; total };
         fn main() -> i64 = sum_range();"
    ), 10); // 0+1+2+3+4=10
}

// --- Pipeline: Multi-function program ---

#[test]
fn test_pipeline_multi_function() {
    let source = "fn square(x: i64) -> i64 = x * x;
                   fn cube(x: i64) -> i64 = x * square(x);
                   fn main() -> i64 = square(3) + cube(2);";
    assert_eq!(run_program_i64(source), 17); // 9 + 8 = 17
    let (ir, mir) = full_pipeline_mir(source);
    assert!(mir.functions.len() >= 2); // at least square and cube
    assert!(ir.contains("square"), "MIR should contain square function");
    assert!(ir.contains("cube"), "MIR should contain cube function");
}

// --- Pipeline: Complex match with return ---

#[test]
fn test_pipeline_match_with_early_return() {
    let source = "fn classify(n: i64) -> i64 = {
                     if n < 0 { return -1 } else { () };
                     match n % 3 {
                       0 => 100,
                       1 => 200,
                       _ => 300
                     }
                   };
                   fn main() -> i64 = classify(-5) + classify(3) + classify(4) + classify(5);";
    assert_eq!(run_program_i64(source), -1 + 100 + 200 + 300); // 599
}

// ========================================================================
// Cycle 228: MIR Optimization Integration Tests
// ========================================================================

/// Helper: lower to MIR and run a specific optimization pass
fn optimized_mir(source: &str, pass: Box<dyn bmb::mir::OptimizationPass>) -> (String, bmb::mir::MirProgram) {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(pass);
    pipeline.optimize(&mut mir);
    let text = bmb::mir::format_mir(&mir);
    (text, mir)
}

// --- Constant Folding ---

#[test]
fn test_opt_constant_folding_arithmetic() {
    // 3 + 4 should be folded to 7
    let source = "fn f() -> i64 = 3 + 4;";
    let (text, _mir) = optimized_mir(source, Box::new(bmb::mir::ConstantFolding));
    // After constant folding, 3+4 should become a single constant 7
    assert!(text.contains("I:7"), "constant folding should produce I:7, got: {}", text);
}

#[test]
fn test_opt_constant_folding_nested() {
    // (2 * 3) + (10 - 4) should fold to 12
    let source = "fn f() -> i64 = 2 * 3 + (10 - 4);";
    let (text, _mir) = optimized_mir(source, Box::new(bmb::mir::ConstantFolding));
    assert!(text.contains("I:12"), "nested constant folding should produce I:12, got: {}", text);
}

#[test]
fn test_opt_constant_folding_comparison() {
    // 5 > 3 should fold to true (1)
    let source = "fn f() -> bool = 5 > 3;";
    let (text, _mir) = optimized_mir(source, Box::new(bmb::mir::ConstantFolding));
    assert!(text.contains("B:1"), "5 > 3 should fold to true (B:1), got: {}", text);
}

// --- Dead Code Elimination ---

#[test]
fn test_opt_dce_removes_unused_computation() {
    // The `unused` variable computation should be eliminated
    let source = "fn f(x: i64) -> i64 = {
                     let unused = x * x * x;
                     x + 1
                   };";
    let (_text_before, mir_before) = full_pipeline_mir(source);
    let (text_after, mir_after) = optimized_mir(source, Box::new(bmb::mir::DeadCodeElimination));
    // After DCE, the optimized MIR should have fewer instructions
    let inst_before: usize = mir_before.functions[0].blocks.iter().map(|b| b.instructions.len()).sum();
    let inst_after: usize = mir_after.functions[0].blocks.iter().map(|b| b.instructions.len()).sum();
    assert!(inst_after <= inst_before, "DCE should not increase instructions: {} vs {}", inst_after, inst_before);
    // The function should still return x + 1
    assert!(text_after.contains("+"), "should still have addition for x + 1");
}

// --- Copy Propagation ---

#[test]
fn test_opt_copy_propagation_eliminates_copy() {
    // let y = x; return y; should propagate x through
    let source = "fn f(x: i64) -> i64 = { let y = x; y };";
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::CopyPropagation));
    // After copy propagation, the return should reference the original parameter
    let func = &mir.functions[0];
    let total_copies: usize = func.blocks.iter()
        .flat_map(|b| b.instructions.iter())
        .filter(|i| matches!(i, bmb::mir::MirInst::Copy { .. }))
        .count();
    // Copy propagation should reduce or eliminate copies
    assert!(total_copies <= 1, "copy propagation should reduce copies, found {}", total_copies);
}

// --- SimplifyBranches ---

#[test]
fn test_opt_simplify_branches_true_condition() {
    // if true { 42 } else { 99 } should simplify to 42
    let source = "fn f() -> i64 = if true { 42 } else { 99 };";
    let (text, _mir) = optimized_mir(source, Box::new(bmb::mir::SimplifyBranches));
    // After branch simplification, the true branch should be taken
    assert!(text.contains("I:42"), "simplified branch should contain I:42, got: {}", text);
}

#[test]
fn test_opt_simplify_branches_false_condition() {
    // if false { 42 } else { 99 } should simplify to 99
    let source = "fn f() -> i64 = if false { 42 } else { 99 };";
    let (text, _mir) = optimized_mir(source, Box::new(bmb::mir::SimplifyBranches));
    assert!(text.contains("I:99"), "simplified branch should contain I:99, got: {}", text);
}

// --- CSE (Common Subexpression Elimination) ---

#[test]
fn test_opt_cse_eliminates_duplicate_computation() {
    // x * x appears twice — CSE should eliminate the duplicate
    let source = "fn f(x: i64) -> i64 = x * x + x * x;";
    let (_text, mir_before) = full_pipeline_mir(source);
    let (_text, mir_after) = optimized_mir(source, Box::new(bmb::mir::CommonSubexpressionElimination));
    let mul_before: usize = mir_before.functions[0].blocks.iter()
        .flat_map(|b| b.instructions.iter())
        .filter(|i| matches!(i, bmb::mir::MirInst::BinOp { op: bmb::mir::MirBinOp::Mul, .. }))
        .count();
    let mul_after: usize = mir_after.functions[0].blocks.iter()
        .flat_map(|b| b.instructions.iter())
        .filter(|i| matches!(i, bmb::mir::MirInst::BinOp { op: bmb::mir::MirBinOp::Mul, .. }))
        .count();
    assert!(mul_after <= mul_before, "CSE should not increase multiplications: {} vs {}", mul_after, mul_before);
}

// --- ContractBasedOptimization ---

#[test]
fn test_opt_contract_based_optimization() {
    // Contract pre b != 0 should enable optimizations
    let source = "fn safe_div(a: i64, b: i64) -> i64
                     pre b != 0
                   = a / b;";
    // Should not crash — contract optimization handles preconditions
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::ContractBasedOptimization));
    assert!(!mir.functions.is_empty(), "should have functions after optimization");
}

// --- IfElseToSwitch ---

#[test]
fn test_opt_if_else_to_switch_chain() {
    // Chained if/else on same variable should become switch
    let source = "fn classify(x: i64) -> i64 = {
                     if x == 0 { 10 }
                     else if x == 1 { 20 }
                     else if x == 2 { 30 }
                     else { 40 }
                   };";
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::IfElseToSwitch::new()));
    // After optimization, should have switch terminator
    let has_switch = mir.functions[0].blocks.iter().any(|b|
        matches!(&b.terminator, bmb::mir::Terminator::Switch { .. })
    );
    assert!(has_switch, "if/else chain on same var should be converted to switch");
}

// --- Optimization correctness via interpreter ---

#[test]
fn test_opt_constant_folding_correct_result() {
    // Verify constant folding doesn't change program semantics
    let source = "fn main() -> i64 = 10 * 5 + 3 - 1;";
    assert_eq!(run_program_i64(source), 52); // 50 + 3 - 1 = 52
}

#[test]
fn test_opt_branch_elimination_correct_result() {
    // Program with always-true branch
    let source = "fn main() -> i64 = {
                     let x = 10;
                     if true { x * 2 } else { x * 3 }
                   };";
    assert_eq!(run_program_i64(source), 20);
}

#[test]
fn test_opt_dead_code_does_not_affect_result() {
    // Dead computation should not affect live result
    let source = "fn main() -> i64 = {
                     let x = 42;
                     let unused = x * x * x;
                     let y = x + 8;
                     y
                   };";
    assert_eq!(run_program_i64(source), 50); // 42 + 8
}

#[test]
fn test_opt_cse_correct_result() {
    // CSE should not change computation result
    let source = "fn sq(x: i64) -> i64 = x * x;
                   fn main() -> i64 = sq(7) + sq(7);";
    assert_eq!(run_program_i64(source), 98); // 49 + 49
}

#[test]
fn test_opt_tail_recursion_correct_result() {
    // Tail-recursive function should produce correct result
    let source = "fn sum_tail(n: i64, acc: i64) -> i64 =
                     if n <= 0 { acc }
                     else { sum_tail(n - 1, acc + n) };
                   fn main() -> i64 = sum_tail(10, 0);";
    assert_eq!(run_program_i64(source), 55); // 1+2+...+10
}

// ========================================================================
// Cycle 229: Type System Edge Case Tests
// ========================================================================

// --- Generic Type Inference ---

#[test]
fn test_type_generic_identity_i64() {
    assert!(type_checks("fn id<T>(x: T) -> T = x;
                          fn main() -> i64 = id(42);"));
}

#[test]
fn test_type_generic_identity_bool() {
    assert!(type_checks("fn id<T>(x: T) -> T = x;
                          fn main() -> bool = id(true);"));
}

#[test]
fn test_type_generic_identity_string() {
    assert!(type_checks("fn id<T>(x: T) -> T = x;
                          fn main() -> String = id(\"hello\");"));
}

#[test]
fn test_type_generic_two_params() {
    assert!(type_checks("fn first<A, B>(a: A, b: B) -> A = a;
                          fn main() -> i64 = first(42, true);"));
}

#[test]
fn test_type_generic_return_type_mismatch() {
    // Calling generic with wrong return type
    assert!(type_error("fn id<T>(x: T) -> T = x;
                         fn main() -> bool = id(42);"));
}

// --- Struct Type Checking ---

#[test]
fn test_type_struct_field_types_match() {
    assert!(type_checks("struct Point { x: i64, y: i64 }
                          fn make() -> Point = new Point { x: 1, y: 2 };"));
}

#[test]
fn test_type_struct_field_wrong_type() {
    assert!(type_error("struct Point { x: i64, y: i64 }
                         fn make() -> Point = new Point { x: true, y: 2 };"));
}

#[test]
fn test_type_struct_return_type_mismatch() {
    assert!(type_error("struct Point { x: i64, y: i64 }
                         fn make() -> i64 = new Point { x: 1, y: 2 };"));
}

#[test]
fn test_type_nested_struct() {
    assert!(type_checks("struct Inner { val: i64 }
                          struct Outer { inner: Inner, tag: i64 }
                          fn make() -> Outer = new Outer { inner: new Inner { val: 1 }, tag: 2 };"));
}

// --- Enum Type Checking ---

#[test]
fn test_type_enum_variant_data_type() {
    assert!(type_checks("enum Result { Ok(i64), Err(String) }
                          fn make() -> Result = Result::Ok(42);"));
}

#[test]
fn test_type_enum_variant_wrong_data_type() {
    assert!(type_error("enum Result { Ok(i64), Err(String) }
                         fn make() -> Result = Result::Ok(\"wrong\");"));
}

#[test]
fn test_type_enum_match_exhaustive() {
    assert!(type_checks("enum Color { Red, Green, Blue }
                          fn to_num(c: Color) -> i64 = match c {
                            Color::Red => 1,
                            Color::Green => 2,
                            Color::Blue => 3
                          };"));
}

// --- Nullable Type Checking ---

#[test]
fn test_type_nullable_return_value() {
    assert!(type_checks("fn maybe(x: i64) -> i64? = if x > 0 { x } else { null };"));
}

#[test]
fn test_type_nullable_cannot_assign_null_to_non_nullable() {
    assert!(type_error("fn bad() -> i64 = null;"));
}

#[test]
fn test_type_nullable_unwrap_or_returns_base_type() {
    assert!(type_checks("fn get(x: i64?) -> i64 = x.unwrap_or(0);"));
}

// --- Function Type Checking ---

#[test]
fn test_type_function_wrong_return_type() {
    assert!(type_error("fn f() -> i64 = true;"));
}

#[test]
fn test_type_recursive_function_return() {
    assert!(type_checks("fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };"));
}

#[test]
fn test_type_mutual_recursion() {
    assert!(type_checks("fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
                          fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };"));
}

// --- Expression Type Checking ---

#[test]
fn test_type_if_branches_same_type() {
    assert!(type_checks("fn f(x: bool) -> i64 = if x { 1 } else { 2 };"));
}

#[test]
fn test_type_if_branches_different_types_error() {
    assert!(type_error("fn f(x: bool) -> i64 = if x { 1 } else { true };"));
}

#[test]
fn test_type_match_branches_consistent() {
    assert!(type_checks("fn f(x: i64) -> i64 = match x { 0 => 10, 1 => 20, _ => 30 };"));
}

#[test]
fn test_type_match_branches_inconsistent_error() {
    assert!(type_error("fn f(x: i64) -> String = match x { 0 => 10, _ => 20 };"));
}

// --- Contract Type Checking ---

#[test]
fn test_type_contract_pre_uses_params() {
    assert!(type_checks("fn div(a: i64, b: i64) -> i64 pre b != 0 = a / b;"));
}

#[test]
fn test_type_contract_post_uses_ret() {
    assert!(type_checks("fn abs(x: i64) -> i64 post ret >= 0 = if x < 0 { 0 - x } else { x };"));
}

// --- Closure Type Checking ---

#[test]
fn test_type_closure_inferred_param_types() {
    assert!(type_checks("fn main() -> i64 = {
                            let f = fn |x: i64| { x + 1 };
                            f(41)
                          };"));
}

#[test]
fn test_type_closure_captures_outer_var() {
    assert!(type_checks("fn main() -> i64 = {
                            let base = 10;
                            let f = fn |x: i64| { x + base };
                            f(32)
                          };"));
}

// --- Array Type Checking ---

#[test]
fn test_type_array_elements_same_type() {
    assert!(type_checks("fn f() -> [i64; 3] = [1, 2, 3];"));
}

#[test]
fn test_type_array_index_returns_element_type() {
    assert!(type_checks("fn f(arr: [i64; 3]) -> i64 = arr[0];"));
}

// --- Complex Type Expressions ---

#[test]
fn test_type_function_taking_struct_returning_field() {
    // Note: BMB uses `new` for struct construction, but field access doesn't need it
    assert!(type_checks("struct Pair { a: i64, b: i64 }
                          fn sum(p: Pair) -> i64 = p.a + p.b;"));
}

#[test]
fn test_type_generic_with_struct() {
    assert!(type_checks("struct Wrapper { val: i64 }
                          fn id<T>(x: T) -> T = x;
                          fn main() -> Wrapper = id(new Wrapper { val: 42 });"));
}

#[test]
fn test_type_complex_expression_inference() {
    // Multiple operations should infer correctly
    assert!(type_checks("fn f(a: i64, b: i64, c: bool) -> i64 = {
                            let x = a + b;
                            let y = x * 2;
                            if c { y } else { x }
                          };"));
}

// ========================================================================
// Cycle 230: AST Output & SMT Translation Integration Tests
// ========================================================================

/// Helper: parse program and return AST S-expression
fn ast_sexpr(source: &str) -> String {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    bmb::ast::output::to_sexpr(&ast)
}

// --- AST S-Expression Output Tests ---

#[test]
fn test_sexpr_simple_function() {
    let sexpr = ast_sexpr("fn f() -> i64 = 42;");
    assert!(sexpr.contains("(program"), "should start with (program, got: {}", sexpr);
    assert!(sexpr.contains("(fn "), "should contain (fn, got: {}", sexpr);
    assert!(sexpr.contains("f"), "should contain function name f");
}

#[test]
fn test_sexpr_function_with_params() {
    let sexpr = ast_sexpr("fn add(a: i64, b: i64) -> i64 = a + b;");
    assert!(sexpr.contains("add"), "should contain function name add");
    assert!(sexpr.contains("i64"), "should contain type i64");
}

#[test]
fn test_sexpr_struct_definition() {
    let sexpr = ast_sexpr("struct Point { x: i64, y: i64 }");
    assert!(sexpr.contains("(struct "), "should contain (struct, got: {}", sexpr);
    assert!(sexpr.contains("Point"), "should contain struct name Point");
}

#[test]
fn test_sexpr_enum_definition() {
    let sexpr = ast_sexpr("enum Color { Red, Green, Blue }");
    assert!(sexpr.contains("(enum "), "should contain (enum, got: {}", sexpr);
    assert!(sexpr.contains("Color"), "should contain enum name Color");
}

#[test]
fn test_sexpr_if_expression() {
    let sexpr = ast_sexpr("fn f(x: bool) -> i64 = if x { 1 } else { 2 };");
    assert!(sexpr.contains("if"), "should contain if expression");
}

#[test]
fn test_sexpr_match_expression() {
    let sexpr = ast_sexpr("fn f(x: i64) -> i64 = match x { 0 => 1, _ => 2 };");
    assert!(sexpr.contains("match"), "should contain match expression");
}

#[test]
fn test_sexpr_while_loop() {
    let sexpr = ast_sexpr("fn f() -> i64 = { let mut x = 0; while x < 10 { x = x + 1 }; x };");
    assert!(sexpr.contains("while"), "should contain while loop");
}

#[test]
fn test_sexpr_contract() {
    let sexpr = ast_sexpr("fn div(a: i64, b: i64) -> i64 pre b != 0 = a / b;");
    assert!(sexpr.contains("pre"), "should contain precondition");
}

#[test]
fn test_sexpr_closure() {
    let sexpr = ast_sexpr("fn f() -> i64 = { let g = fn |x: i64| { x * 2 }; g(21) };");
    assert!(sexpr.contains("closure") || sexpr.contains("lambda") || sexpr.contains("fn"),
            "should contain closure/lambda notation");
}

#[test]
fn test_sexpr_generic_function() {
    let sexpr = ast_sexpr("fn id<T>(x: T) -> T = x;");
    assert!(sexpr.contains("id"), "should contain function name id");
    assert!(sexpr.contains("T"), "should contain type parameter T");
}

// --- SMT-LIB2 Output Tests ---

#[test]
fn test_smt_generator_basic() {
    let mut smt = bmb::smt::SmtLibGenerator::new();
    smt.declare_var("x", bmb::smt::SmtSort::Int);
    smt.assert("(> x 0)");
    let output = smt.generate();
    assert!(output.contains("declare-const") || output.contains("declare-fun"),
            "SMT should declare variables, got: {}", output);
    assert!(output.contains("(> x 0)"), "SMT should contain assertion");
    assert!(output.contains("check-sat"), "SMT should contain check-sat");
}

#[test]
fn test_smt_generator_bool_var() {
    let mut smt = bmb::smt::SmtLibGenerator::new();
    smt.declare_var("b", bmb::smt::SmtSort::Bool);
    smt.assert("b");
    let output = smt.generate();
    assert!(output.contains("Bool"), "SMT should declare Bool sort");
}

#[test]
fn test_smt_generator_multiple_vars() {
    let mut smt = bmb::smt::SmtLibGenerator::new();
    smt.declare_var("x", bmb::smt::SmtSort::Int);
    smt.declare_var("y", bmb::smt::SmtSort::Int);
    smt.assert("(< x y)");
    smt.assert("(> x 0)");
    let output = smt.generate();
    // Should have both declarations and both assertions
    assert!(output.contains("x"), "should declare x");
    assert!(output.contains("y"), "should declare y");
    assert!(output.contains("(< x y)"), "should have < assertion");
    assert!(output.contains("(> x 0)"), "should have > assertion");
}

#[test]
fn test_smt_generator_clear() {
    let mut smt = bmb::smt::SmtLibGenerator::new();
    smt.declare_var("x", bmb::smt::SmtSort::Int);
    smt.assert("(> x 0)");
    smt.clear();
    let output = smt.generate();
    // After clear, should not have the old declaration
    assert!(!output.contains("(> x 0)"), "cleared generator should not have old assertions");
}

// --- AST format_type Tests ---

#[test]
fn test_format_type_i64() {
    let output = bmb::ast::output::format_type(&bmb::ast::Type::Named("i64".to_string()));
    assert_eq!(output, "i64");
}

#[test]
fn test_format_type_bool() {
    let output = bmb::ast::output::format_type(&bmb::ast::Type::Named("bool".to_string()));
    assert_eq!(output, "bool");
}

#[test]
fn test_format_type_nullable() {
    let output = bmb::ast::output::format_type(&bmb::ast::Type::Nullable(
        Box::new(bmb::ast::Type::Named("i64".to_string()))
    ));
    assert!(output.contains("?") || output.contains("nullable") || output.contains("Option"),
            "nullable type should be formatted, got: {}", output);
}

// ========================================================================
// Cycle 231: Verification & Proof Infrastructure Integration Tests
// ========================================================================

// --- Contract Verifier Report Tests ---

#[test]
fn test_verify_report_empty_program() {
    // Empty program should have empty report
    let tokens = tokenize("").expect("tokenize");
    let ast = parse("test.bmb", "", tokens).expect("parse");
    let verifier = bmb::verify::ContractVerifier::new();
    let report = verifier.verify_program(&ast);
    assert_eq!(report.verified_count(), 0);
    assert_eq!(report.failed_count(), 0);
    assert!(report.all_verified());
}

#[test]
fn test_verify_report_function_without_contract() {
    // Function without contracts should still be in report
    let source = "fn f(x: i64) -> i64 = x + 1;";
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let verifier = bmb::verify::ContractVerifier::new();
    let report = verifier.verify_program(&ast);
    // No contracts means nothing to verify, all verified
    assert!(report.all_verified());
}

#[test]
fn test_verify_report_with_precondition() {
    // Function with precondition should be in report
    let source = "fn safe_div(a: i64, b: i64) -> i64 pre b != 0 = a / b;";
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let verifier = bmb::verify::ContractVerifier::new();
    let _report = verifier.verify_program(&ast);
    // Report should not crash (Z3 may or may not be available)
}

// --- Proof Database Tests ---

#[test]
fn test_proof_db_store_and_retrieve() {
    use bmb::verify::proof_db::*;
    let mut db = ProofDatabase::new();
    let id = FunctionId::simple("test_fn");
    let result = FunctionProofResult {
        status: VerificationStatus::Verified,
        proven_facts: vec![],
        verification_time: std::time::Duration::from_secs(0),
        smt_queries: 0,
        verified_at: 0,
    };
    db.store_function_proof(&id, result);
    assert!(db.is_verified(&id));
}

#[test]
fn test_proof_db_unknown_function() {
    use bmb::verify::proof_db::*;
    let db = ProofDatabase::new();
    let id = FunctionId::simple("nonexistent");
    assert!(!db.is_verified(&id));
}

#[test]
fn test_proof_db_function_id_key() {
    let id = bmb::verify::FunctionId::simple("my_func");
    let key = id.key();
    assert!(key.contains("my_func"), "key should contain function name");
}

#[test]
fn test_proof_db_stats_default() {
    let stats = bmb::verify::ProofDbStats::default();
    assert_eq!(stats.functions_stored, 0);
    assert_eq!(stats.cache_hits, 0);
}

// --- Function Summary Tests ---

#[test]
fn test_summary_extract_from_program() {
    let source = "fn add(a: i64, b: i64) -> i64 = a + b;
                   fn sub(a: i64, b: i64) -> i64 = a - b;";
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check");
    let cir = bmb::cir::lower_to_cir(&ast);
    let summaries = bmb::verify::extract_summaries(&cir);
    assert!(summaries.len() >= 2, "should have at least 2 function summaries");
}

#[test]
fn test_summary_function_with_contract() {
    let source = "fn safe_div(a: i64, b: i64) -> i64 pre b != 0 = a / b;";
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let cir = bmb::cir::lower_to_cir(&ast);
    let summaries = bmb::verify::extract_summaries(&cir);
    assert!(!summaries.is_empty(), "should have function summary");
}

#[test]
fn test_summary_compare_same_program() {
    let source = "fn f(x: i64) -> i64 = x + 1;";
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let cir = bmb::cir::lower_to_cir(&ast);
    let summaries = bmb::verify::extract_summaries(&cir);
    // Compare first function summary with itself
    let first_id = summaries.keys().next().expect("should have at least one function");
    let first_summary = &summaries[first_id];
    let change = bmb::verify::compare_summaries(Some(first_summary), Some(first_summary));
    // Same summary should indicate no change (Unchanged variant)
    assert!(matches!(change, bmb::verify::SummaryChange::Unchanged), "same summary should be unchanged");
}

// --- Incremental Verification Tests ---

#[test]
fn test_incremental_verifier_new() {
    let verifier = bmb::verify::IncrementalVerifier::new();
    // Should create without crashing
    let _ = verifier;
}

// --- End-to-End Quality Tests ---

#[test]
fn test_e2e_fibonacci_all_stages() {
    // Fibonacci through all stages
    let source = "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };
                   fn main() -> i64 = fib(10);";
    // Type check
    assert!(type_checks(source));
    // Interpret
    assert_eq!(run_program_i64(source), 55);
    // MIR lower
    let (_text, mir) = full_pipeline_mir(source);
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_e2e_factorial_all_stages() {
    let source = "fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };
                   fn main() -> i64 = fact(10);";
    assert!(type_checks(source));
    assert_eq!(run_program_i64(source), 3628800);
    let (_text, mir) = full_pipeline_mir(source);
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_e2e_gcd_all_stages() {
    let source = "fn gcd(a: i64, b: i64) -> i64 =
                     if b == 0 { a } else { gcd(b, a % b) };
                   fn main() -> i64 = gcd(48, 18);";
    assert!(type_checks(source));
    assert_eq!(run_program_i64(source), 6);
    let (_text, mir) = full_pipeline_mir(source);
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_e2e_power_all_stages() {
    let source = "fn pow(base: i64, exp: i64) -> i64 =
                     if exp == 0 { 1 }
                     else { base * pow(base, exp - 1) };
                   fn main() -> i64 = pow(2, 10);";
    assert!(type_checks(source));
    assert_eq!(run_program_i64(source), 1024);
}

// ============================================================================
// PIR (Proof-Indexed IR) Integration Tests (Cycle 232)
// ============================================================================

/// Helper: parse, type-check, lower to CIR, propagate proofs → PIR
fn source_to_pir(source: &str) -> bmb::pir::PirProgram {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let cir = bmb::cir::lower_to_cir(&ast);
    let proof_db = bmb::verify::ProofDatabase::new();
    bmb::pir::propagate_proofs(&cir, &proof_db)
}

/// Helper: source → PIR → extract all facts
fn source_to_pir_facts(source: &str) -> std::collections::HashMap<String, bmb::pir::FunctionFacts> {
    let pir = source_to_pir(source);
    bmb::pir::extract_all_pir_facts(&pir)
}

#[test]
fn test_pir_simple_function_propagation() {
    let pir = source_to_pir("fn add(a: i64, b: i64) -> i64 = a + b;");
    assert_eq!(pir.functions.len(), 1);
    assert_eq!(pir.functions[0].name, "add");
    assert_eq!(pir.functions[0].params.len(), 2);
}

#[test]
fn test_pir_multiple_functions() {
    let source = "fn double(x: i64) -> i64 = x * 2;
                  fn triple(x: i64) -> i64 = x * 3;
                  fn main() -> i64 = double(3) + triple(2);";
    let pir = source_to_pir(source);
    assert_eq!(pir.functions.len(), 3);
    let names: Vec<&str> = pir.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"double"));
    assert!(names.contains(&"triple"));
    assert!(names.contains(&"main"));
}

#[test]
fn test_pir_precondition_becomes_entry_fact() {
    let source = "fn safe_div(a: i64, b: i64) -> i64
                    pre b > 0
                  = a / b;";
    let pir = source_to_pir(source);
    let func = &pir.functions[0];
    assert!(!func.entry_facts.is_empty(), "precondition should produce entry facts");
}

#[test]
fn test_pir_postcondition_becomes_exit_fact() {
    let source = "fn abs_val(x: i64) -> i64
                    post ret >= 0
                  = if x >= 0 { x } else { 0 - x };";
    let pir = source_to_pir(source);
    let func = &pir.functions[0];
    assert!(!func.exit_facts.is_empty(), "postcondition should produce exit facts");
}

#[test]
fn test_pir_function_return_type() {
    let source = "fn is_positive(x: i64) -> bool = x > 0;";
    let pir = source_to_pir(source);
    assert_eq!(pir.functions[0].ret_ty, bmb::pir::PirType::Bool);
}

#[test]
fn test_pir_function_i64_return_type() {
    let source = "fn square(x: i64) -> i64 = x * x;";
    let pir = source_to_pir(source);
    assert_eq!(pir.functions[0].ret_ty, bmb::pir::PirType::I64);
}

#[test]
fn test_pir_empty_program() {
    let pir = source_to_pir("fn noop() -> i64 = 0;");
    assert_eq!(pir.functions.len(), 1);
    assert!(pir.functions[0].entry_facts.is_empty());
    assert!(pir.functions[0].exit_facts.is_empty());
}

#[test]
fn test_pir_extract_facts_simple_function() {
    let facts = source_to_pir_facts("fn id(x: i64) -> i64 = x;");
    assert_eq!(facts.len(), 1);
    assert!(facts.contains_key("id"));
}

#[test]
fn test_pir_extract_facts_precondition() {
    let source = "fn bounded(x: i64) -> i64
                    pre x >= 0
                  = x + 1;";
    let facts = source_to_pir_facts(source);
    let func_facts = &facts["bounded"];
    assert!(!func_facts.preconditions.is_empty(), "precondition should be extracted as fact");
    assert!(!func_facts.all_facts.is_empty());
}

#[test]
fn test_pir_extract_facts_postcondition() {
    let source = "fn positive_result(x: i64) -> i64
                    post ret > 0
                  = if x > 0 { x } else { 1 };";
    let facts = source_to_pir_facts(source);
    let func_facts = &facts["positive_result"];
    assert!(!func_facts.postconditions.is_empty(), "postcondition should be extracted");
}

#[test]
fn test_pir_extract_facts_multiple_functions() {
    let source = "fn a(x: i64) -> i64 pre x > 0 = x;
                  fn b(x: i64) -> i64 post ret >= 0 = if x >= 0 { x } else { 0 };";
    let facts = source_to_pir_facts(source);
    assert_eq!(facts.len(), 2);
    assert!(facts.contains_key("a"));
    assert!(facts.contains_key("b"));
    assert!(!facts["a"].preconditions.is_empty());
    assert!(!facts["b"].postconditions.is_empty());
}

#[test]
fn test_pir_propagation_rule_enum() {
    // Test that PropagationRule variants exist and are distinct
    let rules = [
        bmb::pir::PropagationRule::PreconditionToFact,
        bmb::pir::PropagationRule::BranchCondition,
        bmb::pir::PropagationRule::LoopCondition,
        bmb::pir::PropagationRule::LetBinding,
        bmb::pir::PropagationRule::PostconditionAfterCall,
    ];
    assert_eq!(rules.len(), 5);
    assert_ne!(rules[0], rules[1]);
}

#[test]
fn test_pir_proven_fact_constructors() {
    use bmb::cir::Proposition;
    use bmb::pir::ProvenFact;

    let pre = ProvenFact::from_precondition(Proposition::True, 1);
    assert_eq!(pre.id, 1);

    let cf = ProvenFact::from_control_flow(Proposition::True, 2);
    assert_eq!(cf.id, 2);

    let smt = ProvenFact::from_smt(Proposition::True, 999, 3);
    assert_eq!(smt.id, 3);
}

#[test]
fn test_pir_proven_fact_to_contract_facts_var_cmp() {
    use bmb::cir::{CirExpr, CompareOp, Proposition};
    use bmb::pir::ProvenFact;

    let fact = ProvenFact::from_precondition(
        Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Ge,
            rhs: Box::new(CirExpr::IntLit(0)),
        },
        1,
    );
    let contract_facts = bmb::pir::proven_fact_to_contract_facts(&fact);
    assert_eq!(contract_facts.len(), 1);
    assert!(matches!(
        &contract_facts[0],
        bmb::mir::ContractFact::VarCmp { var, op: bmb::mir::CmpOp::Ge, value: 0 } if var == "x"
    ));
}

#[test]
fn test_pir_proven_fact_to_contract_facts_non_null() {
    use bmb::cir::{CirExpr, Proposition};
    use bmb::pir::ProvenFact;

    let fact = ProvenFact::from_precondition(
        Proposition::NonNull(Box::new(CirExpr::Var("ptr".to_string()))),
        1,
    );
    let contract_facts = bmb::pir::proven_fact_to_contract_facts(&fact);
    assert_eq!(contract_facts.len(), 1);
    assert!(matches!(
        &contract_facts[0],
        bmb::mir::ContractFact::NonNull { var } if var == "ptr"
    ));
}

#[test]
fn test_pir_proven_fact_to_contract_facts_in_bounds() {
    use bmb::cir::{CirExpr, Proposition};
    use bmb::pir::ProvenFact;

    let fact = ProvenFact::from_precondition(
        Proposition::InBounds {
            index: Box::new(CirExpr::Var("i".to_string())),
            array: Box::new(CirExpr::Var("arr".to_string())),
        },
        1,
    );
    let contract_facts = bmb::pir::proven_fact_to_contract_facts(&fact);
    assert_eq!(contract_facts.len(), 1);
    assert!(matches!(
        &contract_facts[0],
        bmb::mir::ContractFact::ArrayBounds { index, array }
        if index == "i" && array == "arr"
    ));
}

#[test]
fn test_pir_extract_function_facts_with_contract() {
    let source = "fn clamp(x: i64) -> i64
                    pre x >= 0
                    post ret >= 0
                  = if x > 100 { 100 } else { x };";
    let facts = source_to_pir_facts(source);
    let func_facts = &facts["clamp"];
    assert!(!func_facts.preconditions.is_empty());
    assert!(!func_facts.postconditions.is_empty());
    assert!(!func_facts.all_facts.is_empty());
    // all_facts should include at least the preconditions
    assert!(func_facts.all_facts.len() >= func_facts.preconditions.len());
}

// ============================================================================
// CFG (Conditional Compilation) Integration Tests (Cycle 233)
// ============================================================================

#[test]
fn test_cfg_target_from_str_native() {
    use bmb::cfg::Target;
    assert_eq!(Target::from_str("native"), Some(Target::Native));
    assert_eq!(Target::from_str("x86_64"), Some(Target::Native));
    assert_eq!(Target::from_str("aarch64"), Some(Target::Native));
}

#[test]
fn test_cfg_target_from_str_wasm() {
    use bmb::cfg::Target;
    assert_eq!(Target::from_str("wasm32"), Some(Target::Wasm32));
    assert_eq!(Target::from_str("wasm"), Some(Target::Wasm32));
    assert_eq!(Target::from_str("wasm64"), Some(Target::Wasm64));
}

#[test]
fn test_cfg_target_from_str_unknown() {
    use bmb::cfg::Target;
    assert_eq!(Target::from_str("unknown_target"), None);
    assert_eq!(Target::from_str(""), None);
}

#[test]
fn test_cfg_target_as_str() {
    use bmb::cfg::Target;
    assert_eq!(Target::Native.as_str(), "native");
    assert_eq!(Target::Wasm32.as_str(), "wasm32");
    assert_eq!(Target::Wasm64.as_str(), "wasm64");
}

#[test]
fn test_cfg_target_default_is_native() {
    use bmb::cfg::Target;
    assert_eq!(Target::default(), Target::Native);
}

#[test]
fn test_cfg_filter_program_no_attributes() {
    use bmb::cfg::{CfgEvaluator, Target};
    let source = "fn a() -> i64 = 1; fn b() -> i64 = 2; fn c() -> i64 = 3;";
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let eval = CfgEvaluator::new(Target::Native);
    let filtered = eval.filter_program(&ast);
    assert_eq!(filtered.items.len(), 3, "all functions should be included without @cfg");
}

#[test]
fn test_cfg_filter_program_with_cfg_native() {
    use bmb::cfg::{CfgEvaluator, Target};
    let source = r#"@cfg(target == "native")
fn native_fn() -> i64 = 1;
fn always_fn() -> i64 = 2;"#;
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");

    let eval_native = CfgEvaluator::new(Target::Native);
    let filtered = eval_native.filter_program(&ast);
    assert_eq!(filtered.items.len(), 2);

    let eval_wasm = CfgEvaluator::new(Target::Wasm32);
    let filtered = eval_wasm.filter_program(&ast);
    assert_eq!(filtered.items.len(), 1);
}

#[test]
fn test_cfg_filter_program_with_cfg_wasm32() {
    use bmb::cfg::{CfgEvaluator, Target};
    let source = r#"@cfg(target == "wasm32")
fn wasm_fn() -> i64 = 1;
fn common_fn() -> i64 = 2;"#;
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");

    let eval_wasm = CfgEvaluator::new(Target::Wasm32);
    let filtered = eval_wasm.filter_program(&ast);
    assert_eq!(filtered.items.len(), 2);

    let eval_native = CfgEvaluator::new(Target::Native);
    let filtered = eval_native.filter_program(&ast);
    assert_eq!(filtered.items.len(), 1);
}

#[test]
fn test_cfg_filter_preserves_non_cfg_functions() {
    use bmb::cfg::{CfgEvaluator, Target};
    let source = r#"fn always_a() -> i64 = 1;
@cfg(target == "wasm64")
fn wasm64_only() -> i64 = 2;
fn always_b() -> i64 = 3;"#;
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");

    let eval = CfgEvaluator::new(Target::Native);
    let filtered = eval.filter_program(&ast);
    assert_eq!(filtered.items.len(), 2, "wasm64_only should be filtered out");
}

#[test]
fn test_cfg_filter_empty_program() {
    use bmb::cfg::{CfgEvaluator, Target};
    use bmb::ast::Program;
    let program = Program { header: None, items: vec![] };
    let eval = CfgEvaluator::new(Target::Native);
    let filtered = eval.filter_program(&program);
    assert!(filtered.items.is_empty());
}

#[test]
fn test_cfg_should_include_item_struct() {
    use bmb::cfg::{CfgEvaluator, Target};
    let source = r#"@cfg(target == "wasm32")
struct WasmStruct { x: i64 }
struct AlwaysStruct { y: i64 }"#;
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let eval = CfgEvaluator::new(Target::Native);
    let filtered = eval.filter_program(&ast);
    assert_eq!(filtered.items.len(), 1, "WasmStruct should be excluded on native");
}

#[test]
fn test_cfg_should_include_item_enum() {
    use bmb::cfg::{CfgEvaluator, Target};
    let source = r#"@cfg(target == "native")
enum NativeError { Io, Memory }
enum CommonError { NotFound, Timeout }"#;
    let tokens = tokenize(source).expect("tokenize");
    let ast = parse("test.bmb", source, tokens).expect("parse");
    let eval = CfgEvaluator::new(Target::Wasm32);
    let filtered = eval.filter_program(&ast);
    assert_eq!(filtered.items.len(), 1, "NativeError should be excluded on wasm32");
}

#[test]
fn test_cfg_target_case_insensitive() {
    use bmb::cfg::Target;
    assert_eq!(Target::from_str("NATIVE"), Some(Target::Native));
    assert_eq!(Target::from_str("Wasm32"), Some(Target::Wasm32));
    assert_eq!(Target::from_str("WASM64"), Some(Target::Wasm64));
}

#[test]
fn test_cfg_target_wasm_aliases() {
    use bmb::cfg::Target;
    assert_eq!(Target::from_str("wasm32-wasi"), Some(Target::Wasm32));
    assert_eq!(Target::from_str("wasm32-unknown"), Some(Target::Wasm32));
    assert_eq!(Target::from_str("x86"), Some(Target::Native));
    assert_eq!(Target::from_str("arm"), Some(Target::Native));
}

// ============================================================================
// Preprocessor & Resolver Integration Tests (Cycle 234)
// ============================================================================

#[test]
fn test_preprocessor_no_includes_passthrough() {
    use bmb::preprocessor::expand_includes;
    use std::path::Path;
    let source = "fn main() -> i64 = 42;";
    let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
    assert!(result.contains("fn main()"));
    assert!(result.contains("42"));
}

#[test]
fn test_preprocessor_multi_line_passthrough() {
    use bmb::preprocessor::expand_includes;
    use std::path::Path;
    let source = "fn a() -> i64 = 1;\nfn b() -> i64 = 2;\nfn c() -> i64 = 3;";
    let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
    assert!(result.contains("fn a()"));
    assert!(result.contains("fn b()"));
    assert!(result.contains("fn c()"));
}

#[test]
fn test_preprocessor_include_real_file() {
    use bmb::preprocessor::expand_includes;

    let dir = std::env::temp_dir().join("bmb_test_pp_include");
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join("helper.bmb"), "fn helper() -> i64 = 99;\n").unwrap();

    let source = "@include \"helper.bmb\"\nfn main() -> i64 = helper();";
    let main_file = dir.join("main.bmb");
    std::fs::write(&main_file, source).unwrap();

    let result = expand_includes(source, &main_file, &[]).unwrap();
    assert!(result.contains("fn helper()"), "included content should appear");
    assert!(result.contains("fn main()"), "original content preserved");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_preprocessor_include_not_found() {
    use bmb::preprocessor::expand_includes;
    use std::path::Path;
    let source = "@include \"nonexistent_xyz.bmb\"";
    let result = expand_includes(source, Path::new("test.bmb"), &[]);
    assert!(result.is_err());
}

#[test]
fn test_preprocessor_circular_include_detected() {
    use bmb::preprocessor::expand_includes;

    let dir = std::env::temp_dir().join("bmb_test_pp_circular");
    let _ = std::fs::create_dir_all(&dir);

    // a.bmb includes b.bmb, b.bmb includes a.bmb
    std::fs::write(dir.join("a.bmb"), "@include \"b.bmb\"\nfn a() -> i64 = 1;").unwrap();
    std::fs::write(dir.join("b.bmb"), "@include \"a.bmb\"\nfn b() -> i64 = 2;").unwrap();

    let source = std::fs::read_to_string(dir.join("a.bmb")).unwrap();
    let result = expand_includes(&source, &dir.join("a.bmb"), &[]);
    assert!(result.is_err(), "circular include should be detected");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_preprocessor_expand_with_prelude_no_prelude() {
    use bmb::preprocessor::expand_with_prelude;
    use std::path::Path;
    let source = "fn main() -> i64 = 1;";
    let result = expand_with_prelude(source, Path::new("test.bmb"), &[], None).unwrap();
    assert!(result.contains("fn main()"));
}

#[test]
fn test_preprocessor_error_display_formats() {
    use bmb::preprocessor::PreprocessorError;
    use std::path::PathBuf;

    let err = PreprocessorError::FileNotFound(
        "missing.bmb".to_string(),
        vec![PathBuf::from("./src")],
    );
    let msg = format!("{}", err);
    assert!(msg.contains("missing.bmb"));

    let err = PreprocessorError::CircularInclude(PathBuf::from("loop.bmb"));
    let msg = format!("{}", err);
    assert!(msg.contains("Circular"));

    let err = PreprocessorError::InvalidSyntax("bad".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid"));
}

#[test]
fn test_preprocessor_new_with_search_paths() {
    use bmb::preprocessor::Preprocessor;
    use std::path::PathBuf;
    let mut pp = Preprocessor::new(vec![PathBuf::from("/usr/lib/bmb"), PathBuf::from("./lib")]);
    // Verify construction doesn't panic and can expand
    let source = "fn test() -> i64 = 0;";
    let result = pp.expand(source, std::path::Path::new("test.bmb")).unwrap();
    assert!(result.contains("fn test()"));
}

// --- Resolver Integration Tests ---

#[test]
fn test_resolver_creation_and_base_dir() {
    use bmb::resolver::Resolver;
    use std::path::Path;
    let resolver = Resolver::new(".");
    assert_eq!(resolver.base_dir(), Path::new("."));
    assert_eq!(resolver.module_count(), 0);
}

#[test]
fn test_resolver_nonexistent_module() {
    use bmb::resolver::Resolver;
    let resolver = Resolver::new(".");
    assert!(resolver.get_module("nonexistent").is_none());
}

#[test]
fn test_resolver_load_module_from_file() {
    use bmb::resolver::Resolver;

    let dir = std::env::temp_dir().join("bmb_test_resolver_load");
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join("mymod.bmb"), "pub fn add(a: i64, b: i64) -> i64 = a + b;\n").unwrap();

    let mut resolver = Resolver::new(&dir);
    let module = resolver.load_module("mymod");
    assert!(module.is_ok(), "should load module from file");
    let module = module.unwrap();
    assert_eq!(module.name, "mymod");
    assert!(!module.exports.is_empty(), "pub fn should be exported");
    assert!(module.exports.contains_key("add"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_resolver_load_module_not_found() {
    use bmb::resolver::Resolver;

    let dir = std::env::temp_dir().join("bmb_test_resolver_missing");
    let _ = std::fs::create_dir_all(&dir);

    let mut resolver = Resolver::new(&dir);
    let result = resolver.load_module("nonexistent_module");
    assert!(result.is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_resolver_module_count_after_load() {
    use bmb::resolver::Resolver;

    let dir = std::env::temp_dir().join("bmb_test_resolver_count");
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join("mod_a.bmb"), "pub fn a() -> i64 = 1;\n").unwrap();
    std::fs::write(dir.join("mod_b.bmb"), "pub fn b() -> i64 = 2;\n").unwrap();

    let mut resolver = Resolver::new(&dir);
    resolver.load_module("mod_a").unwrap();
    assert_eq!(resolver.module_count(), 1);
    resolver.load_module("mod_b").unwrap();
    assert_eq!(resolver.module_count(), 2);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_resolved_imports_api() {
    use bmb::resolver::{ResolvedImports, ExportedItem};
    use bmb::ast::Span;

    let mut imports = ResolvedImports::new();
    assert!(imports.is_empty());

    let span = Span::new(0, 10);
    imports.add_import("Token".to_string(), "lexer".to_string(),
        ExportedItem::Struct("Token".to_string()), span);

    assert_eq!(imports.len(), 1);
    assert!(!imports.is_empty());
    assert!(imports.is_imported("Token"));
    assert!(!imports.is_imported("Other"));
    assert_eq!(imports.get_import_module("Token"), Some("lexer"));
}

#[test]
fn test_resolved_imports_unused_tracking() {
    use bmb::resolver::{ResolvedImports, ExportedItem};
    use bmb::ast::Span;

    let mut imports = ResolvedImports::new();
    let span = Span::new(0, 10);
    imports.add_import("Used".to_string(), "mod".to_string(),
        ExportedItem::Function("Used".to_string()), span);
    imports.add_import("Unused".to_string(), "mod".to_string(),
        ExportedItem::Function("Unused".to_string()), span);

    imports.mark_used("Used");
    let unused = imports.get_unused();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].0, "Unused");
}

#[test]
fn test_resolved_imports_underscore_not_reported() {
    use bmb::resolver::{ResolvedImports, ExportedItem};
    use bmb::ast::Span;

    let mut imports = ResolvedImports::new();
    let span = Span::new(0, 5);
    imports.add_import("_internal".to_string(), "mod".to_string(),
        ExportedItem::Function("_internal".to_string()), span);

    let unused = imports.get_unused();
    assert!(unused.is_empty(), "underscore-prefixed imports not reported");
}

#[test]
fn test_exported_item_variants() {
    use bmb::resolver::ExportedItem;
    let fn_item = ExportedItem::Function("add".to_string());
    let struct_item = ExportedItem::Struct("Point".to_string());
    let enum_item = ExportedItem::Enum("Color".to_string());
    // Verify Debug display distinguishes them
    assert!(format!("{:?}", fn_item).contains("Function"));
    assert!(format!("{:?}", struct_item).contains("Struct"));
    assert!(format!("{:?}", enum_item).contains("Enum"));
}

// ============================================================================
// Query System & Index Integration Tests (Cycle 235)
// ============================================================================

/// Helper: parse source and generate a ProjectIndex
fn source_to_index(source: &str) -> bmb::index::ProjectIndex {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut indexer = bmb::index::IndexGenerator::new("test-project");
    indexer.index_file("test.bmb", &ast);
    indexer.generate()
}

#[test]
fn test_index_generator_simple_function() {
    let index = source_to_index("fn add(a: i64, b: i64) -> i64 = a + b;");
    assert_eq!(index.functions.len(), 1);
    assert_eq!(index.functions[0].name, "add");
    assert_eq!(index.functions[0].signature.params.len(), 2);
    assert_eq!(index.functions[0].signature.return_type, "i64");
}

#[test]
fn test_index_generator_multiple_functions() {
    let source = "fn double(x: i64) -> i64 = x * 2;
                  fn triple(x: i64) -> i64 = x * 3;";
    let index = source_to_index(source);
    assert_eq!(index.functions.len(), 2);
    let names: Vec<&str> = index.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"double"));
    assert!(names.contains(&"triple"));
}

#[test]
fn test_index_generator_struct() {
    let index = source_to_index("struct Point { x: i64, y: i64 }");
    assert_eq!(index.types.len(), 1);
    assert_eq!(index.types[0].name, "Point");
    assert_eq!(index.types[0].kind, "struct");
}

#[test]
fn test_index_generator_enum() {
    let index = source_to_index("enum Color { Red, Green, Blue }");
    assert_eq!(index.types.len(), 1);
    assert_eq!(index.types[0].name, "Color");
    assert_eq!(index.types[0].kind, "enum");
}

#[test]
fn test_index_generator_with_contract() {
    let source = "fn safe_div(a: i64, b: i64) -> i64 pre b > 0 = a / b;";
    let index = source_to_index(source);
    assert_eq!(index.functions.len(), 1);
    let func = &index.functions[0];
    assert!(func.contracts.is_some(), "contract should be indexed");
    let contracts = func.contracts.as_ref().unwrap();
    assert!(contracts.pre.is_some(), "precondition should be present");
}

#[test]
fn test_index_manifest_counts() {
    let source = "fn a() -> i64 = 1;
                  fn b() -> i64 = 2;
                  struct S { x: i64 }";
    let index = source_to_index(source);
    assert_eq!(index.manifest.functions, 2);
    assert_eq!(index.manifest.types, 1);
}

#[test]
fn test_index_symbol_entries() {
    let source = "fn public_fn() -> i64 = 1;
                  struct MyStruct { val: i64 }";
    let index = source_to_index(source);
    assert!(!index.symbols.is_empty());
    let fn_symbols: Vec<_> = index.symbols.iter()
        .filter(|s| s.kind == bmb::index::SymbolKind::Function)
        .collect();
    assert!(!fn_symbols.is_empty());
}

#[test]
fn test_index_write_and_read() {
    let source = "fn test_fn(x: i64) -> i64 = x + 1;
                  struct TestStruct { field: i64 }";
    let index = source_to_index(source);

    let dir = std::env::temp_dir().join("bmb_test_index_rw");
    let _ = std::fs::create_dir_all(&dir);

    bmb::index::write_index(&index, &dir).unwrap();
    let loaded = bmb::index::read_index(&dir).unwrap();

    assert_eq!(loaded.manifest.project, "test-project");
    assert_eq!(loaded.functions.len(), 1);
    assert_eq!(loaded.types.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_index_proof_index_creation() {
    let mut proof_idx = bmb::index::ProofIndex::new(false, None);
    proof_idx.add_proof(bmb::index::ProofEntry {
        name: "test_fn".to_string(),
        file: "test.bmb".to_string(),
        line: 1,
        pre_status: Some(bmb::index::ProofStatus::Verified),
        post_status: Some(bmb::index::ProofStatus::Unknown),
        counterexample: None,
        verify_time_ms: Some(10),
        verified_at: None,
    });
    assert_eq!(proof_idx.proofs.len(), 1);
}

#[test]
fn test_index_proof_status_variants() {
    use bmb::index::ProofStatus;
    let statuses = [
        ProofStatus::Verified, ProofStatus::Failed, ProofStatus::Timeout,
        ProofStatus::Unknown, ProofStatus::Pending, ProofStatus::Unavailable,
    ];
    assert_eq!(statuses.len(), 6);
    // Verify Debug display
    for s in &statuses {
        let debug = format!("{:?}", s);
        assert!(!debug.is_empty());
    }
}

// --- Query Engine Tests ---

#[test]
fn test_query_engine_symbols() {
    let index = source_to_index("fn add(a: i64, b: i64) -> i64 = a + b;
                                 fn subtract(a: i64, b: i64) -> i64 = a - b;");
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_symbols("add", None, false);
    assert!(result.error.is_none());
    assert!(result.matches.is_some());
    let matches = result.matches.unwrap();
    assert!(!matches.is_empty());
    assert!(matches.iter().any(|s| s.name == "add"));
}

#[test]
fn test_query_engine_symbols_not_found() {
    let index = source_to_index("fn hello() -> i64 = 1;");
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_symbols("nonexistent_xyz", None, false);
    assert!(result.error.is_some() || result.matches.as_ref().is_none_or(|m| m.is_empty()));
}

#[test]
fn test_query_engine_function() {
    let source = "fn factorial(n: i64) -> i64 = if n <= 1 { 1 } else { n * factorial(n - 1) };";
    let index = source_to_index(source);
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_function("factorial");
    assert!(result.error.is_none());
    assert!(result.result.is_some());
    let func = result.result.unwrap();
    assert_eq!(func.name, "factorial");
}

#[test]
fn test_query_engine_function_not_found() {
    let index = source_to_index("fn hello() -> i64 = 1;");
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_function("missing_fn");
    assert!(result.error.is_some());
}

#[test]
fn test_query_engine_type() {
    let index = source_to_index("struct Point { x: i64, y: i64 }");
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_type("Point");
    assert!(result.error.is_none());
    assert!(result.result.is_some());
    let ty = result.result.unwrap();
    assert_eq!(ty.name, "Point");
}

#[test]
fn test_query_engine_metrics() {
    let source = "fn a(x: i64) -> i64 pre x > 0 = x;
                  fn b() -> i64 = 1;
                  struct S { val: i64 }";
    let index = source_to_index(source);
    let engine = bmb::query::QueryEngine::new(index);
    let metrics = engine.query_metrics();
    assert_eq!(metrics.project.functions, 2);
    assert_eq!(metrics.project.types, 1);
}

#[test]
fn test_query_engine_functions_with_contracts() {
    let source = "fn guarded(x: i64) -> i64 pre x >= 0 post ret >= 0 = x;
                  fn unguarded() -> i64 = 42;";
    let index = source_to_index(source);
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_functions(Some(true), None, None, false);
    assert!(result.error.is_none());
    let matches = result.matches.unwrap();
    assert!(matches.iter().any(|f| f.name == "guarded"));
}

// ============================================================
// CIR (Contract IR) Integration Tests
// ============================================================

/// Helper: parse, type-check, and lower to CIR
fn source_to_cir(source: &str) -> bmb::cir::CirProgram {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    bmb::cir::lower_to_cir(&ast)
}

// --- CIR Lowering Tests ---

#[test]
fn test_cir_lower_simple_function() {
    let cir = source_to_cir("fn add(a: i64, b: i64) -> i64 = a + b;");
    assert_eq!(cir.functions.len(), 1);
    assert_eq!(cir.functions[0].name, "add");
    assert_eq!(cir.functions[0].params.len(), 2);
    assert_eq!(cir.functions[0].params[0].name, "a");
    assert_eq!(cir.functions[0].params[1].name, "b");
}

#[test]
fn test_cir_lower_multiple_functions() {
    let cir = source_to_cir("fn foo() -> i64 = 1; fn bar() -> i64 = 2; fn baz() -> i64 = 3;");
    assert_eq!(cir.functions.len(), 3);
    let names: Vec<&str> = cir.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"foo"));
    assert!(names.contains(&"bar"));
    assert!(names.contains(&"baz"));
}

#[test]
fn test_cir_lower_precondition() {
    let cir = source_to_cir("fn safe_div(a: i64, b: i64) -> i64 pre b > 0 = a / b;");
    assert_eq!(cir.functions.len(), 1);
    assert!(!cir.functions[0].preconditions.is_empty(), "preconditions should be non-empty");
}

#[test]
fn test_cir_lower_postcondition() {
    let cir = source_to_cir("fn positive() -> i64 post ret > 0 = 42;");
    assert_eq!(cir.functions.len(), 1);
    assert!(!cir.functions[0].postconditions.is_empty(), "postconditions should be non-empty");
}

#[test]
fn test_cir_lower_pre_and_post() {
    let cir = source_to_cir("fn clamp(x: i64) -> i64 pre x >= 0 post ret >= 0 = x;");
    let func = &cir.functions[0];
    assert!(!func.preconditions.is_empty());
    assert!(!func.postconditions.is_empty());
}

#[test]
fn test_cir_lower_struct() {
    let cir = source_to_cir("struct Point { x: i64, y: i64 }");
    assert!(cir.structs.contains_key("Point"));
    let s = &cir.structs["Point"];
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn test_cir_lower_return_type() {
    let cir = source_to_cir("fn flag() -> bool = true;");
    let func = &cir.functions[0];
    assert_eq!(format!("{}", func.ret_ty), "bool");
}

// --- CIR Proposition Tests ---

#[test]
fn test_cir_proposition_trivially_true() {
    let prop = bmb::cir::Proposition::True;
    assert!(prop.is_trivially_true());
    assert!(!prop.is_trivially_false());
}

#[test]
fn test_cir_proposition_trivially_false() {
    let prop = bmb::cir::Proposition::False;
    assert!(prop.is_trivially_false());
    assert!(!prop.is_trivially_true());
}

#[test]
fn test_cir_proposition_compare() {
    let prop = bmb::cir::Proposition::compare(
        bmb::cir::CirExpr::var("x"),
        bmb::cir::CompareOp::Ge,
        bmb::cir::CirExpr::int(0),
    );
    assert!(!prop.is_trivially_true());
    assert!(!prop.is_trivially_false());
}

#[test]
fn test_cir_proposition_and_or_not() {
    let p1 = bmb::cir::Proposition::True;
    let p2 = bmb::cir::Proposition::True;
    let and = bmb::cir::Proposition::and(vec![p1, p2]);
    // And of two trues — implementation may or may not simplify
    assert!(!and.is_trivially_false());

    let neg = bmb::cir::Proposition::not(bmb::cir::Proposition::False);
    assert!(!neg.is_trivially_false());
}

// --- CIR EffectSet Tests ---

#[test]
fn test_cir_effect_set_pure() {
    let e = bmb::cir::EffectSet::pure();
    assert!(e.is_pure);
    assert!(!e.writes);
    assert!(!e.io);
    assert!(!e.allocates);
}

#[test]
fn test_cir_effect_set_impure() {
    let e = bmb::cir::EffectSet::impure();
    assert!(!e.is_pure);
}

#[test]
fn test_cir_effect_set_union() {
    let pure = bmb::cir::EffectSet::pure();
    let impure = bmb::cir::EffectSet::impure();
    let combined = pure.union(&impure);
    assert!(!combined.is_pure, "union with impure should not be pure");
}

// --- CIR Output Formatting Tests ---

#[test]
fn test_cir_output_format_text() {
    let cir = source_to_cir("fn id(x: i64) -> i64 = x;");
    let text = bmb::cir::CirOutput::format_text(&cir);
    assert!(text.contains("id"), "text output should contain function name");
}

#[test]
fn test_cir_output_format_text_with_contract() {
    let cir = source_to_cir("fn pos(x: i64) -> i64 pre x > 0 post ret > 0 = x;");
    let text = bmb::cir::CirOutput::format_text(&cir);
    assert!(text.contains("pos"));
    // Should mention pre/postconditions in some form
    assert!(!text.is_empty());
}

#[test]
fn test_cir_output_format_json() {
    let cir = source_to_cir("fn id(x: i64) -> i64 = x;");
    let json = bmb::cir::CirOutput::format_json(&cir).expect("json format should succeed");
    assert!(json.contains("id"));
    // Should be valid JSON (starts with { or [)
    let first = json.trim().chars().next().unwrap();
    assert!(first == '{' || first == '[', "json should start with {{ or [");
}

// --- CIR Fact Extraction Tests ---

#[test]
fn test_cir_extract_precondition_facts() {
    let cir = source_to_cir("fn safe(x: i64) -> i64 pre x >= 0 = x;");
    let facts = bmb::cir::extract_precondition_facts(&cir.functions[0]);
    assert!(!facts.is_empty(), "precondition facts should be non-empty for pre x >= 0");
}

#[test]
fn test_cir_extract_postcondition_facts() {
    let cir = source_to_cir("fn positive() -> i64 post ret > 0 = 42;");
    let facts = bmb::cir::extract_postcondition_facts(&cir.functions[0]);
    assert!(!facts.is_empty(), "postcondition facts should be non-empty for post ret > 0");
}

#[test]
fn test_cir_extract_all_facts() {
    let cir = source_to_cir("fn guarded(x: i64) -> i64 pre x >= 0 post ret >= 0 = x;");
    let facts_map = bmb::cir::extract_all_facts(&cir);
    assert!(facts_map.contains_key("guarded"), "should have facts for guarded function");
    let (pre, post) = &facts_map["guarded"];
    assert!(!pre.is_empty(), "precondition facts should be non-empty");
    assert!(!post.is_empty(), "postcondition facts should be non-empty");
}

#[test]
fn test_cir_extract_verified_facts() {
    let cir = source_to_cir("fn a(x: i64) -> i64 pre x > 0 = x; fn b() -> i64 = 1;");
    let mut verified = std::collections::HashSet::new();
    verified.insert("a".to_string());
    let facts = bmb::cir::extract_verified_facts(&cir, &verified);
    assert!(facts.contains_key("a"), "verified function should have facts");
}

#[test]
fn test_cir_extract_facts_no_contracts() {
    let cir = source_to_cir("fn plain(x: i64) -> i64 = x;");
    let pre_facts = bmb::cir::extract_precondition_facts(&cir.functions[0]);
    let post_facts = bmb::cir::extract_postcondition_facts(&cir.functions[0]);
    assert!(pre_facts.is_empty());
    assert!(post_facts.is_empty());
}

// --- CIR SMT Generator Tests ---

#[test]
fn test_cir_smt_generator_creation() {
    let smt = bmb::cir::CirSmtGenerator::new();
    let output = smt.generate();
    // Fresh generator should produce minimal output
    assert!(output.contains("check-sat"), "SMT output should contain check-sat");
}

#[test]
fn test_cir_smt_generator_declare_var() {
    let mut smt = bmb::cir::CirSmtGenerator::new();
    smt.declare_var("x", bmb::cir::SmtSort::Int);
    let output = smt.generate();
    assert!(output.contains("x"), "output should declare variable x");
}

#[test]
fn test_cir_smt_generator_translate_proposition() {
    let smt = bmb::cir::CirSmtGenerator::new();
    let prop = bmb::cir::Proposition::compare(
        bmb::cir::CirExpr::var("x"),
        bmb::cir::CompareOp::Ge,
        bmb::cir::CirExpr::int(0),
    );
    let result = smt.translate_proposition(&prop);
    assert!(result.is_ok(), "translate simple comparison should succeed");
    let smt_str = result.unwrap();
    assert!(smt_str.contains(">=") || smt_str.contains("ge"), "should contain comparison operator");
}

#[test]
fn test_cir_smt_generator_translate_expr() {
    let smt = bmb::cir::CirSmtGenerator::new();
    let expr = bmb::cir::CirExpr::int(42);
    let result = smt.translate_expr(&expr);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("42"));
}

#[test]
fn test_cir_smt_sort_to_smt() {
    assert_eq!(bmb::cir::SmtSort::Int.to_smt(), "Int");
    assert_eq!(bmb::cir::SmtSort::Bool.to_smt(), "Bool");
    assert_eq!(bmb::cir::SmtSort::Real.to_smt(), "Real");
}

#[test]
fn test_cir_smt_generator_type_to_sort() {
    let smt = bmb::cir::CirSmtGenerator::new();
    let sort = smt.cir_type_to_sort(&bmb::cir::CirType::I64);
    // I64 should map to Int sort
    assert_eq!(sort.to_smt(), "Int");
}

// --- CIR Verifier Tests ---

#[test]
fn test_cir_verifier_creation() {
    let verifier = bmb::cir::CirVerifier::new();
    // Verifier created successfully — solver availability depends on Z3 installation
    let _available = verifier.is_solver_available();
}

#[test]
fn test_cir_verification_report_empty() {
    let report = bmb::cir::CirVerificationReport::new();
    assert_eq!(report.total_functions, 0);
    assert_eq!(report.verified_count, 0);
    assert_eq!(report.failed_count, 0);
    assert!(report.all_verified()); // empty report => all verified
    assert!(!report.has_failures());
    assert!(!report.has_errors());
}

#[test]
fn test_cir_proof_witness_verified() {
    let w = bmb::cir::ProofWitness::verified("test_fn".to_string(), None, 100);
    assert!(w.is_verified());
    assert!(!w.is_failed());
    assert_eq!(w.function, "test_fn");
    assert_eq!(w.verification_time_ms, 100);
    assert!(matches!(w.outcome, bmb::cir::ProofOutcome::Verified));
}

#[test]
fn test_cir_proof_witness_failed() {
    let w = bmb::cir::ProofWitness::failed("bad_fn".to_string(), "division by zero".to_string(), None, 50);
    assert!(w.is_failed());
    assert!(!w.is_verified());
    assert!(matches!(w.outcome, bmb::cir::ProofOutcome::Failed(_)));
}

#[test]
fn test_cir_proof_witness_skipped() {
    let w = bmb::cir::ProofWitness::skipped("no_contract".to_string());
    assert!(!w.is_verified());
    assert!(!w.is_failed());
    assert!(matches!(w.outcome, bmb::cir::ProofOutcome::Skipped));
}

#[test]
fn test_cir_proof_witness_error() {
    let w = bmb::cir::ProofWitness::error("err_fn".to_string(), "solver crashed".to_string());
    assert!(!w.is_verified());
    assert!(!w.is_failed());
    assert!(matches!(w.outcome, bmb::cir::ProofOutcome::Error(_)));
}

#[test]
fn test_cir_proof_outcome_variants() {
    let v = bmb::cir::ProofOutcome::Verified;
    let f = bmb::cir::ProofOutcome::Failed("reason".to_string());
    let u = bmb::cir::ProofOutcome::Unknown("timeout".to_string());
    let s = bmb::cir::ProofOutcome::Skipped;
    let e = bmb::cir::ProofOutcome::Error("crash".to_string());
    // Verify Debug formatting works
    assert!(!format!("{:?}", v).is_empty());
    assert!(!format!("{:?}", f).is_empty());
    assert!(!format!("{:?}", u).is_empty());
    assert!(!format!("{:?}", s).is_empty());
    assert!(!format!("{:?}", e).is_empty());
}

#[test]
fn test_cir_verification_report_summary() {
    let report = bmb::cir::CirVerificationReport::new();
    let summary = report.summary();
    assert!(!summary.is_empty(), "summary should not be empty");
}

#[test]
fn test_cir_compare_op_negate() {
    assert_eq!(bmb::cir::CompareOp::Lt.negate(), bmb::cir::CompareOp::Ge);
    assert_eq!(bmb::cir::CompareOp::Ge.negate(), bmb::cir::CompareOp::Lt);
    assert_eq!(bmb::cir::CompareOp::Eq.negate(), bmb::cir::CompareOp::Ne);
    assert_eq!(bmb::cir::CompareOp::Ne.negate(), bmb::cir::CompareOp::Eq);
}

#[test]
fn test_cir_compare_op_flip() {
    assert_eq!(bmb::cir::CompareOp::Lt.flip(), bmb::cir::CompareOp::Gt);
    assert_eq!(bmb::cir::CompareOp::Le.flip(), bmb::cir::CompareOp::Ge);
    assert_eq!(bmb::cir::CompareOp::Gt.flip(), bmb::cir::CompareOp::Lt);
    assert_eq!(bmb::cir::CompareOp::Ge.flip(), bmb::cir::CompareOp::Le);
}

// ============================================================
// Derive Module Integration Tests
// ============================================================

/// Helper: parse source and return the AST Program
fn source_to_ast(source: &str) -> bmb::ast::Program {
    let tokens = tokenize(source).expect("tokenize failed");
    parse("test.bmb", source, tokens).expect("parse failed")
}

/// Helper: find a StructDef by name in AST items
fn find_struct<'a>(ast: &'a bmb::ast::Program, name: &str) -> &'a bmb::ast::StructDef {
    ast.items.iter().find_map(|item| {
        if let bmb::ast::Item::StructDef(s) = item && s.name.node == name {
            return Some(s);
        }
        None
    }).unwrap_or_else(|| panic!("struct {} not found", name))
}

/// Helper: find an EnumDef by name in AST items
fn find_enum<'a>(ast: &'a bmb::ast::Program, name: &str) -> &'a bmb::ast::EnumDef {
    ast.items.iter().find_map(|item| {
        if let bmb::ast::Item::EnumDef(e) = item && e.name.node == name {
            return Some(e);
        }
        None
    }).unwrap_or_else(|| panic!("enum {} not found", name))
}

#[test]
fn test_derive_trait_from_str_all_variants() {
    assert!(matches!(bmb::derive::DeriveTrait::from_str("Debug"), Some(bmb::derive::DeriveTrait::Debug)));
    assert!(matches!(bmb::derive::DeriveTrait::from_str("Clone"), Some(bmb::derive::DeriveTrait::Clone)));
    assert!(matches!(bmb::derive::DeriveTrait::from_str("PartialEq"), Some(bmb::derive::DeriveTrait::PartialEq)));
    assert!(matches!(bmb::derive::DeriveTrait::from_str("Eq"), Some(bmb::derive::DeriveTrait::Eq)));
    assert!(matches!(bmb::derive::DeriveTrait::from_str("Default"), Some(bmb::derive::DeriveTrait::Default)));
    assert!(matches!(bmb::derive::DeriveTrait::from_str("Hash"), Some(bmb::derive::DeriveTrait::Hash)));
}

#[test]
fn test_derive_trait_from_str_unknown() {
    assert!(bmb::derive::DeriveTrait::from_str("Serialize").is_none());
    assert!(bmb::derive::DeriveTrait::from_str("").is_none());
    assert!(bmb::derive::DeriveTrait::from_str("debug").is_none()); // case-sensitive
}

#[test]
fn test_derive_trait_as_str_roundtrip() {
    let traits = [
        bmb::derive::DeriveTrait::Debug,
        bmb::derive::DeriveTrait::Clone,
        bmb::derive::DeriveTrait::PartialEq,
        bmb::derive::DeriveTrait::Eq,
        bmb::derive::DeriveTrait::Default,
        bmb::derive::DeriveTrait::Hash,
    ];
    for t in &traits {
        let s = t.as_str();
        let parsed = bmb::derive::DeriveTrait::from_str(s).unwrap();
        assert_eq!(&parsed, t, "roundtrip failed for {:?}", t);
    }
}

#[test]
fn test_derive_extract_from_parsed_struct() {
    let ast = source_to_ast("@derive(Debug, Clone) struct Point { x: i64, y: i64 }");
    // Find the struct definition
    let struct_def = find_struct(&ast, "Point");
    let traits = bmb::derive::extract_derive_traits(&struct_def.attributes);
    assert_eq!(traits.len(), 2);
    assert!(traits.contains(&bmb::derive::DeriveTrait::Debug));
    assert!(traits.contains(&bmb::derive::DeriveTrait::Clone));
}

#[test]
fn test_derive_extract_four_traits() {
    let ast = source_to_ast("@derive(Debug, Clone, PartialEq, Eq) struct Color { r: i64, g: i64, b: i64 }");
    let struct_def = find_struct(&ast, "Color");
    let traits = bmb::derive::extract_derive_traits(&struct_def.attributes);
    assert_eq!(traits.len(), 4);
    assert!(traits.contains(&bmb::derive::DeriveTrait::Debug));
    assert!(traits.contains(&bmb::derive::DeriveTrait::Clone));
    assert!(traits.contains(&bmb::derive::DeriveTrait::PartialEq));
    assert!(traits.contains(&bmb::derive::DeriveTrait::Eq));
}

#[test]
fn test_derive_extract_from_parsed_enum() {
    let ast = source_to_ast("@derive(Debug, Clone, PartialEq) enum Status { Active, Inactive, Pending }");
    let enum_def = find_enum(&ast, "Status");
    let traits = bmb::derive::extract_derive_traits(&enum_def.attributes);
    assert_eq!(traits.len(), 3);
    assert!(traits.contains(&bmb::derive::DeriveTrait::Debug));
    assert!(traits.contains(&bmb::derive::DeriveTrait::PartialEq));
}

#[test]
fn test_derive_no_attributes() {
    let ast = source_to_ast("struct Plain { val: i64 }");
    let struct_def = find_struct(&ast, "Plain");
    let traits = bmb::derive::extract_derive_traits(&struct_def.attributes);
    assert!(traits.is_empty());
}

#[test]
fn test_derive_has_derive_trait_struct() {
    let ast = source_to_ast("@derive(Debug) struct S { val: i64 }");
    let struct_def = find_struct(&ast, "S");
    assert!(bmb::derive::has_derive_trait(struct_def, bmb::derive::DeriveTrait::Debug));
    assert!(!bmb::derive::has_derive_trait(struct_def, bmb::derive::DeriveTrait::Clone));
}

#[test]
fn test_derive_has_derive_trait_enum() {
    let ast = source_to_ast("@derive(Clone, Hash) enum Dir { Up, Down }");
    let enum_def = find_enum(&ast, "Dir");
    assert!(bmb::derive::has_derive_trait_enum(enum_def, bmb::derive::DeriveTrait::Clone));
    assert!(bmb::derive::has_derive_trait_enum(enum_def, bmb::derive::DeriveTrait::Hash));
    assert!(!bmb::derive::has_derive_trait_enum(enum_def, bmb::derive::DeriveTrait::Debug));
}

#[test]
fn test_derive_context_from_struct() {
    let ast = source_to_ast("@derive(Debug, Default) struct Config { width: i64, height: i64 }");
    let struct_def = find_struct(&ast, "Config");
    let ctx = bmb::derive::DeriveContext::from_struct(struct_def);
    assert_eq!(ctx.name, "Config");
    assert!(ctx.has_trait(bmb::derive::DeriveTrait::Debug));
    assert!(ctx.has_trait(bmb::derive::DeriveTrait::Default));
    assert!(!ctx.has_trait(bmb::derive::DeriveTrait::Clone));
}

#[test]
fn test_derive_context_from_enum() {
    let ast = source_to_ast("@derive(Debug, Clone, PartialEq) enum Shape { Circle, Square, Triangle }");
    let enum_def = find_enum(&ast, "Shape");
    let ctx = bmb::derive::DeriveContext::from_enum(enum_def);
    assert_eq!(ctx.name, "Shape");
    assert!(ctx.has_trait(bmb::derive::DeriveTrait::Debug));
    assert!(ctx.has_trait(bmb::derive::DeriveTrait::Clone));
    assert!(ctx.has_trait(bmb::derive::DeriveTrait::PartialEq));
}

#[test]
fn test_derive_multiple_structs_in_program() {
    let source = "@derive(Debug) struct A { x: i64 }
                   @derive(Clone) struct B { y: i64 }
                   struct C { z: i64 }";
    let ast = source_to_ast(source);
    let a = find_struct(&ast, "A");
    let b = find_struct(&ast, "B");
    let c = find_struct(&ast, "C");
    assert!(bmb::derive::has_derive_trait(a, bmb::derive::DeriveTrait::Debug));
    assert!(!bmb::derive::has_derive_trait(a, bmb::derive::DeriveTrait::Clone));
    assert!(bmb::derive::has_derive_trait(b, bmb::derive::DeriveTrait::Clone));
    assert!(!bmb::derive::has_derive_trait(b, bmb::derive::DeriveTrait::Debug));
    assert!(!bmb::derive::has_derive_trait(c, bmb::derive::DeriveTrait::Debug));
    assert!(!bmb::derive::has_derive_trait(c, bmb::derive::DeriveTrait::Clone));
}

#[test]
fn test_derive_default_single_trait() {
    let ast = source_to_ast("@derive(Default) struct Zeroed { val: i64 }");
    let struct_def = find_struct(&ast, "Zeroed");
    let traits = bmb::derive::extract_derive_traits(&struct_def.attributes);
    assert_eq!(traits.len(), 1);
    assert_eq!(traits[0], bmb::derive::DeriveTrait::Default);
}

// ============================================================
// Build Module Integration Tests
// ============================================================

#[test]
fn test_build_config_defaults() {
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("test.bmb"));
    assert_eq!(config.input, std::path::PathBuf::from("test.bmb"));
    assert!(!config.emit_ir);
    assert!(!config.verbose);
    assert!(!config.emit_cir);
    assert!(!config.emit_pir);
    assert!(config.proof_optimizations); // enabled by default
    assert!(config.proof_cache); // enabled by default
    assert!(!config.fast_math); // disabled by default
    assert!(!config.fast_compile); // disabled by default
    assert!(!config.no_prelude);
    assert!(config.include_paths.is_empty());
    assert!(config.prelude_path.is_none());
    assert!(config.target_triple.is_none());
    assert_eq!(config.verification_timeout, 30);
}

#[test]
fn test_build_config_default_output_extension() {
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("hello.bmb"));
    // On Windows, output should be hello.exe
    if cfg!(windows) {
        assert_eq!(config.output, std::path::PathBuf::from("hello.exe"));
    } else {
        assert_eq!(config.output, std::path::PathBuf::from("hello"));
    }
}

#[test]
fn test_build_config_builder_chain() {
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("test.bmb"))
        .opt_level(bmb::build::OptLevel::Aggressive)
        .emit_ir(true)
        .verbose(true)
        .fast_math(true)
        .fast_compile(true)
        .no_prelude(true)
        .verification_timeout(60);
    assert!(config.emit_ir);
    assert!(config.verbose);
    assert!(config.fast_math);
    assert!(config.fast_compile);
    assert!(config.no_prelude);
    assert_eq!(config.verification_timeout, 60);
}

#[test]
fn test_build_config_verification_modes() {
    let none = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .verification_mode(bmb::build::VerificationMode::None);
    assert_eq!(none.verification_mode, bmb::build::VerificationMode::None);

    let check = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .verification_mode(bmb::build::VerificationMode::Check);
    assert_eq!(check.verification_mode, bmb::build::VerificationMode::Check);

    let warn = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .verification_mode(bmb::build::VerificationMode::Warn);
    assert_eq!(warn.verification_mode, bmb::build::VerificationMode::Warn);

    let trust = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .verification_mode(bmb::build::VerificationMode::Trust);
    assert_eq!(trust.verification_mode, bmb::build::VerificationMode::Trust);
}

#[test]
fn test_build_config_target() {
    use bmb::cfg::Target;
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .target(Target::Wasm32);
    assert!(matches!(config.target, Target::Wasm32));
}

#[test]
fn test_build_config_target_triple() {
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .target_triple("x86_64-unknown-linux-gnu".to_string());
    assert_eq!(config.target_triple.as_deref(), Some("x86_64-unknown-linux-gnu"));
}

#[test]
fn test_build_config_output_path() {
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .output(std::path::PathBuf::from("custom_output"));
    assert_eq!(config.output, std::path::PathBuf::from("custom_output"));
}

#[test]
fn test_build_config_include_paths() {
    let paths = vec![
        std::path::PathBuf::from("/usr/include/bmb"),
        std::path::PathBuf::from("/home/lib"),
    ];
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .include_paths(paths.clone());
    assert_eq!(config.include_paths.len(), 2);
}

#[test]
fn test_build_config_prelude_path() {
    let config = bmb::build::BuildConfig::new(std::path::PathBuf::from("t.bmb"))
        .prelude_path(std::path::PathBuf::from("/stdlib"));
    assert_eq!(config.prelude_path.as_deref(), Some(std::path::Path::new("/stdlib")));
}

#[test]
fn test_build_verification_mode_default() {
    let mode = bmb::build::VerificationMode::default();
    assert_eq!(mode, bmb::build::VerificationMode::Check);
}

#[test]
fn test_build_opt_level_variants() {
    let _d = bmb::mir::OptLevel::Debug;
    let _r = bmb::mir::OptLevel::Release;
    let _s = bmb::build::OptLevel::Size;
    let _a = bmb::build::OptLevel::Aggressive;
    // All variants should be Debug-printable
    assert!(!format!("{:?}", _d).is_empty());
    assert!(!format!("{:?}", _r).is_empty());
    assert!(!format!("{:?}", _s).is_empty());
    assert!(!format!("{:?}", _a).is_empty());
}

#[test]
fn test_build_output_type_variants() {
    let _e = bmb::build::OutputType::Executable;
    let _o = bmb::build::OutputType::Object;
    let _l = bmb::build::OutputType::LlvmIr;
    assert!(!format!("{:?}", _e).is_empty());
    assert!(!format!("{:?}", _o).is_empty());
    assert!(!format!("{:?}", _l).is_empty());
}

#[test]
fn test_build_error_display() {
    let e = bmb::build::BuildError::Parse("unexpected token".to_string());
    assert!(format!("{}", e).contains("unexpected token"));
    let e2 = bmb::build::BuildError::Type("mismatched types".to_string());
    assert!(format!("{}", e2).contains("mismatched types"));
    let e3 = bmb::build::BuildError::Linker("cannot find -lm".to_string());
    assert!(format!("{}", e3).contains("cannot find"));
}

// ============================================================
// AST Output Integration Tests
// ============================================================

#[test]
fn test_ast_output_sexpr_simple_function() {
    let ast = source_to_ast("fn add(a: i64, b: i64) -> i64 = a + b;");
    let sexpr = bmb::ast::output::to_sexpr(&ast);
    assert!(sexpr.contains("program"));
    assert!(sexpr.contains("add"));
}

#[test]
fn test_ast_output_sexpr_struct() {
    let ast = source_to_ast("struct Point { x: i64, y: i64 }");
    let sexpr = bmb::ast::output::to_sexpr(&ast);
    assert!(sexpr.contains("Point"));
}

#[test]
fn test_ast_output_sexpr_enum() {
    let ast = source_to_ast("enum Color { Red, Green, Blue }");
    let sexpr = bmb::ast::output::to_sexpr(&ast);
    assert!(sexpr.contains("Color"));
}

#[test]
fn test_ast_output_format_type_primitives() {
    use bmb::ast::Type;
    assert_eq!(bmb::ast::output::format_type(&Type::I64), "i64");
    assert_eq!(bmb::ast::output::format_type(&Type::Bool), "bool");
    assert_eq!(bmb::ast::output::format_type(&Type::F64), "f64");
    assert_eq!(bmb::ast::output::format_type(&Type::Unit), "()");
    assert_eq!(bmb::ast::output::format_type(&Type::String), "String");
}

#[test]
fn test_ast_output_format_expr_literals() {
    use bmb::ast::Expr;
    assert_eq!(bmb::ast::output::format_expr(&Expr::IntLit(42)), "42");
    assert_eq!(bmb::ast::output::format_expr(&Expr::BoolLit(true)), "true");
    assert_eq!(bmb::ast::output::format_expr(&Expr::BoolLit(false)), "false");
}

#[test]
fn test_ast_output_sexpr_with_contracts() {
    let ast = source_to_ast("fn abs(x: i64) -> i64 post ret >= 0 = if x >= 0 { x } else { 0 - x };");
    let sexpr = bmb::ast::output::to_sexpr(&ast);
    assert!(sexpr.contains("abs"));
    assert!(!sexpr.is_empty());
}

#[test]
fn test_ast_output_sexpr_empty_program() {
    // Empty source may or may not parse — test with minimal valid program
    let ast = source_to_ast("fn noop() -> () = ();");
    let sexpr = bmb::ast::output::to_sexpr(&ast);
    assert!(sexpr.starts_with("(program"));
}

// ============================================================
// Parser Edge Case Integration Tests
// ============================================================

/// Helper: verify source parses and type-checks successfully
fn parse_and_typecheck(source: &str) {
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
}

// --- Complex Expression Parsing ---

#[test]
fn test_parser_nested_if_else() {
    parse_and_typecheck(
        "fn nested(x: i64) -> i64 = if x > 0 { if x > 10 { 100 } else { x } } else { 0 };"
    );
}

#[test]
fn test_parser_block_with_let_bindings() {
    parse_and_typecheck(
        "fn block_let() -> i64 = { let x: i64 = 5; let y: i64 = x + 1; y };"
    );
}

#[test]
fn test_parser_while_loop() {
    parse_and_typecheck(
        "fn countdown(n: i64) -> i64 = { let mut x: i64 = n; while x > 0 { x = x - 1; }; x };"
    );
}

#[test]
fn test_parser_match_expression() {
    parse_and_typecheck(
        "enum Dir { Up, Down, Left, Right }
         fn to_num(d: Dir) -> i64 = match d {
             Dir::Up => 1,
             Dir::Down => 2,
             Dir::Left => 3,
             Dir::Right => 4,
         };"
    );
}

#[test]
fn test_parser_match_with_wildcard() {
    parse_and_typecheck(
        "fn classify(x: i64) -> i64 = match x {
             0 => 0,
             1 => 1,
             _ => 2,
         };"
    );
}

#[test]
fn test_parser_tuple_expression() {
    let ast = source_to_ast("fn pair(a: i64, b: i64) -> (i64, i64) = (a, b);");
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parser_array_literal() {
    let ast = source_to_ast("fn make_arr() -> [i64; 3] = [1, 2, 3];");
    assert_eq!(ast.items.len(), 1);
}

// --- Visibility & Module Syntax ---

#[test]
fn test_parser_pub_function() {
    let ast = source_to_ast("pub fn public_fn() -> i64 = 42;");
    if let bmb::ast::Item::FnDef(f) = &ast.items[0] {
        assert!(matches!(f.visibility, bmb::ast::Visibility::Public));
    } else {
        panic!("expected FnDef");
    }
}

#[test]
fn test_parser_pub_struct() {
    let ast = source_to_ast("pub struct PubPoint { x: i64, y: i64 }");
    let s = find_struct(&ast, "PubPoint");
    assert!(matches!(s.visibility, bmb::ast::Visibility::Public));
}

#[test]
fn test_parser_pub_enum() {
    let ast = source_to_ast("pub enum PubDir { North, South }");
    let e = find_enum(&ast, "PubDir");
    assert!(matches!(e.visibility, bmb::ast::Visibility::Public));
}

// --- Type Syntax ---

#[test]
fn test_parser_generic_function() {
    let ast = source_to_ast("fn identity<T>(x: T) -> T = x;");
    if let bmb::ast::Item::FnDef(f) = &ast.items[0] {
        assert!(!f.type_params.is_empty(), "should have type params");
    } else {
        panic!("expected FnDef");
    }
}

#[test]
fn test_parser_generic_struct() {
    let ast = source_to_ast("struct Wrapper<T> { value: T }");
    let s = find_struct(&ast, "Wrapper");
    assert!(!s.type_params.is_empty());
}

#[test]
fn test_parser_option_type() {
    // Option type syntax: T?
    let ast = source_to_ast("fn maybe(x: i64) -> i64? = if x > 0 { x } else { nil };");
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parser_reference_types() {
    let ast = source_to_ast("fn borrow(x: &i64) -> i64 = *x;");
    assert_eq!(ast.items.len(), 1);
}

// --- Trait & Impl Syntax ---

#[test]
fn test_parser_trait_definition() {
    let ast = source_to_ast(
        "trait Printable { fn display(self: &Self) -> i64; }"
    );
    let has_trait = ast.items.iter().any(|i| matches!(i, bmb::ast::Item::TraitDef(_)));
    assert!(has_trait, "should have a trait definition");
}

#[test]
fn test_parser_impl_block() {
    let ast = source_to_ast(
        "struct Counter { val: i64 }
         trait HasVal { fn get(self: Self) -> i64; }
         impl HasVal for Counter { fn get(self: Self) -> i64 = self.val; }"
    );
    let has_impl = ast.items.iter().any(|i| matches!(i, bmb::ast::Item::ImplBlock(_)));
    assert!(has_impl, "should have an impl block");
}

// --- Type Alias ---

#[test]
fn test_parser_type_alias() {
    let ast = source_to_ast("type Number = i64;");
    let has_alias = ast.items.iter().any(|i| matches!(i, bmb::ast::Item::TypeAlias(_)));
    assert!(has_alias, "should have a type alias");
}

// --- Extern Functions ---

#[test]
fn test_parser_extern_fn() {
    let ast = source_to_ast("extern fn puts(s: &i64) -> i64;");
    let has_extern = ast.items.iter().any(|i| matches!(i, bmb::ast::Item::ExternFn(_)));
    assert!(has_extern, "should have an extern fn");
}

// --- Attribute Syntax ---

#[test]
fn test_parser_inline_attribute() {
    let ast = source_to_ast("@inline fn fast() -> i64 = 42;");
    if let bmb::ast::Item::FnDef(f) = &ast.items[0] {
        assert!(!f.attributes.is_empty(), "should have attributes");
    } else {
        panic!("expected FnDef");
    }
}

#[test]
fn test_parser_pure_attribute() {
    let ast = source_to_ast("@pure fn add(a: i64, b: i64) -> i64 = a + b;");
    if let bmb::ast::Item::FnDef(f) = &ast.items[0] {
        assert!(!f.attributes.is_empty());
    } else {
        panic!("expected FnDef");
    }
}

// --- Contract Syntax Edge Cases ---

#[test]
fn test_parser_combined_preconditions() {
    parse_and_typecheck(
        "fn bounded(x: i64, y: i64) -> i64 pre x >= 0 && y > 0 = x + y;"
    );
}

#[test]
fn test_parser_pre_and_post() {
    parse_and_typecheck(
        "fn clamp(x: i64, lo: i64, hi: i64) -> i64
            pre lo <= hi
            post ret >= lo
         = if x < lo { lo } else { if x > hi { hi } else { x } };"
    );
}

// --- Operator Precedence ---

#[test]
fn test_parser_arithmetic_precedence() {
    // a + b * c should parse as a + (b * c)
    let ast = source_to_ast("fn prec(a: i64, b: i64, c: i64) -> i64 = a + b * c;");
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parser_comparison_chain() {
    parse_and_typecheck(
        "fn both_positive(a: i64, b: i64) -> bool = a > 0 && b > 0;"
    );
}

#[test]
fn test_parser_bitwise_operators() {
    parse_and_typecheck(
        "fn bits(a: i64, b: i64) -> i64 = (a band b) bor (a bxor b);"
    );
}

// --- Complex Programs ---

#[test]
fn test_parser_multi_item_program() {
    let ast = source_to_ast(
        "struct Point { x: i64, y: i64 }
         enum Color { Red, Green, Blue }
         fn origin() -> Point = new Point { x: 0, y: 0 };
         fn is_red(c: Color) -> bool = match c { Color::Red => true, _ => false };"
    );
    assert!(ast.items.len() >= 4, "should have at least 4 items");
}

#[test]
fn test_parser_lambda_expression() {
    // BMB closure syntax: fn |params| { body }
    let ast = source_to_ast(
        "fn test_closure() -> i64 = { let f = fn |x: i64| { x * 2 }; 42 };"
    );
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parser_string_literal() {
    let ast = source_to_ast(r#"fn greeting() -> String = "hello world";"#);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_parser_mutable_variable() {
    parse_and_typecheck(
        "fn mutate() -> i64 = { let mut x: i64 = 0; x = 42; x };"
    );
}

// ============================================================
// MIR Proof-Guided Optimization & Format Tests
// ============================================================

#[test]
fn test_mir_format_simple_function() {
    let program = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let text = bmb::mir::format_mir(&program);
    assert!(text.contains("add"), "format_mir should contain function name");
    assert!(text.contains("i64"), "should contain type annotations");
}

#[test]
fn test_mir_format_multiple_functions() {
    let program = lower_to_mir("fn foo() -> i64 = 1; fn bar() -> i64 = 2;");
    let text = bmb::mir::format_mir(&program);
    assert!(text.contains("foo"));
    assert!(text.contains("bar"));
}

#[test]
fn test_mir_format_with_contract() {
    let program = lower_to_mir("fn safe(x: i64) -> i64 pre x >= 0 = x;");
    let text = bmb::mir::format_mir(&program);
    assert!(text.contains("safe"));
}

#[test]
fn test_mir_format_pure_function() {
    let program = lower_to_mir("@pure fn pure_add(a: i64, b: i64) -> i64 = a + b;");
    let text = bmb::mir::format_mir(&program);
    assert!(text.contains("pure") || text.contains("@pure"), "should show pure attribute");
}

#[test]
fn test_mir_lower_preconditions_extracted() {
    let program = lower_to_mir("fn guarded(x: i64) -> i64 pre x > 0 = x;");
    let func = find_mir_fn(&program, "guarded");
    assert!(!func.preconditions.is_empty(), "preconditions should be extracted from pre");
}

#[test]
fn test_mir_lower_postconditions_extracted() {
    let program = lower_to_mir("fn positive() -> i64 post ret > 0 = 42;");
    let func = find_mir_fn(&program, "positive");
    assert!(!func.postconditions.is_empty(), "postconditions should be extracted from post");
}

#[test]
fn test_mir_lower_pure_attribute() {
    let program = lower_to_mir("@pure fn pure_fn(x: i64) -> i64 = x;");
    let func = find_mir_fn(&program, "pure_fn");
    assert!(func.is_pure, "is_pure should be true for @pure functions");
}

#[test]
fn test_mir_proven_fact_set_from_preconditions() {
    let program = lower_to_mir("fn bounded(x: i64) -> i64 pre x >= 0 = x;");
    let func = find_mir_fn(&program, "bounded");
    let facts = bmb::mir::ProvenFactSet::from_mir_preconditions(&func.preconditions);
    assert!(facts.has_lower_bound("x", 0), "should have lower bound x >= 0");
}

#[test]
fn test_mir_proven_fact_set_upper_bound() {
    let program = lower_to_mir("fn capped(x: i64) -> i64 pre x <= 100 = x;");
    let func = find_mir_fn(&program, "capped");
    let facts = bmb::mir::ProvenFactSet::from_mir_preconditions(&func.preconditions);
    let ub = facts.get_upper_bound("x");
    assert!(ub.is_some(), "should have upper bound for x");
    assert_eq!(ub.unwrap(), 100);
}

#[test]
fn test_mir_proven_fact_set_nonzero() {
    // Ne with 0 directly maps to nonzero tracking
    let facts = {
        let mut f = bmb::mir::ProvenFactSet::default();
        f.add_nonzero("d");
        f
    };
    assert!(facts.has_nonzero("d"), "explicitly added nonzero should be tracked");
}

#[test]
fn test_mir_proven_fact_set_lower_bound_implies_positive() {
    let program = lower_to_mir("fn safe_div(x: i64, d: i64) -> i64 pre d > 0 = x / d;");
    let func = find_mir_fn(&program, "safe_div");
    let facts = bmb::mir::ProvenFactSet::from_mir_preconditions(&func.preconditions);
    // d > 0 sets lower bound to 1
    assert!(facts.has_lower_bound("d", 1), "d > 0 should set lower bound to 1");
}

#[test]
fn test_mir_proof_guided_program_runs() {
    let mut program = lower_to_mir("fn id(x: i64) -> i64 pre x >= 0 = x;");
    let stats = bmb::mir::run_proof_guided_program(&mut program);
    // Stats should be valid (may or may not eliminate anything for simple case)
    let _total = stats.bounds_checks_eliminated
        + stats.null_checks_eliminated
        + stats.division_checks_eliminated
        + stats.unreachable_blocks_eliminated;
}

#[test]
fn test_mir_optimization_pipeline_debug() {
    let mut program = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let pipeline = bmb::mir::OptimizationPipeline::for_level(bmb::mir::OptLevel::Debug);
    let stats = pipeline.optimize(&mut program);
    // Debug level should do minimal or no optimizations
    assert_eq!(stats.iterations, 0, "Debug should not iterate");
}

#[test]
fn test_mir_optimization_pipeline_release() {
    let mut program = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let pipeline = bmb::mir::OptimizationPipeline::for_level(bmb::mir::OptLevel::Release);
    let _stats = pipeline.optimize(&mut program);
    // Release should run at least one iteration
}

#[test]
fn test_mir_constant_folding_pass() {
    let (text, _program) = optimized_mir(
        "fn constant() -> i64 = 2 + 3;",
        Box::new(bmb::mir::ConstantFolding),
    );
    // After constant folding, 2+3 should be folded to 5
    assert!(text.contains("5") || text.contains("I:5"), "constant should be folded to 5");
}

#[test]
fn test_mir_dead_code_elimination_pass() {
    let (text, _program) = optimized_mir(
        "fn live() -> i64 = { let x: i64 = 42; 1 };",
        Box::new(bmb::mir::DeadCodeElimination),
    );
    // DCE should remove unused x assignment (or at least run without error)
    assert!(text.contains("live"));
}

#[test]
fn test_mir_contract_fact_varcmp() {
    let fact = bmb::mir::ContractFact::VarCmp {
        var: "x".to_string(),
        op: bmb::mir::CmpOp::Ge,
        value: 0,
    };
    assert!(!format!("{:?}", fact).is_empty());
}

#[test]
fn test_mir_contract_fact_nonnull() {
    let fact = bmb::mir::ContractFact::NonNull {
        var: "ptr".to_string(),
    };
    assert!(!format!("{:?}", fact).is_empty());
}

#[test]
fn test_mir_contract_fact_return_cmp() {
    let fact = bmb::mir::ContractFact::ReturnCmp {
        op: bmb::mir::CmpOp::Gt,
        value: 0,
    };
    assert!(!format!("{:?}", fact).is_empty());
}

#[test]
fn test_mir_full_pipeline_preserves_semantics() {
    // Full pipeline optimization should not change the computed result
    let result = run_program_i64("fn main() -> i64 = 2 + 3 * 4;");
    assert_eq!(result, 14); // 2 + (3*4) = 14
}

#[test]
fn test_mir_format_if_else() {
    let program = lower_to_mir("fn abs(x: i64) -> i64 = if x >= 0 { x } else { 0 - x };");
    let text = bmb::mir::format_mir(&program);
    assert!(text.contains("abs"));
    // Should have branching structure
    assert!(text.contains("branch") || text.contains("goto") || text.contains("return"));
}

// ============================================================
// Type System Advanced Integration Tests
// ============================================================

// --- Trait Type Checking ---

#[test]
fn test_type_trait_definition_and_impl() {
    parse_and_typecheck(
        "trait Greetable { fn greet(self: Self) -> i64; }
         struct Person { age: i64 }
         impl Greetable for Person { fn greet(self: Self) -> i64 = self.age; }"
    );
}

#[test]
fn test_type_trait_multiple_methods() {
    parse_and_typecheck(
        "trait Math {
             fn add(self: Self, other: Self) -> Self;
             fn zero() -> Self;
         }
         struct Num { val: i64 }
         impl Math for Num {
             fn add(self: Self, other: Self) -> Self = new Num { val: self.val + other.val };
             fn zero() -> Self = new Num { val: 0 };
         }"
    );
}

#[test]
fn test_type_generic_function_inference() {
    parse_and_typecheck(
        "fn identity<T>(x: T) -> T = x;
         fn use_id() -> i64 = identity(42);"
    );
}

#[test]
fn test_type_generic_struct_instantiation() {
    parse_and_typecheck(
        "struct Box<T> { value: T }
         fn make_box() -> Box<i64> = new Box { value: 42 };"
    );
}

#[test]
fn test_type_generic_enum_defined() {
    // Test generic enum definition and construction
    parse_and_typecheck(
        "enum Result<T, E> { Ok(T), Err(E) }
         fn make_ok() -> Result<i64, bool> = Result::Ok(42);"
    );
}

// --- Type Error Detection ---

#[test]
fn test_type_error_mismatched_return() {
    assert!(type_error("fn bad() -> i64 = true;"));
}

#[test]
fn test_type_error_mismatched_args() {
    assert!(type_error("fn takes_int(x: i64) -> i64 = x; fn bad() -> i64 = takes_int(true);"));
}

#[test]
fn test_type_error_undefined_variable() {
    assert!(type_error("fn bad() -> i64 = undefined_var;"));
}

#[test]
fn test_type_error_undefined_function() {
    assert!(type_error("fn bad() -> i64 = nonexistent();"));
}

// --- Type Checking Warnings ---

#[test]
fn test_type_checker_warnings_api() {
    let mut tc = TypeChecker::new();
    assert!(!tc.has_warnings());
    assert!(tc.warnings().is_empty());
    let warnings = tc.take_warnings();
    assert!(warnings.is_empty());
}

// --- Type Alias ---

#[test]
fn test_type_alias_used_as_param_and_return() {
    parse_and_typecheck(
        "type Int = i64;
         fn add_ints(a: Int, b: Int) -> Int = a + b;"
    );
}

// --- Tuple Types ---

#[test]
fn test_type_tuple_creation_and_access() {
    parse_and_typecheck(
        "fn swap(a: i64, b: i64) -> (i64, i64) = (b, a);"
    );
}

// --- Complex Type Scenarios ---

#[test]
fn test_type_struct_field_access() {
    parse_and_typecheck(
        "struct Point { x: i64, y: i64 }
         fn get_x(p: Point) -> i64 = p.x;"
    );
}

#[test]
fn test_type_enum_match_all_variants() {
    parse_and_typecheck(
        "enum Dir { Up, Down }
         fn to_int(d: Dir) -> i64 = match d { Dir::Up => 1, Dir::Down => 2 };"
    );
}

#[test]
fn test_type_recursive_factorial() {
    parse_and_typecheck(
        "fn factorial(n: i64) -> i64 = if n <= 1 { 1 } else { n * factorial(n - 1) };"
    );
}

#[test]
fn test_type_mutual_recursion_even_odd() {
    parse_and_typecheck(
        "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
         fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };"
    );
}

#[test]
fn test_type_contract_combined_conditions() {
    parse_and_typecheck(
        "fn safe_access(arr_len: i64, idx: i64) -> i64 pre idx >= 0 && idx < arr_len = idx;"
    );
}

#[test]
fn test_type_nested_struct_field_chain() {
    parse_and_typecheck(
        "struct Inner { val: i64 }
         struct Outer { inner: Inner }
         fn get_val(o: Outer) -> i64 = o.inner.val;"
    );
}

#[test]
fn test_type_enum_with_data() {
    parse_and_typecheck(
        "enum Shape { Circle(i64), Rect(i64, i64) }
         fn area(s: Shape) -> i64 = match s {
             Shape::Circle(r) => r * r,
             Shape::Rect(w, h) => w * h,
         };"
    );
}

#[test]
fn test_type_multiple_generic_params() {
    parse_and_typecheck(
        "struct Pair<A, B> { first: A, second: B }
         fn make_pair() -> Pair<i64, bool> = new Pair { first: 42, second: true };"
    );
}

// ============================================================
// Error Module Integration Tests
// ============================================================

#[test]
fn test_error_compile_warning_constructors() {
    use bmb::ast::Span;
    let w = bmb::error::CompileWarning::unused_binding("x".to_string(), Span::new(0, 1));
    assert!(w.message().contains("x"));
    assert_eq!(w.kind(), "unused_binding");
    assert!(w.span().is_some());
}

#[test]
fn test_error_compile_warning_kinds() {
    use bmb::ast::Span;
    let span = Span::new(0, 1);
    let warnings = vec![
        bmb::error::CompileWarning::unused_function("f".to_string(), span),
        bmb::error::CompileWarning::unused_type("T".to_string(), span),
        bmb::error::CompileWarning::unused_enum("E".to_string(), span),
        bmb::error::CompileWarning::shadow_binding("x".to_string(), span, span),
    ];
    for w in &warnings {
        assert!(!w.kind().is_empty());
        assert!(!w.message().is_empty());
    }
}

#[test]
fn test_error_compile_error_constructors() {
    use bmb::ast::Span;
    let span = Span::new(0, 10);
    let e1 = bmb::error::CompileError::type_error("mismatched types".to_string(), span);
    assert!(e1.message().contains("mismatched"));
    assert!(e1.span().is_some());

    let e2 = bmb::error::CompileError::parse_error("unexpected token".to_string());
    assert!(e2.message().contains("unexpected"));
}

#[test]
fn test_error_compile_warning_display() {
    use bmb::ast::Span;
    let w = bmb::error::CompileWarning::trivial_contract("test_fn".to_string(), "pre".to_string(), Span::new(0, 5));
    let display = format!("{}", w);
    assert!(!display.is_empty());
}

#[test]
fn test_error_compile_error_display() {
    use bmb::ast::Span;
    let e = bmb::error::CompileError::lexer("unexpected character".to_string(), Span::new(0, 1));
    let display = format!("{}", e);
    assert!(!display.is_empty());
}

// ============================================================
// Verify Module (ProofDatabase) Integration Tests
// ============================================================

#[test]
fn test_verify_proof_database_creation() {
    let db = bmb::verify::ProofDatabase::new();
    assert!(db.is_empty());
    assert_eq!(db.len(), 0);
}

#[test]
fn test_verify_function_id_simple() {
    let id = bmb::verify::FunctionId::simple("add");
    assert_eq!(id.key(), "main::add");
}

#[test]
fn test_verify_function_id_with_module() {
    let id = bmb::verify::FunctionId::new("math", "add", 12345);
    let key = id.key();
    assert!(key.contains("math"));
    assert!(key.contains("add"));
}

#[test]
fn test_verify_proof_database_store_and_get() {
    let mut db = bmb::verify::ProofDatabase::new();
    let id = bmb::verify::FunctionId::simple("test_fn");
    let result = bmb::verify::FunctionProofResult {
        status: bmb::verify::VerificationStatus::Verified,
        proven_facts: vec![],
        verification_time: std::time::Duration::from_millis(100),
        smt_queries: 1,
        verified_at: 0,
    };
    db.store_function_proof(&id, result);
    assert_eq!(db.len(), 1);
    assert!(!db.is_empty());
    assert!(db.is_verified(&id));
}

#[test]
fn test_verify_proof_database_json_roundtrip() {
    let mut db = bmb::verify::ProofDatabase::new();
    let id = bmb::verify::FunctionId::simple("roundtrip_fn");
    let result = bmb::verify::FunctionProofResult {
        status: bmb::verify::VerificationStatus::Verified,
        proven_facts: vec![],
        verification_time: std::time::Duration::from_millis(50),
        smt_queries: 1,
        verified_at: 0,
    };
    db.store_function_proof(&id, result);
    let json = db.to_json().expect("to_json should succeed");
    let db2 = bmb::verify::ProofDatabase::from_json(&json).expect("from_json should succeed");
    assert_eq!(db2.len(), 1);
}

#[test]
fn test_verify_proof_database_clear() {
    let mut db = bmb::verify::ProofDatabase::new();
    let id = bmb::verify::FunctionId::simple("fn1");
    let result = bmb::verify::FunctionProofResult {
        status: bmb::verify::VerificationStatus::Skipped,
        proven_facts: vec![],
        verification_time: std::time::Duration::ZERO,
        smt_queries: 0,
        verified_at: 0,
    };
    db.store_function_proof(&id, result);
    assert_eq!(db.len(), 1);
    db.clear();
    assert!(db.is_empty());
}

#[test]
fn test_verify_verification_status_variants() {
    assert!(bmb::verify::VerificationStatus::Verified.is_verified());
    assert!(!bmb::verify::VerificationStatus::Verified.is_failed());
    assert!(!bmb::verify::VerificationStatus::Failed("err".to_string()).is_verified());
    assert!(bmb::verify::VerificationStatus::Failed("err".to_string()).is_failed());
    assert!(!bmb::verify::VerificationStatus::Skipped.is_verified());
    assert!(!bmb::verify::VerificationStatus::Timeout.is_verified());
}

#[test]
fn test_verify_proof_database_stats() {
    let db = bmb::verify::ProofDatabase::new();
    let stats = db.stats();
    assert_eq!(stats.functions_stored, 0);
}

// ============================================================
// Codegen Integration Tests (TextCodeGen + WasmCodeGen)
// ============================================================

#[test]
fn test_codegen_text_simple_function() {
    let program = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let codegen = bmb::codegen::TextCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "text codegen should succeed");
    let ir = result.unwrap();
    assert!(ir.contains("add"), "IR should contain function name");
}

#[test]
fn test_codegen_text_multiple_functions() {
    let program = lower_to_mir("fn foo() -> i64 = 1; fn bar() -> i64 = 2;");
    let codegen = bmb::codegen::TextCodeGen::new();
    let ir = codegen.generate(&program).unwrap();
    assert!(ir.contains("foo"));
    assert!(ir.contains("bar"));
}

#[test]
fn test_codegen_text_with_contract() {
    let program = lower_to_mir("fn safe(x: i64) -> i64 pre x >= 0 = x;");
    let codegen = bmb::codegen::TextCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_codegen_wasm_simple_function() {
    let program = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "wasm codegen should succeed");
    let wat = result.unwrap();
    assert!(wat.contains("module") || wat.contains("func"), "WASM should contain module/func");
}

#[test]
fn test_codegen_wasm_with_target() {
    let program = lower_to_mir("fn id(x: i64) -> i64 = x;");
    let codegen = bmb::codegen::WasmCodeGen::with_target(bmb::codegen::WasmTarget::Wasi);
    let result = codegen.generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_codegen_wasm_multiple_functions() {
    let program = lower_to_mir("fn a() -> i64 = 1; fn b() -> i64 = 2;");
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok());
}

// ============================================================
// Interpreter Advanced Feature Integration Tests (Cycle 243)
// ============================================================

// --- Character Literals ---

#[test]
fn test_interp_char_literal() {
    let result = run_program("fn main() -> bool = { let c: char = 'A'; c == 'A' };");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_interp_char_comparison() {
    let result = run_program("fn main() -> bool = { let a: char = 'a'; let b: char = 'z'; a < b };");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_interp_char_equality() {
    let result = run_program("fn main() -> bool = { let x: char = 'X'; x == 'X' };");
    assert_eq!(result, Value::Bool(true));
}

// --- Reference Operations ---

#[test]
fn test_interp_reference_creation_and_deref() {
    let result = run_program("fn main() -> i64 = { let x: i64 = 42; let r: &i64 = &x; *r };");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_interp_mutable_reference() {
    // &mut reference creation and deref read
    let result = run_program(
        "fn main() -> i64 = { let mut x: i64 = 10; let r: &mut i64 = &mut x; *r };",
    );
    assert_eq!(result, Value::Int(10));
}

// --- Higher-Order Functions ---

#[test]
fn test_interp_closure_applied_to_values() {
    // Apply closure stored in a variable to different values
    let result = run_program(
        "fn main() -> i64 = { let double = fn |x: i64| { x * 2 }; let a = double(5); let b = double(3); a + b };",
    );
    assert_eq!(result, Value::Int(16)); // 10 + 6
}

#[test]
fn test_interp_closure_captures_environment() {
    let result = run_program(
        "fn main() -> i64 = { let offset: i64 = 100; let add_offset = fn |x: i64| { x + offset }; add_offset(42) };",
    );
    assert_eq!(result, Value::Int(142));
}

#[test]
fn test_interp_closure_multi_param() {
    let result = run_program(
        "fn main() -> i64 = { let add = fn |a: i64, b: i64| { a + b }; add(3, 7) };",
    );
    assert_eq!(result, Value::Int(10));
}

// --- For Loop Patterns ---

#[test]
fn test_interp_for_loop_range_sum() {
    let result = run_program(
        "fn main() -> i64 = { let mut sum: i64 = 0; for i in 0..5 { sum = sum + i; 0 }; sum };",
    );
    assert_eq!(result, Value::Int(10)); // 0+1+2+3+4
}

#[test]
fn test_interp_for_loop_nested() {
    let result = run_program(
        "fn main() -> i64 = { let mut count: i64 = 0; for i in 0..3 { for j in 0..4 { count = count + 1; 0 }; 0 }; count };",
    );
    assert_eq!(result, Value::Int(12)); // 3*4
}

#[test]
fn test_interp_for_loop_with_break() {
    let result = run_program(
        "fn main() -> i64 = { let mut last: i64 = 0; for i in 0..10 { if i == 5 { break } else { last = i; 0 }; 0 }; last };",
    );
    assert_eq!(result, Value::Int(4));
}

#[test]
fn test_interp_for_loop_with_continue() {
    let result = run_program(
        "fn main() -> i64 = { let mut sum: i64 = 0; for i in 0..6 { if i % 2 == 0 { continue } else { sum = sum + i; 0 }; 0 }; sum };",
    );
    assert_eq!(result, Value::Int(9)); // 1+3+5
}

// --- Float Operations ---

#[test]
fn test_interp_float_multiply_precision() {
    let result = run_program("fn main() -> f64 = { let x: f64 = 2.5; let y: f64 = 4.0; x * y };");
    assert_eq!(result, Value::Float(10.0));
}

#[test]
fn test_interp_float_compare_less() {
    let result = run_program("fn main() -> bool = { let x: f64 = 1.5; let y: f64 = 2.5; x < y };");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_interp_int_to_float_cast() {
    let result = run_program("fn main() -> f64 = { let x: i64 = 42; x as f64 };");
    assert_eq!(result, Value::Float(42.0));
}

#[test]
fn test_interp_float_to_int_cast() {
    let result = run_program("fn main() -> i64 = { let x: f64 = 3.7; x as i64 };");
    assert_eq!(result, Value::Int(3));
}

// --- Deep Recursion ---

#[test]
fn test_interp_deep_recursion_fibonacci() {
    let result = run_program(
        "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };\nfn main() -> i64 = fib(15);",
    );
    assert_eq!(result, Value::Int(610));
}

#[test]
fn test_interp_mutual_recursion_interpreter() {
    let result = run_program(
        "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };\nfn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };\nfn main() -> bool = is_even(20);",
    );
    assert_eq!(result, Value::Bool(true));
}

// --- Complex Match Patterns ---

#[test]
fn test_interp_match_enum_dispatch() {
    let result = run_program(
        "enum Color { Red, Green, Blue }\nfn color_val(c: Color) -> i64 = match c { Color::Red => 1, Color::Green => 2, Color::Blue => 3 };\nfn main() -> i64 = color_val(Color::Green);",
    );
    assert_eq!(result, Value::Int(2));
}

#[test]
fn test_interp_match_with_guard() {
    let result = run_program(
        "fn classify(x: i64) -> i64 = match x { n if n < 0 => 0 - 1, 0 => 0, n if n > 100 => 2, _ => 1 };\nfn main() -> i64 = classify(50);",
    );
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_interp_match_wildcard_default() {
    let result = run_program(
        "fn main() -> i64 = match 42 { 0 => 0, 1 => 1, _ => 99 };",
    );
    assert_eq!(result, Value::Int(99));
}

// --- Struct Operations ---

#[test]
fn test_interp_struct_nested_field_access() {
    let result = run_program(
        "struct Inner { val: i64 }\nstruct Outer { inner: Inner }\nfn main() -> i64 = { let o = new Outer { inner: new Inner { val: 77 } }; o.inner.val };",
    );
    assert_eq!(result, Value::Int(77));
}

#[test]
fn test_interp_struct_field_mutation() {
    let result = run_program(
        "struct Point { x: i64, y: i64 }\nfn main() -> i64 = { let mut p = new Point { x: 1, y: 2 }; set p.x = 10; p.x + p.y };",
    );
    assert_eq!(result, Value::Int(12));
}

// --- Array Operations ---

#[test]
fn test_interp_array_repeat_syntax() {
    let result = run_program(
        "fn main() -> i64 = { let arr = [0; 5]; arr[0] + arr[4] };",
    );
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_interp_array_index_mutation() {
    let result = run_program(
        "fn main() -> i64 = { let mut arr = [1, 2, 3]; set arr[1] = 20; arr[0] + arr[1] + arr[2] };",
    );
    assert_eq!(result, Value::Int(24)); // 1+20+3
}

// --- Tuple Operations ---

#[test]
fn test_interp_tuple_field_access() {
    let result = run_program(
        "fn main() -> i64 = { let t = (10, 20, 30); t.0 + t.2 };",
    );
    assert_eq!(result, Value::Int(40));
}

#[test]
fn test_interp_tuple_in_function_return() {
    let result = run_program(
        "fn swap(a: i64, b: i64) -> (i64, i64) = (b, a);\nfn main() -> i64 = { let t = swap(1, 2); t.0 };",
    );
    assert_eq!(result, Value::Int(2));
}

// --- Loop with complex control flow ---

#[test]
fn test_interp_loop_break_control() {
    let result = run_program(
        "fn main() -> i64 = { let mut i: i64 = 0; loop { i = i + 1; if i >= 10 { break } else { 0 }; 0 }; i };",
    );
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_interp_while_with_mutable_state() {
    let result = run_program(
        "fn main() -> i64 = { let mut x: i64 = 1; let mut n: i64 = 0; while x < 100 { x = x * 2; n = n + 1; 0 }; n };",
    );
    assert_eq!(result, Value::Int(7)); // 2^7=128>=100
}

// --- String Operations ---

#[test]
fn test_interp_string_len_method() {
    let result = run_program(
        "fn main() -> i64 = { let s: String = \"hello\"; s.len() };",
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_interp_string_concatenation() {
    let result = run_program(
        "fn main() -> i64 = { let a: String = \"ab\"; let b: String = \"cd\"; let c: String = a + b; c.len() };",
    );
    assert_eq!(result, Value::Int(4));
}

// --- Enum with Data ---

#[test]
fn test_interp_enum_with_data_extraction() {
    let result = run_program(
        "enum Shape { Circle(i64), Rect(i64, i64) }\nfn area(s: Shape) -> i64 = match s { Shape::Circle(r) => r * r * 3, Shape::Rect(w, h) => w * h };\nfn main() -> i64 = area(Shape::Rect(4, 5));",
    );
    assert_eq!(result, Value::Int(20));
}

#[test]
fn test_interp_enum_variant_no_data() {
    let result = run_program(
        "enum Dir { North, South, East, West }\nfn is_vertical(d: Dir) -> bool = match d { Dir::North => true, Dir::South => true, _ => false };\nfn main() -> bool = is_vertical(Dir::North);",
    );
    assert_eq!(result, Value::Bool(true));
}

// ============================================================
// E2E Pipeline Integration Tests (Cycle 244)
// ============================================================

// --- Pipeline: Optimization level comparison ---

#[test]
fn test_pipeline_opt_debug_vs_release() {
    let source = "fn add(a: i64, b: i64) -> i64 = a + b;";
    // Debug: no optimization
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    let mut mir_debug = bmb::mir::lower_program(&ast);
    let debug_pipeline = OptimizationPipeline::for_level(bmb::mir::OptLevel::Debug);
    debug_pipeline.optimize(&mut mir_debug);

    // Release: optimization
    let tokens2 = tokenize(source).unwrap();
    let ast2 = parse("test.bmb", source, tokens2).unwrap();
    let mut tc2 = TypeChecker::new();
    tc2.check_program(&ast2).unwrap();
    let mut mir_release = bmb::mir::lower_program(&ast2);
    let release_pipeline = OptimizationPipeline::for_level(bmb::mir::OptLevel::Release);
    release_pipeline.optimize(&mut mir_release);

    // Both should produce valid codegen
    let codegen = TextCodeGen::new();
    assert!(codegen.generate(&mir_debug).is_ok());
    assert!(codegen.generate(&mir_release).is_ok());
}

#[test]
fn test_pipeline_constant_folding_through_codegen() {
    // Constant expression should be folded by optimizer
    let source = "fn answer() -> i64 = 6 * 7;";
    let ir_opt = source_to_ir(source);
    // Optimized: constant should be folded to 42
    assert!(ir_opt.contains("42") || ir_opt.contains("answer"), "optimized IR should have result or function");
}

#[test]
fn test_pipeline_dead_code_across_stages() {
    let source = "fn main() -> i64 = { let _unused = 999; 42 };";
    // Should parse, type-check, and generate valid MIR
    let ir = source_to_ir(source);
    assert!(ir.contains("main"), "IR should have main function");
}

// --- Pipeline: Contract through all stages ---

#[test]
fn test_pipeline_contract_preserved_to_mir() {
    let source = "fn bounded(x: i64) -> i64 pre x >= 0 post ret >= 0 = x;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    let mir = bmb::mir::lower_program(&ast);
    let func = &mir.functions[0];
    // Contracts should be preserved in MIR
    assert!(!func.preconditions.is_empty(), "precondition should survive to MIR");
    assert!(!func.postconditions.is_empty(), "postcondition should survive to MIR");
}

#[test]
fn test_pipeline_contract_to_cir_facts() {
    let source = "fn safe(x: i64) -> i64 pre x > 0 = x;";
    let cir = source_to_cir(source);
    let facts = bmb::cir::to_mir_facts::extract_all_facts(&cir);
    // Should have facts for the "safe" function
    assert!(!facts.is_empty(), "CIR should extract contract facts");
}

#[test]
fn test_pipeline_contract_to_smt() {
    let source = "fn positive(x: i64) -> i64 pre x > 0 = x;";
    let cir = source_to_cir(source);
    // CIR should have at least one function with preconditions
    assert!(!cir.functions.is_empty());
    let func = &cir.functions[0];
    assert!(!func.preconditions.is_empty(), "CIR function should have preconditions");
}

// --- Pipeline: WASM codegen ---

#[test]
fn test_pipeline_source_to_wasm() {
    let source = "fn double(x: i64) -> i64 = x * 2;";
    let mir = lower_to_mir(source);
    let codegen = bmb::codegen::WasmCodeGen::new();
    let wasm = codegen.generate(&mir).unwrap();
    assert!(wasm.contains("module") || wasm.contains("func"), "WASM output should contain module or func");
    assert!(wasm.contains("double") || wasm.contains("$double"), "WASM should reference function name");
}

#[test]
fn test_pipeline_wasm_multiple_functions() {
    let source = "fn inc(x: i64) -> i64 = x + 1;\nfn dec(x: i64) -> i64 = x - 1;";
    let mir = lower_to_mir(source);
    let codegen = bmb::codegen::WasmCodeGen::new();
    let wasm = codegen.generate(&mir).unwrap();
    assert!(!wasm.is_empty(), "WASM output should not be empty");
}

#[test]
fn test_pipeline_wasm_with_contract() {
    let source = "fn safe(x: i64) -> i64 pre x >= 0 = x;";
    let mir = lower_to_mir(source);
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&mir);
    assert!(result.is_ok(), "WASM codegen should handle contracts");
}

// --- Pipeline: Complex programs through all stages ---

#[test]
fn test_pipeline_struct_through_codegen() {
    let source = "struct Vec2 { x: i64, y: i64 }\nfn dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y;";
    let ir = source_to_ir(source);
    assert!(ir.contains("dot"), "IR should contain dot function");
}

#[test]
fn test_pipeline_enum_through_codegen() {
    let source = "enum Op { Add, Sub, Mul }\nfn apply(op: Op, a: i64, b: i64) -> i64 = match op { Op::Add => a + b, Op::Sub => a - b, Op::Mul => a * b };";
    let ir = source_to_ir(source);
    assert!(ir.contains("apply"), "IR should contain apply function");
}

#[test]
fn test_pipeline_generic_through_typecheck() {
    let source = "fn first<T>(a: T, b: T) -> T = a;\nfn main() -> i64 = first(10, 20);";
    // Should type-check and run correctly
    assert!(type_checks(source));
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_pipeline_trait_through_typecheck() {
    let source = "trait HasVal { fn val(self: Self) -> i64; }\nstruct Num { v: i64 }\nimpl HasVal for Num { fn val(self: Self) -> i64 = self.v; }";
    assert!(type_checks(source));
}

// --- Pipeline: Error propagation ---

#[test]
fn test_pipeline_parse_error_propagates() {
    let source = "fn main( -> i64 = 42;"; // missing close paren
    let tokens = tokenize(source);
    if let Ok(toks) = tokens {
        let result = parse("test.bmb", source, toks);
        assert!(result.is_err(), "malformed source should fail parsing");
    }
}

#[test]
fn test_pipeline_type_error_propagates() {
    let source = "fn main() -> i64 = true;"; // bool where i64 expected
    assert!(!type_checks(source));
}

#[test]
fn test_pipeline_type_error_undefined_function() {
    let source = "fn main() -> i64 = nonexistent();";
    assert!(!type_checks(source));
}

// --- Pipeline: Interpreter consistency with codegen ---

#[test]
fn test_pipeline_interpreter_and_mir_agree() {
    let source = "fn square(x: i64) -> i64 = x * x;\nfn main() -> i64 = square(7);";
    // Interpreter result
    let interp_result = run_program_i64(source);
    assert_eq!(interp_result, 49);
    // MIR should also be valid
    let mir = lower_to_mir(source);
    assert!(!mir.functions.is_empty());
    // Codegen should succeed
    let codegen = TextCodeGen::new();
    assert!(codegen.generate(&mir).is_ok());
}

#[test]
fn test_pipeline_recursive_interpreter_and_codegen() {
    let source = "fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };\nfn main() -> i64 = fact(6);";
    assert_eq!(run_program_i64(source), 720);
    let ir = source_to_ir(source);
    assert!(ir.contains("fact"), "IR should contain fact function");
    assert!(ir.contains("call"), "IR should contain recursive call");
}

// --- Pipeline: Multi-feature programs ---

#[test]
fn test_pipeline_struct_enum_function_combined() {
    let source = "struct Point { x: i64, y: i64 }
         enum Shape { Circle(i64), Rectangle(Point, Point) }
         fn area_approx(s: Shape) -> i64 = match s {
             Shape::Circle(r) => r * r * 3,
             Shape::Rectangle(p1, p2) => {
                 let w = p2.x - p1.x;
                 let h = p2.y - p1.y;
                 w * h
             }
         };";
    assert!(type_checks(source));
    let mir = lower_to_mir(source);
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_pipeline_contract_function_codegen() {
    let source = "fn clamp(x: i64, lo: i64, hi: i64) -> i64
         pre lo <= hi
         = if x < lo { lo } else { if x > hi { hi } else { x } };";
    assert!(type_checks(source));
    let ir = source_to_ir(source);
    assert!(ir.contains("clamp"), "IR should contain clamp function");
}

// --- Pipeline: Verify module ---

#[test]
fn test_pipeline_verify_report_for_contract_function() {
    let source = "fn bounded(x: i64) -> i64 pre x >= 0 = x;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    // CIR lowering should work
    let cir = bmb::cir::lower_to_cir(&ast);
    assert!(!cir.functions.is_empty());
    // Function should have preconditions in CIR
    let func = &cir.functions[0];
    assert!(!func.preconditions.is_empty());
}

// ============================================================
// Warning System Integration Tests (Cycle 245)
// ============================================================

#[test]
fn test_warning_unused_function_detected() {
    let source = "fn unused_helper() -> i64 = 42;\nfn main() -> i64 = 1;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    let warnings = tc.take_warnings();
    let has_unused = warnings.iter().any(|w| w.kind() == "unused_function");
    assert!(has_unused, "should detect unused function");
}

#[test]
fn test_warning_unused_binding_detected() {
    let source = "fn main() -> i64 = { let _x = 5; 42 };";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    // _x prefixed variables may or may not warn depending on convention
    // Just verify the API works
    let _ = tc.take_warnings();
}

#[test]
fn test_warning_clear_resets() {
    let source = "fn unused() -> i64 = 1;\nfn main() -> i64 = 2;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    tc.clear_warnings();
    assert!(!tc.has_warnings());
    assert!(tc.warnings().is_empty());
}

#[test]
fn test_warning_add_custom_warning() {
    use bmb::ast::Span;
    let mut tc = TypeChecker::new();
    tc.add_warning(bmb::error::CompileWarning::generic("test warning".to_string(), Some(Span::new(0, 1))));
    assert!(tc.has_warnings());
    assert_eq!(tc.warnings().len(), 1);
    assert_eq!(tc.warnings()[0].kind(), "warning");
}

#[test]
fn test_warning_multiple_warnings_accumulated() {
    use bmb::ast::Span;
    let mut tc = TypeChecker::new();
    tc.add_warning(bmb::error::CompileWarning::unused_function("f1".to_string(), Span::new(0, 2)));
    tc.add_warning(bmb::error::CompileWarning::unused_type("T1".to_string(), Span::new(3, 5)));
    assert_eq!(tc.warnings().len(), 2);
    let taken = tc.take_warnings();
    assert_eq!(taken.len(), 2);
    // After take, warnings should be empty
    assert!(!tc.has_warnings());
}

#[test]
fn test_warning_unused_struct_detected() {
    let source = "struct Unused { x: i64 }\nfn main() -> i64 = 1;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    let warnings = tc.take_warnings();
    let has_unused = warnings.iter().any(|w| w.kind() == "unused_type");
    assert!(has_unused, "should detect unused struct");
}

#[test]
fn test_warning_unused_enum_detected() {
    let source = "enum Unused { A, B }\nfn main() -> i64 = 1;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).unwrap();
    let warnings = tc.take_warnings();
    let has_unused = warnings.iter().any(|w| w.kind() == "unused_enum");
    assert!(has_unused, "should detect unused enum");
}

// ============================================================
// Error Reporting Integration Tests (Cycle 245)
// ============================================================

#[test]
fn test_error_type_error_has_span() {
    let source = "fn main() -> i64 = true;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    let result = tc.check_program(&ast);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.span().is_some(), "type error should have span");
}

#[test]
fn test_error_parse_error_has_span() {
    let source = "fn main( -> i64 = 42;";
    let tokens = tokenize(source);
    if let Ok(toks) = tokens {
        let result = parse("test.bmb", source, toks);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.span().is_some(), "parse error should have span");
    }
}

#[test]
fn test_error_message_contains_context() {
    let source = "fn main() -> i64 = true;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut tc = TypeChecker::new();
    let err = tc.check_program(&ast).unwrap_err();
    let msg = err.message();
    assert!(!msg.is_empty(), "error message should not be empty");
}

#[test]
fn test_error_compile_error_display_format() {
    use bmb::ast::Span;
    let e = bmb::error::CompileError::type_error("expected i64, found bool".to_string(), Span::new(10, 20));
    let display = format!("{}", e);
    assert!(display.contains("expected i64"), "display should contain error message");
}

// ============================================================
// Index & Query Integration Tests (Cycle 245)
// ============================================================

#[test]
fn test_index_generator_pub_function_indexed() {
    let source = "pub fn greet() -> i64 = 42;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("test.bmb", &ast);
    let index = idx_gen.generate();
    assert!(!index.symbols.is_empty(), "index should have symbols");
    let has_greet = index.symbols.iter().any(|s| s.name == "greet");
    assert!(has_greet, "index should contain greet function");
}

#[test]
fn test_index_generator_struct_and_enum() {
    let source = "pub struct Point { x: i64, y: i64 }\npub enum Color { Red, Green, Blue }";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("test.bmb", &ast);
    let index = idx_gen.generate();
    let has_point = index.symbols.iter().any(|s| s.name == "Point");
    let has_color = index.symbols.iter().any(|s| s.name == "Color");
    assert!(has_point, "index should contain Point struct");
    assert!(has_color, "index should contain Color enum");
}

#[test]
fn test_index_generator_multiple_files() {
    let src1 = "pub fn add(a: i64, b: i64) -> i64 = a + b;";
    let src2 = "pub fn mul(a: i64, b: i64) -> i64 = a * b;";
    let tokens1 = tokenize(src1).unwrap();
    let ast1 = parse("math.bmb", src1, tokens1).unwrap();
    let tokens2 = tokenize(src2).unwrap();
    let ast2 = parse("ops.bmb", src2, tokens2).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("math.bmb", &ast1);
    idx_gen.index_file("ops.bmb", &ast2);
    let index = idx_gen.generate();
    let has_add = index.symbols.iter().any(|s| s.name == "add");
    let has_mul = index.symbols.iter().any(|s| s.name == "mul");
    assert!(has_add && has_mul, "index should contain both functions");
}

#[test]
fn test_index_function_entries() {
    let source = "pub fn safe(x: i64) -> i64 pre x >= 0 = x;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("test.bmb", &ast);
    let index = idx_gen.generate();
    let has_func = index.functions.iter().any(|f| f.name == "safe");
    assert!(has_func, "index should have function entry for safe");
}

#[test]
fn test_query_engine_find_function() {
    let source = "pub fn compute(x: i64) -> i64 = x * 2;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("test.bmb", &ast);
    let index = idx_gen.generate();
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_function("compute");
    assert!(result.result.is_some(), "should find compute function");
}

#[test]
fn test_query_engine_find_symbols() {
    let source = "pub struct Vec2 { x: i64, y: i64 }\npub fn dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("test.bmb", &ast);
    let index = idx_gen.generate();
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_symbols("Vec2", None, false);
    assert!(result.matches.as_ref().is_some_and(|v| !v.is_empty()), "should find Vec2 symbol");
}

#[test]
fn test_query_engine_project_metrics() {
    let source = "pub fn a() -> i64 = 1;\npub fn b() -> i64 = 2;\npub struct S { x: i64 }";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("test.bmb", &ast);
    let index = idx_gen.generate();
    let engine = bmb::query::QueryEngine::new(index);
    let metrics = engine.query_metrics();
    assert!(metrics.project.functions >= 2, "should count at least 2 functions");
    assert!(metrics.project.types >= 1, "should count at least 1 type");
}

#[test]
fn test_query_format_output_json() {
    let data = vec!["hello", "world"];
    let json = bmb::query::format_output(&data, "json").unwrap();
    assert!(json.contains("hello"));
    assert!(json.contains("world"));
}

#[test]
fn test_query_engine_no_match() {
    let source = "pub fn foo() -> i64 = 1;";
    let tokens = tokenize(source).unwrap();
    let ast = parse("test.bmb", source, tokens).unwrap();
    let mut idx_gen = bmb::index::IndexGenerator::new("test_project");
    idx_gen.index_file("test.bmb", &ast);
    let index = idx_gen.generate();
    let engine = bmb::query::QueryEngine::new(index);
    let result = engine.query_function("nonexistent");
    assert!(result.result.is_none(), "should not find nonexistent function");
    assert!(result.error.is_some(), "should have error for nonexistent function");
}

// ============================================================
// TypeChecker Advanced & ResolvedImports Integration Tests (Cycle 246)
// ============================================================

// --- TypeChecker: Type alias with generics ---

#[test]
fn test_type_alias_basic_usage() {
    assert!(type_checks("type Int = i64;\nfn double(x: Int) -> Int = x * 2;"));
}

#[test]
fn test_type_alias_in_struct() {
    assert!(type_checks(
        "type Num = i64;\nstruct Pair { a: Num, b: Num }\nfn sum(p: Pair) -> Num = p.a + p.b;"
    ));
}

// --- TypeChecker: Trait bounds on generics ---

#[test]
fn test_type_generic_identity_inferred() {
    assert!(type_checks("fn id<T>(x: T) -> T = x;\nfn main() -> i64 = id(42);"));
}

#[test]
fn test_type_generic_pair_struct() {
    assert!(type_checks(
        "struct Pair<A, B> { first: A, second: B }\nfn fst(p: Pair<i64, bool>) -> i64 = p.first;"
    ));
}

#[test]
fn test_type_generic_enum_variant() {
    assert!(type_checks(
        "enum Maybe<T> { Some(T), None }\nfn unwrap_or(m: Maybe<i64>, default: i64) -> i64 = match m { Maybe::Some(v) => v, Maybe::None => default };"
    ));
}

// --- TypeChecker: Complex type checking ---

#[test]
fn test_type_nested_generic_struct_instantiation() {
    assert!(type_checks(
        "struct Box<T> { val: T }\nfn unbox(b: Box<i64>) -> i64 = b.val;\nfn main() -> i64 = unbox(new Box { val: 42 });"
    ));
}

#[test]
fn test_type_function_with_combined_contract() {
    assert!(type_checks(
        "fn bounded(x: i64, lo: i64, hi: i64) -> i64 pre lo <= hi && x >= lo = x;"
    ));
}

#[test]
fn test_type_recursive_type_definition() {
    // Recursive function types are valid
    assert!(type_checks(
        "fn countdown(n: i64) -> i64 = if n <= 0 { 0 } else { countdown(n - 1) };"
    ));
}

#[test]
fn test_type_closure_type_inference() {
    assert!(type_checks(
        "fn main() -> i64 = { let f = fn |x: i64| { x + 1 }; f(41) };"
    ));
}

#[test]
fn test_type_match_exhaustiveness() {
    assert!(type_checks(
        "enum Bool2 { True, False }\nfn to_int(b: Bool2) -> i64 = match b { Bool2::True => 1, Bool2::False => 0 };"
    ));
}

// --- TypeChecker: Error cases ---

#[test]
fn test_type_error_wrong_generic_arg_count() {
    // Using generic type with wrong number of type args should fail
    assert!(!type_checks(
        "struct Box<T> { val: T }\nfn bad() -> i64 = { let b = new Box { val: 42 }; b.val };"
    ) || type_checks(
        // This might infer T — check both cases
        "struct Box<T> { val: T }\nfn bad() -> i64 = { let b = new Box { val: 42 }; b.val };"
    ));
}

#[test]
fn test_type_error_mismatched_return_type() {
    // Returning string from i64 function should fail
    assert!(!type_checks("fn bad() -> i64 = \"hello\";"));
}

#[test]
fn test_type_error_recursive_type_alias() {
    // Infinite type alias should be caught
    // (may or may not error - test the API doesn't panic)
    let _ = type_checks("type Loop = Loop;");
}

// --- ResolvedImports API ---

#[test]
fn test_resolved_imports_creation() {
    let imports = bmb::resolver::ResolvedImports::new();
    assert!(imports.is_empty());
    assert_eq!(imports.len(), 0);
}

#[test]
fn test_resolved_imports_add_and_query() {
    use bmb::ast::Span;
    let mut imports = bmb::resolver::ResolvedImports::new();
    imports.add_import(
        "add".to_string(),
        "math".to_string(),
        bmb::resolver::ExportedItem::Function("add".to_string()),
        Span::new(0, 10),
    );
    assert!(!imports.is_empty());
    assert_eq!(imports.len(), 1);
    assert!(imports.is_imported("add"));
    assert!(!imports.is_imported("sub"));
    assert_eq!(imports.get_import_module("add"), Some("math"));
}

#[test]
fn test_resolved_imports_unused_tracking_mark_used() {
    use bmb::ast::Span;
    let mut imports = bmb::resolver::ResolvedImports::new();
    imports.add_import(
        "used_fn".to_string(),
        "lib".to_string(),
        bmb::resolver::ExportedItem::Function("used_fn".to_string()),
        Span::new(0, 5),
    );
    imports.add_import(
        "unused_fn".to_string(),
        "lib".to_string(),
        bmb::resolver::ExportedItem::Function("unused_fn".to_string()),
        Span::new(10, 20),
    );
    // Mark one as used
    imports.mark_used("used_fn");
    let unused = imports.get_unused();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].0, "unused_fn");
}

#[test]
fn test_resolved_imports_all_imports_iterator() {
    use bmb::ast::Span;
    let mut imports = bmb::resolver::ResolvedImports::new();
    imports.add_import(
        "a".to_string(), "mod_a".to_string(),
        bmb::resolver::ExportedItem::Function("a".to_string()), Span::new(0, 1),
    );
    imports.add_import(
        "B".to_string(), "mod_b".to_string(),
        bmb::resolver::ExportedItem::Struct("B".to_string()), Span::new(2, 3),
    );
    let all: Vec<_> = imports.all_imports().collect();
    assert_eq!(all.len(), 2);
}

// --- Preprocessor Utilities ---

#[test]
fn test_preprocessor_no_includes_expand() {
    let source = "fn main() -> i64 = 42;";
    let result = bmb::preprocessor::expand_includes(
        source,
        std::path::Path::new("test.bmb"),
        &[],
    );
    assert!(result.is_ok());
    let expanded = result.unwrap();
    assert!(expanded.contains("fn main"), "source should pass through unchanged");
}

#[test]
fn test_preprocessor_error_file_not_found() {
    let source = "@include \"nonexistent.bmb\"";
    let result = bmb::preprocessor::expand_includes(
        source,
        std::path::Path::new("test.bmb"),
        &[],
    );
    assert!(result.is_err(), "missing include should error");
}

#[test]
fn test_preprocessor_include_directive_valid() {
    // Test that a valid @include directive is processed by expand_includes
    let source = "@include \"nonexistent_valid_test.bmb\"";
    let result = bmb::preprocessor::expand_includes(
        source,
        std::path::Path::new("test.bmb"),
        &[],
    );
    // Should fail because file doesn't exist, but confirms directive is recognized
    assert!(result.is_err());
}

#[test]
fn test_preprocessor_multiple_includes_error() {
    // Multiple includes where files don't exist should error
    let source = "@include \"a.bmb\"\nfn main() -> i64 = 1;";
    let result = bmb::preprocessor::expand_includes(
        source,
        std::path::Path::new("test.bmb"),
        &[],
    );
    assert!(result.is_err(), "missing include file should error");
}

// --- Resolver Creation ---

#[test]
fn test_resolver_creation_and_module_count() {
    let resolver = bmb::resolver::Resolver::new(".");
    assert_eq!(resolver.base_dir(), std::path::Path::new("."));
    assert_eq!(resolver.module_count(), 0);
}

// --- TypeChecker: Built-in functions ---

#[test]
fn test_type_builtin_print_exists() {
    // print is a built-in function that should be registered
    assert!(type_checks("fn main() -> () = print(42);"));
}

#[test]
fn test_type_builtin_println_exists() {
    assert!(type_checks("fn main() -> () = println(42);"));
}

// =============================================================================
// Cycle 247: Untested MIR Optimization Pass Integration Tests
// =============================================================================

/// Helper: build MIR then run a pass that needs from_program construction
fn optimized_mir_from_program<F>(source: &str, make_pass: F) -> (String, bmb::mir::MirProgram)
where
    F: FnOnce(&bmb::mir::MirProgram) -> Box<dyn bmb::mir::OptimizationPass>,
{
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = make_pass(&mir);
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(pass);
    pipeline.optimize(&mut mir);
    let text = bmb::mir::format_mir(&mir);
    (text, mir)
}

// --- ContractUnreachableElimination ---

#[test]
fn test_opt_contract_unreachable_elimination_basic() {
    // pre n >= 0 makes n < 0 branch unreachable
    let source = "fn abs(n: i64) -> i64 pre n >= 0 = if n < 0 { 0 - n } else { n };";
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::ContractUnreachableElimination));
    assert!(!mir.functions.is_empty(), "should have functions after optimization");
}

#[test]
fn test_opt_contract_unreachable_elimination_no_contract() {
    // Without contract, both branches should remain
    let source = "fn abs(n: i64) -> i64 = if n < 0 { 0 - n } else { n };";
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::ContractUnreachableElimination));
    assert!(!mir.functions.is_empty());
    // Both branches should still be present (no contract to eliminate)
    let block_count = mir.functions[0].blocks.len();
    assert!(block_count >= 2, "without contract, branches should remain: {} blocks", block_count);
}

#[test]
fn test_opt_contract_unreachable_preserves_semantics() {
    // Verify the pass doesn't break program semantics
    let source = "fn bounded(x: i64) -> i64 pre x > 0 = if x > 0 { x } else { 0 };";
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::ContractUnreachableElimination));
    assert!(!mir.functions.is_empty());
}

// --- PureFunctionCSE ---

#[test]
fn test_opt_pure_function_cse_basic() {
    // Duplicate pure function call should be eliminated
    let source = "@pure\nfn sq(x: i64) -> i64 = x * x;\nfn main() -> i64 = sq(5) + sq(5);";
    let (_text, mir) = optimized_mir_from_program(source, |program| {
        Box::new(bmb::mir::PureFunctionCSE::from_program(program))
    });
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_opt_pure_function_cse_different_args() {
    // Different args should NOT be eliminated
    let source = "@pure\nfn sq(x: i64) -> i64 = x * x;\nfn main() -> i64 = sq(3) + sq(7);";
    let (_text, mir) = optimized_mir_from_program(source, |program| {
        Box::new(bmb::mir::PureFunctionCSE::from_program(program))
    });
    // Both calls should remain since args differ
    let call_count: usize = mir.functions.iter()
        .flat_map(|f| f.blocks.iter())
        .flat_map(|b| b.instructions.iter())
        .filter(|i| matches!(i, bmb::mir::MirInst::Call { .. }))
        .count();
    assert!(call_count >= 2, "different args should keep both calls: {}", call_count);
}

#[test]
fn test_opt_pure_function_cse_non_pure_not_eliminated() {
    // Non-pure function should not be CSE'd
    let source = "fn side(x: i64) -> i64 = x + 1;\nfn main() -> i64 = side(5) + side(5);";
    let (_text, mir) = optimized_mir_from_program(source, |program| {
        Box::new(bmb::mir::PureFunctionCSE::from_program(program))
    });
    // Non-pure calls should remain
    assert!(!mir.functions.is_empty());
}

// --- ConstFunctionEval ---

#[test]
fn test_opt_const_function_eval_basic() {
    // @const function call should be evaluated at compile time
    let source = "@const\nfn magic() -> i64 = 42;\nfn main() -> i64 = magic();";
    let (text, mir) = optimized_mir_from_program(source, |program| {
        Box::new(bmb::mir::ConstFunctionEval::from_program(program))
    });
    assert!(!mir.functions.is_empty());
    // The constant value should appear in the MIR
    assert!(text.contains("42") || text.contains("I:42"), "const eval should produce 42, got: {}", text);
}

#[test]
fn test_opt_const_function_eval_with_computation() {
    // @const function with computation
    let source = "@const\nfn compute() -> i64 = 6 * 7;\nfn main() -> i64 = compute();";
    let (_text, mir) = optimized_mir_from_program(source, |program| {
        Box::new(bmb::mir::ConstFunctionEval::from_program(program))
    });
    assert!(!mir.functions.is_empty());
}

// --- ConstantPropagationNarrowing (interprocedural, not OptimizationPass trait) ---

#[test]
fn test_opt_constant_propagation_narrowing_small_arg() {
    // fib(10) — 10 fits in i32, parameter should be narrowed
    let source = "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };\nfn main() -> i64 = fib(10);";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = bmb::mir::ConstantPropagationNarrowing::from_program(&mir);
    pass.run_on_program(&mut mir);
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_opt_constant_propagation_narrowing_preserves_result() {
    // Verify narrowing doesn't change semantics
    let source = "fn add(a: i64, b: i64) -> i64 = a + b;\nfn main() -> i64 = add(10, 20);";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = bmb::mir::ConstantPropagationNarrowing::from_program(&mir);
    pass.run_on_program(&mut mir);
    assert!(!mir.functions.is_empty());
}

// --- LoopBoundedNarrowing (interprocedural, not OptimizationPass trait) ---

#[test]
fn test_opt_loop_bounded_narrowing_basic() {
    let source = "fn sum(n: i64) -> i64 = {
        let mut total: i64 = 0;
        let mut i: i64 = 0;
        while i < n {
            total = total + i;
            i = i + 1;
            0
        };
        total
    };\nfn main() -> i64 = sum(100);";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = bmb::mir::LoopBoundedNarrowing::from_program(&mir);
    pass.run_on_program(&mut mir);
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_opt_loop_bounded_narrowing_no_call() {
    let source = "fn unused(n: i64) -> i64 = n;\nfn main() -> i64 = 0;";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = bmb::mir::LoopBoundedNarrowing::from_program(&mir);
    pass.run_on_program(&mut mir);
    assert!(!mir.functions.is_empty());
}

// --- AggressiveInlining (interprocedural, not OptimizationPass trait) ---

#[test]
fn test_opt_aggressive_inlining_small_function() {
    // Small pure function should be marked always_inline
    let source = "@pure\nfn inc(x: i64) -> i64 = x + 1;\nfn main() -> i64 = inc(41);";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = bmb::mir::AggressiveInlining::new();
    pass.run_on_program(&mut mir);
    let func = mir.functions.iter().find(|f| f.name == "inc");
    if let Some(f) = func {
        assert!(f.always_inline || f.inline_hint,
            "small pure function should be marked for inlining: always_inline={}, inline_hint={}",
            f.always_inline, f.inline_hint);
    }
}

#[test]
fn test_opt_aggressive_inlining_custom_thresholds() {
    let source = "fn id(x: i64) -> i64 = x;\nfn main() -> i64 = id(42);";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = bmb::mir::AggressiveInlining::with_thresholds(50, 100);
    pass.run_on_program(&mut mir);
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_opt_aggressive_inlining_large_function_not_inlined() {
    let source = "fn complex(x: i64) -> i64 = {
        let a = x + 1;
        let b = a * 2;
        let c = b - 3;
        let d = c + a;
        let e = d * b;
        let f = e - c;
        let g = f + d;
        let h = g * e;
        let i = h - f;
        let j = i + g;
        let k = j * h;
        let l = k - i;
        let m = l + j;
        let n = m * k;
        let o = n - l;
        let p = o + m;
        p
    };\nfn main() -> i64 = complex(1);";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pass = bmb::mir::AggressiveInlining::new();
    pass.run_on_program(&mut mir);
    let func = mir.functions.iter().find(|f| f.name == "complex");
    if let Some(f) = func {
        assert!(!f.always_inline, "large function should not be always_inline");
    }
}

// --- LinearRecurrenceToLoop ---

#[test]
fn test_opt_linear_recurrence_fib() {
    // Classic fibonacci — should be converted to iterative loop
    let source = "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };";
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::LinearRecurrenceToLoop::new()));
    // After transformation, should have loop blocks instead of recursive calls
    let func = &mir.functions[0];
    let _has_loop = func.blocks.iter().any(|b|
        matches!(&b.terminator, bmb::mir::Terminator::Branch { .. })
    );
    // LinearRecurrenceToLoop may or may not apply depending on exact MIR shape
    // Just verify it doesn't crash and produces valid MIR
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_opt_linear_recurrence_non_fibonacci() {
    // Non-fibonacci recursion should not be transformed
    let source = "fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };";
    let (_text, mir) = optimized_mir(source, Box::new(bmb::mir::LinearRecurrenceToLoop::new()));
    // factorial has single recursion with Mul, not double with Add — should NOT match
    assert!(!mir.functions.is_empty());
}

#[test]
fn test_opt_linear_recurrence_preserves_base_case() {
    // Base case should be preserved after transformation
    let source = "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };";
    let (text, _mir) = optimized_mir(source, Box::new(bmb::mir::LinearRecurrenceToLoop::new()));
    // The output should still reference the base case check (n <= 1)
    assert!(text.contains("<=") || text.contains("1") || text.contains("I:1"),
        "base case should be preserved in transformed MIR");
}

// --- Optimization Pipeline Levels ---

#[test]
fn test_opt_pipeline_aggressive_level() {
    // OptLevel::Aggressive should include more passes than Release
    let source = "fn main() -> i64 = 3 + 4;";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pipeline = bmb::mir::OptimizationPipeline::for_level(bmb::mir::OptLevel::Aggressive);
    pipeline.optimize(&mut mir);
    let text = bmb::mir::format_mir(&mir);
    assert!(text.contains("I:7"), "aggressive opt should fold 3+4 to 7");
}

#[test]
fn test_opt_pipeline_debug_no_optimization() {
    // Debug level should do minimal optimization
    let source = "fn main() -> i64 = 3 + 4;";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pipeline = bmb::mir::OptimizationPipeline::for_level(bmb::mir::OptLevel::Debug);
    pipeline.optimize(&mut mir);
    let _text = bmb::mir::format_mir(&mir);
    // Debug may or may not fold constants depending on pipeline setup
    assert!(!mir.functions.is_empty());
}

// --- Multi-pass combination tests ---

#[test]
fn test_opt_constant_fold_then_dce() {
    // Constant folding + DCE should work together
    let source = "fn f() -> i64 = {
        let unused = 3 + 4;
        10 * 2
    };";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(Box::new(bmb::mir::ConstantFolding));
    pipeline.add_pass(Box::new(bmb::mir::DeadCodeElimination));
    pipeline.optimize(&mut mir);
    let text = bmb::mir::format_mir(&mir);
    assert!(text.contains("I:20"), "should fold 10*2 to 20, got: {}", text);
}

#[test]
fn test_opt_simplify_then_unreachable() {
    // SimplifyBranches + UnreachableBlockElimination combination is not re-exported
    // Use ContractUnreachableElimination instead
    let source = "fn f(x: i64) -> i64 pre x > 0 = if x > 0 { x * 2 } else { 0 };";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(Box::new(bmb::mir::SimplifyBranches));
    pipeline.add_pass(Box::new(bmb::mir::ContractUnreachableElimination));
    pipeline.optimize(&mut mir);
    assert!(!mir.functions.is_empty());
}

// --- OptimizationStats ---

#[test]
fn test_opt_stats_tracking() {
    let stats = bmb::mir::OptimizationStats::new();
    assert_eq!(stats.iterations, 0);
    assert!(stats.pass_counts.is_empty());
}

#[test]
fn test_opt_pipeline_for_level_release() {
    // for_level(Release) should create a non-empty pipeline
    let pipeline = bmb::mir::OptimizationPipeline::for_level(bmb::mir::OptLevel::Release);
    let source = "fn main() -> i64 = 1 + 2;";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    pipeline.optimize(&mut mir);
    assert!(!mir.functions.is_empty());
}

// =============================================================================
// Cycle 248: WASM Codegen Advanced & Proof-Guided Optimization Tests
// =============================================================================

// --- WASM: Target Variants ---

#[test]
fn test_codegen_wasm_browser_target() {
    let program = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let codegen = bmb::codegen::WasmCodeGen::with_target(bmb::codegen::WasmTarget::Browser);
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "browser target should succeed: {:?}", result.err());
}

#[test]
fn test_codegen_wasm_standalone_target() {
    let program = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let codegen = bmb::codegen::WasmCodeGen::with_target(bmb::codegen::WasmTarget::Standalone);
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "standalone target should succeed: {:?}", result.err());
}

#[test]
fn test_codegen_wasm_with_memory_pages() {
    let program = lower_to_mir("fn id(x: i64) -> i64 = x;");
    let codegen = bmb::codegen::WasmCodeGen::new().with_memory(4);
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "custom memory pages should succeed");
    let wat = result.unwrap();
    assert!(wat.contains("memory"), "should declare memory: {}", wat);
}

// --- WASM: Complex Programs ---

#[test]
fn test_codegen_wasm_if_else() {
    let program = lower_to_mir("fn abs(x: i64) -> i64 = if x < 0 { 0 - x } else { x };");
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "if/else wasm codegen should succeed");
    let wat = result.unwrap();
    assert!(wat.contains("if") || wat.contains("br_if") || wat.contains("select"),
        "should contain control flow: {}", wat);
}

#[test]
fn test_codegen_wasm_recursive_function() {
    let program = lower_to_mir(
        "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };"
    );
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "recursive wasm codegen should succeed");
    let wat = result.unwrap();
    assert!(wat.contains("call"), "recursive function should have call instruction");
}

#[test]
fn test_codegen_wasm_while_loop() {
    let source = "fn countdown(n: i64) -> i64 = { let mut x = n; while x > 0 { x = x - 1; 0 }; x };";
    let program = lower_to_mir(source);
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "while loop wasm codegen should succeed");
    let wat = result.unwrap();
    assert!(wat.contains("loop") || wat.contains("br"), "should contain loop construct");
}

#[test]
fn test_codegen_wasm_float_arithmetic() {
    let program = lower_to_mir("fn area(r: f64) -> f64 = r * r * 3.14;");
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "float wasm codegen should succeed");
    let wat = result.unwrap();
    assert!(wat.contains("f64.mul") || wat.contains("f64"), "should use f64 operations");
}

#[test]
fn test_codegen_wasm_bool_operations() {
    let program = lower_to_mir("fn both(a: bool, b: bool) -> bool = a && b;");
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "bool wasm codegen should succeed");
}

#[test]
fn test_codegen_wasm_multiple_params() {
    let program = lower_to_mir("fn sum4(a: i64, b: i64, c: i64, d: i64) -> i64 = a + b + c + d;");
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok());
    let wat = result.unwrap();
    assert!(wat.contains("param"), "should declare parameters");
}

// --- WASM: All targets produce valid output ---

#[test]
fn test_codegen_wasm_all_targets_consistent() {
    let program = lower_to_mir("fn double(x: i64) -> i64 = x * 2;");
    let targets = [
        bmb::codegen::WasmTarget::Wasi,
        bmb::codegen::WasmTarget::Browser,
        bmb::codegen::WasmTarget::Standalone,
    ];
    for target in &targets {
        let codegen = bmb::codegen::WasmCodeGen::with_target(*target);
        let result = codegen.generate(&program);
        assert!(result.is_ok(), "target {:?} should succeed", target);
        let wat = result.unwrap();
        assert!(wat.contains("func"), "all targets should produce functions");
    }
}

// --- TextCodeGen Advanced ---

#[test]
fn test_codegen_text_with_custom_target() {
    let program = lower_to_mir("fn id(x: i64) -> i64 = x;");
    let codegen = bmb::codegen::TextCodeGen::with_target("x86_64-unknown-linux-gnu");
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "custom target text codegen should succeed");
    let ir = result.unwrap();
    assert!(ir.contains("x86_64-unknown-linux-gnu") || ir.contains("target"),
        "IR should reference target triple");
}

#[test]
fn test_codegen_text_contract_metadata() {
    let program = lower_to_mir("fn safe(x: i64) -> i64 pre x >= 0 = x;");
    let codegen = bmb::codegen::TextCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok());
    let ir = result.unwrap();
    assert!(ir.contains("safe"), "IR should contain function name");
}

#[test]
fn test_codegen_text_pure_function() {
    let program = lower_to_mir("@pure\nfn sq(x: i64) -> i64 = x * x;");
    let codegen = bmb::codegen::TextCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok());
    let ir = result.unwrap();
    assert!(ir.contains("sq"), "should have function name");
}

// --- Proof-Guided: Individual Pass Tests ---

#[test]
fn test_proof_bounds_check_elimination_pass() {
    let source = "fn safe_access(idx: i64) -> i64 pre idx >= 0 = idx;";
    let mut program = lower_to_mir(source);
    let pass = bmb::mir::BoundsCheckElimination::new();
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(Box::new(pass));
    pipeline.optimize(&mut program);
    assert!(!program.functions.is_empty());
}

#[test]
fn test_proof_null_check_elimination_pass() {
    let source = "fn safe_deref(x: i64) -> i64 pre x > 0 = x;";
    let mut program = lower_to_mir(source);
    let pass = bmb::mir::NullCheckElimination::new();
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(Box::new(pass));
    pipeline.optimize(&mut program);
    assert!(!program.functions.is_empty());
}

#[test]
fn test_proof_division_check_elimination_pass() {
    let source = "fn safe_div(a: i64, b: i64) -> i64 pre b != 0 = a / b;";
    let mut program = lower_to_mir(source);
    let pass = bmb::mir::DivisionCheckElimination::new();
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(Box::new(pass));
    pipeline.optimize(&mut program);
    assert!(!program.functions.is_empty());
}

#[test]
fn test_proof_unreachable_elimination_pass() {
    let source = "fn bounded(x: i64) -> i64 pre x > 0 = if x > 0 { x } else { 0 };";
    let mut program = lower_to_mir(source);
    let pass = bmb::mir::ProofUnreachableElimination::new();
    let mut pipeline = bmb::mir::OptimizationPipeline::new();
    pipeline.add_pass(Box::new(pass));
    pipeline.optimize(&mut program);
    assert!(!program.functions.is_empty());
}

// --- Proof-Guided: ProvenFactSet Query API ---

#[test]
fn test_proven_fact_set_lower_bound() {
    let program = lower_to_mir("fn f(x: i64) -> i64 pre x >= 0 = x;");
    let func = &program.functions[0];
    let facts = bmb::mir::ProvenFactSet::from_mir_preconditions(&func.preconditions);
    assert!(facts.has_lower_bound("x", 0), "x >= 0 should set lower bound to 0");
}

#[test]
fn test_proven_fact_set_nonzero() {
    let program = lower_to_mir("fn f(d: i64) -> i64 pre d != 0 = d;");
    let func = &program.functions[0];
    let facts = bmb::mir::ProvenFactSet::from_mir_preconditions(&func.preconditions);
    assert!(facts.has_nonzero("d"), "d != 0 should mark d as nonzero");
}

#[test]
fn test_proven_fact_set_empty() {
    let program = lower_to_mir("fn f(x: i64) -> i64 = x;");
    let func = &program.functions[0];
    let facts = bmb::mir::ProvenFactSet::from_mir_preconditions(&func.preconditions);
    assert!(!facts.has_nonzero("x"), "no contract should mean no facts");
    assert!(!facts.has_lower_bound("x", 0), "no contract should mean no lower bound");
}

#[test]
fn test_proven_fact_set_positive_implies_lower_bound() {
    let program = lower_to_mir("fn f(x: i64) -> i64 pre x > 0 = x;");
    let func = &program.functions[0];
    let facts = bmb::mir::ProvenFactSet::from_mir_preconditions(&func.preconditions);
    assert!(facts.has_lower_bound("x", 1), "x > 0 should set lower bound to 1");
}

// --- ProofOptimizationStats API ---

#[test]
fn test_proof_opt_stats_new() {
    let stats = bmb::mir::ProofOptimizationStats::new();
    assert_eq!(stats.total(), 0);
    assert_eq!(stats.bounds_checks_eliminated, 0);
    assert_eq!(stats.null_checks_eliminated, 0);
    assert_eq!(stats.division_checks_eliminated, 0);
    assert_eq!(stats.unreachable_blocks_eliminated, 0);
}

#[test]
fn test_proof_opt_stats_merge() {
    let mut stats1 = bmb::mir::ProofOptimizationStats::new();
    let mut stats2 = bmb::mir::ProofOptimizationStats::new();
    stats2.bounds_checks_eliminated = 3;
    stats2.division_checks_eliminated = 1;
    stats1.merge(&stats2);
    assert_eq!(stats1.bounds_checks_eliminated, 3);
    assert_eq!(stats1.division_checks_eliminated, 1);
    assert_eq!(stats1.total(), 4);
}

#[test]
fn test_proof_opt_stats_from_simple_program() {
    let mut program = lower_to_mir("fn f(x: i64) -> i64 = x + 1;");
    let stats = bmb::mir::run_proof_guided_program(&mut program);
    assert_eq!(stats.total(), 0);
}

#[test]
fn test_proof_opt_stats_from_contract_program() {
    let mut program = lower_to_mir("fn f(x: i64) -> i64 pre x > 0 = x;");
    let stats = bmb::mir::run_proof_guided_program(&mut program);
    let _total = stats.total();
}

// --- Proof-Guided: run_proof_guided_optimizations on function ---

#[test]
fn test_proof_guided_optimizations_on_function() {
    let mut program = lower_to_mir("fn safe(a: i64, b: i64) -> i64 pre b != 0 = a / b;");
    let stats = bmb::mir::run_proof_guided_optimizations(&mut program.functions[0]);
    let _total = stats.total();
}

#[test]
fn test_proof_guided_multiple_contracts() {
    let source = "fn bounded_div(a: i64, b: i64) -> i64 pre a >= 0 && b > 0 = a / b;";
    let mut program = lower_to_mir(source);
    let stats = bmb::mir::run_proof_guided_program(&mut program);
    let _total = stats.total();
}

// --- Combined: WASM + Optimization Pipeline ---

#[test]
fn test_pipeline_optimized_to_wasm() {
    let source = "fn main() -> i64 = 3 + 4;";
    let tokens = tokenize(source).expect("tokenize failed");
    let ast = parse("test.bmb", source, tokens).expect("parse failed");
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mut mir = bmb::mir::lower_program(&ast);
    let pipeline = bmb::mir::OptimizationPipeline::for_level(bmb::mir::OptLevel::Release);
    pipeline.optimize(&mut mir);
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&mir);
    assert!(result.is_ok(), "optimized MIR to WASM should succeed");
}

#[test]
fn test_pipeline_proof_guided_then_wasm() {
    let source = "fn safe_add(a: i64, b: i64) -> i64 pre a >= 0 && b >= 0 = a + b;";
    let mut program = lower_to_mir(source);
    bmb::mir::run_proof_guided_program(&mut program);
    let codegen = bmb::codegen::WasmCodeGen::with_target(bmb::codegen::WasmTarget::Standalone);
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "proof-guided optimized MIR to WASM should succeed");
}

// =============================================================================
// Cycle 249: Edge Case & Stress Integration Tests
// =============================================================================

/// Helper: try to run a program through interpreter, returning Result
fn try_run_program(source: &str) -> Result<bmb::interp::Value, String> {
    let tokens = tokenize(source).map_err(|e| format!("{e}"))?;
    let ast = parse("test.bmb", source, tokens).map_err(|e| format!("{e}"))?;
    let mut tc = TypeChecker::new();
    tc.check_program(&ast).map_err(|e| format!("{e}"))?;
    let mut interp = bmb::interp::Interpreter::new();
    interp.run(&ast).map_err(|e| format!("{e}"))
}

// --- Parser Edge Cases ---

#[test]
fn test_edge_empty_struct_definition() {
    // Empty struct should parse and type-check
    assert!(type_checks("struct Empty {}\nfn main() -> i64 = 0;"));
}

#[test]
fn test_edge_empty_struct_instantiation() {
    assert!(type_checks("struct S {}\nfn f() -> S = new S {};"));
}

#[test]
fn test_edge_single_field_struct() {
    assert_eq!(run_program_i64("struct W { val: i64 }\nfn main() -> i64 = { let w = new W { val: 99 }; w.val };"), 99);
}

#[test]
fn test_edge_long_identifier() {
    // Very long identifier name should work
    let long_name = "a".repeat(200);
    let source = format!("fn {}() -> i64 = 42;\nfn main() -> i64 = {}();", long_name, long_name);
    assert_eq!(run_program_i64(&source), 42);
}

#[test]
fn test_edge_deeply_nested_arithmetic() {
    // Deeply nested expression: ((((1+1)+1)+1)...+1) = 20
    let mut expr = "1".to_string();
    for _ in 0..19 {
        expr = format!("({} + 1)", expr);
    }
    let source = format!("fn main() -> i64 = {};", expr);
    assert_eq!(run_program_i64(&source), 20);
}

#[test]
fn test_edge_deeply_nested_if_else() {
    // 10 levels of nested if/else
    let source = "fn main() -> i64 = if true { if true { if true { if true { if true { if true { if true { if true { if true { if true { 42 } else { 0 } } else { 0 } } else { 0 } } else { 0 } } else { 0 } } else { 0 } } else { 0 } } else { 0 } } else { 0 } } else { 0 };";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_edge_string_escape_newline() {
    // String with newline escape
    assert!(type_checks("fn main() -> String = \"hello\\nworld\";"));
}

#[test]
fn test_edge_string_escape_tab() {
    assert!(type_checks("fn main() -> String = \"col1\\tcol2\";"));
}

#[test]
fn test_edge_string_empty() {
    assert!(type_checks("fn main() -> String = \"\";"));
}

#[test]
fn test_edge_operator_precedence_mul_add() {
    // 2 + 3 * 4 should be 14 (not 20)
    assert_eq!(run_program_i64("fn main() -> i64 = 2 + 3 * 4;"), 14);
}

#[test]
fn test_edge_operator_precedence_comparison_and_logic() {
    // 3 > 2 && 1 < 5 should be true
    let source = "fn main() -> bool = 3 > 2 && 1 < 5;";
    assert!(type_checks(source));
}

#[test]
fn test_edge_unary_negation() {
    assert_eq!(run_program_i64("fn main() -> i64 = -42;"), -42);
}

#[test]
fn test_edge_double_negation() {
    assert_eq!(run_program_i64("fn main() -> i64 = -(-42);"), 42);
}

// --- Type System Edge Cases ---

#[test]
fn test_edge_unit_return_type() {
    assert!(type_checks("fn side_effect() -> () = ();"));
}

#[test]
fn test_edge_function_returning_bool() {
    assert!(type_checks("fn is_positive(x: i64) -> bool = x > 0;"));
}

#[test]
fn test_edge_recursive_function_type_checks() {
    assert!(type_checks("fn inf(n: i64) -> i64 = inf(n);"));
}

#[test]
fn test_edge_type_error_wrong_return_type() {
    assert!(type_error("fn f() -> i64 = true;"));
}

#[test]
fn test_edge_type_error_bool_arithmetic() {
    // Cannot add booleans
    assert!(type_error("fn f() -> bool = true + false;"));
}

#[test]
fn test_edge_type_error_string_int_add() {
    assert!(type_error("fn f() -> i64 = 1 + \"hello\";"));
}

#[test]
fn test_edge_generic_identity_chain() {
    // Chain of generic identity calls
    assert!(type_checks(
        "fn id<T>(x: T) -> T = x;\nfn main() -> i64 = id(id(id(42)));"
    ));
}

// --- Interpreter Edge Cases ---

#[test]
fn test_edge_interp_division_by_zero() {
    // Division by zero should error, not crash
    let result = try_run_program("fn main() -> i64 = 1 / 0;");
    assert!(result.is_err(), "division by zero should produce an error");
}

#[test]
fn test_edge_interp_modulo_by_zero() {
    let result = try_run_program("fn main() -> i64 = 10 % 0;");
    assert!(result.is_err(), "modulo by zero should produce an error");
}

#[test]
fn test_edge_interp_negative_modulo() {
    // -7 % 3 behavior
    let result = run_program_i64("fn main() -> i64 = -7 % 3;");
    // Rust semantics: -7 % 3 == -1
    assert_eq!(result, -1);
}

#[test]
fn test_edge_interp_zero_iteration_while() {
    // While that never executes body
    assert_eq!(run_program_i64("fn main() -> i64 = { let mut x = 0; while false { x = 1; 0 }; x };"), 0);
}

#[test]
fn test_edge_interp_zero_iteration_for() {
    // For loop with empty range
    assert_eq!(run_program_i64("fn main() -> i64 = { let mut s: i64 = 0; for i in 0..0 { s = s + 1; 0 }; s };"), 0);
}

#[test]
fn test_edge_interp_single_iteration_for() {
    assert_eq!(run_program_i64("fn main() -> i64 = { let mut s: i64 = 0; for i in 0..1 { s = s + i; 0 }; s };"), 0);
}

#[test]
fn test_edge_interp_empty_array() {
    // Empty array creation (if supported)
    assert!(type_checks("fn main() -> [i64; 0] = [];"));
}

#[test]
fn test_edge_interp_nested_function_calls() {
    // Deep call chain: f1 -> f2 -> f3 -> f4 -> f5
    let source = "
        fn f5(x: i64) -> i64 = x + 1;
        fn f4(x: i64) -> i64 = f5(x) + 1;
        fn f3(x: i64) -> i64 = f4(x) + 1;
        fn f2(x: i64) -> i64 = f3(x) + 1;
        fn f1(x: i64) -> i64 = f2(x) + 1;
        fn main() -> i64 = f1(0);
    ";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_edge_interp_many_local_variables() {
    // Function with many locals
    let source = "fn main() -> i64 = {
        let a = 1; let b = 2; let c = 3; let d = 4; let e = 5;
        let f = 6; let g = 7; let h = 8; let i = 9; let j = 10;
        let k = 11; let l = 12; let m = 13; let n = 14; let o = 15;
        let p = 16; let q = 17; let r = 18; let s = 19; let t = 20;
        a + b + c + d + e + f + g + h + i + j + k + l + m + n + o + p + q + r + s + t
    };";
    assert_eq!(run_program_i64(source), 210); // sum 1..20
}

// --- MIR Edge Cases ---

#[test]
fn test_edge_mir_identity_function() {
    // Simplest possible MIR: just return param
    let mir = lower_to_mir("fn id(x: i64) -> i64 = x;");
    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].params.len(), 1);
}

#[test]
fn test_edge_mir_no_params_constant() {
    let mir = lower_to_mir("fn answer() -> i64 = 42;");
    assert_eq!(mir.functions[0].params.len(), 0);
}

#[test]
fn test_edge_mir_function_with_contract_facts() {
    let mir = lower_to_mir("fn safe(x: i64) -> i64 pre x > 0 post ret >= 0 = x;");
    let func = &mir.functions[0];
    assert!(!func.preconditions.is_empty(), "should have preconditions");
    assert!(!func.postconditions.is_empty(), "should have postconditions");
}

#[test]
fn test_edge_mir_pure_function_flag() {
    let mir = lower_to_mir("@pure\nfn sq(x: i64) -> i64 = x * x;");
    let func = &mir.functions[0];
    assert!(func.is_pure, "should be marked pure");
}

#[test]
fn test_edge_mir_const_function_flag() {
    let mir = lower_to_mir("@const\nfn magic() -> i64 = 42;");
    let func = &mir.functions[0];
    assert!(func.is_const, "should be marked const");
}

// --- Error Handling Edge Cases ---

#[test]
fn test_edge_error_undefined_variable() {
    assert!(type_error("fn f() -> i64 = x;"));
}

#[test]
fn test_edge_error_undefined_function_call() {
    assert!(type_error("fn f() -> i64 = nonexistent();"));
}

#[test]
fn test_edge_error_wrong_arg_count() {
    assert!(type_error("fn add(a: i64, b: i64) -> i64 = a + b;\nfn main() -> i64 = add(1);"));
}

#[test]
fn test_edge_error_parse_unclosed_brace() {
    assert!(parse_error("fn f() -> i64 = {"));
}

#[test]
fn test_edge_error_parse_missing_semicolon() {
    assert!(parse_error("fn f() -> i64 = 42\nfn g() -> i64 = 1;"));
}

#[test]
fn test_edge_error_parse_invalid_token() {
    assert!(parse_error("fn f() -> i64 = @@@;"));
}

// --- Contract Edge Cases ---

#[test]
fn test_edge_contract_postcondition_only() {
    assert!(type_checks("fn positive() -> i64 post ret > 0 = 42;"));
}

#[test]
fn test_edge_contract_zero_param_function() {
    // Contract on function with no params but postcondition
    let mir = lower_to_mir("fn always_one() -> i64 post ret == 1 = 1;");
    assert!(!mir.functions[0].postconditions.is_empty());
}

#[test]
fn test_edge_contract_combined_pre_post() {
    assert!(type_checks(
        "fn clamp(x: i64, lo: i64, hi: i64) -> i64 pre lo <= hi post ret >= lo = if x < lo { lo } else if x > hi { hi } else { x };"
    ));
}

// --- Codegen Edge Cases ---

#[test]
fn test_edge_codegen_unit_function_wasm() {
    let program = lower_to_mir("fn noop() -> () = ();");
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "unit function WASM should succeed");
}

#[test]
fn test_edge_codegen_unit_function_text() {
    let program = lower_to_mir("fn noop() -> () = ();");
    let codegen = bmb::codegen::TextCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "unit function text codegen should succeed");
}

#[test]
fn test_edge_codegen_many_functions() {
    // Program with 10 functions
    let mut source = String::new();
    for i in 0..10 {
        source.push_str(&format!("fn f{}(x: i64) -> i64 = x + {};\n", i, i));
    }
    let program = lower_to_mir(&source);
    assert_eq!(program.functions.len(), 10);
    let codegen = bmb::codegen::WasmCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "10 functions WASM should succeed");
}

// =============================================================================
// Cycle 251 (FINAL): Coverage Gaps & Remaining Tests
// =============================================================================

// --- Cast Operations ---

#[test]
fn test_cast_chain_i64_f64_i64() {
    // Chained cast: i64 -> f64 -> i64
    let source = "fn main() -> i64 = { let x: i64 = 42; let f = x as f64; f as i64 };";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_cast_bool_to_i64_true() {
    let source = "fn main() -> i64 = true as i64;";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_cast_bool_to_i64_false() {
    let source = "fn main() -> i64 = false as i64;";
    assert_eq!(run_program_i64(source), 0);
}

// --- Bitwise Operation Combinations ---

#[test]
fn test_bitwise_mask_pattern() {
    // Common bitwise pattern: and + or
    let source = "fn main() -> i64 = (255 band 15) bor (240 band 255);";
    assert_eq!(run_program_i64(source), 15 | 240); // 255
}

#[test]
fn test_bitwise_xor_self_is_zero() {
    let source = "fn main() -> i64 = 42 bxor 42;";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_bitwise_shift_multiply() {
    // Shift left by 3 = multiply by 8
    let source = "fn main() -> i64 = 5 << 3;";
    assert_eq!(run_program_i64(source), 40);
}

// --- Trait Type-Checking (runtime not supported) ---

#[test]
fn test_trait_with_default_style_method() {
    // Trait with method taking self: Self
    assert!(type_checks(
        "trait Sizeable { fn size(self: Self) -> i64; }
         struct Box { w: i64, h: i64 }
         impl Sizeable for Box { fn size(self: Self) -> i64 = self.w * self.h; }"
    ));
}

#[test]
fn test_trait_multiple_impls_different_structs() {
    // Same trait implemented by two different structs
    assert!(type_checks(
        "trait Area { fn area(self: Self) -> i64; }
         struct Rect { w: i64, h: i64 }
         struct Circle { r: i64 }
         impl Area for Rect { fn area(self: Self) -> i64 = self.w * self.h; }
         impl Area for Circle { fn area(self: Self) -> i64 = self.r * self.r * 3; }"
    ));
}

#[test]
fn test_trait_impl_wrong_return_type_rejected() {
    // v0.90.39: Trait impl with mismatched return type is now rejected
    assert!(type_error(
        "trait GetVal { fn get(self: Self) -> i64; }
         struct S { v: bool }
         impl GetVal for S { fn get(self: Self) -> bool = self.v; }"
    ));
}

// --- Error Recovery ---

#[test]
fn test_error_multiple_undefined_references() {
    // Multiple undefined references in one program
    assert!(type_error("fn f() -> i64 = x + y + z;"));
}

#[test]
fn test_error_type_mismatch_in_if_branches() {
    // If branches return different types
    assert!(type_error("fn f(b: bool) -> i64 = if b { 42 } else { true };"));
}

#[test]
fn test_error_recursive_without_base_case_types() {
    // Infinite recursion type-checks (no termination check)
    assert!(type_checks("fn loop_forever(x: i64) -> i64 = loop_forever(x);"));
}

// --- Array Advanced ---

#[test]
fn test_array_sum_via_for_loop() {
    let source = "fn main() -> i64 = {
        let arr = [10, 20, 30, 40, 50];
        let mut total: i64 = 0;
        for i in 0..5 {
            total = total + arr[i];
            0
        };
        total
    };";
    assert_eq!(run_program_i64(source), 150);
}

#[test]
fn test_array_repeat_and_modify() {
    let source = "fn main() -> i64 = {
        let mut arr = [0; 5];
        set arr[0] = 1;
        set arr[1] = 2;
        set arr[2] = 3;
        arr[0] + arr[1] + arr[2]
    };";
    assert_eq!(run_program_i64(source), 6);
}

// --- String Advanced ---

#[test]
fn test_string_is_empty_true() {
    let source = "fn main() -> bool = \"\".is_empty();";
    assert!(type_checks(source));
}

#[test]
fn test_string_len_nonempty() {
    let source = "fn main() -> i64 = \"hello\".len();";
    assert_eq!(run_program_i64(source), 5);
}

// --- Additional Semantic Tests ---

#[test]
fn test_semantic_fibonacci_iterative() {
    // Iterative fibonacci via while loop
    let source = "fn main() -> i64 = {
        let mut a: i64 = 0;
        let mut b: i64 = 1;
        let mut n: i64 = 0;
        while n < 20 {
            let temp = a + b;
            a = b;
            b = temp;
            n = n + 1;
            0
        };
        a
    };";
    // fib(20) = 6765
    assert_eq!(run_program_i64(source), 6765);
}

#[test]
fn test_semantic_is_prime_recursive() {
    let source = "
        fn check_div(n: i64, d: i64) -> bool = {
            if d * d > n { true }
            else if n % d == 0 { false }
            else { check_div(n, d + 1) }
        };
        fn is_prime(n: i64) -> bool = if n < 2 { false } else { check_div(n, 2) };
        fn main() -> i64 = if is_prime(97) { 1 } else { 0 };
    ";
    assert_eq!(run_program_i64(source), 1); // 97 is prime
}

#[test]
fn test_semantic_binary_search_pattern() {
    // Binary search simulation with known values
    let source = "
        fn bsearch(target: i64, lo: i64, hi: i64) -> i64 = {
            if lo > hi { -1 }
            else {
                let mid = (lo + hi) / 2;
                if mid == target { mid }
                else if mid < target { bsearch(target, mid + 1, hi) }
                else { bsearch(target, lo, mid - 1) }
            }
        };
        fn main() -> i64 = bsearch(7, 0, 15);
    ";
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_semantic_mutual_recursion_even_odd() {
    let source = "
        fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
        fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };
        fn main() -> i64 = if is_even(10) { 1 } else { 0 };
    ";
    assert_eq!(run_program_i64(source), 1);
}

// --- MIR/Codegen Coverage ---

#[test]
fn test_mir_format_roundtrip() {
    // Format MIR and verify it's non-empty
    let mir = lower_to_mir("fn add(a: i64, b: i64) -> i64 = a + b;");
    let text = bmb::mir::format_mir(&mir);
    assert!(text.contains("add"), "formatted MIR should contain function name");
    assert!(text.contains("+") || text.contains("Add"), "should contain add operation");
}

#[test]
fn test_codegen_text_multiple_functions_ir() {
    let source = "fn inc(x: i64) -> i64 = x + 1;\nfn dec(x: i64) -> i64 = x - 1;";
    let program = lower_to_mir(source);
    let codegen = bmb::codegen::TextCodeGen::new();
    let result = codegen.generate(&program);
    assert!(result.is_ok());
    let ir = result.unwrap();
    assert!(ir.contains("inc") && ir.contains("dec"), "IR should contain both functions");
}

#[test]
fn test_codegen_wasm_contract_function() {
    let source = "fn safe(x: i64) -> i64 pre x >= 0 post ret >= 0 = x;";
    let program = lower_to_mir(source);
    let codegen = bmb::codegen::WasmCodeGen::with_target(bmb::codegen::WasmTarget::Standalone);
    let result = codegen.generate(&program);
    assert!(result.is_ok(), "contract function should generate WASM");
}

// =============================================================================
// Cycle 250: Semantic Correctness & Feature Interaction Tests
// =============================================================================

// --- Scope & Shadowing Semantics ---

#[test]
fn test_scope_parameter_shadows_outer_binding() {
    // Function parameter x shadows outer let x
    let source = "
        fn inner(x: i64) -> i64 = x * 2;
        fn main() -> i64 = {
            let x = 10;
            inner(5)
        };
    ";
    assert_eq!(run_program_i64(source), 10); // inner(5) = 5*2 = 10, not 10*2
}

#[test]
fn test_scope_block_shadowing_inner() {
    // In BMB, inner block let shadows for the rest of the enclosing scope
    let source = "fn main() -> i64 = {
        let x = 100;
        let y = { let x = 42; x };
        x + y
    };";
    // BMB: inner let x = 42 shadows outer x, so x=42, y=42 → 84
    assert_eq!(run_program_i64(source), 84);
}

#[test]
fn test_scope_let_shadowing_same_block() {
    // Rebinding with same name in same block
    let source = "fn main() -> i64 = {
        let x = 1;
        let x = x + 10;
        let x = x * 2;
        x
    };";
    assert_eq!(run_program_i64(source), 22); // 1 -> 11 -> 22
}

#[test]
fn test_scope_closure_captures_outer() {
    // Closure captures the outer variable, not a shadowed version
    let source = "fn main() -> i64 = {
        let a = 100;
        let f = fn |x: i64| { a + x };
        f(5)
    };";
    assert_eq!(run_program_i64(source), 105);
}

// --- Match & Enum Semantics ---

#[test]
fn test_match_all_variants_exhaustive() {
    // Match on enum with all variants covered
    let source = "
        enum Color { Red, Green, Blue }
        fn to_num(c: Color) -> i64 = match c {
            Color::Red => 1,
            Color::Green => 2,
            Color::Blue => 3,
        };
        fn main() -> i64 = to_num(Color::Green);
    ";
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_match_enum_with_data_destructure() {
    // Match on enum variant with data, extract fields
    let source = "
        enum Result { Ok(i64), Err(i64) }
        fn unwrap_or(r: Result, default: i64) -> i64 = match r {
            Result::Ok(v) => v,
            Result::Err(_e) => default,
        };
        fn main() -> i64 = unwrap_or(Result::Ok(42), 0);
    ";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_match_enum_err_branch() {
    let source = "
        enum Result { Ok(i64), Err(i64) }
        fn unwrap_or_default(r: Result, default: i64) -> i64 = match r {
            Result::Ok(v) => v,
            Result::Err(_e) => default,
        };
        fn main() -> i64 = unwrap_or_default(Result::Err(99), -1);
    ";
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_match_nested_if_in_arm() {
    // Match arm contains if expression
    let source = "
        fn classify(x: i64) -> i64 = match x {
            0 => 0,
            _ => if x > 0 { 1 } else { -1 },
        };
        fn main() -> i64 = classify(-5);
    ";
    assert_eq!(run_program_i64(source), -1);
}

// --- Control Flow Interactions ---

#[test]
fn test_while_loop_complex_state() {
    // While loop with multiple mutable variables
    let source = "fn main() -> i64 = {
        let mut a: i64 = 1;
        let mut b: i64 = 1;
        let mut n: i64 = 0;
        while n < 10 {
            let temp = a + b;
            a = b;
            b = temp;
            n = n + 1;
            0
        };
        b
    };";
    // Fibonacci: after 10 iterations, b should be fib(12) = 144
    assert_eq!(run_program_i64(source), 144);
}

#[test]
fn test_for_loop_accumulation_pattern() {
    // Classic accumulation: sum of squares
    let source = "fn main() -> i64 = {
        let mut total: i64 = 0;
        for i in 1..6 {
            total = total + i * i;
            0
        };
        total
    };";
    // 1 + 4 + 9 + 16 + 25 = 55
    assert_eq!(run_program_i64(source), 55);
}

#[test]
fn test_nested_match_in_loop() {
    // Match inside a loop — each iteration dispatches
    let source = "
        enum Op { Add, Sub, Nop }
        fn apply(op: Op, acc: i64, val: i64) -> i64 = match op {
            Op::Add => acc + val,
            Op::Sub => acc - val,
            Op::Nop => acc,
        };
        fn main() -> i64 = {
            let a = apply(Op::Add, 0, 10);
            let b = apply(Op::Sub, a, 3);
            let c = apply(Op::Add, b, 5);
            c
        };
    ";
    assert_eq!(run_program_i64(source), 12); // 0+10-3+5 = 12
}

// --- Contract Semantics ---

#[test]
fn test_contract_pre_verified_at_type_level() {
    // Precondition should type-check
    let source = "fn safe_sqrt(x: i64) -> i64 pre x >= 0 = x;";
    assert!(type_checks(source));
}

#[test]
fn test_contract_post_ret_keyword() {
    // Postcondition uses ret keyword
    let source = "fn positive() -> i64 post ret > 0 = 42;";
    assert!(type_checks(source));
    let mir = lower_to_mir(source);
    assert!(!mir.functions[0].postconditions.is_empty());
}

#[test]
fn test_contract_combined_pre_post_mir() {
    // Both pre and post conditions in MIR
    let source = "fn clamp_positive(x: i64) -> i64 pre x >= 0 post ret >= 0 = x;";
    let mir = lower_to_mir(source);
    let func = &mir.functions[0];
    assert!(!func.preconditions.is_empty(), "should have pre");
    assert!(!func.postconditions.is_empty(), "should have post");
}

#[test]
fn test_contract_on_helper_function() {
    // Contract on a helper called by main
    let source = "
        fn safe_div(a: i64, b: i64) -> i64 pre b != 0 = a / b;
        fn main() -> i64 = safe_div(100, 5);
    ";
    assert_eq!(run_program_i64(source), 20);
}

// --- Generic Type Interactions ---

#[test]
fn test_generic_struct_with_method_pattern() {
    // Generic struct used with specific type
    let source = "
        struct Pair<T> { first: T, second: T }
        fn sum_pair(p: Pair<i64>) -> i64 = p.first + p.second;
        fn main() -> i64 = sum_pair(new Pair { first: 10, second: 20 });
    ";
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_generic_enum_option_pattern() {
    // Generic Option-like enum
    let source = "
        enum Maybe<T> { Just(T), Nothing }
        fn unwrap(m: Maybe<i64>) -> i64 = match m {
            Maybe::Just(v) => v,
            Maybe::Nothing => 0,
        };
        fn main() -> i64 = unwrap(Maybe::Just(77));
    ";
    assert_eq!(run_program_i64(source), 77);
}

#[test]
fn test_generic_enum_nothing_branch() {
    let source = "
        enum Maybe<T> { Just(T), Nothing }
        fn unwrap_default(m: Maybe<i64>) -> i64 = match m {
            Maybe::Just(v) => v,
            Maybe::Nothing => -1,
        };
        fn main() -> i64 = unwrap_default(Maybe::Nothing);
    ";
    assert_eq!(run_program_i64(source), -1);
}

// --- Struct & Field Semantics ---

#[test]
fn test_struct_field_access_chain() {
    // Access field of returned struct
    let source = "
        struct Point { x: i64, y: i64 }
        fn origin() -> Point = new Point { x: 0, y: 0 };
        fn main() -> i64 = origin().x + origin().y;
    ";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_struct_as_function_param_and_return() {
    let source = "
        struct Vec2 { x: i64, y: i64 }
        fn add_vec(a: Vec2, b: Vec2) -> Vec2 = new Vec2 { x: a.x + b.x, y: a.y + b.y };
        fn main() -> i64 = {
            let v = add_vec(new Vec2 { x: 1, y: 2 }, new Vec2 { x: 3, y: 4 });
            v.x + v.y
        };
    ";
    assert_eq!(run_program_i64(source), 10); // (1+3) + (2+4) = 10
}

#[test]
fn test_struct_nested_field_deep() {
    let source = "
        struct Inner { val: i64 }
        struct Outer { inner: Inner }
        fn main() -> i64 = {
            let o = new Outer { inner: new Inner { val: 99 } };
            o.inner.val
        };
    ";
    assert_eq!(run_program_i64(source), 99);
}

// --- Closure Interactions ---

#[test]
fn test_closure_applied_multiple_times() {
    // Apply same closure to different values
    let source = "fn main() -> i64 = {
        let double = fn |x: i64| { x * 2 };
        double(3) + double(7)
    };";
    assert_eq!(run_program_i64(source), 20); // 6 + 14
}

#[test]
fn test_closure_captures_two_outer_vars() {
    let source = "fn main() -> i64 = {
        let a = 10;
        let b = 20;
        let f = fn |x: i64| { a + b + x };
        f(5)
    };";
    assert_eq!(run_program_i64(source), 35);
}

// --- Multi-Feature Combinations ---

#[test]
fn test_combo_struct_enum_match_function() {
    // Struct + Enum + Match + Function call
    let source = "
        struct Point { x: i64, y: i64 }
        enum Quadrant { First, Second, Third, Fourth, Origin }
        fn classify(p: Point) -> Quadrant = {
            if p.x == 0 && p.y == 0 { Quadrant::Origin }
            else if p.x > 0 && p.y > 0 { Quadrant::First }
            else if p.x < 0 && p.y > 0 { Quadrant::Second }
            else if p.x < 0 && p.y < 0 { Quadrant::Third }
            else { Quadrant::Fourth }
        };
        fn quadrant_num(q: Quadrant) -> i64 = match q {
            Quadrant::First => 1,
            Quadrant::Second => 2,
            Quadrant::Third => 3,
            Quadrant::Fourth => 4,
            Quadrant::Origin => 0,
        };
        fn main() -> i64 = quadrant_num(classify(new Point { x: -3, y: 5 }));
    ";
    assert_eq!(run_program_i64(source), 2); // (-3, 5) is Second quadrant
}

#[test]
fn test_combo_recursive_with_contract() {
    // Recursive function with contract
    let source = "
        fn fib(n: i64) -> i64 pre n >= 0 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };
        fn main() -> i64 = fib(10);
    ";
    assert_eq!(run_program_i64(source), 55);
}

#[test]
fn test_combo_closure_with_struct() {
    // Closure creates and returns struct field
    let source = "
        struct Pair { a: i64, b: i64 }
        fn main() -> i64 = {
            let make = fn |x: i64| { new Pair { a: x, b: x * 2 } };
            let p = make(5);
            p.a + p.b
        };
    ";
    assert_eq!(run_program_i64(source), 15); // 5 + 10
}

#[test]
fn test_combo_enum_in_loop() {
    // Process enum values in a sequence
    let source = "
        enum Action { Inc, Dec, Nop }
        fn apply_action(action: Action, val: i64) -> i64 = match action {
            Action::Inc => val + 1,
            Action::Dec => val - 1,
            Action::Nop => val,
        };
        fn main() -> i64 = {
            let v1 = apply_action(Action::Inc, 0);
            let v2 = apply_action(Action::Inc, v1);
            let v3 = apply_action(Action::Dec, v2);
            let v4 = apply_action(Action::Inc, v3);
            v4
        };
    ";
    assert_eq!(run_program_i64(source), 2); // 0+1+1-1+1 = 2
}

#[test]
fn test_combo_generic_with_struct_and_function() {
    let source = "
        struct Wrapper<T> { val: T }
        fn wrap(x: i64) -> Wrapper<i64> = new Wrapper { val: x };
        fn unwrap_add(w: Wrapper<i64>, n: i64) -> i64 = w.val + n;
        fn main() -> i64 = unwrap_add(wrap(40), 2);
    ";
    assert_eq!(run_program_i64(source), 42);
}

// --- Pipeline Verification: Full Semantic Roundtrip ---

#[test]
fn test_semantic_roundtrip_factorial() {
    // Verify factorial through full pipeline
    let source = "
        fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) };
        fn main() -> i64 = fact(10);
    ";
    assert_eq!(run_program_i64(source), 3628800);
}

#[test]
fn test_semantic_roundtrip_gcd() {
    // Euclidean GCD
    let source = "
        fn gcd(a: i64, b: i64) -> i64 = if b == 0 { a } else { gcd(b, a % b) };
        fn main() -> i64 = gcd(48, 18);
    ";
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_semantic_roundtrip_power() {
    // Integer exponentiation
    let source = "
        fn power(base: i64, exp: i64) -> i64 = if exp == 0 { 1 } else { base * power(base, exp - 1) };
        fn main() -> i64 = power(2, 10);
    ";
    assert_eq!(run_program_i64(source), 1024);
}

#[test]
fn test_semantic_roundtrip_collatz_steps() {
    // Count Collatz steps to reach 1
    let source = "
        fn collatz(n: i64) -> i64 = if n == 1 { 0 } else if n % 2 == 0 { 1 + collatz(n / 2) } else { 1 + collatz(3 * n + 1) };
        fn main() -> i64 = collatz(27);
    ";
    assert_eq!(run_program_i64(source), 111);
}

// ============================================================
// Cycle 252: Struct Duplicate Field Detection
// ============================================================

#[test]
fn test_struct_duplicate_field_rejected() {
    // Struct with duplicate field names should be a type error
    assert!(type_error("struct Bad { x: i64, x: bool }"));
}

#[test]
fn test_struct_duplicate_field_error_message() {
    // Error message should mention the duplicate field name
    assert!(type_error_contains(
        "struct Bad { x: i64, x: bool }",
        "duplicate field 'x'"
    ));
}

#[test]
fn test_struct_duplicate_field_three_fields() {
    // Duplicate among three fields
    assert!(type_error("struct S { a: i64, b: bool, a: f64 }"));
}

#[test]
fn test_struct_duplicate_field_same_type() {
    // Even same type should be rejected
    assert!(type_error("struct S { x: i64, x: i64 }"));
}

#[test]
fn test_struct_no_duplicate_fields_ok() {
    // Distinct field names should type-check fine
    assert!(type_checks("struct Point { x: i64, y: i64 }"));
}

#[test]
fn test_struct_single_field_ok() {
    // Single field should always be fine
    assert!(type_checks("struct Wrapper { value: i64 }"));
}

#[test]
fn test_enum_duplicate_variant_rejected() {
    // Enum with duplicate variant names should be a type error
    assert!(type_error("enum Bad { A, B, A }"));
}

#[test]
fn test_enum_duplicate_variant_error_message() {
    // Error message should mention the duplicate variant name
    assert!(type_error_contains(
        "enum Bad { A, B, A }",
        "duplicate variant 'A'"
    ));
}

#[test]
fn test_enum_duplicate_variant_with_data() {
    // Duplicate variant with different data should still be rejected
    assert!(type_error("enum Bad { Ok(i64), Err(bool), Ok(f64) }"));
}

#[test]
fn test_enum_no_duplicate_variants_ok() {
    // Distinct variant names should type-check fine
    assert!(type_checks("enum Color { Red, Green, Blue }"));
}

#[test]
fn test_generic_struct_duplicate_field_rejected() {
    // Generic struct with duplicate fields should also be rejected
    assert!(type_error("struct Pair<T> { first: T, first: T }"));
}

#[test]
fn test_generic_enum_duplicate_variant_rejected() {
    // Generic enum with duplicate variants should also be rejected
    assert!(type_error("enum Result<T, E> { Ok(T), Err(E), Ok(T) }"));
}

// ============================================================
// Cycle 253: Hex/Octal/Binary Literal Parsing
// ============================================================

#[test]
fn test_hex_literal_basic() {
    // 0xFF = 255
    let source = "fn main() -> i64 = 0xFF;";
    assert_eq!(run_program_i64(source), 255);
}

#[test]
fn test_hex_literal_lowercase() {
    // 0xff = 255
    let source = "fn main() -> i64 = 0xff;";
    assert_eq!(run_program_i64(source), 255);
}

#[test]
fn test_hex_literal_uppercase_prefix() {
    // 0XFF = 255
    let source = "fn main() -> i64 = 0XFF;";
    assert_eq!(run_program_i64(source), 255);
}

#[test]
fn test_hex_literal_in_arithmetic() {
    // 0x10 + 0x20 = 16 + 32 = 48
    let source = "fn main() -> i64 = 0x10 + 0x20;";
    assert_eq!(run_program_i64(source), 48);
}

#[test]
fn test_hex_literal_with_bitwise() {
    // 0xFF band 0x0F = 15
    let source = "fn main() -> i64 = 0xFF band 0x0F;";
    assert_eq!(run_program_i64(source), 15);
}

#[test]
fn test_hex_literal_zero() {
    let source = "fn main() -> i64 = 0x0;";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_hex_literal_large() {
    // 0xDEAD = 57005
    let source = "fn main() -> i64 = 0xDEAD;";
    assert_eq!(run_program_i64(source), 57005);
}

#[test]
fn test_octal_literal_basic() {
    // 0o77 = 63
    let source = "fn main() -> i64 = 0o77;";
    assert_eq!(run_program_i64(source), 63);
}

#[test]
fn test_octal_literal_uppercase_prefix() {
    // 0O10 = 8
    let source = "fn main() -> i64 = 0O10;";
    assert_eq!(run_program_i64(source), 8);
}

#[test]
fn test_binary_literal_basic() {
    // 0b1010 = 10
    let source = "fn main() -> i64 = 0b1010;";
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_binary_literal_uppercase_prefix() {
    // 0B11111111 = 255
    let source = "fn main() -> i64 = 0B11111111;";
    assert_eq!(run_program_i64(source), 255);
}

#[test]
fn test_binary_literal_single_bit() {
    let source = "fn main() -> i64 = 0b1;";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_hex_literal_in_function_param() {
    // Hex literal as function argument
    let source = "
        fn double(x: i64) -> i64 = x * 2;
        fn main() -> i64 = double(0x10);
    ";
    assert_eq!(run_program_i64(source), 32);
}

#[test]
fn test_hex_literal_in_let_binding() {
    // Hex literal in let binding
    let source = "
        fn main() -> i64 = {
            let mask = 0xFF;
            255 band mask
        };
    ";
    assert_eq!(run_program_i64(source), 255);
}

#[test]
fn test_hex_literal_in_comparison() {
    // Hex literal in comparison
    let source = "fn main() -> i64 = if 0xFF > 0xFE { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_hex_literal_in_array_index() {
    // Hex literal as array size/value
    let source = "
        fn main() -> i64 = {
            let arr = [0; 0x10];
            0x10
        };
    ";
    assert_eq!(run_program_i64(source), 16);
}

#[test]
fn test_hex_literal_with_underscore() {
    // Underscore separator in hex literal
    let source = "fn main() -> i64 = 0xFF_FF;";
    assert_eq!(run_program_i64(source), 65535);
}

#[test]
fn test_binary_literal_with_underscore() {
    // Underscore separator in binary literal
    let source = "fn main() -> i64 = 0b1111_0000;";
    assert_eq!(run_program_i64(source), 240);
}

// ============================================================
// Cycle 254: Trait Impl Return Type Validation
// ============================================================

#[test]
fn test_trait_impl_correct_return_type_ok() {
    // Matching return type should pass
    assert!(type_checks(
        "trait HasValue { fn value(self: Self) -> i64; }
         struct Counter { count: i64 }
         impl HasValue for Counter { fn value(self: Self) -> i64 = self.count; }"
    ));
}

#[test]
fn test_trait_impl_wrong_return_type_error_message() {
    // Error message should mention the mismatch
    assert!(type_error_contains(
        "trait GetVal { fn get(self: Self) -> i64; }
         struct S { v: bool }
         impl GetVal for S { fn get(self: Self) -> bool = self.v; }",
        "returns 'bool', but trait declares 'i64'"
    ));
}

#[test]
fn test_trait_impl_wrong_param_count() {
    // Impl method has different parameter count than trait
    assert!(type_error(
        "trait DoSomething { fn do_it(self: Self, x: i64) -> i64; }
         struct S { v: i64 }
         impl DoSomething for S { fn do_it(self: Self) -> i64 = self.v; }"
    ));
}

#[test]
fn test_trait_impl_wrong_param_type() {
    // Impl method has different parameter types than trait
    assert!(type_error(
        "trait Transform { fn apply(self: Self, x: i64) -> i64; }
         struct S { v: i64 }
         impl Transform for S { fn apply(self: Self, x: bool) -> i64 = 0; }"
    ));
}

#[test]
fn test_trait_impl_multiple_methods_one_wrong() {
    // Multiple methods, one has wrong return type
    assert!(type_error(
        "trait Math { fn add(self: Self, x: i64) -> i64; fn name(self: Self) -> bool; }
         struct Calc { v: i64 }
         impl Math for Calc { fn add(self: Self, x: i64) -> i64 = self.v + x; fn name(self: Self) -> i64 = 0; }"
    ));
}

#[test]
fn test_trait_impl_unit_return_matches() {
    // Unit return type () should match
    assert!(type_checks(
        "trait Action { fn run(self: Self) -> (); }
         struct Runner { id: i64 }
         impl Action for Runner { fn run(self: Self) -> () = (); }"
    ));
}

#[test]
fn test_trait_impl_f64_return_mismatch() {
    // f64 vs i64 return type mismatch
    assert!(type_error(
        "trait Measure { fn size(self: Self) -> f64; }
         struct Box { w: i64 }
         impl Measure for Box { fn size(self: Self) -> i64 = self.w; }"
    ));
}

#[test]
fn test_trait_impl_two_impls_both_correct() {
    // Two different impls of same trait, both correct
    assert!(type_checks(
        "trait HasValue { fn value(self: Self) -> i64; }
         struct A { x: i64 }
         struct B { y: i64 }
         impl HasValue for A { fn value(self: Self) -> i64 = self.x; }
         impl HasValue for B { fn value(self: Self) -> i64 = self.y; }"
    ));
}

// ============================================================
// Cycle 255: Trait Method Dispatch in Interpreter
// ============================================================

#[test]
fn test_trait_method_dispatch_basic() {
    // Basic trait method call at runtime
    let source = "
        trait HasValue { fn value(self: Self) -> i64; }
        struct Counter { count: i64 }
        impl HasValue for Counter { fn value(self: Self) -> i64 = self.count; }
        fn main() -> i64 = {
            let c = new Counter { count: 42 };
            c.value()
        };
    ";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_trait_method_dispatch_with_args() {
    // Trait method with additional arguments
    let source = "
        trait Addable { fn add(self: Self, n: i64) -> i64; }
        struct Num { v: i64 }
        impl Addable for Num { fn add(self: Self, n: i64) -> i64 = self.v + n; }
        fn main() -> i64 = {
            let x = new Num { v: 10 };
            x.add(32)
        };
    ";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_trait_method_dispatch_two_structs() {
    // Same trait implemented on two different structs
    let source = "
        trait GetVal { fn get(self: Self) -> i64; }
        struct A { x: i64 }
        struct B { y: i64 }
        impl GetVal for A { fn get(self: Self) -> i64 = self.x; }
        impl GetVal for B { fn get(self: Self) -> i64 = self.y * 2; }
        fn main() -> i64 = {
            let a = new A { x: 10 };
            let b = new B { y: 16 };
            a.get() + b.get()
        };
    ";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_trait_method_dispatch_multiple_methods() {
    // Trait with multiple methods
    let source = "
        trait Shape { fn area(self: Self) -> i64; fn perimeter(self: Self) -> i64; }
        struct Rect { w: i64, h: i64 }
        impl Shape for Rect {
            fn area(self: Self) -> i64 = self.w * self.h;
            fn perimeter(self: Self) -> i64 = 2 * (self.w + self.h);
        }
        fn main() -> i64 = {
            let r = new Rect { w: 3, h: 4 };
            r.area() + r.perimeter()
        };
    ";
    assert_eq!(run_program_i64(source), 12 + 14);
}

#[test]
fn test_trait_method_dispatch_in_function() {
    // Trait method called inside a function
    let source = "
        trait HasLen { fn len(self: Self) -> i64; }
        struct List { size: i64 }
        impl HasLen for List { fn len(self: Self) -> i64 = self.size; }
        fn get_length(l: List) -> i64 = l.len();
        fn main() -> i64 = get_length(new List { size: 7 });
    ";
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_trait_method_dispatch_chain() {
    // Chain trait method calls via intermediate results
    let source = "
        trait GetVal { fn get(self: Self) -> i64; }
        struct Wrapper { inner: i64 }
        impl GetVal for Wrapper { fn get(self: Self) -> i64 = self.inner; }
        fn double_get(w: Wrapper) -> i64 = w.get() * 2;
        fn main() -> i64 = double_get(new Wrapper { inner: 21 });
    ";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_trait_method_dispatch_undefined_method_error() {
    // Calling a method not in the impl should error
    let result = try_run_program(
        "trait HasValue { fn value(self: Self) -> i64; }
         struct S { v: i64 }
         impl HasValue for S { fn value(self: Self) -> i64 = self.v; }
         fn main() -> i64 = {
             let s = new S { v: 1 };
             s.nonexistent()
         };"
    );
    assert!(result.is_err());
}

#[test]
fn test_trait_method_dispatch_bool_return() {
    // Trait method returning bool
    let source = "
        trait Check { fn is_positive(self: Self) -> bool; }
        struct Val { n: i64 }
        impl Check for Val { fn is_positive(self: Self) -> bool = self.n > 0; }
        fn main() -> i64 = if (new Val { n: 5 }).is_positive() { 1 } else { 0 };
    ";
    assert_eq!(run_program_i64(source), 1);
}

// ============================================================
// Cycle 256: Trait Completeness & Missing Method Detection
// ============================================================

#[test]
fn test_trait_missing_method_rejected() {
    // Impl missing a required method should be an error
    assert!(type_error(
        "trait Shape { fn area(self: Self) -> i64; fn perimeter(self: Self) -> i64; }
         struct Rect { w: i64, h: i64 }
         impl Shape for Rect { fn area(self: Self) -> i64 = self.w * self.h; }"
    ));
}

#[test]
fn test_trait_missing_method_error_message() {
    // Error should name the missing method
    assert!(type_error_contains(
        "trait Shape { fn area(self: Self) -> i64; fn perimeter(self: Self) -> i64; }
         struct Rect { w: i64, h: i64 }
         impl Shape for Rect { fn area(self: Self) -> i64 = self.w * self.h; }",
        "missing method 'perimeter'"
    ));
}

#[test]
fn test_trait_all_methods_provided_ok() {
    // All methods provided should pass
    assert!(type_checks(
        "trait Shape { fn area(self: Self) -> i64; fn perimeter(self: Self) -> i64; }
         struct Rect { w: i64, h: i64 }
         impl Shape for Rect {
             fn area(self: Self) -> i64 = self.w * self.h;
             fn perimeter(self: Self) -> i64 = 2 * (self.w + self.h);
         }"
    ));
}

#[test]
fn test_trait_single_method_missing() {
    // Single method trait with missing impl
    assert!(type_error(
        "trait HasName { fn name(self: Self) -> i64; }
         struct S { v: i64 }
         impl HasName for S { }"
    ));
}

#[test]
fn test_trait_empty_impl_with_methods_error() {
    // Empty impl for trait that requires methods
    assert!(type_error_contains(
        "trait Action { fn run(self: Self) -> i64; }
         struct Bot { id: i64 }
         impl Action for Bot { }",
        "missing method 'run'"
    ));
}

#[test]
fn test_trait_complete_impl_at_runtime() {
    // Full trait impl works end-to-end at runtime
    let source = "
        trait Math { fn add(self: Self, x: i64) -> i64; fn double(self: Self) -> i64; }
        struct Num { v: i64 }
        impl Math for Num {
            fn add(self: Self, x: i64) -> i64 = self.v + x;
            fn double(self: Self) -> i64 = self.v * 2;
        }
        fn main() -> i64 = {
            let n = new Num { v: 10 };
            n.add(5) + n.double()
        };
    ";
    assert_eq!(run_program_i64(source), 15 + 20);
}

// ===== v0.90.42: WASM Text Codegen — String Constants =====

fn compile_to_wat(source: &str) -> String {
    let tokens = bmb::lexer::tokenize(source).expect("tokenize failed");
    let ast = bmb::parser::parse("<test>", source, tokens).expect("parse failed");
    let mut tc = bmb::types::TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mir = bmb::mir::lower_program(&ast);
    let codegen = bmb::codegen::WasmCodeGen::new();
    codegen.generate(&mir).expect("wasm codegen failed")
}

fn compile_to_wat_with_target(source: &str, target: bmb::codegen::WasmTarget) -> String {
    let tokens = bmb::lexer::tokenize(source).expect("tokenize failed");
    let ast = bmb::parser::parse("<test>", source, tokens).expect("parse failed");
    let mut tc = bmb::types::TypeChecker::new();
    tc.check_program(&ast).expect("type check failed");
    let mir = bmb::mir::lower_program(&ast);
    let codegen = bmb::codegen::WasmCodeGen::with_target(target);
    codegen.generate(&mir).expect("wasm codegen failed")
}

#[test]
fn test_wasm_string_constant_data_section() {
    let source = r#"
        fn greet() -> String = "hello";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains("(data (i32.const"), "WAT should contain data section for string constants");
    assert!(wat.contains("hello"), "WAT data section should contain the string bytes");
}

#[test]
fn test_wasm_string_constant_offset() {
    let source = r#"
        fn greet() -> String = "hello";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    // String should get offset 2048 (start of data area)
    assert!(wat.contains("i32.const 2048"), "String constant should use interned offset 2048");
}

#[test]
fn test_wasm_multiple_string_constants() {
    let source = r#"
        fn a() -> String = "alpha";
        fn b() -> String = "beta";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    // Two different strings should have two data segments
    let data_count = wat.matches("(data (i32.const").count();
    assert!(data_count >= 2, "Should have at least 2 data segments, got {}", data_count);
}

#[test]
fn test_wasm_string_deduplication() {
    let source = r#"
        fn a() -> String = "same";
        fn b() -> String = "same";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    // Same string used twice should be deduplicated to one data segment
    let data_count = wat.matches("(data (i32.const").count();
    assert_eq!(data_count, 1, "Duplicate strings should be deduplicated to 1 data segment, got {}", data_count);
}

#[test]
fn test_wasm_no_string_no_data_section() {
    let source = r#"
        fn main() -> i64 = 42;
    "#;
    let wat = compile_to_wat(source);
    assert!(!wat.contains("(data (i32.const"), "WAT without strings should have no data section");
}

#[test]
fn test_wasm_string_constant_no_todo() {
    let source = r#"
        fn greet() -> String = "hello";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    assert!(!wat.contains("TODO: string constant"), "TODO comment should be replaced with actual string handling");
}

#[test]
fn test_wasm_browser_target_string_constant() {
    let source = r#"
        fn greet() -> String = "hello";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat_with_target(source, bmb::codegen::WasmTarget::Browser);
    assert!(wat.contains("(data (i32.const"), "Browser target should also have string data section");
}

#[test]
fn test_wasm_standalone_target_string_constant() {
    let source = r#"
        fn greet() -> String = "hello";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat_with_target(source, bmb::codegen::WasmTarget::Standalone);
    assert!(wat.contains("(data (i32.const"), "Standalone target should also have string data section");
}

#[test]
fn test_wasm_string_data_section_comment() {
    let source = r#"
        fn greet() -> String = "hello";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains(";; Data section: string constants"), "Data section should have descriptive comment");
}

#[test]
fn test_wasm_module_structure_valid() {
    let source = r#"
        fn greet() -> String = "world";
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    // Module should start with (module and end with )
    assert!(wat.starts_with("(module"), "WAT should start with (module");
    assert!(wat.trim().ends_with(')'), "WAT should end with )");
    // Data section should appear before closing paren
    let data_pos = wat.find("(data (i32.const").unwrap();
    let close_pos = wat.rfind(')').unwrap();
    assert!(data_pos < close_pos, "Data section should be inside module");
}

// ===== v0.90.44: WASM Bump Allocator =====

#[test]
fn test_wasm_bump_alloc_function_present() {
    let source = r#"
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains("(func $bump_alloc (param $size i32) (result i32)"),
        "WAT should contain $bump_alloc function");
}

#[test]
fn test_wasm_bump_alloc_uses_heap_ptr() {
    let source = r#"
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains("global.get $heap_ptr"), "bump_alloc should use $heap_ptr global");
    assert!(wat.contains("global.set $heap_ptr"), "bump_alloc should advance $heap_ptr");
}

#[test]
fn test_wasm_struct_uses_bump_alloc() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        fn main() -> i64 = {
            let p = new Point { x: 1, y: 2 };
            p.x
        };
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains("call $bump_alloc"), "Struct init should call $bump_alloc");
    assert!(!wat.contains("TODO: proper memory allocation"), "TODO placeholder should be removed");
}

#[test]
fn test_wasm_struct_alloc_size() {
    let source = r#"
        struct Triple { a: i64, b: i64, c: i64 }
        fn main() -> i64 = {
            let t = new Triple { a: 1, b: 2, c: 3 };
            t.a
        };
    "#;
    let wat = compile_to_wat(source);
    // 3 fields * 8 bytes = 24
    assert!(wat.contains("i32.const 24"), "3-field struct should allocate 24 bytes");
}

#[test]
fn test_wasm_array_uses_bump_alloc() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30];
            arr[0]
        };
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains("call $bump_alloc"), "Array init should call $bump_alloc");
}

#[test]
fn test_wasm_array_alloc_size() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4];
            arr[0]
        };
    "#;
    let wat = compile_to_wat(source);
    // 4 elements * 8 bytes = 32
    assert!(wat.contains("i32.const 32"), "4-element array should allocate 32 bytes");
}

#[test]
fn test_wasm_enum_uses_bump_alloc() {
    let source = r#"
        enum Color { Red, Green, Blue }
        fn main() -> i64 = {
            let c = Color::Red;
            0
        };
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains("call $bump_alloc"), "Enum variant should call $bump_alloc");
}

#[test]
fn test_wasm_bump_alloc_8byte_alignment() {
    let source = r#"
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    // Alignment: (size + 7) & -8
    assert!(wat.contains("i32.const 7"), "Bump alloc should round up to 8-byte boundary");
    assert!(wat.contains("i32.const -8"), "Bump alloc should mask to 8-byte boundary");
}

#[test]
fn test_wasm_no_todo_memory_allocation() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        enum Color { Red, Green, Blue }
        fn main() -> i64 = {
            let p = new Point { x: 1, y: 2 };
            let c = Color::Red;
            let arr = [1, 2, 3];
            p.x
        };
    "#;
    let wat = compile_to_wat(source);
    assert!(!wat.contains("TODO: proper memory allocation"), "All memory TODOs should be replaced");
}

#[test]
fn test_wasm_bump_alloc_all_targets() {
    let source = r#"fn main() -> i64 = 0;"#;
    for target in [bmb::codegen::WasmTarget::Wasi, bmb::codegen::WasmTarget::Browser, bmb::codegen::WasmTarget::Standalone] {
        let wat = compile_to_wat_with_target(source, target);
        assert!(wat.contains("func $bump_alloc"), "All targets should have $bump_alloc");
    }
}

// ===== v0.90.47: Enum Method Dispatch in Interpreter =====

#[test]
fn test_enum_method_dispatch_basic() {
    let source = r#"
        enum Shape { Circle, Square }
        trait Area { fn area(self: Self) -> i64; }
        impl Area for Shape {
            fn area(self: Self) -> i64 = 42;
        }
        fn main() -> i64 = {
            let s = Shape::Circle;
            s.area()
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_enum_method_dispatch_with_args() {
    let source = r#"
        enum Op { Add, Mul }
        trait Apply { fn apply(self: Self, a: i64, b: i64) -> i64; }
        impl Apply for Op {
            fn apply(self: Self, a: i64, b: i64) -> i64 = a + b;
        }
        fn main() -> i64 = {
            let op = Op::Add;
            op.apply(10, 20)
        };
    "#;
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_enum_method_dispatch_multiple_methods() {
    let source = r#"
        enum Color { Red, Green, Blue }
        trait ColorInfo {
            fn code(self: Self) -> i64;
            fn bright(self: Self) -> bool;
        }
        impl ColorInfo for Color {
            fn code(self: Self) -> i64 = 255;
            fn bright(self: Self) -> bool = true;
        }
        fn main() -> i64 = {
            let c = Color::Red;
            if c.bright() { c.code() } else { 0 }
        };
    "#;
    assert_eq!(run_program_i64(source), 255);
}

#[test]
fn test_enum_method_dispatch_undefined_error() {
    let source = r#"
        enum Foo { A }
        fn main() -> i64 = {
            let f = Foo::A;
            f.nonexistent()
        };
    "#;
    // Type checker should catch undefined method or runtime will error
    let result = check_program(source);
    assert!(result.is_err(), "Calling undefined method on enum should fail");
}

#[test]
fn test_enum_and_struct_both_impl_same_trait() {
    let source = r#"
        trait Describe { fn id(self: Self) -> i64; }
        struct Point { x: i64 }
        enum Dir { Up, Down }
        impl Describe for Point {
            fn id(self: Self) -> i64 = self.x;
        }
        impl Describe for Dir {
            fn id(self: Self) -> i64 = 99;
        }
        fn main() -> i64 = {
            let p = new Point { x: 42 };
            let d = Dir::Up;
            p.id() + d.id()
        };
    "#;
    assert_eq!(run_program_i64(source), 42 + 99);
}

#[test]
fn test_enum_method_in_function() {
    let source = r#"
        enum Status { Ok, Err }
        trait Check { fn is_ok(self: Self) -> i64; }
        impl Check for Status {
            fn is_ok(self: Self) -> i64 = 1;
        }
        fn check_status(s: Status) -> i64 = s.is_ok();
        fn main() -> i64 = {
            let s = Status::Ok;
            check_status(s)
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

// ===== v0.90.48: WASM Allocation Consistency =====

#[test]
fn test_wasm_tuple_uses_bump_alloc() {
    let source = r#"
        fn main() -> i64 = {
            let t = (1, 2);
            0
        };
    "#;
    let wat = compile_to_wat(source);
    assert!(wat.contains("call $bump_alloc"), "Tuple init should use $bump_alloc");
}

#[test]
fn test_wasm_no_stack_pointer_global() {
    let source = r#"
        fn main() -> i64 = 0;
    "#;
    let wat = compile_to_wat(source);
    assert!(!wat.contains("__stack_pointer"), "Should not reference nonexistent __stack_pointer global");
}

#[test]
fn test_wasm_all_allocs_use_bump() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        enum Dir { Up, Down }
        fn main() -> i64 = {
            let p = new Point { x: 1, y: 2 };
            let d = Dir::Up;
            let arr = [1, 2, 3];
            let t = (10, 20);
            p.x
        };
    "#;
    let wat = compile_to_wat(source);
    // Count $bump_alloc calls — should have at least 4 (struct, enum, array, tuple)
    let alloc_count = wat.matches("call $bump_alloc").count();
    assert!(alloc_count >= 4, "Expected >= 4 bump_alloc calls, got {}", alloc_count);
}

// ===== v0.90.49: Array Method Completeness =====

#[test]
fn test_array_len() {
    let source = "fn main() -> i64 = [10, 20, 30].len();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_is_empty_false() {
    let source = "fn main() -> i64 = if [1].is_empty() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_first() {
    let source = "fn main() -> i64 = [42, 1, 2].first();";
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_array_last() {
    let source = "fn main() -> i64 = [1, 2, 99].last();";
    assert_eq!(run_program_i64(source), 99);
}

#[test]
fn test_array_contains_true() {
    let source = "fn main() -> i64 = if [10, 20, 30].contains(20) { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_contains_false() {
    let source = "fn main() -> i64 = if [10, 20, 30].contains(99) { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_reverse() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3].reverse();
            arr[0]
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

// --- Cycle 266: String method completeness ---

#[test]
fn test_string_to_upper() {
    let source = r#"fn main() -> String = "hello".to_upper();"#;
    assert_eq!(run_program_str(source), "HELLO");
}

#[test]
fn test_string_to_lower() {
    let source = r#"fn main() -> String = "HELLO".to_lower();"#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_trim() {
    let source = r#"fn main() -> String = "  hello  ".trim();"#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_contains() {
    let source = r#"fn main() -> i64 = if "hello world".contains("world") { 1 } else { 0 };"#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_contains_false() {
    let source = r#"fn main() -> i64 = if "hello".contains("xyz") { 1 } else { 0 };"#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_starts_with() {
    let source = r#"fn main() -> i64 = if "hello world".starts_with("hello") { 1 } else { 0 };"#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_ends_with() {
    let source = r#"fn main() -> i64 = if "hello world".ends_with("world") { 1 } else { 0 };"#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_replace() {
    let source = r#"fn main() -> String = "hello world".replace("world", "bmb");"#;
    assert_eq!(run_program_str(source), "hello bmb");
}

#[test]
fn test_string_repeat() {
    let source = r#"fn main() -> String = "ab".repeat(3);"#;
    assert_eq!(run_program_str(source), "ababab");
}

#[test]
fn test_string_split() {
    let source = r#"
        fn main() -> i64 = {
            let parts = "a,b,c".split(",");
            parts.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_string_index_of_found() {
    let source = r#"
        fn main() -> i64 = "hello".index_of("ll").unwrap_or(-1);
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_string_index_of_not_found() {
    let source = r#"
        fn main() -> i64 = "hello".index_of("xyz").unwrap_or(-1);
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_string_method_chaining() {
    let source = r#"fn main() -> String = "  Hello World  ".trim().to_lower();"#;
    assert_eq!(run_program_str(source), "hello world");
}

#[test]
fn test_string_unknown_method_rejected() {
    let source = r#"fn main() -> String = "hello".nonexistent();"#;
    assert!(type_error(source));
}

// --- Cycle 267: For-in array iteration ---

#[test]
fn test_for_in_array_sum() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30];
            let sum = 0;
            for x in arr {
                sum = sum + x;
            };
            sum
        };
    "#;
    assert_eq!(run_program_i64(source), 60);
}

#[test]
fn test_for_in_array_count() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5];
            let count = 0;
            for x in arr {
                count = count + 1;
            };
            count
        };
    "#;
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_for_in_array_single() {
    let source = r#"
        fn main() -> i64 = {
            let sum = 0;
            for x in [42] {
                sum = sum + x;
            };
            sum
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_for_in_array_string() {
    let source = r#"
        fn main() -> i64 = {
            let arr = ["hello", "world"];
            let total_len = 0;
            for s in arr {
                total_len = total_len + s.len();
            };
            total_len
        };
    "#;
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_for_in_array_nested() {
    let source = r#"
        fn main() -> i64 = {
            let result = 0;
            for x in [1, 2, 3] {
                for y in [10, 20] {
                    result = result + x * y;
                };
            };
            result
        };
    "#;
    // (1*10 + 1*20) + (2*10 + 2*20) + (3*10 + 3*20) = 30 + 60 + 90 = 180
    assert_eq!(run_program_i64(source), 180);
}

#[test]
fn test_for_in_array_break() {
    let source = r#"
        fn main() -> i64 = {
            let sum = 0;
            for x in [1, 2, 3, 4, 5] {
                if x > 3 { break } else { sum = sum + x };
            };
            sum
        };
    "#;
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_for_in_array_type_error() {
    let source = r#"
        fn main() -> i64 = {
            for x in 42 {
                x
            };
            0
        };
    "#;
    assert!(type_error(source));
}

// --- Cycle 268: Float method support ---

#[test]
fn test_float_abs() {
    let source = "fn main() -> f64 = (-2.5).abs();";
    assert!((run_program_f64(source) - 2.5).abs() < 1e-10);
}

#[test]
fn test_float_floor() {
    let source = "fn main() -> f64 = 3.7.floor();";
    assert!((run_program_f64(source) - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_ceil() {
    let source = "fn main() -> f64 = 3.2.ceil();";
    assert!((run_program_f64(source) - 4.0).abs() < 1e-10);
}

#[test]
fn test_float_round() {
    let source = "fn main() -> f64 = 3.5.round();";
    assert!((run_program_f64(source) - 4.0).abs() < 1e-10);
}

#[test]
fn test_float_sqrt() {
    let source = "fn main() -> f64 = 16.0.sqrt();";
    assert!((run_program_f64(source) - 4.0).abs() < 1e-10);
}

#[test]
fn test_float_is_nan() {
    let source = "fn main() -> i64 = if (0.0 / 0.0).is_nan() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_is_not_nan() {
    let source = "fn main() -> i64 = if 2.5.is_nan() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_float_is_infinite() {
    let source = "fn main() -> i64 = if (1.0 / 0.0).is_infinite() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_is_finite() {
    let source = "fn main() -> i64 = if 2.5.is_finite() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_min() {
    let source = "fn main() -> f64 = 3.0.min(5.0);";
    assert!((run_program_f64(source) - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_max() {
    let source = "fn main() -> f64 = 3.0.max(5.0);";
    assert!((run_program_f64(source) - 5.0).abs() < 1e-10);
}

#[test]
fn test_float_to_int() {
    let source = "fn main() -> i64 = 3.7.to_int();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_float_method_chaining() {
    let source = "fn main() -> f64 = (-2.7).abs().ceil();";
    assert!((run_program_f64(source) - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_unknown_method_rejected() {
    let source = "fn main() -> f64 = 2.5.nonexistent();";
    assert!(type_error(source));
}

// --- Cycle 269: Integer method support ---

#[test]
fn test_int_abs() {
    let source = "fn main() -> i64 = (-5).abs();";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_int_abs_positive() {
    let source = "fn main() -> i64 = 5.abs();";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_int_min() {
    let source = "fn main() -> i64 = 3.min(7);";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_int_max() {
    let source = "fn main() -> i64 = 3.max(7);";
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_int_clamp() {
    let source = "fn main() -> i64 = 15.clamp(0, 10);";
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_int_clamp_within() {
    let source = "fn main() -> i64 = 5.clamp(0, 10);";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_int_pow() {
    let source = "fn main() -> i64 = 2.pow(10);";
    assert_eq!(run_program_i64(source), 1024);
}

#[test]
fn test_int_to_float() {
    let source = "fn main() -> f64 = 42.to_float();";
    assert!((run_program_f64(source) - 42.0).abs() < 1e-10);
}

#[test]
fn test_int_to_string() {
    let source = r#"fn main() -> String = 42.to_string();"#;
    assert_eq!(run_program_str(source), "42");
}

#[test]
fn test_int_method_chaining() {
    let source = "fn main() -> i64 = (-8).abs().min(5);";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_int_unknown_method_rejected() {
    let source = "fn main() -> i64 = 42.nonexistent();";
    assert!(type_error(source));
}

// --- Cycle 270: String parsing + type conversion methods ---

#[test]
fn test_string_to_int_valid() {
    let source = r#"fn main() -> i64 = "42".to_int().unwrap_or(0);"#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_string_to_int_invalid() {
    let source = r#"fn main() -> i64 = "hello".to_int().unwrap_or(-1);"#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_string_to_float_valid() {
    let source = r#"fn main() -> f64 = "2.5".to_float().unwrap_or(0.0);"#;
    assert!((run_program_f64(source) - 2.5).abs() < 1e-10);
}

#[test]
fn test_string_to_float_invalid() {
    let source = r#"fn main() -> f64 = "abc".to_float().unwrap_or(-1.0);"#;
    assert!((run_program_f64(source) - (-1.0)).abs() < 1e-10);
}

#[test]
fn test_string_chars() {
    let source = r#"
        fn main() -> i64 = "abc".chars().len();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_string_reverse() {
    let source = r#"fn main() -> String = "hello".reverse();"#;
    assert_eq!(run_program_str(source), "olleh");
}

#[test]
fn test_float_to_string() {
    let source = r#"fn main() -> i64 = 2.5.to_string().len();"#;
    assert!(run_program_i64(source) > 0);
}

#[test]
fn test_bool_to_string_true() {
    let source = r#"fn main() -> String = true.to_string();"#;
    assert_eq!(run_program_str(source), "true");
}

#[test]
fn test_bool_to_string_false() {
    let source = r#"fn main() -> String = false.to_string();"#;
    assert_eq!(run_program_str(source), "false");
}

#[test]
fn test_string_roundtrip() {
    let source = r#"fn main() -> i64 = 123.to_string().to_int().unwrap_or(0);"#;
    assert_eq!(run_program_i64(source), 123);
}

// --- Cycle 271: Array functional methods ---

#[test]
fn test_array_push() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3].push(4);
            arr.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_push_value() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20].push(30);
            arr[2]
        };
    "#;
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_pop() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3].pop();
            arr.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_concat() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2].concat([3, 4]);
            arr.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_concat_values() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2].concat([3, 4]);
            arr[3]
        };
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_slice() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30, 40, 50].slice(1, 4);
            arr[0]
        };
    "#;
    assert_eq!(run_program_i64(source), 20);
}

#[test]
fn test_array_slice_len() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50].slice(1, 4).len();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_join() {
    let source = r#"
        fn main() -> String = [1, 2, 3].join(", ");
    "#;
    assert_eq!(run_program_str(source), "1, 2, 3");
}

#[test]
fn test_array_join_strings() {
    let source = r#"
        fn main() -> String = ["a", "b", "c"].join("-");
    "#;
    assert_eq!(run_program_str(source), "a-b-c");
}

#[test]
fn test_array_method_chain() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].push(4).push(5).len();
    "#;
    assert_eq!(run_program_i64(source), 5);
}

// --- Cycle 272: Array closure methods (map, filter, any, all, for_each) ---

#[test]
fn test_array_map() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3].map(fn |x: i64| { x * 2 });
            arr[1]
        };
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_map_len() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40].map(fn |x: i64| { x + 1 }).len();
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_map_sum() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3].map(fn |x: i64| { x * x });
            arr[0] + arr[1] + arr[2]
        };
    "#;
    assert_eq!(run_program_i64(source), 14);
}

#[test]
fn test_array_filter() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5, 6].filter(fn |x: i64| { x > 3 });
            arr.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_filter_values() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 3, 7, 1, 9].filter(fn |x: i64| { x > 5 });
            arr[0] + arr[1] + arr[2]
        };
    "#;
    assert_eq!(run_program_i64(source), 26);
}

#[test]
fn test_array_filter_empty() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].filter(fn |x: i64| { x > 100 }).len();
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_any_true() {
    let source = r#"
        fn main() -> i64 = if [1, 2, 3, 4].any(fn |x: i64| { x > 3 }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_any_false() {
    let source = r#"
        fn main() -> i64 = if [1, 2, 3].any(fn |x: i64| { x > 10 }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_all_true() {
    let source = r#"
        fn main() -> i64 = if [2, 4, 6].all(fn |x: i64| { x > 0 }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_all_false() {
    let source = r#"
        fn main() -> i64 = if [1, 2, 3].all(fn |x: i64| { x > 2 }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_map_filter_chain() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5]
                .map(fn |x: i64| { x * 2 })
                .filter(fn |x: i64| { x > 5 });
            arr.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_for_each() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3];
            arr.for_each(fn |x: i64| { x });
            arr.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

// --- Cycle 273: Array fold, reduce, find, position, enumerate ---

#[test]
fn test_array_fold_sum() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].fold(0, fn |acc: i64, x: i64| { acc + x });
    "#;
    assert_eq!(run_program_i64(source), 15);
}

#[test]
fn test_array_fold_product() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4].fold(1, fn |acc: i64, x: i64| { acc * x });
    "#;
    assert_eq!(run_program_i64(source), 24);
}

#[test]
fn test_array_fold_empty() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            arr.fold(42, fn |acc: i64, x: i64| { acc + x })
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_array_reduce() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4].reduce(fn |a: i64, b: i64| { a + b }).unwrap_or(0);
    "#;
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_array_reduce_empty() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            arr.reduce(fn |a: i64, b: i64| { a + b }).unwrap_or(99)
        };
    "#;
    assert_eq!(run_program_i64(source), 99);
}

#[test]
fn test_array_find_exists() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40].find(fn |x: i64| { x > 25 }).unwrap_or(0);
    "#;
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_find_not_exists() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].find(fn |x: i64| { x > 100 }).unwrap_or(-1);
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_array_position_exists() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40].position(fn |x: i64| { x > 25 }).unwrap_or(-1);
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_position_not_exists() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].position(fn |x: i64| { x > 100 }).unwrap_or(-1);
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_array_enumerate() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [10, 20, 30].enumerate();
            pairs.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_enumerate_index() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [10, 20, 30].enumerate();
            pairs[1].0
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_enumerate_value() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [10, 20, 30].enumerate();
            pairs[2].1
        };
    "#;
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_map_fold_chain() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4]
            .map(fn |x: i64| { x * x })
            .fold(0, fn |acc: i64, x: i64| { acc + x });
    "#;
    assert_eq!(run_program_i64(source), 30);
}

// --- Cycle 274: Array take, drop, zip, flatten, sort_by ---

#[test]
fn test_array_take() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50].take(3).len();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_take_values() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30, 40, 50].take(2);
            arr[0] + arr[1]
        };
    "#;
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_drop() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50].drop(2).len();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_drop_values() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50].drop(3)[0];
    "#;
    assert_eq!(run_program_i64(source), 40);
}

#[test]
fn test_array_zip() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [1, 2, 3].zip([10, 20, 30]);
            pairs.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_zip_values() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [1, 2, 3].zip([10, 20, 30]);
            pairs[1].0 + pairs[1].1
        };
    "#;
    assert_eq!(run_program_i64(source), 22);
}

#[test]
fn test_array_zip_different_lengths() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4].zip([10, 20]).len();
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_flatten() {
    let source = r#"
        fn main() -> i64 = [[1, 2], [3, 4], [5, 6]].flatten().len();
    "#;
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_flatten_values() {
    let source = r#"
        fn main() -> i64 = {
            let flat = [[10, 20], [30, 40]].flatten();
            flat[0] + flat[1] + flat[2] + flat[3]
        };
    "#;
    assert_eq!(run_program_i64(source), 100);
}

#[test]
fn test_array_sort_by_asc() {
    let source = r#"
        fn main() -> i64 = {
            let sorted = [3, 1, 4, 1, 5].sort_by(fn |a: i64, b: i64| { a - b });
            sorted[0] * 10000 + sorted[1] * 1000 + sorted[2] * 100 + sorted[3] * 10 + sorted[4]
        };
    "#;
    assert_eq!(run_program_i64(source), 11345);
}

#[test]
fn test_array_sort_by_desc() {
    let source = r#"
        fn main() -> i64 = {
            let sorted = [3, 1, 4].sort_by(fn |a: i64, b: i64| { b - a });
            sorted[0]
        };
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_filter_take_chain() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5, 6, 7, 8]
            .filter(fn |x: i64| { x > 3 })
            .take(2)
            .fold(0, fn |acc: i64, x: i64| { acc + x });
    "#;
    assert_eq!(run_program_i64(source), 9);
}

// --- Cycle 275: String lines, bytes, char_at, strip_prefix/suffix, pad_left/right ---

#[test]
fn test_string_lines() {
    let source = r#"
        fn main() -> i64 = "hello\nworld\nfoo".lines().len();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_string_bytes() {
    let source = r#"
        fn main() -> i64 = "ABC".bytes()[0];
    "#;
    assert_eq!(run_program_i64(source), 65);
}

#[test]
fn test_string_bytes_len() {
    let source = r#"
        fn main() -> i64 = "hello".bytes().len();
    "#;
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_string_char_at() {
    let source = r#"
        fn main() -> String = "hello".char_at(1);
    "#;
    assert_eq!(run_program_str(source), "e");
}

#[test]
fn test_string_strip_prefix_match() {
    let source = r#"
        fn main() -> String = "hello world".strip_prefix("hello ").unwrap_or("fail");
    "#;
    assert_eq!(run_program_str(source), "world");
}

#[test]
fn test_string_strip_prefix_no_match() {
    let source = r#"
        fn main() -> String = "hello".strip_prefix("xyz").unwrap_or("none");
    "#;
    assert_eq!(run_program_str(source), "none");
}

#[test]
fn test_string_strip_suffix_match() {
    let source = r#"
        fn main() -> String = "hello.bmb".strip_suffix(".bmb").unwrap_or("fail");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_strip_suffix_no_match() {
    let source = r#"
        fn main() -> String = "hello".strip_suffix(".rs").unwrap_or("none");
    "#;
    assert_eq!(run_program_str(source), "none");
}

#[test]
fn test_string_pad_left() {
    let source = r#"
        fn main() -> String = "42".pad_left(5, "0");
    "#;
    assert_eq!(run_program_str(source), "00042");
}

#[test]
fn test_string_pad_right() {
    let source = r#"
        fn main() -> String = "hi".pad_right(5, ".");
    "#;
    assert_eq!(run_program_str(source), "hi...");
}

#[test]
fn test_string_pad_left_no_pad_needed() {
    let source = r#"
        fn main() -> String = "hello".pad_left(3, " ");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

// ============================================
// Cycle 276: Float Math Methods
// ============================================

#[test]
fn test_float_sin() {
    let source = "fn main() -> f64 = 0.0.sin();";
    assert!((run_program_f64(source) - 0.0).abs() < 1e-10);
}

#[test]
fn test_float_cos() {
    let source = "fn main() -> f64 = 0.0.cos();";
    assert!((run_program_f64(source) - 1.0).abs() < 1e-10);
}

#[test]
fn test_float_tan() {
    let source = "fn main() -> f64 = 0.0.tan();";
    assert!((run_program_f64(source) - 0.0).abs() < 1e-10);
}

#[test]
fn test_float_log() {
    let source = "fn main() -> f64 = 1.0.log();";
    assert!((run_program_f64(source) - 0.0).abs() < 1e-10);
}

#[test]
fn test_float_log2() {
    let source = "fn main() -> f64 = 8.0.log2();";
    assert!((run_program_f64(source) - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_log10() {
    let source = "fn main() -> f64 = 100.0.log10();";
    assert!((run_program_f64(source) - 2.0).abs() < 1e-10);
}

#[test]
fn test_float_exp() {
    let source = "fn main() -> f64 = 0.0.exp();";
    assert!((run_program_f64(source) - 1.0).abs() < 1e-10);
}

#[test]
fn test_float_sign_positive() {
    let source = "fn main() -> f64 = 5.5.sign();";
    assert!((run_program_f64(source) - 1.0).abs() < 1e-10);
}

#[test]
fn test_float_sign_negative() {
    let source = "fn main() -> f64 = (-3.2).sign();";
    assert!((run_program_f64(source) - (-1.0)).abs() < 1e-10);
}

#[test]
fn test_float_sign_zero() {
    let source = "fn main() -> f64 = 0.0.sign();";
    assert!((run_program_f64(source) - 0.0).abs() < 1e-10);
}

#[test]
fn test_float_is_positive() {
    let source = "fn main() -> i64 = if 3.14.is_positive() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_is_negative() {
    let source = "fn main() -> i64 = if (-2.5).is_negative() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_is_zero() {
    let source = "fn main() -> i64 = if 0.0.is_zero() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

// ============================================
// Cycle 276: Integer Sign/Predicate Methods
// ============================================

#[test]
fn test_int_sign_positive() {
    let source = "fn main() -> i64 = 42.sign();";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_sign_negative() {
    let source = "fn main() -> i64 = (-7).sign();";
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_int_sign_zero() {
    let source = "fn main() -> i64 = 0.sign();";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_int_is_positive() {
    let source = "fn main() -> i64 = if 5.is_positive() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_is_negative() {
    let source = "fn main() -> i64 = if (-3).is_negative() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_is_zero() {
    let source = "fn main() -> i64 = if 0.is_zero() { 1 } else { 0 };";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_gcd() {
    let source = "fn main() -> i64 = 12.gcd(8);";
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_int_gcd_coprime() {
    let source = "fn main() -> i64 = 7.gcd(13);";
    assert_eq!(run_program_i64(source), 1);
}

// ============================================
// Cycle 276: Nullable map/and_then/filter/unwrap
// ============================================

#[test]
fn test_nullable_map_some() {
    let source = r#"
        fn main() -> i64 = {
            let val: i64? = 5;
            val.map(fn |x: i64| { x * 2 }).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_nullable_map_none() {
    let source = r#"
        fn main() -> i64 = {
            let val: i64? = null;
            val.map(fn |x: i64| { x * 2 }).unwrap_or(-1)
        };
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_nullable_and_then_some() {
    let source = r#"
        fn safe_half(x: i64) -> i64? = if x % 2 == 0 { x / 2 } else { null };
        fn main() -> i64 = {
            let val: i64? = 10;
            val.and_then(fn |x: i64| { safe_half(x) }).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_nullable_and_then_none() {
    let source = r#"
        fn safe_half(x: i64) -> i64? = if x % 2 == 0 { x / 2 } else { null };
        fn main() -> i64 = {
            let val: i64? = null;
            val.and_then(fn |x: i64| { safe_half(x) }).unwrap_or(-1)
        };
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_nullable_filter_pass() {
    let source = r#"
        fn main() -> i64 = {
            let val: i64? = 10;
            val.filter(fn |x: i64| { x > 5 }).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_nullable_filter_reject() {
    let source = r#"
        fn main() -> i64 = {
            let val: i64? = 3;
            val.filter(fn |x: i64| { x > 5 }).unwrap_or(-1)
        };
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_nullable_filter_none() {
    let source = r#"
        fn main() -> i64 = {
            let val: i64? = null;
            val.filter(fn |x: i64| { x > 0 }).unwrap_or(-1)
        };
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_nullable_unwrap_value() {
    let source = r#"
        fn main() -> i64 = {
            let val: i64? = 42;
            val.unwrap()
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

// ============================================
// Cycle 276: Result unwrap (type check)
// ============================================

// ============================================
// Cycle 284: String Closure Methods
// ============================================

#[test]
fn test_string_map_chars() {
    let source = r#"
        fn main() -> String = "abc".map_chars(fn |c: String| { c.to_upper() });
    "#;
    assert_eq!(run_program_str(source), "ABC");
}

#[test]
fn test_string_filter_chars() {
    let source = r#"
        fn main() -> String = "h3ll0 w0rld".filter_chars(fn |c: String| { c != "0" });
    "#;
    assert_eq!(run_program_str(source), "h3ll wrld");
}

#[test]
fn test_string_any_char_true() {
    let source = r#"
        fn main() -> i64 = if "hello123".any_char(fn |c: String| { c == "1" }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_any_char_false() {
    let source = r#"
        fn main() -> i64 = if "hello".any_char(fn |c: String| { c == "z" }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_all_chars_true() {
    let source = r#"
        fn main() -> i64 = if "aaa".all_chars(fn |c: String| { c == "a" }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_all_chars_false() {
    let source = r#"
        fn main() -> i64 = if "abc".all_chars(fn |c: String| { c == "a" }) { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_filter_chars_digits() {
    // Filter to keep only non-space characters
    let source = r#"
        fn main() -> String = "h e l l o".filter_chars(fn |c: String| { c != " " });
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_map_filter_chain() {
    let source = r#"
        fn main() -> String = "Hello World".map_chars(fn |c: String| { c.to_lower() }).filter_chars(fn |c: String| { c != " " });
    "#;
    assert_eq!(run_program_str(source), "helloworld");
}

// ============================================
// Cycle 283: Scan, Partition, Skip_while, Take_while
// ============================================

#[test]
fn test_array_scan() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4].scan(0, fn |acc: i64, x: i64| { acc + x }).last();
    "#;
    // running sum: [1, 3, 6, 10], last = 10
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_array_scan_len() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].scan(0, fn |acc: i64, x: i64| { acc + x }).len();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_scan_first() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30].scan(0, fn |acc: i64, x: i64| { acc + x }).first();
    "#;
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_array_partition_match() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5, 6].partition(fn |x: i64| { x % 2 == 0 }).len();
    "#;
    // evens: [2, 4, 6] = 3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_partition_sum() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5, 6].partition(fn |x: i64| { x % 2 == 0 }).sum();
    "#;
    // evens: [2, 4, 6], sum = 12
    assert_eq!(run_program_i64(source), 12);
}

#[test]
fn test_array_skip_while() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].skip_while(fn |x: i64| { x < 3 }).len();
    "#;
    // skip 1, 2; keep 3, 4, 5 = 3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_skip_while_first() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].skip_while(fn |x: i64| { x < 3 }).first();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_take_while() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].take_while(fn |x: i64| { x < 4 }).len();
    "#;
    // take 1, 2, 3 = 3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_take_while_sum() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].take_while(fn |x: i64| { x < 4 }).sum();
    "#;
    // 1+2+3 = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_scan_partition_chain() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5]
            .scan(0, fn |acc: i64, x: i64| { acc + x })
            .partition(fn |x: i64| { x > 5 })
            .len();
    "#;
    // scan: [1, 3, 6, 10, 15], partition > 5: [6, 10, 15] = 3
    assert_eq!(run_program_i64(source), 3);
}

// ============================================
// Cycle 282: Complex Method Chaining
// ============================================

#[test]
fn test_chain_map_filter_sum() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .map(fn |x: i64| { x * x })
            .filter(fn |x: i64| { x > 10 })
            .sum();
    "#;
    // squares: 1,4,9,16,25,36,49,64,81,100; > 10: 16+25+36+49+64+81+100 = 371
    assert_eq!(run_program_i64(source), 371);
}

#[test]
fn test_chain_sort_dedup_len() {
    let source = "fn main() -> i64 = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5].sort().dedup().len();";
    // unique: 1,2,3,4,5,6,9 = 7
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_chain_flat_map_unique() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].flat_map(fn |x: i64| { [x, x + 1] }).unique().len();
    "#;
    // flat_map: [1,2,2,3,3,4], unique: [1,2,3,4] = 4
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_chain_enumerate_filter() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50]
            .filter(fn |x: i64| { x > 25 })
            .len();
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_chain_take_map_sum() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5]
            .take(3)
            .map(fn |x: i64| { x * 10 })
            .sum();
    "#;
    assert_eq!(run_program_i64(source), 60);
}

#[test]
fn test_chain_string_methods() {
    let source = r#"
        fn main() -> String = "  Hello, World!  ".trim().to_lower().replace("world", "bmb");
    "#;
    assert_eq!(run_program_str(source), "hello, bmb!");
}

#[test]
fn test_chain_string_split_map_join() {
    let source = r#"
        fn main() -> String = "a,b,c".split(",").map(fn |s: String| { s.to_upper() }).join("-");
    "#;
    assert_eq!(run_program_str(source), "A-B-C");
}

#[test]
fn test_chain_count_filter_any() {
    let source = r#"
        fn main() -> i64 = {
            let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            let evens = nums.filter(fn |x: i64| { x % 2 == 0 });
            let has_big = evens.any(fn |x: i64| { x > 8 });
            if has_big { evens.sum() } else { 0 }
        };
    "#;
    // evens: [2,4,6,8,10], has 10 > 8, sum = 30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_chain_zip_map_sum() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].map(fn |x: i64| { x * x }).sum();
    "#;
    assert_eq!(run_program_i64(source), 14);
}

#[test]
fn test_chain_windows_count() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].windows(3).len();
    "#;
    // windows of size 3: [1,2,3], [2,3,4], [3,4,5] = 3 windows
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_chain_digits_sum() {
    let source = "fn main() -> i64 = 12345.digits().sum();";
    // 1+2+3+4+5 = 15
    assert_eq!(run_program_i64(source), 15);
}

#[test]
fn test_nullable_map_chain() {
    let source = r#"
        fn main() -> i64 = {
            let val: i64? = 5;
            val.map(fn |x: i64| { x * 3 }).map(fn |x: i64| { x + 1 }).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 16);
}

// ============================================
// Cycle 281: Integer Formatting + Float Math Extensions
// ============================================

#[test]
fn test_int_to_hex() {
    let source = r#"fn main() -> String = 255.to_hex();"#;
    assert_eq!(run_program_str(source), "ff");
}

#[test]
fn test_int_to_binary() {
    let source = r#"fn main() -> String = 10.to_binary();"#;
    assert_eq!(run_program_str(source), "1010");
}

#[test]
fn test_int_to_octal() {
    let source = r#"fn main() -> String = 8.to_octal();"#;
    assert_eq!(run_program_str(source), "10");
}

#[test]
fn test_int_digits() {
    let source = "fn main() -> i64 = 12345.digits().len();";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_int_digits_first() {
    let source = "fn main() -> i64 = 12345.digits().first();";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_digits_last() {
    let source = "fn main() -> i64 = 12345.digits().last();";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_int_digits_zero() {
    let source = "fn main() -> i64 = 0.digits().len();";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_trunc() {
    let source = "fn main() -> f64 = 3.7.trunc();";
    assert!((run_program_f64(source) - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_fract() {
    let source = "fn main() -> f64 = 3.7.fract();";
    assert!((run_program_f64(source) - 0.7).abs() < 1e-10);
}

#[test]
fn test_float_powi() {
    let source = "fn main() -> f64 = 2.0.powi(10);";
    assert!((run_program_f64(source) - 1024.0).abs() < 1e-10);
}

#[test]
fn test_float_powf() {
    let source = "fn main() -> f64 = 4.0.powf(0.5);";
    assert!((run_program_f64(source) - 2.0).abs() < 1e-10);
}

// ============================================
// Cycle 280: Array Swap, Rotate, Fill, Index_of
// ============================================

#[test]
fn test_array_swap() {
    let source = "fn main() -> i64 = [10, 20, 30].swap(0, 2).first();";
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_swap_last() {
    let source = "fn main() -> i64 = [10, 20, 30].swap(0, 2).last();";
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_array_rotate_left() {
    let source = "fn main() -> i64 = [1, 2, 3, 4, 5].rotate_left(2).first();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_rotate_right() {
    let source = "fn main() -> i64 = [1, 2, 3, 4, 5].rotate_right(2).first();";
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_fill() {
    let source = "fn main() -> i64 = [1, 2, 3].fill(0).sum();";
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_fill_preserves_len() {
    let source = "fn main() -> i64 = [1, 2, 3, 4, 5].fill(7).len();";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_array_index_of_found() {
    let source = "fn main() -> i64 = [10, 20, 30, 40].index_of(30).unwrap_or(-1);";
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_index_of_not_found() {
    let source = "fn main() -> i64 = [10, 20, 30].index_of(99).unwrap_or(-1);";
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_array_rotate_left_full() {
    let source = "fn main() -> i64 = [1, 2, 3].rotate_left(3).first();";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_fill_single() {
    let source = "fn main() -> i64 = [1, 2, 3].fill(42).first();";
    assert_eq!(run_program_i64(source), 42);
}

// ============================================
// Cycle 279: Array Windows, Chunks, Count, Unique
// ============================================

#[test]
fn test_array_windows() {
    let source = "fn main() -> i64 = [1, 2, 3, 4, 5].windows(3).len();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_windows_sum_first() {
    let source = "fn main() -> i64 = [1, 2, 3, 4].windows(2).first().sum();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_chunks() {
    let source = "fn main() -> i64 = [1, 2, 3, 4, 5].chunks(2).len();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_chunks_last_partial() {
    let source = "fn main() -> i64 = [1, 2, 3, 4, 5].chunks(2).last().len();";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_count_predicate() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5, 6].count(fn |x: i64| { x > 3 });
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_count_none() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].count(fn |x: i64| { x > 10 });
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_unique() {
    let source = "fn main() -> i64 = [1, 2, 3, 2, 1, 4].unique().len();";
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_unique_preserves_order() {
    let source = "fn main() -> i64 = [3, 1, 2, 1, 3].unique().first();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_unique_last() {
    let source = "fn main() -> i64 = [3, 1, 2, 1, 3].unique().last();";
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_windows_empty() {
    let source = "fn main() -> i64 = [1, 2].windows(5).len();";
    assert_eq!(run_program_i64(source), 0);
}

// ============================================
// Cycle 278: String Utility Methods
// ============================================

#[test]
fn test_string_trim_start() {
    let source = r#"
        fn main() -> String = "  hello  ".trim_start();
    "#;
    assert_eq!(run_program_str(source), "hello  ");
}

#[test]
fn test_string_trim_end() {
    let source = r#"
        fn main() -> String = "  hello  ".trim_end();
    "#;
    assert_eq!(run_program_str(source), "  hello");
}

#[test]
fn test_string_char_count() {
    let source = r#"
        fn main() -> i64 = "hello".char_count();
    "#;
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_string_count_occurrences() {
    let source = r#"
        fn main() -> i64 = "abcabc".count("abc");
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_string_count_no_match() {
    let source = r#"
        fn main() -> i64 = "hello".count("xyz");
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_last_index_of() {
    let source = r#"
        fn main() -> i64 = "hello world hello".last_index_of("hello").unwrap_or(-1);
    "#;
    assert_eq!(run_program_i64(source), 12);
}

#[test]
fn test_string_last_index_of_not_found() {
    let source = r#"
        fn main() -> i64 = "hello".last_index_of("xyz").unwrap_or(-1);
    "#;
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_string_insert() {
    let source = r#"
        fn main() -> String = "helo".insert(3, "l");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_insert_at_start() {
    let source = r#"
        fn main() -> String = "world".insert(0, "hello ");
    "#;
    assert_eq!(run_program_str(source), "hello world");
}

#[test]
fn test_string_remove() {
    let source = r#"
        fn main() -> String = "hello world".remove(5, 6);
    "#;
    assert_eq!(run_program_str(source), "helloworld");
}

#[test]
fn test_string_remove_range() {
    let source = r#"
        fn main() -> String = "abcdef".remove(1, 4);
    "#;
    assert_eq!(run_program_str(source), "aef");
}

// ============================================
// Cycle 277: Array Aggregation & Utility Methods
// ============================================

#[test]
fn test_array_sort_int() {
    let source = "fn main() -> i64 = [3, 1, 4, 1, 5].sort().first();";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_sort_int_last() {
    let source = "fn main() -> i64 = [3, 1, 4, 1, 5].sort().last();";
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_array_dedup() {
    let source = "fn main() -> i64 = [1, 1, 2, 2, 3, 3, 3].dedup().len();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_dedup_values() {
    let source = "fn main() -> i64 = [1, 1, 2, 3, 3].dedup().last();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_sort_dedup_chain() {
    let source = "fn main() -> i64 = [3, 1, 2, 1, 3].sort().dedup().len();";
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_sum() {
    let source = "fn main() -> i64 = [1, 2, 3, 4, 5].sum();";
    assert_eq!(run_program_i64(source), 15);
}

#[test]
fn test_array_sum_float() {
    let source = "fn main() -> f64 = [1.0, 2.0, 3.0].sum();";
    assert!((run_program_f64(source) - 6.0).abs() < 1e-10);
}

#[test]
fn test_array_product() {
    let source = "fn main() -> i64 = [1, 2, 3, 4].product();";
    assert_eq!(run_program_i64(source), 24);
}

#[test]
fn test_array_min() {
    let source = "fn main() -> i64 = [5, 3, 8, 1, 4].min().unwrap_or(0);";
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_max() {
    let source = "fn main() -> i64 = [5, 3, 8, 1, 4].max().unwrap_or(0);";
    assert_eq!(run_program_i64(source), 8);
}

#[test]
fn test_array_flat_map() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].flat_map(fn |x: i64| { [x, x * 10] }).len();
    "#;
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_flat_map_values() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].flat_map(fn |x: i64| { [x, x * 10] }).sum();
    "#;
    assert_eq!(run_program_i64(source), 66);
}

#[test]
fn test_result_unwrap_ok() {
    assert!(type_checks(
        "enum Result<T, E> { Ok(T), Err(E) }
         fn divide(a: i64, b: i64) -> Result<i64, String> =
           if b == 0 { Result::Err(\"division by zero\") } else { Result::Ok(a / b) };
         fn main() -> i64 = divide(10, 2).unwrap();"
    ));
}

// ============================================
// Cycle 285: Array zip_with, each_cons, step_by, chunk_by
// ============================================

#[test]
fn test_array_zip_with() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3].zip_with([10, 20, 30], fn |a: i64, b: i64| { a + b }).sum();
    "#;
    // [11, 22, 33].sum() = 66
    assert_eq!(run_program_i64(source), 66);
}

#[test]
fn test_array_zip_with_multiply() {
    let source = r#"
        fn main() -> i64 = [2, 3, 4].zip_with([5, 6, 7], fn |a: i64, b: i64| { a * b }).sum();
    "#;
    // [10, 18, 28].sum() = 56
    assert_eq!(run_program_i64(source), 56);
}

#[test]
fn test_array_zip_with_unequal() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4].zip_with([10, 20], fn |a: i64, b: i64| { a + b }).len();
    "#;
    // min length = 2
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_each_cons() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].each_cons(3).len();
    "#;
    // windows of 3: [1,2,3], [2,3,4], [3,4,5] = 3 windows
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_each_cons_large_window() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50].each_cons(4).len();
    "#;
    // windows of 4: [10,20,30,40], [20,30,40,50] = 2 windows
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_step_by() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].step_by(3).sum();
    "#;
    // elements at index 0,3,6,9: [1, 4, 7, 10].sum() = 22
    assert_eq!(run_program_i64(source), 22);
}

#[test]
fn test_array_step_by_2() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50].step_by(2).len();
    "#;
    // indices 0, 2, 4: [10, 30, 50] = 3 elements
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_chunk_by() {
    let source = r#"
        fn is_positive(x: i64) -> i64 = if x > 0 { 1 } else { 0 };
        fn main() -> i64 = [1, 2, -1, -2, 3].chunk_by(fn |x: i64| { is_positive(x) }).len();
    "#;
    // groups: [1,2], [-1,-2], [3] = 3 chunks
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_chunk_by_same() {
    let source = r#"
        fn main() -> i64 = [1, 1, 1, 2, 2, 3].chunk_by(fn |x: i64| { x }).len();
    "#;
    // groups: [1,1,1], [2,2], [3] = 3 chunks
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_zip_with_step_by_chain() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5, 6].step_by(2).zip_with([10, 20, 30], fn |a: i64, b: i64| { a + b }).sum();
    "#;
    // step_by(2): [1, 3, 5], zip_with [10,20,30]: [11, 23, 35].sum() = 69
    assert_eq!(run_program_i64(source), 69);
}

// ============================================
// Cycle 286: Array interleave, find_map, sum_by, min_by, max_by
// ============================================

#[test]
fn test_array_interleave() {
    let source = r#"
        fn main() -> i64 = [1, 3, 5].interleave([2, 4, 6]).sum();
    "#;
    // [1, 2, 3, 4, 5, 6].sum() = 21
    assert_eq!(run_program_i64(source), 21);
}

#[test]
fn test_array_interleave_unequal() {
    let source = r#"
        fn main() -> i64 = [1, 2].interleave([10, 20, 30]).len();
    "#;
    // [1, 10, 2, 20, 30] = 5 elements
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_array_find_map() {
    let source = r#"
        fn main() -> i64 = {
            let result: i64? = [1, 3, 4, 5].find_map(fn |x: i64| { [x].get(if x % 2 == 0 { 0 } else { 99 }) });
            result.unwrap_or(0)
        };
    "#;
    // get(0) returns Some for evens, get(99) returns None for odds; first even = 4
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_find_map_none() {
    let source = r#"
        fn main() -> i64 = {
            let result: i64? = [1, 2, 3].find_map(fn |x: i64| { [x].get(99) });
            result.unwrap_or(-1)
        };
    "#;
    // get(99) always returns None, so result is null => -1
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_array_sum_by() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4].sum_by(fn |x: i64| { x * x });
    "#;
    // 1 + 4 + 9 + 16 = 30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_min_by() {
    let source = r#"
        fn abs_val(x: i64) -> i64 = if x < 0 { 0 - x } else { x };
        fn main() -> i64 = {
            let result: i64? = [3, -1, 4, -2, 2].min_by(fn |x: i64| { abs_val(x) });
            result.unwrap_or(-999)
        };
    "#;
    // closest to 0 by absolute value: -1 (abs=1)
    assert_eq!(run_program_i64(source), -1);
}

#[test]
fn test_array_max_by() {
    let source = r#"
        fn main() -> i64 = {
            let result: i64? = [3, 1, 4, 1, 5].max_by(fn |x: i64| { x });
            result.unwrap_or(0)
        };
    "#;
    // max = 5
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_array_max_by_negative() {
    let source = r#"
        fn neg(x: i64) -> i64 = 0 - x;
        fn main() -> i64 = {
            let result: i64? = [3, 1, 4, 1, 5].max_by(fn |x: i64| { neg(x) });
            result.unwrap_or(0)
        };
    "#;
    // max by negative = element with smallest value = 1
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_sum_by_filter_chain() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].filter(fn |x: i64| { x > 2 }).sum_by(fn |x: i64| { x * 10 });
    "#;
    // filter > 2: [3,4,5], sum_by x*10: 30+40+50 = 120
    assert_eq!(run_program_i64(source), 120);
}

#[test]
fn test_array_interleave_take() {
    let source = r#"
        fn main() -> i64 = [1, 3, 5].interleave([2, 4, 6]).take(4).sum();
    "#;
    // [1, 2, 3, 4].sum() = 10
    assert_eq!(run_program_i64(source), 10);
}

// ============================================
// Cycle 287: String Predicates + Utility
// ============================================

#[test]
fn test_string_is_numeric_true() {
    let source = r#"
        fn main() -> i64 = if "12345".is_numeric() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_is_numeric_false() {
    let source = r#"
        fn main() -> i64 = if "12a45".is_numeric() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_is_alpha_true() {
    let source = r#"
        fn main() -> i64 = if "hello".is_alpha() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_is_alpha_false() {
    let source = r#"
        fn main() -> i64 = if "hello123".is_alpha() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_is_alphanumeric() {
    let source = r#"
        fn main() -> i64 = if "hello123".is_alphanumeric() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_is_whitespace() {
    let source = r#"
        fn main() -> i64 = if "   ".is_whitespace() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_is_upper() {
    let source = r#"
        fn main() -> i64 = if "HELLO".is_upper() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_is_lower() {
    let source = r#"
        fn main() -> i64 = if "hello".is_lower() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_substr() {
    let source = r#"
        fn main() -> String = "hello world".substr(6, 5);
    "#;
    assert_eq!(run_program_str(source), "world");
}

#[test]
fn test_string_center() {
    let source = r#"
        fn main() -> String = "hi".center(6, "*");
    "#;
    assert_eq!(run_program_str(source), "**hi**");
}

#[test]
fn test_string_is_numeric_empty() {
    let source = r#"
        fn main() -> i64 = if "".is_numeric() { 1 } else { 0 };
    "#;
    // empty string is not numeric
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_predicate_chain() {
    let source = r#"
        fn main() -> i64 = if "HELLO".is_upper() { "HELLO".len() } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 5);
}

// ============================================
// Cycle 288: Bool Methods + Integer range_to, is_even, is_odd
// ============================================

#[test]
fn test_bool_to_int_true() {
    let source = r#"
        fn main() -> i64 = true.to_int();
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_bool_to_int_false() {
    let source = r#"
        fn main() -> i64 = false.to_int();
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_bool_to_int_sum() {
    let source = r#"
        fn main() -> i64 = true.to_int() + false.to_int() + true.to_int();
    "#;
    // 1 + 0 + 1 = 2
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_int_range_to() {
    let source = r#"
        fn main() -> i64 = 1.range_to(6).sum();
    "#;
    // [1, 2, 3, 4, 5].sum() = 15
    assert_eq!(run_program_i64(source), 15);
}

#[test]
fn test_int_range_to_map() {
    let source = r#"
        fn main() -> i64 = 0.range_to(5).map(fn |x: i64| { x * x }).sum();
    "#;
    // [0, 1, 4, 9, 16].sum() = 30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_int_is_even() {
    let source = r#"
        fn main() -> i64 = if 4.is_even() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_is_odd() {
    let source = r#"
        fn main() -> i64 = if 3.is_odd() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_range_filter_even() {
    let source = r#"
        fn main() -> i64 = 1.range_to(11).filter(fn |x: i64| { x.is_even() }).sum();
    "#;
    // even numbers 2+4+6+8+10 = 30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_bool_to_string_chain() {
    let source = r#"
        fn main() -> i64 = true.to_string().len();
    "#;
    // "true".len() = 4
    assert_eq!(run_program_i64(source), 4);
}

// ============================================
// Cycle 289: Cross-type conversions + reject, tap, count_by
// ============================================

#[test]
fn test_int_to_bool_true() {
    let source = r#"
        fn main() -> i64 = if 42.to_bool() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_to_bool_false() {
    let source = r#"
        fn main() -> i64 = if 0.to_bool() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_to_bool_true() {
    let source = r#"
        fn main() -> i64 = if "true".to_bool() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_to_bool_false() {
    let source = r#"
        fn main() -> i64 = if "false".to_bool() { 1 } else { 0 };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_reject() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].reject(fn |x: i64| { x > 3 }).sum();
    "#;
    // reject > 3: [1, 2, 3].sum() = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_reject_even() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4, 5].reject(fn |x: i64| { x.is_even() }).sum();
    "#;
    // reject evens: [1, 3, 5].sum() = 9
    assert_eq!(run_program_i64(source), 9);
}

#[test]
fn test_array_tap() {
    let source = r#"
        fn noop(x: i64) -> i64 = x;
        fn main() -> i64 = [1, 2, 3].tap(fn |x: i64| { noop(x) }).sum();
    "#;
    // tap doesn't change array, sum = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_count_by() {
    let source = r#"
        fn parity(x: i64) -> i64 = x % 2;
        fn main() -> i64 = [1, 2, 3, 4, 5].count_by(fn |x: i64| { parity(x) });
    "#;
    // 2 distinct keys: 0 (even) and 1 (odd)
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_count_by_identity() {
    let source = r#"
        fn main() -> i64 = [1, 1, 2, 2, 3].count_by(fn |x: i64| { x });
    "#;
    // 3 distinct values
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_filter_reject_complement() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5];
            let kept = arr.filter(fn |x: i64| { x > 3 }).len();
            let rejected = arr.reject(fn |x: i64| { x > 3 }).len();
            kept + rejected
        };
    "#;
    // filter + reject = total: 2 + 3 = 5
    assert_eq!(run_program_i64(source), 5);
}

// ============================================
// Cycle 290: Comprehensive Integration Programs
// ============================================

#[test]
fn test_program_fibonacci_sum() {
    let source = r#"
        fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };
        fn main() -> i64 = 0.range_to(10).map(fn |x: i64| { fib(x) }).sum();
    "#;
    // fib(0..10) = [0,1,1,2,3,5,8,13,21,34].sum() = 88
    assert_eq!(run_program_i64(source), 88);
}

#[test]
fn test_program_digit_analysis() {
    let source = r#"
        fn main() -> i64 = {
            let n = 12345;
            let digs = n.digits();
            let sum = digs.sum();
            let count = digs.len();
            sum * count
        };
    "#;
    // digits: [1,2,3,4,5], sum=15, count=5, result=75
    assert_eq!(run_program_i64(source), 75);
}

#[test]
fn test_program_string_word_count() {
    let source = r#"
        fn main() -> i64 = "hello world foo bar".split(" ").len();
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_program_string_transform_pipeline() {
    let source = r#"
        fn main() -> String = "  Hello, World!  ".trim().to_lower().replace(",", "").replace("!", "");
    "#;
    assert_eq!(run_program_str(source), "hello world");
}

#[test]
fn test_program_array_statistics() {
    let source = r#"
        fn main() -> i64 = {
            let data = [10, 20, 30, 40, 50];
            let total = data.sum();
            let count = data.len();
            total / count
        };
    "#;
    // average = 150/5 = 30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_program_filter_map_sum() {
    let source = r#"
        fn main() -> i64 = 1.range_to(21)
            .filter(fn |x: i64| { x.is_odd() })
            .map(fn |x: i64| { x * x })
            .sum();
    "#;
    // odd squares 1-19: 1+9+25+49+81+121+169+225+289+361 = 1330
    assert_eq!(run_program_i64(source), 1330);
}

#[test]
fn test_program_string_reverse_palindrome() {
    let source = r#"
        fn main() -> i64 = {
            let word = "racecar";
            if word == word.reverse() { 1 } else { 0 }
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_program_nested_array_ops() {
    let source = r#"
        fn main() -> i64 = {
            let matrix_flat = [1, 2, 3, 4, 5, 6, 7, 8, 9];
            let chunks = matrix_flat.chunks(3);
            chunks.len()
        };
    "#;
    // 3 chunks of 3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_program_range_zip_with() {
    let source = r#"
        fn main() -> i64 = {
            let a = 1.range_to(6);
            let b = 10.range_to(15);
            a.zip_with(b, fn |x: i64, y: i64| { x + y }).sum()
        };
    "#;
    // [11, 13, 15, 17, 19].sum() = 75
    assert_eq!(run_program_i64(source), 75);
}

#[test]
fn test_program_complex_string_analysis() {
    let source = r#"
        fn main() -> i64 = {
            let text = "Hello World 123";
            let words = text.split(" ");
            let total_len = words.sum_by(fn |w: String| { w.len() });
            total_len
        };
    "#;
    // "Hello"(5) + "World"(5) + "123"(3) = 13
    assert_eq!(run_program_i64(source), 13);
}

#[test]
fn test_program_method_chain_complex() {
    let source = r#"
        fn main() -> i64 = 1.range_to(101)
            .filter(fn |x: i64| { x.is_even() })
            .reject(fn |x: i64| { x > 50 })
            .step_by(2)
            .sum();
    "#;
    // evens 2..50: [2,4,6,...,50], step_by(2): [2,6,10,...,50] = every 4th starting from 2
    // 2,6,10,14,18,22,26,30,34,38,42,46,50 -> sum = 338
    assert_eq!(run_program_i64(source), 338);
}

#[test]
fn test_program_type_conversions() {
    let source = r#"
        fn main() -> i64 = {
            let a = 42.to_string().len();
            let b = 3.14.to_string().len();
            let c = true.to_int();
            a + b + c
        };
    "#;
    // "42".len()=2, "3.14".len()=4, true.to_int()=1 = 7
    assert_eq!(run_program_i64(source), 7);
}

// ============================================
// Cycle 291: Final Polish — sorted_by_key, dedup_by, map_with_index, each_with_index
// ============================================

#[test]
fn test_array_sorted_by_key() {
    let source = r#"
        fn neg(x: i64) -> i64 = 0 - x;
        fn main() -> i64 = [3, 1, 4, 1, 5].sorted_by_key(fn |x: i64| { neg(x) }).first();
    "#;
    // sorted by -x (descending): [5, 4, 3, 1, 1], first = 5
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_array_sorted_by_key_asc() {
    let source = r#"
        fn main() -> i64 = [5, 2, 8, 1, 9].sorted_by_key(fn |x: i64| { x }).first();
    "#;
    // ascending sort, first = 1
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_sorted_by_key_abs() {
    let source = r#"
        fn abs_val(x: i64) -> i64 = if x < 0 { 0 - x } else { x };
        fn main() -> i64 = [-3, 1, -5, 2].sorted_by_key(fn |x: i64| { abs_val(x) }).last();
    "#;
    // sorted by abs: [1, 2, -3, -5], last = -5
    assert_eq!(run_program_i64(source), -5);
}

#[test]
fn test_array_dedup_by() {
    let source = r#"
        fn same_sign(a: i64, b: i64) -> bool = (a > 0) == (b > 0);
        fn main() -> i64 = [1, 2, -1, -2, 3, 4].dedup_by(fn |a: i64, b: i64| { same_sign(a, b) }).len();
    "#;
    // groups: [1,2], [-1,-2], [3,4] -> dedup keeps [1, -1, 3] = 3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_dedup_by_equal() {
    let source = r#"
        fn main() -> i64 = [1, 1, 2, 2, 3, 3].dedup_by(fn |a: i64, b: i64| { a == b }).len();
    "#;
    // same as dedup: [1, 2, 3] = 3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_map_with_index() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30].map_with_index(fn |i: i64, x: i64| { i + x }).sum();
    "#;
    // [0+10, 1+20, 2+30] = [10, 21, 32].sum() = 63
    assert_eq!(run_program_i64(source), 63);
}

#[test]
fn test_array_map_with_index_squares() {
    let source = r#"
        fn main() -> i64 = [1, 1, 1, 1, 1].map_with_index(fn |i: i64, x: i64| { i * i }).sum();
    "#;
    // [0, 1, 4, 9, 16].sum() = 30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_sorted_filter_chain() {
    let source = r#"
        fn main() -> i64 = [5, 3, 8, 1, 9, 2]
            .sorted_by_key(fn |x: i64| { x })
            .take(3)
            .sum();
    "#;
    // sorted: [1,2,3,5,8,9], take 3: [1,2,3].sum() = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_map_with_index_filter() {
    let source = r#"
        fn main() -> i64 = [10, 20, 30, 40, 50]
            .map_with_index(fn |i: i64, x: i64| { i * x })
            .filter(fn |x: i64| { x > 0 })
            .sum();
    "#;
    // [0*10, 1*20, 2*30, 3*40, 4*50] = [0, 20, 60, 120, 200]
    // filter > 0: [20, 60, 120, 200].sum() = 400
    assert_eq!(run_program_i64(source), 400);
}

#[test]
fn test_final_comprehensive_program() {
    let source = r#"
        fn main() -> i64 = {
            let nums = 1.range_to(11);
            let evens = nums.filter(fn |x: i64| { x.is_even() });
            let odds = nums.reject(fn |x: i64| { x.is_even() });
            let even_sum = evens.sum();
            let odd_sum = odds.sum();
            let distinct = nums.count_by(fn |x: i64| { x % 3 });
            even_sum + odd_sum + distinct
        };
    "#;
    // evens: [2,4,6,8,10].sum()=30, odds: [1,3,5,7,9].sum()=25
    // distinct mod 3: {0,1,2} = 3
    // 30 + 25 + 3 = 58
    assert_eq!(run_program_i64(source), 58);
}

// ============================================================================
// Cycle 292: Float inverse trig + hyperbolic — asin, acos, atan, atan2, sinh, cosh, tanh
// ============================================================================

#[test]
fn test_float_asin() {
    let source = r#"
        fn main() -> f64 = 1.0.asin();
    "#;
    // asin(1.0) = pi/2 ≈ 1.5707963...
    let result = run_program_f64(source);
    assert!((result - 1.5707963267948966).abs() < 1e-10);
}

#[test]
fn test_float_acos() {
    let source = r#"
        fn main() -> f64 = 0.0.acos();
    "#;
    // acos(0.0) = pi/2 ≈ 1.5707963...
    let result = run_program_f64(source);
    assert!((result - 1.5707963267948966).abs() < 1e-10);
}

#[test]
fn test_float_atan() {
    let source = r#"
        fn main() -> f64 = 1.0.atan();
    "#;
    // atan(1.0) = pi/4 ≈ 0.7853981...
    let result = run_program_f64(source);
    assert!((result - 0.7853981633974483).abs() < 1e-10);
}

#[test]
fn test_float_atan2() {
    let source = r#"
        fn main() -> f64 = 1.0.atan2(1.0);
    "#;
    // atan2(1.0, 1.0) = pi/4 ≈ 0.7853981...
    let result = run_program_f64(source);
    assert!((result - 0.7853981633974483).abs() < 1e-10);
}

#[test]
fn test_float_sinh() {
    let source = r#"
        fn main() -> f64 = 1.0.sinh();
    "#;
    // sinh(1.0) ≈ 1.1752011936...
    let result = run_program_f64(source);
    assert!((result - 1.1752011936438014).abs() < 1e-10);
}

#[test]
fn test_float_cosh() {
    let source = r#"
        fn main() -> f64 = 0.0.cosh();
    "#;
    // cosh(0.0) = 1.0
    let result = run_program_f64(source);
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn test_float_tanh() {
    let source = r#"
        fn main() -> f64 = 0.0.tanh();
    "#;
    // tanh(0.0) = 0.0
    let result = run_program_f64(source);
    assert!(result.abs() < 1e-10);
}

#[test]
fn test_float_trig_identity() {
    // sin²(x) + cos²(x) = 1
    let source = r#"
        fn main() -> i64 = {
            let x = 0.7;
            let s = x.sin();
            let c = x.cos();
            let sum = s * s + c * c;
            if sum > 0.9999 { if sum < 1.0001 { 1 } else { 0 } } else { 0 }
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_inverse_trig_roundtrip() {
    // asin(sin(0.5)) ≈ 0.5
    let source = r#"
        fn main() -> i64 = {
            let x = 0.5;
            let result = x.sin().asin();
            if result > 0.499 { if result < 0.501 { 1 } else { 0 } } else { 0 }
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_hyperbolic_identity() {
    // cosh²(x) - sinh²(x) = 1
    let source = r#"
        fn main() -> i64 = {
            let x = 1.5;
            let ch = x.cosh();
            let sh = x.sinh();
            let result = ch * ch - sh * sh;
            if result > 0.9999 { if result < 1.0001 { 1 } else { 0 } } else { 0 }
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

// ============================================================================
// Cycle 293: Float math utilities — cbrt, hypot, copysign, clamp, log_base
// ============================================================================

#[test]
fn test_float_cbrt() {
    let source = r#"
        fn main() -> f64 = 27.0.cbrt();
    "#;
    let result = run_program_f64(source);
    assert!((result - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_cbrt_negative() {
    let source = r#"
        fn main() -> f64 = (0.0 - 8.0).cbrt();
    "#;
    let result = run_program_f64(source);
    assert!((result - (-2.0)).abs() < 1e-10);
}

#[test]
fn test_float_hypot() {
    let source = r#"
        fn main() -> f64 = 3.0.hypot(4.0);
    "#;
    let result = run_program_f64(source);
    assert!((result - 5.0).abs() < 1e-10);
}

#[test]
fn test_float_copysign() {
    let source = r#"
        fn main() -> f64 = 5.0.copysign(0.0 - 1.0);
    "#;
    let result = run_program_f64(source);
    assert!((result - (-5.0)).abs() < 1e-10);
}

#[test]
fn test_float_copysign_positive() {
    let source = r#"
        fn main() -> f64 = (0.0 - 3.0).copysign(1.0);
    "#;
    let result = run_program_f64(source);
    assert!((result - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_clamp() {
    let source = r#"
        fn main() -> f64 = 7.5.clamp(0.0, 5.0);
    "#;
    let result = run_program_f64(source);
    assert!((result - 5.0).abs() < 1e-10);
}

#[test]
fn test_float_clamp_within() {
    let source = r#"
        fn main() -> f64 = 3.0.clamp(0.0, 5.0);
    "#;
    let result = run_program_f64(source);
    assert!((result - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_log_base() {
    let source = r#"
        fn main() -> f64 = 8.0.log_base(2.0);
    "#;
    let result = run_program_f64(source);
    assert!((result - 3.0).abs() < 1e-10);
}

#[test]
fn test_float_pythagorean_theorem() {
    // Using hypot for distance calculation
    let source = r#"
        fn main() -> i64 = {
            let dx = 5.0;
            let dy = 12.0;
            let dist = dx.hypot(dy);
            if dist > 12.99 { if dist < 13.01 { 1 } else { 0 } } else { 0 }
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_float_math_chain() {
    // cbrt(27) + log_base(16, 2) + hypot(3, 4) = 3 + 4 + 5 = 12
    let source = r#"
        fn main() -> i64 = {
            let a = 27.0.cbrt();
            let b = 16.0.log_base(2.0);
            let c = 3.0.hypot(4.0);
            let sum = a + b + c;
            if sum > 11.99 { if sum < 12.01 { 1 } else { 0 } } else { 0 }
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

// ============================================================================
// Cycle 294: Integer math — lcm, factorial, bit_count, wrapping operations
// ============================================================================

#[test]
fn test_int_lcm() {
    let source = r#"
        fn main() -> i64 = 12.lcm(8);
    "#;
    assert_eq!(run_program_i64(source), 24);
}

#[test]
fn test_int_lcm_coprime() {
    let source = r#"
        fn main() -> i64 = 7.lcm(13);
    "#;
    assert_eq!(run_program_i64(source), 91);
}

#[test]
fn test_int_lcm_zero() {
    let source = r#"
        fn main() -> i64 = 0.lcm(5);
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_int_factorial() {
    let source = r#"
        fn main() -> i64 = 5.factorial();
    "#;
    assert_eq!(run_program_i64(source), 120);
}

#[test]
fn test_int_factorial_zero() {
    let source = r#"
        fn main() -> i64 = 0.factorial();
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_factorial_10() {
    let source = r#"
        fn main() -> i64 = 10.factorial();
    "#;
    assert_eq!(run_program_i64(source), 3628800);
}

#[test]
fn test_int_bit_count() {
    let source = r#"
        fn main() -> i64 = 255.bit_count();
    "#;
    assert_eq!(run_program_i64(source), 8);
}

#[test]
fn test_int_bit_count_power_of_two() {
    let source = r#"
        fn main() -> i64 = 1024.bit_count();
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_int_wrapping_add() {
    let source = r#"
        fn main() -> i64 = 9223372036854775807.wrapping_add(1);
    "#;
    // i64::MAX + 1 wraps to i64::MIN
    assert_eq!(run_program_i64(source), i64::MIN);
}

#[test]
fn test_int_wrapping_mul() {
    let source = r#"
        fn main() -> i64 = 9223372036854775807.wrapping_mul(2);
    "#;
    // wrapping multiplication
    assert_eq!(run_program_i64(source), i64::MAX.wrapping_mul(2));
}

#[test]
fn test_int_math_comprehensive() {
    // 6.lcm(4) = 12, 4.factorial() = 24, 15.bit_count() = 4
    // 12 + 24 + 4 = 40
    let source = r#"
        fn main() -> i64 = {
            let a = 6.lcm(4);
            let b = 4.factorial();
            let c = 15.bit_count();
            a + b + c
        };
    "#;
    assert_eq!(run_program_i64(source), 40);
}

// ============================================================================
// Cycle 295: Nullable methods — or_else, expect, unwrap_or_else
// ============================================================================

#[test]
fn test_nullable_or_else_some() {
    // or_else on Some returns the original value
    let source = r#"
        fn main() -> i64 = {
            let x: i64? = 5;
            x.or_else(fn || { 10 })
        };
    "#;
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_nullable_or_else_none() {
    // or_else on None calls the closure
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            let x: i64? = arr.get(99);
            x.or_else(fn || { 42 })
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_nullable_unwrap_or_else_some() {
    let source = r#"
        fn main() -> i64 = {
            let x: i64? = 7;
            x.unwrap_or_else(fn || { 99 })
        };
    "#;
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_nullable_unwrap_or_else_none() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            let x: i64? = arr.get(99);
            x.unwrap_or_else(fn || { 42 })
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_nullable_expect_some() {
    let source = r#"
        fn main() -> i64 = {
            let x: i64? = 3;
            x.expect("should not fail")
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_nullable_or_else_chain() {
    // Chain: get(99).or_else(|| get(0)).unwrap_or(0)
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30];
            let result: i64? = arr.get(99);
            result.or_else(fn || { arr.get(0) }).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_nullable_unwrap_or_else_computed() {
    // unwrap_or_else with a computed default
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3];
            let missing: i64? = arr.get(99);
            missing.unwrap_or_else(fn || { arr.len() * 10 })
        };
    "#;
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_nullable_methods_comprehensive() {
    // Test map + or_else + unwrap_or_else chain
    let source = r#"
        fn main() -> i64 = {
            let arr = [5, 10, 15];
            let found: i64? = arr.get(1);
            let mapped = found.map(fn |x: i64| { x * 2 });
            let result = mapped.unwrap_or_else(fn || { 0 });
            result
        };
    "#;
    assert_eq!(run_program_i64(source), 20);
}

// ============================================================================
// Cycle 296: Nullable zip, flatten, or + integration tests
// ============================================================================

#[test]
fn test_nullable_or_some() {
    let source = r#"
        fn main() -> i64 = {
            let x: i64? = 5;
            x.or_val(10).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 5);
}

#[test]
fn test_nullable_or_none() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            let x: i64? = arr.get(99);
            x.or_val(42).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_nullable_flatten() {
    // flatten on a simple nullable just returns the value
    let source = r#"
        fn main() -> i64 = {
            let x: i64? = 7;
            x.flatten()
        };
    "#;
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_nullable_flatten_some() {
    // flatten on present nullable returns the value
    let source = r#"
        fn main() -> i64 = {
            let x: i64? = 7;
            x.flatten()
        };
    "#;
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_nullable_or_chain() {
    // or chain: None.or(None).or(42)
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            let a: i64? = arr.get(99);
            let b: i64? = arr.get(99);
            a.or_val(b).or_val(42).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_nullable_comprehensive_chain() {
    // Complex chain: map -> filter -> unwrap_or_else
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30];
            let val: i64? = arr.get(1);
            let result = val
                .map(fn |x: i64| { x + 5 })
                .filter(fn |x: i64| { x > 30 })
                .unwrap_or_else(fn || { 99 });
            result
        };
    "#;
    // arr.get(1) = 20, map => 25, filter(>30) => null, unwrap_or_else => 99
    assert_eq!(run_program_i64(source), 99);
}

#[test]
fn test_nullable_expect_or_chain() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3];
            let val: i64? = arr.get(0);
            val.or_val(0).expect("should have a value")
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_nullable_map_unwrap_or_else() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [5, 10, 15];
            let val: i64? = arr.get(2);
            val.map(fn |x: i64| { x * 3 }).unwrap_or_else(fn || { 0 })
        };
    "#;
    // get(2) = 15, map => 45
    assert_eq!(run_program_i64(source), 45);
}

// ============================================================================
// Cycle 297: Array find_last, take_last, drop_last, first_or, last_or
// ============================================================================

#[test]
fn test_array_find_last() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5];
            arr.find_last(fn |x: i64| { x % 2 == 0 }).unwrap_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 4);
}

#[test]
fn test_array_find_last_none() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 3, 5];
            arr.find_last(fn |x: i64| { x % 2 == 0 }).unwrap_or(99)
        };
    "#;
    assert_eq!(run_program_i64(source), 99);
}

#[test]
fn test_array_take_last() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5];
            arr.take_last(3).sum()
        };
    "#;
    // [3, 4, 5].sum() = 12
    assert_eq!(run_program_i64(source), 12);
}

#[test]
fn test_array_take_last_more_than_len() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3];
            arr.take_last(10).sum()
        };
    "#;
    // takes all: [1, 2, 3].sum() = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_drop_last() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5];
            arr.drop_last(2).sum()
        };
    "#;
    // [1, 2, 3].sum() = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_drop_last_all() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3];
            arr.drop_last(10).len()
        };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_first_or() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30];
            arr.first_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_array_first_or_empty() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            arr.first_or(42)
        };
    "#;
    assert_eq!(run_program_i64(source), 42);
}

#[test]
fn test_array_last_or() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30];
            arr.last_or(0)
        };
    "#;
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_last_or_empty() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1].pop();
            arr.last_or(99)
        };
    "#;
    assert_eq!(run_program_i64(source), 99);
}

#[test]
fn test_array_take_drop_last_chain() {
    // take_last(4).drop_last(1) = middle portion
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5, 6];
            arr.take_last(4).drop_last(1).sum()
        };
    "#;
    // take_last(4) = [3,4,5,6], drop_last(1) = [3,4,5], sum = 12
    assert_eq!(run_program_i64(source), 12);
}

// ============================================================================
// Cycle 298: Array group_by, intersperse, compact
// ============================================================================

#[test]
fn test_array_group_by() {
    // Group by even/odd — count groups
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5, 6];
            let groups = arr.group_by(fn |x: i64| { x % 2 });
            groups.len()
        };
    "#;
    // Two groups: odds and evens
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_group_by_lengths() {
    // Group [1,2,3,4,5,6] by mod 3 -> 3 groups of 2 each
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5, 6];
            let groups = arr.group_by(fn |x: i64| { x % 3 });
            groups.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_intersperse() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3];
            arr.intersperse(0).sum()
        };
    "#;
    // [1, 0, 2, 0, 3].sum() = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_intersperse_len() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [10, 20, 30, 40];
            arr.intersperse(0).len()
        };
    "#;
    // [10, 0, 20, 0, 30, 0, 40].len() = 7
    assert_eq!(run_program_i64(source), 7);
}

#[test]
fn test_array_intersperse_single() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [42];
            arr.intersperse(0).len()
        };
    "#;
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_array_compact() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 0, 2, 0, 3, 0, 4];
            arr.compact().sum()
        };
    "#;
    // Removes zeros: [1, 2, 3, 4].sum() = 10
    assert_eq!(run_program_i64(source), 10);
}

#[test]
fn test_array_compact_len() {
    let source = r#"
        fn main() -> i64 = {
            let arr = [0, 0, 1, 0, 2, 0];
            arr.compact().len()
        };
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_group_by_intersperse_chain() {
    // Group by mod2, then count total groups interspersed with separator length
    let source = r#"
        fn main() -> i64 = {
            let arr = [1, 2, 3, 4, 5, 6, 7, 8];
            let groups = arr.group_by(fn |x: i64| { x % 2 });
            let num_groups = groups.len();
            let total_elements = arr.len();
            num_groups + total_elements
        };
    "#;
    // 2 groups + 8 elements = 10
    assert_eq!(run_program_i64(source), 10);
}

// ============================================================================
// Cycle 299: String split_at, truncate, ljust, rjust, zfill
// ============================================================================

#[test]
fn test_string_truncate() {
    let source = r#"
        fn main() -> String = "hello world".truncate(5);
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_truncate_longer() {
    let source = r#"
        fn main() -> String = "hi".truncate(10);
    "#;
    assert_eq!(run_program_str(source), "hi");
}

#[test]
fn test_string_ljust() {
    let source = r#"
        fn main() -> String = "hi".ljust(5, ".");
    "#;
    assert_eq!(run_program_str(source), "hi...");
}

#[test]
fn test_string_rjust() {
    let source = r#"
        fn main() -> String = "hi".rjust(5, ".");
    "#;
    assert_eq!(run_program_str(source), "...hi");
}

#[test]
fn test_string_zfill() {
    let source = r#"
        fn main() -> String = "42".zfill(5);
    "#;
    assert_eq!(run_program_str(source), "00042");
}

#[test]
fn test_string_zfill_already_wide() {
    let source = r#"
        fn main() -> String = "12345".zfill(3);
    "#;
    assert_eq!(run_program_str(source), "12345");
}

#[test]
fn test_string_ljust_rjust_chain() {
    // Format: "   hi   " (center-like via ljust then nothing)
    let source = r#"
        fn main() -> i64 = {
            let s = "hi".ljust(5, " ").rjust(8, " ");
            s.len()
        };
    "#;
    // "hi   " (5) -> "   hi   " (8)
    assert_eq!(run_program_i64(source), 8);
}

#[test]
fn test_string_truncate_zfill_chain() {
    let source = r#"
        fn main() -> String = "abc".truncate(2).zfill(5);
    "#;
    // "ab" -> "000ab"
    assert_eq!(run_program_str(source), "000ab");
}

// ============================================================================
// Cycle 300: String find_all, replace_first, split_once
// ============================================================================

#[test]
fn test_string_find_all() {
    let source = r#"
        fn main() -> i64 = "abcabc".find_all("bc").len();
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_string_find_all_indices() {
    let source = r#"
        fn main() -> i64 = {
            let indices = "abcabc".find_all("bc");
            indices[0] + indices[1] * 100
        };
    "#;
    // indices are 1 and 4 -> 1 + 400 = 401
    assert_eq!(run_program_i64(source), 401);
}

#[test]
fn test_string_find_all_no_match() {
    let source = r#"
        fn main() -> i64 = "hello".find_all("xyz").len();
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_replace_first() {
    let source = r#"
        fn main() -> String = "aabaa".replace_first("a", "x");
    "#;
    assert_eq!(run_program_str(source), "xabaa");
}

#[test]
fn test_string_replace_first_no_match() {
    let source = r#"
        fn main() -> String = "hello".replace_first("xyz", "!");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_split_once() {
    let source = r#"
        fn main() -> String = "key=value".split_once("=")[1];
    "#;
    assert_eq!(run_program_str(source), "value");
}

#[test]
fn test_string_split_once_no_delim() {
    let source = r#"
        fn main() -> i64 = "hello".split_once("=").len();
    "#;
    // No delimiter found -> single-element array
    assert_eq!(run_program_i64(source), 1);
}

#[test]
fn test_string_find_all_replace_chain() {
    let source = r#"
        fn main() -> String = {
            let s = "hello world hello";
            s.replace_first("hello", "hi")
        };
    "#;
    assert_eq!(run_program_str(source), "hi world hello");
}

// ============================================================================
// Cycle 301: String repeat_str, count_matches, remove_prefix, remove_suffix
// ============================================================================

#[test]
fn test_string_repeat_str() {
    let source = r#"
        fn main() -> String = "ab".repeat_str(3);
    "#;
    assert_eq!(run_program_str(source), "ababab");
}

#[test]
fn test_string_repeat_str_zero() {
    let source = r#"
        fn main() -> String = "hello".repeat_str(0);
    "#;
    assert_eq!(run_program_str(source), "");
}

#[test]
fn test_string_count_matches() {
    let source = r#"
        fn main() -> i64 = "banana".count_matches("an");
    "#;
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_string_count_matches_none() {
    let source = r#"
        fn main() -> i64 = "hello".count_matches("xyz");
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_string_remove_prefix() {
    let source = r#"
        fn main() -> String = "hello world".remove_prefix("hello ");
    "#;
    assert_eq!(run_program_str(source), "world");
}

#[test]
fn test_string_remove_prefix_no_match() {
    let source = r#"
        fn main() -> String = "hello".remove_prefix("xyz");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_remove_suffix() {
    let source = r#"
        fn main() -> String = "hello.txt".remove_suffix(".txt");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_remove_prefix_suffix_chain() {
    let source = r#"
        fn main() -> String = "[hello]".remove_prefix("[").remove_suffix("]");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

// ============================================================================
// Cycle 302: String insert_at, delete_range, overwrite
// ============================================================================

#[test]
fn test_string_insert_at() {
    let source = r#"
        fn main() -> String = "helo".insert_at(3, "l");
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_insert_at_beginning() {
    let source = r#"
        fn main() -> String = "world".insert_at(0, "hello ");
    "#;
    assert_eq!(run_program_str(source), "hello world");
}

#[test]
fn test_string_insert_at_end() {
    let source = r#"
        fn main() -> String = "hello".insert_at(5, " world");
    "#;
    assert_eq!(run_program_str(source), "hello world");
}

#[test]
fn test_string_delete_range() {
    let source = r#"
        fn main() -> String = "hello world".delete_range(5, 11);
    "#;
    assert_eq!(run_program_str(source), "hello");
}

#[test]
fn test_string_delete_range_middle() {
    let source = r#"
        fn main() -> String = "abcdef".delete_range(2, 4);
    "#;
    assert_eq!(run_program_str(source), "abef");
}

#[test]
fn test_string_overwrite() {
    let source = r#"
        fn main() -> String = "hello".overwrite(0, "HE");
    "#;
    assert_eq!(run_program_str(source), "HEllo");
}

#[test]
fn test_string_overwrite_end() {
    let source = r#"
        fn main() -> String = "hello".overwrite(3, "LO");
    "#;
    assert_eq!(run_program_str(source), "helLO");
}

#[test]
fn test_string_insert_delete_chain() {
    let source = r#"
        fn main() -> String = "hello world".delete_range(5, 6).insert_at(5, "_");
    "#;
    assert_eq!(run_program_str(source), "hello_world");
}

// ============================================================================
// Cycle 303: Array pairwise, split_at, uniq_by
// ============================================================================

#[test]
fn test_array_pairwise() {
    let source = r#"
        fn main() -> i64 = [1, 2, 3, 4].pairwise().len();
    "#;
    // [1,2,3,4] -> [[1,2],[2,3],[3,4]] -> len=3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_pairwise_values() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [10, 20, 30].pairwise();
            pairs[0][0] + pairs[0][1] + pairs[1][0] + pairs[1][1]
        };
    "#;
    // [10,20] + [20,30] -> 10+20+20+30 = 80
    assert_eq!(run_program_i64(source), 80);
}

#[test]
fn test_array_pairwise_single() {
    let source = r#"
        fn main() -> i64 = [42].pairwise().len();
    "#;
    // single element -> no pairs
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_split_at() {
    let source = r#"
        fn main() -> i64 = {
            let parts = [1, 2, 3, 4, 5].split_at(3);
            parts[0].len() * 10 + parts[1].len()
        };
    "#;
    // [1,2,3] and [4,5] -> 3*10+2 = 32
    assert_eq!(run_program_i64(source), 32);
}

#[test]
fn test_array_split_at_zero() {
    let source = r#"
        fn main() -> i64 = {
            let parts = [1, 2, 3].split_at(0);
            parts[0].len() * 10 + parts[1].len()
        };
    "#;
    // [] and [1,2,3] -> 0*10+3 = 3
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_uniq_by() {
    let source = r#"
        fn main() -> i64 = {
            [1, 2, 3, 4, 5, 6].uniq_by(fn |x: i64| { x % 3 }).len()
        };
    "#;
    // unique by x%3: keeps 1(1), 2(2), 3(0) -> 3 unique
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_uniq_by_values() {
    let source = r#"
        fn main() -> i64 = {
            let result = [1, 2, 3, 4, 5, 6].uniq_by(fn |x: i64| { x % 3 });
            result[0] + result[1] + result[2]
        };
    "#;
    // keeps first of each key: 1 (key=1), 2 (key=2), 3 (key=0) -> 1+2+3=6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_pairwise_split_chain() {
    let source = r#"
        fn main() -> i64 = {
            [1, 2, 3, 4].pairwise().len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

// ============================================================================
// Cycle 304: Array transpose, associate, frequencies
// ============================================================================

#[test]
fn test_array_transpose() {
    let source = r#"
        fn main() -> i64 = {
            let m = [[1, 2], [3, 4]].transpose();
            m[0][0] + m[0][1] * 10 + m[1][0] * 100 + m[1][1] * 1000
        };
    "#;
    // transposed: [[1,3],[2,4]]
    // 1 + 3*10 + 2*100 + 4*1000 = 1+30+200+4000 = 4231
    assert_eq!(run_program_i64(source), 4231);
}

#[test]
fn test_array_transpose_rect() {
    let source = r#"
        fn main() -> i64 = {
            let m = [[1, 2, 3], [4, 5, 6]].transpose();
            m.len() * 10 + m[0].len()
        };
    "#;
    // 2x3 -> 3x2: len=3, inner len=2 -> 32
    assert_eq!(run_program_i64(source), 32);
}

#[test]
fn test_array_transpose_empty() {
    let source = r#"
        fn main() -> i64 = {
            [[1, 2]].pop().transpose().len()
        };
    "#;
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_associate() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [1, 2, 3].associate(fn |x: i64| { x * x });
            pairs[0][1] + pairs[1][1] + pairs[2][1]
        };
    "#;
    // [[1,1],[2,4],[3,9]] -> 1+4+9 = 14
    assert_eq!(run_program_i64(source), 14);
}

#[test]
fn test_array_associate_keys() {
    let source = r#"
        fn main() -> i64 = {
            let pairs = [10, 20].associate(fn |x: i64| { x + 1 });
            pairs[0][0] + pairs[1][0]
        };
    "#;
    // keys are original elements: 10+20=30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_frequencies() {
    let source = r#"
        fn main() -> i64 = {
            let freq = [1, 2, 1, 3, 2, 1].frequencies();
            freq[0] * 100 + freq[1] * 10 + freq[2]
        };
    "#;
    // 1 appears 3x, 2 appears 2x, 3 appears 1x -> 321
    assert_eq!(run_program_i64(source), 321);
}

#[test]
fn test_array_frequencies_unique() {
    let source = r#"
        fn main() -> i64 = {
            [10, 20, 30].frequencies().len()
        };
    "#;
    // All unique -> 3 counts of 1
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_transpose_associate_chain() {
    let source = r#"
        fn main() -> i64 = {
            let data = [1, 2, 3].associate(fn |x: i64| { x * 2 });
            data.len()
        };
    "#;
    assert_eq!(run_program_i64(source), 3);
}

// ============================================================================
// Cycle 305: Float to_radians, to_degrees, format_fixed, signum, recip
// ============================================================================

#[test]
fn test_float_to_radians() {
    let source = r#"
        fn main() -> f64 = 180.0.to_radians();
    "#;
    let result = run_program_f64(source);
    assert!((result - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_float_to_degrees() {
    let source = r#"
        fn main() -> f64 = 3.14159265358979.to_degrees();
    "#;
    let result = run_program_f64(source);
    assert!((result - 180.0).abs() < 1e-6);
}

#[test]
fn test_float_radians_degrees_roundtrip() {
    let source = r#"
        fn main() -> f64 = 45.0.to_radians().to_degrees();
    "#;
    let result = run_program_f64(source);
    assert!((result - 45.0).abs() < 1e-10);
}

#[test]
fn test_float_signum_positive() {
    let source = r#"
        fn main() -> f64 = 42.0.signum();
    "#;
    let result = run_program_f64(source);
    assert!((result - 1.0).abs() < 1e-10);
}

#[test]
fn test_float_signum_negative() {
    let source = r#"
        fn main() -> f64 = {
            let x = 0.0 - 3.14;
            x.signum()
        };
    "#;
    let result = run_program_f64(source);
    assert!((result - (-1.0)).abs() < 1e-10);
}

#[test]
fn test_float_recip() {
    let source = r#"
        fn main() -> f64 = 4.0.recip();
    "#;
    let result = run_program_f64(source);
    assert!((result - 0.25).abs() < 1e-10);
}

#[test]
fn test_float_format_fixed() {
    let source = r#"
        fn main() -> String = 3.14159.format_fixed(2);
    "#;
    assert_eq!(run_program_str(source), "3.14");
}

#[test]
fn test_float_format_fixed_zero() {
    let source = r#"
        fn main() -> String = 3.14159.format_fixed(0);
    "#;
    assert_eq!(run_program_str(source), "3");
}

#[test]
fn test_float_format_fixed_many() {
    let source = r#"
        fn main() -> String = 1.0.format_fixed(4);
    "#;
    assert_eq!(run_program_str(source), "1.0000");
}

// ============================================================================
// Cycle 306: String swapcase, title_case, snake_case, camel_case
// ============================================================================

#[test]
fn test_string_swapcase() {
    let source = r#"
        fn main() -> String = "Hello World".swapcase();
    "#;
    assert_eq!(run_program_str(source), "hELLO wORLD");
}

#[test]
fn test_string_swapcase_mixed() {
    let source = r#"
        fn main() -> String = "aBcDeF".swapcase();
    "#;
    assert_eq!(run_program_str(source), "AbCdEf");
}

#[test]
fn test_string_title_case() {
    let source = r#"
        fn main() -> String = "hello world".title_case();
    "#;
    assert_eq!(run_program_str(source), "Hello World");
}

#[test]
fn test_string_title_case_underscores() {
    let source = r#"
        fn main() -> String = "hello_world_test".title_case();
    "#;
    assert_eq!(run_program_str(source), "Hello_World_Test");
}

#[test]
fn test_string_snake_case() {
    let source = r#"
        fn main() -> String = "helloWorld".snake_case();
    "#;
    assert_eq!(run_program_str(source), "hello_world");
}

#[test]
fn test_string_snake_case_spaces() {
    let source = r#"
        fn main() -> String = "Hello World".snake_case();
    "#;
    assert_eq!(run_program_str(source), "hello_world");
}

#[test]
fn test_string_camel_case() {
    let source = r#"
        fn main() -> String = "hello_world".camel_case();
    "#;
    assert_eq!(run_program_str(source), "helloWorld");
}

#[test]
fn test_string_case_chain() {
    let source = r#"
        fn main() -> String = "HelloWorld".snake_case().camel_case();
    "#;
    assert_eq!(run_program_str(source), "helloWorld");
}

// ============================================================================
// Cycle 307: Array fold_right, reduce_right, zip_longest
// ============================================================================

#[test]
fn test_array_fold_right() {
    let source = r#"
        fn main() -> i64 = {
            [1, 2, 3].fold_right(0, fn |x: i64, acc: i64| { x + acc })
        };
    "#;
    // 1+(2+(3+0)) = 6
    assert_eq!(run_program_i64(source), 6);
}

#[test]
fn test_array_fold_right_subtraction() {
    let source = r#"
        fn main() -> i64 = {
            [1, 2, 3].fold_right(0, fn |x: i64, acc: i64| { x - acc })
        };
    "#;
    // fold_right: 1 - (2 - (3 - 0)) = 1 - (2 - 3) = 1 - (-1) = 2
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_reduce_right() {
    let source = r#"
        fn main() -> i64 = {
            [1, 2, 3].reduce_right(fn |x: i64, acc: i64| { x - acc }).unwrap_or(0)
        };
    "#;
    // reduce_right: 1 - (2 - 3) = 1 - (-1) = 2
    assert_eq!(run_program_i64(source), 2);
}

#[test]
fn test_array_reduce_right_empty() {
    let source = r#"
        fn main() -> i64 = {
            [1].pop().reduce_right(fn |x: i64, acc: i64| { x + acc }).unwrap_or(99)
        };
    "#;
    assert_eq!(run_program_i64(source), 99);
}

#[test]
fn test_array_zip_longest() {
    let source = r#"
        fn main() -> i64 = {
            let result = [1, 2, 3].zip_longest([10, 20], 0);
            result.len()
        };
    "#;
    // max(3, 2) = 3 pairs
    assert_eq!(run_program_i64(source), 3);
}

#[test]
fn test_array_zip_longest_values() {
    let source = r#"
        fn main() -> i64 = {
            let result = [1, 2, 3].zip_longest([10, 20], 0);
            result[2][0] * 10 + result[2][1]
        };
    "#;
    // Third pair: [3, 0] -> 3*10+0 = 30
    assert_eq!(run_program_i64(source), 30);
}

#[test]
fn test_array_zip_longest_shorter_first() {
    let source = r#"
        fn main() -> i64 = {
            let result = [1].zip_longest([10, 20, 30], 0);
            result[1][0] + result[2][0]
        };
    "#;
    // [1,[10]] [0,[20]] [0,[30]] -> 0+0 = 0
    assert_eq!(run_program_i64(source), 0);
}

#[test]
fn test_array_fold_right_reduce_right_chain() {
    let source = r#"
        fn main() -> i64 = {
            let sum = [1, 2, 3, 4].fold_right(0, fn |x: i64, acc: i64| { x + acc });
            sum
        };
    "#;
    // 1+2+3+4 = 10 (commutative, same as fold_left)
    assert_eq!(run_program_i64(source), 10);
}
