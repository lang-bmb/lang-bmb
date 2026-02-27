//! Derive macro support (v0.13.3)
//!
//! This module provides @derive attribute support for automatically
//! generating trait implementations.
//!
//! Supported traits:
//! - Debug: Generate debug string representation
//! - Clone: Generate clone implementation
//! - PartialEq: Generate equality comparison
//! - Eq: Marker trait for total equality
//! - Default: Generate default value constructor

use crate::ast::{Attribute, Expr, StructDef, EnumDef};

/// Derivable traits supported by @derive attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeriveTrait {
    /// Debug: Generate string representation for debugging
    Debug,
    /// Clone: Generate clone implementation
    Clone,
    /// PartialEq: Generate equality comparison
    PartialEq,
    /// Eq: Marker for total equality (requires PartialEq)
    Eq,
    /// Default: Generate default value constructor
    Default,
    /// Hash: Generate hash implementation
    Hash,
}

impl DeriveTrait {
    /// Parse trait name from string
    pub fn from_str(s: &str) -> Option<DeriveTrait> {
        match s {
            "Debug" => Some(DeriveTrait::Debug),
            "Clone" => Some(DeriveTrait::Clone),
            "PartialEq" => Some(DeriveTrait::PartialEq),
            "Eq" => Some(DeriveTrait::Eq),
            "Default" => Some(DeriveTrait::Default),
            "Hash" => Some(DeriveTrait::Hash),
            _ => None,
        }
    }

    /// Get trait name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            DeriveTrait::Debug => "Debug",
            DeriveTrait::Clone => "Clone",
            DeriveTrait::PartialEq => "PartialEq",
            DeriveTrait::Eq => "Eq",
            DeriveTrait::Default => "Default",
            DeriveTrait::Hash => "Hash",
        }
    }
}

/// Extract derive traits from attributes
pub fn extract_derive_traits(attrs: &[Attribute]) -> Vec<DeriveTrait> {
    let mut traits = Vec::new();

    for attr in attrs {
        if attr.name() == "derive"
            && let Attribute::WithArgs { args, .. } = attr
        {
            for arg in args {
                // Each arg should be a Var expression representing trait name
                if let Expr::Var(name) = &arg.node
                    && let Some(derive_trait) = DeriveTrait::from_str(name)
                {
                    traits.push(derive_trait);
                }
            }
        }
    }

    traits
}

/// Check if a struct has a specific derive trait
pub fn has_derive_trait(def: &StructDef, trait_kind: DeriveTrait) -> bool {
    extract_derive_traits(&def.attributes).contains(&trait_kind)
}

/// Check if an enum has a specific derive trait
pub fn has_derive_trait_enum(def: &EnumDef, trait_kind: DeriveTrait) -> bool {
    extract_derive_traits(&def.attributes).contains(&trait_kind)
}

/// Derive context for code generation
pub struct DeriveContext<'a> {
    /// Name of the type being derived
    pub name: &'a str,
    /// Traits to derive
    pub traits: Vec<DeriveTrait>,
}

impl<'a> DeriveContext<'a> {
    /// Create context from struct definition
    pub fn from_struct(def: &'a StructDef) -> Self {
        DeriveContext {
            name: &def.name.node,
            traits: extract_derive_traits(&def.attributes),
        }
    }

    /// Create context from enum definition
    pub fn from_enum(def: &'a EnumDef) -> Self {
        DeriveContext {
            name: &def.name.node,
            traits: extract_derive_traits(&def.attributes),
        }
    }

    /// Check if context has a specific trait
    pub fn has_trait(&self, trait_kind: DeriveTrait) -> bool {
        self.traits.contains(&trait_kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Spanned, Span};

    fn make_derive_attr(traits: &[&str]) -> Attribute {
        let args: Vec<_> = traits.iter().map(|t| {
            Spanned::new(Expr::Var(t.to_string()), Span::new(0, t.len()))
        }).collect();

        Attribute::WithArgs {
            name: Spanned::new("derive".to_string(), Span::new(0, 6)),
            args,
            span: Span::new(0, 20),
        }
    }

    #[test]
    fn test_derive_trait_from_str() {
        assert_eq!(DeriveTrait::from_str("Debug"), Some(DeriveTrait::Debug));
        assert_eq!(DeriveTrait::from_str("Clone"), Some(DeriveTrait::Clone));
        assert_eq!(DeriveTrait::from_str("PartialEq"), Some(DeriveTrait::PartialEq));
        assert_eq!(DeriveTrait::from_str("Unknown"), None);
    }

    #[test]
    fn test_extract_derive_traits() {
        let attrs = vec![make_derive_attr(&["Debug", "Clone", "PartialEq"])];
        let traits = extract_derive_traits(&attrs);

        assert_eq!(traits.len(), 3);
        assert!(traits.contains(&DeriveTrait::Debug));
        assert!(traits.contains(&DeriveTrait::Clone));
        assert!(traits.contains(&DeriveTrait::PartialEq));
    }

    #[test]
    fn test_extract_derive_traits_multiple_attrs() {
        let attrs = vec![
            make_derive_attr(&["Debug"]),
            make_derive_attr(&["Clone", "Eq"]),
        ];
        let traits = extract_derive_traits(&attrs);

        assert_eq!(traits.len(), 3);
        assert!(traits.contains(&DeriveTrait::Debug));
        assert!(traits.contains(&DeriveTrait::Clone));
        assert!(traits.contains(&DeriveTrait::Eq));
    }

    #[test]
    fn test_derive_context() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![make_derive_attr(&["Debug", "Clone"])],
            visibility: Visibility::Private,
            name: Spanned::new("Point".to_string(), Span::new(0, 5)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };

        let ctx = DeriveContext::from_struct(&def);
        assert_eq!(ctx.name, "Point");
        assert!(ctx.has_trait(DeriveTrait::Debug));
        assert!(ctx.has_trait(DeriveTrait::Clone));
        assert!(!ctx.has_trait(DeriveTrait::Eq));
    }

    #[test]
    fn test_derive_trait_as_str() {
        assert_eq!(DeriveTrait::Debug.as_str(), "Debug");
        assert_eq!(DeriveTrait::Clone.as_str(), "Clone");
        assert_eq!(DeriveTrait::PartialEq.as_str(), "PartialEq");
        assert_eq!(DeriveTrait::Eq.as_str(), "Eq");
        assert_eq!(DeriveTrait::Default.as_str(), "Default");
        assert_eq!(DeriveTrait::Hash.as_str(), "Hash");
    }

    #[test]
    fn test_derive_trait_from_str_all() {
        assert_eq!(DeriveTrait::from_str("Default"), Some(DeriveTrait::Default));
        assert_eq!(DeriveTrait::from_str("Hash"), Some(DeriveTrait::Hash));
        assert_eq!(DeriveTrait::from_str("Eq"), Some(DeriveTrait::Eq));
        assert_eq!(DeriveTrait::from_str("Bogus"), None);
    }

    #[test]
    fn test_has_derive_trait() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![make_derive_attr(&["Debug", "Hash"])],
            visibility: Visibility::Public,
            name: Spanned::new("MyStruct".to_string(), Span::new(0, 8)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };

        assert!(has_derive_trait(&def, DeriveTrait::Debug));
        assert!(has_derive_trait(&def, DeriveTrait::Hash));
        assert!(!has_derive_trait(&def, DeriveTrait::Clone));
    }

    #[test]
    fn test_has_derive_trait_enum() {
        use crate::ast::EnumDef;

        let def = EnumDef {
            attributes: vec![make_derive_attr(&["PartialEq", "Eq"])],
            visibility: crate::ast::Visibility::Private,
            name: Spanned::new("Color".to_string(), Span::new(0, 5)),
            type_params: vec![],
            variants: vec![],
            span: Span::new(0, 50),
        };

        assert!(has_derive_trait_enum(&def, DeriveTrait::PartialEq));
        assert!(has_derive_trait_enum(&def, DeriveTrait::Eq));
        assert!(!has_derive_trait_enum(&def, DeriveTrait::Debug));
    }

    #[test]
    fn test_derive_context_from_enum() {
        use crate::ast::EnumDef;

        let def = EnumDef {
            attributes: vec![make_derive_attr(&["Clone", "Default"])],
            visibility: crate::ast::Visibility::Private,
            name: Spanned::new("Shape".to_string(), Span::new(0, 5)),
            type_params: vec![],
            variants: vec![],
            span: Span::new(0, 50),
        };

        let ctx = DeriveContext::from_enum(&def);
        assert_eq!(ctx.name, "Shape");
        assert!(ctx.has_trait(DeriveTrait::Clone));
        assert!(ctx.has_trait(DeriveTrait::Default));
    }

    #[test]
    fn test_extract_derive_empty() {
        let traits = extract_derive_traits(&[]);
        assert!(traits.is_empty());
    }

    // --- Cycle 1226: Additional Derive Tests ---

    #[test]
    fn test_extract_derive_ignores_non_derive_attrs() {
        let attrs = vec![
            Attribute::Simple {
                name: Spanned::new("inline".to_string(), Span::new(0, 6)),
                span: Span::new(0, 7),
            },
            make_derive_attr(&["Debug"]),
        ];
        let traits = extract_derive_traits(&attrs);
        assert_eq!(traits.len(), 1);
        assert!(traits.contains(&DeriveTrait::Debug));
    }

    #[test]
    fn test_extract_derive_ignores_unknown_traits() {
        let attrs = vec![make_derive_attr(&["Debug", "Unknown", "Clone"])];
        let traits = extract_derive_traits(&attrs);
        assert_eq!(traits.len(), 2);
        assert!(traits.contains(&DeriveTrait::Debug));
        assert!(traits.contains(&DeriveTrait::Clone));
    }

    #[test]
    fn test_derive_trait_equality() {
        assert_eq!(DeriveTrait::Debug, DeriveTrait::Debug);
        assert_ne!(DeriveTrait::Debug, DeriveTrait::Clone);
        assert_ne!(DeriveTrait::PartialEq, DeriveTrait::Eq);
    }

    #[test]
    fn test_derive_context_no_derives() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: Spanned::new("Empty".to_string(), Span::new(0, 5)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        let ctx = DeriveContext::from_struct(&def);
        assert_eq!(ctx.name, "Empty");
        assert!(ctx.traits.is_empty());
        assert!(!ctx.has_trait(DeriveTrait::Debug));
    }

    #[test]
    fn test_derive_trait_from_str_case_sensitive() {
        assert_eq!(DeriveTrait::from_str("debug"), None);
        assert_eq!(DeriveTrait::from_str("clone"), None);
        assert_eq!(DeriveTrait::from_str("HASH"), None);
        assert_eq!(DeriveTrait::from_str("partialeq"), None);
    }

    #[test]
    fn test_has_derive_trait_no_attrs() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: Spanned::new("Bare".to_string(), Span::new(0, 4)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        assert!(!has_derive_trait(&def, DeriveTrait::Debug));
        assert!(!has_derive_trait(&def, DeriveTrait::Clone));
    }

    #[test]
    fn test_has_derive_trait_enum_no_attrs() {
        let def = EnumDef {
            attributes: vec![],
            visibility: crate::ast::Visibility::Private,
            name: Spanned::new("Empty".to_string(), Span::new(0, 5)),
            type_params: vec![],
            variants: vec![],
            span: Span::new(0, 50),
        };
        assert!(!has_derive_trait_enum(&def, DeriveTrait::PartialEq));
    }

    #[test]
    fn test_derive_context_all_traits() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![
                make_derive_attr(&["Debug", "Clone", "PartialEq", "Eq", "Default", "Hash"]),
            ],
            visibility: Visibility::Public,
            name: Spanned::new("Full".to_string(), Span::new(0, 4)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        let ctx = DeriveContext::from_struct(&def);
        assert_eq!(ctx.traits.len(), 6);
        assert!(ctx.has_trait(DeriveTrait::Debug));
        assert!(ctx.has_trait(DeriveTrait::Clone));
        assert!(ctx.has_trait(DeriveTrait::PartialEq));
        assert!(ctx.has_trait(DeriveTrait::Eq));
        assert!(ctx.has_trait(DeriveTrait::Default));
        assert!(ctx.has_trait(DeriveTrait::Hash));
    }

    #[test]
    fn test_extract_derive_with_reason_attr_ignored() {
        // WithReason attributes should not be treated as derive
        let attrs = vec![
            Attribute::WithReason {
                name: Spanned::new("derive".to_string(), Span::new(0, 6)),
                reason: Spanned::new("wrong format".to_string(), Span::new(0, 12)),
                span: Span::new(0, 20),
            },
        ];
        let traits = extract_derive_traits(&attrs);
        assert!(traits.is_empty());
    }

    // ================================================================
    // Additional derive tests (Cycle 1235)
    // ================================================================

    #[test]
    fn test_derive_trait_copy() {
        let t = DeriveTrait::Debug;
        let t2 = t; // Copy
        assert_eq!(t, t2);
    }

    #[test]
    fn test_derive_trait_clone() {
        let t = DeriveTrait::Hash;
        let cloned = t.clone();
        assert_eq!(t, cloned);
    }

    #[test]
    fn test_derive_trait_from_str_empty() {
        assert_eq!(DeriveTrait::from_str(""), None);
    }

    #[test]
    fn test_derive_trait_roundtrip() {
        for t in [DeriveTrait::Debug, DeriveTrait::Clone, DeriveTrait::PartialEq,
                  DeriveTrait::Eq, DeriveTrait::Default, DeriveTrait::Hash] {
            let s = t.as_str();
            assert_eq!(DeriveTrait::from_str(s), Some(t));
        }
    }

    #[test]
    fn test_extract_derive_duplicate_traits() {
        let attrs = vec![make_derive_attr(&["Debug", "Debug", "Clone"])];
        let traits = extract_derive_traits(&attrs);
        // Duplicates are not deduplicated
        assert_eq!(traits.len(), 3);
    }

    #[test]
    fn test_derive_context_name() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![make_derive_attr(&["Debug"])],
            visibility: Visibility::Public,
            name: Spanned::new("MyType".to_string(), Span::new(0, 6)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        let ctx = DeriveContext::from_struct(&def);
        assert_eq!(ctx.name, "MyType");
    }

    #[test]
    fn test_has_derive_trait_multiple_attrs() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![
                Attribute::Simple {
                    name: Spanned::new("inline".to_string(), Span::new(0, 6)),
                    span: Span::new(0, 7),
                },
                make_derive_attr(&["Clone"]),
            ],
            visibility: Visibility::Private,
            name: Spanned::new("Mixed".to_string(), Span::new(0, 5)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        assert!(has_derive_trait(&def, DeriveTrait::Clone));
        assert!(!has_derive_trait(&def, DeriveTrait::Debug));
    }

    #[test]
    fn test_derive_context_has_trait_negative() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![make_derive_attr(&["Debug"])],
            visibility: Visibility::Private,
            name: Spanned::new("Single".to_string(), Span::new(0, 6)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        let ctx = DeriveContext::from_struct(&def);
        assert!(ctx.has_trait(DeriveTrait::Debug));
        assert!(!ctx.has_trait(DeriveTrait::Clone));
        assert!(!ctx.has_trait(DeriveTrait::Hash));
    }

    #[test]
    fn test_extract_derive_only_var_args() {
        // Non-Var args in derive should be ignored
        let attrs = vec![Attribute::WithArgs {
            name: Spanned::new("derive".to_string(), Span::new(0, 6)),
            args: vec![
                Spanned::new(Expr::Var("Debug".to_string()), Span::new(0, 5)),
                Spanned::new(Expr::IntLit(42), Span::new(0, 2)), // Not a Var
            ],
            span: Span::new(0, 20),
        }];
        let traits = extract_derive_traits(&attrs);
        assert_eq!(traits.len(), 1);
        assert!(traits.contains(&DeriveTrait::Debug));
    }

    #[test]
    fn test_derive_trait_debug_format() {
        let t = DeriveTrait::Default;
        let debug_str = format!("{:?}", t);
        assert_eq!(debug_str, "Default");
    }

    // ================================================================
    // Additional derive tests (Cycle 1239)
    // ================================================================

    #[test]
    fn test_derive_trait_ne_all_pairs() {
        let all = [
            DeriveTrait::Debug, DeriveTrait::Clone, DeriveTrait::PartialEq,
            DeriveTrait::Eq, DeriveTrait::Default, DeriveTrait::Hash,
        ];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j]);
            }
        }
    }

    #[test]
    fn test_derive_trait_as_str_hash() {
        assert_eq!(DeriveTrait::Hash.as_str(), "Hash");
    }

    #[test]
    fn test_extract_derive_single_trait() {
        let attrs = vec![make_derive_attr(&["Clone"])];
        let traits = extract_derive_traits(&attrs);
        assert_eq!(traits.len(), 1);
        assert_eq!(traits[0], DeriveTrait::Clone);
    }

    #[test]
    fn test_derive_context_from_enum_all_traits() {
        let def = EnumDef {
            attributes: vec![
                make_derive_attr(&["Debug", "Clone", "PartialEq", "Eq", "Default", "Hash"]),
            ],
            visibility: crate::ast::Visibility::Public,
            name: Spanned::new("Full".to_string(), Span::new(0, 4)),
            type_params: vec![],
            variants: vec![],
            span: Span::new(0, 50),
        };
        let ctx = DeriveContext::from_enum(&def);
        assert_eq!(ctx.traits.len(), 6);
    }

    #[test]
    fn test_has_derive_trait_enum_all() {
        let def = EnumDef {
            attributes: vec![
                make_derive_attr(&["Debug", "Clone", "PartialEq", "Eq", "Default", "Hash"]),
            ],
            visibility: crate::ast::Visibility::Public,
            name: Spanned::new("AllTraits".to_string(), Span::new(0, 9)),
            type_params: vec![],
            variants: vec![],
            span: Span::new(0, 50),
        };
        assert!(has_derive_trait_enum(&def, DeriveTrait::Debug));
        assert!(has_derive_trait_enum(&def, DeriveTrait::Hash));
        assert!(has_derive_trait_enum(&def, DeriveTrait::Default));
    }

    #[test]
    fn test_derive_context_traits_count() {
        use crate::ast::Visibility;

        let def = StructDef {
            attributes: vec![make_derive_attr(&["Debug", "Clone"])],
            visibility: Visibility::Private,
            name: Spanned::new("Two".to_string(), Span::new(0, 3)),
            type_params: vec![],
            fields: vec![],
            span: Span::new(0, 50),
        };
        let ctx = DeriveContext::from_struct(&def);
        assert_eq!(ctx.traits.len(), 2);
    }

    #[test]
    fn test_extract_derive_multiple_derive_attrs() {
        let attrs = vec![
            make_derive_attr(&["Debug"]),
            make_derive_attr(&["Clone"]),
            make_derive_attr(&["Hash"]),
        ];
        let traits = extract_derive_traits(&attrs);
        assert_eq!(traits.len(), 3);
        assert!(traits.contains(&DeriveTrait::Debug));
        assert!(traits.contains(&DeriveTrait::Clone));
        assert!(traits.contains(&DeriveTrait::Hash));
    }

    #[test]
    fn test_derive_trait_from_str_whitespace() {
        assert_eq!(DeriveTrait::from_str(" Debug"), None);
        assert_eq!(DeriveTrait::from_str("Debug "), None);
        assert_eq!(DeriveTrait::from_str(" "), None);
    }

    #[test]
    fn test_extract_derive_empty_args() {
        let attrs = vec![Attribute::WithArgs {
            name: Spanned::new("derive".to_string(), Span::new(0, 6)),
            args: vec![],
            span: Span::new(0, 10),
        }];
        let traits = extract_derive_traits(&attrs);
        assert!(traits.is_empty());
    }

    #[test]
    fn test_derive_trait_debug_all_variants() {
        assert_eq!(format!("{:?}", DeriveTrait::Debug), "Debug");
        assert_eq!(format!("{:?}", DeriveTrait::Clone), "Clone");
        assert_eq!(format!("{:?}", DeriveTrait::PartialEq), "PartialEq");
        assert_eq!(format!("{:?}", DeriveTrait::Eq), "Eq");
        assert_eq!(format!("{:?}", DeriveTrait::Default), "Default");
        assert_eq!(format!("{:?}", DeriveTrait::Hash), "Hash");
    }
}
