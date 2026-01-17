//! Type AST nodes

use super::{Spanned, Expr};
use serde::{Deserialize, Serialize};

/// Type parameter (v0.13.1)
/// e.g., `T`, `T: Ord`, `T: Clone + Debug`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeParam {
    /// Name of the type parameter (e.g., "T")
    pub name: String,
    /// Trait bounds (e.g., ["Ord", "Clone"])
    pub bounds: Vec<String>,
}

impl TypeParam {
    /// Create a simple type parameter without bounds
    pub fn new(name: impl Into<String>) -> Self {
        TypeParam {
            name: name.into(),
            bounds: vec![],
        }
    }

    /// Create a type parameter with bounds
    pub fn with_bounds(name: impl Into<String>, bounds: Vec<String>) -> Self {
        TypeParam {
            name: name.into(),
            bounds,
        }
    }
}

/// Type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    // v0.38: Unsigned integer types
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer
    U64,
    /// 64-bit floating point
    F64,
    /// Boolean
    Bool,
    /// Unit type ()
    Unit,
    /// String type (v0.5 Phase 2)
    String,
    /// Character type (v0.64)
    Char,
    /// Range type (v0.5 Phase 3) - represents start..end
    Range(Box<Type>),
    /// Named type (struct or enum)
    Named(String),
    /// Type variable (v0.13.1): T, U, etc.
    /// Used in generic context, e.g., `fn identity<T>(x: T) -> T`
    TypeVar(String),
    /// Generic type (v0.13.1): `Container<T>`, `Result<T, E>`, etc.
    /// Used for applying type arguments to generic types
    Generic {
        name: String,
        type_args: Vec<Box<Type>>,
    },
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
    /// Reference type (v0.5 Phase 5): &T
    Ref(Box<Type>),
    /// Mutable reference type (v0.5 Phase 5): &mut T
    RefMut(Box<Type>),
    /// Fixed-size array type (v0.5 Phase 6): [T; N]
    Array(Box<Type>, usize),
    /// Inline refinement type (v0.2): T{constraints}
    /// e.g., i64{!= 0}, i64{>= lo, <= hi}
    /// The constraints are expressions relative to the refined value
    Refined {
        base: Box<Type>,
        constraints: Vec<Spanned<Expr>>,
    },
    /// Function/Closure type (v0.20.0): fn(T1, T2) -> R
    /// Represents both closures and function references
    Fn {
        params: Vec<Box<Type>>,
        ret: Box<Type>,
    },
    /// Never type (v0.31): represents unreachable code
    /// Used for `todo` expressions - compatible with any type
    Never,
    /// Nullable type (v0.37): T?
    /// Sugar for `Option<T>`. Per spec, T? is the only nullable syntax.
    /// Example: i64?, String?, MyStruct?
    Nullable(Box<Type>),
    /// Tuple type (v0.42): (T, U, V)
    /// Heterogeneous fixed-size collection
    Tuple(Vec<Box<Type>>),
}

/// Manual PartialEq implementation for Type
/// Ignores refinement constraints for type equality checks (structural equality)
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::I32, Type::I32) => true,
            (Type::I64, Type::I64) => true,
            // v0.38: Unsigned types
            (Type::U32, Type::U32) => true,
            (Type::U64, Type::U64) => true,
            (Type::F64, Type::F64) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Unit, Type::Unit) => true,
            (Type::String, Type::String) => true,
            // v0.64: Char type comparison
            (Type::Char, Type::Char) => true,
            (Type::Range(a), Type::Range(b)) => a == b,
            (Type::Named(a), Type::Named(b)) => a == b,
            // v0.13.1: TypeVar equality
            (Type::TypeVar(a), Type::TypeVar(b)) => a == b,
            // v0.13.1: Generic type equality
            (
                Type::Generic { name: n1, type_args: a1 },
                Type::Generic { name: n2, type_args: a2 },
            ) => n1 == n2 && a1 == a2,
            (Type::Struct { name: n1, fields: f1 }, Type::Struct { name: n2, fields: f2 }) => {
                n1 == n2 && f1 == f2
            }
            (Type::Enum { name: n1, variants: v1 }, Type::Enum { name: n2, variants: v2 }) => {
                n1 == n2 && v1 == v2
            }
            (Type::Ref(a), Type::Ref(b)) => a == b,
            (Type::RefMut(a), Type::RefMut(b)) => a == b,
            (Type::Array(t1, s1), Type::Array(t2, s2)) => t1 == t2 && s1 == s2,
            // Refined types are equal if base types are equal
            // (constraints are semantic, not structural)
            (Type::Refined { base: b1, .. }, Type::Refined { base: b2, .. }) => b1 == b2,
            (Type::Refined { base, .. }, other) | (other, Type::Refined { base, .. }) => {
                base.as_ref() == other
            }
            // v0.20.0: Fn type equality
            (Type::Fn { params: p1, ret: r1 }, Type::Fn { params: p2, ret: r2 }) => {
                p1 == p2 && r1 == r2
            }
            // v0.31: Never type is compatible with any type (bottom type)
            (Type::Never, _) | (_, Type::Never) => true,
            // v0.37: Nullable type equality
            (Type::Nullable(a), Type::Nullable(b)) => a == b,
            // Nullable<T> is equivalent to Option<T>
            (Type::Nullable(inner), Type::Generic { name, type_args })
            | (Type::Generic { name, type_args }, Type::Nullable(inner))
                if name == "Option" && type_args.len() == 1 =>
            {
                inner.as_ref() == type_args[0].as_ref()
            }
            // v0.42: Tuple type equality
            (Type::Tuple(a), Type::Tuple(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Type {}

impl Type {
    /// Get the base type for refined types, or self for non-refined types.
    /// This is useful for type checking arithmetic/comparison operations.
    /// e.g., i64{it > 0}.base_type() returns &Type::I64
    pub fn base_type(&self) -> &Type {
        match self {
            Type::Refined { base, .. } => base.base_type(),
            _ => self,
        }
    }

    /// Check if this type is numeric (i32, i64, u32, u64, f64) including refined numeric types
    pub fn is_numeric(&self) -> bool {
        // v0.38: Include unsigned types
        matches!(self.base_type(), Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64)
    }

    /// Check if this type is comparable (numeric, bool, string) including refined types
    pub fn is_comparable(&self) -> bool {
        // v0.38: Include unsigned types
        // v0.64: Include Char type
        matches!(
            self.base_type(),
            Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 | Type::Bool | Type::String | Type::Char
        )
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            // v0.38: Unsigned types
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Unit => write!(f, "()"),
            Type::String => write!(f, "String"),
            // v0.64: Char type display
            Type::Char => write!(f, "char"),
            Type::Range(elem_ty) => write!(f, "Range<{elem_ty}>"),
            Type::Named(name) => write!(f, "{name}"),
            // v0.13.1: Type variable display
            Type::TypeVar(name) => write!(f, "{name}"),
            // v0.13.1: Generic type display
            Type::Generic { name, type_args } => {
                write!(f, "{name}<")?;
                for (i, arg) in type_args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ">")
            }
            Type::Struct { name, .. } => write!(f, "{name}"),
            Type::Enum { name, .. } => write!(f, "{name}"),
            Type::Ref(inner) => write!(f, "&{inner}"),
            Type::RefMut(inner) => write!(f, "&mut {inner}"),
            Type::Array(elem, size) => write!(f, "[{elem}; {size}]"),
            Type::Refined { base, constraints } => {
                write!(f, "{}{{", base)?;
                for (i, _) in constraints.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "<constraint>")?;
                }
                write!(f, "}}")
            }
            // v0.20.0: Fn type display
            Type::Fn { params, ret } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{param}")?;
                }
                write!(f, ") -> {ret}")
            }
            // v0.31: Never type display
            Type::Never => write!(f, "!"),
            // v0.37: Nullable type display
            Type::Nullable(inner) => write!(f, "{}?", inner),
            // v0.42: Tuple type display
            Type::Tuple(elems) => {
                write!(f, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{elem}")?;
                }
                write!(f, ")")
            }
        }
    }
}
