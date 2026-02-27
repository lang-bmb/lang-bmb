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

        Program {
            header: program.header.clone(),
            items,
        }
    }

    /// Check if an item should be included for the current target
    pub fn should_include_item(&self, item: &Item) -> bool {
        match item {
            Item::FnDef(f) => self.evaluate_attrs(&f.attributes),
            Item::StructDef(s) => self.evaluate_attrs(&s.attributes),
            Item::EnumDef(e) => self.evaluate_attrs(&e.attributes),
            Item::Use(_) => true, // Use statements are always included
            Item::ExternFn(e) => self.evaluate_attrs(&e.attributes), // v0.13.0
            Item::TraitDef(t) => self.evaluate_attrs(&t.attributes), // v0.20.1
            Item::ImplBlock(i) => self.evaluate_attrs(&i.attributes), // v0.20.1
            Item::TypeAlias(t) => self.evaluate_attrs(&t.attributes), // v0.50.6
        }
    }

    /// Evaluate @cfg attributes for an item
    /// Returns true if item should be included
    fn evaluate_attrs(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.name() == "cfg"
                && let Attribute::WithArgs { args, .. } = attr
            {
                // Evaluate cfg condition
                if !self.evaluate_cfg_args(args) {
                    return false;
                }
            }
            // @cfg without args is invalid, skip
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
                if let (Expr::Var(name), Expr::StringLit(value)) = (&left.node, &right.node)
                    && name == "target"
                    && let Some(target) = Target::from_str(value)
                {
                    return self.target == target;
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
            is_async: false,
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
            header: None,
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

    #[test]
    fn test_target_as_str() {
        assert_eq!(Target::Native.as_str(), "native");
        assert_eq!(Target::Wasm32.as_str(), "wasm32");
        assert_eq!(Target::Wasm64.as_str(), "wasm64");
    }

    #[test]
    fn test_target_default() {
        assert_eq!(Target::default(), Target::Native);
    }

    #[test]
    fn test_target_from_str_aliases() {
        assert_eq!(Target::from_str("aarch64"), Some(Target::Native));
        assert_eq!(Target::from_str("x86"), Some(Target::Native));
        assert_eq!(Target::from_str("arm"), Some(Target::Native));
        assert_eq!(Target::from_str("wasm32-wasi"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("wasm32-unknown"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("NATIVE"), Some(Target::Native)); // case-insensitive
    }

    #[test]
    fn test_should_include_item_no_cfg() {
        let eval = CfgEvaluator::new(Target::Native);
        let item = Item::FnDef(make_fn("test", vec![]));
        assert!(eval.should_include_item(&item));
    }

    #[test]
    fn test_cfg_evaluator_wasm64() {
        let eval = CfgEvaluator::new(Target::Wasm64);
        let fn_wasm64 = make_fn("wasm64_only", vec![make_cfg_attr("wasm64")]);
        assert!(eval.evaluate_attrs(&fn_wasm64.attributes));

        let fn_native = make_fn("native_only", vec![make_cfg_attr("native")]);
        assert!(!eval.evaluate_attrs(&fn_native.attributes));
    }

    #[test]
    fn test_filter_program_empty() {
        let eval = CfgEvaluator::new(Target::Native);
        let program = Program { header: None, items: vec![] };
        let filtered = eval.filter_program(&program);
        assert!(filtered.items.is_empty());
    }

    // --- Cycle 1226: Additional Cfg Tests ---

    #[test]
    fn test_target_roundtrip() {
        for target in [Target::Native, Target::Wasm32, Target::Wasm64] {
            let s = target.as_str();
            assert_eq!(Target::from_str(s), Some(target));
        }
    }

    #[test]
    fn test_should_include_struct_no_cfg() {
        let eval = CfgEvaluator::new(Target::Wasm32);
        let sd = StructDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: Spanned::new("Point".to_string(), Span::new(0, 5)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        assert!(eval.should_include_item(&Item::StructDef(sd)));
    }

    #[test]
    fn test_should_include_struct_with_cfg() {
        let eval = CfgEvaluator::new(Target::Wasm32);
        let sd = StructDef {
            attributes: vec![make_cfg_attr("native")],
            visibility: Visibility::Private,
            name: Spanned::new("NativeOnly".to_string(), Span::new(0, 10)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        assert!(!eval.should_include_item(&Item::StructDef(sd)));
    }

    #[test]
    fn test_should_include_enum_with_cfg() {
        let eval = CfgEvaluator::new(Target::Native);
        let ed = EnumDef {
            attributes: vec![make_cfg_attr("wasm32")],
            visibility: Visibility::Private,
            name: Spanned::new("WasmError".to_string(), Span::new(0, 9)),
            type_params: vec![],
            variants: vec![],
            span: Span::new(0, 50),
        };
        assert!(!eval.should_include_item(&Item::EnumDef(ed)));
    }

    #[test]
    fn test_should_include_use_always() {
        let eval = CfgEvaluator::new(Target::Wasm32);
        let us = UseStmt {
            path: vec![Spanned::new("std".to_string(), Span::new(0, 3))],
            span: Span::new(0, 10),
        };
        // Use statements should always be included regardless of target
        assert!(eval.should_include_item(&Item::Use(us)));
    }

    #[test]
    fn test_should_include_extern_fn() {
        let eval = CfgEvaluator::new(Target::Native);
        let ef = ExternFn {
            attributes: vec![make_cfg_attr("native")],
            visibility: Visibility::Public,
            abi: Abi::C,
            link_name: None,
            name: Spanned::new("malloc".to_string(), Span::new(0, 6)),
            params: vec![],
            ret_ty: Spanned::new(Type::I64, Span::new(0, 3)),
            span: Span::new(0, 50),
        };
        assert!(eval.should_include_item(&Item::ExternFn(ef)));
    }

    #[test]
    fn test_should_include_trait_def() {
        let eval = CfgEvaluator::new(Target::Wasm32);
        let td = TraitDef {
            attributes: vec![make_cfg_attr("wasm32")],
            visibility: Visibility::Public,
            name: Spanned::new("WasmTrait".to_string(), Span::new(0, 9)),
            type_params: vec![],
            methods: vec![],
            span: Span::new(0, 50),
        };
        assert!(eval.should_include_item(&Item::TraitDef(td)));
    }

    #[test]
    fn test_should_include_impl_block() {
        let eval = CfgEvaluator::new(Target::Native);
        let ib = ImplBlock {
            attributes: vec![make_cfg_attr("wasm32")],
            type_params: vec![],
            trait_name: Spanned::new("Display".to_string(), Span::new(0, 7)),
            target_type: Spanned::new(Type::Named("Point".to_string()), Span::new(0, 5)),
            methods: vec![],
            span: Span::new(0, 50),
        };
        // impl with @cfg(target = "wasm32") should be excluded on native
        assert!(!eval.should_include_item(&Item::ImplBlock(ib)));
    }

    #[test]
    fn test_should_include_type_alias() {
        let eval = CfgEvaluator::new(Target::Native);
        let ta = TypeAliasDef {
            attributes: vec![make_cfg_attr("native")],
            visibility: Visibility::Private,
            name: Spanned::new("Size".to_string(), Span::new(0, 4)),
            type_params: vec![],
            target: Spanned::new(Type::I64, Span::new(0, 3)),
            refinement: None,
            span: Span::new(0, 50),
        };
        assert!(eval.should_include_item(&Item::TypeAlias(ta)));
    }

    #[test]
    fn test_filter_program_mixed_items() {
        let eval = CfgEvaluator::new(Target::Native);
        let program = Program {
            header: None,
            items: vec![
                Item::FnDef(make_fn("fn_native", vec![make_cfg_attr("native")])),
                Item::FnDef(make_fn("fn_wasm", vec![make_cfg_attr("wasm32")])),
                Item::Use(UseStmt {
                    path: vec![Spanned::new("std".to_string(), Span::new(0, 3))],
                    span: Span::new(0, 10),
                }),
            ],
        };
        let filtered = eval.filter_program(&program);
        assert_eq!(filtered.items.len(), 2); // fn_native + Use
    }

    #[test]
    fn test_evaluate_attrs_simple_attribute_ignored() {
        let eval = CfgEvaluator::new(Target::Native);
        // Simple attribute (not @cfg) should not affect inclusion
        let attrs = vec![Attribute::Simple {
            name: Spanned::new("inline".to_string(), Span::new(0, 6)),
            span: Span::new(0, 7),
        }];
        assert!(eval.evaluate_attrs(&attrs));
    }

    #[test]
    fn test_cfg_unknown_expression_defaults_true() {
        let eval = CfgEvaluator::new(Target::Native);
        // cfg with non-Binary expression should default to true
        let attr = Attribute::WithArgs {
            name: Spanned::new("cfg".to_string(), Span::new(0, 3)),
            args: vec![Spanned::new(Expr::BoolLit(true), Span::new(0, 4))],
            span: Span::new(0, 10),
        };
        assert!(eval.evaluate_attrs(&[attr]));
    }

    #[test]
    fn test_filter_program_preserves_header() {
        let eval = CfgEvaluator::new(Target::Native);
        let header = ModuleHeader {
            name: Spanned::new("test".to_string(), Span::new(0, 4)),
            version: None,
            summary: None,
            exports: vec![],
            depends: vec![],
            span: Span::new(0, 10),
        };
        let program = Program {
            header: Some(header),
            items: vec![],
        };
        let filtered = eval.filter_program(&program);
        assert!(filtered.header.is_some());
    }

    // ================================================================
    // Additional cfg tests (Cycle 1235)
    // ================================================================

    #[test]
    fn test_target_copy() {
        let t = Target::Wasm32;
        let t2 = t; // Copy
        assert_eq!(t, t2);
    }

    #[test]
    fn test_target_clone() {
        let t = Target::Wasm64;
        let cloned = t.clone();
        assert_eq!(t, cloned);
    }

    #[test]
    fn test_target_debug_format() {
        let t = Target::Native;
        let s = format!("{:?}", t);
        assert_eq!(s, "Native");
    }

    #[test]
    fn test_target_from_str_wasm_aliases() {
        // All wasm32 aliases
        assert_eq!(Target::from_str("wasm32"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("wasm"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("wasm32-wasi"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("wasm32-unknown"), Some(Target::Wasm32));
    }

    #[test]
    fn test_target_from_str_case_insensitive() {
        assert_eq!(Target::from_str("NATIVE"), Some(Target::Native));
        assert_eq!(Target::from_str("Wasm32"), Some(Target::Wasm32));
        assert_eq!(Target::from_str("WASM64"), Some(Target::Wasm64));
        assert_eq!(Target::from_str("X86_64"), Some(Target::Native));
    }

    #[test]
    fn test_target_from_str_empty_and_none() {
        assert_eq!(Target::from_str(""), None);
        assert_eq!(Target::from_str("riscv64"), None);
        assert_eq!(Target::from_str("mips"), None);
    }

    #[test]
    fn test_filter_program_all_excluded() {
        let eval = CfgEvaluator::new(Target::Wasm64);
        let program = Program {
            header: None,
            items: vec![
                Item::FnDef(make_fn("native_fn", vec![make_cfg_attr("native")])),
                Item::FnDef(make_fn("wasm32_fn", vec![make_cfg_attr("wasm32")])),
            ],
        };
        let filtered = eval.filter_program(&program);
        assert!(filtered.items.is_empty());
    }

    #[test]
    fn test_filter_program_all_included() {
        let eval = CfgEvaluator::new(Target::Native);
        let program = Program {
            header: None,
            items: vec![
                Item::FnDef(make_fn("no_cfg", vec![])),
                Item::FnDef(make_fn("native_fn", vec![make_cfg_attr("native")])),
            ],
        };
        let filtered = eval.filter_program(&program);
        assert_eq!(filtered.items.len(), 2);
    }

    #[test]
    fn test_evaluate_attrs_empty() {
        let eval = CfgEvaluator::new(Target::Wasm32);
        assert!(eval.evaluate_attrs(&[]));
    }

    #[test]
    fn test_cfg_evaluator_unknown_binary_key() {
        let eval = CfgEvaluator::new(Target::Native);
        // cfg with non-target key should default to true (permissive)
        let attr = Attribute::WithArgs {
            name: Spanned::new("cfg".to_string(), Span::new(0, 3)),
            args: vec![Spanned::new(
                Expr::Binary {
                    left: Box::new(Spanned::new(
                        Expr::Var("feature".to_string()),
                        Span::new(4, 11),
                    )),
                    op: BinOp::Eq,
                    right: Box::new(Spanned::new(
                        Expr::StringLit("foo".to_string()),
                        Span::new(14, 19),
                    )),
                },
                Span::new(4, 19),
            )],
            span: Span::new(0, 20),
        };
        assert!(eval.evaluate_attrs(&[attr]));
    }

    // ================================================================
    // Additional cfg tests (Cycle 1241)
    // ================================================================

    #[test]
    fn test_target_ne_variants() {
        assert_ne!(Target::Native, Target::Wasm32);
        assert_ne!(Target::Native, Target::Wasm64);
        assert_ne!(Target::Wasm32, Target::Wasm64);
    }

    #[test]
    fn test_target_eq_same() {
        assert_eq!(Target::Native, Target::Native);
        assert_eq!(Target::Wasm32, Target::Wasm32);
        assert_eq!(Target::Wasm64, Target::Wasm64);
    }

    #[test]
    fn test_cfg_multiple_cfg_attrs_all_must_pass() {
        let eval = CfgEvaluator::new(Target::Native);
        // Multiple @cfg attrs: both must pass
        let fn_with_two = make_fn("multi", vec![
            make_cfg_attr("native"),
            make_cfg_attr("wasm32"), // This one fails for native
        ]);
        assert!(!eval.evaluate_attrs(&fn_with_two.attributes));
    }

    #[test]
    fn test_filter_program_single_fn_included() {
        let eval = CfgEvaluator::new(Target::Native);
        let program = Program {
            header: None,
            items: vec![Item::FnDef(make_fn("only", vec![make_cfg_attr("native")]))],
        };
        let filtered = eval.filter_program(&program);
        assert_eq!(filtered.items.len(), 1);
    }

    #[test]
    fn test_should_include_enum_no_cfg() {
        let eval = CfgEvaluator::new(Target::Wasm32);
        let ed = EnumDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: Spanned::new("Color".to_string(), Span::new(0, 5)),
            type_params: vec![],
            variants: vec![],
            span: Span::new(0, 20),
        };
        assert!(eval.should_include_item(&Item::EnumDef(ed)));
    }

    #[test]
    fn test_cfg_non_eq_binary_op_defaults_true() {
        let eval = CfgEvaluator::new(Target::Native);
        // Binary expression with non-Eq op should default true
        let attr = Attribute::WithArgs {
            name: Spanned::new("cfg".to_string(), Span::new(0, 3)),
            args: vec![Spanned::new(
                Expr::Binary {
                    left: Box::new(Spanned::new(
                        Expr::Var("target".to_string()),
                        Span::new(4, 10),
                    )),
                    op: BinOp::Ne,
                    right: Box::new(Spanned::new(
                        Expr::StringLit("wasm32".to_string()),
                        Span::new(14, 22),
                    )),
                },
                Span::new(4, 22),
            )],
            span: Span::new(0, 23),
        };
        // Ne is not handled, defaults to true
        assert!(eval.evaluate_attrs(&[attr]));
    }

    #[test]
    fn test_cfg_evaluator_new_constructor() {
        let eval = CfgEvaluator::new(Target::Wasm64);
        // Verify it accepts wasm64 target functions
        let fn_wasm64 = make_fn("test", vec![make_cfg_attr("wasm64")]);
        assert!(eval.evaluate_attrs(&fn_wasm64.attributes));
    }

    #[test]
    fn test_filter_program_preserves_none_header() {
        let eval = CfgEvaluator::new(Target::Native);
        let program = Program { header: None, items: vec![] };
        let filtered = eval.filter_program(&program);
        assert!(filtered.header.is_none());
    }

    #[test]
    fn test_multiple_non_cfg_attrs_included() {
        let eval = CfgEvaluator::new(Target::Native);
        // Non-cfg attributes should not affect inclusion
        let attrs = vec![
            Attribute::Simple {
                name: Spanned::new("inline".to_string(), Span::new(0, 6)),
                span: Span::new(0, 7),
            },
            Attribute::Simple {
                name: Spanned::new("test".to_string(), Span::new(0, 4)),
                span: Span::new(0, 5),
            },
        ];
        assert!(eval.evaluate_attrs(&attrs));
    }

    #[test]
    fn test_cfg_with_non_cfg_and_cfg_attrs_mixed() {
        let eval = CfgEvaluator::new(Target::Wasm32);
        let attrs = vec![
            Attribute::Simple {
                name: Spanned::new("inline".to_string(), Span::new(0, 6)),
                span: Span::new(0, 7),
            },
            make_cfg_attr("wasm32"),
        ];
        // Non-cfg is ignored, cfg(wasm32) passes for Wasm32 target
        assert!(eval.evaluate_attrs(&attrs));
    }
}
