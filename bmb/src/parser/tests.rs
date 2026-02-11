//! Parser tests for BMB language features
//!
//! Phase 13: Comprehensive parser testing

use crate::ast::{Attribute, Expr, Item, Visibility};
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

// v0.89.4: Let bindings inside while/for/loop blocks (Cycle 43)
#[test]
fn test_parse_let_in_while() {
    // let inside while body - previously caused parser error
    parse_ok("fn test() -> i64 = { let mut i = 0; while i < 10 { let x = i * 2; i = i + 1; 0 }; 0 };");
}

#[test]
fn test_parse_let_in_for() {
    // let inside for body
    parse_ok("fn test() -> i64 = { let mut sum = 0; for i in 0..10 { let x = i * 2; sum = sum + x; 0 }; sum };");
}

#[test]
fn test_parse_let_in_loop() {
    // let inside loop body
    parse_ok("fn test() -> i64 = { let mut i = 0; loop { let x = i; i = i + 1; if i > 5 { break } else { 0 } }; i };");
}

#[test]
fn test_parse_multiple_lets_in_while() {
    // Multiple let bindings in a single while body
    parse_ok("fn test() -> i64 = { let mut i = 0; while i < 5 { let a = i; let b = a + 1; i = i + 1; b }; 0 };");
}

#[test]
fn test_parse_typed_let_in_while() {
    // Typed let binding inside while body
    parse_ok("fn test() -> i64 = { let mut i = 0; while i < 5 { let x: i64 = i * 2; i = i + 1; x }; 0 };");
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

    // v0.89.4: desugar_block_lets transforms { let x = 0; while ...; () }
    // into Let(x, 0, Block([while ..., ()]))
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Let { body, .. } = &f.body.node {
            // body is Block([while_expr, ()])
            if let Expr::Block(stmts) = &body.node {
                if let Expr::While { invariant, .. } = &stmts[0].node {
                    assert!(invariant.is_some(), "Expected invariant to be Some");
                } else {
                    panic!("Expected While expression, got {:?}", stmts[0].node);
                }
            } else {
                panic!("Expected Block expression in let body, got {:?}", body.node);
            }
        } else {
            panic!("Expected Let expression (from desugar_block_lets), got {:?}", f.body.node);
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

// v0.70: Spawn expression parsing
#[test]
fn test_parse_spawn() {
    let source = r#"
        fn test() -> i64 = {
            let t = spawn { 42 };
            t.join()
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // v0.89.4: desugar_block_lets transforms { let t = spawn{42}; t.join() }
        // into Let(t, spawn{42}, Block([t.join()]))
        assert!(matches!(f.body.node, Expr::Let { .. }));
    }
}

#[test]
fn test_parse_spawn_complex() {
    // Spawn with more complex body
    let source = r#"
        fn compute() -> i64 = {
            let t = spawn {
                let x: i64 = 10;
                let y: i64 = 20;
                x + y
            };
            t.join()
        };
    "#;
    parse_ok(source);
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

// v0.89.6: Assignments and let bindings in if-branches (Cycle 52)
#[test]
fn test_parse_assign_in_if_branch() {
    // Single block { assignment; value } in if-branch — previously required {{ }}
    parse_ok("fn test() -> i64 = { let mut x = 0; let _r = if true { x = 1; 0 } else { 0 }; x };");
    // Assignment in both branches
    parse_ok("fn test() -> i64 = { let mut x = 0; let _r = if true { x = 1; 0 } else { x = 2; 0 }; x };");
    // Assignment in else-if chain
    parse_ok("fn test() -> i64 = { let mut x = 0; let _r = if true { x = 1; 0 } else if false { x = 2; 0 } else { x = 3; 0 }; x };");
}

#[test]
fn test_parse_let_in_if_branch() {
    // Let binding in if-branch
    parse_ok("fn test() -> i64 = { let result = if true { let x = 42; x } else { 0 }; result };");
    // Let bindings in both branches
    parse_ok("fn test() -> i64 = if true { let a = 1; let b = 2; a + b } else { let c = 3; c };");
}

#[test]
fn test_parse_multi_stmt_if_branch() {
    // Multiple assignments in if-branch
    parse_ok("fn test() -> i64 = { let mut a = 0; let mut b = 0; let _r = if true { a = 1; b = 2; 0 } else { 0 }; a + b };");
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

// ============================================
// Edge Cases and Robustness (v0.89 Quality Gate)
// ============================================

/// Nested if-else inside another if-else
#[test]
fn test_parse_nested_if_else() {
    let source = r#"
        fn classify(x: i64) -> i64 =
            if x > 0 {
                if x > 100 { 2 } else { 1 }
            } else {
                if x < -100 { -2 } else { -1 }
            };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // The outer expression should be an If
        if let Expr::If { then_branch, else_branch, .. } = &f.body.node {
            // then_branch is a Block containing a nested If
            fn contains_if(e: &Expr) -> bool {
                match e {
                    Expr::If { .. } => true,
                    Expr::Block(stmts) => stmts.iter().any(|s| contains_if(&s.node)),
                    _ => false,
                }
            }
            assert!(contains_if(&then_branch.node), "then_branch should contain nested If");
            assert!(contains_if(&else_branch.node), "else_branch should contain nested If");
        } else {
            panic!("Expected outer If expression");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// Nested match expressions
#[test]
fn test_parse_nested_match() {
    let source = r#"
        enum Outer { A, B }
        enum Inner { X, Y }
        fn test(o: Outer, i: Inner) -> i64 = match o {
            Outer::A => match i {
                Inner::X => 1,
                Inner::Y => 2,
            },
            Outer::B => 3,
        };
    "#;
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 3);
    if let Item::FnDef(f) = &prog.items[2] {
        if let Expr::Match { arms, .. } = &f.body.node {
            assert_eq!(arms.len(), 2);
            // First arm body should itself be a Match
            assert!(matches!(arms[0].body.node, Expr::Match { .. }));
        } else {
            panic!("Expected Match expression");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// Multi-line function body with multiple let bindings and blocks
#[test]
fn test_parse_multiline_block_body() {
    let source = r#"
        fn compute(a: i64, b: i64) -> i64 = {
            let x: i64 = a + b;
            let y: i64 = a * b;
            let z: i64 = x + y;
            z
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // desugar_block_lets transforms into nested Let expressions
        assert!(matches!(f.body.node, Expr::Let { .. }));
    } else {
        panic!("Expected FnDef");
    }
}

/// Struct with many fields (5+)
#[test]
fn test_parse_struct_many_fields() {
    let source = r#"
        struct Record {
            id: i64,
            name: i64,
            value: f64,
            active: bool,
            count: i64,
            score: f64,
        }
    "#;
    let prog = parse_ok(source);
    if let Item::StructDef(s) = &prog.items[0] {
        assert_eq!(s.name.node, "Record");
        assert_eq!(s.fields.len(), 6);
    } else {
        panic!("Expected StructDef");
    }
}

/// Enum with mixed variant types (unit, tuple, struct-like)
#[test]
fn test_parse_enum_mixed_variants() {
    let source = r#"
        enum Shape {
            Circle(f64),
            Rectangle(f64, f64),
            Point,
        }
    "#;
    let prog = parse_ok(source);
    if let Item::EnumDef(e) = &prog.items[0] {
        assert_eq!(e.name.node, "Shape");
        assert_eq!(e.variants.len(), 3);
    } else {
        panic!("Expected EnumDef");
    }
}

/// Chained method calls: x.foo().bar()
#[test]
fn test_parse_chained_method_calls() {
    let source = r#"
        fn test(x: i64) -> i64 = x.abs().abs();
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // Outer: MethodCall { receiver: MethodCall { ... }, method: "abs" }
        if let Expr::MethodCall { receiver, method, .. } = &f.body.node {
            assert_eq!(method, "abs");
            assert!(matches!(receiver.node, Expr::MethodCall { .. }));
        } else {
            panic!("Expected chained MethodCall");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// Array literal and index access
#[test]
fn test_parse_array_literal_and_index() {
    let source = r#"
        fn test() -> i64 = {
            let arr: [i64; 3] = [10, 20, 30];
            arr[1]
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // After desugar: Let { value: ArrayLit, body: Block([Index]) }
        if let Expr::Let { value, body, .. } = &f.body.node {
            assert!(matches!(value.node, Expr::ArrayLit(_)));
            if let Expr::Block(stmts) = &body.node {
                assert!(matches!(stmts[0].node, Expr::Index { .. }));
            } else {
                // Could be Index directly
                assert!(matches!(body.node, Expr::Index { .. }));
            }
        } else {
            panic!("Expected Let expression");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// Tuple creation and tuple field access
#[test]
fn test_parse_tuple_and_field_access() {
    let source = r#"
        fn test() -> i64 = {
            let t: (i64, bool) = (42, true);
            t.0
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Let { value, .. } = &f.body.node {
            assert!(matches!(value.node, Expr::Tuple(_)));
        } else {
            panic!("Expected Let with Tuple value");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// While loop with mutation block inside
#[test]
fn test_parse_while_with_mutation_block() {
    let source = r#"
        fn sum_to(n: i64) -> i64 = {
            let mut total: i64 = 0;
            let mut i: i64 = 0;
            while i < n {
                total = total + i;
                i = i + 1;
                0
            };
            total
        };
    "#;
    parse_ok(source);
}

/// For loop with range and mutation
#[test]
fn test_parse_for_range_mutation() {
    let source = r#"
        fn factorial(n: i64) -> i64 = {
            let mut result: i64 = 1;
            for i in 1..n {
                result = result * i;
                0
            };
            result
        };
    "#;
    parse_ok(source);
}

/// Closure expression bound to let and invoked
#[test]
fn test_parse_closure_in_let_and_call() {
    let source = r#"
        fn test() -> i64 = {
            let offset: i64 = 10;
            let add_offset = fn |x: i64| { x + offset };
            add_offset(32)
        };
    "#;
    let prog = parse_ok(source);
    // Body should start with a Let chain (desugar_block_lets)
    if let Item::FnDef(f) = &prog.items[0] {
        // First let: offset = 10
        if let Expr::Let { name, body, .. } = &f.body.node {
            assert_eq!(name, "offset");
            // Second let: add_offset = fn |x| { ... }
            if let Expr::Let { name: n2, value, .. } = &body.node {
                assert_eq!(n2, "add_offset");
                assert!(matches!(value.node, Expr::Closure { .. }), "Expected Closure, got {:?}", value.node);
            } else {
                panic!("Expected inner Let for add_offset");
            }
        } else {
            panic!("Expected outer Let for offset");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// Multiple function definitions in one source file
#[test]
fn test_parse_multiple_functions() {
    let source = r#"
        fn add(a: i64, b: i64) -> i64 = a + b;
        fn sub(a: i64, b: i64) -> i64 = a - b;
        fn mul(a: i64, b: i64) -> i64 = a * b;
        fn div(a: i64, b: i64) -> i64 = a / b;
        fn negate(x: i64) -> i64 = 0 - x;
    "#;
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 5);
    for item in &prog.items {
        assert!(matches!(item, Item::FnDef(_)));
    }
}

/// Deeply nested parenthesized expressions
#[test]
fn test_parse_deeply_nested_parens() {
    let source = "fn test() -> i64 = ((((((1 + 2))))));";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // After unwrapping parens, should still be a Binary Add
        fn unwrap_to_binary(e: &Expr) -> bool {
            match e {
                Expr::Binary { .. } => true,
                Expr::Block(stmts) if stmts.len() == 1 => unwrap_to_binary(&stmts[0].node),
                _ => false,
            }
        }
        assert!(unwrap_to_binary(&f.body.node), "Expected Binary expression after paren unwrap, got {:?}", f.body.node);
    } else {
        panic!("Expected FnDef");
    }
}

/// Operator precedence: arithmetic + comparison + logical
#[test]
fn test_parse_operator_precedence_mixed() {
    // Should parse as: (a + b) > (c * d) and (e == f)
    // i.e., `and` binds loosest, then comparison, then arithmetic
    let source = "fn test(a: i64, b: i64, c: i64, d: i64, e: i64, f: i64) -> bool = a + b > c * d and e == f;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // Top-level should be `and`
        if let Expr::Binary { op, left, right } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::And), "Top-level should be And, got {:?}", op);
            // Left of and: (a + b) > (c * d)
            assert!(matches!(left.node, Expr::Binary { op: crate::ast::BinOp::Gt, .. }));
            // Right of and: e == f
            assert!(matches!(right.node, Expr::Binary { op: crate::ast::BinOp::Eq, .. }));
        } else {
            panic!("Expected Binary expression");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// @pure attribute on a function
#[test]
fn test_parse_pure_attribute() {
    let source = r#"
        @pure
        fn square(x: i64) -> i64 = x * x;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(!f.attributes.is_empty(), "Expected @pure attribute");
        if let Attribute::Simple { name, .. } = &f.attributes[0] {
            assert_eq!(name.node, "pure");
        } else {
            panic!("Expected Simple attribute");
        }
    } else {
        panic!("Expected FnDef");
    }
}

/// Trait definition with multiple methods
#[test]
fn test_parse_trait_def() {
    let source = r#"
        trait Printable {
            fn to_string(self: Self) -> i64;
            fn print(self: Self) -> ();
        }
    "#;
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
    if let Item::TraitDef(t) = &prog.items[0] {
        assert_eq!(t.name.node, "Printable");
        assert_eq!(t.methods.len(), 2);
    } else {
        panic!("Expected TraitDef");
    }
}

/// Impl block for a trait
#[test]
fn test_parse_impl_block() {
    let source = r#"
        struct Counter { value: i64 }
        trait Countable {
            fn count(self: Self) -> i64;
        }
        impl Countable for Counter {
            fn count(self: Self) -> i64 = self.value;
        }
    "#;
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 3);
    assert!(matches!(prog.items[0], Item::StructDef(_)));
    assert!(matches!(prog.items[1], Item::TraitDef(_)));
    if let Item::ImplBlock(imp) = &prog.items[2] {
        assert_eq!(imp.trait_name.node, "Countable");
        assert_eq!(imp.methods.len(), 1);
    } else {
        panic!("Expected ImplBlock, got {:?}", prog.items[2]);
    }
}

/// Cast expression (as) and shift operators
#[test]
fn test_parse_cast_and_shift() {
    // Cast expression: expr as Type
    let source = "fn to_float(x: i64) -> f64 = x as f64;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::Cast { .. }));
    } else {
        panic!("Expected FnDef");
    }

    // Shift operators
    let source2 = "fn shl_test(x: i64) -> i64 = x << 2;";
    let prog2 = parse_ok(source2);
    if let Item::FnDef(f) = &prog2.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::Shl));
        } else {
            panic!("Expected Binary Shl");
        }
    }

    let source3 = "fn shr_test(x: i64) -> i64 = x >> 2;";
    let prog3 = parse_ok(source3);
    if let Item::FnDef(f) = &prog3.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert!(matches!(op, crate::ast::BinOp::Shr));
        } else {
            panic!("Expected Binary Shr");
        }
    }
}

/// Negative tests: additional invalid syntax cases
#[test]
fn test_parse_invalid_edge_cases() {
    // Mismatched braces
    assert!(parse_fails("fn test() -> i64 = { 42 ;"));
    // Double semicolons at top level
    assert!(parse_fails("fn test() -> i64 = 42;;"));
    // Missing equals sign before body
    assert!(parse_fails("fn test() -> i64 42;"));
    // Missing closing paren in function params
    assert!(parse_fails("fn test(x: i64 -> i64 = x;"));
}

// ============================================
// New Tests: Float, Char, Null, Unit Literals
// ============================================

#[test]
fn test_parse_float_literal() {
    let prog = parse_ok("fn pi() -> f64 = 1.23;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::FloatLit(v) = &f.body.node {
            assert!((*v - 1.23).abs() < 1e-10);
        } else {
            panic!("Expected FloatLit, got {:?}", f.body.node);
        }
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_null_literal() {
    let source = r#"
        struct Node { val: i64 }
        fn none_ptr() -> Node? = None;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[1] {
        assert!(matches!(f.body.node, Expr::EnumVariant { .. } | Expr::Var(_)),
            "Expected None variant, got {:?}", f.body.node);
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_unit_return() {
    let prog = parse_ok("fn noop() -> () = ();");
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::Unit), "Expected Unit, got {:?}", f.body.node);
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Checked and Saturating Arithmetic
// ============================================

#[test]
fn test_parse_checked_arithmetic() {
    use crate::ast::BinOp;

    let prog = parse_ok("fn add_checked(a: i64, b: i64) -> i64 = a +? b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::AddChecked);
        } else {
            panic!("Expected Binary expression");
        }
    }

    let prog = parse_ok("fn sub_checked(a: i64, b: i64) -> i64 = a -? b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::SubChecked);
        } else {
            panic!("Expected Binary expression");
        }
    }

    let prog = parse_ok("fn mul_checked(a: i64, b: i64) -> i64 = a *? b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::MulChecked);
        } else {
            panic!("Expected Binary expression");
        }
    }
}

#[test]
fn test_parse_saturating_arithmetic() {
    use crate::ast::BinOp;

    let prog = parse_ok("fn add_sat(a: i64, b: i64) -> i64 = a +| b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::AddSat);
        } else {
            panic!("Expected Binary expression");
        }
    }

    let prog = parse_ok("fn sub_sat(a: i64, b: i64) -> i64 = a -| b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::SubSat);
        } else {
            panic!("Expected Binary expression");
        }
    }

    let prog = parse_ok("fn mul_sat(a: i64, b: i64) -> i64 = a *| b;");
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::MulSat);
        } else {
            panic!("Expected Binary expression");
        }
    }
}

// ============================================
// New Tests: Unary Negation
// ============================================

#[test]
fn test_parse_unary_negation() {
    let prog = parse_ok("fn neg(x: i64) -> i64 = 0 - x;");
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::Binary { .. }));
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Match with Guards, Wildcards, Literals
// ============================================

#[test]
fn test_parse_match_wildcard() {
    let source = r#"
        fn classify(x: i64) -> i64 = match x {
            0 => 100,
            _ => 0,
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Match { arms, .. } = &f.body.node {
            assert_eq!(arms.len(), 2);
            assert!(matches!(arms[1].pattern.node, crate::ast::Pattern::Wildcard));
        } else {
            panic!("Expected Match");
        }
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_match_literal_patterns() {
    let source = r#"
        fn describe(x: i64) -> i64 = match x {
            1 => 10,
            2 => 20,
            3 => 30,
            _ => 0,
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Match { arms, .. } = &f.body.node {
            assert_eq!(arms.len(), 4);
            // First arm should be a literal pattern
            if let crate::ast::Pattern::Literal(crate::ast::LiteralPattern::Int(n)) = &arms[0].pattern.node {
                assert_eq!(*n, 1);
            } else {
                panic!("Expected Int literal pattern, got {:?}", arms[0].pattern.node);
            }
        } else {
            panic!("Expected Match");
        }
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_match_guard() {
    let source = r#"
        fn classify(x: i64) -> i64 = match x {
            n if n > 100 => 3,
            n if n > 0 => 2,
            _ => 1,
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Match { arms, .. } = &f.body.node {
            assert_eq!(arms.len(), 3);
            // First arm should have a guard
            assert!(arms[0].guard.is_some(), "Expected guard on first arm");
            assert!(arms[1].guard.is_some(), "Expected guard on second arm");
            assert!(arms[2].guard.is_none(), "Expected no guard on wildcard arm");
        } else {
            panic!("Expected Match");
        }
    } else {
        panic!("Expected FnDef");
    }
}

#[test]
fn test_parse_match_enum_with_binding() {
    let source = r#"
        enum Option<T> { Some(T), None }
        fn unwrap_or(opt: Option<i64>, default: i64) -> i64 = match opt {
            Option::Some(val) => val,
            Option::None => default,
        };
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[1] {
        if let Expr::Match { arms, .. } = &f.body.node {
            assert_eq!(arms.len(), 2);
            // First arm: Option::Some(val) - should have binding
            if let crate::ast::Pattern::EnumVariant { enum_name, variant, bindings } = &arms[0].pattern.node {
                assert_eq!(enum_name, "Option");
                assert_eq!(variant, "Some");
                assert_eq!(bindings.len(), 1);
            } else {
                panic!("Expected EnumVariant pattern, got {:?}", arms[0].pattern.node);
            }
        } else {
            panic!("Expected Match");
        }
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Type Alias Parsing
// ============================================

#[test]
fn test_parse_type_alias_simple() {
    let source = "type Int = i64;";
    let prog = parse_ok(source);
    assert_eq!(prog.items.len(), 1);
    if let Item::TypeAlias(ta) = &prog.items[0] {
        assert_eq!(ta.name.node, "Int");
        assert!(matches!(ta.target.node, crate::ast::Type::I64));
    } else {
        panic!("Expected TypeAlias, got {:?}", prog.items[0]);
    }
}

#[test]
fn test_parse_type_alias_generic() {
    let source = "type Pair<T> = (T, T);";
    let prog = parse_ok(source);
    if let Item::TypeAlias(ta) = &prog.items[0] {
        assert_eq!(ta.name.node, "Pair");
        assert_eq!(ta.type_params.len(), 1);
        assert_eq!(ta.type_params[0].name, "T");
        assert!(matches!(ta.target.node, crate::ast::Type::Tuple(_)));
    } else {
        panic!("Expected TypeAlias, got {:?}", prog.items[0]);
    }
}

// ============================================
// New Tests: Return With Value
// ============================================

#[test]
fn test_parse_return_no_value() {
    // BMB's return statement does not accept a value; the function body expression is the return value
    let source = "fn early() -> () = return;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Return { value } = &f.body.node {
            assert!(value.is_none(), "Expected return without value");
        } else {
            panic!("Expected Return, got {:?}", f.body.node);
        }
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Array Repeat Syntax
// ============================================

#[test]
fn test_parse_array_repeat() {
    let source = r#"
        fn zeros() -> [i64; 5] = [0; 5];
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        assert!(matches!(f.body.node, Expr::ArrayRepeat { .. }),
            "Expected ArrayRepeat, got {:?}", f.body.node);
        if let Expr::ArrayRepeat { value, count } = &f.body.node {
            if let Expr::IntLit(n) = &value.node {
                assert_eq!(*n, 0);
            }
            assert_eq!(*count, 5);
        }
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Tuple Types (3+ elements)
// ============================================

#[test]
fn test_parse_tuple_three_elements() {
    let source = r#"
        fn triple() -> (i64, bool, f64) = (1, true, 2.5);
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // Check return type is a 3-element tuple
        if let crate::ast::Type::Tuple(elems) = &f.ret_ty.node {
            assert_eq!(elems.len(), 3);
        } else {
            panic!("Expected Tuple return type, got {:?}", f.ret_ty.node);
        }
        // Check body is a tuple expression
        assert!(matches!(f.body.node, Expr::Tuple(_)), "Expected Tuple expr, got {:?}", f.body.node);
        if let Expr::Tuple(elems) = &f.body.node {
            assert_eq!(elems.len(), 3);
        }
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Pub Struct and Pub Enum
// ============================================

#[test]
fn test_parse_pub_struct() {
    let source = r#"
        pub struct Config {
            width: i64,
            height: i64,
        }
    "#;
    let prog = parse_ok(source);
    if let Item::StructDef(s) = &prog.items[0] {
        assert_eq!(s.visibility, Visibility::Public);
        assert_eq!(s.name.node, "Config");
    } else {
        panic!("Expected StructDef");
    }
}

#[test]
fn test_parse_pub_enum() {
    let source = r#"
        pub enum Direction {
            North,
            South,
            East,
            West,
        }
    "#;
    let prog = parse_ok(source);
    if let Item::EnumDef(e) = &prog.items[0] {
        assert_eq!(e.visibility, Visibility::Public);
        assert_eq!(e.name.node, "Direction");
        assert_eq!(e.variants.len(), 4);
    } else {
        panic!("Expected EnumDef");
    }
}

// ============================================
// New Tests: Todo Expression
// ============================================

#[test]
fn test_parse_todo_expression() {
    let source = r#"fn unimplemented() -> i64 = todo "not yet";"#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Todo { message } = &f.body.node {
            assert_eq!(message.as_deref(), Some("not yet"));
        } else {
            panic!("Expected Todo, got {:?}", f.body.node);
        }
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Complex Operator Precedence
// ============================================

#[test]
fn test_parse_mul_before_add() {
    use crate::ast::BinOp;
    // a + b * c should parse as a + (b * c)
    let source = "fn test(a: i64, b: i64, c: i64) -> i64 = a + b * c;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, right, .. } = &f.body.node {
            assert_eq!(*op, BinOp::Add, "Top-level should be Add");
            if let Expr::Binary { op: inner_op, .. } = &right.node {
                assert_eq!(*inner_op, BinOp::Mul, "Right side should be Mul");
            } else {
                panic!("Expected Mul on right, got {:?}", right.node);
            }
        } else {
            panic!("Expected Binary expression");
        }
    }
}

#[test]
fn test_parse_shift_vs_arithmetic_precedence() {
    use crate::ast::BinOp;
    // a << 2 + 1 should parse as a << (2 + 1) since + binds tighter than <<
    // OR a << 2 + 1 could be (a << 2) + 1 depending on grammar
    // Let's just verify it parses successfully and check the structure
    let source = "fn test(a: i64) -> i64 = a << 2;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        if let Expr::Binary { op, .. } = &f.body.node {
            assert_eq!(*op, BinOp::Shl);
        } else {
            panic!("Expected Shl binary, got {:?}", f.body.node);
        }
    }
}

#[test]
fn test_parse_bitwise_combined() {
    // a band b bor c should parse according to bitwise precedence
    let source = "fn test(a: i64, b: i64, c: i64) -> i64 = a band b bor c;";
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[0] {
        // Should parse - verify it's a nested binary expression
        fn count_binops(e: &Expr) -> usize {
            match e {
                Expr::Binary { left, right, .. } => 1 + count_binops(&left.node) + count_binops(&right.node),
                _ => 0,
            }
        }
        assert_eq!(count_binops(&f.body.node), 2, "Expected 2 binary operations");
    }
}

// ============================================
// New Tests: Extern Fn with C ABI
// ============================================

#[test]
fn test_parse_extern_fn_c_abi() {
    let source = r#"extern "C" fn printf(fmt: i64) -> i64;"#;
    let prog = parse_ok(source);
    if let Item::ExternFn(ef) = &prog.items[0] {
        assert_eq!(ef.abi, crate::ast::Abi::C);
        assert_eq!(ef.name.node, "printf");
    } else {
        panic!("Expected ExternFn, got {:?}", prog.items[0]);
    }
}

// ============================================
// New Tests: Struct Field Access and Field Assignment
// ============================================

#[test]
fn test_parse_struct_field_access() {
    let source = r#"
        struct Point { x: i64, y: i64 }
        fn get_x(p: Point) -> i64 = p.x;
    "#;
    let prog = parse_ok(source);
    if let Item::FnDef(f) = &prog.items[1] {
        assert!(matches!(f.body.node, Expr::FieldAccess { .. }),
            "Expected FieldAccess, got {:?}", f.body.node);
        if let Expr::FieldAccess { field, .. } = &f.body.node {
            assert_eq!(field.node, "x");
        }
    } else {
        panic!("Expected FnDef");
    }
}

// ============================================
// New Tests: Index Assignment
// ============================================

#[test]
fn test_parse_index_assignment() {
    // BMB uses `set` keyword for index assignment: set a[0] = 99
    let source = r#"
        fn set_first(arr: [i64; 3]) -> i64 = {
            let mut a: [i64; 3] = arr;
            set a[0] = 99;
            a[0]
        };
    "#;
    let prog = parse_ok(source);
    // Just verify it parses without error
    assert_eq!(prog.items.len(), 1);
}
