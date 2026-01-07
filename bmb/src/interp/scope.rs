//! Scope Stack for efficient environment management
//! v0.30.279: Alternative to Rc<RefCell<Environment>> chain
//!
//! This module provides a stack-based scope management that allows
//! immediate memory deallocation when scopes exit, avoiding the
//! memory pressure caused by Rc chains in deep recursion.

use super::Value;
use std::collections::HashMap;

/// Stack-based scope management for interpreter
///
/// Unlike the Rc<RefCell<Environment>> chain approach, this uses a simple
/// Vec<HashMap> which allows immediate deallocation on scope exit.
#[derive(Debug)]
pub struct ScopeStack {
    /// Stack of scopes, index 0 is global
    scopes: Vec<HashMap<String, Value>>,
}

impl ScopeStack {
    /// Create a new scope stack with a global scope
    pub fn new() -> Self {
        ScopeStack {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push a new scope onto the stack
    /// Returns the new scope depth (for debugging)
    pub fn push_scope(&mut self) -> usize {
        self.scopes.push(HashMap::new());
        self.scopes.len() - 1
    }

    /// Pop the current scope from the stack
    /// Panics if trying to pop the global scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() <= 1 {
            panic!("Cannot pop global scope");
        }
        self.scopes.pop();
    }

    /// Current scope depth
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// Define a variable in the current (topmost) scope
    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Look up a variable, searching from current scope to global
    pub fn get(&self, name: &str) -> Option<Value> {
        // Search from top of stack (current scope) to bottom (global)
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Set/update a variable in the scope chain
    /// Returns true if variable was found and updated
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        // Search from top to bottom for existing binding
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    /// Check if a variable exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        self.scopes.iter().any(|scope| scope.contains_key(name))
    }

    /// Get bindings in the current scope (for debugging)
    pub fn current_bindings(&self) -> Option<&HashMap<String, Value>> {
        self.scopes.last()
    }

    /// Clear all scopes except global
    pub fn reset(&mut self) {
        self.scopes.truncate(1);
        self.scopes[0].clear();
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_basic_define_get() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(42));
        assert_eq!(stack.get("x"), Some(Value::Int(42)));
        assert_eq!(stack.get("y"), None);
    }

    #[test]
    fn test_scope_push_pop() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));

        // Push new scope
        stack.push_scope();
        stack.define("y".to_string(), Value::Int(2));

        // Both visible
        assert_eq!(stack.get("x"), Some(Value::Int(1)));
        assert_eq!(stack.get("y"), Some(Value::Int(2)));

        // Pop scope
        stack.pop_scope();

        // y is gone, x remains
        assert_eq!(stack.get("x"), Some(Value::Int(1)));
        assert_eq!(stack.get("y"), None);
    }

    #[test]
    fn test_shadowing() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));

        stack.push_scope();
        stack.define("x".to_string(), Value::Int(2));

        // Inner x shadows outer
        assert_eq!(stack.get("x"), Some(Value::Int(2)));

        stack.pop_scope();

        // Original x restored
        assert_eq!(stack.get("x"), Some(Value::Int(1)));
    }

    #[test]
    fn test_set_in_parent_scope() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));

        stack.push_scope();

        // Modify parent's x from child scope
        assert!(stack.set("x", Value::Int(99)));
        assert_eq!(stack.get("x"), Some(Value::Int(99)));

        stack.pop_scope();

        // Change persisted
        assert_eq!(stack.get("x"), Some(Value::Int(99)));
    }

    #[test]
    fn test_deep_nesting() {
        let mut stack = ScopeStack::new();

        // Create 1000 nested scopes
        for i in 0..1000 {
            stack.push_scope();
            stack.define(format!("var_{}", i), Value::Int(i));
        }

        assert_eq!(stack.depth(), 1001); // global + 1000

        // All variables accessible
        assert_eq!(stack.get("var_0"), Some(Value::Int(0)));
        assert_eq!(stack.get("var_999"), Some(Value::Int(999)));

        // Pop all scopes
        for _ in 0..1000 {
            stack.pop_scope();
        }

        assert_eq!(stack.depth(), 1);
        assert_eq!(stack.get("var_0"), None);
    }

    #[test]
    fn test_string_values() {
        let mut stack = ScopeStack::new();
        stack.define("s".to_string(), Value::Str(Rc::new("hello".to_string())));

        if let Some(Value::Str(s)) = stack.get("s") {
            assert_eq!(s.as_str(), "hello");
        } else {
            panic!("Expected string value");
        }
    }
}
