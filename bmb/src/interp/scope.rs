//! Scope Stack for efficient environment management
//! v0.30.279: Alternative to `Rc<RefCell<Environment>>` chain
//!
//! This module provides a stack-based scope management that allows
//! immediate memory deallocation when scopes exit, avoiding the
//! memory pressure caused by Rc chains in deep recursion.

use super::Value;
use std::collections::HashMap;

/// Stack-based scope management for interpreter
///
/// Unlike the `Rc<RefCell<Environment>>` chain approach, this uses a simple
/// `Vec<HashMap>` which allows immediate deallocation on scope exit.
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

    // ---- Cycle 75: Additional scope tests ----

    #[test]
    fn test_default() {
        let stack = ScopeStack::default();
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_contains() {
        let mut stack = ScopeStack::new();
        assert!(!stack.contains("x"));
        stack.define("x".to_string(), Value::Int(1));
        assert!(stack.contains("x"));
    }

    #[test]
    fn test_contains_in_parent() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        stack.push_scope();
        assert!(stack.contains("x")); // visible from child scope
    }

    #[test]
    fn test_set_returns_false_for_missing() {
        let mut stack = ScopeStack::new();
        assert!(!stack.set("x", Value::Int(42)));
    }

    #[test]
    fn test_set_updates_current_scope() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        assert!(stack.set("x", Value::Int(99)));
        assert_eq!(stack.get("x"), Some(Value::Int(99)));
    }

    #[test]
    fn test_current_bindings() {
        let mut stack = ScopeStack::new();
        stack.define("a".to_string(), Value::Int(1));
        stack.define("b".to_string(), Value::Int(2));

        let bindings = stack.current_bindings().unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings.get("a"), Some(&Value::Int(1)));
    }

    #[test]
    fn test_reset() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        stack.push_scope();
        stack.define("y".to_string(), Value::Int(2));
        stack.push_scope();

        stack.reset();
        assert_eq!(stack.depth(), 1);
        assert!(!stack.contains("x"));
        assert!(!stack.contains("y"));
    }

    #[test]
    fn test_push_scope_returns_depth() {
        let mut stack = ScopeStack::new();
        let d1 = stack.push_scope();
        let d2 = stack.push_scope();
        assert_eq!(d1, 1);
        assert_eq!(d2, 2);
    }

    #[test]
    #[should_panic(expected = "Cannot pop global scope")]
    fn test_pop_global_panics() {
        let mut stack = ScopeStack::new();
        stack.pop_scope(); // Should panic
    }

    #[test]
    fn test_multiple_values_same_scope() {
        let mut stack = ScopeStack::new();
        stack.define("a".to_string(), Value::Int(1));
        stack.define("b".to_string(), Value::Bool(true));
        stack.define("c".to_string(), Value::Float(1.5));

        assert_eq!(stack.get("a"), Some(Value::Int(1)));
        assert_eq!(stack.get("b"), Some(Value::Bool(true)));
        assert_eq!(stack.get("c"), Some(Value::Float(1.5)));
    }

    #[test]
    fn test_set_in_deeply_nested_scope() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(0));

        for _ in 0..10 {
            stack.push_scope();
        }

        assert!(stack.set("x", Value::Int(42)));
        assert_eq!(stack.get("x"), Some(Value::Int(42)));

        // Pop all child scopes
        for _ in 0..10 {
            stack.pop_scope();
        }

        // Change should persist
        assert_eq!(stack.get("x"), Some(Value::Int(42)));
    }

    // --- Cycle 1230: Additional ScopeStack Tests ---

    #[test]
    fn test_define_overwrites_in_same_scope() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        stack.define("x".to_string(), Value::Int(99));
        assert_eq!(stack.get("x"), Some(Value::Int(99)));
    }

    #[test]
    fn test_set_with_shadowing_updates_inner() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));

        stack.push_scope();
        stack.define("x".to_string(), Value::Int(2));

        // set should update the inner (topmost) x
        assert!(stack.set("x", Value::Int(99)));
        assert_eq!(stack.get("x"), Some(Value::Int(99)));

        stack.pop_scope();
        // Outer x should be unchanged
        assert_eq!(stack.get("x"), Some(Value::Int(1)));
    }

    #[test]
    fn test_current_bindings_in_child_scope() {
        let mut stack = ScopeStack::new();
        stack.define("global".to_string(), Value::Int(1));

        stack.push_scope();
        stack.define("local".to_string(), Value::Int(2));

        let bindings = stack.current_bindings().unwrap();
        // Current scope only has local
        assert_eq!(bindings.len(), 1);
        assert!(bindings.contains_key("local"));
        assert!(!bindings.contains_key("global"));
    }

    #[test]
    fn test_reset_clears_all_but_maintains_structure() {
        let mut stack = ScopeStack::new();
        stack.define("a".to_string(), Value::Int(1));
        stack.push_scope();
        stack.push_scope();

        stack.reset();
        assert_eq!(stack.depth(), 1);

        // Can still define and get after reset
        stack.define("b".to_string(), Value::Int(2));
        assert_eq!(stack.get("b"), Some(Value::Int(2)));
    }

    #[test]
    fn test_empty_scope_bindings() {
        let mut stack = ScopeStack::new();
        stack.push_scope();
        let bindings = stack.current_bindings().unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_contains_not_in_any_scope() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        stack.push_scope();
        stack.define("y".to_string(), Value::Int(2));

        assert!(!stack.contains("z"));
    }

    #[test]
    fn test_get_returns_none_after_pop() {
        let mut stack = ScopeStack::new();
        stack.push_scope();
        stack.define("temp".to_string(), Value::Int(42));
        assert_eq!(stack.get("temp"), Some(Value::Int(42)));

        stack.pop_scope();
        assert_eq!(stack.get("temp"), None);
    }

    #[test]
    fn test_multiple_push_pop_cycles() {
        let mut stack = ScopeStack::new();
        stack.define("persistent".to_string(), Value::Int(1));

        for i in 0..5 {
            stack.push_scope();
            stack.define(format!("temp_{}", i), Value::Int(i));
            assert_eq!(stack.get(&format!("temp_{}", i)), Some(Value::Int(i)));
            stack.pop_scope();
            assert_eq!(stack.get(&format!("temp_{}", i)), None);
        }

        assert_eq!(stack.get("persistent"), Some(Value::Int(1)));
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_scope_depth_tracking() {
        let mut stack = ScopeStack::new();
        assert_eq!(stack.depth(), 1);

        let d1 = stack.push_scope();
        assert_eq!(d1, 1);
        assert_eq!(stack.depth(), 2);

        let d2 = stack.push_scope();
        assert_eq!(d2, 2);
        assert_eq!(stack.depth(), 3);

        stack.pop_scope();
        assert_eq!(stack.depth(), 2);

        stack.pop_scope();
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_unit_and_bool_values() {
        let mut stack = ScopeStack::new();
        stack.define("u".to_string(), Value::Unit);
        stack.define("b".to_string(), Value::Bool(false));

        assert_eq!(stack.get("u"), Some(Value::Unit));
        assert_eq!(stack.get("b"), Some(Value::Bool(false)));
    }

    // ================================================================
    // Additional ScopeStack tests (Cycle 1233)
    // ================================================================

    #[test]
    fn test_get_finds_nearest_shadow() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        stack.push_scope();
        stack.define("x".to_string(), Value::Int(2));
        stack.push_scope();
        stack.define("x".to_string(), Value::Int(3));

        // Should find the innermost (nearest) shadow
        assert_eq!(stack.get("x"), Some(Value::Int(3)));

        stack.pop_scope();
        assert_eq!(stack.get("x"), Some(Value::Int(2)));

        stack.pop_scope();
        assert_eq!(stack.get("x"), Some(Value::Int(1)));
    }

    #[test]
    fn test_define_in_child_does_not_affect_parent() {
        let mut stack = ScopeStack::new();
        stack.push_scope();
        stack.define("child_only".to_string(), Value::Int(99));
        stack.pop_scope();

        // Parent (global) should not have the child's variable
        assert!(!stack.contains("child_only"));
        assert_eq!(stack.get("child_only"), None);
    }

    #[test]
    fn test_set_does_not_create_variable() {
        let mut stack = ScopeStack::new();
        // Attempting to set a non-existent variable should fail
        assert!(!stack.set("nonexistent", Value::Int(42)));
        // And should not create it
        assert!(!stack.contains("nonexistent"));
        assert_eq!(stack.get("nonexistent"), None);
    }

    #[test]
    fn test_contains_after_reset() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        stack.push_scope();
        stack.define("y".to_string(), Value::Int(2));

        assert!(stack.contains("x"));
        assert!(stack.contains("y"));

        stack.reset();
        assert!(!stack.contains("x"));
        assert!(!stack.contains("y"));
    }

    #[test]
    fn test_push_pop_preserves_global_defines() {
        let mut stack = ScopeStack::new();
        stack.define("global_var".to_string(), Value::Int(100));

        // Multiple push/pop cycles shouldn't affect global
        for _ in 0..10 {
            stack.push_scope();
            assert_eq!(stack.get("global_var"), Some(Value::Int(100)));
            stack.pop_scope();
        }

        assert_eq!(stack.get("global_var"), Some(Value::Int(100)));
    }

    #[test]
    fn test_char_value_in_scope() {
        let mut stack = ScopeStack::new();
        stack.define("ch".to_string(), Value::Char('z'));
        assert_eq!(stack.get("ch"), Some(Value::Char('z')));
    }

    #[test]
    fn test_set_updates_middle_scope() {
        let mut stack = ScopeStack::new();
        // Global scope
        stack.define("a".to_string(), Value::Int(1));

        // Middle scope
        stack.push_scope();
        stack.define("b".to_string(), Value::Int(2));

        // Inner scope
        stack.push_scope();
        // Set 'b' which lives in middle scope
        assert!(stack.set("b", Value::Int(99)));
        assert_eq!(stack.get("b"), Some(Value::Int(99)));

        stack.pop_scope(); // pop inner
        assert_eq!(stack.get("b"), Some(Value::Int(99))); // middle scope updated

        stack.pop_scope(); // pop middle
        assert_eq!(stack.get("b"), None); // b gone with middle scope
    }

    #[test]
    fn test_depth_after_reset_and_push() {
        let mut stack = ScopeStack::new();
        stack.push_scope();
        stack.push_scope();
        assert_eq!(stack.depth(), 3);

        stack.reset();
        assert_eq!(stack.depth(), 1);

        stack.push_scope();
        assert_eq!(stack.depth(), 2);
    }

    #[test]
    fn test_define_multiple_types_same_scope() {
        let mut stack = ScopeStack::new();
        stack.define("i".to_string(), Value::Int(42));
        stack.define("f".to_string(), Value::Float(3.14));
        stack.define("b".to_string(), Value::Bool(true));
        stack.define("s".to_string(), Value::Str(Rc::new("hello".to_string())));
        stack.define("u".to_string(), Value::Unit);
        stack.define("c".to_string(), Value::Char('x'));

        assert_eq!(stack.get("i"), Some(Value::Int(42)));
        assert_eq!(stack.get("f"), Some(Value::Float(3.14)));
        assert_eq!(stack.get("b"), Some(Value::Bool(true)));
        assert_eq!(stack.get("u"), Some(Value::Unit));
        assert_eq!(stack.get("c"), Some(Value::Char('x')));
        if let Some(Value::Str(s)) = stack.get("s") {
            assert_eq!(s.as_str(), "hello");
        } else {
            panic!("Expected Str value");
        }

        let bindings = stack.current_bindings().unwrap();
        assert_eq!(bindings.len(), 6);
    }

    // ================================================================
    // Additional ScopeStack tests (Cycle 1242)
    // ================================================================

    #[test]
    fn test_scope_stack_debug_format() {
        let stack = ScopeStack::new();
        let debug = format!("{:?}", stack);
        assert!(debug.contains("ScopeStack"));
        assert!(debug.contains("scopes"));
    }

    #[test]
    fn test_define_and_get_bool_values() {
        let mut stack = ScopeStack::new();
        stack.define("t".to_string(), Value::Bool(true));
        stack.define("f".to_string(), Value::Bool(false));
        assert_eq!(stack.get("t"), Some(Value::Bool(true)));
        assert_eq!(stack.get("f"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_new_has_exactly_one_scope() {
        let stack = ScopeStack::new();
        assert_eq!(stack.depth(), 1);
        assert!(stack.current_bindings().is_some());
        assert!(stack.current_bindings().unwrap().is_empty());
    }

    #[test]
    fn test_current_bindings_after_define_correct_count() {
        let mut stack = ScopeStack::new();
        stack.define("a".to_string(), Value::Int(1));
        stack.define("b".to_string(), Value::Int(2));
        stack.define("c".to_string(), Value::Int(3));
        assert_eq!(stack.current_bindings().unwrap().len(), 3);
    }

    #[test]
    fn test_set_returns_true_for_current_scope() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        assert!(stack.set("x", Value::Int(2)));
        assert_eq!(stack.get("x"), Some(Value::Int(2)));
    }

    #[test]
    fn test_push_pop_depth_consistency() {
        let mut stack = ScopeStack::new();
        assert_eq!(stack.depth(), 1);
        stack.push_scope();
        assert_eq!(stack.depth(), 2);
        stack.push_scope();
        assert_eq!(stack.depth(), 3);
        stack.pop_scope();
        assert_eq!(stack.depth(), 2);
        stack.pop_scope();
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn test_contains_after_define_and_pop() {
        let mut stack = ScopeStack::new();
        stack.push_scope();
        stack.define("temp".to_string(), Value::Int(1));
        assert!(stack.contains("temp"));
        stack.pop_scope();
        assert!(!stack.contains("temp"));
    }

    #[test]
    fn test_reset_then_define_works() {
        let mut stack = ScopeStack::new();
        stack.define("old".to_string(), Value::Int(1));
        stack.push_scope();
        stack.reset();

        // After reset, should be able to define fresh variables
        stack.define("new".to_string(), Value::Int(99));
        assert_eq!(stack.get("new"), Some(Value::Int(99)));
        assert_eq!(stack.get("old"), None);
    }

    #[test]
    fn test_get_empty_scope_chain_returns_none() {
        let stack = ScopeStack::new();
        assert_eq!(stack.get("anything"), None);
        assert_eq!(stack.get(""), None);
    }

    #[test]
    fn test_set_with_two_level_nesting_updates_correct_scope() {
        let mut stack = ScopeStack::new();
        stack.define("x".to_string(), Value::Int(1));
        stack.push_scope();
        stack.define("y".to_string(), Value::Int(2));

        // Update x (in global) from child scope
        assert!(stack.set("x", Value::Int(10)));
        // Update y (in child) from child scope
        assert!(stack.set("y", Value::Int(20)));

        assert_eq!(stack.get("x"), Some(Value::Int(10)));
        assert_eq!(stack.get("y"), Some(Value::Int(20)));

        stack.pop_scope();
        // x persists, y is gone
        assert_eq!(stack.get("x"), Some(Value::Int(10)));
        assert_eq!(stack.get("y"), None);
    }
}
