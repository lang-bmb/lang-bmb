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
    /// Character value (v0.64)
    Char(char),
    /// String rope for lazy concatenation (v0.30.283)
    /// Stores fragments that are concatenated only when materialized
    StringRope(Rc<RefCell<Vec<Rc<String>>>>),
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
    /// Tuple value (v0.42): heterogeneous fixed-size collection
    Tuple(Vec<Value>),
}

impl Value {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            // v0.64: Char is truthy if not null char
            Value::Char(c) => *c != '\0',
            Value::StringRope(fragments) => !fragments.borrow().is_empty(),
            Value::Unit => false,
            Value::Struct(_, _) => true,
            Value::Enum(_, _, _) => true,
            Value::Range(start, end) => start < end,
            Value::Ref(r) => r.borrow().is_truthy(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Tuple(_) => true, // Tuples are always truthy
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Str(_) => "String",
            // v0.64: Char type
            Value::Char(_) => "char",
            Value::StringRope(_) => "String",  // Same type name for user
            Value::Unit => "()",
            Value::Struct(name, _) => name,
            Value::Enum(name, _, _) => name,
            Value::Range(_, _) => "Range",
            Value::Ref(_) => "&ref",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
        }
    }

    /// Materialize a StringRope into a regular String (v0.30.283)
    pub fn materialize_string(&self) -> Option<String> {
        match self {
            Value::Str(s) => Some(s.as_ref().clone()),
            Value::StringRope(fragments) => {
                let frags = fragments.borrow();
                let total_len: usize = frags.iter().map(|s| s.len()).sum();
                let mut result = String::with_capacity(total_len);
                for frag in frags.iter() {
                    result.push_str(frag);
                }
                Some(result)
            }
            _ => None,
        }
    }

    /// Create a StringRope from two string values (v0.30.283)
    pub fn concat_strings(a: &Value, b: &Value) -> Option<Value> {
        let mut fragments: Vec<Rc<String>> = Vec::new();

        // Collect fragments from first operand
        match a {
            Value::Str(s) => fragments.push(s.clone()),
            Value::StringRope(frags) => {
                fragments.extend(frags.borrow().iter().cloned());
            }
            _ => return None,
        }

        // Collect fragments from second operand
        match b {
            Value::Str(s) => fragments.push(s.clone()),
            Value::StringRope(frags) => {
                fragments.extend(frags.borrow().iter().cloned());
            }
            _ => return None,
        }

        Some(Value::StringRope(Rc::new(RefCell::new(fragments))))
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
            // v0.64: Char display
            Value::Char(c) => write!(f, "'{c}'"),
            Value::StringRope(fragments) => {
                // Materialize for display (v0.30.283)
                let frags = fragments.borrow();
                let total_len: usize = frags.iter().map(|s| s.len()).sum();
                let mut result = String::with_capacity(total_len);
                for frag in frags.iter() {
                    result.push_str(frag);
                }
                write!(f, "\"{}\"", result)
            }
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
            // v0.42: Tuple display
            Value::Tuple(elems) => {
                write!(f, "(")?;
                for (i, v) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                // Add trailing comma for 1-tuple to distinguish from grouped expression
                if elems.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
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
            // v0.64: Character equality (by Unicode codepoint)
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            // StringRope comparisons (v0.30.283): materialize and compare
            (Value::StringRope(a), Value::StringRope(b)) => {
                let a_frags = a.borrow();
                let b_frags = b.borrow();
                if a_frags.len() == b_frags.len() {
                    a_frags.iter().zip(b_frags.iter()).all(|(x, y)| x == y)
                } else {
                    // Different fragment counts, materialize to compare
                    let a_str: String = a_frags.iter().map(|s| s.as_str()).collect();
                    let b_str: String = b_frags.iter().map(|s| s.as_str()).collect();
                    a_str == b_str
                }
            }
            (Value::Str(a), Value::StringRope(b)) => {
                let b_str: String = b.borrow().iter().map(|s| s.as_str()).collect();
                a.as_str() == b_str
            }
            (Value::StringRope(a), Value::Str(b)) => {
                let a_str: String = a.borrow().iter().map(|s| s.as_str()).collect();
                a_str == b.as_str()
            }
            (Value::Unit, Value::Unit) => true,
            (Value::Struct(n1, f1), Value::Struct(n2, f2)) => n1 == n2 && f1 == f2,
            (Value::Enum(e1, v1, a1), Value::Enum(e2, v2, a2)) => e1 == e2 && v1 == v2 && a1 == a2,
            (Value::Range(s1, e1), Value::Range(s2, e2)) => s1 == s2 && e1 == e2,
            (Value::Ref(r1), Value::Ref(r2)) => *r1.borrow() == *r2.borrow(),
            (Value::Array(a1), Value::Array(a2)) => a1 == a2,
            // v0.42: Tuple equality
            (Value::Tuple(t1), Value::Tuple(t2)) => t1 == t2,
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
        assert_eq!(format!("{}", Value::Float(3.5)), "3.5");
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

    // v0.30.283: StringRope tests
    #[test]
    fn test_string_rope_concat() {
        let a = Value::Str(Rc::new("hello".to_string()));
        let b = Value::Str(Rc::new(" world".to_string()));
        let result = Value::concat_strings(&a, &b).unwrap();
        assert_eq!(result.materialize_string().unwrap(), "hello world");
    }

    #[test]
    fn test_string_rope_multi_concat() {
        let a = Value::Str(Rc::new("a".to_string()));
        let b = Value::Str(Rc::new("b".to_string()));
        let c = Value::Str(Rc::new("c".to_string()));

        let ab = Value::concat_strings(&a, &b).unwrap();
        let abc = Value::concat_strings(&ab, &c).unwrap();

        assert_eq!(abc.materialize_string().unwrap(), "abc");
    }

    #[test]
    fn test_string_rope_equality() {
        let a = Value::Str(Rc::new("hello".to_string()));
        let b = Value::Str(Rc::new("hello".to_string()));
        let rope = Value::concat_strings(
            &Value::Str(Rc::new("hel".to_string())),
            &Value::Str(Rc::new("lo".to_string()))
        ).unwrap();

        assert_eq!(a, b);
        assert_eq!(a, rope);
        assert_eq!(rope, a);
    }

    #[test]
    fn test_string_rope_display() {
        let rope = Value::concat_strings(
            &Value::Str(Rc::new("foo".to_string())),
            &Value::Str(Rc::new("bar".to_string()))
        ).unwrap();
        assert_eq!(format!("{}", rope), "\"foobar\"");
    }

    #[test]
    fn test_string_rope_memory_efficiency() {
        // Simulate many concatenations (like bootstrap's 253)
        let mut result = Value::Str(Rc::new(String::new()));
        for i in 0..100 {
            let fragment = Value::Str(Rc::new(format!("fragment{}", i)));
            result = Value::concat_strings(&result, &fragment).unwrap();
        }

        // Should have 101 fragments (empty + 100 fragments)
        if let Value::StringRope(frags) = &result {
            assert_eq!(frags.borrow().len(), 101);
        }

        // Materialize should produce correct result
        let materialized = result.materialize_string().unwrap();
        assert!(materialized.starts_with("fragment0"));
        assert!(materialized.ends_with("fragment99"));
    }
}
