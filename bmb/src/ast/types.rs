//! Type AST nodes

use serde::{Deserialize, Serialize};

/// Type representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 64-bit floating point
    F64,
    /// Boolean
    Bool,
    /// Unit type ()
    Unit,
    /// Named type (struct or enum)
    Named(String),
    /// Struct type with fields (resolved after type checking)
    Struct {
        name: String,
        fields: Vec<(String, Box<Type>)>,
    },
    /// Enum type with variants (resolved after type checking)
    Enum {
        name: String,
        variants: Vec<(String, Vec<Box<Type>>)>,
    },
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
            Type::Named(name) => write!(f, "{name}"),
            Type::Struct { name, .. } => write!(f, "{name}"),
            Type::Enum { name, .. } => write!(f, "{name}"),
        }
    }
}
