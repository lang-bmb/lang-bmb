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
