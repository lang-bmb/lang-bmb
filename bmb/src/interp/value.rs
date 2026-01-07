//! Runtime values for the interpreter

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

/// Runtime value
#[derive(Debug, Clone)]
pub enum Value {
    /// 64-bit integer (covers both i32 and i64)
    Int(i64),
    /// 64-bit floating point
    Float(f64),
    /// Boolean
    Bool(bool),
    /// String value (v0.5 Phase 2, v0.30.268: Rc for efficient cloning)
    Str(Rc<String>),
    /// Unit value
    Unit,
    /// Struct value: (type_name, fields)
    Struct(String, std::collections::HashMap<String, Value>),
    /// Enum variant: (enum_name, variant_name, values)
    Enum(String, String, Vec<Value>),
    /// Range value (v0.5 Phase 3): (start, end) exclusive end
    Range(i64, i64),
    /// Reference value (v0.5 Phase 5): points to a value
    Ref(Rc<RefCell<Value>>),
    /// Array value (v0.5 Phase 6): array of values
    Array(Vec<Value>),
}

impl Value {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Unit => false,
            Value::Struct(_, _) => true,
            Value::Enum(_, _, _) => true,
            Value::Range(start, end) => start < end,
            Value::Ref(r) => r.borrow().is_truthy(),
            Value::Array(arr) => !arr.is_empty(),
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Str(_) => "String",
            Value::Unit => "()",
            Value::Struct(name, _) => name,
            Value::Enum(name, _, _) => name,
            Value::Range(_, _) => "Range",
            Value::Ref(_) => "&ref",
            Value::Array(_) => "array",
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

    /// Try to convert to string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s.as_str()),
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
            Value::Str(s) => write!(f, "\"{s}\""),
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
            Value::Range(start, end) => write!(f, "{}..{}", start, end),
            Value::Ref(r) => write!(f, "&{}", r.borrow()),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
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
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Struct(n1, f1), Value::Struct(n2, f2)) => n1 == n2 && f1 == f2,
            (Value::Enum(e1, v1, a1), Value::Enum(e2, v2, a2)) => e1 == e2 && v1 == v2 && a1 == a2,
            (Value::Range(s1, e1), Value::Range(s2, e2)) => s1 == s2 && e1 == e2,
            (Value::Ref(r1), Value::Ref(r2)) => *r1.borrow() == *r2.borrow(),
            (Value::Array(a1), Value::Array(a2)) => a1 == a2,
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
        assert_eq!(format!("{}", Value::Str(Rc::new("hello".to_string()))), "\"hello\"");
    }

    #[test]
    fn test_value_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Str(Rc::new("hello".to_string())).is_truthy());
        assert!(!Value::Str(Rc::new("".to_string())).is_truthy());
    }
}
