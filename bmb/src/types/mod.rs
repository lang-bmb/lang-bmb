//! Type checking

pub mod exhaustiveness;

use std::collections::HashMap;

use crate::ast::*;
use crate::error::{CompileError, CompileWarning, Result};
use crate::resolver::{Module, ResolvedImports};
use crate::util::{find_similar_name, format_suggestion_hint};

/// Trait method signature info (v0.20.1)
#[derive(Debug, Clone)]
pub struct TraitMethodInfo {
    /// Method name
    pub name: String,
    /// Parameter types (excluding self)
    pub param_types: Vec<Type>,
    /// Return type
    pub ret_type: Type,
}

/// Trait definition info (v0.20.1)
#[derive(Debug, Clone)]
pub struct TraitInfo {
    /// Trait name
    pub name: String,
    /// Type parameters
    pub type_params: Vec<TypeParam>,
    /// Method signatures
    pub methods: Vec<TraitMethodInfo>,
}

/// Impl block info (v0.20.1)
/// Stores the mapping from (type, trait) to implemented methods
#[derive(Debug, Clone)]
pub struct ImplInfo {
    /// Trait being implemented
    pub trait_name: String,
    /// Type implementing the trait
    pub target_type: Type,
    /// Implemented methods: name -> (param_types, ret_type)
    pub methods: HashMap<String, (Vec<Type>, Type)>,
}

// ============================================================================
// v0.48: Binding Usage Tracking
// ============================================================================

/// Tracks a single variable binding for unused detection
#[derive(Debug, Clone)]
struct BindingInfo {
    /// Location where the variable was bound
    span: Span,
    /// Whether this binding has been used
    used: bool,
    /// v0.52: Whether this is a mutable binding (var)
    is_mutable: bool,
    /// v0.52: Whether this binding has been mutated (assigned to)
    was_mutated: bool,
}

/// Tracks variable bindings and usage for unused warning detection (v0.48)
/// P0 Correctness: Detects unused variables at compile-time
#[derive(Debug, Default)]
struct BindingTracker {
    /// Stack of scopes, each containing bound variables
    /// Outer scope = index 0, inner scopes pushed on top
    scopes: Vec<HashMap<String, BindingInfo>>,
}

impl BindingTracker {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Start with global scope
        }
    }

    /// Enter a new scope (for blocks, match arms, closures)
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit current scope and return unused bindings and mutable-but-never-mutated bindings
    /// Returns: (unused_bindings, unused_mutable_bindings)
    fn pop_scope(&mut self) -> (Vec<(String, Span)>, Vec<(String, Span)>) {
        let scope = self.scopes.pop().unwrap_or_default();
        let mut unused = Vec::new();
        let mut unused_mut = Vec::new();

        for (name, info) in scope {
            // Skip underscore-prefixed names
            if name.starts_with('_') {
                continue;
            }

            // Check for unused binding
            if !info.used {
                unused.push((name.clone(), info.span));
            }

            // v0.52: Check for mutable but never mutated
            if info.is_mutable && !info.was_mutated {
                unused_mut.push((name, info.span));
            }
        }

        (unused, unused_mut)
    }

    /// Bind a variable in the current scope
    fn bind(&mut self, name: String, span: Span) {
        self.bind_with_mutability(name, span, false);
    }

    /// v0.52: Bind a variable with explicit mutability flag
    fn bind_with_mutability(&mut self, name: String, span: Span, is_mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, BindingInfo {
                span,
                used: false,
                is_mutable,
                was_mutated: false,
            });
        }
    }

    /// v0.79: Check if a name shadows a binding in an outer scope
    /// Returns the span of the original binding if it exists in an outer scope
    fn find_shadow(&self, name: &str) -> Option<Span> {
        // Skip if underscore-prefixed (intentionally ignored)
        if name.starts_with('_') {
            return None;
        }
        // Search all scopes except the current one (from second-to-last to first)
        for scope in self.scopes.iter().rev().skip(1) {
            if let Some(info) = scope.get(name) {
                return Some(info.span);
            }
        }
        None
    }

    /// Mark a variable as used (searches all scopes from innermost)
    fn mark_used(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                info.used = true;
                return;
            }
        }
    }

    /// v0.52: Mark a variable as mutated (assigned to after declaration)
    fn mark_mutated(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                info.was_mutated = true;
                return;
            }
        }
    }

    /// Check if a variable exists in any scope
    #[allow(dead_code)]
    fn is_bound(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains_key(name))
    }
}

/// Type checker
pub struct TypeChecker {
    /// Variable environment
    env: HashMap<String, Type>,
    /// Function signatures (non-generic)
    functions: HashMap<String, (Vec<Type>, Type)>,
    /// Generic function signatures: name -> (type_params, param_types, return_type)
    /// v0.15: Support for generic functions like `fn identity<T>(x: T) -> T`
    generic_functions: HashMap<String, (Vec<TypeParam>, Vec<Type>, Type)>,
    /// Generic struct definitions: name -> (type_params, fields)
    /// v0.15: Support for generic structs like `struct Container<T> { value: T }`
    generic_structs: HashMap<String, (Vec<TypeParam>, Vec<(String, Type)>)>,
    /// Struct definitions: name -> field types
    structs: HashMap<String, Vec<(String, Type)>>,
    /// Generic enum definitions: name -> (type_params, variants)
    /// v0.16: Support for generic enums like `enum Option<T> { Some(T), None }`
    generic_enums: HashMap<String, (Vec<TypeParam>, Vec<(String, Vec<Type>)>)>,
    /// Enum definitions: name -> variant info (variant_name, field types)
    enums: HashMap<String, Vec<(String, Vec<Type>)>>,
    /// Current function return type (for `ret` keyword)
    current_ret_ty: Option<Type>,
    /// Current type parameter environment (for checking generic function bodies)
    /// v0.15: Maps type parameter names to their bounds
    type_param_env: HashMap<String, Vec<String>>,
    /// Trait definitions (v0.20.1)
    /// trait_name -> TraitInfo
    traits: HashMap<String, TraitInfo>,
    /// Impl blocks (v0.20.1)
    /// (type_name, trait_name) -> ImplInfo
    impls: HashMap<(String, String), ImplInfo>,
    /// Collected warnings during type checking (v0.47)
    /// P0 Correctness: Non-fatal diagnostics for potential issues
    warnings: Vec<CompileWarning>,
    /// Variable binding tracker for unused detection (v0.48)
    /// P0 Correctness: Detects unused variables at compile-time
    binding_tracker: BindingTracker,
    /// v0.74: Set of imported names for tracking usage
    /// Contains names from `use` statements that may or may not be used
    imported_names: std::collections::HashSet<String>,
    /// v0.74: Set of names actually used during type checking
    /// Used to determine which imports are unused
    used_names: std::collections::HashSet<String>,
    /// v0.76: Private functions defined in the program (name -> span)
    /// Used for unused function detection
    private_functions: HashMap<String, Span>,
    /// v0.76: Functions that were called during type checking
    /// Used for unused function detection
    called_functions: std::collections::HashSet<String>,
    /// v0.77: Private structs defined in the program (name -> span)
    /// Used for unused type detection
    private_structs: HashMap<String, Span>,
    /// v0.78: Private enums defined in the program (name -> span)
    /// Used for unused enum detection
    private_enums: HashMap<String, Span>,
    /// v0.80: Private traits defined in the program (name -> span)
    /// Used for unused trait detection
    private_traits: HashMap<String, Span>,
    /// v0.80: Traits that have been implemented
    /// Used for unused trait detection
    implemented_traits: std::collections::HashSet<String>,
    /// v0.84: Functions with contracts for semantic duplication detection
    /// Key: (signature_hash, postcondition_hash) -> (first_function_name, span)
    /// Used to detect functions with equivalent contracts
    contract_signatures: HashMap<(String, String), (String, Span)>,
    /// v0.50.6: Type alias definitions
    /// name -> (type_params, target_type, refinement_expr, span)
    type_aliases: HashMap<String, (Vec<TypeParam>, Type, Option<Expr>, Span)>,
    /// v0.50.11: Function definition spans for duplicate detection
    /// name -> span of first definition
    function_spans: HashMap<String, Span>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        // Register built-in functions
        // print(x) -> Unit
        functions.insert("print".to_string(), (vec![Type::I64], Type::Unit));
        // println(x) -> Unit
        functions.insert("println".to_string(), (vec![Type::I64], Type::Unit));
        // v0.31.21: print_str(s: String) -> i64 (for gotgan string output)
        functions.insert("print_str".to_string(), (vec![Type::String], Type::I64));
        // v0.100: println_str(s: String) -> Unit
        functions.insert("println_str".to_string(), (vec![Type::String], Type::Unit));
        // v0.60.44: Float output functions for spectral_norm, n_body benchmarks
        functions.insert("println_f64".to_string(), (vec![Type::F64], Type::Unit));
        functions.insert("print_f64".to_string(), (vec![Type::F64], Type::Unit));
        // assert(cond) -> Unit
        functions.insert("assert".to_string(), (vec![Type::Bool], Type::Unit));
        // read_int() -> i64
        functions.insert("read_int".to_string(), (vec![], Type::I64));
        // abs(n) -> i64
        functions.insert("abs".to_string(), (vec![Type::I64], Type::I64));
        // min(a, b) -> i64
        functions.insert("min".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // max(a, b) -> i64
        functions.insert("max".to_string(), (vec![Type::I64, Type::I64], Type::I64));

        // v0.31.10: File I/O builtins for Phase 32.0 Bootstrap Infrastructure
        // read_file(path: String) -> String
        functions.insert("read_file".to_string(), (vec![Type::String], Type::String));
        // write_file(path: String, content: String) -> i64 (0 = success, -1 = error)
        functions.insert("write_file".to_string(), (vec![Type::String, Type::String], Type::I64));
        // v0.60.80: write_file_newlines - converts | to newlines (for bootstrap compiler)
        functions.insert("write_file_newlines".to_string(), (vec![Type::String, Type::String], Type::I64));
        // append_file(path: String, content: String) -> i64
        functions.insert("append_file".to_string(), (vec![Type::String, Type::String], Type::I64));
        // file_exists(path: String) -> i64 (1 = exists, 0 = not found)
        functions.insert("file_exists".to_string(), (vec![Type::String], Type::I64));
        // file_size(path: String) -> i64 (-1 = error)
        functions.insert("file_size".to_string(), (vec![Type::String], Type::I64));

        // v0.31.11: Process execution builtins for Phase 32.0.2 Bootstrap Infrastructure
        // exec(command: String, args: String) -> i64 (exit code)
        functions.insert("exec".to_string(), (vec![Type::String, Type::String], Type::I64));
        // exec_output(command: String, args: String) -> String (stdout)
        functions.insert("exec_output".to_string(), (vec![Type::String, Type::String], Type::String));
        // system(command: String) -> i64 (exit code via shell)
        functions.insert("system".to_string(), (vec![Type::String], Type::I64));
        // v0.88.2: system_capture(command: String) -> String (capture stdout)
        functions.insert("system_capture".to_string(), (vec![Type::String], Type::String));
        // getenv(name: String) -> String (env var value)
        functions.insert("getenv".to_string(), (vec![Type::String], Type::String));

        // v0.88.2: Memory management functions
        functions.insert("free_string".to_string(), (vec![Type::String], Type::I64));
        functions.insert("sb_free".to_string(), (vec![Type::I64], Type::I64));
        functions.insert("arena_mode".to_string(), (vec![Type::I64], Type::I64));
        functions.insert("arena_reset".to_string(), (vec![], Type::I64));
        functions.insert("arena_usage".to_string(), (vec![], Type::I64));
        functions.insert("arena_save".to_string(), (vec![], Type::I64));
        functions.insert("arena_restore".to_string(), (vec![], Type::I64));
        // bmb_ prefixed variants (used by bootstrap compiler.bmb)
        functions.insert("bmb_arena_save".to_string(), (vec![], Type::I64));
        functions.insert("bmb_arena_restore".to_string(), (vec![], Type::I64));
        functions.insert("bmb_sb_contains".to_string(), (vec![Type::I64, Type::String], Type::I64));
        functions.insert("sb_contains".to_string(), (vec![Type::I64, Type::String], Type::I64));

        // v0.63: Timing builtin for bmb-bench benchmark framework
        // time_ns() -> i64 (nanoseconds since epoch)
        functions.insert("time_ns".to_string(), (vec![], Type::I64));

        // v0.31.22: Command-line argument builtins for Phase 32.3.D CLI Independence
        // arg_count() -> i64 (number of arguments including program name)
        functions.insert("arg_count".to_string(), (vec![], Type::I64));
        // get_arg(n: i64) -> String (nth argument, 0 = program name)
        functions.insert("get_arg".to_string(), (vec![Type::I64], Type::String));

        // v0.31.13: StringBuilder builtins for Phase 32.0.4 O(n²) fix
        // sb_new() -> i64 (builder ID)
        functions.insert("sb_new".to_string(), (vec![], Type::I64));
        // v0.51.45: sb_with_capacity(capacity: i64) -> i64 (builder ID with pre-allocated capacity)
        functions.insert("sb_with_capacity".to_string(), (vec![Type::I64], Type::I64));
        // sb_push(id: i64, str: String) -> i64 (same ID for chaining)
        functions.insert("sb_push".to_string(), (vec![Type::I64, Type::String], Type::I64));
        // sb_push_char(id: i64, char_code: i64) -> i64 (push single char, no allocation)
        functions.insert("sb_push_char".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // v0.50.73: sb_push_int(id: i64, n: i64) -> i64 (push integer directly, O(1) allocation)
        functions.insert("sb_push_int".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // v0.50.74: sb_push_escaped(id: i64, str: String) -> i64 (escape & push entire string in one call)
        functions.insert("sb_push_escaped".to_string(), (vec![Type::I64, Type::String], Type::I64));
        // sb_build(id: i64) -> String (final string)
        functions.insert("sb_build".to_string(), (vec![Type::I64], Type::String));
        // sb_len(id: i64) -> i64 (total length)
        functions.insert("sb_len".to_string(), (vec![Type::I64], Type::I64));
        // sb_clear(id: i64) -> i64 (same ID)
        functions.insert("sb_clear".to_string(), (vec![Type::I64], Type::I64));
        // sb_println(id: i64) -> i64 (v0.60.64: print without allocation)
        functions.insert("sb_println".to_string(), (vec![Type::I64], Type::I64));
        // puts_cstr(ptr: i64) -> i64 (v0.60.65: print C string from raw pointer)
        functions.insert("puts_cstr".to_string(), (vec![Type::I64], Type::I64));

        // v0.60.246: String-key HashMap for O(1) lookups in bootstrap compiler
        // (Named strmap_* to avoid conflict with existing i64-key hashmap_* functions)
        // strmap_new() -> i64 (create new hashmap, returns handle)
        functions.insert("strmap_new".to_string(), (vec![], Type::I64));
        // strmap_insert(handle: i64, key: String, value: i64) -> i64 (returns 1 on success)
        functions.insert("strmap_insert".to_string(), (vec![Type::I64, Type::String, Type::I64], Type::I64));
        // strmap_get(handle: i64, key: String) -> i64 (returns value or -1 if not found)
        functions.insert("strmap_get".to_string(), (vec![Type::I64, Type::String], Type::I64));
        // strmap_contains(handle: i64, key: String) -> i64 (returns 1 if found, 0 if not)
        functions.insert("strmap_contains".to_string(), (vec![Type::I64, Type::String], Type::I64));
        // strmap_size(handle: i64) -> i64 (returns number of entries)
        functions.insert("strmap_size".to_string(), (vec![Type::I64], Type::I64));

        // v0.50.36: find_close_paren is now defined in BMB, not as builtin

        // v0.31.21: Character conversion builtins
        // v0.50.18: chr returns String to match runtime behavior (C runtime returns char*)
        // chr(code: i64) -> String (creates single-char string from code point)
        functions.insert("chr".to_string(), (vec![Type::I64], Type::String));
        // ord(c: char) -> i64 (character to Unicode codepoint)
        functions.insert("ord".to_string(), (vec![Type::Char], Type::I64));

        // v0.66: String-char interop utilities
        // char_at(s: String, idx: i64) -> char (get character at index, Unicode-aware)
        functions.insert("char_at".to_string(), (vec![Type::String, Type::I64], Type::Char));
        // char_to_string(c: char) -> String (convert character to single-char string)
        functions.insert("char_to_string".to_string(), (vec![Type::Char], Type::String));
        // str_len(s: String) -> i64 (Unicode character count, O(n))
        // Note: s.len() returns byte length (O(1)), str_len returns char count
        functions.insert("str_len".to_string(), (vec![Type::String], Type::I64));

        // v0.34: Math intrinsics for Phase 34.4 Benchmark Gate (n_body, mandelbrot_fp)
        // sqrt(x: f64) -> f64 (square root)
        functions.insert("sqrt".to_string(), (vec![Type::F64], Type::F64));
        // i64_to_f64(x: i64) -> f64 (type conversion)
        functions.insert("i64_to_f64".to_string(), (vec![Type::I64], Type::F64));
        // f64_to_i64(x: f64) -> i64 (type conversion, truncates toward zero)
        functions.insert("f64_to_i64".to_string(), (vec![Type::F64], Type::I64));

        // v0.51.47: i32 conversion functions for performance-critical code
        // i32_to_f64(x: i32) -> f64 (type conversion)
        functions.insert("i32_to_f64".to_string(), (vec![Type::I32], Type::F64));
        // i32_to_i64(x: i32) -> i64 (sign extension)
        functions.insert("i32_to_i64".to_string(), (vec![Type::I32], Type::I64));
        // i64_to_i32(x: i64) -> i32 (truncation)
        functions.insert("i64_to_i32".to_string(), (vec![Type::I64], Type::I32));

        // v0.34.2: Memory allocation builtins for Phase 34.2 Dynamic Collections
        // malloc(size: i64) -> i64 (pointer as integer)
        functions.insert("malloc".to_string(), (vec![Type::I64], Type::I64));
        // v0.89.4: free returns i64(0) so it can be used as expression without wrapper
        functions.insert("free".to_string(), (vec![Type::I64], Type::I64));
        // realloc(ptr: i64, new_size: i64) -> i64 (new pointer)
        functions.insert("realloc".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // calloc(count: i64, size: i64) -> i64 (zeroed memory pointer)
        functions.insert("calloc".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // v0.89.4: store builtins return i64(0) for expression use
        functions.insert("store_i64".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // load_i64(ptr: i64) -> i64 (read from memory)
        functions.insert("load_i64".to_string(), (vec![Type::I64], Type::I64));
        // v0.51.5: f64 memory operations for numerical benchmarks
        functions.insert("store_f64".to_string(), (vec![Type::I64, Type::F64], Type::I64));
        // load_f64(ptr: i64) -> f64 (read f64 from memory)
        functions.insert("load_f64".to_string(), (vec![Type::I64], Type::F64));
        // v0.51.51: Byte-level memory access for high-performance string parsing
        // load_u8(ptr: i64) -> i64 (read single byte from memory)
        functions.insert("load_u8".to_string(), (vec![Type::I64], Type::I64));
        functions.insert("store_u8".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // v0.60.58: 32-bit integer intrinsics for efficient struct packing
        // load_i32(ptr: i64) -> i64 (read 32-bit signed integer, sign-extended to i64)
        functions.insert("load_i32".to_string(), (vec![Type::I64], Type::I64));
        functions.insert("store_i32".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // str_data(s: String) -> i64 (get raw pointer to string data)
        functions.insert("str_data".to_string(), (vec![Type::String], Type::I64));
        // Box convenience functions
        // box_new_i64(value: i64) -> i64 (allocate + store)
        functions.insert("box_new_i64".to_string(), (vec![Type::I64], Type::I64));
        // box_get_i64(ptr: i64) -> i64 (alias for load_i64)
        functions.insert("box_get_i64".to_string(), (vec![Type::I64], Type::I64));
        // box_set_i64(ptr: i64, value: i64) -> i64 (alias for store_i64)
        functions.insert("box_set_i64".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // box_free_i64(ptr: i64) -> i64 (alias for free)
        functions.insert("box_free_i64".to_string(), (vec![Type::I64], Type::I64));

        // v0.34.2.3: Vec<i64> dynamic array builtins (RFC-0007)
        // vec_new() -> i64 (create empty vector, returns header pointer)
        functions.insert("vec_new".to_string(), (vec![], Type::I64));
        // vec_with_capacity(cap: i64) -> i64 (create vector with pre-allocated capacity)
        functions.insert("vec_with_capacity".to_string(), (vec![Type::I64], Type::I64));
        // vec_push(vec: i64, value: i64) -> Unit (append element with auto-grow)
        functions.insert("vec_push".to_string(), (vec![Type::I64, Type::I64], Type::Unit));
        // vec_pop(vec: i64) -> i64 (remove and return last element)
        functions.insert("vec_pop".to_string(), (vec![Type::I64], Type::I64));
        // vec_get(vec: i64, index: i64) -> i64 (read element at index)
        functions.insert("vec_get".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // vec_set(vec: i64, index: i64, value: i64) -> Unit (write element at index)
        functions.insert("vec_set".to_string(), (vec![Type::I64, Type::I64, Type::I64], Type::Unit));
        // vec_len(vec: i64) -> i64 (get current length)
        functions.insert("vec_len".to_string(), (vec![Type::I64], Type::I64));
        // vec_cap(vec: i64) -> i64 (get capacity)
        functions.insert("vec_cap".to_string(), (vec![Type::I64], Type::I64));
        // vec_free(vec: i64) -> Unit (deallocate vector and its data)
        functions.insert("vec_free".to_string(), (vec![Type::I64], Type::Unit));
        // vec_clear(vec: i64) -> Unit (set length to 0 without deallocating)
        functions.insert("vec_clear".to_string(), (vec![Type::I64], Type::Unit));

        // v0.34.24: Hash builtins
        // hash_i64(x: i64) -> i64 (hash function for integers)
        functions.insert("hash_i64".to_string(), (vec![Type::I64], Type::I64));

        // v0.34.24: HashMap<i64, i64> builtins (RFC-0007)
        // hashmap_new() -> i64 (create empty hashmap)
        functions.insert("hashmap_new".to_string(), (vec![], Type::I64));
        // hashmap_insert(map: i64, key: i64, value: i64) -> i64 (returns old value or 0)
        functions.insert("hashmap_insert".to_string(), (vec![Type::I64, Type::I64, Type::I64], Type::I64));
        // hashmap_get(map: i64, key: i64) -> i64 (returns value or i64::MIN if not found)
        functions.insert("hashmap_get".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // hashmap_contains(map: i64, key: i64) -> i64 (returns 1 if exists, 0 otherwise)
        functions.insert("hashmap_contains".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // hashmap_remove(map: i64, key: i64) -> i64 (returns removed value or i64::MIN)
        functions.insert("hashmap_remove".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // hashmap_len(map: i64) -> i64 (returns entry count)
        functions.insert("hashmap_len".to_string(), (vec![Type::I64], Type::I64));
        // hashmap_free(map: i64) -> Unit (deallocate hashmap)
        functions.insert("hashmap_free".to_string(), (vec![Type::I64], Type::Unit));

        // v0.34.24: HashSet<i64> builtins (thin wrapper around HashMap)
        // hashset_new() -> i64 (create empty hashset)
        functions.insert("hashset_new".to_string(), (vec![], Type::I64));
        // hashset_insert(set: i64, value: i64) -> i64 (returns 1 if new, 0 if existed)
        functions.insert("hashset_insert".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // hashset_contains(set: i64, value: i64) -> i64 (returns 1 if exists, 0 otherwise)
        functions.insert("hashset_contains".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // hashset_remove(set: i64, value: i64) -> i64 (returns 1 if removed, 0 if not found)
        functions.insert("hashset_remove".to_string(), (vec![Type::I64, Type::I64], Type::I64));
        // hashset_len(set: i64) -> i64 (returns entry count)
        functions.insert("hashset_len".to_string(), (vec![Type::I64], Type::I64));
        // hashset_free(set: i64) -> Unit (deallocate hashset)
        functions.insert("hashset_free".to_string(), (vec![Type::I64], Type::Unit));

        // v0.60.27: Initialize generic builtin functions
        let mut generic_functions = HashMap::new();

        // free<T>(ptr: *T) -> i64
        // Allows calling free with typed pointers directly, avoiding unnecessary inttoptr/ptrtoint
        // The i64 version above is kept for backward compatibility with legacy pointer code
        // v0.89.4: Generic free also returns i64(0) for consistency
        generic_functions.insert(
            "free".to_string(),
            (
                vec![TypeParam::new("T")],
                vec![Type::Ptr(Box::new(Type::TypeVar("T".to_string())))],
                Type::I64,
            ),
        );

        // v0.78: block_on<T>(future: Future<T>) -> T
        // Runs a future to completion and returns its result
        generic_functions.insert(
            "block_on".to_string(),
            (
                vec![TypeParam::new("T")],
                vec![Type::Future(Box::new(Type::TypeVar("T".to_string())))],
                Type::TypeVar("T".to_string()),
            ),
        );

        // v0.83: async_open(path: String) -> Future<AsyncFile>
        // Opens a file asynchronously for reading/writing
        functions.insert(
            "async_open".to_string(),
            (vec![Type::String], Type::Future(Box::new(Type::AsyncFile))),
        );

        // v0.83.1: tcp_connect(host: String, port: i64) -> Future<AsyncSocket>
        // Connects to a TCP server asynchronously
        functions.insert(
            "tcp_connect".to_string(),
            (vec![Type::String, Type::I64], Type::Future(Box::new(Type::AsyncSocket))),
        );

        // v0.84: thread_pool_new(size: i64) -> ThreadPool
        // Creates a new thread pool with the specified number of worker threads
        functions.insert(
            "thread_pool_new".to_string(),
            (vec![Type::I64], Type::ThreadPool),
        );

        // v0.85: thread_scope() -> Scope
        // Creates a new scope for scoped threads (structured concurrency)
        functions.insert(
            "thread_scope".to_string(),
            (vec![], Type::Scope),
        );

        Self {
            env: HashMap::new(),
            functions,
            generic_functions,
            generic_structs: HashMap::new(),
            structs: HashMap::new(),
            generic_enums: HashMap::new(),
            enums: HashMap::new(),
            current_ret_ty: None,
            type_param_env: HashMap::new(),
            traits: HashMap::new(),
            impls: HashMap::new(),
            warnings: Vec::new(), // v0.47: Warning collection
            binding_tracker: BindingTracker::new(), // v0.48: Unused binding detection
            imported_names: std::collections::HashSet::new(), // v0.74: Import tracking
            used_names: std::collections::HashSet::new(), // v0.74: Used name tracking
            private_functions: HashMap::new(), // v0.76: Private function tracking
            called_functions: std::collections::HashSet::new(), // v0.76: Called function tracking
            private_structs: HashMap::new(), // v0.77: Private struct tracking
            private_enums: HashMap::new(), // v0.78: Private enum tracking
            private_traits: HashMap::new(), // v0.80: Private trait tracking
            implemented_traits: std::collections::HashSet::new(), // v0.80: Implemented trait tracking
            contract_signatures: HashMap::new(), // v0.84: Contract signature tracking
            type_aliases: HashMap::new(), // v0.50.6: Type alias definitions
            function_spans: HashMap::new(), // v0.50.11: Function span tracking for duplicate detection
        }
    }

    /// v0.17: Register public items from an imported module
    /// This allows the type checker to recognize types/functions from other modules
    pub fn register_module(&mut self, module: &Module) {
        for item in &module.program.items {
            match item {
                // Register public struct definitions
                Item::StructDef(s) if s.visibility == Visibility::Public => {
                    let fields: Vec<_> = s.fields.iter()
                        .map(|f| (f.name.node.clone(), f.ty.node.clone()))
                        .collect();
                    if s.type_params.is_empty() {
                        self.structs.insert(s.name.node.clone(), fields);
                    } else {
                        self.generic_structs.insert(
                            s.name.node.clone(),
                            (s.type_params.clone(), fields)
                        );
                    }
                }
                // Register public enum definitions
                Item::EnumDef(e) if e.visibility == Visibility::Public => {
                    let variants: Vec<_> = e.variants.iter()
                        .map(|v| (v.name.node.clone(), v.fields.iter().map(|f| f.node.clone()).collect()))
                        .collect();
                    if e.type_params.is_empty() {
                        self.enums.insert(e.name.node.clone(), variants);
                    } else {
                        self.generic_enums.insert(
                            e.name.node.clone(),
                            (e.type_params.clone(), variants)
                        );
                    }
                }
                // Register public function signatures
                Item::FnDef(f) if f.visibility == Visibility::Public => {
                    if f.type_params.is_empty() {
                        let param_tys: Vec<_> = f.params.iter().map(|p| p.ty.node.clone()).collect();
                        // v0.75: Async functions return Future<T> instead of T
                        let ret_ty = if f.is_async {
                            Type::Future(Box::new(f.ret_ty.node.clone()))
                        } else {
                            f.ret_ty.node.clone()
                        };
                        self.functions.insert(f.name.node.clone(), (param_tys, ret_ty));
                    } else {
                        let type_param_names: Vec<_> = f.type_params.iter().map(|tp| tp.name.as_str()).collect();
                        let param_tys: Vec<_> = f.params.iter()
                            .map(|p| self.resolve_type_vars(&p.ty.node, &type_param_names))
                            .collect();
                        let ret_ty = self.resolve_type_vars(&f.ret_ty.node, &type_param_names);
                        // v0.75: Async functions return Future<T> instead of T
                        let ret_ty = if f.is_async {
                            Type::Future(Box::new(ret_ty))
                        } else {
                            ret_ty
                        };
                        self.generic_functions.insert(
                            f.name.node.clone(),
                            (f.type_params.clone(), param_tys, ret_ty)
                        );
                    }
                }
                // Register public extern function signatures
                Item::ExternFn(e) if e.visibility == Visibility::Public => {
                    let param_tys: Vec<_> = e.params.iter().map(|p| p.ty.node.clone()).collect();
                    self.functions.insert(e.name.node.clone(), (param_tys, e.ret_ty.node.clone()));
                }
                _ => {}
            }
        }
    }

    // ========================================================================
    // v0.47: Warning Collection Methods
    // ========================================================================

    /// Add a warning to the collection (v0.47)
    pub fn add_warning(&mut self, warning: CompileWarning) {
        self.warnings.push(warning);
    }

    /// Get collected warnings as a slice (v0.47)
    pub fn warnings(&self) -> &[CompileWarning] {
        &self.warnings
    }

    /// Take all warnings (clears the internal collection) (v0.47)
    pub fn take_warnings(&mut self) -> Vec<CompileWarning> {
        std::mem::take(&mut self.warnings)
    }

    /// Check if there are any warnings (v0.47)
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Clear all warnings (v0.47)
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();
    }

    /// Check entire program
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect type definitions (structs and enums)
        for item in &program.items {
            match item {
                Item::StructDef(s) => {
                    // v0.90.37: Check for duplicate field names
                    {
                        let mut seen_fields = std::collections::HashSet::new();
                        for f in &s.fields {
                            if !seen_fields.insert(&f.name.node) {
                                return Err(CompileError::type_error(
                                    format!("duplicate field '{}' in struct '{}'", f.name.node, s.name.node),
                                    f.name.span,
                                ));
                            }
                        }
                    }
                    let fields: Vec<_> = s.fields.iter()
                        .map(|f| (f.name.node.clone(), f.ty.node.clone()))
                        .collect();
                    // v0.15: Handle generic structs
                    if s.type_params.is_empty() {
                        self.structs.insert(s.name.node.clone(), fields);
                    } else {
                        self.generic_structs.insert(
                            s.name.node.clone(),
                            (s.type_params.clone(), fields)
                        );
                    }
                    // v0.77: Track private structs for unused type detection
                    if s.visibility != Visibility::Public && !s.name.node.starts_with('_') {
                        self.private_structs.insert(s.name.node.clone(), s.name.span);
                    }
                }
                Item::EnumDef(e) => {
                    // v0.90.37: Check for duplicate variant names
                    {
                        let mut seen_variants = std::collections::HashSet::new();
                        for v in &e.variants {
                            if !seen_variants.insert(&v.name.node) {
                                return Err(CompileError::type_error(
                                    format!("duplicate variant '{}' in enum '{}'", v.name.node, e.name.node),
                                    v.name.span,
                                ));
                            }
                        }
                    }
                    let variants: Vec<_> = e.variants.iter()
                        .map(|v| (v.name.node.clone(), v.fields.iter().map(|f| f.node.clone()).collect()))
                        .collect();
                    // v0.16: Handle generic enums separately
                    if e.type_params.is_empty() {
                        self.enums.insert(e.name.node.clone(), variants);
                    } else {
                        self.generic_enums.insert(
                            e.name.node.clone(),
                            (e.type_params.clone(), variants)
                        );
                    }
                    // v0.78: Track private enums for unused enum detection
                    if e.visibility != Visibility::Public && !e.name.node.starts_with('_') {
                        self.private_enums.insert(e.name.node.clone(), e.name.span);
                    }
                }
                Item::FnDef(_) | Item::ExternFn(_) => {}
                // v0.5 Phase 4: Use statements are processed at module resolution time
                Item::Use(_) => {}
                // v0.20.1: Register trait definitions
                Item::TraitDef(t) => {
                    let methods: Vec<TraitMethodInfo> = t.methods.iter().map(|m| {
                        // Skip the first param if it's "self: Self"
                        let param_types: Vec<Type> = m.params.iter()
                            .filter(|p| p.name.node != "self")
                            .map(|p| p.ty.node.clone())
                            .collect();
                        TraitMethodInfo {
                            name: m.name.node.clone(),
                            param_types,
                            ret_type: m.ret_ty.node.clone(),
                        }
                    }).collect();

                    self.traits.insert(t.name.node.clone(), TraitInfo {
                        name: t.name.node.clone(),
                        type_params: t.type_params.clone(),
                        methods,
                    });

                    // v0.80: Track private traits for unused trait detection
                    if t.visibility != Visibility::Public && !t.name.node.starts_with('_') {
                        self.private_traits.insert(t.name.node.clone(), t.name.span);
                    }
                }
                // v0.20.1: ImplBlocks are processed in a later pass
                Item::ImplBlock(_) => {}
                // v0.50.6: Type aliases
                Item::TypeAlias(t) => {
                    // Register type alias: name -> (type_params, target_type, refinement, span)
                    let refinement = t.refinement.as_ref().map(|r| r.node.clone());
                    self.type_aliases.insert(
                        t.name.node.clone(),
                        (t.type_params.clone(), t.target.node.clone(), refinement, t.name.span)
                    );
                }
            }
        }

        // v0.50.11: Validate type aliases for cycles
        self.validate_type_alias_cycles()?;

        // Second pass: collect function signatures (including extern fn)
        for item in &program.items {
            match item {
                Item::FnDef(f) => {
                    // v0.76: Track private functions for unused function detection
                    // Skip: main, pub functions, underscore-prefixed functions
                    if f.visibility != Visibility::Public
                        && f.name.node != "main"
                        && !f.name.node.starts_with('_')
                    {
                        self.private_functions.insert(f.name.node.clone(), f.name.span);
                    }

                    // v0.50.11: Check for duplicate function definitions
                    if let Some(original_span) = self.function_spans.get(&f.name.node) {
                        self.add_warning(CompileWarning::duplicate_function(
                            &f.name.node,
                            f.name.span,
                            *original_span,
                        ));
                    } else {
                        self.function_spans.insert(f.name.node.clone(), f.name.span);
                    }

                    // v0.15: Handle generic functions separately
                    if f.type_params.is_empty() {
                        let param_tys: Vec<_> = f.params.iter().map(|p| p.ty.node.clone()).collect();
                        // v0.75: Async functions return Future<T> instead of T
                        let ret_ty = if f.is_async {
                            Type::Future(Box::new(f.ret_ty.node.clone()))
                        } else {
                            f.ret_ty.node.clone()
                        };
                        self.functions
                            .insert(f.name.node.clone(), (param_tys, ret_ty));
                    } else {
                        // Convert Named types that match type params to TypeVar
                        let type_param_names: Vec<_> = f.type_params.iter().map(|tp| tp.name.as_str()).collect();
                        let param_tys: Vec<_> = f.params.iter()
                            .map(|p| self.resolve_type_vars(&p.ty.node, &type_param_names))
                            .collect();
                        let ret_ty = self.resolve_type_vars(&f.ret_ty.node, &type_param_names);
                        // v0.75: Async functions return Future<T> instead of T
                        let ret_ty = if f.is_async {
                            Type::Future(Box::new(ret_ty))
                        } else {
                            ret_ty
                        };
                        self.generic_functions.insert(
                            f.name.node.clone(),
                            (f.type_params.clone(), param_tys, ret_ty)
                        );
                    }
                }
                // v0.13.0: Register extern function signatures
                Item::ExternFn(e) => {
                    // v0.50.11: Check for duplicate function definitions (extern fn)
                    if let Some(original_span) = self.function_spans.get(&e.name.node) {
                        self.add_warning(CompileWarning::duplicate_function(
                            &e.name.node,
                            e.name.span,
                            *original_span,
                        ));
                    } else {
                        self.function_spans.insert(e.name.node.clone(), e.name.span);
                    }

                    let param_tys: Vec<_> = e.params.iter().map(|p| p.ty.node.clone()).collect();
                    self.functions
                        .insert(e.name.node.clone(), (param_tys, e.ret_ty.node.clone()));
                }
                Item::StructDef(_) | Item::EnumDef(_) | Item::Use(_) | Item::TypeAlias(_) => {}
                // v0.20.1: TraitDef already registered in first pass
                Item::TraitDef(_) => {}
                // v0.20.1: Register impl blocks
                Item::ImplBlock(i) => {
                    let type_name = self.type_to_string(&i.target_type.node);
                    let trait_name = i.trait_name.node.clone();

                    // Register methods from impl block
                    let mut methods = HashMap::new();
                    for method in &i.methods {
                        // Substitute Self with target type in method signature
                        let param_types: Vec<Type> = method.params.iter()
                            .filter(|p| p.name.node != "self")
                            .map(|p| self.substitute_self(&p.ty.node, &i.target_type.node))
                            .collect();
                        let ret_type = self.substitute_self(&method.ret_ty.node, &i.target_type.node);
                        methods.insert(method.name.node.clone(), (param_types, ret_type));
                    }

                    // v0.90.39: Validate impl methods match trait declaration
                    if let Some(trait_info) = self.traits.get(&trait_name) {
                        for method in &i.methods {
                            let method_name = &method.name.node;
                            if let Some(trait_method) = trait_info.methods.iter().find(|m| &m.name == method_name) {
                                // Check return type matches (substitute Self in both sides)
                                let impl_ret = self.substitute_self(&method.ret_ty.node, &i.target_type.node);
                                let trait_ret = self.substitute_self(&trait_method.ret_type, &i.target_type.node);
                                let trait_ret_str = self.type_to_string(&trait_ret);
                                let impl_ret_str = self.type_to_string(&impl_ret);
                                if trait_ret_str != impl_ret_str {
                                    return Err(CompileError::type_error(
                                        format!(
                                            "method '{}' in impl {} for {} returns '{}', but trait declares '{}'",
                                            method_name, trait_name, self.type_to_string(&i.target_type.node),
                                            impl_ret_str, trait_ret_str,
                                        ),
                                        method.ret_ty.span,
                                    ));
                                }
                                // Check parameter count matches (excluding self)
                                let impl_params: Vec<_> = method.params.iter()
                                    .filter(|p| p.name.node != "self")
                                    .collect();
                                if impl_params.len() != trait_method.param_types.len() {
                                    return Err(CompileError::type_error(
                                        format!(
                                            "method '{}' in impl {} for {} has {} parameters, but trait declares {}",
                                            method_name, trait_name, self.type_to_string(&i.target_type.node),
                                            impl_params.len(), trait_method.param_types.len(),
                                        ),
                                        method.name.span,
                                    ));
                                }
                                // Check parameter types match (substitute Self in trait param types)
                                for (idx, (impl_param, trait_param_ty)) in impl_params.iter().zip(trait_method.param_types.iter()).enumerate() {
                                    let impl_ty = self.substitute_self(&impl_param.ty.node, &i.target_type.node);
                                    let trait_ty = self.substitute_self(trait_param_ty, &i.target_type.node);
                                    let impl_ty_str = self.type_to_string(&impl_ty);
                                    let trait_ty_str = self.type_to_string(&trait_ty);
                                    if impl_ty_str != trait_ty_str {
                                        return Err(CompileError::type_error(
                                            format!(
                                                "method '{}' parameter {} type is '{}', but trait declares '{}'",
                                                method_name, idx + 1, impl_ty_str, trait_ty_str,
                                            ),
                                            impl_param.ty.span,
                                        ));
                                    }
                                }
                            }
                        }
                        // v0.90.41: Check all trait methods are implemented
                        let impl_method_names: std::collections::HashSet<_> = i.methods.iter()
                            .map(|m| m.name.node.as_str())
                            .collect();
                        for trait_method in &trait_info.methods {
                            if !impl_method_names.contains(trait_method.name.as_str()) {
                                return Err(CompileError::type_error(
                                    format!(
                                        "impl {} for {} is missing method '{}'",
                                        trait_name, self.type_to_string(&i.target_type.node),
                                        trait_method.name,
                                    ),
                                    i.span,
                                ));
                            }
                        }
                    }

                    // v0.80: Track that this trait is implemented
                    self.implemented_traits.insert(trait_name.clone());

                    self.impls.insert((type_name, trait_name.clone()), ImplInfo {
                        trait_name,
                        target_type: i.target_type.node.clone(),
                        methods,
                    });
                }
            }
        }

        // Third pass: type check function bodies (extern fn has no body)
        for item in &program.items {
            match item {
                Item::FnDef(f) => self.check_fn(f)?,
                Item::StructDef(_) | Item::EnumDef(_) | Item::Use(_) | Item::ExternFn(_) => {}
                // v0.20.1: Traits and impls already registered
                Item::TraitDef(_) | Item::ImplBlock(_) => {}
                // v0.50.6: Type aliases already processed
                Item::TypeAlias(_) => {}
            }
        }

        // v0.31: Validate module header exports (RFC-0002)
        if let Some(header) = &program.header {
            self.validate_module_exports(header, program)?;
        }

        // v0.76: Generate unused function warnings
        // P0 Correctness: Detect private functions that are never called
        for (name, span) in &self.private_functions {
            if !self.called_functions.contains(name) {
                self.warnings.push(CompileWarning::unused_function(name, *span));
            }
        }

        // v0.77: Generate unused type warnings
        // P0 Correctness: Detect private structs that are never used
        for (name, span) in &self.private_structs {
            if !self.used_names.contains(name) {
                self.warnings.push(CompileWarning::unused_type(name, *span));
            }
        }

        // v0.78: Generate unused enum warnings
        // P0 Correctness: Detect private enums that are never used
        for (name, span) in &self.private_enums {
            if !self.used_names.contains(name) {
                self.warnings.push(CompileWarning::unused_enum(name, *span));
            }
        }

        // v0.80: Generate unused trait warnings
        // P0 Correctness: Detect private traits that are never implemented
        for (name, span) in &self.private_traits {
            if !self.implemented_traits.contains(name) {
                self.warnings.push(CompileWarning::unused_trait(name, *span));
            }
        }

        Ok(())
    }

    /// v0.74: Type check with import usage tracking
    /// P0 Correctness: Detects unused imports at compile-time
    pub fn check_program_with_imports(&mut self, program: &Program, imports: &mut ResolvedImports) -> Result<()> {
        // Record which names are imported
        for (name, _info) in imports.all_imports() {
            self.imported_names.insert(name.clone());
        }

        // Run normal type checking (this populates used_names)
        self.check_program(program)?;

        // Mark imports as used if they appear in used_names
        // Collect names first to avoid borrow conflict
        let names_to_mark: Vec<String> = imports
            .all_imports()
            .filter(|(name, _)| self.used_names.contains(*name))
            .map(|(name, _)| name.clone())
            .collect();

        for name in names_to_mark {
            imports.mark_used(&name);
        }

        Ok(())
    }

    /// v0.74: Mark a name as used (for import and local type tracking)
    /// v0.77: Also tracks local struct/enum usage for unused type detection
    fn mark_name_used(&mut self, name: &str) {
        self.used_names.insert(name.to_string());
    }

    /// v0.75: Mark all type names in a type as used (for import tracking)
    /// Recursively walks the type to find Named and Generic types
    fn mark_type_names_used(&mut self, ty: &Type) {
        match ty {
            Type::Named(name) => {
                self.mark_name_used(name);
            }
            Type::Generic { name, type_args } => {
                self.mark_name_used(name);
                for arg in type_args {
                    self.mark_type_names_used(arg);
                }
            }
            Type::Array(inner, _) => {
                self.mark_type_names_used(inner);
            }
            Type::Ref(inner) | Type::RefMut(inner) => {
                self.mark_type_names_used(inner);
            }
            Type::Fn { params, ret } => {
                for param in params {
                    self.mark_type_names_used(param);
                }
                self.mark_type_names_used(ret);
            }
            Type::Tuple(elements) => {
                for elem in elements {
                    self.mark_type_names_used(elem);
                }
            }
            // v0.51.37: Added Ptr to list of wrapper types
            // v0.70: Added Thread to list of wrapper types
            // v0.71: Added Mutex to list of wrapper types
            // v0.72: Added Arc and Atomic to list of wrapper types
            // v0.73: Added Sender and Receiver to list of wrapper types
            // v0.74: Added RwLock to list of wrapper types (Barrier/Condvar are unit types)
            // v0.75: Added Future to list of wrapper types
            Type::Range(inner) | Type::Nullable(inner) | Type::Ptr(inner) | Type::Thread(inner) | Type::Mutex(inner) | Type::Arc(inner) | Type::Atomic(inner) | Type::Sender(inner) | Type::Receiver(inner) | Type::RwLock(inner) | Type::Future(inner) => {
                self.mark_type_names_used(inner);
            }
            // v0.74: Barrier and Condvar are unit types - no inner type to mark
            // v0.83: AsyncFile and AsyncSocket are unit types - no inner type to mark
            // v0.84: ThreadPool is a unit type - no inner type to mark
            // v0.85: Scope is a unit type - no inner type to mark
            Type::Barrier | Type::Condvar | Type::AsyncFile | Type::AsyncSocket | Type::ThreadPool | Type::Scope => {}
            Type::Refined { base, .. } => {
                self.mark_type_names_used(base);
            }
            Type::Struct { fields, .. } => {
                for (_, field_ty) in fields {
                    self.mark_type_names_used(field_ty);
                }
            }
            Type::Enum { variants, .. } => {
                for (_, field_tys) in variants {
                    for field_ty in field_tys {
                        self.mark_type_names_used(field_ty);
                    }
                }
            }
            // Primitive types don't have names to track
            Type::I64 | Type::I32 | Type::U32 | Type::U64 | Type::F64
            | Type::Bool | Type::String | Type::Char | Type::Unit
            | Type::Never | Type::TypeVar(_) => {}
        }
    }

    /// v0.50.11: Validate type aliases for cycles
    /// Detects circular type alias definitions like `type A = B; type B = A;`
    fn validate_type_alias_cycles(&self) -> Result<()> {
        use std::collections::HashSet;

        /// Helper: Extract type names referenced by a type
        fn collect_type_refs(ty: &Type, refs: &mut HashSet<String>) {
            match ty {
                Type::Named(name) => { refs.insert(name.clone()); }
                Type::Generic { name, type_args } => {
                    refs.insert(name.clone());
                    for arg in type_args {
                        collect_type_refs(arg, refs);
                    }
                }
                // v0.51.37: Added Ptr to wrapper types
                Type::Array(inner, _) | Type::Ref(inner) | Type::RefMut(inner)
                | Type::Nullable(inner) | Type::Range(inner) | Type::Ptr(inner) => {
                    collect_type_refs(inner, refs);
                }
                Type::Fn { params, ret } => {
                    for p in params {
                        collect_type_refs(p, refs);
                    }
                    collect_type_refs(ret, refs);
                }
                Type::Tuple(elements) => {
                    for e in elements {
                        collect_type_refs(e, refs);
                    }
                }
                _ => {}
            }
        }

        /// DFS cycle detection
        fn detect_cycle(
            name: &str,
            aliases: &HashMap<String, (Vec<TypeParam>, Type, Option<Expr>, Span)>,
            visiting: &mut HashSet<String>,
            visited: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Option<(Vec<String>, Span)> {
            if visited.contains(name) {
                return None; // Already validated, no cycle
            }
            if visiting.contains(name) {
                // Found a cycle! Return the path from first occurrence
                if let Some(pos) = path.iter().position(|n| n == name) {
                    let cycle_path: Vec<String> = path[pos..].to_vec();
                    // Get span of the first alias in the cycle
                    if let Some((_, _, _, span)) = aliases.get(name) {
                        return Some((cycle_path, *span));
                    }
                }
                return None;
            }

            // Only check if this is actually a type alias
            if let Some((_, target, _, _)) = aliases.get(name) {
                visiting.insert(name.to_string());
                path.push(name.to_string());

                // Find all type references in the target
                let mut refs = HashSet::new();
                collect_type_refs(target, &mut refs);

                // Check each referenced type for cycles
                for ref_name in refs {
                    if let Some(result) = detect_cycle(&ref_name, aliases, visiting, visited, path) {
                        return Some(result);
                    }
                }

                path.pop();
                visiting.remove(name);
                visited.insert(name.to_string());
            }

            None
        }

        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        // Check each type alias for cycles
        for name in self.type_aliases.keys() {
            if let Some((cycle, span)) = detect_cycle(name, &self.type_aliases, &mut visiting, &mut visited, &mut path) {
                let cycle_str = cycle.join(" -> ") + " -> " + &cycle[0];
                return Err(CompileError::type_error(
                    format!("cyclic type alias detected: {cycle_str}"),
                    span
                ));
            }
        }

        Ok(())
    }

    /// v0.50.6: Resolve type alias
    /// If the type is a named type that's a type alias, expand it to the target type.
    /// Non-generic type aliases are expanded recursively.
    fn resolve_type_alias(&self, ty: &Type) -> Type {
        match ty {
            Type::Named(name) => {
                // Check if this is a type alias
                if let Some((type_params, target, _refinement, _span)) = self.type_aliases.get(name) {
                    if type_params.is_empty() {
                        // Non-generic type alias: recursively resolve
                        self.resolve_type_alias(target)
                    } else {
                        // Generic type alias needs type arguments - keep as is
                        ty.clone()
                    }
                } else {
                    // Not a type alias, keep as is
                    ty.clone()
                }
            }
            Type::Generic { name, type_args } => {
                // Check if this is a generic type alias
                if let Some((type_params, target, _refinement, _span)) = self.type_aliases.get(name) {
                    if type_params.len() == type_args.len() {
                        // Substitute type arguments
                        let mut substituted = target.clone();
                        for (param, arg) in type_params.iter().zip(type_args.iter()) {
                            substituted = self.substitute_type_param(&substituted, &param.name, arg);
                        }
                        self.resolve_type_alias(&substituted)
                    } else {
                        // Wrong number of type arguments, keep as is
                        ty.clone()
                    }
                } else {
                    // Not a type alias, but resolve type args
                    Type::Generic {
                        name: name.clone(),
                        type_args: type_args.iter().map(|a| Box::new(self.resolve_type_alias(a))).collect(),
                    }
                }
            }
            // Recursively resolve nested types
            Type::Array(inner, size) => Type::Array(Box::new(self.resolve_type_alias(inner)), *size),
            Type::Ref(inner) => Type::Ref(Box::new(self.resolve_type_alias(inner))),
            Type::RefMut(inner) => Type::RefMut(Box::new(self.resolve_type_alias(inner))),
            Type::Nullable(inner) => Type::Nullable(Box::new(self.resolve_type_alias(inner))),
            Type::Range(inner) => Type::Range(Box::new(self.resolve_type_alias(inner))),
            Type::Fn { params, ret } => Type::Fn {
                params: params.iter().map(|p| Box::new(self.resolve_type_alias(p))).collect(),
                ret: Box::new(self.resolve_type_alias(ret)),
            },
            Type::Tuple(elements) => Type::Tuple(elements.iter().map(|e| Box::new(self.resolve_type_alias(e))).collect()),
            // Primitive and other types don't need resolution
            _ => ty.clone(),
        }
    }

    /// v0.50.6: Substitute a type parameter with a concrete type
    fn substitute_type_param(&self, ty: &Type, param_name: &str, replacement: &Type) -> Type {
        match ty {
            Type::Named(name) if name == param_name => replacement.clone(),
            Type::TypeVar(name) if name == param_name => replacement.clone(),
            Type::Generic { name, type_args } => {
                if name == param_name {
                    replacement.clone()
                } else {
                    Type::Generic {
                        name: name.clone(),
                        type_args: type_args.iter()
                            .map(|a| Box::new(self.substitute_type_param(a, param_name, replacement)))
                            .collect(),
                    }
                }
            }
            Type::Array(inner, size) => Type::Array(
                Box::new(self.substitute_type_param(inner, param_name, replacement)),
                *size
            ),
            Type::Ref(inner) => Type::Ref(Box::new(self.substitute_type_param(inner, param_name, replacement))),
            Type::RefMut(inner) => Type::RefMut(Box::new(self.substitute_type_param(inner, param_name, replacement))),
            Type::Nullable(inner) => Type::Nullable(Box::new(self.substitute_type_param(inner, param_name, replacement))),
            Type::Range(inner) => Type::Range(Box::new(self.substitute_type_param(inner, param_name, replacement))),
            Type::Fn { params, ret } => Type::Fn {
                params: params.iter()
                    .map(|p| Box::new(self.substitute_type_param(p, param_name, replacement)))
                    .collect(),
                ret: Box::new(self.substitute_type_param(ret, param_name, replacement)),
            },
            Type::Tuple(elements) => Type::Tuple(
                elements.iter()
                    .map(|e| Box::new(self.substitute_type_param(e, param_name, replacement)))
                    .collect()
            ),
            _ => ty.clone(),
        }
    }

    /// Check function definition
    fn check_fn(&mut self, f: &FnDef) -> Result<()> {
        // Clear environment and add parameters
        self.env.clear();
        self.type_param_env.clear();

        // v0.49: Reset binding tracker and push function scope
        self.binding_tracker = BindingTracker::new();
        self.binding_tracker.push_scope();

        // v0.15: Register type parameters for generic functions
        let type_param_names: Vec<_> = f.type_params.iter().map(|tp| tp.name.as_str()).collect();
        for tp in &f.type_params {
            self.type_param_env.insert(tp.name.clone(), tp.bounds.clone());
        }

        // v0.75: Mark type names in parameter and return types as used
        for param in &f.params {
            self.mark_type_names_used(&param.ty.node);
        }
        self.mark_type_names_used(&f.ret_ty.node);

        // v0.15: Convert Named types that match type params to TypeVar for env
        for param in &f.params {
            let resolved_ty = if f.type_params.is_empty() {
                param.ty.node.clone()
            } else {
                self.resolve_type_vars(&param.ty.node, &type_param_names)
            };
            self.env.insert(param.name.node.clone(), resolved_ty);
            // v0.49: Track parameter binding for unused detection
            self.binding_tracker.bind(param.name.node.clone(), param.name.span);
        }

        // Set current return type for `ret` keyword
        // v0.15: Resolve type vars in return type too
        let resolved_ret_ty = if f.type_params.is_empty() {
            f.ret_ty.node.clone()
        } else {
            self.resolve_type_vars(&f.ret_ty.node, &type_param_names)
        };
        self.current_ret_ty = Some(resolved_ret_ty.clone());

        // Check pre condition (must be bool)
        if let Some(pre) = &f.pre {
            let pre_ty = self.infer(&pre.node, pre.span)?;
            self.unify(&Type::Bool, &pre_ty, pre.span)?;
        }

        // Check post condition (must be bool)
        if let Some(post) = &f.post {
            let post_ty = self.infer(&post.node, post.span)?;
            self.unify(&Type::Bool, &post_ty, post.span)?;
        }

        // Check body
        let body_ty = self.infer(&f.body.node, f.body.span)?;
        // v0.15: Use resolved return type for generic functions
        self.unify(&resolved_ret_ty, &body_ty, f.body.span)?;

        // v0.81: Check for missing postcondition
        // Skip: main, underscore-prefixed, @trust functions, unit return type
        let has_postcondition = f.post.is_some();
        let is_main = f.name.node == "main";
        let is_underscore = f.name.node.starts_with('_');
        let is_trusted = f.attributes.iter().any(|a| a.is_trust());
        let is_unit_return = matches!(f.ret_ty.node, Type::Unit);

        if !has_postcondition && !is_main && !is_underscore && !is_trusted && !is_unit_return {
            self.add_warning(CompileWarning::missing_postcondition(
                &f.name.node,
                f.name.span,
            ));
        }

        // v0.84: Check for semantic duplication (equivalent contracts)
        // Only for functions that have postconditions
        if let Some(post) = &f.post {
            // Create signature key: (param_types, return_type) - span-agnostic
            let sig_key = format!(
                "({}) -> {}",
                f.params.iter().map(|p| output::format_type(&p.ty.node)).collect::<Vec<_>>().join(", "),
                output::format_type(&f.ret_ty.node)
            );
            // Create postcondition key: span-agnostic S-expression
            let post_key = output::format_expr(&post.node);

            let key = (sig_key, post_key);

            if let Some((existing_name, _)) = self.contract_signatures.get(&key) {
                // Found a function with equivalent contract
                self.add_warning(CompileWarning::semantic_duplication(
                    &f.name.node,
                    existing_name,
                    f.name.span,
                ));
            } else {
                // First function with this signature+postcondition
                self.contract_signatures.insert(key, (f.name.node.clone(), f.name.span));
            }
        }

        // v0.49: Check for unused parameters and emit warnings
        // Note: Function parameters are immutable, so no unused_mut check needed
        let (unused, _unused_mut) = self.binding_tracker.pop_scope();
        for (unused_name, unused_span) in unused {
            self.add_warning(CompileWarning::unused_binding(unused_name, unused_span));
        }

        self.current_ret_ty = None;
        self.type_param_env.clear();
        Ok(())
    }

    /// v0.31: Validate module header exports (RFC-0002)
    /// Ensures all exported symbols are actually defined in the module
    fn validate_module_exports(&self, header: &ModuleHeader, program: &Program) -> Result<()> {
        // Collect all defined symbols (public visibility only for exports)
        let mut defined_symbols: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for item in &program.items {
            match item {
                Item::FnDef(f) => {
                    defined_symbols.insert(&f.name.node);
                }
                Item::StructDef(s) => {
                    defined_symbols.insert(&s.name.node);
                }
                Item::EnumDef(e) => {
                    defined_symbols.insert(&e.name.node);
                }
                Item::TraitDef(t) => {
                    defined_symbols.insert(&t.name.node);
                }
                Item::ExternFn(e) => {
                    defined_symbols.insert(&e.name.node);
                }
                Item::Use(_) | Item::ImplBlock(_) => {}
                // v0.50.6: Type aliases can be exported
                Item::TypeAlias(t) => {
                    defined_symbols.insert(&t.name.node);
                }
            }
        }

        // Check each export matches a defined symbol
        for export in &header.exports {
            if !defined_symbols.contains(export.node.as_str()) {
                return Err(CompileError::type_error(
                    format!(
                        "Module '{}' exports '{}' but no such definition exists",
                        header.name.node, export.node
                    ),
                    export.span,
                ));
            }
        }

        Ok(())
    }

    /// Infer expression type
    fn infer(&mut self, expr: &Expr, span: Span) -> Result<Type> {
        match expr {
            Expr::IntLit(_) => Ok(Type::I64),
            Expr::FloatLit(_) => Ok(Type::F64),
            Expr::BoolLit(_) => Ok(Type::Bool),
            Expr::StringLit(_) => Ok(Type::String),
            // v0.64: Character literal type inference
            Expr::CharLit(_) => Ok(Type::Char),
            // v0.51.40: Null pointer literal - creates a polymorphic pointer type
            // that will unify with any expected pointer type via TypeVar matching
            Expr::Null => Ok(Type::Ptr(Box::new(Type::TypeVar("_null".to_string())))),
            // v0.51.41: Sizeof returns i64 (size in bytes)
            Expr::Sizeof { .. } => Ok(Type::I64),

            // v0.70: Spawn expression - creates a thread that produces T
            Expr::Spawn { body } => {
                let body_type = self.infer(&body.node, body.span)?;
                Ok(Type::Thread(Box::new(body_type)))
            }

            // v0.72: Atomic creation - returns Atomic<T> where T is inferred from value
            Expr::AtomicNew { value } => {
                let value_ty = self.infer(&value.node, value.span)?;
                Ok(Type::Atomic(Box::new(value_ty)))
            }

            // v0.71: Mutex creation - returns Mutex<T> where T is inferred from value
            Expr::MutexNew { value } => {
                let value_ty = self.infer(&value.node, value.span)?;
                Ok(Type::Mutex(Box::new(value_ty)))
            }

            // v0.73: Channel creation - returns (Sender<T>, Receiver<T>)
            Expr::ChannelNew { elem_ty, capacity } => {
                let cap_ty = self.infer(&capacity.node, capacity.span)?;
                self.unify(&cap_ty, &Type::I64, capacity.span)?;
                Ok(Type::Tuple(vec![
                    Box::new(Type::Sender(Box::new(elem_ty.node.clone()))),
                    Box::new(Type::Receiver(Box::new(elem_ty.node.clone()))),
                ]))
            }

            // v0.74: RwLock creation - returns RwLock<T> where T is inferred from value
            Expr::RwLockNew { value } => {
                let value_ty = self.infer(&value.node, value.span)?;
                Ok(Type::RwLock(Box::new(value_ty)))
            }

            // v0.74: Barrier creation - returns Barrier
            Expr::BarrierNew { count } => {
                let count_ty = self.infer(&count.node, count.span)?;
                self.unify(&count_ty, &Type::I64, count.span)?;
                Ok(Type::Barrier)
            }

            // v0.74: Condvar creation - returns Condvar
            Expr::CondvarNew => Ok(Type::Condvar),

            // v0.75: Await expression - unwraps Future<T> to T
            Expr::Await { future } => {
                let future_ty = self.infer(&future.node, future.span)?;
                match future_ty {
                    Type::Future(inner) => Ok(*inner),
                    other => Err(CompileError::type_error(
                        format!("cannot await non-future type '{}'", other),
                        span,
                    )),
                }
            }

            // v0.82: Select expression - all arms must have same result type
            Expr::Select { arms } => {
                if arms.is_empty() {
                    return Err(CompileError::type_error(
                        "select expression must have at least one arm",
                        span,
                    ));
                }

                let mut result_ty: Option<Type> = None;

                for arm in arms {
                    // Type-check the operation (e.g., rx.recv())
                    let op_ty = self.infer(&arm.operation.node, arm.operation.span)?;

                    // If there's a binding, add it to the environment for this arm
                    if let Some(binding_name) = &arm.binding {
                        self.env.insert(binding_name.clone(), op_ty.clone());
                    }

                    // Type-check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_ty = self.infer(&guard.node, guard.span)?;
                        if guard_ty != Type::Bool {
                            // Clean up binding before returning error
                            if let Some(binding_name) = &arm.binding {
                                self.env.remove(binding_name);
                            }
                            return Err(CompileError::type_error(
                                format!("select guard must be bool, found '{}'", guard_ty),
                                guard.span,
                            ));
                        }
                    }

                    // Type-check the arm body with the binding in scope
                    let body_ty = self.infer(&arm.body.node, arm.body.span)?;

                    // Clean up binding after type-checking this arm
                    if let Some(binding_name) = &arm.binding {
                        self.env.remove(binding_name);
                    }

                    // All arms must have the same result type
                    match &result_ty {
                        None => result_ty = Some(body_ty),
                        Some(existing) => {
                            if &body_ty != existing {
                                return Err(CompileError::type_error(
                                    format!(
                                        "select arms have incompatible types: expected '{}', found '{}'",
                                        existing, body_ty
                                    ),
                                    arm.body.span,
                                ));
                            }
                        }
                    }
                }

                Ok(result_ty.unwrap_or(Type::Unit))
            }

            Expr::Unit => Ok(Type::Unit),

            Expr::Ret => self.current_ret_ty.clone().ok_or_else(|| {
                CompileError::type_error("'ret' used outside function", span)
            }),

            Expr::Var(name) => {
                // v0.48: Mark variable as used for unused binding detection
                self.binding_tracker.mark_used(name);
                self.env.get(name).cloned().ok_or_else(|| {
                    // v0.62: Suggest similar variable names
                    let var_names: Vec<&str> = self.env.keys().map(|s| s.as_str()).collect();
                    let suggestion = find_similar_name(name, &var_names, 2);
                    CompileError::type_error(
                        format!("undefined variable: `{}`{}", name, format_suggestion_hint(suggestion)),
                        span,
                    )
                })
            }

            Expr::Binary { left, op, right } => {
                let left_ty = self.infer(&left.node, left.span)?;
                let right_ty = self.infer(&right.node, right.span)?;
                self.check_binary_op(*op, &left_ty, &right_ty, span)
            }

            Expr::Unary { op, expr } => {
                let ty = self.infer(&expr.node, expr.span)?;
                self.check_unary_op(*op, &ty, span)
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_ty = self.infer(&cond.node, cond.span)?;
                self.unify(&Type::Bool, &cond_ty, cond.span)?;

                let then_ty = self.infer(&then_branch.node, then_branch.span)?;
                let else_ty = self.infer(&else_branch.node, else_branch.span)?;

                // v0.89.17: Nullable-aware branch unification
                // When one branch is T and the other is null, result is T?
                let is_then_null = matches!(&then_ty, Type::Ptr(inner) if matches!(inner.as_ref(), Type::TypeVar(n) if n == "_null"));
                let is_else_null = matches!(&else_ty, Type::Ptr(inner) if matches!(inner.as_ref(), Type::TypeVar(n) if n == "_null"));

                if is_else_null && !is_then_null {
                    // then: T, else: null → T?
                    Ok(Type::Nullable(Box::new(then_ty)))
                } else if is_then_null && !is_else_null {
                    // then: null, else: T → T?
                    Ok(Type::Nullable(Box::new(else_ty)))
                } else {
                    self.unify(&then_ty, &else_ty, else_branch.span)?;
                    Ok(then_ty)
                }
            }

            Expr::Let {
                name,
                mutable,
                ty,
                value,
                body,
            } => {
                let value_ty = self.infer(&value.node, value.span)?;

                // v0.51.48: When type annotation is present, use it as the variable's type
                // This fixes i32 variables being incorrectly stored as i64 when initialized with literals
                let stored_ty = if let Some(ann_ty) = ty {
                    // v0.75: Mark type names in annotation as used
                    self.mark_type_names_used(&ann_ty.node);
                    self.unify(&ann_ty.node, &value_ty, value.span)?;
                    ann_ty.node.clone()
                } else {
                    value_ty
                };

                // v0.48: Track binding for unused detection
                // v0.52: Track mutability for unused-mut detection
                self.binding_tracker.push_scope();

                // v0.79: Check for shadow binding before adding
                if let Some(original_span) = self.binding_tracker.find_shadow(name) {
                    self.add_warning(CompileWarning::shadow_binding(name, span, original_span));
                }

                self.binding_tracker.bind_with_mutability(name.clone(), span, *mutable);

                self.env.insert(name.clone(), stored_ty);
                let result = self.infer(&body.node, body.span)?;

                // v0.48: Check for unused bindings and emit warnings
                // v0.52: Also check for mutable-but-never-mutated
                let (unused, unused_mut) = self.binding_tracker.pop_scope();
                for (unused_name, unused_span) in unused {
                    self.add_warning(CompileWarning::unused_binding(unused_name, unused_span));
                }
                for (name, span) in unused_mut {
                    self.add_warning(CompileWarning::unused_mut(name, span));
                }

                Ok(result)
            }

            // v0.60.21: Uninitialized let binding for stack arrays
            // Only allowed for array types - uninitialized primitives are dangerous
            Expr::LetUninit { name, mutable, ty, body } => {
                // v0.75: Mark type names in annotation as used
                self.mark_type_names_used(&ty.node);

                // Only allow for array types (safety check)
                match &ty.node {
                    Type::Array(_, _) => {
                        // Track binding
                        self.binding_tracker.push_scope();

                        // v0.79: Check for shadow binding
                        if let Some(original_span) = self.binding_tracker.find_shadow(name) {
                            self.add_warning(CompileWarning::shadow_binding(name, span, original_span));
                        }

                        self.binding_tracker.bind_with_mutability(name.clone(), span, *mutable);
                        self.env.insert(name.clone(), ty.node.clone());

                        let result = self.infer(&body.node, body.span)?;

                        // Check for unused bindings
                        let (unused, unused_mut) = self.binding_tracker.pop_scope();
                        for (unused_name, unused_span) in unused {
                            self.add_warning(CompileWarning::unused_binding(unused_name, unused_span));
                        }
                        for (name, span) in unused_mut {
                            self.add_warning(CompileWarning::unused_mut(name, span));
                        }

                        Ok(result)
                    }
                    _ => Err(CompileError::type_error(
                        "uninitialized declaration only allowed for array types".to_string(),
                        ty.span,
                    )),
                }
            }

            Expr::Assign { name, value } => {
                // Check that variable exists
                let var_ty = self.env.get(name).cloned().ok_or_else(|| {
                    // v0.62: Suggest similar variable names
                    let var_names: Vec<&str> = self.env.keys().map(|s| s.as_str()).collect();
                    let suggestion = find_similar_name(name, &var_names, 2);
                    CompileError::type_error(
                        format!("undefined variable: `{}`{}", name, format_suggestion_hint(suggestion)),
                        span,
                    )
                })?;

                // Check that value type matches variable type
                let value_ty = self.infer(&value.node, value.span)?;
                self.unify(&var_ty, &value_ty, value.span)?;

                // v0.52: Mark variable as mutated for unused-mut detection
                self.binding_tracker.mark_mutated(name);

                // Assignment returns unit
                Ok(Type::Unit)
            }

            // v0.37: Include invariant type checking
            Expr::While { cond, invariant, body } => {
                // Condition must be bool
                let cond_ty = self.infer(&cond.node, cond.span)?;
                self.unify(&Type::Bool, &cond_ty, cond.span)?;

                // v0.37: Invariant must be bool if present
                if let Some(inv) = invariant {
                    let inv_ty = self.infer(&inv.node, inv.span)?;
                    self.unify(&Type::Bool, &inv_ty, inv.span)?;
                }

                // Type check body (result is discarded)
                let _ = self.infer(&body.node, body.span)?;

                // While returns unit
                Ok(Type::Unit)
            }

            // v0.2: Range expression with kind
            Expr::Range { start, end, .. } => {
                let start_ty = self.infer(&start.node, start.span)?;
                let end_ty = self.infer(&end.node, end.span)?;

                // Both must be the same integer type
                self.unify(&start_ty, &end_ty, end.span)?;
                match &start_ty {
                    // v0.38: Include unsigned types
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 => Ok(Type::Range(Box::new(start_ty))),
                    _ => Err(CompileError::type_error(
                        format!("range requires integer types, got {start_ty}"),
                        span,
                    )),
                }
            }

            // v0.5 Phase 3: For loop
            Expr::For { var, iter, body } => {
                let iter_ty = self.infer(&iter.node, iter.span)?;

                // Iterator must be a Range, Array, or Receiver<T> type
                let elem_ty = match &iter_ty {
                    Type::Range(elem) => (**elem).clone(),
                    // v0.90.33: Array iteration
                    Type::Array(elem, _) => (**elem).clone(),
                    // v0.81: Receiver<T> can be iterated (calls recv_opt until closed)
                    Type::Receiver(inner) => (**inner).clone(),
                    Type::Generic { name, type_args } if name == "Receiver" => {
                        type_args.first().map(|t| (**t).clone()).unwrap_or(Type::I64)
                    }
                    _ => {
                        return Err(CompileError::type_error(
                            format!("for loop requires Range, Array, or Receiver type, got {iter_ty}"),
                            iter.span,
                        ));
                    }
                };

                // Bind loop variable
                self.env.insert(var.clone(), elem_ty);

                // Type check body (result is discarded)
                let _ = self.infer(&body.node, body.span)?;

                // For returns unit
                Ok(Type::Unit)
            }

            Expr::Call { func, args } => {
                // v0.50: Mark function variable as used for binding detection
                self.binding_tracker.mark_used(func);
                // v0.74: Mark imported function as used
                self.mark_name_used(func);
                // v0.76: Track function calls for unused function detection
                self.called_functions.insert(func.clone());

                // v0.20.0: First try closure/function variable
                if let Some(var_ty) = self.env.get(func).cloned()
                    && let Type::Fn { params: param_tys, ret: ret_ty } = var_ty
                {
                    if args.len() != param_tys.len() {
                        return Err(CompileError::type_error(
                            format!(
                                "closure expects {} arguments, got {}",
                                param_tys.len(),
                                args.len()
                            ),
                            span,
                        ));
                    }

                    for (arg, param_ty) in args.iter().zip(param_tys.iter()) {
                        let arg_ty = self.infer(&arg.node, arg.span)?;
                        self.unify(param_ty.as_ref(), &arg_ty, arg.span)?;
                    }

                    return Ok(*ret_ty);
                }

                // v0.60.27: Try generic functions FIRST for pointer-accepting overloads
                // This allows `free(ptr)` to match `free<T>(*T)` before `free(i64)` fails
                if let Some((type_params, param_tys, ret_ty)) = self.generic_functions.get(func).cloned()
                    && args.len() == param_tys.len() {
                        // Try to infer type arguments - if this fails, fall through to non-generic
                        let mut type_subst: HashMap<String, Type> = HashMap::new();
                        let mut generic_match = true;

                        for (arg, param_ty) in args.iter().zip(param_tys.iter()) {
                            let arg_ty = self.infer(&arg.node, arg.span)?;
                            if self.infer_type_args(param_ty, &arg_ty, &mut type_subst, arg.span).is_err() {
                                generic_match = false;
                                break;
                            }
                        }

                        // Check that all type parameters were inferred
                        if generic_match {
                            let uninferred: Vec<_> = type_params
                                .iter()
                                .filter(|tp| !type_subst.contains_key(&tp.name))
                                .collect();

                            if uninferred.is_empty() {
                                // All type params inferred - use generic version
                                let instantiated_ret_ty = self.substitute_type(&ret_ty, &type_subst);
                                return Ok(instantiated_ret_ty);
                            }
                        }
                        // Fall through to try non-generic version
                    }

                // v0.15: Try non-generic functions
                if let Some((param_tys, ret_ty)) = self.functions.get(func).cloned() {
                    if args.len() != param_tys.len() {
                        return Err(CompileError::type_error(
                            format!(
                                "expected {} arguments, got {}",
                                param_tys.len(),
                                args.len()
                            ),
                            span,
                        ));
                    }

                    for (arg, param_ty) in args.iter().zip(param_tys.iter()) {
                        let arg_ty = self.infer(&arg.node, arg.span)?;
                        self.unify(param_ty, &arg_ty, arg.span)?;
                    }

                    return Ok(ret_ty);
                }

                // v0.15: Try generic functions (original position - for functions only in generic_functions)
                if let Some((type_params, param_tys, ret_ty)) = self.generic_functions.get(func).cloned() {
                    if args.len() != param_tys.len() {
                        return Err(CompileError::type_error(
                            format!(
                                "expected {} arguments, got {}",
                                param_tys.len(),
                                args.len()
                            ),
                            span,
                        ));
                    }

                    // Infer type arguments from actual arguments
                    let mut type_subst: HashMap<String, Type> = HashMap::new();

                    for (arg, param_ty) in args.iter().zip(param_tys.iter()) {
                        let arg_ty = self.infer(&arg.node, arg.span)?;
                        self.infer_type_args(param_ty, &arg_ty, &mut type_subst, arg.span)?;
                    }

                    // Check that all type parameters are inferred
                    // v0.59: Enhanced error message with inferred types and hints
                    let uninferred: Vec<_> = type_params
                        .iter()
                        .filter(|tp| !type_subst.contains_key(&tp.name))
                        .map(|tp| tp.name.clone())
                        .collect();
                    if !uninferred.is_empty() {
                        let mut msg = format!(
                            "could not infer type for type parameter{}",
                            if uninferred.len() > 1 { "s" } else { "" }
                        );
                        msg.push_str(&format!(": {}", uninferred.join(", ")));
                        // Show what was successfully inferred
                        if !type_subst.is_empty() {
                            let inferred: Vec<_> = type_subst
                                .iter()
                                .map(|(k, v)| format!("{} = {}", k, v))
                                .collect();
                            msg.push_str(&format!("\n  note: inferred {}", inferred.join(", ")));
                        }
                        msg.push_str(&format!(
                            "\n  hint: add explicit type arguments: `{}<{}>`",
                            func,
                            type_params.iter().map(|tp| tp.name.as_str()).collect::<Vec<_>>().join(", ")
                        ));
                        return Err(CompileError::type_error(msg, span));
                    }

                    // Substitute type parameters in return type
                    let instantiated_ret_ty = self.substitute_type(&ret_ty, &type_subst);
                    return Ok(instantiated_ret_ty);
                }

                // v0.61: Suggest similar function names
                let mut all_functions: Vec<&str> = self.functions.keys().map(|s| s.as_str()).collect();
                all_functions.extend(self.generic_functions.keys().map(|s| s.as_str()));
                // Also include closure/function variables from environment
                for (name, ty) in &self.env {
                    if matches!(ty, Type::Fn { .. }) {
                        all_functions.push(name.as_str());
                    }
                }
                let suggestion = find_similar_name(func, &all_functions, 2);
                Err(CompileError::type_error(
                    format!("undefined function: `{}`{}", func, format_suggestion_hint(suggestion)),
                    span,
                ))
            }

            Expr::Block(exprs) => {
                if exprs.is_empty() {
                    return Ok(Type::Unit);
                }

                let mut last_ty = Type::Unit;
                let mut diverged = false;
                let mut diverge_span: Option<Span> = None;

                for expr in exprs {
                    // v0.53: Check for unreachable code after divergent expression
                    if diverged {
                        self.add_warning(CompileWarning::unreachable_code(expr.span));
                        // Still type-check for error reporting, but don't update last_ty
                        let _ = self.infer(&expr.node, expr.span);
                        continue;
                    }

                    last_ty = self.infer(&expr.node, expr.span)?;

                    // v0.53: Track divergence (return, break, continue, Never type)
                    if matches!(last_ty, Type::Never) || self.is_divergent_expr(&expr.node) {
                        diverged = true;
                        diverge_span = Some(expr.span);
                    }
                }

                // If block diverged, the type is Never (unless we want last_ty for partial analysis)
                if diverged && diverge_span.is_some() {
                    Ok(Type::Never)
                } else {
                    Ok(last_ty)
                }
            }

            // v0.5: Struct and Enum expressions
            Expr::StructInit { name, fields } => {
                // v0.74: Mark imported struct as used
                self.mark_name_used(name);
                // v0.16: First try non-generic structs
                if let Some(struct_fields) = self.structs.get(name).cloned() {
                    // Check that all required fields are provided
                    for (field_name, field_ty) in &struct_fields {
                        let provided = fields.iter().find(|(n, _)| &n.node == field_name);
                        match provided {
                            Some((_, expr)) => {
                                let expr_ty = self.infer(&expr.node, expr.span)?;
                                self.unify(field_ty, &expr_ty, expr.span)?;
                            }
                            None => {
                                return Err(CompileError::type_error(
                                    format!("missing field: {field_name}"),
                                    span,
                                ));
                            }
                        }
                    }
                    return Ok(Type::Named(name.clone()));
                }

                // v0.16: Try generic structs with type inference
                if let Some((type_params, struct_fields)) = self.generic_structs.get(name).cloned() {
                    let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();

                    // Infer type arguments from field values
                    let mut type_subst: HashMap<String, Type> = HashMap::new();
                    for (field_name, field_ty) in &struct_fields {
                        let provided = fields.iter().find(|(n, _)| &n.node == field_name);
                        match provided {
                            Some((_, expr)) => {
                                let expr_ty = self.infer(&expr.node, expr.span)?;
                                let resolved_field_ty = self.resolve_type_vars(field_ty, &type_param_names);
                                self.infer_type_args(&resolved_field_ty, &expr_ty, &mut type_subst, expr.span)?;
                            }
                            None => {
                                return Err(CompileError::type_error(
                                    format!("missing field: {field_name}"),
                                    span,
                                ));
                            }
                        }
                    }

                    // Build instantiated type: e.g., Pair<i64, bool>
                    let type_args: Vec<Box<Type>> = type_params.iter()
                        .map(|tp| Box::new(type_subst.get(&tp.name).cloned().unwrap_or(Type::TypeVar(tp.name.clone()))))
                        .collect();

                    return Ok(Type::Generic {
                        name: name.clone(),
                        type_args,
                    });
                }

                // v0.63: Suggest similar type names (structs and enums)
                let mut all_types: Vec<&str> = self.structs.keys().map(|s| s.as_str()).collect();
                all_types.extend(self.generic_structs.keys().map(|s| s.as_str()));
                all_types.extend(self.enums.keys().map(|s| s.as_str()));
                all_types.extend(self.generic_enums.keys().map(|s| s.as_str()));
                let suggestion = find_similar_name(name, &all_types, 2);
                Err(CompileError::type_error(
                    format!("undefined struct: `{}`{}", name, format_suggestion_hint(suggestion)),
                    span,
                ))
            }

            Expr::FieldAccess { expr: obj_expr, field } => {
                let obj_ty = self.infer(&obj_expr.node, obj_expr.span)?;

                match &obj_ty {
                    Type::Named(struct_name) => {
                        let struct_fields = self.structs.get(struct_name).ok_or_else(|| {
                            CompileError::type_error(format!("not a struct: {struct_name}"), span)
                        })?;

                        for (fname, fty) in struct_fields {
                            if fname == &field.node {
                                return Ok(fty.clone());
                            }
                        }

                        // v0.60: Suggest similar field names
                        let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                        let suggestion = find_similar_name(&field.node, &field_names, 2);
                        Err(CompileError::type_error(
                            format!("unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                            span,
                        ))
                    }
                    // v0.16: Handle generic struct field access (e.g., Pair<i64, bool>.fst)
                    Type::Generic { name: struct_name, type_args } => {
                        if let Some((type_params, struct_fields)) = self.generic_structs.get(struct_name).cloned() {
                            // Build type substitution
                            let mut type_subst: HashMap<String, Type> = HashMap::new();
                            for (tp, arg) in type_params.iter().zip(type_args.iter()) {
                                type_subst.insert(tp.name.clone(), (**arg).clone());
                            }

                            let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();

                            for (fname, fty) in &struct_fields {
                                if fname == &field.node {
                                    // Substitute type parameters in field type
                                    let resolved_fty = self.resolve_type_vars(fty, &type_param_names);
                                    let substituted_fty = self.substitute_type(&resolved_fty, &type_subst);
                                    return Ok(substituted_fty);
                                }
                            }

                            // v0.60: Suggest similar field names
                            let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                            let suggestion = find_similar_name(&field.node, &field_names, 2);
                            return Err(CompileError::type_error(
                                format!("unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                                span,
                            ));
                        }
                        Err(CompileError::type_error(
                            format!("not a struct: {struct_name}"),
                            span,
                        ))
                    }
                    // v0.51.37: Auto-dereference pointer types for field access
                    // *Node.left automatically dereferences to access Node.left
                    Type::Ptr(inner) => {
                        // Recursively look up field on the pointee type
                        match inner.as_ref() {
                            Type::Named(struct_name) => {
                                let struct_fields = self.structs.get(struct_name).ok_or_else(|| {
                                    CompileError::type_error(format!("not a struct: {struct_name}"), span)
                                })?;

                                for (fname, fty) in struct_fields {
                                    if fname == &field.node {
                                        return Ok(fty.clone());
                                    }
                                }

                                let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                                let suggestion = find_similar_name(&field.node, &field_names, 2);
                                Err(CompileError::type_error(
                                    format!("unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                                    span,
                                ))
                            }
                            Type::Generic { name: struct_name, type_args } => {
                                if let Some((type_params, struct_fields)) = self.generic_structs.get(struct_name).cloned() {
                                    let mut type_subst: HashMap<String, Type> = HashMap::new();
                                    for (tp, arg) in type_params.iter().zip(type_args.iter()) {
                                        type_subst.insert(tp.name.clone(), (**arg).clone());
                                    }

                                    let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();

                                    for (fname, fty) in &struct_fields {
                                        if fname == &field.node {
                                            let resolved_fty = self.resolve_type_vars(fty, &type_param_names);
                                            let substituted_fty = self.substitute_type(&resolved_fty, &type_subst);
                                            return Ok(substituted_fty);
                                        }
                                    }

                                    let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                                    let suggestion = find_similar_name(&field.node, &field_names, 2);
                                    return Err(CompileError::type_error(
                                        format!("unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                                        span,
                                    ));
                                }
                                Err(CompileError::type_error(
                                    format!("not a struct: {struct_name}"),
                                    span,
                                ))
                            }
                            _ => Err(CompileError::type_error(
                                format!("cannot access field on pointer to non-struct type: {inner}"),
                                span,
                            )),
                        }
                    }
                    _ => Err(CompileError::type_error(
                        format!("field access on non-struct type: {obj_ty}"),
                        span,
                    )),
                }
            }

            // v0.43: Tuple field access: expr.0, expr.1, etc.
            Expr::TupleField { expr: tuple_expr, index } => {
                let tuple_ty = self.infer(&tuple_expr.node, tuple_expr.span)?;

                match &tuple_ty {
                    Type::Tuple(elem_types) => {
                        if *index >= elem_types.len() {
                            return Err(CompileError::type_error(
                                format!(
                                    "tuple index {} out of bounds for tuple with {} elements",
                                    index,
                                    elem_types.len()
                                ),
                                span,
                            ));
                        }
                        Ok((*elem_types[*index]).clone())
                    }
                    _ => Err(CompileError::type_error(
                        format!("tuple field access on non-tuple type: {tuple_ty}"),
                        span,
                    )),
                }
            }

            Expr::EnumVariant { enum_name, variant, args } => {
                // v0.74: Mark imported enum as used
                self.mark_name_used(enum_name);
                // v0.16: First try non-generic enums
                if let Some(variants) = self.enums.get(enum_name).cloned() {
                    let variant_fields = variants.iter()
                        .find(|(name, _)| name == variant)
                        .map(|(_, fields)| fields.clone())
                        .ok_or_else(|| {
                            // v0.60: Suggest similar variant names
                            let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
                            let suggestion = find_similar_name(variant, &names, 2);
                            CompileError::type_error(
                                format!("unknown variant `{}` on enum `{}`{}", variant, enum_name, format_suggestion_hint(suggestion)),
                                span,
                            )
                        })?;

                    if args.len() != variant_fields.len() {
                        return Err(CompileError::type_error(
                            format!("expected {} args, got {}", variant_fields.len(), args.len()),
                            span,
                        ));
                    }

                    for (arg, expected_ty) in args.iter().zip(variant_fields.iter()) {
                        let arg_ty = self.infer(&arg.node, arg.span)?;
                        self.unify(expected_ty, &arg_ty, arg.span)?;
                    }

                    return Ok(Type::Named(enum_name.clone()));
                }

                // v0.16: Try generic enums with type inference
                if let Some((type_params, variants)) = self.generic_enums.get(enum_name).cloned() {
                    let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();

                    let variant_fields = variants.iter()
                        .find(|(name, _)| name == variant)
                        .map(|(_, fields)| fields.clone())
                        .ok_or_else(|| {
                            // v0.60: Suggest similar variant names
                            let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
                            let suggestion = find_similar_name(variant, &names, 2);
                            CompileError::type_error(
                                format!("unknown variant `{}` on enum `{}`{}", variant, enum_name, format_suggestion_hint(suggestion)),
                                span,
                            )
                        })?;

                    if args.len() != variant_fields.len() {
                        return Err(CompileError::type_error(
                            format!("expected {} args, got {}", variant_fields.len(), args.len()),
                            span,
                        ));
                    }

                    // Infer type arguments from actual arguments
                    let mut type_subst: HashMap<String, Type> = HashMap::new();
                    for (arg, field_ty) in args.iter().zip(variant_fields.iter()) {
                        let arg_ty = self.infer(&arg.node, arg.span)?;
                        // Convert Named types to TypeVar for inference
                        let resolved_field_ty = self.resolve_type_vars(field_ty, &type_param_names);
                        self.infer_type_args(&resolved_field_ty, &arg_ty, &mut type_subst, arg.span)?;
                    }

                    // v0.16: Type params not appearing in variant fields remain as TypeVar
                    // They will be resolved from context (return type annotation, unification)
                    // e.g., Result::Ok(value) infers T from value, E remains TypeVar

                    // Build instantiated type: e.g., Option<i64>
                    let type_args: Vec<Box<Type>> = type_params.iter()
                        .map(|tp| Box::new(type_subst.get(&tp.name).cloned().unwrap_or(Type::TypeVar(tp.name.clone()))))
                        .collect();

                    return Ok(Type::Generic {
                        name: enum_name.clone(),
                        type_args,
                    });
                }

                // v0.63: Suggest similar type names (enums and structs)
                let mut all_types: Vec<&str> = self.enums.keys().map(|s| s.as_str()).collect();
                all_types.extend(self.generic_enums.keys().map(|s| s.as_str()));
                all_types.extend(self.structs.keys().map(|s| s.as_str()));
                all_types.extend(self.generic_structs.keys().map(|s| s.as_str()));
                let suggestion = find_similar_name(enum_name, &all_types, 2);
                Err(CompileError::type_error(
                    format!("undefined enum: `{}`{}", enum_name, format_suggestion_hint(suggestion)),
                    span,
                ))
            }

            Expr::Match { expr: match_expr, arms } => {
                let match_ty = self.infer(&match_expr.node, match_expr.span)?;

                if arms.is_empty() {
                    return Ok(Type::Unit);
                }

                // All arms must have the same result type
                let mut result_ty: Option<Type> = None;

                for arm in arms {
                    // v0.48: Push scope for match arm bindings
                    self.binding_tracker.push_scope();

                    // Check pattern against match expression type
                    self.check_pattern(&arm.pattern.node, &match_ty, arm.pattern.span)?;

                    // v0.40: Check guard expression if present
                    if let Some(guard) = &arm.guard {
                        let guard_ty = self.infer(&guard.node, guard.span)?;
                        self.unify(&Type::Bool, &guard_ty, guard.span)?;
                    }

                    // Infer body type with pattern bindings
                    let body_ty = self.infer(&arm.body.node, arm.body.span)?;

                    // v0.48: Check for unused bindings and emit warnings
                    // Note: Match bindings are immutable, so no unused_mut check needed
                    let (unused, _unused_mut) = self.binding_tracker.pop_scope();
                    for (unused_name, unused_span) in unused {
                        self.add_warning(CompileWarning::unused_binding(unused_name, unused_span));
                    }

                    match &result_ty {
                        None => result_ty = Some(body_ty),
                        Some(expected) => self.unify(expected, &body_ty, arm.body.span)?,
                    }
                }

                // v0.46: Exhaustiveness checking
                let exhaustiveness_result = self.check_match_exhaustiveness(&match_ty, arms, span)?;

                // v0.47: Emit warnings for unreachable arms
                for &arm_idx in &exhaustiveness_result.unreachable_arms {
                    if arm_idx < arms.len() {
                        let arm = &arms[arm_idx];
                        self.add_warning(CompileWarning::unreachable_pattern(
                            "this pattern will never match because previous patterns cover all cases",
                            arm.pattern.span,
                            arm_idx,
                        ));
                    }
                }

                // v0.51: Warn if guards are present without unconditional fallback
                // This catches potential runtime "no match found" errors
                if exhaustiveness_result.has_guards_without_fallback {
                    self.add_warning(CompileWarning::guarded_non_exhaustive(span));
                }

                // Error if not exhaustive (unless there's a guard, which makes analysis harder)
                let has_guards = arms.iter().any(|a| a.guard.is_some());
                if !exhaustiveness_result.is_exhaustive && !has_guards {
                    // v0.59: Enhanced error formatting for missing patterns
                    let missing = &exhaustiveness_result.missing_patterns;
                    let error_msg = if missing.len() == 1 {
                        format!("non-exhaustive patterns: `{}` not covered", missing[0])
                    } else if missing.len() <= 3 {
                        format!(
                            "non-exhaustive patterns: {} not covered",
                            missing.iter().map(|p| format!("`{}`", p)).collect::<Vec<_>>().join(", ")
                        )
                    } else {
                        // Truncate long lists with "and N more"
                        let shown: Vec<_> = missing.iter().take(3).map(|p| format!("`{}`", p)).collect();
                        format!(
                            "non-exhaustive patterns: {} and {} more not covered",
                            shown.join(", "),
                            missing.len() - 3
                        )
                    };
                    // Add hint for how to fix
                    let hint = "\n  hint: add a wildcard pattern `_ => ...` to handle remaining cases";
                    return Err(CompileError::type_error(format!("{}{}", error_msg, hint), span));
                }

                Ok(result_ty.unwrap_or(Type::Unit))
            }

            // v0.5 Phase 5: References
            Expr::Ref(inner) => {
                let inner_ty = self.infer(&inner.node, inner.span)?;
                Ok(Type::Ref(Box::new(inner_ty)))
            }

            Expr::RefMut(inner) => {
                let inner_ty = self.infer(&inner.node, inner.span)?;
                Ok(Type::RefMut(Box::new(inner_ty)))
            }

            Expr::Deref(inner) => {
                let inner_ty = self.infer(&inner.node, inner.span)?;
                match inner_ty {
                    Type::Ref(t) | Type::RefMut(t) => Ok(*t),
                    // v0.60.20: Support dereferencing raw pointers (*T)
                    // This enables native pointer operations for performance-critical code
                    Type::Ptr(t) => Ok(*t),
                    _ => Err(CompileError::type_error(format!("Cannot dereference non-pointer type: {}", inner_ty), span)),
                }
            }

            // v0.5 Phase 6: Arrays
            Expr::ArrayLit(elems) => {
                if elems.is_empty() {
                    // Empty array needs type annotation (for now, default to i64)
                    Ok(Type::Array(Box::new(Type::I64), 0))
                } else {
                    let first_ty = self.infer(&elems[0].node, elems[0].span)?;
                    for elem in elems.iter().skip(1) {
                        let elem_ty = self.infer(&elem.node, elem.span)?;
                        self.unify(&first_ty, &elem_ty, elem.span)?;
                    }
                    Ok(Type::Array(Box::new(first_ty), elems.len()))
                }
            }

            // v0.60.22: Array repeat syntax [val; N]
            Expr::ArrayRepeat { value, count } => {
                let elem_ty = self.infer(&value.node, value.span)?;
                Ok(Type::Array(Box::new(elem_ty), *count))
            }

            // v0.42: Tuple expressions
            Expr::Tuple(elems) => {
                // Tuples are heterogeneous - each element has its own type
                let mut elem_types = Vec::with_capacity(elems.len());
                for elem in elems {
                    elem_types.push(Box::new(self.infer(&elem.node, elem.span)?));
                }
                Ok(Type::Tuple(elem_types))
            }

            Expr::Index { expr, index } => {
                let expr_ty = self.infer(&expr.node, expr.span)?;
                let index_ty = self.infer(&index.node, index.span)?;

                // Index must be an integer (v0.2: handle refined types, v0.38: include unsigned)
                match index_ty.base_type() {
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 => {}
                    _ => return Err(CompileError::type_error(format!("Array index must be integer, got: {}", index_ty), index.span)),
                }

                // Expression must be an array, reference to array, or pointer (v0.50.26, v0.60.19)
                match &expr_ty {
                    Type::Array(elem_ty, _) => Ok(*elem_ty.clone()),
                    // v0.50.26: Support indexing through references to arrays
                    Type::Ref(inner) => match inner.as_ref() {
                        Type::Array(elem_ty, _) => Ok(*elem_ty.clone()),
                        Type::String => Ok(Type::I64),
                        _ => Err(CompileError::type_error(
                            format!("Cannot index into reference type: &{}", inner),
                            expr.span,
                        )),
                    },
                    // v0.60.19: Support pointer indexing: ptr[i] returns element type
                    // This enables proper LLVM GEP generation instead of inttoptr
                    Type::Ptr(elem_ty) => Ok(*elem_ty.clone()),
                    Type::String => Ok(Type::I64), // String indexing returns char code
                    _ => Err(CompileError::type_error(format!("Cannot index into type: {}", expr_ty), expr.span)),
                }
            }

            // v0.51: Index assignment: arr[i] = value
            Expr::IndexAssign { array, index, value } => {
                let array_ty = self.infer(&array.node, array.span)?;
                let index_ty = self.infer(&index.node, index.span)?;
                let value_ty = self.infer(&value.node, value.span)?;

                // Index must be an integer
                match index_ty.base_type() {
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 => {}
                    _ => return Err(CompileError::type_error(
                        format!("Array index must be integer, got: {}", index_ty),
                        index.span,
                    )),
                }

                // Array must be a mutable array type or pointer (v0.60.19)
                let elem_ty = match &array_ty {
                    Type::Array(elem_ty, _) => *elem_ty.clone(),
                    // v0.60.19: Support pointer index assignment: ptr[i] = value
                    Type::Ptr(elem_ty) => *elem_ty.clone(),
                    _ => return Err(CompileError::type_error(
                        format!("Cannot assign to index of non-array type: {}", array_ty),
                        array.span,
                    )),
                };

                // Value must match element type (v0.51.40: use unify for null support)
                self.unify(&elem_ty, &value_ty, value.span)?;

                // IndexAssign returns unit
                Ok(Type::Unit)
            }

            // v0.51.23: Field assignment: obj.field = value
            Expr::FieldAssign { object, field, value } => {
                let obj_ty = self.infer(&object.node, object.span)?;
                let value_ty = self.infer(&value.node, value.span)?;

                // Object must be a struct type
                let field_ty = match &obj_ty {
                    Type::Named(struct_name) => {
                        let struct_fields = self.structs.get(struct_name).ok_or_else(|| {
                            CompileError::type_error(
                                format!("Cannot assign to field of non-struct type: {}", struct_name),
                                object.span,
                            )
                        })?;

                        // Find the field
                        let mut found_ty = None;
                        for (fname, fty) in struct_fields {
                            if fname == &field.node {
                                found_ty = Some(fty.clone());
                                break;
                            }
                        }

                        found_ty.ok_or_else(|| {
                            // v0.60: Suggest similar field names
                            let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                            let suggestion = find_similar_name(&field.node, &field_names, 2);
                            CompileError::type_error(
                                format!("Unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                                field.span,
                            )
                        })?
                    }
                    Type::Generic { name: struct_name, type_args } => {
                        // Handle generic struct field assignment
                        if let Some((type_params, struct_fields)) = self.generic_structs.get(struct_name).cloned() {
                            // Build type substitution
                            let mut type_subst: HashMap<String, Type> = HashMap::new();
                            for (tp, arg) in type_params.iter().zip(type_args.iter()) {
                                type_subst.insert(tp.name.clone(), (**arg).clone());
                            }

                            // Find the field and substitute types
                            let mut found_ty = None;
                            for (fname, fty) in &struct_fields {
                                if fname == &field.node {
                                    found_ty = Some(self.substitute_type(fty, &type_subst));
                                    break;
                                }
                            }

                            found_ty.ok_or_else(|| {
                                let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                                let suggestion = find_similar_name(&field.node, &field_names, 2);
                                CompileError::type_error(
                                    format!("Unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                                    field.span,
                                )
                            })?
                        } else {
                            return Err(CompileError::type_error(
                                format!("Cannot assign to field of non-struct type: {}", obj_ty),
                                object.span,
                            ));
                        }
                    }
                    // v0.51.37: Auto-dereference pointer types for field assignment
                    // set (*Node).left = value automatically dereferences to set Node.left
                    Type::Ptr(inner) => {
                        match inner.as_ref() {
                            Type::Named(struct_name) => {
                                let struct_fields = self.structs.get(struct_name).ok_or_else(|| {
                                    CompileError::type_error(
                                        format!("Cannot assign to field of non-struct type: {}", struct_name),
                                        object.span,
                                    )
                                })?;

                                let mut found_ty = None;
                                for (fname, fty) in struct_fields {
                                    if fname == &field.node {
                                        found_ty = Some(fty.clone());
                                        break;
                                    }
                                }

                                found_ty.ok_or_else(|| {
                                    let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                                    let suggestion = find_similar_name(&field.node, &field_names, 2);
                                    CompileError::type_error(
                                        format!("Unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                                        field.span,
                                    )
                                })?
                            }
                            Type::Generic { name: struct_name, type_args } => {
                                if let Some((type_params, struct_fields)) = self.generic_structs.get(struct_name).cloned() {
                                    let mut type_subst: HashMap<String, Type> = HashMap::new();
                                    for (tp, arg) in type_params.iter().zip(type_args.iter()) {
                                        type_subst.insert(tp.name.clone(), (**arg).clone());
                                    }

                                    let mut found_ty = None;
                                    for (fname, fty) in &struct_fields {
                                        if fname == &field.node {
                                            found_ty = Some(self.substitute_type(fty, &type_subst));
                                            break;
                                        }
                                    }

                                    found_ty.ok_or_else(|| {
                                        let field_names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                                        let suggestion = find_similar_name(&field.node, &field_names, 2);
                                        CompileError::type_error(
                                            format!("Unknown field `{}` on struct `{}`{}", field.node, struct_name, format_suggestion_hint(suggestion)),
                                            field.span,
                                        )
                                    })?
                                } else {
                                    return Err(CompileError::type_error(
                                        format!("Cannot assign to field of non-struct type: {}", obj_ty),
                                        object.span,
                                    ));
                                }
                            }
                            _ => {
                                return Err(CompileError::type_error(
                                    format!("Cannot assign to field on pointer to non-struct type: {}", inner),
                                    object.span,
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(CompileError::type_error(
                            format!("Cannot assign to field of non-struct type: {}", obj_ty),
                            object.span,
                        ));
                    }
                };

                // Value must match field type (v0.51.40: use unify for null support)
                self.unify(&field_ty, &value_ty, value.span)?;

                // FieldAssign returns unit
                Ok(Type::Unit)
            }

            // v0.60.21: Dereference assignment: set *ptr = value
            Expr::DerefAssign { ptr, value } => {
                let ptr_ty = self.infer(&ptr.node, ptr.span)?;
                let value_ty = self.infer(&value.node, value.span)?;

                // ptr must be a pointer type
                let elem_ty = match &ptr_ty {
                    Type::Ptr(inner) => *inner.clone(),
                    _ => return Err(CompileError::type_error(
                        format!("Cannot dereference and assign to non-pointer type: {}", ptr_ty),
                        ptr.span,
                    )),
                };

                // Value must match element type
                self.unify(&elem_ty, &value_ty, value.span)?;

                // DerefAssign returns unit
                Ok(Type::Unit)
            }

            // v0.5 Phase 8: Method calls
            Expr::MethodCall { receiver, method, args } => {
                let receiver_ty = self.infer(&receiver.node, receiver.span)?;
                self.check_method_call(&receiver_ty, method, args, span)
            }

            // v0.2: State references for contracts
            Expr::StateRef { expr, .. } => {
                // The type of a state reference is the same as the underlying expression
                self.infer(&expr.node, expr.span)
            }

            // v0.2: Refinement self-reference (type depends on context)
            // When used in T{constraints}, 'it' has type T
            Expr::It => {
                // For now, return a placeholder type; actual type comes from context
                Ok(Type::I64)
            }

            // v0.20.0: Closure expressions
            Expr::Closure { params, ret_ty, body } => {
                // Save current environment for capture analysis
                let outer_env = self.env.clone();

                // v0.50: Push scope for closure parameter tracking
                self.binding_tracker.push_scope();

                // Collect parameter types and add to environment
                let mut param_types: Vec<Box<Type>> = Vec::new();
                for param in params {
                    let param_ty = if let Some(ty) = &param.ty {
                        ty.node.clone()
                    } else {
                        // Type inference for unannotated parameters is future work
                        return Err(CompileError::type_error(
                            format!("closure parameter '{}' requires type annotation", param.name.node),
                            param.name.span,
                        ));
                    };
                    param_types.push(Box::new(param_ty.clone()));
                    self.env.insert(param.name.node.clone(), param_ty);
                    // v0.50: Track closure parameter binding for unused detection
                    self.binding_tracker.bind(param.name.node.clone(), param.name.span);
                }

                // Infer body type
                let body_ty = self.infer(&body.node, body.span)?;

                // Check against explicit return type if provided
                if let Some(explicit_ret) = ret_ty {
                    self.unify(&explicit_ret.node, &body_ty, body.span)?;
                }

                // v0.50: Check for unused closure parameters and emit warnings
                // Note: Closure parameters are immutable, so no unused_mut check needed
                let (unused, _unused_mut) = self.binding_tracker.pop_scope();
                for (unused_name, unused_span) in unused {
                    self.add_warning(CompileWarning::unused_binding(unused_name, unused_span));
                }

                // Restore outer environment (closure doesn't pollute outer scope)
                self.env = outer_env;

                // Return function type: fn(params) -> body_ty
                Ok(Type::Fn {
                    params: param_types,
                    ret: Box::new(body_ty),
                })
            }

            // v0.31: Todo expression - type checks as the "never" type
            // Never type is compatible with any type (bottom type)
            // This allows `todo` to be used as a placeholder in any context
            Expr::Todo { .. } => {
                Ok(Type::Never)
            }

            // v0.36: Additional control flow
            // Loop returns Never (infinite loop or break)
            Expr::Loop { body } => {
                // Type check the body but return Never
                self.infer(&body.node, body.span)?;
                Ok(Type::Never)
            }

            // Break returns Never (control flow transfer)
            Expr::Break { value } => {
                if let Some(v) = value {
                    self.infer(&v.node, v.span)?;
                }
                Ok(Type::Never)
            }

            // Continue returns Never (control flow transfer)
            Expr::Continue => {
                Ok(Type::Never)
            }

            // Return returns Never (control flow transfer)
            Expr::Return { value } => {
                if let Some(v) = value {
                    self.infer(&v.node, v.span)?;
                }
                Ok(Type::Never)
            }

            // v0.37: Quantifiers - return Bool
            // forall x: T, body
            Expr::Forall { var, ty, body } => {
                // Add bound variable to environment for body type checking
                self.env.insert(var.node.clone(), ty.node.clone());
                let body_ty = self.infer(&body.node, body.span)?;
                // Remove bound variable from environment
                self.env.remove(&var.node);
                // Body must be a boolean expression
                self.unify(&Type::Bool, &body_ty, body.span)?;
                Ok(Type::Bool)
            }

            // exists x: T, body
            Expr::Exists { var, ty, body } => {
                // Add bound variable to environment for body type checking
                self.env.insert(var.node.clone(), ty.node.clone());
                let body_ty = self.infer(&body.node, body.span)?;
                // Remove bound variable from environment
                self.env.remove(&var.node);
                // Body must be a boolean expression
                self.unify(&Type::Bool, &body_ty, body.span)?;
                Ok(Type::Bool)
            }

            // v0.39: Type cast: expr as Type
            // v0.51.32: Extended to support struct pointer casts
            Expr::Cast { expr, ty } => {
                // Infer source expression type
                let src_ty = self.infer(&expr.node, expr.span)?;
                let target_ty = ty.node.clone();

                // Validate cast is allowed
                let src_numeric = matches!(&src_ty, Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 | Type::Bool);
                let tgt_numeric = matches!(&target_ty, Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 | Type::Bool);

                // v0.51.32: Allow struct <-> i64 casts for pointer operations
                // v0.51.37: Extended to support typed pointer types (*T)
                let src_struct = matches!(&src_ty, Type::Struct { .. } | Type::Named(_));
                let tgt_struct = matches!(&target_ty, Type::Struct { .. } | Type::Named(_));
                let src_is_i64 = matches!(&src_ty, Type::I64);
                let tgt_is_i64 = matches!(&target_ty, Type::I64);
                let src_ptr = matches!(&src_ty, Type::Ptr(_));
                let tgt_ptr = matches!(&target_ty, Type::Ptr(_));

                // v0.60.23: Array to pointer cast - [T; N] -> *T
                let array_to_ptr = match (&src_ty, &target_ty) {
                    (Type::Array(elem_ty, _), Type::Ptr(ptr_elem_ty)) => elem_ty == ptr_elem_ty,
                    _ => false,
                };

                // Allow: numeric <-> numeric, struct -> i64, i64 -> struct
                // v0.51.37: Also allow i64 <-> *T and *T <-> *U casts
                // v0.60.23: Also allow [T; N] -> *T (array decay to pointer)
                let valid_cast = (src_numeric && tgt_numeric)
                    || (src_struct && tgt_is_i64)  // struct pointer to i64
                    || (src_is_i64 && tgt_struct)  // i64 to struct pointer
                    || (src_is_i64 && tgt_ptr)     // i64 to *T (malloc result)
                    || (src_ptr && tgt_is_i64)     // *T to i64 (null check, arithmetic)
                    || (src_ptr && tgt_ptr)        // *T to *U (pointer cast)
                    || array_to_ptr;               // [T; N] -> *T (array decay)

                if !valid_cast {
                    return Err(CompileError::type_error(
                        format!("cannot cast {} to {}: only numeric types and struct pointers are supported", src_ty, target_ty),
                        span,
                    ));
                }

                Ok(target_ty)
            }
        }
    }

    /// Check method call types (v0.5 Phase 8)
    fn check_method_call(&mut self, receiver_ty: &Type, method: &str, args: &[Spanned<Expr>], span: Span) -> Result<Type> {
        match receiver_ty {
            // v0.90.36: Bool methods
            Type::Bool => {
                match method {
                    "to_string" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_string() takes no arguments", span));
                        }
                        Ok(Type::String)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for bool", method), span)),
                }
            }
            // v0.90.35: Integer methods
            Type::I64 | Type::I32 | Type::U32 | Type::U64 => {
                match method {
                    "abs" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("abs() takes no arguments", span));
                        }
                        Ok(receiver_ty.clone())
                    }
                    "min" | "max" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error(
                                format!("{}() takes 1 argument", method), span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, receiver_ty, args[0].span)?;
                        Ok(receiver_ty.clone())
                    }
                    "clamp" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("clamp() takes 2 arguments", span));
                        }
                        for arg in args {
                            let arg_ty = self.infer(&arg.node, arg.span)?;
                            self.unify(&arg_ty, receiver_ty, arg.span)?;
                        }
                        Ok(receiver_ty.clone())
                    }
                    "pow" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("pow() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, receiver_ty, args[0].span)?;
                        Ok(receiver_ty.clone())
                    }
                    "to_float" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_float() takes no arguments", span));
                        }
                        Ok(Type::F64)
                    }
                    "to_string" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_string() takes no arguments", span));
                        }
                        Ok(Type::String)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for {}", method, receiver_ty), span)),
                }
            }
            // v0.90.34: Float methods
            Type::F64 => {
                match method {
                    "abs" | "floor" | "ceil" | "round" | "sqrt" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error(
                                format!("{}() takes no arguments", method), span));
                        }
                        Ok(Type::F64)
                    }
                    "is_nan" | "is_infinite" | "is_finite" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error(
                                format!("{}() takes no arguments", method), span));
                        }
                        Ok(Type::Bool)
                    }
                    "min" | "max" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error(
                                format!("{}() takes 1 argument", method), span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &Type::F64, args[0].span)?;
                        Ok(Type::F64)
                    }
                    "to_int" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_int() takes no arguments", span));
                        }
                        Ok(Type::I64)
                    }
                    // v0.90.36: to_string() -> String
                    "to_string" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_string() takes no arguments", span));
                        }
                        Ok(Type::String)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for f64", method), span)),
                }
            }
            Type::String => {
                match method {
                    // len() -> i64
                    "len" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("len() takes no arguments", span));
                        }
                        Ok(Type::I64)
                    }
                    // byte_at(index: i64) -> i64
                    // v0.67: Renamed from char_at for clarity (returns byte, not Unicode char)
                    // Use char_at(s, idx) function for Unicode character access
                    "byte_at" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("byte_at() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        match arg_ty {
                            // v0.38: Include unsigned types
                            Type::I32 | Type::I64 | Type::U32 | Type::U64 => Ok(Type::I64),
                            _ => Err(CompileError::type_error(
                                format!("byte_at() requires integer argument, got {}", arg_ty),
                                args[0].span,
                            )),
                        }
                    }
                    // slice(start: i64, end: i64) -> String
                    "slice" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("slice() takes 2 arguments", span));
                        }
                        for arg in args {
                            let arg_ty = self.infer(&arg.node, arg.span)?;
                            match arg_ty {
                                // v0.38: Include unsigned types
                                Type::I32 | Type::I64 | Type::U32 | Type::U64 => {}
                                _ => return Err(CompileError::type_error(
                                    format!("slice() requires integer arguments, got {}", arg_ty),
                                    arg.span,
                                )),
                            }
                        }
                        Ok(Type::String)
                    }
                    // is_empty() -> bool
                    "is_empty" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("is_empty() takes no arguments", span));
                        }
                        Ok(Type::Bool)
                    }
                    // v0.90.32: to_upper() -> String
                    "to_upper" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_upper() takes no arguments", span));
                        }
                        Ok(Type::String)
                    }
                    // v0.90.32: to_lower() -> String
                    "to_lower" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_lower() takes no arguments", span));
                        }
                        Ok(Type::String)
                    }
                    // v0.90.32: trim() -> String
                    "trim" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("trim() takes no arguments", span));
                        }
                        Ok(Type::String)
                    }
                    // v0.90.32: contains(String) -> bool
                    "contains" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("contains() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &Type::String, args[0].span)?;
                        Ok(Type::Bool)
                    }
                    // v0.90.32: starts_with(String) -> bool
                    "starts_with" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("starts_with() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &Type::String, args[0].span)?;
                        Ok(Type::Bool)
                    }
                    // v0.90.32: ends_with(String) -> bool
                    "ends_with" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("ends_with() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &Type::String, args[0].span)?;
                        Ok(Type::Bool)
                    }
                    // v0.90.32: replace(String, String) -> String
                    "replace" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("replace() takes 2 arguments", span));
                        }
                        for arg in args {
                            let arg_ty = self.infer(&arg.node, arg.span)?;
                            self.unify(&arg_ty, &Type::String, arg.span)?;
                        }
                        Ok(Type::String)
                    }
                    // v0.90.32: repeat(i64) -> String
                    "repeat" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("repeat() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        match arg_ty {
                            Type::I32 | Type::I64 | Type::U32 | Type::U64 => {}
                            _ => return Err(CompileError::type_error(
                                format!("repeat() requires integer argument, got {}", arg_ty),
                                args[0].span,
                            )),
                        }
                        Ok(Type::String)
                    }
                    // v0.90.32: split(String) -> [String]
                    "split" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("split() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &Type::String, args[0].span)?;
                        Ok(Type::Array(Box::new(Type::String), 0))
                    }
                    // v0.90.32: index_of(String) -> i64?
                    "index_of" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("index_of() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &Type::String, args[0].span)?;
                        Ok(Type::Nullable(Box::new(Type::I64)))
                    }
                    // v0.90.36: to_int() -> i64?
                    "to_int" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_int() takes no arguments", span));
                        }
                        Ok(Type::Nullable(Box::new(Type::I64)))
                    }
                    // v0.90.36: to_float() -> f64?
                    "to_float" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("to_float() takes no arguments", span));
                        }
                        Ok(Type::Nullable(Box::new(Type::F64)))
                    }
                    // v0.90.36: chars() -> [String]
                    "chars" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("chars() takes no arguments", span));
                        }
                        Ok(Type::Array(Box::new(Type::String), 0))
                    }
                    // v0.90.36: reverse() -> String
                    "reverse" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("reverse() takes no arguments", span));
                        }
                        Ok(Type::String)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for String", method),
                        span,
                    )),
                }
            }
            Type::Array(elem_ty, _) => {
                match method {
                    // len() -> i64
                    "len" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("len() takes no arguments", span));
                        }
                        Ok(Type::I64)
                    }
                    // v0.90.31: is_empty() -> bool
                    "is_empty" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("is_empty() takes no arguments", span));
                        }
                        Ok(Type::Bool)
                    }
                    // v0.90.31: first() -> T
                    "first" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("first() takes no arguments", span));
                        }
                        Ok(*elem_ty.clone())
                    }
                    // v0.90.31: last() -> T
                    "last" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("last() takes no arguments", span));
                        }
                        Ok(*elem_ty.clone())
                    }
                    // v0.90.31: contains(T) -> bool
                    "contains" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("contains() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, elem_ty, args[0].span)?;
                        Ok(Type::Bool)
                    }
                    // v0.90.31: get(i64) -> T?
                    "get" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("get() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        match arg_ty {
                            Type::I32 | Type::I64 | Type::U32 | Type::U64 => {}
                            _ => return Err(CompileError::type_error(
                                format!("get() requires integer argument, got {}", arg_ty),
                                args[0].span,
                            )),
                        }
                        Ok(Type::Nullable(elem_ty.clone()))
                    }
                    // v0.90.31: reverse() -> [T]
                    "reverse" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("reverse() takes no arguments", span));
                        }
                        Ok(Type::Array(elem_ty.clone(), 0))
                    }
                    // v0.90.37: push(T) -> [T]
                    "push" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("push() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, elem_ty, args[0].span)?;
                        Ok(Type::Array(elem_ty.clone(), 0))
                    }
                    // v0.90.37: pop() -> [T]
                    "pop" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("pop() takes no arguments", span));
                        }
                        Ok(Type::Array(elem_ty.clone(), 0))
                    }
                    // v0.90.37: concat([T]) -> [T]
                    "concat" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("concat() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, receiver_ty, args[0].span)?;
                        Ok(Type::Array(elem_ty.clone(), 0))
                    }
                    // v0.90.37: slice(i64, i64) -> [T]
                    "slice" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("slice() takes 2 arguments", span));
                        }
                        for arg in args {
                            let arg_ty = self.infer(&arg.node, arg.span)?;
                            match arg_ty {
                                Type::I32 | Type::I64 | Type::U32 | Type::U64 => {}
                                _ => return Err(CompileError::type_error(
                                    format!("slice() requires integer arguments, got {}", arg_ty),
                                    arg.span,
                                )),
                            }
                        }
                        Ok(Type::Array(elem_ty.clone(), 0))
                    }
                    // v0.90.37: join(String) -> String
                    "join" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("join() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &Type::String, args[0].span)?;
                        Ok(Type::String)
                    }
                    // v0.90.38: map(fn(T) -> U) -> [U]
                    "map" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("map() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 1 {
                                    return Err(CompileError::type_error(
                                        format!("map() closure must take 1 parameter, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                Ok(Type::Array(ret, 0))
                            }
                            _ => Err(CompileError::type_error(
                                "map() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.38: filter(fn(T) -> bool) -> [T]
                    "filter" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("filter() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 1 {
                                    return Err(CompileError::type_error(
                                        format!("filter() closure must take 1 parameter, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                self.unify(&ret, &Type::Bool, args[0].span)?;
                                Ok(Type::Array(elem_ty.clone(), 0))
                            }
                            _ => Err(CompileError::type_error(
                                "filter() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.38: any(fn(T) -> bool) -> bool
                    "any" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("any() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 1 {
                                    return Err(CompileError::type_error(
                                        format!("any() closure must take 1 parameter, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                self.unify(&ret, &Type::Bool, args[0].span)?;
                                Ok(Type::Bool)
                            }
                            _ => Err(CompileError::type_error(
                                "any() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.38: all(fn(T) -> bool) -> bool
                    "all" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("all() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 1 {
                                    return Err(CompileError::type_error(
                                        format!("all() closure must take 1 parameter, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                self.unify(&ret, &Type::Bool, args[0].span)?;
                                Ok(Type::Bool)
                            }
                            _ => Err(CompileError::type_error(
                                "all() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.38: for_each(fn(T) -> ()) -> ()
                    "for_each" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("for_each() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, .. } => {
                                if params.len() != 1 {
                                    return Err(CompileError::type_error(
                                        format!("for_each() closure must take 1 parameter, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                Ok(Type::Unit)
                            }
                            _ => Err(CompileError::type_error(
                                "for_each() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.39: fold(init, fn(acc, T) -> acc) -> acc
                    "fold" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("fold() takes 2 arguments (initial value and closure)", span));
                        }
                        let init_ty = self.infer(&args[0].node, args[0].span)?;
                        let fn_ty = self.infer(&args[1].node, args[1].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 2 {
                                    return Err(CompileError::type_error(
                                        format!("fold() closure must take 2 parameters (accumulator, element), got {}", params.len()),
                                        args[1].span,
                                    ));
                                }
                                self.unify(&params[0], &init_ty, args[1].span)?;
                                self.unify(&params[1], elem_ty, args[1].span)?;
                                self.unify(&ret, &init_ty, args[1].span)?;
                                Ok(init_ty)
                            }
                            _ => Err(CompileError::type_error(
                                "fold() requires a closure as second argument",
                                args[1].span,
                            )),
                        }
                    }
                    // v0.90.39: reduce(fn(T, T) -> T) -> T?
                    "reduce" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("reduce() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 2 {
                                    return Err(CompileError::type_error(
                                        format!("reduce() closure must take 2 parameters, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                self.unify(&params[1], elem_ty, args[0].span)?;
                                self.unify(&ret, elem_ty, args[0].span)?;
                                Ok(Type::Nullable(elem_ty.clone()))
                            }
                            _ => Err(CompileError::type_error(
                                "reduce() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.39: find(fn(T) -> bool) -> T?
                    "find" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("find() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 1 {
                                    return Err(CompileError::type_error(
                                        format!("find() closure must take 1 parameter, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                self.unify(&ret, &Type::Bool, args[0].span)?;
                                Ok(Type::Nullable(elem_ty.clone()))
                            }
                            _ => Err(CompileError::type_error(
                                "find() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.39: position(fn(T) -> bool) -> i64?
                    "position" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("position() takes 1 argument (a closure)", span));
                        }
                        let fn_ty = self.infer(&args[0].node, args[0].span)?;
                        match fn_ty {
                            Type::Fn { params, ret } => {
                                if params.len() != 1 {
                                    return Err(CompileError::type_error(
                                        format!("position() closure must take 1 parameter, got {}", params.len()),
                                        args[0].span,
                                    ));
                                }
                                self.unify(&params[0], elem_ty, args[0].span)?;
                                self.unify(&ret, &Type::Bool, args[0].span)?;
                                Ok(Type::Nullable(Box::new(Type::I64)))
                            }
                            _ => Err(CompileError::type_error(
                                "position() requires a closure argument",
                                args[0].span,
                            )),
                        }
                    }
                    // v0.90.39: enumerate() -> [(i64, T)]
                    "enumerate" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("enumerate() takes no arguments", span));
                        }
                        Ok(Type::Array(Box::new(Type::Tuple(vec![
                            Box::new(Type::I64),
                            elem_ty.clone(),
                        ])), 0))
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Array", method),
                        span,
                    )),
                }
            }
            // v0.18: Option<T> methods
            Type::Named(name) if name == "Option" => {
                self.check_option_method(method, args, None, span)
            }
            Type::Generic { name, type_args } if name == "Option" => {
                let inner_ty = type_args.first().map(|t| t.as_ref().clone());
                self.check_option_method(method, args, inner_ty, span)
            }
            // v0.76: Nullable<T> (T?) methods - same as Option<T>
            Type::Nullable(inner_ty) => {
                self.check_option_method(method, args, Some(*inner_ty.clone()), span)
            }
            // v0.18: Result<T, E> methods
            Type::Named(name) if name == "Result" => {
                self.check_result_method(method, args, None, None, span)
            }
            Type::Generic { name, type_args } if name == "Result" => {
                let ok_ty = type_args.first().map(|t| t.as_ref().clone());
                let err_ty = type_args.get(1).map(|t| t.as_ref().clone());
                self.check_result_method(method, args, ok_ty, err_ty, span)
            }
            // v0.70: Thread<T> methods
            Type::Thread(inner_ty) => {
                match method {
                    // join() -> T - blocks until thread completes, returns result
                    "join" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("join() takes no arguments", span));
                        }
                        Ok(*inner_ty.clone())
                    }
                    // is_alive() -> bool - checks if thread is still running
                    "is_alive" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("is_alive() takes no arguments", span));
                        }
                        Ok(Type::Bool)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Thread<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.71: Mutex<T> methods
            Type::Mutex(inner_ty) => {
                match method {
                    // lock() -> T - acquires lock and returns current value
                    "lock" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("lock() takes no arguments", span));
                        }
                        Ok(*inner_ty.clone())
                    }
                    // unlock(value: T) -> () - stores value and releases lock
                    "unlock" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("unlock() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(Type::Unit)
                    }
                    // try_lock() -> i64 - non-blocking lock attempt (returns 1 if success, 0 if failed)
                    "try_lock" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("try_lock() takes no arguments", span));
                        }
                        Ok(Type::I64)
                    }
                    // free() -> () - deallocates the mutex
                    "free" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("free() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    // with(fn(&mut T) -> R) -> R - RAII-based lock pattern
                    // Acquires lock, calls closure with mutable reference, releases lock
                    "with" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error(
                                "with() takes exactly one closure argument",
                                span,
                            ));
                        }
                        // The closure should be fn(&mut T) -> R
                        // For now, infer the closure's return type as the method's return type
                        let closure_ty = self.infer(&args[0].node, args[0].span)?;
                        match closure_ty {
                            Type::Fn { ret, .. } => Ok(*ret),
                            _ => Err(CompileError::type_error(
                                "with() requires a closure argument",
                                span,
                            )),
                        }
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Mutex<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.71: Handle Mutex<T> as Generic type (parsed as Generic { name: "Mutex", ... })
            Type::Generic { name, type_args } if name == "Mutex" => {
                let inner_ty = type_args.first().map(|t| t.as_ref().clone()).unwrap_or(Type::I64);
                match method {
                    "with" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error(
                                "with() takes exactly one closure argument",
                                span,
                            ));
                        }
                        let closure_ty = self.infer(&args[0].node, args[0].span)?;
                        match closure_ty {
                            Type::Fn { ret, .. } => Ok(*ret),
                            _ => Err(CompileError::type_error(
                                "with() requires a closure argument",
                                span,
                            )),
                        }
                    }
                    "try_lock" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("try_lock() takes no arguments", span));
                        }
                        Ok(Type::Nullable(Box::new(Type::RefMut(Box::new(inner_ty)))))
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Mutex<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.72: Arc<T> methods
            Type::Arc(inner_ty) => {
                match method {
                    // clone() -> Arc<T> - increments reference count
                    "clone" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("clone() takes no arguments", span));
                        }
                        Ok(Type::Arc(inner_ty.clone()))
                    }
                    // get() -> T - returns a copy of the value (for primitives)
                    "get" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("get() takes no arguments", span));
                        }
                        Ok(*inner_ty.clone())
                    }
                    // get_ref() -> &T - returns a reference to the value
                    "get_ref" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("get_ref() takes no arguments", span));
                        }
                        Ok(Type::Ref(inner_ty.clone()))
                    }
                    // strong_count() -> i64 - returns the reference count
                    "strong_count" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("strong_count() takes no arguments", span));
                        }
                        Ok(Type::I64)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Arc<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.72: Handle Arc<T> as Generic type
            Type::Generic { name, type_args } if name == "Arc" => {
                let inner_ty = type_args.first().map(|t| t.as_ref().clone()).unwrap_or(Type::I64);
                match method {
                    "clone" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("clone() takes no arguments", span));
                        }
                        Ok(Type::Generic { name: "Arc".to_string(), type_args: type_args.clone() })
                    }
                    "get" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("get() takes no arguments", span));
                        }
                        Ok(inner_ty)
                    }
                    "get_ref" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("get_ref() takes no arguments", span));
                        }
                        Ok(Type::Ref(Box::new(type_args.first().map(|t| t.as_ref().clone()).unwrap_or(Type::I64))))
                    }
                    "strong_count" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("strong_count() takes no arguments", span));
                        }
                        Ok(Type::I64)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Arc<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.72: Atomic<i64> methods
            Type::Atomic(inner_ty) => {
                match method {
                    // load() -> T - atomically load the current value
                    "load" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("load() takes no arguments", span));
                        }
                        Ok(*inner_ty.clone())
                    }
                    // store(value: T) -> () - atomically store a value
                    "store" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("store() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(Type::Unit)
                    }
                    // fetch_add(delta: T) -> T - atomically add and return old value
                    "fetch_add" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("fetch_add() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(*inner_ty.clone())
                    }
                    // fetch_sub(delta: T) -> T - atomically subtract and return old value
                    "fetch_sub" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("fetch_sub() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(*inner_ty.clone())
                    }
                    // swap(new: T) -> T - atomically swap and return old value
                    "swap" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("swap() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(*inner_ty.clone())
                    }
                    // compare_exchange(expected: T, new: T) -> T
                    // Returns the old value (caller checks if exchange happened)
                    "compare_exchange" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("compare_exchange() takes exactly two arguments", span));
                        }
                        let expected_ty = self.infer(&args[0].node, args[0].span)?;
                        let new_ty = self.infer(&args[1].node, args[1].span)?;
                        self.unify(&expected_ty, inner_ty, args[0].span)?;
                        self.unify(&new_ty, inner_ty, args[1].span)?;
                        Ok(*inner_ty.clone())
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Atomic<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.72: Handle Atomic<T> as Generic type
            Type::Generic { name, type_args } if name == "Atomic" => {
                let inner_ty = type_args.first().map(|t| t.as_ref().clone()).unwrap_or(Type::I64);
                match method {
                    "load" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("load() takes no arguments", span));
                        }
                        Ok(inner_ty)
                    }
                    "store" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("store() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &inner_ty, args[0].span)?;
                        Ok(Type::Unit)
                    }
                    "fetch_add" | "fetch_sub" | "swap" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error(format!("{}() takes exactly one argument", method), span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &inner_ty, args[0].span)?;
                        Ok(inner_ty)
                    }
                    "compare_exchange" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("compare_exchange() takes exactly two arguments", span));
                        }
                        let expected_ty = self.infer(&args[0].node, args[0].span)?;
                        let new_ty = self.infer(&args[1].node, args[1].span)?;
                        self.unify(&expected_ty, &inner_ty, args[0].span)?;
                        self.unify(&new_ty, &inner_ty, args[1].span)?;
                        Ok(inner_ty)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Atomic<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.73: Sender<T> methods
            Type::Sender(inner_ty) => {
                match method {
                    // send(value: T) -> () - blocking send
                    "send" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("send() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(Type::Unit)
                    }
                    // try_send(value: T) -> bool - non-blocking send
                    "try_send" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("try_send() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(Type::Bool)
                    }
                    // v0.79: send_timeout(value: T, ms: i64) -> bool - send with timeout
                    "send_timeout" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("send_timeout() takes exactly two arguments (value, timeout_ms)", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        let timeout_ty = self.infer(&args[1].node, args[1].span)?;
                        self.unify(&Type::I64, &timeout_ty, args[1].span)?;
                        Ok(Type::Bool)
                    }
                    // clone() -> Sender<T> - clone the sender for MPSC
                    "clone" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("clone() takes no arguments", span));
                        }
                        Ok(Type::Sender(inner_ty.clone()))
                    }
                    // v0.80: close() -> () - closes the channel, no more sends allowed
                    "close" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("close() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Sender<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.73: Handle Sender<T> as Generic type
            Type::Generic { name, type_args } if name == "Sender" => {
                let inner_ty = type_args.first().map(|t| t.as_ref().clone()).unwrap_or(Type::I64);
                match method {
                    "send" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("send() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &inner_ty, args[0].span)?;
                        Ok(Type::Unit)
                    }
                    "try_send" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("try_send() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &inner_ty, args[0].span)?;
                        Ok(Type::Bool)
                    }
                    // v0.79: send_timeout(value: T, ms: i64) -> bool - send with timeout
                    "send_timeout" => {
                        if args.len() != 2 {
                            return Err(CompileError::type_error("send_timeout() takes exactly two arguments (value, timeout_ms)", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &inner_ty, args[0].span)?;
                        let timeout_ty = self.infer(&args[1].node, args[1].span)?;
                        self.unify(&Type::I64, &timeout_ty, args[1].span)?;
                        Ok(Type::Bool)
                    }
                    "clone" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("clone() takes no arguments", span));
                        }
                        Ok(Type::Generic { name: "Sender".to_string(), type_args: type_args.clone() })
                    }
                    // v0.80: close() -> () - closes the channel, no more sends allowed
                    "close" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("close() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Sender<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.73: Receiver<T> methods
            Type::Receiver(inner_ty) => {
                match method {
                    // recv() -> T - blocking receive
                    "recv" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("recv() takes no arguments", span));
                        }
                        Ok(*inner_ty.clone())
                    }
                    // try_recv() -> T? - non-blocking receive
                    "try_recv" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("try_recv() takes no arguments", span));
                        }
                        Ok(Type::Nullable(inner_ty.clone()))
                    }
                    // v0.77: recv_timeout(ms: i64) -> T? - receive with timeout
                    "recv_timeout" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("recv_timeout() takes exactly one argument (timeout in ms)", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&Type::I64, &arg_ty, args[0].span)?;
                        Ok(Type::Nullable(inner_ty.clone()))
                    }
                    // v0.80: is_closed() -> bool - check if channel is closed
                    "is_closed" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("is_closed() takes no arguments", span));
                        }
                        Ok(Type::Bool)
                    }
                    // v0.80: recv_opt() -> T? - receive that distinguishes closed from empty
                    "recv_opt" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("recv_opt() takes no arguments", span));
                        }
                        Ok(Type::Nullable(inner_ty.clone()))
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Receiver<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.73: Handle Receiver<T> as Generic type
            Type::Generic { name, type_args } if name == "Receiver" => {
                let inner_ty = type_args.first().map(|t| t.as_ref().clone()).unwrap_or(Type::I64);
                match method {
                    "recv" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("recv() takes no arguments", span));
                        }
                        Ok(inner_ty)
                    }
                    "try_recv" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("try_recv() takes no arguments", span));
                        }
                        Ok(Type::Nullable(Box::new(inner_ty.clone())))
                    }
                    // v0.77: recv_timeout(ms: i64) -> T? - receive with timeout
                    "recv_timeout" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("recv_timeout() takes exactly one argument (timeout in ms)", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&Type::I64, &arg_ty, args[0].span)?;
                        Ok(Type::Nullable(Box::new(inner_ty.clone())))
                    }
                    // v0.80: is_closed() -> bool - check if channel is closed
                    "is_closed" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("is_closed() takes no arguments", span));
                        }
                        Ok(Type::Bool)
                    }
                    // v0.80: recv_opt() -> T? - receive that distinguishes closed from empty
                    "recv_opt" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("recv_opt() takes no arguments", span));
                        }
                        Ok(Type::Nullable(Box::new(inner_ty)))
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Receiver<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.74: RwLock<T> methods
            Type::RwLock(inner_ty) => {
                match method {
                    // read() -> T - acquires read lock and returns current value
                    "read" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("read() takes no arguments", span));
                        }
                        Ok(*inner_ty.clone())
                    }
                    // read_unlock() -> () - releases read lock
                    "read_unlock" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("read_unlock() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    // write() -> T - acquires write lock and returns current value
                    "write" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("write() takes no arguments", span));
                        }
                        Ok(*inner_ty.clone())
                    }
                    // write_unlock(value: T) -> () - stores value and releases write lock
                    "write_unlock" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("write_unlock() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, inner_ty, args[0].span)?;
                        Ok(Type::Unit)
                    }
                    // free() -> () - deallocates the rwlock
                    "free" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("free() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for RwLock<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.74: Handle RwLock<T> as Generic type
            Type::Generic { name, type_args } if name == "RwLock" => {
                let inner_ty = type_args.first().map(|t| t.as_ref().clone()).unwrap_or(Type::I64);
                match method {
                    "read" | "write" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error(format!("{}() takes no arguments", method), span));
                        }
                        Ok(inner_ty)
                    }
                    "read_unlock" | "free" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error(format!("{}() takes no arguments", method), span));
                        }
                        Ok(Type::Unit)
                    }
                    "write_unlock" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("write_unlock() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&arg_ty, &inner_ty, args[0].span)?;
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for RwLock<{}>", method, inner_ty),
                        span,
                    )),
                }
            }
            // v0.74: Barrier methods
            Type::Barrier => {
                match method {
                    // wait() -> bool - wait at barrier, returns true for "leader" thread
                    "wait" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("wait() takes no arguments", span));
                        }
                        Ok(Type::Bool)
                    }
                    // free() -> () - deallocates the barrier
                    "free" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("free() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Barrier", method),
                        span,
                    )),
                }
            }
            // v0.74: Condvar methods
            Type::Condvar => {
                match method {
                    // wait(mutex: Mutex<T>) -> T - wait on condvar, returns mutex value after wakeup
                    "wait" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("wait() takes exactly one mutex argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        match arg_ty {
                            Type::Mutex(inner_ty) => Ok(*inner_ty),
                            _ => Err(CompileError::type_error("wait() requires a Mutex argument", args[0].span)),
                        }
                    }
                    // notify_one() -> () - wake one waiting thread
                    "notify_one" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("notify_one() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    // notify_all() -> () - wake all waiting threads
                    "notify_all" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("notify_all() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    // free() -> () - deallocates the condvar
                    "free" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("free() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Condvar", method),
                        span,
                    )),
                }
            }
            // v0.83: AsyncFile methods for async I/O
            Type::AsyncFile => {
                match method {
                    // read() -> Future<String> - async read file content
                    "read" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("read() takes no arguments", span));
                        }
                        Ok(Type::Future(Box::new(Type::String)))
                    }
                    // write(content: String) -> Future<()> - async write content
                    "write" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("write() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&Type::String, &arg_ty, args[0].span)?;
                        Ok(Type::Future(Box::new(Type::Unit)))
                    }
                    // close() -> Future<()> - async close file
                    "close" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("close() takes no arguments", span));
                        }
                        Ok(Type::Future(Box::new(Type::Unit)))
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for AsyncFile", method),
                        span,
                    )),
                }
            }
            // v0.83.1: AsyncSocket methods for async network I/O
            // Note: Uses recv/send instead of read/write to distinguish from AsyncFile
            Type::AsyncSocket => {
                match method {
                    // recv() -> Future<String> - async receive from socket
                    "recv" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("recv() takes no arguments", span));
                        }
                        Ok(Type::Future(Box::new(Type::String)))
                    }
                    // send(content: String) -> Future<()> - async send to socket
                    "send" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("send() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        self.unify(&Type::String, &arg_ty, args[0].span)?;
                        Ok(Type::Future(Box::new(Type::Unit)))
                    }
                    // disconnect() -> Future<()> - async close socket
                    "disconnect" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("disconnect() takes no arguments", span));
                        }
                        Ok(Type::Future(Box::new(Type::Unit)))
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for AsyncSocket", method),
                        span,
                    )),
                }
            }
            // v0.84: ThreadPool methods
            Type::ThreadPool => {
                match method {
                    // execute(f: fn() -> ()) - execute a task on a worker thread
                    "execute" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("execute() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        // Check that argument is a function that takes no args and returns unit
                        match &arg_ty {
                            Type::Fn { params, ret } if params.is_empty() && **ret == Type::Unit => {}
                            _ => {
                                return Err(CompileError::type_error(
                                    format!("execute() expects fn() -> (), got {}", arg_ty),
                                    args[0].span,
                                ));
                            }
                        }
                        Ok(Type::Unit)
                    }
                    // join() - wait for all tasks to complete and shutdown
                    "join" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("join() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    // shutdown() - request shutdown (tasks may still be running)
                    "shutdown" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("shutdown() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for ThreadPool", method),
                        span,
                    )),
                }
            }
            // v0.85: Scope methods for scoped threads
            Type::Scope => {
                match method {
                    // spawn(f: fn() -> ()) - spawn a scoped thread
                    "spawn" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("spawn() takes exactly one argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        // Check that argument is a function that takes no args and returns unit
                        match &arg_ty {
                            Type::Fn { params, ret } if params.is_empty() && **ret == Type::Unit => {}
                            _ => {
                                return Err(CompileError::type_error(
                                    format!("spawn() expects fn() -> (), got {}", arg_ty),
                                    args[0].span,
                                ));
                            }
                        }
                        Ok(Type::Unit)
                    }
                    // wait() - wait for all spawned threads to complete
                    "wait" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("wait() takes no arguments", span));
                        }
                        Ok(Type::Unit)
                    }
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for Scope", method),
                        span,
                    )),
                }
            }
            // v0.20.1: For other types, look up trait methods
            _ => {
                if let Some((param_types, ret_type)) = self.lookup_trait_method(receiver_ty, method) {
                    // Check argument count (excluding self)
                    if args.len() != param_types.len() {
                        return Err(CompileError::type_error(
                            format!("method '{}' expects {} arguments, got {}", method, param_types.len(), args.len()),
                            span,
                        ));
                    }
                    // Check argument types
                    for (i, (arg, expected_ty)) in args.iter().zip(param_types.iter()).enumerate() {
                        let arg_ty = self.infer(&arg.node, arg.span)?;
                        self.unify(expected_ty, &arg_ty, args[i].span)?;
                    }
                    Ok(ret_type)
                } else {
                    Err(CompileError::type_error(
                        format!("type {} has no method '{}'", receiver_ty, method),
                        span,
                    ))
                }
            }
        }
    }

    /// v0.18: Check `Option<T>` method calls
    fn check_option_method(&mut self, method: &str, args: &[Spanned<Expr>], inner_ty: Option<Type>, span: Span) -> Result<Type> {
        match method {
            // is_some() -> bool
            "is_some" => {
                if !args.is_empty() {
                    return Err(CompileError::type_error("is_some() takes no arguments", span));
                }
                Ok(Type::Bool)
            }
            // is_none() -> bool
            "is_none" => {
                if !args.is_empty() {
                    return Err(CompileError::type_error("is_none() takes no arguments", span));
                }
                Ok(Type::Bool)
            }
            // unwrap_or(default: T) -> T
            "unwrap_or" => {
                if args.len() != 1 {
                    return Err(CompileError::type_error("unwrap_or() takes 1 argument", span));
                }
                let arg_ty = self.infer(&args[0].node, args[0].span)?;
                // If we know the inner type, check it matches
                if let Some(ref expected) = inner_ty {
                    self.unify(expected, &arg_ty, args[0].span)?;
                }
                // Return the concrete type: prefer arg_ty if inner_ty is a TypeVar
                match &inner_ty {
                    Some(Type::TypeVar(_)) => Ok(arg_ty),
                    Some(ty) => Ok(ty.clone()),
                    None => Ok(arg_ty),
                }
            }
            _ => Err(CompileError::type_error(
                format!("unknown method '{}' for Option", method),
                span,
            )),
        }
    }

    /// v0.18: Check Result<T, E> method calls
    fn check_result_method(&mut self, method: &str, args: &[Spanned<Expr>], ok_ty: Option<Type>, _err_ty: Option<Type>, span: Span) -> Result<Type> {
        match method {
            // is_ok() -> bool
            "is_ok" => {
                if !args.is_empty() {
                    return Err(CompileError::type_error("is_ok() takes no arguments", span));
                }
                Ok(Type::Bool)
            }
            // is_err() -> bool
            "is_err" => {
                if !args.is_empty() {
                    return Err(CompileError::type_error("is_err() takes no arguments", span));
                }
                Ok(Type::Bool)
            }
            // unwrap_or(default: T) -> T
            "unwrap_or" => {
                if args.len() != 1 {
                    return Err(CompileError::type_error("unwrap_or() takes 1 argument", span));
                }
                let arg_ty = self.infer(&args[0].node, args[0].span)?;
                // If we know the ok type, check it matches
                if let Some(ref expected) = ok_ty {
                    self.unify(expected, &arg_ty, args[0].span)?;
                }
                // Return the concrete type: prefer arg_ty if ok_ty is a TypeVar
                match &ok_ty {
                    Some(Type::TypeVar(_)) => Ok(arg_ty),
                    Some(ty) => Ok(ty.clone()),
                    None => Ok(arg_ty),
                }
            }
            _ => Err(CompileError::type_error(
                format!("unknown method '{}' for Result", method),
                span,
            )),
        }
    }

    /// v0.46: Check match exhaustiveness
    /// Returns exhaustiveness result with missing patterns and unreachable arms
    fn check_match_exhaustiveness(
        &self,
        match_ty: &Type,
        arms: &[MatchArm],
        _span: Span,
    ) -> Result<exhaustiveness::ExhaustivenessResult> {
        use exhaustiveness::{check_exhaustiveness, ExhaustivenessContext};

        // Build context with enum definitions
        let mut ctx = ExhaustivenessContext::new();

        // Add all known enums
        for (name, variants) in &self.enums {
            ctx.add_enum(name, variants.clone());
        }

        // Add generic enums with type parameters for substitution
        for (name, (type_params, variants)) in &self.generic_enums {
            ctx.add_enum(name, variants.clone());
            // v0.58: Also store type param names for substitution during exhaustiveness
            let param_names: Vec<String> = type_params.iter().map(|tp| tp.name.clone()).collect();
            ctx.add_generic_enum_params(name, param_names);
        }

        // v0.54: Add all known structs
        for (name, fields) in &self.structs {
            ctx.add_struct(name, fields.clone());
        }

        // v0.54: Add generic structs (instantiated with concrete types would need special handling)
        for (name, (_, fields)) in &self.generic_structs {
            ctx.add_struct(name, fields.clone());
        }

        // Convert arms to the format expected by exhaustiveness checker
        let arms_for_check: Vec<_> = arms
            .iter()
            .map(|arm| (arm.pattern.clone(), arm.guard.clone()))
            .collect();

        Ok(check_exhaustiveness(match_ty, &arms_for_check, &ctx))
    }

    /// v0.53: Check if an expression is divergent (never returns normally)
    /// This is used to detect unreachable code after return, break, continue
    fn is_divergent_expr(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Return { .. } | Expr::Break { .. } | Expr::Continue)
    }

    /// Check pattern validity
    fn check_pattern(&mut self, pattern: &crate::ast::Pattern, expected_ty: &Type, span: Span) -> Result<()> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Wildcard => Ok(()),
            Pattern::Var(name) => {
                // v0.79: Check for shadow binding before adding
                if let Some(original_span) = self.binding_tracker.find_shadow(name) {
                    self.add_warning(CompileWarning::shadow_binding(name, span, original_span));
                }

                // Bind the variable to the expected type
                self.env.insert(name.clone(), expected_ty.clone());
                // v0.48: Track binding for unused detection
                self.binding_tracker.bind(name.clone(), span);
                Ok(())
            }
            Pattern::Literal(lit) => {
                let lit_ty = match lit {
                    crate::ast::LiteralPattern::Int(_) => Type::I64,
                    crate::ast::LiteralPattern::Float(_) => Type::F64,
                    crate::ast::LiteralPattern::Bool(_) => Type::Bool,
                    crate::ast::LiteralPattern::String(_) => Type::String,
                };
                self.unify(expected_ty, &lit_ty, span)
            }
            Pattern::EnumVariant { enum_name, variant, bindings } => {
                // v0.75: Mark imported enum as used in pattern
                self.mark_name_used(enum_name);
                // Check that pattern matches expected type
                match expected_ty {
                    Type::Named(name) if name == enum_name => {
                        // Non-generic enum pattern matching
                        let variants = self.enums.get(enum_name).ok_or_else(|| {
                            // v0.63: Suggest similar type names
                            let mut all_types: Vec<&str> = self.enums.keys().map(|s| s.as_str()).collect();
                            all_types.extend(self.generic_enums.keys().map(|s| s.as_str()));
                            all_types.extend(self.structs.keys().map(|s| s.as_str()));
                            all_types.extend(self.generic_structs.keys().map(|s| s.as_str()));
                            let suggestion = find_similar_name(enum_name, &all_types, 2);
                            CompileError::type_error(
                                format!("undefined enum: `{}`{}", enum_name, format_suggestion_hint(suggestion)),
                                span,
                            )
                        })?;

                        let variant_fields = variants.iter()
                            .find(|(n, _)| n == variant)
                            .map(|(_, fields)| fields.clone())
                            .ok_or_else(|| {
                                // v0.60: Suggest similar variant names
                                let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
                                let suggestion = find_similar_name(variant, &names, 2);
                                CompileError::type_error(
                                    format!("unknown variant `{}` on enum `{}`{}", variant, enum_name, format_suggestion_hint(suggestion)),
                                    span,
                                )
                            })?;

                        if bindings.len() != variant_fields.len() {
                            // v0.59: Enhanced pattern binding error with hints
                            let suggestion = if bindings.len() > variant_fields.len() {
                                "\n  hint: remove extra bindings from pattern"
                            } else if variant_fields.len() == 1 {
                                "\n  hint: try using `_` as a wildcard binding"
                            } else {
                                "\n  hint: use `_` for unused bindings"
                            };
                            return Err(CompileError::type_error(
                                format!(
                                    "pattern `{}::{}` expects {} binding{}, got {}{}",
                                    enum_name, variant,
                                    variant_fields.len(),
                                    if variant_fields.len() == 1 { "" } else { "s" },
                                    bindings.len(),
                                    suggestion
                                ),
                                span,
                            ));
                        }

                        // v0.41: Recursively check nested patterns
                        for (binding, field_ty) in bindings.iter().zip(variant_fields.iter()) {
                            self.check_pattern(&binding.node, field_ty, binding.span)?;
                        }

                        Ok(())
                    }
                    // v0.16: Generic enum pattern matching (e.g., MyOption<i64>)
                    Type::Generic { name, type_args } if name == enum_name => {
                        let (type_params, variants) = self.generic_enums.get(enum_name).cloned().ok_or_else(|| {
                            CompileError::type_error(format!("undefined generic enum: {enum_name}"), span)
                        })?;

                        let variant_fields = variants.iter()
                            .find(|(n, _)| n == variant)
                            .map(|(_, fields)| fields.clone())
                            .ok_or_else(|| {
                                // v0.60: Suggest similar variant names
                                let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
                                let suggestion = find_similar_name(variant, &names, 2);
                                CompileError::type_error(
                                    format!("unknown variant `{}` on enum `{}`{}", variant, enum_name, format_suggestion_hint(suggestion)),
                                    span,
                                )
                            })?;

                        if bindings.len() != variant_fields.len() {
                            // v0.59: Enhanced pattern binding error with hints
                            let suggestion = if bindings.len() > variant_fields.len() {
                                "\n  hint: remove extra bindings from pattern"
                            } else if variant_fields.len() == 1 {
                                "\n  hint: try using `_` as a wildcard binding"
                            } else {
                                "\n  hint: use `_` for unused bindings"
                            };
                            return Err(CompileError::type_error(
                                format!(
                                    "pattern `{}::{}` expects {} binding{}, got {}{}",
                                    enum_name, variant,
                                    variant_fields.len(),
                                    if variant_fields.len() == 1 { "" } else { "s" },
                                    bindings.len(),
                                    suggestion
                                ),
                                span,
                            ));
                        }

                        // Build type substitution from type_params to type_args
                        let mut type_subst: HashMap<String, Type> = HashMap::new();
                        for (tp, arg) in type_params.iter().zip(type_args.iter()) {
                            type_subst.insert(tp.name.clone(), (**arg).clone());
                        }

                        // v0.41: Recursively check nested patterns with substituted types
                        let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();
                        for (binding, field_ty) in bindings.iter().zip(variant_fields.iter()) {
                            let resolved_ty = self.resolve_type_vars(field_ty, &type_param_names);
                            let substituted_ty = self.substitute_type(&resolved_ty, &type_subst);
                            self.check_pattern(&binding.node, &substituted_ty, binding.span)?;
                        }

                        Ok(())
                    }
                    // v0.16: TypeVar pattern matching (for generic function bodies)
                    Type::TypeVar(_) => {
                        // When matching in a generic context, allow any enum pattern
                        // and bind variables as TypeVar
                        if let Some((type_params, variants)) = self.generic_enums.get(enum_name).cloned() {
                            let variant_fields = variants.iter()
                                .find(|(n, _)| n == variant)
                                .map(|(_, fields)| fields.clone())
                                .ok_or_else(|| {
                                    // v0.60: Suggest similar variant names
                                    let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
                                    let suggestion = find_similar_name(variant, &names, 2);
                                    CompileError::type_error(
                                        format!("unknown variant `{}` on enum `{}`{}", variant, enum_name, format_suggestion_hint(suggestion)),
                                        span,
                                    )
                                })?;

                            if bindings.len() != variant_fields.len() {
                                // v0.59: Enhanced pattern binding error with hints
                                let suggestion = if bindings.len() > variant_fields.len() {
                                    "\n  hint: remove extra bindings from pattern"
                                } else if variant_fields.len() == 1 {
                                    "\n  hint: try using `_` as a wildcard binding"
                                } else {
                                    "\n  hint: use `_` for unused bindings"
                                };
                                return Err(CompileError::type_error(
                                    format!(
                                        "pattern `{}::{}` expects {} binding{}, got {}{}",
                                        enum_name, variant,
                                        variant_fields.len(),
                                        if variant_fields.len() == 1 { "" } else { "s" },
                                        bindings.len(),
                                        suggestion
                                    ),
                                    span,
                                ));
                            }

                            // v0.41: Recursively check nested patterns
                            let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();
                            for (binding, field_ty) in bindings.iter().zip(variant_fields.iter()) {
                                let resolved_ty = self.resolve_type_vars(field_ty, &type_param_names);
                                self.check_pattern(&binding.node, &resolved_ty, binding.span)?;
                            }

                            Ok(())
                        } else {
                            // v0.63: Suggest similar type names
                            let mut all_types: Vec<&str> = self.enums.keys().map(|s| s.as_str()).collect();
                            all_types.extend(self.generic_enums.keys().map(|s| s.as_str()));
                            all_types.extend(self.structs.keys().map(|s| s.as_str()));
                            all_types.extend(self.generic_structs.keys().map(|s| s.as_str()));
                            let suggestion = find_similar_name(enum_name, &all_types, 2);
                            Err(CompileError::type_error(
                                format!("undefined enum: `{}`{}", enum_name, format_suggestion_hint(suggestion)),
                                span,
                            ))
                        }
                    }
                    _ => Err(CompileError::type_error(
                        format!("expected {}, got enum pattern", expected_ty),
                        span,
                    )),
                }
            }
            Pattern::Struct { name, fields } => {
                // v0.75: Mark imported struct as used in pattern
                self.mark_name_used(name);
                match expected_ty {
                    Type::Named(expected_name) if expected_name == name => {
                        let struct_fields = self.structs.get(name).cloned().ok_or_else(|| {
                            // v0.63: Suggest similar type names
                            let mut all_types: Vec<&str> = self.structs.keys().map(|s| s.as_str()).collect();
                            all_types.extend(self.generic_structs.keys().map(|s| s.as_str()));
                            all_types.extend(self.enums.keys().map(|s| s.as_str()));
                            all_types.extend(self.generic_enums.keys().map(|s| s.as_str()));
                            let suggestion = find_similar_name(name, &all_types, 2);
                            CompileError::type_error(
                                format!("undefined struct: `{}`{}", name, format_suggestion_hint(suggestion)),
                                span,
                            )
                        })?;

                        for (field_name, field_pat) in fields {
                            let field_ty = struct_fields.iter()
                                .find(|(n, _)| n == &field_name.node)
                                .map(|(_, ty)| ty.clone())
                                .ok_or_else(|| {
                                    // v0.60: Suggest similar field names
                                    let names: Vec<&str> = struct_fields.iter().map(|(n, _)| n.as_str()).collect();
                                    let suggestion = find_similar_name(&field_name.node, &names, 2);
                                    CompileError::type_error(
                                        format!("unknown field `{}` on struct `{}`{}", field_name.node, name, format_suggestion_hint(suggestion)),
                                        span,
                                    )
                                })?;

                            self.check_pattern(&field_pat.node, &field_ty, field_pat.span)?;
                        }

                        Ok(())
                    }
                    _ => Err(CompileError::type_error(
                        format!("expected {}, got struct pattern", expected_ty),
                        span,
                    )),
                }
            }
            // v0.39: Range pattern
            Pattern::Range { start, end, inclusive: _ } => {
                // Check that expected type is numeric
                if !matches!(expected_ty.base_type(), Type::I32 | Type::I64 | Type::U32 | Type::U64) {
                    return Err(CompileError::type_error(
                        format!("range patterns only work with integer types, got {}", expected_ty),
                        span,
                    ));
                }
                // Check that start and end are the same type
                let start_ty = match start {
                    LiteralPattern::Int(_) => Type::I64,
                    _ => return Err(CompileError::type_error(
                        "range pattern bounds must be integers".to_string(),
                        span,
                    )),
                };
                let end_ty = match end {
                    LiteralPattern::Int(_) => Type::I64,
                    _ => return Err(CompileError::type_error(
                        "range pattern bounds must be integers".to_string(),
                        span,
                    )),
                };
                if start_ty != end_ty {
                    return Err(CompileError::type_error(
                        "range pattern bounds must have the same type".to_string(),
                        span,
                    ));
                }
                Ok(())
            }
            // v0.40: Or-pattern
            Pattern::Or(alts) => {
                // All alternatives must be compatible with the expected type
                for alt in alts {
                    self.check_pattern(&alt.node, expected_ty, alt.span)?;
                }
                Ok(())
            }
            // v0.41: Binding pattern: name @ pattern
            Pattern::Binding { name, pattern } => {
                // v0.79: Check for shadow binding before adding
                if let Some(original_span) = self.binding_tracker.find_shadow(name) {
                    self.add_warning(CompileWarning::shadow_binding(name, span, original_span));
                }

                // Bind the name to the expected type
                self.env.insert(name.clone(), expected_ty.clone());
                // v0.48: Track binding for unused detection
                self.binding_tracker.bind(name.clone(), span);
                // Check the inner pattern
                self.check_pattern(&pattern.node, expected_ty, pattern.span)
            }
            // v0.42: Tuple pattern
            Pattern::Tuple(patterns) => {
                // Expected type must be a tuple with matching arity
                if let Type::Tuple(elem_types) = expected_ty {
                    if patterns.len() != elem_types.len() {
                        return Err(CompileError::type_error(
                            format!(
                                "tuple pattern has {} elements but expected {}",
                                patterns.len(),
                                elem_types.len()
                            ),
                            span,
                        ));
                    }
                    // Check each element pattern against its corresponding type
                    for (pat, elem_ty) in patterns.iter().zip(elem_types.iter()) {
                        self.check_pattern(&pat.node, elem_ty, pat.span)?;
                    }
                    Ok(())
                } else {
                    Err(CompileError::type_error(
                        format!("expected tuple type, got {}", expected_ty),
                        span,
                    ))
                }
            }
            // v0.44: Array pattern
            Pattern::Array(patterns) => {
                // Expected type must be an array with matching size
                if let Type::Array(elem_ty, size) = expected_ty {
                    if patterns.len() != *size {
                        return Err(CompileError::type_error(
                            format!(
                                "array pattern has {} elements but expected {} (array size)",
                                patterns.len(),
                                size
                            ),
                            span,
                        ));
                    }
                    // Check each element pattern against the element type
                    for pat in patterns.iter() {
                        self.check_pattern(&pat.node, elem_ty, pat.span)?;
                    }
                    Ok(())
                } else {
                    Err(CompileError::type_error(
                        format!("expected array type, got {}", expected_ty),
                        span,
                    ))
                }
            }
            // v0.45: Array rest pattern - matches arrays with prefix..suffix
            Pattern::ArrayRest { prefix, suffix } => {
                if let Type::Array(elem_ty, size) = expected_ty {
                    let required_len = prefix.len() + suffix.len();
                    // Array must have at least enough elements for prefix + suffix
                    if *size < required_len {
                        return Err(CompileError::type_error(
                            format!(
                                "array rest pattern requires at least {} elements but array has only {}",
                                required_len,
                                size
                            ),
                            span,
                        ));
                    }
                    // Check prefix patterns against the element type
                    for pat in prefix.iter() {
                        self.check_pattern(&pat.node, elem_ty, pat.span)?;
                    }
                    // Check suffix patterns against the element type
                    for pat in suffix.iter() {
                        self.check_pattern(&pat.node, elem_ty, pat.span)?;
                    }
                    Ok(())
                } else {
                    Err(CompileError::type_error(
                        format!("expected array type for array rest pattern, got {}", expected_ty),
                        span,
                    ))
                }
            }
        }
    }

    /// Check binary operation types
    /// v0.2: Uses base_type() to handle refined types correctly
    fn check_binary_op(&self, op: BinOp, left: &Type, right: &Type, span: Span) -> Result<Type> {
        // v0.50.6: Resolve type aliases before checking
        let left_resolved = self.resolve_type_alias(left);
        let right_resolved = self.resolve_type_alias(right);
        // v0.2: Extract base types for refined types
        let left_base = left_resolved.base_type();
        let right_base = right_resolved.base_type();

        match op {
            BinOp::Add => {
                // v0.60.19: Support pointer arithmetic: ptr + i64 or i64 + ptr
                match (left_base, right_base) {
                    // Pointer arithmetic: *T + integer = *T
                    (Type::Ptr(elem_ty), Type::I64 | Type::I32 | Type::U64 | Type::U32) => {
                        Ok(Type::Ptr(elem_ty.clone()))
                    }
                    // Commutative: integer + *T = *T
                    (Type::I64 | Type::I32 | Type::U64 | Type::U32, Type::Ptr(elem_ty)) => {
                        Ok(Type::Ptr(elem_ty.clone()))
                    }
                    // Standard numeric and string cases
                    _ => {
                        self.unify(left_base, right_base, span)?;
                        match left_base {
                            // v0.38: Include unsigned types
                            Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 => Ok(left_base.clone()),
                            Type::String => Ok(Type::String), // String concatenation
                            _ => Err(CompileError::type_error(
                                format!("+ operator requires numeric, String, or pointer type, got {left}"),
                                span,
                            )),
                        }
                    }
                }
            }

            BinOp::Sub => {
                // v0.60.19: Support pointer arithmetic: ptr - integer = *T
                match (left_base, right_base) {
                    // Pointer arithmetic: *T - integer = *T
                    (Type::Ptr(elem_ty), Type::I64 | Type::I32 | Type::U64 | Type::U32) => {
                        Ok(Type::Ptr(elem_ty.clone()))
                    }
                    // Standard numeric case
                    _ => {
                        self.unify(left_base, right_base, span)?;
                        match left_base {
                            // v0.38: Include unsigned types
                            Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 => Ok(left_base.clone()),
                            _ => Err(CompileError::type_error(
                                format!("- operator requires numeric or pointer type, got {left}"),
                                span,
                            )),
                        }
                    }
                }
            }

            BinOp::Mul | BinOp::Div | BinOp::Mod => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    // v0.38: Include unsigned types
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 => Ok(left_base.clone()),
                    _ => Err(CompileError::type_error(
                        format!("arithmetic operator requires numeric type, got {left}"),
                        span,
                    )),
                }
            }

            // v0.37: Wrapping arithmetic operators (integer only, no floats)
            BinOp::AddWrap | BinOp::SubWrap | BinOp::MulWrap => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    // v0.38: Include unsigned types
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 => Ok(left_base.clone()),
                    _ => Err(CompileError::type_error(
                        format!("wrapping arithmetic operator requires integer type, got {left}"),
                        span,
                    )),
                }
            }

            // v0.38: Checked arithmetic operators (return Option<T>)
            BinOp::AddChecked | BinOp::SubChecked | BinOp::MulChecked => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 => {
                        // Return Option<T> where T is the integer type
                        Ok(Type::Generic {
                            name: "Option".to_string(),
                            type_args: vec![Box::new(left_base.clone())],
                        })
                    }
                    _ => Err(CompileError::type_error(
                        format!("checked arithmetic operator requires integer type, got {left}"),
                        span,
                    )),
                }
            }

            // v0.38: Saturating arithmetic operators (clamp to min/max)
            BinOp::AddSat | BinOp::SubSat | BinOp::MulSat => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 => Ok(left_base.clone()),
                    _ => Err(CompileError::type_error(
                        format!("saturating arithmetic operator requires integer type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Eq | BinOp::Ne => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    // v0.38: Include unsigned types, v0.64: Include Char type
                    // v0.51.37: Include pointer types for null checks
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 | Type::Bool | Type::String | Type::Char | Type::Ptr(_) => Ok(Type::Bool),
                    _ => Err(CompileError::type_error(
                        format!("equality operator requires comparable type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    // v0.38: Include unsigned types, v0.64: Include Char type (ordinal comparison)
                    Type::I32 | Type::I64 | Type::U32 | Type::U64 | Type::F64 | Type::Char => Ok(Type::Bool),
                    _ => Err(CompileError::type_error(
                        format!("comparison operator requires numeric type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::And | BinOp::Or => {
                self.unify(&Type::Bool, left_base, span)?;
                self.unify(&Type::Bool, right_base, span)?;
                Ok(Type::Bool)
            }

            // v0.32: Shift operators require integer types
            // v0.38: Include unsigned types (preserve signedness)
            BinOp::Shl | BinOp::Shr => {
                match (left_base, right_base) {
                    (Type::I32, Type::I32) => Ok(Type::I32),
                    (Type::I64, Type::I64) | (Type::I64, Type::I32) | (Type::I32, Type::I64) => Ok(Type::I64),
                    (Type::U32, Type::U32) | (Type::U32, Type::I32) => Ok(Type::U32),
                    (Type::U64, Type::U64) | (Type::U64, Type::I32) | (Type::U64, Type::U32) => Ok(Type::U64),
                    _ => Err(CompileError::type_error(
                        format!("shift operators require integer types, got {left_base} and {right_base}"),
                        span,
                    )),
                }
            }

            // v0.36: Bitwise operators require integer types
            // v0.38: Include unsigned types (preserve signedness)
            BinOp::Band | BinOp::Bor | BinOp::Bxor => {
                match (left_base, right_base) {
                    (Type::I32, Type::I32) => Ok(Type::I32),
                    (Type::I64, Type::I64) | (Type::I64, Type::I32) | (Type::I32, Type::I64) => Ok(Type::I64),
                    (Type::U32, Type::U32) => Ok(Type::U32),
                    (Type::U64, Type::U64) | (Type::U64, Type::U32) | (Type::U32, Type::U64) => Ok(Type::U64),
                    _ => Err(CompileError::type_error(
                        format!("bitwise operators require integer types, got {left_base} and {right_base}"),
                        span,
                    )),
                }
            }

            // v0.36: Logical implication requires boolean types
            BinOp::Implies => {
                self.unify(&Type::Bool, left_base, span)?;
                self.unify(&Type::Bool, right_base, span)?;
                Ok(Type::Bool)
            }
        }
    }

    /// Check unary operation types
    /// v0.2: Uses base_type() to handle refined types correctly
    fn check_unary_op(&self, op: UnOp, ty: &Type, span: Span) -> Result<Type> {
        // v0.2: Extract base type for refined types
        let ty_base = ty.base_type();

        match op {
            UnOp::Neg => match ty_base {
                Type::I32 | Type::I64 | Type::F64 => Ok(ty_base.clone()),
                _ => Err(CompileError::type_error(
                    format!("negation requires numeric type, got {ty}"),
                    span,
                )),
            },
            UnOp::Not => {
                self.unify(&Type::Bool, ty_base, span)?;
                Ok(Type::Bool)
            }
            // v0.36: Bitwise not requires integer type
            // v0.38: Include unsigned types
            UnOp::Bnot => match ty_base {
                Type::I32 | Type::I64 | Type::U32 | Type::U64 => Ok(ty_base.clone()),
                _ => Err(CompileError::type_error(
                    format!("bitwise not requires integer type, got {ty}"),
                    span,
                )),
            },
        }
    }

    /// Unify two types
    /// v0.15: Updated to handle TypeVar in generic function body checking
    fn unify(&self, expected: &Type, actual: &Type, span: Span) -> Result<()> {
        // v0.50.6: Resolve type aliases before unification
        let expected = self.resolve_type_alias(expected);
        let actual = self.resolve_type_alias(actual);

        // v0.15: TypeVar in function body context matches any type
        // When type checking a generic function body, TypeVar acts as a placeholder
        if let Type::TypeVar(name) = &expected
            && self.type_param_env.contains_key(name)
        {
            // TypeVar is bound in current generic context - accept any type
            return Ok(());
        }
        if let Type::TypeVar(name) = &actual
            && self.type_param_env.contains_key(name)
        {
            // TypeVar is bound in current generic context - accept any type
            return Ok(());
        }

        // Both are TypeVar with same name
        if let (Type::TypeVar(a), Type::TypeVar(b)) = (&expected, &actual)
            && a == b
        {
            return Ok(());
        }

        // v0.16: Handle Generic types with TypeVar in type_args
        // e.g., unify Option<i64> with Option<T> where T is a type parameter
        if let (Type::Generic { name: n1, type_args: a1 }, Type::Generic { name: n2, type_args: a2 }) = (&expected, &actual)
            && n1 == n2
            && a1.len() == a2.len()
        {
            // Same generic name and same number of args - unify each arg
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                self.unify(arg1, arg2, span)?;
            }
            return Ok(());
        }

        // v0.16: Handle unbound TypeVar (from nullary variants like Option::None)
        // In non-generic context, TypeVar acts as a wildcard that matches concrete types
        if let Type::TypeVar(_) = &expected {
            // Allow any type to match an unbound TypeVar
            return Ok(());
        }
        if let Type::TypeVar(_) = &actual {
            // Allow unbound TypeVar to match any expected type
            return Ok(());
        }

        // v0.51.40: Handle Ptr types - recursively unify inner types
        // This allows null (*_null) to match any pointer type (*T)
        if let (Type::Ptr(inner1), Type::Ptr(inner2)) = (&expected, &actual) {
            return self.unify(inner1, inner2, span);
        }

        // v0.89.17: Handle Nullable types
        // null (Ptr(TypeVar("_null"))) is compatible with Nullable(T)
        if let Type::Nullable(_) = &expected
            && let Type::Ptr(inner) = &actual
            && let Type::TypeVar(name) = inner.as_ref()
            && name == "_null"
        {
            return Ok(());
        }
        // T is compatible with Nullable(T) — auto-wrap value into nullable
        if let Type::Nullable(inner) = &expected
            && self.unify(inner, &actual, span).is_ok()
        {
            return Ok(());
        }
        // Nullable(T) with Nullable(U) — recursively unify inner types
        if let (Type::Nullable(inner1), Type::Nullable(inner2)) = (&expected, &actual) {
            return self.unify(inner1, inner2, span);
        }

        if expected == actual {
            Ok(())
        } else {
            // v0.38: Allow integer type coercion for literals
            // i64 literals (default) can be used where u32/u64 is expected
            // This enables: let x: u32 = 10; (where 10 infers as i64)
            let is_integer_coercion = matches!(
                (&expected, &actual),
                (Type::U32, Type::I64) | (Type::U64, Type::I64)
                | (Type::I32, Type::I64) | (Type::U32, Type::I32)
            );
            if is_integer_coercion {
                Ok(())
            } else {
                Err(CompileError::type_error(
                    format!("expected {expected}, got {actual}"),
                    span,
                ))
            }
        }
    }

    /// v0.15: Infer type arguments by matching parameter types with argument types
    /// Populates type_subst with inferred type parameter -> concrete type mappings
    fn infer_type_args(
        &self,
        param_ty: &Type,
        arg_ty: &Type,
        type_subst: &mut HashMap<String, Type>,
        span: Span,
    ) -> Result<()> {
        match param_ty {
            Type::TypeVar(name) => {
                // Found a type variable - infer its concrete type from the argument
                if let Some(existing) = type_subst.get(name) {
                    // Already inferred - check consistency
                    if existing != arg_ty {
                        return Err(CompileError::type_error(
                            format!(
                                "conflicting type inference for {}: {} vs {}",
                                name, existing, arg_ty
                            ),
                            span,
                        ));
                    }
                } else {
                    type_subst.insert(name.clone(), arg_ty.clone());
                }
                Ok(())
            }
            Type::Ref(inner) => {
                if let Type::Ref(arg_inner) = arg_ty {
                    self.infer_type_args(inner, arg_inner, type_subst, span)
                } else {
                    Err(CompileError::type_error(
                        format!("expected reference type, got {}", arg_ty),
                        span,
                    ))
                }
            }
            Type::RefMut(inner) => {
                if let Type::RefMut(arg_inner) = arg_ty {
                    self.infer_type_args(inner, arg_inner, type_subst, span)
                } else {
                    Err(CompileError::type_error(
                        format!("expected mutable reference type, got {}", arg_ty),
                        span,
                    ))
                }
            }
            // v0.60.27: Support generic pointer types like *T
            Type::Ptr(inner) => {
                if let Type::Ptr(arg_inner) = arg_ty {
                    self.infer_type_args(inner, arg_inner, type_subst, span)
                } else {
                    Err(CompileError::type_error(
                        format!("expected pointer type, got {}", arg_ty),
                        span,
                    ))
                }
            }
            Type::Array(elem, size) => {
                if let Type::Array(arg_elem, arg_size) = arg_ty {
                    if size != arg_size {
                        return Err(CompileError::type_error(
                            format!("array size mismatch: expected {}, got {}", size, arg_size),
                            span,
                        ));
                    }
                    self.infer_type_args(elem, arg_elem, type_subst, span)
                } else {
                    Err(CompileError::type_error(
                        format!("expected array type, got {}", arg_ty),
                        span,
                    ))
                }
            }
            Type::Generic { name, type_args } => {
                if let Type::Generic { name: arg_name, type_args: arg_type_args } = arg_ty {
                    if name != arg_name {
                        return Err(CompileError::type_error(
                            format!("generic type mismatch: expected {}, got {}", name, arg_name),
                            span,
                        ));
                    }
                    if type_args.len() != arg_type_args.len() {
                        return Err(CompileError::type_error(
                            "generic type argument count mismatch".to_string(),
                            span,
                        ));
                    }
                    for (param_arg, actual_arg) in type_args.iter().zip(arg_type_args.iter()) {
                        self.infer_type_args(param_arg, actual_arg, type_subst, span)?;
                    }
                    Ok(())
                } else {
                    Err(CompileError::type_error(
                        format!("expected generic type {}, got {}", name, arg_ty),
                        span,
                    ))
                }
            }
            // v0.20.0: Fn type
            Type::Fn { params, ret } => {
                if let Type::Fn { params: arg_params, ret: arg_ret } = arg_ty {
                    if params.len() != arg_params.len() {
                        return Err(CompileError::type_error(
                            format!("function parameter count mismatch: expected {}, got {}", params.len(), arg_params.len()),
                            span,
                        ));
                    }
                    for (p, ap) in params.iter().zip(arg_params.iter()) {
                        self.infer_type_args(p, ap, type_subst, span)?;
                    }
                    self.infer_type_args(ret, arg_ret, type_subst, span)
                } else {
                    Err(CompileError::type_error(
                        format!("expected function type, got {}", arg_ty),
                        span,
                    ))
                }
            }
            // v0.78: Future<T> type
            Type::Future(inner) => {
                if let Type::Future(arg_inner) = arg_ty {
                    self.infer_type_args(inner, arg_inner, type_subst, span)
                } else {
                    Err(CompileError::type_error(
                        format!("expected Future type, got {}", arg_ty),
                        span,
                    ))
                }
            }
            // For concrete types, just check equality
            _ => {
                if param_ty == arg_ty {
                    Ok(())
                } else {
                    Err(CompileError::type_error(
                        format!("type mismatch: expected {}, got {}", param_ty, arg_ty),
                        span,
                    ))
                }
            }
        }
    }

    /// v0.15: Convert Named types to TypeVar when they match type parameters
    /// This is needed because the parser treats type parameter references as Named types
    fn resolve_type_vars(&self, ty: &Type, type_param_names: &[&str]) -> Type {
        match ty {
            Type::Named(name) => {
                if type_param_names.contains(&name.as_str()) {
                    Type::TypeVar(name.clone())
                } else {
                    ty.clone()
                }
            }
            Type::Ref(inner) => {
                Type::Ref(Box::new(self.resolve_type_vars(inner, type_param_names)))
            }
            Type::RefMut(inner) => {
                Type::RefMut(Box::new(self.resolve_type_vars(inner, type_param_names)))
            }
            Type::Array(elem, size) => {
                Type::Array(Box::new(self.resolve_type_vars(elem, type_param_names)), *size)
            }
            Type::Range(elem) => {
                Type::Range(Box::new(self.resolve_type_vars(elem, type_param_names)))
            }
            Type::Generic { name, type_args } => {
                let resolved_args: Vec<_> = type_args
                    .iter()
                    .map(|arg| Box::new(self.resolve_type_vars(arg, type_param_names)))
                    .collect();
                Type::Generic {
                    name: name.clone(),
                    type_args: resolved_args,
                }
            }
            Type::Refined { base, constraints } => {
                Type::Refined {
                    base: Box::new(self.resolve_type_vars(base, type_param_names)),
                    constraints: constraints.clone(),
                }
            }
            // v0.20.0: Fn type
            Type::Fn { params, ret } => {
                Type::Fn {
                    params: params.iter()
                        .map(|p| Box::new(self.resolve_type_vars(p, type_param_names)))
                        .collect(),
                    ret: Box::new(self.resolve_type_vars(ret, type_param_names)),
                }
            }
            // Other types remain unchanged
            _ => ty.clone(),
        }
    }

    /// v0.15: Substitute type variables with concrete types
    fn substitute_type(&self, ty: &Type, type_subst: &HashMap<String, Type>) -> Type {
        match ty {
            Type::TypeVar(name) => {
                type_subst.get(name).cloned().unwrap_or_else(|| ty.clone())
            }
            Type::Ref(inner) => {
                Type::Ref(Box::new(self.substitute_type(inner, type_subst)))
            }
            Type::RefMut(inner) => {
                Type::RefMut(Box::new(self.substitute_type(inner, type_subst)))
            }
            Type::Array(elem, size) => {
                Type::Array(Box::new(self.substitute_type(elem, type_subst)), *size)
            }
            Type::Range(elem) => {
                Type::Range(Box::new(self.substitute_type(elem, type_subst)))
            }
            Type::Generic { name, type_args } => {
                let substituted_args: Vec<_> = type_args
                    .iter()
                    .map(|arg| Box::new(self.substitute_type(arg, type_subst)))
                    .collect();
                Type::Generic {
                    name: name.clone(),
                    type_args: substituted_args,
                }
            }
            Type::Refined { base, constraints } => {
                Type::Refined {
                    base: Box::new(self.substitute_type(base, type_subst)),
                    constraints: constraints.clone(),
                }
            }
            // v0.20.0: Fn type
            Type::Fn { params, ret } => {
                Type::Fn {
                    params: params.iter()
                        .map(|p| Box::new(self.substitute_type(p, type_subst)))
                        .collect(),
                    ret: Box::new(self.substitute_type(ret, type_subst)),
                }
            }
            // Concrete types remain unchanged
            _ => ty.clone(),
        }
    }

    /// v0.20.1: Convert Type to string key for impls HashMap lookup
    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            // v0.38: Unsigned types
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
            // v0.64: Char type
            Type::Char => "char".to_string(),
            Type::Unit => "unit".to_string(),
            Type::Named(name) => name.clone(),
            Type::TypeVar(name) => name.clone(),
            Type::Generic { name, type_args } => {
                let args: Vec<String> = type_args.iter()
                    .map(|arg| self.type_to_string(arg))
                    .collect();
                format!("{}<{}>", name, args.join(", "))
            }
            Type::Struct { name, .. } => name.clone(),
            Type::Enum { name, .. } => name.clone(),
            Type::Ref(inner) => format!("&{}", self.type_to_string(inner)),
            Type::RefMut(inner) => format!("&mut {}", self.type_to_string(inner)),
            Type::Array(elem, size) => format!("[{}; {}]", self.type_to_string(elem), size),
            Type::Range(elem) => format!("Range<{}>", self.type_to_string(elem)),
            Type::Refined { base, .. } => self.type_to_string(base),
            Type::Fn { params, ret } => {
                let param_strs: Vec<String> = params.iter()
                    .map(|p| self.type_to_string(p))
                    .collect();
                format!("Fn({}) -> {}", param_strs.join(", "), self.type_to_string(ret))
            }
            // v0.31: Never type
            Type::Never => "!".to_string(),
            // v0.37: Nullable type
            Type::Nullable(inner) => format!("{}?", self.type_to_string(inner)),
            // v0.42: Tuple type
            Type::Tuple(elems) => {
                let elems_str: Vec<_> = elems.iter().map(|t| self.type_to_string(t)).collect();
                format!("({})", elems_str.join(", "))
            }
            // v0.51.37: Pointer type
            Type::Ptr(inner) => format!("*{}", self.type_to_string(inner)),
            // v0.70: Thread type
            Type::Thread(inner) => format!("Thread<{}>", self.type_to_string(inner)),
            // v0.71: Mutex type
            Type::Mutex(inner) => format!("Mutex<{}>", self.type_to_string(inner)),
            // v0.72: Arc and Atomic types
            Type::Arc(inner) => format!("Arc<{}>", self.type_to_string(inner)),
            Type::Atomic(inner) => format!("Atomic<{}>", self.type_to_string(inner)),
            // v0.73: Sender and Receiver types
            Type::Sender(inner) => format!("Sender<{}>", self.type_to_string(inner)),
            Type::Receiver(inner) => format!("Receiver<{}>", self.type_to_string(inner)),
            // v0.74: RwLock, Barrier, Condvar
            Type::RwLock(inner) => format!("RwLock<{}>", self.type_to_string(inner)),
            Type::Barrier => "Barrier".to_string(),
            Type::Condvar => "Condvar".to_string(),
            // v0.75: Future type
            Type::Future(inner) => format!("Future<{}>", self.type_to_string(inner)),
            // v0.83: AsyncFile type
            Type::AsyncFile => "AsyncFile".to_string(),
            // v0.83.1: AsyncSocket type
            Type::AsyncSocket => "AsyncSocket".to_string(),
            // v0.84: ThreadPool type
            Type::ThreadPool => "ThreadPool".to_string(),
            // v0.85: Scope type
            Type::Scope => "Scope".to_string(),
        }
    }

    /// v0.20.1: Substitute Self type with target type in trait method signatures
    fn substitute_self(&self, ty: &Type, target_type: &Type) -> Type {
        match ty {
            // Named("Self") is replaced with target type
            Type::Named(name) if name == "Self" => target_type.clone(),
            // Recursively substitute in compound types
            Type::Ref(inner) => {
                Type::Ref(Box::new(self.substitute_self(inner, target_type)))
            }
            Type::RefMut(inner) => {
                Type::RefMut(Box::new(self.substitute_self(inner, target_type)))
            }
            Type::Array(elem, size) => {
                Type::Array(Box::new(self.substitute_self(elem, target_type)), *size)
            }
            Type::Range(elem) => {
                Type::Range(Box::new(self.substitute_self(elem, target_type)))
            }
            Type::Generic { name, type_args } => {
                let substituted_args: Vec<_> = type_args.iter()
                    .map(|arg| Box::new(self.substitute_self(arg, target_type)))
                    .collect();
                Type::Generic {
                    name: name.clone(),
                    type_args: substituted_args,
                }
            }
            Type::Refined { base, constraints } => {
                Type::Refined {
                    base: Box::new(self.substitute_self(base, target_type)),
                    constraints: constraints.clone(),
                }
            }
            Type::Fn { params, ret } => {
                Type::Fn {
                    params: params.iter()
                        .map(|p| Box::new(self.substitute_self(p, target_type)))
                        .collect(),
                    ret: Box::new(self.substitute_self(ret, target_type)),
                }
            }
            // Other types remain unchanged
            _ => ty.clone(),
        }
    }

    /// v0.20.1: Look up trait method for a given receiver type
    fn lookup_trait_method(&self, receiver_ty: &Type, method: &str) -> Option<(Vec<Type>, Type)> {
        let type_name = self.type_to_string(receiver_ty);

        // Search all impls for this type to find the method
        for ((impl_type, _trait_name), impl_info) in &self.impls {
            if impl_type == &type_name
                && let Some((param_types, ret_type)) = impl_info.methods.get(method)
            {
                return Some((param_types.clone(), ret_type.clone()));
            }
        }
        None
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// v0.89: Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // Levenshtein Distance Tests (v0.90.43: delegated to crate::util)
    // ====================================================================

    use crate::util::levenshtein_distance;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_empty_strings() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "xyz"), 3);
    }

    #[test]
    fn test_levenshtein_single_edit() {
        // Substitution
        assert_eq!(levenshtein_distance("cat", "bat"), 1);
        // Insertion
        assert_eq!(levenshtein_distance("cat", "cats"), 1);
        // Deletion
        assert_eq!(levenshtein_distance("cats", "cat"), 1);
    }

    #[test]
    fn test_levenshtein_multiple_edits() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("sunday", "saturday"), 3);
    }

    #[test]
    fn test_levenshtein_completely_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
    }

    // ====================================================================
    // find_similar_name Tests
    // ====================================================================

    #[test]
    fn test_find_similar_name_exact_match() {
        let candidates = vec!["foo", "bar", "baz"];
        // Distance 0 is within any threshold
        assert_eq!(find_similar_name("foo", &candidates, 2), Some("foo"));
    }

    #[test]
    fn test_find_similar_name_typo() {
        let candidates = vec!["println", "print", "parse"];
        assert_eq!(find_similar_name("printn", &candidates, 2), Some("println"));
    }

    #[test]
    fn test_find_similar_name_no_match() {
        let candidates = vec!["foo", "bar", "baz"];
        assert_eq!(find_similar_name("xyz", &candidates, 1), None);
    }

    #[test]
    fn test_find_similar_name_empty_candidates() {
        let candidates: Vec<&str> = vec![];
        assert_eq!(find_similar_name("foo", &candidates, 2), None);
    }

    #[test]
    fn test_find_similar_name_picks_closest() {
        let candidates = vec!["read_file", "read_line", "readline"];
        // "read_lin" is distance 1 from "read_line", distance 2 from "readline"
        assert_eq!(find_similar_name("read_lin", &candidates, 2), Some("read_line"));
    }

    // ====================================================================
    // format_suggestion_hint Tests
    // ====================================================================

    #[test]
    fn test_format_suggestion_hint_some() {
        let hint = format_suggestion_hint(Some("println"));
        assert!(hint.contains("did you mean"));
        assert!(hint.contains("println"));
    }

    #[test]
    fn test_format_suggestion_hint_none() {
        let hint = format_suggestion_hint(None);
        assert!(hint.is_empty());
    }

    // ====================================================================
    // BindingTracker Tests
    // ====================================================================

    #[test]
    fn test_binding_tracker_basic_bind_and_use() {
        let mut tracker = BindingTracker::new();
        let span = Span { start: 0, end: 1 };
        tracker.bind("x".to_string(), span);
        tracker.mark_used("x");
        let (unused, _) = tracker.pop_scope();
        assert!(unused.is_empty(), "Used binding should not appear in unused list");
    }

    #[test]
    fn test_binding_tracker_unused_detection() {
        let mut tracker = BindingTracker::new();
        let span = Span { start: 0, end: 1 };
        tracker.bind("x".to_string(), span);
        // Don't mark as used
        let (unused, _) = tracker.pop_scope();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].0, "x");
    }

    #[test]
    fn test_binding_tracker_underscore_prefix_ignored() {
        let mut tracker = BindingTracker::new();
        let span = Span { start: 0, end: 1 };
        tracker.bind("_unused".to_string(), span);
        let (unused, _) = tracker.pop_scope();
        assert!(unused.is_empty(), "Underscore-prefixed bindings should be ignored");
    }

    #[test]
    fn test_binding_tracker_nested_scopes() {
        let mut tracker = BindingTracker::new();
        let span = Span { start: 0, end: 1 };

        // Outer scope
        tracker.bind("x".to_string(), span);

        // Inner scope
        tracker.push_scope();
        tracker.bind("y".to_string(), span);
        tracker.mark_used("y");
        tracker.mark_used("x"); // Use outer variable from inner scope
        let (inner_unused, _) = tracker.pop_scope();
        assert!(inner_unused.is_empty());

        // Outer scope: x was used from inner scope
        let (outer_unused, _) = tracker.pop_scope();
        assert!(outer_unused.is_empty(), "x was used from inner scope");
    }

    #[test]
    fn test_binding_tracker_mutable_not_mutated() {
        let mut tracker = BindingTracker::new();
        let span = Span { start: 0, end: 1 };
        tracker.bind_with_mutability("x".to_string(), span, true);
        tracker.mark_used("x");
        // Used but never mutated
        let (unused, unused_mut) = tracker.pop_scope();
        assert!(unused.is_empty());
        assert_eq!(unused_mut.len(), 1, "Mutable but never mutated should be detected");
        assert_eq!(unused_mut[0].0, "x");
    }

    #[test]
    fn test_binding_tracker_mutable_and_mutated() {
        let mut tracker = BindingTracker::new();
        let span = Span { start: 0, end: 1 };
        tracker.bind_with_mutability("x".to_string(), span, true);
        tracker.mark_used("x");
        tracker.mark_mutated("x");
        let (_, unused_mut) = tracker.pop_scope();
        assert!(unused_mut.is_empty(), "Mutated mutable should not appear in unused_mut");
    }

    #[test]
    fn test_binding_tracker_shadow_detection() {
        let mut tracker = BindingTracker::new();
        let outer_span = Span { start: 0, end: 5 };
        let inner_span = Span { start: 10, end: 15 };

        tracker.bind("x".to_string(), outer_span);
        tracker.push_scope();

        // Before shadowing, find_shadow should find the outer binding
        let shadow = tracker.find_shadow("x");
        assert!(shadow.is_some(), "Should find outer binding");
        assert_eq!(shadow.unwrap().start, 0);

        tracker.bind("x".to_string(), inner_span); // Shadow
        // Now find_shadow in a deeper scope would find the outer
        tracker.push_scope();
        let shadow2 = tracker.find_shadow("x");
        assert!(shadow2.is_some(), "Should find shadowed binding");
    }

    #[test]
    fn test_binding_tracker_underscore_shadow_ignored() {
        let mut tracker = BindingTracker::new();
        let span = Span { start: 0, end: 1 };
        tracker.bind("_x".to_string(), span);
        tracker.push_scope();
        // Underscore-prefixed should be ignored
        let shadow = tracker.find_shadow("_x");
        assert!(shadow.is_none(), "Underscore-prefixed should not trigger shadow warning");
    }

    // ====================================================================
    // TypeChecker Integration Tests (via check_program)
    // ====================================================================

    /// Helper to parse and type-check a source string
    fn check(source: &str) -> Result<TypeChecker> {
        let tokens = crate::lexer::tokenize(source).expect("tokenize failed");
        let ast = crate::parser::parse("test.bmb", source, tokens).expect("parse failed");
        let mut tc = TypeChecker::new();
        tc.check_program(&ast)?;
        Ok(tc)
    }

    /// Helper to check that source type-checks successfully
    fn ok(source: &str) -> bool {
        check(source).is_ok()
    }

    /// Helper to check that source fails type-checking
    fn err(source: &str) -> bool {
        check(source).is_err()
    }

    #[test]
    fn test_tc_basic_int_function() {
        assert!(ok("fn add(a: i64, b: i64) -> i64 = a + b;"));
    }

    #[test]
    fn test_tc_type_mismatch() {
        // Returning bool from i64 function
        assert!(err("fn bad() -> i64 = true;"));
    }

    #[test]
    fn test_tc_undefined_variable() {
        assert!(err("fn bad() -> i64 = x;"));
    }

    #[test]
    fn test_tc_undefined_function() {
        assert!(err("fn bad() -> i64 = nonexistent(1);"));
    }

    #[test]
    fn test_tc_recursive_function() {
        assert!(ok(
            "fn factorial(n: i64) -> i64 = if n <= 1 { 1 } else { n * factorial(n - 1) };"
        ));
    }

    #[test]
    fn test_tc_struct_definition() {
        assert!(ok(
            "struct Point { x: i64, y: i64 }
             fn origin() -> Point = new Point { x: 0, y: 0 };"
        ));
    }

    #[test]
    fn test_tc_struct_field_access() {
        assert!(ok(
            "struct Point { x: i64, y: i64 }
             fn get_x(p: Point) -> i64 = p.x;"
        ));
    }

    #[test]
    fn test_tc_enum_variant() {
        assert!(ok(
            "enum Color { Red, Green, Blue }
             fn default_color() -> Color = Color::Red;"
        ));
    }

    #[test]
    fn test_tc_match_exhaustiveness() {
        assert!(ok(
            "enum Color { Red, Green, Blue }
             fn to_int(c: Color) -> i64 = match c {
                 Color::Red => 0,
                 Color::Green => 1,
                 Color::Blue => 2,
             };"
        ));
    }

    #[test]
    fn test_tc_contract_precondition() {
        assert!(ok(
            "fn safe_div(a: i64, b: i64) -> i64
               pre b != 0
             = a / b;"
        ));
    }

    #[test]
    fn test_tc_contract_postcondition() {
        assert!(ok(
            "fn abs(x: i64) -> i64
               post ret >= 0
             = if x >= 0 { x } else { 0 - x };"
        ));
    }

    #[test]
    fn test_tc_unused_binding_warning() {
        let tc = check(
            "fn test() -> i64 = { let x: i64 = 42; 0 };"
        ).expect("should type-check");
        assert!(tc.warnings().iter().any(|w| w.kind() == "unused_binding"),
            "Should warn about unused binding 'x'");
    }

    #[test]
    fn test_tc_unused_binding_underscore_no_warning() {
        let tc = check(
            "fn test() -> i64 = { let _x: i64 = 42; 0 };"
        ).expect("should type-check");
        assert!(!tc.warnings().iter().any(|w|
            w.kind() == "unused_binding" && format!("{:?}", w).contains("_x")),
            "Underscore-prefixed should not warn");
    }

    #[test]
    fn test_tc_f64_operations() {
        assert!(ok("fn add_f(a: f64, b: f64) -> f64 = a + b;"));
    }

    #[test]
    fn test_tc_bool_operations() {
        assert!(ok("fn and_op(a: bool, b: bool) -> bool = a and b;"));
    }

    #[test]
    fn test_tc_string_type() {
        assert!(ok(r#"fn greet() -> String = "hello";"#));
    }

    #[test]
    fn test_tc_if_else_type_consistency() {
        // Both branches must have same type
        assert!(err(
            "fn bad(x: bool) -> i64 = if x { 1 } else { true };"
        ));
    }

    #[test]
    fn test_tc_builtin_functions() {
        // println, print, etc. are registered as builtins
        assert!(ok(
            "fn test() -> () = { println(42); () };"
        ));
    }

    // ====================================================================
    // Cycle 79: Extended type checker tests
    // ====================================================================

    #[test]
    fn test_tc_let_binding() {
        assert!(ok("fn test() -> i64 = { let x: i64 = 10; x + 1 };"));
    }

    #[test]
    fn test_tc_nested_if() {
        assert!(ok(
            "fn test(a: bool, b: bool) -> i64 = if a { if b { 1 } else { 2 } } else { 3 };"
        ));
    }

    #[test]
    fn test_tc_while_loop() {
        assert!(ok(
            "fn test() -> i64 = { let mut i: i64 = 0; while i < 10 { i = i + 1 }; i };"
        ));
    }

    #[test]
    fn test_tc_for_loop() {
        assert!(ok(
            "fn test() -> i64 = { let mut sum: i64 = 0; for i in 0..10 { sum = sum + i }; sum };"
        ));
    }

    #[test]
    fn test_tc_tuple_type() {
        assert!(ok(
            "fn test() -> (i64, bool) = (42, true);"
        ));
    }

    #[test]
    fn test_tc_index_access() {
        assert!(ok(
            "fn test(arr: *i64, i: i64) -> i64 = arr[i];"
        ));
    }

    #[test]
    fn test_tc_enum_with_data() {
        assert!(ok(
            "enum Option { Some(i64), None }
             fn test() -> Option = Option::Some(42);"
        ));
    }

    #[test]
    fn test_tc_match_with_wildcard() {
        assert!(ok(
            "fn test(x: i64) -> i64 = match x { 0 => 1, _ => x };"
        ));
    }

    #[test]
    fn test_tc_arity_mismatch() {
        // Calling with wrong number of arguments
        assert!(err(
            "fn add(a: i64, b: i64) -> i64 = a + b;
             fn test() -> i64 = add(1);"
        ));
    }

    #[test]
    fn test_tc_arity_too_many() {
        assert!(err(
            "fn id(x: i64) -> i64 = x;
             fn test() -> i64 = id(1, 2);"
        ));
    }

    #[test]
    fn test_tc_multiple_functions() {
        assert!(ok(
            "fn double(x: i64) -> i64 = x * 2;
             fn test() -> i64 = double(21);"
        ));
    }

    #[test]
    fn test_tc_mutual_recursion() {
        assert!(ok(
            "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
             fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };"
        ));
    }

    #[test]
    fn test_tc_struct_wrong_field_type() {
        assert!(err(
            "struct Point { x: i64, y: i64 }
             fn bad() -> Point = new Point { x: true, y: 0 };"
        ));
    }

    #[test]
    fn test_tc_unary_neg() {
        assert!(ok("fn neg(x: i64) -> i64 = -x;"));
    }

    #[test]
    fn test_tc_unary_not() {
        assert!(ok("fn flip(x: bool) -> bool = not x;"));
    }

    #[test]
    fn test_tc_comparison_returns_bool() {
        assert!(ok("fn gt(a: i64, b: i64) -> bool = a > b;"));
    }

    #[test]
    fn test_tc_block_returns_last_expr() {
        assert!(ok("fn test() -> i64 = { 1; 2; 3 };"));
    }

    #[test]
    fn test_tc_i32_type() {
        assert!(ok("fn test(x: i32) -> i32 = x;"));
    }

    #[test]
    fn test_tc_pre_and_post() {
        assert!(ok(
            "fn safe_abs(x: i64) -> i64
               pre x > -1000000
               post ret >= 0
             = if x >= 0 { x } else { 0 - x };"
        ));
    }

    #[test]
    fn test_tc_string_concat() {
        assert!(ok(r#"fn greet(name: String) -> String = "Hello, " + name;"#));
    }

    #[test]
    fn test_tc_mutable_binding() {
        assert!(ok(
            "fn test() -> i64 = { let mut x: i64 = 0; x = 42; x };"
        ));
    }

    #[test]
    fn test_tc_nested_struct_access() {
        assert!(ok(
            "struct Inner { val: i64 }
             struct Outer { inner: Inner }
             fn get(o: Outer) -> i64 = o.inner.val;"
        ));
    }

    #[test]
    fn test_tc_enum_match_all_variants() {
        assert!(ok(
            "enum Shape { Circle(f64), Rect(f64, f64) }
             fn area(s: Shape) -> f64 = match s {
                 Shape::Circle(r) => r * r,
                 Shape::Rect(w, h) => w * h,
             };"
        ));
    }

    // --- Cycle 89: Extended type checking tests ---

    #[test]
    fn test_tc_generic_function() {
        assert!(ok(
            "fn identity<T>(x: T) -> T = x;"
        ));
    }

    #[test]
    fn test_tc_generic_struct() {
        assert!(ok(
            "struct Pair<A, B> { first: A, second: B }
             fn make_pair(a: i64, b: f64) -> Pair<i64, f64> = new Pair { first: a, second: b };"
        ));
    }

    #[test]
    fn test_tc_u32_type() {
        assert!(ok("fn to_u32(x: u32) -> u32 = x;"));
    }

    #[test]
    fn test_tc_u64_type() {
        assert!(ok("fn to_u64(x: u64) -> u64 = x;"));
    }

    #[test]
    fn test_tc_char_type() {
        assert!(ok("fn get_char() -> char = 'a';"));
    }

    #[test]
    fn test_tc_return_unit() {
        assert!(ok("fn noop() -> () = ();"));
    }

    #[test]
    fn test_tc_nested_match() {
        assert!(ok(
            "enum Option { Some(i64), None }
             fn unwrap_or(opt: Option, default: i64) -> i64 = match opt {
                 Option::Some(v) => v,
                 Option::None => default,
             };"
        ));
    }

    #[test]
    fn test_tc_while_with_mutation() {
        assert!(ok(
            "fn sum_to_ten() -> i64 = { let mut i: i64 = 0; let mut sum: i64 = 0; while i < 10 { i = i + 1; sum = sum + i }; sum };"
        ));
    }

    #[test]
    fn test_tc_for_range_extended() {
        assert!(ok(
            "fn sum_range() -> i64 = { let mut total: i64 = 0; for i in 0..10 { total = total + i }; total };"
        ));
    }

    #[test]
    fn test_tc_wrong_return_type() {
        assert!(err(
            "fn bad() -> i64 = true;"
        ));
    }

    #[test]
    fn test_tc_wrong_if_branch_types() {
        assert!(err(
            "fn bad(x: bool) -> i64 = if x { 1 } else { true };"
        ));
    }

    #[test]
    fn test_tc_struct_missing_field() {
        assert!(err(
            "struct Point { x: i64, y: i64 }
             fn bad() -> Point = new Point { x: 1 };"
        ));
    }

    #[test]
    fn test_tc_struct_field_type_wrong() {
        // Wrong type for a field
        assert!(err(
            "struct Pt { x: i64, y: i64 }
             fn bad() -> Pt = new Pt { x: true, y: 2 };"
        ));
    }

    #[test]
    fn test_tc_bitwise_operators() {
        assert!(ok(
            "fn bits(a: i64, b: i64) -> i64 = (a band b) bor (a bxor b);"
        ));
    }

    #[test]
    fn test_tc_shift_operators() {
        assert!(ok(
            "fn shifts(x: i64) -> i64 = (x << 2) >> 1;"
        ));
    }

    #[test]
    fn test_tc_wrapping_arithmetic() {
        assert!(ok(
            "fn wrap(a: i64, b: i64) -> i64 = a +% b;"
        ));
    }

    #[test]
    fn test_tc_extern_function() {
        assert!(ok(
            "extern fn puts(s: i64) -> i64;"
        ));
    }

    #[test]
    fn test_tc_type_alias() {
        assert!(ok(
            "type Index = i64;
             fn get_idx() -> Index = 0;"
        ));
    }

    #[test]
    fn test_tc_multiple_params_type_mismatch() {
        assert!(err(
            "fn add(a: i64, b: i64) -> i64 = a + b;
             fn bad() -> i64 = add(1, true);"
        ));
    }

    #[test]
    fn test_tc_recursive_type_struct() {
        assert!(ok(
            "struct Node { value: i64 }
             fn get_val(n: Node) -> i64 = n.value;"
        ));
    }

    #[test]
    fn test_tc_as_cast() {
        assert!(ok(
            "fn cast_to_f64(x: i64) -> f64 = x as f64;"
        ));
    }

    #[test]
    fn test_tc_nested_function_calls() {
        assert!(ok(
            "fn inc(x: i64) -> i64 = x + 1;
             fn double(x: i64) -> i64 = x + x;
             fn compose() -> i64 = double(inc(5));"
        ));
    }

    #[test]
    fn test_tc_pointer_index() {
        assert!(ok(
            "fn get(arr: *i64, i: i64) -> i64 = arr[i];"
        ));
    }

    #[test]
    fn test_tc_invariant() {
        assert!(ok(
            "fn checked(x: i64) -> i64
                 pre x > 0
                 post ret >= x
             = x + 1;"
        ));
    }

    #[test]
    fn test_tc_logical_operators() {
        assert!(ok(
            "fn logic(a: bool, b: bool) -> bool = (a && b) || (not a);"
        ));
    }

    #[test]
    fn test_tc_cast_i32_to_i64() {
        assert!(ok("fn widen(x: i32) -> i64 = x as i64;"));
    }

    #[test]
    fn test_tc_block_with_let() {
        assert!(ok("fn test() -> i64 = { let a: i64 = 10; let b: i64 = 20; a + b };"));
    }

    #[test]
    fn test_tc_nested_blocks() {
        assert!(ok("fn test() -> i64 = { let a: i64 = { let b: i64 = 5; b + 1 }; a };"));
    }

    #[test]
    fn test_tc_enum_no_data() {
        assert!(ok(
            "enum Dir { N, S, E, W }
             fn is_north(d: Dir) -> bool = match d { Dir::N => true, _ => false };"
        ));
    }

    // ====================================================================
    // Extended integration tests: float types, mixed numerics, method calls,
    // generics, contracts, while loops, block expressions
    // ====================================================================

    // --- f64 float type inference ---

    #[test]
    fn test_tc_f64_let_binding() {
        assert!(ok("fn test() -> f64 = { let x: f64 = 3.14; x };"));
    }

    #[test]
    fn test_tc_f64_literal_return() {
        assert!(ok("fn pi() -> f64 = 3.14159;"));
    }

    // --- Float arithmetic type checking ---

    #[test]
    fn test_tc_f64_add() {
        assert!(ok("fn add_f(a: f64, b: f64) -> f64 = a + b;"));
    }

    #[test]
    fn test_tc_f64_sub() {
        assert!(ok("fn sub_f(a: f64, b: f64) -> f64 = a - b;"));
    }

    #[test]
    fn test_tc_f64_mul() {
        assert!(ok("fn mul_f(a: f64, b: f64) -> f64 = a * b;"));
    }

    #[test]
    fn test_tc_f64_div() {
        assert!(ok("fn div_f(a: f64, b: f64) -> f64 = a / b;"));
    }

    #[test]
    fn test_tc_f64_negation() {
        assert!(ok("fn neg_f(x: f64) -> f64 = -x;"));
    }

    // --- Float comparison type checking ---

    #[test]
    fn test_tc_f64_less_than() {
        assert!(ok("fn lt_f(a: f64, b: f64) -> bool = a < b;"));
    }

    #[test]
    fn test_tc_f64_greater_equal() {
        assert!(ok("fn ge_f(a: f64, b: f64) -> bool = a >= b;"));
    }

    #[test]
    fn test_tc_f64_equality() {
        assert!(ok("fn eq_f(a: f64, b: f64) -> bool = a == b;"));
    }

    // --- Mixed numeric type errors ---

    #[test]
    fn test_tc_mixed_i64_f64_add_error() {
        // i64 + f64 should be a type error (no implicit coercion)
        assert!(err("fn bad(a: i64, b: f64) -> f64 = a + b;"));
    }

    #[test]
    fn test_tc_mixed_f64_i64_sub_error() {
        assert!(err("fn bad(a: f64, b: i64) -> f64 = a - b;"));
    }

    #[test]
    fn test_tc_mixed_i64_f64_comparison_error() {
        // Comparing i64 with f64 should be a type error
        assert!(err("fn bad(a: i64, b: f64) -> bool = a < b;"));
    }

    // --- Option / Nullable types ---

    #[test]
    fn test_tc_option_enum_some() {
        assert!(ok(
            "enum Option { Some(i64), None }
             fn wrap(x: i64) -> Option = Option::Some(x);"
        ));
    }

    #[test]
    fn test_tc_option_enum_none() {
        assert!(ok(
            "enum Option { Some(i64), None }
             fn empty() -> Option = Option::None;"
        ));
    }

    #[test]
    fn test_tc_option_enum_match_unwrap() {
        assert!(ok(
            "enum Option { Some(i64), None }
             fn unwrap(opt: Option) -> i64 = match opt {
                 Option::Some(v) => v,
                 Option::None => 0,
             };"
        ));
    }

    // --- Method call type checking for built-in types ---

    #[test]
    fn test_tc_string_len_returns_i64() {
        assert!(ok(
            r#"fn length(s: String) -> i64 = s.len();"#
        ));
    }

    #[test]
    fn test_tc_string_is_empty_returns_bool() {
        assert!(ok(
            r#"fn check(s: String) -> bool = s.is_empty();"#
        ));
    }

    #[test]
    fn test_tc_string_len_wrong_return_type() {
        // String.len() returns i64, not bool
        assert!(err(
            r#"fn bad(s: String) -> bool = s.len();"#
        ));
    }

    #[test]
    fn test_tc_string_slice_returns_string() {
        assert!(ok(
            r#"fn take(s: String) -> String = s.slice(0, 5);"#
        ));
    }

    // --- Generic function type inference ---

    #[test]
    fn test_tc_generic_identity_with_i64() {
        assert!(ok(
            "fn identity<T>(x: T) -> T = x;
             fn test() -> i64 = identity(42);"
        ));
    }

    #[test]
    fn test_tc_generic_identity_with_bool() {
        assert!(ok(
            "fn identity<T>(x: T) -> T = x;
             fn test() -> bool = identity(true);"
        ));
    }

    #[test]
    fn test_tc_generic_pair_struct() {
        assert!(ok(
            "struct Pair<A, B> { first: A, second: B }
             fn swap<A, B>(p: Pair<A, B>) -> Pair<B, A> = new Pair { first: p.second, second: p.first };"
        ));
    }

    // --- Contract pre/post condition type checking ---

    #[test]
    fn test_tc_contract_pre_must_be_bool() {
        // pre condition uses boolean expression
        assert!(ok(
            "fn safe_sqrt(x: f64) -> f64
               pre x >= 0.0
             = x;"
        ));
    }

    #[test]
    fn test_tc_contract_post_with_ret() {
        // post condition references 'ret' which should have the return type
        assert!(ok(
            "fn clamp(x: i64) -> i64
               pre x >= 0
               post ret >= 0
             = if x > 100 { 100 } else { x };"
        ));
    }

    #[test]
    fn test_tc_contract_pre_and_post_combined() {
        assert!(ok(
            "fn bounded_add(a: i64, b: i64) -> i64
               pre a >= 0 and b >= 0
               post ret >= a
             = a + b;"
        ));
    }

    // --- While loop type (returns unit) ---

    #[test]
    fn test_tc_while_loop_unit_type() {
        // while loop itself produces unit (); we use it in a block
        assert!(ok(
            "fn count() -> () = { let mut i: i64 = 0; while i < 5 { i = i + 1 }; () };"
        ));
    }

    #[test]
    fn test_tc_while_condition_must_be_bool() {
        // The while condition must be bool -- using an integer should fail
        assert!(err(
            "fn bad() -> () = { let mut i: i64 = 10; while i { i = i - 1 }; () };"
        ));
    }

    // --- Block expression type (returns last expression's type) ---

    #[test]
    fn test_tc_block_returns_last_expr_type() {
        assert!(ok("fn test() -> bool = { let _x: i64 = 1; true };"));
    }

    #[test]
    fn test_tc_block_nested_returns_inner_type() {
        assert!(ok(
            "fn test() -> i64 = { let a: i64 = { let b: i64 = 10; b * 2 }; a + 1 };"
        ));
    }

    #[test]
    fn test_tc_block_type_mismatch() {
        // Block returns bool but function expects i64
        assert!(err("fn bad() -> i64 = { true };"));
    }

    #[test]
    fn test_tc_empty_block_returns_unit() {
        // An empty block or block ending in statement should return unit
        assert!(ok("fn test() -> () = { let _x: i64 = 1; () };"));
    }

    // v0.89.17: Nullable type checking tests

    #[test]
    fn test_tc_nullable_if_else_with_null() {
        // if-else with T and null should produce T?
        assert!(ok("fn maybe(x: i64) -> i64? = if x > 0 { x } else { null };"));
    }

    #[test]
    fn test_tc_nullable_if_else_null_first() {
        // null in then-branch, value in else-branch
        assert!(ok("fn maybe(x: i64) -> i64? = if x <= 0 { null } else { x };"));
    }

    #[test]
    fn test_tc_nullable_direct_value() {
        // Non-null value assigned to nullable type
        assert!(ok("fn get() -> i64? = 42;"));
    }

    #[test]
    fn test_tc_nullable_direct_null() {
        // null assigned to nullable type
        assert!(ok("fn get() -> i64? = null;"));
    }

    #[test]
    fn test_tc_nullable_string() {
        // String? nullable
        assert!(ok("fn find(s: String) -> String? = if s == \"\" { null } else { s };"));
    }

    #[test]
    fn test_tc_nullable_f64() {
        // f64? nullable
        assert!(ok("fn safe_div(a: f64, b: f64) -> f64? = if b == 0.0 { null } else { a };"));
    }

    #[test]
    fn test_tc_nullable_type_mismatch() {
        // i64? should not accept bool
        assert!(err("fn bad() -> i64? = true;"));
    }

    // ============================================
    // v0.89.18: Edge Case Tests (Cycle 106)
    // ============================================

    #[test]
    fn test_tc_modulo_operator() {
        assert!(ok("fn rem(a: i64, b: i64) -> i64 = a % b;"));
    }

    #[test]
    fn test_tc_deep_nested_struct_access() {
        assert!(ok(
            "struct Inner { val: i64 }
             struct Outer { inner: Inner }
             fn get_val(o: Outer) -> i64 = o.inner.val;"
        ));
    }

    #[test]
    fn test_tc_multiple_params_same_type() {
        assert!(ok("fn sum3(a: i64, b: i64, c: i64) -> i64 = a + b + c;"));
    }

    #[test]
    fn test_tc_generic_with_two_type_params() {
        assert!(ok(
            "struct Pair<A, B> { first: A, second: B }
             fn make_pair(x: i64, y: bool) -> Pair<i64, bool> = new Pair { first: x, second: y };"
        ));
    }

    #[test]
    fn test_tc_match_with_guard() {
        assert!(ok(
            "fn classify(x: i64) -> i64 = match x {
                n if n > 100 => 3,
                n if n > 10 => 2,
                n if n > 0 => 1,
                _ => 0
             };"
        ));
    }

    #[test]
    fn test_tc_recursive_fibonacci() {
        assert!(ok(
            "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };"
        ));
    }

    #[test]
    fn test_tc_mutual_recursion_even_odd() {
        assert!(ok(
            "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
             fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };"
        ));
    }

    #[test]
    fn test_tc_boolean_expressions() {
        assert!(ok(
            "fn complex_bool(a: bool, b: bool, c: bool) -> bool = (a and b) or (not c);"
        ));
    }

    #[test]
    fn test_tc_chained_comparison() {
        assert!(ok(
            "fn in_range(x: i64, lo: i64, hi: i64) -> bool = x >= lo and x <= hi;"
        ));
    }

    #[test]
    fn test_tc_nested_match_two_levels() {
        assert!(ok(
            "fn nested(x: i64, y: i64) -> i64 =
                match x {
                    0 => match y { 0 => 0, _ => 1 },
                    _ => match y { 0 => 2, _ => 3 }
                };"
        ));
    }

    #[test]
    fn test_tc_struct_with_multiple_fields() {
        assert!(ok(
            "struct Vec3 { x: f64, y: f64, z: f64 }
             fn dot(a: Vec3, b: Vec3) -> f64 = a.x * b.x + a.y * b.y + a.z * b.z;"
        ));
    }

    #[test]
    fn test_tc_function_returning_struct() {
        assert!(ok(
            "struct Point { x: i64, y: i64 }
             fn origin() -> Point = new Point { x: 0, y: 0 };"
        ));
    }

    #[test]
    fn test_tc_complex_expression_types() {
        // Verify arithmetic with nested if produces correct type
        assert!(ok(
            "fn calc(x: i64) -> i64 = x * (if x > 0 { 2 } else { 3 }) + 1;"
        ));
    }

    #[test]
    fn test_tc_string_equality() {
        assert!(ok(
            "fn is_hello(s: String) -> bool = s == \"hello\";"
        ));
    }

    #[test]
    fn test_tc_wrong_arg_count() {
        assert!(err("fn add(a: i64, b: i64) -> i64 = a + b; fn main() -> i64 = add(1);"));
    }

    #[test]
    fn test_tc_wrong_arg_type() {
        assert!(err("fn add(a: i64, b: i64) -> i64 = a + b; fn main() -> i64 = add(1, true);"));
    }

    #[test]
    fn test_tc_pure_annotation() {
        assert!(ok(
            "@pure
             fn square(x: i64) -> i64 = x * x;"
        ));
    }

    #[test]
    fn test_tc_contract_pre_post_combined() {
        assert!(ok(
            "fn abs(x: i64) -> i64
                pre x > 0 - 1000000
                post ret >= 0
             = if x >= 0 { x } else { 0 - x };"
        ));
    }

    #[test]
    fn test_tc_nullable_method_is_some() {
        assert!(ok("fn check(opt: i64?) -> bool = opt.is_some();"));
    }

    #[test]
    fn test_tc_nullable_method_is_none() {
        assert!(ok("fn check(opt: i64?) -> bool = opt.is_none();"));
    }

    #[test]
    fn test_tc_nullable_method_unwrap_or() {
        assert!(ok("fn safe_get(opt: i64?) -> i64 = opt.unwrap_or(0);"));
    }

    #[test]
    fn test_tc_bitwise_operations() {
        assert!(ok("fn mask(x: i64) -> i64 = x band 255;"));
        assert!(ok("fn flags(a: i64, b: i64) -> i64 = a bor b;"));
        assert!(ok("fn toggle(x: i64) -> i64 = x bxor 1;"));
    }

    #[test]
    fn test_tc_shift_operations() {
        assert!(ok("fn shl(x: i64) -> i64 = x << 3;"));
        assert!(ok("fn shr(x: i64) -> i64 = x >> 1;"));
    }

    // ============================================
    // Error Handling Tests: Verifying the compiler
    // rejects invalid programs and produces correct
    // error diagnostics.
    // ============================================

    /// Helper: parse and type-check, returning the error message if it fails
    fn err_msg(source: &str) -> String {
        match check(source) {
            Err(e) => e.message().to_string(),
            Ok(_) => panic!("expected type error, but source type-checked successfully"),
        }
    }

    #[test]
    fn test_err_add_i64_and_bool() {
        // i64 + bool should be a type error: operands must have the same numeric type
        let msg = err_msg("fn bad(a: i64, b: bool) -> i64 = a + b;");
        assert!(msg.contains("expected") || msg.contains("i64") || msg.contains("bool"),
            "Error should mention type mismatch, got: {msg}");
    }

    #[test]
    fn test_err_mul_bool_operands() {
        // bool * bool is not a valid arithmetic operation
        assert!(err("fn bad(a: bool, b: bool) -> bool = a * b;"));
    }

    #[test]
    fn test_err_undefined_struct_in_constructor() {
        // Using a struct name that was never defined
        let msg = err_msg("fn bad() -> i64 = { let _p = new Nonexistent { x: 1 }; 0 };");
        assert!(msg.contains("undefined struct") || msg.contains("Nonexistent"),
            "Error should mention undefined struct, got: {msg}");
    }

    #[test]
    fn test_err_undefined_enum_variant_access() {
        // Referencing an enum that does not exist
        let msg = err_msg("fn bad() -> i64 = { let _c = Ghost::Phantom; 0 };");
        assert!(msg.contains("undefined enum") || msg.contains("Ghost"),
            "Error should mention undefined enum, got: {msg}");
    }

    #[test]
    fn test_err_unknown_variant_on_enum() {
        // Referencing a variant that doesn't exist on a defined enum
        let msg = err_msg(
            "enum Color { Red, Green, Blue }
             fn bad() -> Color = Color::Yellow;"
        );
        assert!(msg.contains("unknown variant") || msg.contains("Yellow"),
            "Error should mention unknown variant, got: {msg}");
    }

    #[test]
    fn test_err_unknown_field_on_struct() {
        // Accessing a field that doesn't exist on the struct
        let msg = err_msg(
            "struct Point { x: i64, y: i64 }
             fn bad(p: Point) -> i64 = p.z;"
        );
        assert!(msg.contains("unknown field") || msg.contains("z"),
            "Error should mention unknown field, got: {msg}");
    }

    #[test]
    fn test_err_negation_on_bool() {
        // Unary minus on bool should fail (negation requires numeric type)
        let msg = err_msg("fn bad(b: bool) -> bool = -b;");
        assert!(msg.contains("negation") || msg.contains("numeric"),
            "Error should mention negation requires numeric type, got: {msg}");
    }

    #[test]
    fn test_err_not_on_i64() {
        // Logical `not` on an integer should fail (requires bool)
        let msg = err_msg("fn bad(x: i64) -> bool = not x;");
        assert!(msg.contains("expected") || msg.contains("bool") || msg.contains("i64"),
            "Error should mention type mismatch, got: {msg}");
    }

    #[test]
    fn test_err_unknown_method_on_string() {
        // Calling a method that doesn't exist on String
        let msg = err_msg(r#"fn bad(s: String) -> i64 = s.nonexistent();"#);
        assert!(msg.contains("unknown method") || msg.contains("nonexistent"),
            "Error should mention unknown method, got: {msg}");
    }

    #[test]
    fn test_err_method_call_on_integer() {
        // Calling a method on i64 which has no methods
        assert!(err("fn bad(x: i64) -> i64 = x.len();"));
    }

    #[test]
    fn test_err_if_condition_not_bool() {
        // Using an integer as if-condition should fail
        let msg = err_msg("fn bad(x: i64) -> i64 = if x { 1 } else { 0 };");
        assert!(msg.contains("expected") || msg.contains("bool"),
            "Error should mention bool expected for condition, got: {msg}");
    }

    #[test]
    fn test_err_enum_variant_too_many_args() {
        // Providing more arguments than the variant expects
        let msg = err_msg(
            "enum Option { Some(i64), None }
             fn bad() -> Option = Option::Some(1, 2);"
        );
        assert!(msg.contains("expected") || msg.contains("args") || msg.contains("argument"),
            "Error should mention argument count mismatch, got: {msg}");
    }

    #[test]
    fn test_err_enum_variant_too_few_args() {
        // Providing fewer arguments than the variant expects
        let msg = err_msg(
            "enum Option { Some(i64), None }
             fn bad() -> Option = Option::Some();"
        );
        assert!(msg.contains("expected") || msg.contains("args") || msg.contains("argument"),
            "Error should mention argument count mismatch, got: {msg}");
    }

    #[test]
    fn test_err_field_access_on_non_struct() {
        // Accessing a field on a primitive type (i64)
        let msg = err_msg("fn bad(x: i64) -> i64 = x.foo;");
        assert!(msg.contains("non-struct") || msg.contains("field"),
            "Error should mention field access on non-struct type, got: {msg}");
    }

    #[test]
    fn test_err_index_non_array() {
        // Indexing into a bool which is not an array or pointer
        let msg = err_msg("fn bad(x: bool) -> bool = x[0];");
        assert!(msg.contains("Cannot index") || msg.contains("index"),
            "Error should mention indexing into wrong type, got: {msg}");
    }

    #[test]
    fn test_err_let_binding_type_mismatch() {
        // Let binding declares i64 but assigns bool
        assert!(err("fn bad() -> i64 = { let x: i64 = true; x };"));
    }

    #[test]
    fn test_err_match_arm_type_mismatch() {
        // Match arms return different types (i64 vs bool)
        assert!(err(
            "fn bad(x: i64) -> i64 = match x { 0 => 1, _ => true };"
        ));
    }

    #[test]
    fn test_err_struct_extra_field_wrong_type() {
        // Providing correct field names but wrong types for both fields
        assert!(err(
            "struct Pair { a: i64, b: bool }
             fn bad() -> Pair = new Pair { a: true, b: 42 };"
        ));
    }

    #[test]
    fn test_err_bitwise_on_bool() {
        // Bitwise operators require integer type, not bool
        assert!(err("fn bad(a: bool, b: bool) -> bool = a band b;"));
    }

    // ================================================================
    // Cycle 123: Extended Type Checker Tests
    // ================================================================

    #[test]
    fn test_tc_nullable_is_some_branch() {
        assert!(ok(
            "fn safe_get(opt: i64?) -> i64 = if opt.is_some() { opt.unwrap_or(0) } else { 0 };"
        ));
    }

    #[test]
    fn test_tc_nullable_assignment_null() {
        assert!(ok("fn make_none() -> i64? = null;"));
    }

    #[test]
    fn test_tc_generic_pair() {
        assert!(ok(
            "struct Pair<T, U> { first: T, second: U }
             fn make_pair(a: i64, b: bool) -> Pair<i64, bool> = new Pair { first: a, second: b };"
        ));
    }

    #[test]
    fn test_tc_generic_function_identity() {
        assert!(ok("fn identity<T>(x: T) -> T = x;"));
    }

    #[test]
    fn test_tc_type_alias_basic() {
        assert!(ok("type Int = i64; fn add(a: Int, b: Int) -> Int = a + b;"));
    }

    #[test]
    fn test_tc_enum_with_data_types() {
        assert!(ok(
            "enum Shape { Circle(f64), Rectangle(f64, f64), Point }
             fn area(s: Shape) -> f64 = match s {
                 Shape::Circle(r) => r * r * 3.14,
                 Shape::Rectangle(w, h) => w * h,
                 Shape::Point => 0.0,
             };"
        ));
    }

    #[test]
    fn test_tc_recursive_struct() {
        // Self-referential struct via pointer
        assert!(ok(
            "struct Node { value: i64, next: Node? }
             fn get_value(n: Node) -> i64 = n.value;"
        ));
    }

    #[test]
    fn test_tc_multiple_return_paths_same_type() {
        assert!(ok(
            "fn classify(x: i64) -> i64 = if x > 100 { 3 } else if x > 10 { 2 } else if x > 0 { 1 } else { 0 };"
        ));
    }

    #[test]
    fn test_tc_nested_struct_field_access() {
        assert!(ok(
            "struct Inner { value: i64 }
             struct Outer { inner: Inner }
             fn get_inner_value(o: Outer) -> i64 = o.inner.value;"
        ));
    }

    #[test]
    fn test_tc_for_loop_type_inference() {
        assert!(ok(
            "fn sum_range(n: i64) -> i64 = { let mut s = 0; for i in 0..n { s = s + i }; s };"
        ));
    }

    #[test]
    fn test_tc_while_loop_with_mutation() {
        assert!(ok(
            "fn count_down(n: i64) -> i64 = { let mut x = n; while x > 0 { x = x - 1 }; x };"
        ));
    }

    #[test]
    fn test_tc_contract_with_combined_pre() {
        assert!(ok(
            "fn safe_sub(a: i64, b: i64) -> i64
                pre a >= 0 and b >= 0 and a >= b
             = a - b;"
        ));
    }

    #[test]
    fn test_tc_contract_with_post_ret() {
        assert!(ok(
            "fn positive(x: i64) -> i64
                post ret > 0
             = if x > 0 { x } else { 1 };"
        ));
    }

    #[test]
    fn test_err_nullable_method_on_non_nullable() {
        // is_some() should not be callable on non-nullable i64
        assert!(err("fn bad(x: i64) -> bool = x.is_some();"));
    }

    #[test]
    fn test_err_return_nullable_for_non_nullable() {
        // Returning i64? where i64 is expected
        assert!(err(
            "fn bad(opt: i64?) -> i64 = opt;"
        ));
    }

    #[test]
    fn test_err_assign_wrong_type_to_mut() {
        // Assigning bool to a mut i64 variable
        assert!(err(
            "fn bad() -> i64 = { let mut x = 42; x = true; x };"
        ));
    }

    #[test]
    fn test_err_struct_missing_field() {
        // Struct constructor missing a required field
        assert!(err(
            "struct Point { x: i64, y: i64 }
             fn bad() -> Point = new Point { x: 1 };"
        ));
    }

    #[test]
    fn test_err_call_with_too_many_args() {
        // Calling a function with more args than it accepts
        assert!(err(
            "fn single(x: i64) -> i64 = x;
             fn main() -> i64 = single(1, 2, 3);"
        ));
    }

    #[test]
    fn test_err_recursive_return_type_mismatch() {
        // Recursive function where base case returns wrong type
        assert!(err(
            "fn bad(n: i64) -> i64 = if n <= 0 { true } else { bad(n - 1) };"
        ));
    }

    #[test]
    fn test_tc_array_literal_type() {
        assert!(ok("fn first() -> i64 = { let arr = [1, 2, 3]; arr[0] };"));
    }

    #[test]
    fn test_tc_string_methods() {
        assert!(ok("fn slen(s: String) -> i64 = s.len();"));
    }

    #[test]
    fn test_tc_tuple_types() {
        assert!(ok("fn swap(t: (i64, bool)) -> (bool, i64) = (t.1, t.0);"));
    }

    #[test]
    fn test_err_tuple_index_out_of_bounds() {
        assert!(err("fn bad(t: (i64, bool)) -> i64 = t.5;"));
    }

    #[test]
    fn test_tc_deeply_nested_if_else() {
        // Deeply nested if-else with consistent return types
        assert!(ok(
            "fn deep(x: i64, y: i64) -> i64 =
                 if x > 10 {
                     if y > 5 { x + y }
                     else { x - y }
                 } else {
                     if y > 5 { y - x }
                     else { 0 }
                 };"
        ));
    }

    // ================================================================
    // Extended Edge Case Tests
    // ================================================================

    #[test]
    fn test_err_cast_string_to_i64() {
        // Casting String to i64 is not a valid numeric cast
        assert!(err(r#"fn bad(s: String) -> i64 = s as i64;"#));
    }

    #[test]
    fn test_err_cast_unit_to_i64() {
        // Casting unit () to i64 is not a valid cast
        assert!(err("fn bad() -> i64 = () as i64;"));
    }

    #[test]
    fn test_tc_cast_u32_to_u64() {
        // Unsigned integer widening cast should succeed
        assert!(ok("fn widen(x: u32) -> u64 = x as u64;"));
    }

    #[test]
    fn test_tc_cast_f64_to_i64() {
        // Float to integer truncation cast should succeed
        assert!(ok("fn trunc(x: f64) -> i64 = x as i64;"));
    }

    #[test]
    fn test_err_u32_arithmetic_with_i64() {
        // Mixing u32 and i64 in arithmetic should be a type error (no implicit coercion)
        assert!(err("fn bad(a: u32, b: i64) -> i64 = a + b;"));
    }

    #[test]
    fn test_tc_u64_arithmetic() {
        // Pure u64 arithmetic should type check
        assert!(ok("fn add_u(a: u64, b: u64) -> u64 = a + b;"));
    }

    #[test]
    fn test_tc_u32_comparison_returns_bool() {
        // Comparison of unsigned integers should return bool
        assert!(ok("fn gt_u(a: u32, b: u32) -> bool = a > b;"));
    }

    #[test]
    fn test_tc_char_literal_in_function() {
        // char type should work as parameter and return type
        assert!(ok("fn echo_char(c: char) -> char = c;"));
    }

    #[test]
    fn test_err_char_arithmetic() {
        // Arithmetic on char should fail (char is not a numeric type)
        assert!(err("fn bad(c: char) -> char = c + c;"));
    }

    #[test]
    fn test_err_range_with_float_bounds() {
        // Range requires integer types, float should be rejected
        assert!(err(
            "fn bad() -> () = { for _x in 0.0..10.0 { () }; () };"
        ));
    }

    #[test]
    fn test_err_for_loop_non_range_iterator() {
        // For loop requires Range or Receiver type, not an integer
        assert!(err(
            "fn bad(n: i64) -> () = { for _x in n { () }; () };"
        ));
    }

    #[test]
    fn test_err_array_elements_mixed_types() {
        // Array literal with mixed element types should fail
        assert!(err("fn bad() -> i64 = { let arr = [1, true, 3]; arr[0] };"));
    }

    #[test]
    fn test_tc_array_repeat_syntax() {
        // Array repeat [val; N] syntax should type check
        assert!(ok("fn zeroes() -> i64 = { let arr = [0; 5]; arr[0] };"));
    }

    #[test]
    fn test_err_index_with_bool() {
        // Array index must be an integer, not bool
        assert!(err("fn bad(arr: *i64, b: bool) -> i64 = arr[b];"));
    }

    #[test]
    fn test_err_nullable_unwrap_or_wrong_type() {
        // unwrap_or(default) default must match the inner type
        let msg = err_msg("fn bad(opt: i64?) -> i64 = opt.unwrap_or(true);");
        assert!(msg.contains("expected") || msg.contains("i64") || msg.contains("bool"),
            "Error should mention type mismatch in unwrap_or default, got: {msg}");
    }

    #[test]
    fn test_tc_nullable_struct() {
        // Nullable should work with user-defined struct types
        assert!(ok(
            "struct Point { x: i64, y: i64 }
             fn maybe_point(flag: bool) -> Point? = if flag { new Point { x: 1, y: 2 } } else { null };"
        ));
    }

    #[test]
    fn test_err_string_byte_at_wrong_arg_type() {
        // byte_at requires integer argument, not bool
        let msg = err_msg(r#"fn bad(s: String) -> i64 = s.byte_at(true);"#);
        assert!(msg.contains("integer") || msg.contains("byte_at"),
            "Error should mention byte_at requires integer, got: {msg}");
    }

    #[test]
    fn test_err_string_slice_wrong_arg_count() {
        // slice() takes exactly 2 arguments
        let msg = err_msg(r#"fn bad(s: String) -> String = s.slice(0);"#);
        assert!(msg.contains("2 arguments") || msg.contains("slice"),
            "Error should mention slice takes 2 arguments, got: {msg}");
    }
}
