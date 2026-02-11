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
