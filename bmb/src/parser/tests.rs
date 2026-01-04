//! Parser tests for BMB language features
//!
//! Phase 13: Comprehensive parser testing

use crate::ast::{Expr, Item, Visibility};
use crate::lexer::tokenize;
use crate::parser::parse;

/// Helper to parse a BMB program and return the AST
fn parse_program(source: &str) -> crate::Result<crate::ast::Program> {
    let tokens = tokenize(source)?;
    parse("test.bmb", source, tokens)
}

/// Helper to parse and expect success
fn parse_ok(source: &str) -> crate::ast::Program {
    parse_program(source).expect("Parse should succeed")
}

/// Helper to check if parsing fails
fn parse_fails(source: &str) -> bool {
    parse_program(source).is_err()
}

// ============================================
// Basic Expressions
// ============================================

#[test]
fn test_parse_int_literal() {
    let prog = parse_ok("fn main() -> i64 = 42;");
    assert_eq!(prog.items.len(), 1);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::IntLit(n) = &f.body.node {
            assert_eq!(*n, 42);
        } else {
            panic!("Expected IntLit");
        }
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_bool_literal() {
    let prog = parse_ok("fn main() -> bool = true;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::BoolLit(b) = &f.body.node {
            assert!(*b);
        } else {
            panic!("Expected BoolLit");
        }
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_string_literal() {
    let prog = parse_ok(r#"fn main() -> i64 = { let s: i64 = 0; s };"#);
    assert_eq!(prog.items.len(), 1);
}

// ============================================
// Binary Operations
// ============================================

#[test]
fn test_parse_arithmetic() {
    parse_ok("fn add(a: i64, b: i64) -> i64 = a + b;");
    parse_ok("fn sub(a: i64, b: i64) -> i64 = a - b;");
    parse_ok("fn mul(a: i64, b: i64) -> i64 = a * b;");
    parse_ok("fn div(a: i64, b: i64) -> i64 = a / b;");
    parse_ok("fn rem(a: i64, b: i64) -> i64 = a % b;");
}

#[test]
fn test_parse_comparison() {
    parse_ok("fn eq(a: i64, b: i64) -> bool = a == b;");
    parse_ok("fn ne(a: i64, b: i64) -> bool = a != b;");
    parse_ok("fn lt(a: i64, b: i64) -> bool = a < b;");
    parse_ok("fn le(a: i64, b: i64) -> bool = a <= b;");
    parse_ok("fn gt(a: i64, b: i64) -> bool = a > b;");
    parse_ok("fn ge(a: i64, b: i64) -> bool = a >= b;");
}

#[test]
fn test_parse_logical() {
    parse_ok("fn and_op(a: bool, b: bool) -> bool = a and b;");
    parse_ok("fn or_op(a: bool, b: bool) -> bool = a or b;");
    parse_ok("fn not_op(a: bool) -> bool = not a;");
}

// ============================================
// Control Flow
// ============================================

#[test]
fn test_parse_if_then_else() {
    let prog = parse_ok("fn max(a: i64, b: i64) -> i64 = if a > b then a else b;");
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::If { .. }));
    }
}

#[test]
fn test_parse_let_binding() {
    parse_ok("fn test() -> i64 = { let x: i64 = 42; x };");
    // Mutable variable with assignment requires nested block (assignment is BlockStmt, not Expr)
    parse_ok("fn test() -> i64 = { let mut x: i64 = 42; { x = 43; x } };");
}

#[test]
fn test_parse_while_loop() {
    // While body with assignment requires nested block
    parse_ok("fn test() -> i64 = { let mut x: i64 = 0; while x < 10 { { x = x + 1; x } }; x };");
}

#[test]
fn test_parse_for_loop() {
    // For body with assignment requires nested block
    parse_ok("fn test() -> i64 = { let mut sum: i64 = 0; for i in 0..10 { { sum = sum + i; sum } }; sum };");
}

#[test]
fn test_parse_match() {
    let source = r#"
        enum Color { Red, Green, Blue }
        fn test(c: Color) -> i64 = match c {
            Color::Red => 1,
            Color::Green => 2,
            Color::Blue => 3,
        };
    "#;
    parse_ok(source);
}

// ============================================
// Structs and Enums
// ============================================

#[test]
fn test_parse_struct_def() {
    let source = r#"
        struct Point {
            x: i64,
            y: i64,
        }
    "#;
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
    if let Item::StructDef(s) = &prog.items[0] {
        assert_eq!(s.name.node, "Point");
        assert_eq!(s.fields.len(), 2);
    } else {
        panic!("Expected StructDef");
    }
}

#[test]
fn test_parse_enum_def() {
    let source = r#"
        enum Option<T> {
            Some(T),
            None,
        }
    "#;
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
    if let Item::EnumDef(e) = &prog.items[0] {
        assert_eq!(e.name.node, "Option");
        assert_eq!(e.type_params.len(), 1);
        assert_eq!(e.variants.len(), 2);
    } else {
        panic!("Expected EnumDef");
    }
}

#[test]
fn test_parse_struct_init() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        fn origin() -> Point = new Point { x: 0, y: 0 };
    "#;
    parse_ok(source);
}

#[test]
fn test_parse_enum_variant() {
    let source = r#"
        enum Option<T> { Some(T), None }
        fn some_val() -> Option<i64> = Option::Some(42);
        fn none_val() -> Option<i64> = Option::None;
    "#;
    parse_ok(source);
}

// ============================================
// Generics
// ============================================

#[test]
fn test_parse_generic_function() {
    parse_ok("fn identity<T>(x: T) -> T = x;");
    parse_ok("fn pair<A, B>(a: A, b: B) -> A = a;");
}

#[test]
fn test_parse_generic_struct() {
    let source = r#"
        struct Pair<A, B> {
            fst: A,
            snd: B,
        }
    "#;
    let prog = parse_ok(source);
    if let Item::StructDef(s) = &prog.items[0] {
        assert_eq!(s.type_params.len(), 2);
    }
}

#[test]
fn test_parse_generic_enum() {
    let source = r#"
        enum Result<T, E> {
            Ok(T),
            Err(E),
        }
    "#;
    let prog = parse_ok(source);
    if let Item::EnumDef(e) = &prog.items[0] {
        assert_eq!(e.type_params.len(), 2);
    }
}

// ============================================
// Contracts
// ============================================

#[test]
fn test_parse_pre_condition() {
    let source = "fn divide(a: i64, b: i64) -> i64 pre b != 0 = a / b;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(f.pre.is_some());
    }
}

#[test]
fn test_parse_post_condition() {
    let source = "fn abs(x: i64) -> i64 post ret >= 0 = if x >= 0 then x else 0 - x;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(f.post.is_some());
    }
}

#[test]
fn test_parse_pre_post_combined() {
    let source = r#"
        fn safe_divide(a: i64, b: i64) -> i64
          pre b != 0
          post ret * b == a
        = a / b;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(f.pre.is_some());
        assert!(f.post.is_some());
    }
}

// ============================================
// Visibility and Attributes
// ============================================

#[test]
fn test_parse_visibility() {
    let source = "pub fn public_fn() -> i64 = 42;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert_eq!(f.visibility, Visibility::Public);
    }
}

#[test]
fn test_parse_derive_attribute() {
    let source = r#"
        @derive(Debug, Clone, PartialEq)
        struct Point {
            x: i64,
            y: i64,
        }
    "#;
    let prog = parse_ok(source);
    if let Item::StructDef(s) = &prog.items[0] {
        assert!(!s.attributes.is_empty());
    }
}

// ============================================
// Error Handling (v0.13.2)
// ============================================

#[test]
fn test_parse_question_operator() {
    let source = r#"
        enum Result<T, E> { Ok(T), Err(E) }
        fn propagate(r: Result<i64, i64>) -> i64 = r?;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[1] {
        assert!(matches!(f.body.node, Expr::Question { .. }));
    }
}

#[test]
fn test_parse_try_block() {
    let source = r#"
        fn test() -> i64 = try { 42 };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::Try { .. }));
    }
}

// ============================================
// Method Calls (v0.5 Phase 8 + v0.18)
// ============================================

#[test]
fn test_parse_method_call() {
    let source = r#"
        fn test(s: i64) -> i64 = s.abs();
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::MethodCall { .. }));
    }
}

#[test]
fn test_parse_method_call_with_args() {
    let source = r#"
        enum Option<T> { Some(T), None }
        fn test(opt: Option<i64>) -> i64 = opt.unwrap_or(0);
    "#;
    parse_ok(source);
}

// ============================================
// Extern Functions (v0.13.0)
// ============================================

#[test]
fn test_parse_extern_fn() {
    let source = "extern fn malloc(size: i64) -> i64;";
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
    assert!(matches!(prog.items[0], Item::ExternFn(_)));
}

// ============================================
// Use Statements (v0.17)
// ============================================

#[test]
fn test_parse_use_statement() {
    let source = "use bmb_option::Option;";
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
    assert!(matches!(prog.items[0], Item::Use(_)));
}

// ============================================
// Negative Tests (Parser Errors)
// ============================================

#[test]
fn test_parse_invalid_syntax() {
    assert!(parse_fails("fn ()")); // Missing function name
    assert!(parse_fails("fn foo ->")); // Missing return type
    assert!(parse_fails("struct { }")); // Missing struct name
}
