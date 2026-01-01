//! Runtime values for the interpreter

use std::fmt;

/// Runtime value
#[derive(Debug, Clone)]
pub enum Value {
    /// 64-bit integer (covers both i32 and i64)
    Int(i64),
    /// 64-bit floating point
    Float(f64),
    /// Boolean
    Bool(bool),
    /// Unit value
    Unit,
    /// Struct value: (type_name, fields)
    Struct(String, std::collections::HashMap<String, Value>),
    /// Enum variant: (enum_name, variant_name, values)
    Enum(String, String, Vec<Value>),
}

impl Value {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Unit => false,
            Value::Struct(_, _) => true,
            Value::Enum(_, _, _) => true,
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Unit => "()",
            Value::Struct(name, _) => name,
            Value::Enum(name, _, _) => name,
        }
    }

    /// Try to convert to i64
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to convert to f64
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Try to convert to bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Unit => write!(f, "()"),
            Value::Struct(name, fields) => {
                write!(f, "{} {{ ", name)?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, " }}")
            }
            Value::Enum(enum_name, variant, args) => {
                write!(f, "{}::{}", enum_name, variant)?;
                if !args.is_empty() {
                    write!(f, "(")?;
                    for (i, v) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", v)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Struct(n1, f1), Value::Struct(n2, f2)) => n1 == n2 && f1 == f2,
            (Value::Enum(e1, v1, a1), Value::Enum(e2, v2, a2)) => e1 == e2 && v1 == v2 && a1 == a2,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Unit), "()");
    }

    #[test]
    fn test_value_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
    }
}
