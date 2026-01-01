//! Environment for variable bindings

use super::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Shared reference to an environment
pub type EnvRef = Rc<RefCell<Environment>>;

/// Environment holding variable bindings
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variable bindings in this scope
    bindings: HashMap<String, Value>,
    /// Parent environment for lexical scoping
    parent: Option<EnvRef>,
}

impl Environment {
    /// Create a new global environment
    pub fn new() -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new environment with a parent
    pub fn with_parent(parent: EnvRef) -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Wrap in Rc<RefCell<>>
    pub fn into_ref(self) -> EnvRef {
        Rc::new(RefCell::new(self))
    }

    /// Define a new variable in the current scope
    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Look up a variable in the scope chain
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    /// Set/update a variable in the scope chain (v0.5 Phase 2)
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            false
        }
    }

    /// Check if a variable exists in the scope chain
    pub fn contains(&self, name: &str) -> bool {
        if self.bindings.contains_key(name) {
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow().contains(name)
        } else {
            false
        }
    }

    /// Get all bindings (for debugging)
    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a child environment from a parent reference
pub fn child_env(parent: &EnvRef) -> EnvRef {
    Environment::with_parent(Rc::clone(parent)).into_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Int(42));
        assert_eq!(env.get("x"), Some(Value::Int(42)));
        assert_eq!(env.get("y"), None);
    }

    #[test]
    fn test_scope_chain() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));

        let child = child_env(&parent);
        child.borrow_mut().define("y".to_string(), Value::Int(2));

        // Child can see parent's bindings
        assert_eq!(child.borrow().get("x"), Some(Value::Int(1)));
        assert_eq!(child.borrow().get("y"), Some(Value::Int(2)));

        // Parent cannot see child's bindings
        assert_eq!(parent.borrow().get("y"), None);
    }

    #[test]
    fn test_shadowing() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));

        let child = child_env(&parent);
        child.borrow_mut().define("x".to_string(), Value::Int(2));

        // Child sees its own x
        assert_eq!(child.borrow().get("x"), Some(Value::Int(2)));
        // Parent still has original x
        assert_eq!(parent.borrow().get("x"), Some(Value::Int(1)));
    }
}
