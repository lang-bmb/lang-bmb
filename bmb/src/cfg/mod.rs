//! Conditional Compilation (v0.12.3)
//!
//! This module provides support for @cfg attributes to enable
//! target-specific code compilation.
//!
//! Supported syntax:
//! - `@cfg(target == "wasm32")` - WASM 32-bit target
//! - `@cfg(target == "wasm64")` - WASM 64-bit target (future)
//! - `@cfg(target == "native")` - Native target (LLVM)
//! - `@cfg(not(target == "wasm32"))` - Negation (future)
//! - `@cfg(any(target == "wasm32", target == "wasm64"))` - Disjunction (future)

use crate::ast::{Attribute, Expr, Item, Program};

/// Compilation target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Target {
    /// Native target (x86_64, aarch64, etc.) - uses LLVM
    #[default]
    Native,
    /// WebAssembly 32-bit target
    Wasm32,
    /// WebAssembly 64-bit target (future)
    Wasm64,
}

impl Target {
    /// Parse target from string
    pub fn from_str(s: &str) -> Option<Target> {
        match s.to_lowercase().as_str() {
            "native" | "x86_64" | "aarch64" | "x86" | "arm" => Some(Target::Native),
            "wasm32" | "wasm" | "wasm32-wasi" | "wasm32-unknown" => Some(Target::Wasm32),
            "wasm64" => Some(Target::Wasm64),
            _ => None,
        }
    }

    /// Get target name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Target::Native => "native",
            Target::Wasm32 => "wasm32",
            Target::Wasm64 => "wasm64",
        }
    }
}

/// Configuration evaluator for @cfg attributes
pub struct CfgEvaluator {
    target: Target,
}

impl CfgEvaluator {
    /// Create a new evaluator with the given target
    pub fn new(target: Target) -> Self {
        Self { target }
    }

    /// Filter program items based on @cfg attributes
    pub fn filter_program(&self, program: &Program) -> Program {
        let items = program
            .items
            .iter()
            .filter(|item| self.should_include_item(item))
            .cloned()
            .collect();

        Program { items }
    }

    /// Check if an item should be included for the current target
    pub fn should_include_item(&self, item: &Item) -> bool {
        match item {
            Item::FnDef(f) => self.evaluate_attrs(&f.attributes),
            Item::StructDef(s) => self.evaluate_attrs(&s.attributes),
            Item::EnumDef(e) => self.evaluate_attrs(&e.attributes),
            Item::Use(_) => true, // Use statements are always included
            Item::ExternFn(e) => self.evaluate_attrs(&e.attributes), // v0.13.0
        }
    }

    /// Evaluate @cfg attributes for an item
    /// Returns true if item should be included
    fn evaluate_attrs(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.name() == "cfg" {
                if let Attribute::WithArgs { args, .. } = attr {
                    // Evaluate cfg condition
                    if !self.evaluate_cfg_args(args) {
                        return false;
                    }
                }
                // @cfg without args is invalid, skip
            }
        }
        true // No @cfg or all @cfg passed
    }

    /// Evaluate @cfg arguments
    /// Supports: @cfg(target = "wasm32"), @cfg(target = "native")
    fn evaluate_cfg_args(&self, args: &[crate::ast::Spanned<Expr>]) -> bool {
        for arg in args {
            if !self.evaluate_cfg_expr(&arg.node) {
                return false;
            }
        }
        true
    }

    /// Evaluate a single cfg expression
    fn evaluate_cfg_expr(&self, expr: &Expr) -> bool {
        match expr {
            // @cfg(target = "wasm32")
            Expr::Binary { left, op, right } if *op == crate::ast::BinOp::Eq => {
                if let (Expr::Var(name), Expr::StringLit(value)) =
                    (&left.node, &right.node)
                {
                    if name == "target" {
                        if let Some(target) = Target::from_str(value) {
                            return self.target == target;
                        }
                    }
                }
                // Unknown cfg key, default to true (permissive)
                true
            }
            // @cfg(feature = "xyz") - future support
            _ => true, // Unknown expression, default to true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    fn make_cfg_attr(target_value: &str) -> Attribute {
        Attribute::WithArgs {
            name: Spanned::new("cfg".to_string(), Span::new(0, 3)),
            args: vec![Spanned::new(
                Expr::Binary {
                    left: Box::new(Spanned::new(
                        Expr::Var("target".to_string()),
                        Span::new(4, 10),
                    )),
                    op: BinOp::Eq,
                    right: Box::new(Spanned::new(
                        Expr::StringLit(target_value.to_string()),
                        Span::new(13, 20),
                    )),
                },
                Span::new(4, 20),
            )],
            span: Span::new(0, 21),
        }
    }

    fn make_fn(name: &str, attrs: Vec<Attribute>) -> FnDef {
        FnDef {
            attributes: attrs,
            visibility: Visibility::Private,
            name: Spanned::new(name.to_string(), Span::new(0, name.len())),
            type_params: vec![],
            params: vec![],
            ret_name: None,
            ret_ty: Spanned::new(Type::Unit, Span::new(0, 4)),
            pre: None,
            post: None,
            contracts: vec![],
            body: Spanned::new(Expr::Unit, Span::new(0, 2)),
            span: Span::new(0, 50),
        }
    }

    #[test]
    fn test_target_from_str() {
        assert_eq!(Target::from_str("wasm32"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("wasm"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("native"), Some(Target::Native));
        assert_eq!(Target::from_str("x86_64"), Some(Target::Native));
        assert_eq!(Target::from_str("wasm64"), Some(Target::Wasm64));
        assert_eq!(Target::from_str("unknown"), None);
    }

    #[test]
    fn test_cfg_evaluator_native() {
        let eval = CfgEvaluator::new(Target::Native);

        // Function without @cfg should be included
        let fn_no_cfg = make_fn("no_cfg", vec![]);
        assert!(eval.evaluate_attrs(&fn_no_cfg.attributes));

        // Function with @cfg(target = "native") should be included
        let fn_native = make_fn("native_only", vec![make_cfg_attr("native")]);
        assert!(eval.evaluate_attrs(&fn_native.attributes));

        // Function with @cfg(target = "wasm32") should be excluded
        let fn_wasm = make_fn("wasm_only", vec![make_cfg_attr("wasm32")]);
        assert!(!eval.evaluate_attrs(&fn_wasm.attributes));
    }

    #[test]
    fn test_cfg_evaluator_wasm32() {
        let eval = CfgEvaluator::new(Target::Wasm32);

        // Function without @cfg should be included
        let fn_no_cfg = make_fn("no_cfg", vec![]);
        assert!(eval.evaluate_attrs(&fn_no_cfg.attributes));

        // Function with @cfg(target = "wasm32") should be included
        let fn_wasm = make_fn("wasm_only", vec![make_cfg_attr("wasm32")]);
        assert!(eval.evaluate_attrs(&fn_wasm.attributes));

        // Function with @cfg(target = "native") should be excluded
        let fn_native = make_fn("native_only", vec![make_cfg_attr("native")]);
        assert!(!eval.evaluate_attrs(&fn_native.attributes));
    }

    #[test]
    fn test_filter_program() {
        let eval = CfgEvaluator::new(Target::Wasm32);

        let program = Program {
            items: vec![
                Item::FnDef(make_fn("always", vec![])),
                Item::FnDef(make_fn("wasm_only", vec![make_cfg_attr("wasm32")])),
                Item::FnDef(make_fn("native_only", vec![make_cfg_attr("native")])),
            ],
        };

        let filtered = eval.filter_program(&program);
        assert_eq!(filtered.items.len(), 2);

        // Verify correct functions are included
        let fn_names: Vec<&str> = filtered
            .items
            .iter()
            .filter_map(|item| {
                if let Item::FnDef(f) = item {
                    Some(f.name.node.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(fn_names.contains(&"always"));
        assert!(fn_names.contains(&"wasm_only"));
        assert!(!fn_names.contains(&"native_only"));
    }
}
