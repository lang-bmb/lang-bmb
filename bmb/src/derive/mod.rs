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
}
