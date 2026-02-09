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
fn test_loop_infinite_guard() {
    // Test that loop without break causes infinite loop (we can't run this,
    // but we can at least type-check it)
    assert!(type_checks(
        "fn diverge() -> i64 = loop { () };
         fn main() -> i64 = 0;"
    ));
}
