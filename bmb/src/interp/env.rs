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

    #[test]
    fn test_default() {
        let env = Environment::default();
        assert!(env.bindings().is_empty());
    }

    #[test]
    fn test_set_existing_variable() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Int(1));
        let updated = env.set("x", Value::Int(42));
        assert!(updated);
        assert_eq!(env.get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_set_nonexistent_variable() {
        let mut env = Environment::new();
        let updated = env.set("x", Value::Int(1));
        assert!(!updated);
    }

    #[test]
    fn test_set_in_parent_scope() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));

        let child = child_env(&parent);
        let updated = child.borrow_mut().set("x", Value::Int(99));
        assert!(updated);

        // Parent's x should be updated
        assert_eq!(parent.borrow().get("x"), Some(Value::Int(99)));
    }

    #[test]
    fn test_contains() {
        let mut env = Environment::new();
        assert!(!env.contains("x"));
        env.define("x".to_string(), Value::Int(1));
        assert!(env.contains("x"));
        assert!(!env.contains("y"));
    }

    #[test]
    fn test_contains_parent_chain() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));

        let child = child_env(&parent);
        assert!(child.borrow().contains("x"));
        assert!(!child.borrow().contains("y"));
    }

    #[test]
    fn test_bindings() {
        let mut env = Environment::new();
        env.define("a".to_string(), Value::Int(1));
        env.define("b".to_string(), Value::Int(2));
        assert_eq!(env.bindings().len(), 2);
    }

    #[test]
    fn test_three_level_scope_chain() {
        let grandparent = Environment::new().into_ref();
        grandparent.borrow_mut().define("x".to_string(), Value::Int(1));

        let parent = child_env(&grandparent);
        parent.borrow_mut().define("y".to_string(), Value::Int(2));

        let child = child_env(&parent);
        child.borrow_mut().define("z".to_string(), Value::Int(3));

        // Child sees all three
        assert_eq!(child.borrow().get("x"), Some(Value::Int(1)));
        assert_eq!(child.borrow().get("y"), Some(Value::Int(2)));
        assert_eq!(child.borrow().get("z"), Some(Value::Int(3)));

        // Grandparent sees only x
        assert_eq!(grandparent.borrow().get("x"), Some(Value::Int(1)));
        assert_eq!(grandparent.borrow().get("y"), None);
    }

    // --- Cycle 1227: Additional Environment Tests ---

    #[test]
    fn test_define_overwrite() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Int(1));
        env.define("x".to_string(), Value::Int(2));
        assert_eq!(env.get("x"), Some(Value::Int(2)));
    }

    #[test]
    fn test_define_multiple_types() {
        let mut env = Environment::new();
        env.define("a".to_string(), Value::Int(42));
        env.define("b".to_string(), Value::Bool(true));
        env.define("c".to_string(), Value::Float(3.14));
        env.define("d".to_string(), Value::Unit);
        assert_eq!(env.get("a"), Some(Value::Int(42)));
        assert_eq!(env.get("b"), Some(Value::Bool(true)));
        assert!(matches!(env.get("c"), Some(Value::Float(f)) if (f - 3.14).abs() < f64::EPSILON));
        assert_eq!(env.get("d"), Some(Value::Unit));
    }

    #[test]
    fn test_set_in_grandparent() {
        let grandparent = Environment::new().into_ref();
        grandparent.borrow_mut().define("x".to_string(), Value::Int(1));

        let parent = child_env(&grandparent);
        let child = child_env(&parent);

        // Set from child should update grandparent
        let updated = child.borrow_mut().set("x", Value::Int(99));
        assert!(updated);
        assert_eq!(grandparent.borrow().get("x"), Some(Value::Int(99)));
    }

    #[test]
    fn test_set_in_child_does_not_create() {
        let parent = Environment::new().into_ref();
        let child = child_env(&parent);

        // Setting a nonexistent variable should fail
        let updated = child.borrow_mut().set("x", Value::Int(1));
        assert!(!updated);
        assert_eq!(child.borrow().get("x"), None);
    }

    #[test]
    fn test_contains_three_levels() {
        let gp = Environment::new().into_ref();
        gp.borrow_mut().define("a".to_string(), Value::Int(1));

        let p = child_env(&gp);
        p.borrow_mut().define("b".to_string(), Value::Int(2));

        let c = child_env(&p);
        c.borrow_mut().define("c".to_string(), Value::Int(3));

        assert!(c.borrow().contains("a"));
        assert!(c.borrow().contains("b"));
        assert!(c.borrow().contains("c"));
        assert!(!c.borrow().contains("d"));
    }

    #[test]
    fn test_child_env_function() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));

        let c = child_env(&parent);
        // child_env creates a new environment with parent
        assert_eq!(c.borrow().get("x"), Some(Value::Int(1)));
        assert!(c.borrow().bindings().is_empty()); // No own bindings yet
    }

    #[test]
    fn test_into_ref() {
        let env = Environment::new();
        let env_ref = env.into_ref();
        // Should be usable as EnvRef
        env_ref.borrow_mut().define("x".to_string(), Value::Int(42));
        assert_eq!(env_ref.borrow().get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_with_parent_constructor() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("p".to_string(), Value::Bool(true));

        let child = Environment::with_parent(Rc::clone(&parent));
        assert_eq!(child.get("p"), Some(Value::Bool(true)));
        assert!(child.bindings().is_empty());
    }

    #[test]
    fn test_set_shadows_correctly() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));

        let child = child_env(&parent);
        // Define x in child (shadow)
        child.borrow_mut().define("x".to_string(), Value::Int(100));

        // Set should update child's x, not parent's
        child.borrow_mut().set("x", Value::Int(200));
        assert_eq!(child.borrow().get("x"), Some(Value::Int(200)));
        // Parent unchanged
        assert_eq!(parent.borrow().get("x"), Some(Value::Int(1)));
    }

    #[test]
    fn test_empty_environment_get() {
        let env = Environment::new();
        assert_eq!(env.get("anything"), None);
        assert!(!env.contains("anything"));
    }

    // ================================================================
    // Additional environment tests (Cycle 1235)
    // ================================================================

    #[test]
    fn test_clone_environment() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Int(42));
        let cloned = env.clone();
        assert_eq!(cloned.get("x"), Some(Value::Int(42)));
        // Modifying original doesn't affect clone
        env.define("x".to_string(), Value::Int(99));
        assert_eq!(cloned.get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_debug_format() {
        let env = Environment::new();
        let debug = format!("{:?}", env);
        assert!(debug.contains("Environment"));
    }

    #[test]
    fn test_define_str_value() {
        use std::rc::Rc;
        let mut env = Environment::new();
        env.define("name".to_string(), Value::Str(Rc::new("hello".to_string())));
        assert_eq!(env.get("name"), Some(Value::Str(Rc::new("hello".to_string()))));
    }

    #[test]
    fn test_set_updates_in_middle_scope() {
        let gp = Environment::new().into_ref();
        gp.borrow_mut().define("x".to_string(), Value::Int(1));

        let parent = child_env(&gp);
        parent.borrow_mut().define("x".to_string(), Value::Int(10));

        let child = child_env(&parent);

        // Set should update parent's x (nearest definition)
        child.borrow_mut().set("x", Value::Int(99));
        assert_eq!(parent.borrow().get("x"), Some(Value::Int(99)));
        // Grandparent unchanged
        assert_eq!(gp.borrow().get("x"), Some(Value::Int(1)));
    }

    #[test]
    fn test_bindings_returns_only_local() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("a".to_string(), Value::Int(1));

        let child = child_env(&parent);
        child.borrow_mut().define("b".to_string(), Value::Int(2));

        // bindings() only returns local bindings, not parent
        assert_eq!(child.borrow().bindings().len(), 1);
        assert!(child.borrow().bindings().contains_key("b"));
        assert!(!child.borrow().bindings().contains_key("a"));
    }

    #[test]
    fn test_multiple_children_independent() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("shared".to_string(), Value::Int(0));

        let child1 = child_env(&parent);
        let child2 = child_env(&parent);

        child1.borrow_mut().define("x".to_string(), Value::Int(1));
        child2.borrow_mut().define("y".to_string(), Value::Int(2));

        // Children don't see each other's bindings
        assert_eq!(child1.borrow().get("y"), None);
        assert_eq!(child2.borrow().get("x"), None);

        // Both see parent
        assert_eq!(child1.borrow().get("shared"), Some(Value::Int(0)));
        assert_eq!(child2.borrow().get("shared"), Some(Value::Int(0)));
    }

    #[test]
    fn test_set_returns_false_for_nonexistent_in_chain() {
        let gp = Environment::new().into_ref();
        let parent = child_env(&gp);
        let child = child_env(&parent);

        // Variable doesn't exist anywhere in chain
        assert!(!child.borrow_mut().set("missing", Value::Int(1)));
    }

    #[test]
    fn test_redefine_in_child_after_set_in_parent() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));

        let child = child_env(&parent);
        // Set parent's x through child
        child.borrow_mut().set("x", Value::Int(50));
        assert_eq!(parent.borrow().get("x"), Some(Value::Int(50)));

        // Now define x locally in child (shadows)
        child.borrow_mut().define("x".to_string(), Value::Int(100));
        assert_eq!(child.borrow().get("x"), Some(Value::Int(100)));
        // Parent still has 50
        assert_eq!(parent.borrow().get("x"), Some(Value::Int(50)));
    }

    #[test]
    fn test_with_parent_has_parent() {
        let parent = Environment::new().into_ref();
        let child = Environment::with_parent(Rc::clone(&parent));
        // child should be able to access parent bindings
        parent.borrow_mut().define("p".to_string(), Value::Bool(true));
        // Need to re-check since parent was modified after child creation
        // But child holds Rc to same parent, so it should see updates
        assert_eq!(child.get("p"), Some(Value::Bool(true)));
    }

    // ================================================================
    // Additional environment tests (Cycle 1241)
    // ================================================================

    #[test]
    fn test_env_new_no_parent() {
        let env = Environment::new();
        // New environment has no parent — get on unknown returns None
        assert_eq!(env.get("anything"), None);
        assert!(env.bindings().is_empty());
    }

    #[test]
    fn test_define_many_bindings() {
        let mut env = Environment::new();
        for i in 0..100 {
            env.define(format!("var_{}", i), Value::Int(i));
        }
        assert_eq!(env.bindings().len(), 100);
        assert_eq!(env.get("var_0"), Some(Value::Int(0)));
        assert_eq!(env.get("var_99"), Some(Value::Int(99)));
    }

    #[test]
    fn test_get_does_not_remove() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Int(42));
        // Multiple gets should all return the same value
        assert_eq!(env.get("x"), Some(Value::Int(42)));
        assert_eq!(env.get("x"), Some(Value::Int(42)));
        assert_eq!(env.get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_set_does_not_add_to_bindings_count() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Int(1));
        assert_eq!(env.bindings().len(), 1);
        env.set("x", Value::Int(2));
        // Set updates, doesn't add a new binding
        assert_eq!(env.bindings().len(), 1);
    }

    #[test]
    fn test_child_env_into_ref_usable() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("p".to_string(), Value::Int(1));
        let child = child_env(&parent);
        // Use child through EnvRef interface
        child.borrow_mut().define("c".to_string(), Value::Int(2));
        assert_eq!(child.borrow().get("p"), Some(Value::Int(1)));
        assert_eq!(child.borrow().get("c"), Some(Value::Int(2)));
    }

    #[test]
    fn test_define_unit_value() {
        let mut env = Environment::new();
        env.define("u".to_string(), Value::Unit);
        assert_eq!(env.get("u"), Some(Value::Unit));
    }

    #[test]
    fn test_four_level_scope_chain() {
        let l1 = Environment::new().into_ref();
        l1.borrow_mut().define("a".to_string(), Value::Int(1));

        let l2 = child_env(&l1);
        l2.borrow_mut().define("b".to_string(), Value::Int(2));

        let l3 = child_env(&l2);
        l3.borrow_mut().define("c".to_string(), Value::Int(3));

        let l4 = child_env(&l3);
        l4.borrow_mut().define("d".to_string(), Value::Int(4));

        // Deepest level sees all
        assert_eq!(l4.borrow().get("a"), Some(Value::Int(1)));
        assert_eq!(l4.borrow().get("b"), Some(Value::Int(2)));
        assert_eq!(l4.borrow().get("c"), Some(Value::Int(3)));
        assert_eq!(l4.borrow().get("d"), Some(Value::Int(4)));
    }

    #[test]
    fn test_contains_shadowed_variable() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("x".to_string(), Value::Int(1));
        let child = child_env(&parent);
        child.borrow_mut().define("x".to_string(), Value::Int(2));
        // Both levels contain "x"
        assert!(child.borrow().contains("x"));
        assert!(parent.borrow().contains("x"));
    }

    #[test]
    fn test_set_bool_value_in_parent() {
        let parent = Environment::new().into_ref();
        parent.borrow_mut().define("flag".to_string(), Value::Bool(false));
        let child = child_env(&parent);
        let updated = child.borrow_mut().set("flag", Value::Bool(true));
        assert!(updated);
        assert_eq!(parent.borrow().get("flag"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_default_same_as_new() {
        let a = Environment::new();
        let b = Environment::default();
        assert_eq!(a.bindings().len(), b.bindings().len());
        assert_eq!(a.get("x"), b.get("x"));
    }
}
