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

// v0.37: Wrapping arithmetic operators
#[test]
fn test_parse_wrapping_arithmetic() {
    use crate::ast::BinOp;

    let prog = parse_ok("fn add_wrap(a: i64, b: i64) -> i64 = a +% b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::AddWrap);
        } else {
            panic!("Expected Binary expression");
        }
    }

    let prog = parse_ok("fn sub_wrap(a: i64, b: i64) -> i64 = a -% b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::SubWrap);
        } else {
            panic!("Expected Binary expression");
        }
    }

    let prog = parse_ok("fn mul_wrap(a: i64, b: i64) -> i64 = a *% b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::MulWrap);
        } else {
            panic!("Expected Binary expression");
        }
    }
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

// v0.32: if-then-else now uses braced syntax: if cond { then } else { else }
#[test]
fn test_parse_if_then_else() {
    let prog = parse_ok("fn max(a: i64, b: i64) -> i64 = if a > b { a } else { b };");
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

// v0.37: Loop invariant syntax
#[test]
fn test_parse_while_loop_invariant() {
    // Simple test: just check it parses without error
    let source = r#"
        fn test() -> () = {
            let mut x: i64 = 0;
            while x < 10 invariant x >= 0 { { x = x + 1; () } };
            ()
        };
    "#;
    let prog = parse_ok(source);

    // Navigate to the while loop through the nested let structure
    // let x = 0; body is: while ...
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Block(stmts) = &f.body.node {
            // stmts[0] is the let expression
            if let Expr::Let { body, .. } = &stmts[0].node {
                // body is directly the while expression
                if let Expr::While { invariant, .. } = &body.node {
                    assert!(invariant.is_some(), "Expected invariant to be Some");
                } else {
                    panic!("Expected While expression, got {:?}", body.node);
                }
            } else {
                panic!("Expected Let expression");
            }
        } else {
            panic!("Expected Block expression");
        }
    } else {
        panic!("Expected FnDef");
    }
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

// v0.32: if-then-else now uses braced syntax
#[test]
fn test_parse_post_condition() {
    let source = "fn abs(x: i64) -> i64 post ret >= 0 = if x >= 0 { x } else { 0 - x };";
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
// Control Flow (v0.36)
// ============================================

#[test]
fn test_parse_loop() {
    let source = r#"
        fn count() -> i64 = loop { break };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::Loop { .. }));
    }
}

#[test]
fn test_parse_break_continue() {
    let source = r#"
        fn test() -> () = { break; continue };
    "#;
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_parse_return() {
    let source = r#"
        fn early() -> () = return;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::Return { .. }));
    }
}

// ============================================
// Nullable Type Syntax (v0.37)
// ============================================

#[test]
fn test_parse_nullable_type() {
    use crate::ast::Type;
    // v0.37: T? syntax for nullable types
    let source = r#"
        struct Value { x: i64 }
        fn find(x: i64) -> Value? = None;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[1] {
        // T? is Type::Nullable
        if let Type::Nullable(inner) = &f.ret_ty.node {
            assert!(matches!(inner.as_ref(), Type::Named(n) if n == "Value"));
        } else {
            panic!("Expected Nullable type, got {:?}", f.ret_ty.node);
        }
    }
}

#[test]
fn test_parse_nullable_primitive() {
    use crate::ast::Type;
    // v0.37: Primitive nullable types
    let source = r#"
        fn maybe_int() -> i64? = None;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Type::Nullable(inner) = &f.ret_ty.node {
            assert!(matches!(inner.as_ref(), Type::I64));
        } else {
            panic!("Expected Nullable(I64), got {:?}", f.ret_ty.node);
        }
    }
}

#[test]
fn test_parse_nullable_generic() {
    use crate::ast::Type;
    // v0.37: Generic nullable types like Vec<i64>?
    let source = r#"
        fn maybe_list() -> Vec<i64>? = None;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Type::Nullable(inner) = &f.ret_ty.node {
            if let Type::Generic { name, type_args } = inner.as_ref() {
                assert_eq!(name, "Vec");
                assert_eq!(type_args.len(), 1);
                assert!(matches!(type_args[0].as_ref(), Type::I64));
            } else {
                panic!("Expected Generic inner type, got {:?}", inner);
            }
        } else {
            panic!("Expected Nullable, got {:?}", f.ret_ty.node);
        }
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
// Closures (v0.20.0)
// ============================================

#[test]
fn test_parse_closure_single_param() {
    let source = "fn test() -> i64 = fn |x: i64| { x + 1 };";
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Closure { params, ret_ty, body } = &f.body.node {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name.node, "x");
            assert!(params[0].ty.is_some());
            assert!(ret_ty.is_none());
            assert!(matches!(body.node, Expr::Block(_)));
        } else {
            panic!("Expected Closure");
        }
    } else {
        panic!("Expected FnDef");
    }
}

// Closures use fn |params| { body } syntax
#[test]
fn test_parse_closure_empty_params() {
    let source = "fn test() -> i64 = fn || { 42 };";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Closure { params, .. } = &f.body.node {
            assert!(params.is_empty());
        } else {
            panic!("Expected Closure");
        }
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_closure_multi_params() {
    let source = "fn test() -> i64 = fn |x: i64, y: i64| { x + y };";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Closure { params, .. } = &f.body.node {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name.node, "x");
            assert_eq!(params[1].name.node, "y");
        } else {
            panic!("Expected Closure");
        }
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// Bitwise Operators (v0.36)
// ============================================

#[test]
fn test_parse_bitwise_and() {
    let source = r#"
        fn test(a: i64, b: i64) -> i64 = a band b;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::Band));
        } else {
            panic!("Expected Binary expression");
        }
    }
}

#[test]
fn test_parse_bitwise_or() {
    let source = r#"
        fn test(a: i64, b: i64) -> i64 = a bor b;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::Bor));
        } else {
            panic!("Expected Binary expression");
        }
    }
}

#[test]
fn test_parse_bitwise_xor() {
    let source = r#"
        fn test(a: i64, b: i64) -> i64 = a bxor b;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::Bxor));
        } else {
            panic!("Expected Binary expression");
        }
    }
}

#[test]
fn test_parse_bitwise_not() {
    let source = r#"
        fn test(a: i64) -> i64 = bnot a;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Unary { op, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::UnOp::Bnot));
        } else {
            panic!("Expected Unary expression");
        }
    }
}

// ============================================
// Logical Implication (v0.36)
// ============================================

#[test]
fn test_parse_implies() {
    let source = r#"
        fn test(a: bool, b: bool) -> bool = a implies b;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::Implies));
        } else {
            panic!("Expected Binary expression");
        }
    }
}

#[test]
fn test_parse_implies_precedence() {
    // implies has lower precedence than or
    // "a or b implies c" should parse as "(a or b) implies c"
    let source = r#"
        fn test(a: bool, b: bool, c: bool) -> bool = a or b implies c;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, left, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::Implies));
            // Left side should be "a or b"
            if let Expr::Binary { op: inner_op, .. } = &left.node {
                assert!(matches!(inner_op, crate::ast::BinOp::Or));
            } else {
                panic!("Expected Binary (or) expression on left");
            }
        } else {
            panic!("Expected Binary expression");
        }
    }
}

// ============================================
// Quantifiers (v0.37)
// ============================================

#[test]
fn test_parse_quantifiers() {
    // forall x: i64, x >= 0
    let source = r#"
        fn test() -> bool = forall x: i64, x >= 0;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Forall { var, ty, body } = &f.body.node {
            assert_eq!(var.node, "x");
            assert!(matches!(ty.node, crate::ast::Type::I64));
            // body should be x >= 0
            if let Expr::Binary { op, .. } = &body.node {
                assert!(matches!(op, crate::ast::BinOp::Ge));
            } else {
                panic!("Expected Binary expression in forall body");
            }
        } else {
            panic!("Expected Forall expression");
        }
    }

    // exists y: bool, y
    let source = r#"
        fn test() -> bool = exists y: bool, y;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Exists { var, ty, body } = &f.body.node {
            assert_eq!(var.node, "y");
            assert!(matches!(ty.node, crate::ast::Type::Bool));
            // body should be just "y"
            if let Expr::Var(name) = &body.node {
                assert_eq!(name, "y");
            } else {
                panic!("Expected Var expression in exists body");
            }
        } else {
            panic!("Expected Exists expression");
        }
    }
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
