//! Expression evaluator

use super::env::{child_env, EnvRef, Environment};
use super::error::{ErrorKind, InterpResult, RuntimeError};
use super::scope::ScopeStack;
use super::value::Value;
use crate::ast::{BinOp, EnumDef, Expr, FnDef, LiteralPattern, Pattern, Program, Spanned, StructDef, Type, UnOp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

// v0.46: Thread-local storage for program arguments
// Used by arg_count() and get_arg() builtins to access program arguments
// passed via `bmb run file.bmb arg1 arg2 ...`
thread_local! {
    static PROGRAM_ARGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// v0.46: Set program arguments for the interpreter
/// Called before running a BMB program to pass command-line arguments
pub fn set_program_args(args: Vec<String>) {
    PROGRAM_ARGS.with(|cell| {
        *cell.borrow_mut() = args;
    });
}

/// v0.46: Get program argument count
fn get_program_arg_count() -> usize {
    PROGRAM_ARGS.with(|cell| cell.borrow().len())
}

/// v0.46: Get program argument by index
fn get_program_arg(index: usize) -> String {
    PROGRAM_ARGS.with(|cell| {
        cell.borrow().get(index).cloned().unwrap_or_default()
    })
}

/// Maximum recursion depth (v0.30.248: increased for bootstrap compiler Stage 3 verification)
const MAX_RECURSION_DEPTH: usize = 100000;

/// Stack growth parameters for deep recursion
/// v0.30.248: 128KB red zone, 4MB growth (original for bootstrap)
const STACK_RED_ZONE: usize = 128 * 1024; // 128KB remaining triggers growth
const STACK_GROW_SIZE: usize = 4 * 1024 * 1024; // Grow by 4MB each time

/// Builtin function type
pub type BuiltinFn = fn(&[Value]) -> InterpResult<Value>;

/// The interpreter
pub struct Interpreter {
    /// Global environment
    global_env: EnvRef,
    /// User-defined functions
    functions: HashMap<String, FnDef>,
    /// Struct definitions
    struct_defs: HashMap<String, StructDef>,
    /// Enum definitions
    enum_defs: HashMap<String, EnumDef>,
    /// Builtin functions
    builtins: HashMap<String, BuiltinFn>,
    /// Current recursion depth
    recursion_depth: usize,
    /// v0.30.280: Stack-based scope for efficient let binding evaluation
    scope_stack: ScopeStack,
    /// v0.30.280: Flag to enable ScopeStack-based evaluation
    use_scope_stack: bool,
    /// v0.35.1: String intern table for O(1) literal reuse (json_parse optimization)
    string_intern: HashMap<String, Rc<String>>,
    /// v0.51.41: Heap storage for typed pointer support
    /// Maps pointer address to struct values for field access through pointers
    heap: RefCell<HashMap<i64, Value>>,
    /// v0.90.40: Trait impl methods: (type_name, method_name) -> FnDef
    impl_methods: HashMap<(String, String), FnDef>,
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        let mut interp = Interpreter {
            global_env: Environment::new().into_ref(),
            functions: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            builtins: HashMap::new(),
            recursion_depth: 0,
            scope_stack: ScopeStack::new(),
            use_scope_stack: false,
            string_intern: HashMap::new(),
            heap: RefCell::new(HashMap::new()),
            impl_methods: HashMap::new(),
        };
        interp.register_builtins();
        interp
    }

    /// v0.51.41: Compute size of a type in bytes (for sizeof operator)
    fn compute_type_size(&self, ty: &Type) -> i64 {
        use crate::ast::Type;
        match ty {
            Type::I32 | Type::U32 => 4,
            Type::I64 | Type::U64 | Type::F64 => 8,
            Type::Bool => 1,
            Type::Char => 4,
            Type::Unit => 0,
            Type::Ptr(_) | Type::Ref(_) | Type::RefMut(_) => 8,
            Type::Named(name) => {
                // Look up struct size
                if let Some(struct_def) = self.struct_defs.get(name) {
                    struct_def.fields.iter().map(|f| self.compute_type_size(&f.ty.node)).sum()
                } else {
                    8 // Default
                }
            }
            Type::Array(elem_ty, size) => {
                self.compute_type_size(elem_ty) * (*size as i64)
            }
            Type::Tuple(elems) => {
                elems.iter().map(|e| self.compute_type_size(e)).sum()
            }
            _ => 8, // Default for unknown types
        }
    }

    /// v0.35.1: Intern a string literal for O(1) reuse
    /// Returns Rc::clone() if already interned, otherwise creates new Rc and stores it
    fn intern_string(&mut self, s: &str) -> Rc<String> {
        if let Some(rc) = self.string_intern.get(s) {
            Rc::clone(rc)
        } else {
            let rc = Rc::new(s.to_string());
            self.string_intern.insert(s.to_string(), Rc::clone(&rc));
            rc
        }
    }

    /// Register built-in functions
    fn register_builtins(&mut self) {
        self.builtins.insert("print".to_string(), builtin_print);
        self.builtins.insert("println".to_string(), builtin_println);
        self.builtins.insert("print_str".to_string(), builtin_print_str);
        self.builtins.insert("println_str".to_string(), builtin_println_str);
        // v0.60.44: Float output functions for spectral_norm, n_body benchmarks
        self.builtins.insert("println_f64".to_string(), builtin_println_f64);
        self.builtins.insert("print_f64".to_string(), builtin_print_f64);
        self.builtins.insert("assert".to_string(), builtin_assert);
        self.builtins.insert("read_int".to_string(), builtin_read_int);
        self.builtins.insert("abs".to_string(), builtin_abs);
        self.builtins.insert("min".to_string(), builtin_min);
        self.builtins.insert("max".to_string(), builtin_max);
        // v0.31.10: File I/O builtins for Phase 32.0 Bootstrap Infrastructure
        self.builtins.insert("read_file".to_string(), builtin_read_file);
        self.builtins.insert("write_file".to_string(), builtin_write_file);
        self.builtins.insert("append_file".to_string(), builtin_append_file);
        self.builtins.insert("file_exists".to_string(), builtin_file_exists);
        self.builtins.insert("file_size".to_string(), builtin_file_size);

        // v0.31.11: Process execution builtins for Phase 32.0.2 Bootstrap Infrastructure
        self.builtins.insert("exec".to_string(), builtin_exec);
        self.builtins.insert("exec_output".to_string(), builtin_exec_output);
        self.builtins.insert("system".to_string(), builtin_system);
        self.builtins.insert("getenv".to_string(), builtin_getenv);

        // v0.63: Timing builtin for bmb-bench
        self.builtins.insert("time_ns".to_string(), builtin_time_ns);

        // v0.31.22: Command-line argument builtins for Phase 32.3.D CLI Independence
        self.builtins.insert("arg_count".to_string(), builtin_arg_count);
        self.builtins.insert("get_arg".to_string(), builtin_get_arg);

        // v0.31.13: StringBuilder builtins for Phase 32.0.4 O(n²) fix
        self.builtins.insert("sb_new".to_string(), builtin_sb_new);
        self.builtins
            .insert("sb_with_capacity".to_string(), builtin_sb_with_capacity);  // v0.51.45
        self.builtins.insert("sb_push".to_string(), builtin_sb_push);
        self.builtins.insert("sb_push_char".to_string(), builtin_sb_push_char);
        self.builtins.insert("sb_build".to_string(), builtin_sb_build);
        self.builtins.insert("sb_len".to_string(), builtin_sb_len);
        self.builtins.insert("sb_clear".to_string(), builtin_sb_clear);
        self.builtins.insert("sb_println".to_string(), builtin_sb_println);
        self.builtins.insert("puts_cstr".to_string(), builtin_puts_cstr);

        // v0.31.21: Character conversion builtins for gotgan string handling
        self.builtins.insert("chr".to_string(), builtin_chr);
        self.builtins.insert("ord".to_string(), builtin_ord);

        // v0.66: String-char interop utilities
        self.builtins.insert("char_at".to_string(), builtin_char_at);
        self.builtins
            .insert("char_to_string".to_string(), builtin_char_to_string);
        // v0.67: String utilities
        self.builtins.insert("str_len".to_string(), builtin_str_len);

        // v0.34: Math intrinsics for Phase 34.4 Benchmark Gate (n_body, mandelbrot_fp)
        self.builtins.insert("sqrt".to_string(), builtin_sqrt);
        self.builtins.insert("i64_to_f64".to_string(), builtin_i64_to_f64);
        self.builtins.insert("f64_to_i64".to_string(), builtin_f64_to_i64);
        // v0.51.47: i32 conversion functions for performance-critical code
        self.builtins.insert("i32_to_f64".to_string(), builtin_i32_to_f64);
        self.builtins.insert("i32_to_i64".to_string(), builtin_i32_to_i64);
        self.builtins.insert("i64_to_i32".to_string(), builtin_i64_to_i32);

        // v0.34.2: Memory allocation for Phase 34.2 Dynamic Collections
        self.builtins.insert("malloc".to_string(), builtin_malloc);
        self.builtins.insert("free".to_string(), builtin_free);
        self.builtins.insert("realloc".to_string(), builtin_realloc);
        self.builtins.insert("calloc".to_string(), builtin_calloc);
        self.builtins.insert("store_i64".to_string(), builtin_store_i64);
        self.builtins.insert("load_i64".to_string(), builtin_load_i64);
        // v0.51.5: f64 memory operations
        self.builtins.insert("store_f64".to_string(), builtin_store_f64);
        self.builtins.insert("load_f64".to_string(), builtin_load_f64);
        // Box convenience functions
        self.builtins.insert("box_new_i64".to_string(), builtin_box_new_i64);
        self.builtins.insert("box_get_i64".to_string(), builtin_load_i64); // alias
        self.builtins.insert("box_set_i64".to_string(), builtin_store_i64); // alias
        self.builtins.insert("box_free_i64".to_string(), builtin_free); // alias

        // v0.34.2.3: Vec<i64> dynamic array builtins (RFC-0007)
        self.builtins.insert("vec_new".to_string(), builtin_vec_new);
        self.builtins.insert("vec_with_capacity".to_string(), builtin_vec_with_capacity);
        self.builtins.insert("vec_push".to_string(), builtin_vec_push);
        self.builtins.insert("vec_pop".to_string(), builtin_vec_pop);
        self.builtins.insert("vec_get".to_string(), builtin_vec_get);
        self.builtins.insert("vec_set".to_string(), builtin_vec_set);
        self.builtins.insert("vec_len".to_string(), builtin_vec_len);
        self.builtins.insert("vec_cap".to_string(), builtin_vec_cap);
        self.builtins.insert("vec_free".to_string(), builtin_vec_free);
        self.builtins.insert("vec_clear".to_string(), builtin_vec_clear);

        // v0.34.24: Hash builtins
        self.builtins.insert("hash_i64".to_string(), builtin_hash_i64);

        // v0.34.24: HashMap builtins
        self.builtins.insert("hashmap_new".to_string(), builtin_hashmap_new);
        self.builtins
            .insert("hashmap_insert".to_string(), builtin_hashmap_insert);
        self.builtins
            .insert("hashmap_get".to_string(), builtin_hashmap_get);
        self.builtins
            .insert("hashmap_contains".to_string(), builtin_hashmap_contains);
        self.builtins
            .insert("hashmap_remove".to_string(), builtin_hashmap_remove);
        self.builtins
            .insert("hashmap_len".to_string(), builtin_hashmap_len);
        self.builtins
            .insert("hashmap_free".to_string(), builtin_hashmap_free);

        // v0.34.24: HashSet builtins
        self.builtins
            .insert("hashset_new".to_string(), builtin_hashset_new);
        self.builtins
            .insert("hashset_insert".to_string(), builtin_hashset_insert);
        self.builtins
            .insert("hashset_contains".to_string(), builtin_hashset_contains);
        self.builtins
            .insert("hashset_remove".to_string(), builtin_hashset_remove);
        self.builtins
            .insert("hashset_len".to_string(), builtin_hashset_len);
        self.builtins
            .insert("hashset_free".to_string(), builtin_hashset_free);
    }

    /// v0.30.280: Enable ScopeStack-based evaluation for better memory efficiency
    pub fn enable_scope_stack(&mut self) {
        self.use_scope_stack = true;
        self.scope_stack.reset();
    }

    /// v0.30.280: Disable ScopeStack-based evaluation
    pub fn disable_scope_stack(&mut self) {
        self.use_scope_stack = false;
    }

    /// Load a program (register functions, structs, enums)
    pub fn load(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                crate::ast::Item::FnDef(fn_def) => {
                    self.functions
                        .insert(fn_def.name.node.clone(), fn_def.clone());
                }
                crate::ast::Item::StructDef(struct_def) => {
                    self.struct_defs
                        .insert(struct_def.name.node.clone(), struct_def.clone());
                }
                crate::ast::Item::EnumDef(enum_def) => {
                    self.enum_defs
                        .insert(enum_def.name.node.clone(), enum_def.clone());
                }
                // v0.5 Phase 4: Use statements are processed at module resolution time
                crate::ast::Item::Use(_) => {}
                // v0.13.0: Extern functions are handled at compile time (FFI)
                crate::ast::Item::ExternFn(_) => {}
                // v0.20.1: Trait definitions (signatures only)
                crate::ast::Item::TraitDef(_) => {}
                // v0.90.40: Register impl block methods for trait dispatch
                crate::ast::Item::ImplBlock(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        crate::ast::Type::Named(name) => name.clone(),
                        crate::ast::Type::Generic { name, .. } => name.clone(),
                        other => format!("{}", other),
                    };
                    for method in &impl_block.methods {
                        self.impl_methods.insert(
                            (type_name.clone(), method.name.node.clone()),
                            method.clone(),
                        );
                    }
                }
                // v0.50.6: Type aliases are resolved at compile time
                crate::ast::Item::TypeAlias(_) => {}
            }
        }
    }

    /// Run a program (find and call main)
    pub fn run(&mut self, program: &Program) -> InterpResult<Value> {
        self.load(program);

        // Look for a main function or evaluate the last function
        if let Some(main_fn) = self.functions.get("main").cloned() {
            self.call_function(&main_fn, &[])
        } else if let Some(last_item) = program.items.last() {
            match last_item {
                crate::ast::Item::FnDef(fn_def) => {
                    // If no main, just evaluate the body of the last function
                    // (for simple scripts without main)
                    self.call_function(fn_def, &[])
                }
                crate::ast::Item::StructDef(_) | crate::ast::Item::EnumDef(_) => {
                    // Struct/Enum definitions don't produce values
                    Ok(Value::Unit)
                }
                // v0.5 Phase 4: Use statements don't produce values
                crate::ast::Item::Use(_) => Ok(Value::Unit),
                // v0.13.0: Extern functions don't produce values (FFI declarations)
                crate::ast::Item::ExternFn(_) => Ok(Value::Unit),
                // v0.20.1: Trait system doesn't produce values
                crate::ast::Item::TraitDef(_) | crate::ast::Item::ImplBlock(_) => Ok(Value::Unit),
                // v0.50.6: Type aliases don't produce values
                crate::ast::Item::TypeAlias(_) => Ok(Value::Unit),
            }
        } else {
            Ok(Value::Unit)
        }
    }

    /// Evaluate a single expression (for REPL)
    pub fn eval_expr(&mut self, expr: &Spanned<Expr>) -> InterpResult<Value> {
        self.eval(expr, &self.global_env.clone())
    }

    /// Get list of test function names (functions starting with "test_")
    pub fn get_test_functions(&self) -> Vec<String> {
        self.functions
            .keys()
            .filter(|name| name.starts_with("test_"))
            .cloned()
            .collect()
    }

    /// Run a single function by name (for testing)
    pub fn run_function(&mut self, name: &str) -> InterpResult<Value> {
        if let Some(fn_def) = self.functions.get(name).cloned() {
            self.call_function(&fn_def, &[])
        } else {
            Err(RuntimeError::undefined_variable(name))
        }
    }

    /// Call a function by name with arguments (v0.30.246: Stage 3 verification support)
    pub fn call_function_with_args(&mut self, name: &str, args: Vec<Value>) -> InterpResult<Value> {
        // Check builtins first
        if let Some(builtin) = self.builtins.get(name) {
            return builtin(&args);
        }

        // Then user-defined functions
        if let Some(fn_def) = self.functions.get(name).cloned() {
            // v0.30.280: Use ScopeStack fast path when enabled
            if self.use_scope_stack {
                return self.call_function_fast(&fn_def, &args);
            }
            return self.call_function(&fn_def, &args);
        }

        Err(RuntimeError::undefined_function(name))
    }

    /// Evaluate an expression with automatic stack growth for deep recursion
    fn eval(&mut self, expr: &Spanned<Expr>, env: &EnvRef) -> InterpResult<Value> {
        // Grow stack if we're running low
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROW_SIZE, || self.eval_inner(expr, env))
    }

    /// Inner eval implementation
    fn eval_inner(&mut self, expr: &Spanned<Expr>, env: &EnvRef) -> InterpResult<Value> {
        match &expr.node {
            Expr::IntLit(n) => Ok(Value::Int(*n)),
            Expr::FloatLit(f) => Ok(Value::Float(*f)),
            Expr::BoolLit(b) => Ok(Value::Bool(*b)),
            Expr::StringLit(s) => Ok(Value::Str(self.intern_string(s))),
            // v0.64: Character literal evaluation
            Expr::CharLit(c) => Ok(Value::Char(*c)),
            Expr::Unit => Ok(Value::Unit),

            // v0.51.40: Null pointer literal - interpreted as 0
            Expr::Null => Ok(Value::Int(0)),

            // v0.51.41: Sizeof - return size in bytes
            Expr::Sizeof { ty } => {
                let size = self.compute_type_size(&ty.node);
                Ok(Value::Int(size))
            }

            // v0.70: Spawn expression - not supported in interpreter (requires native threads)
            Expr::Spawn { .. } => {
                Err(RuntimeError::todo("spawn expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }

            // v0.72: Atomic creation - not supported in interpreter (requires native)
            Expr::AtomicNew { .. } => {
                Err(RuntimeError::todo("atomic expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }

            // v0.71: Mutex creation - not supported in interpreter (requires native threads)
            Expr::MutexNew { .. } => {
                Err(RuntimeError::todo("mutex expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }

            // v0.73: Channel creation - not supported in interpreter (requires native threads)
            Expr::ChannelNew { .. } => {
                Err(RuntimeError::todo("channel expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }

            // v0.74: RwLock, Barrier, Condvar - not supported in interpreter
            Expr::RwLockNew { .. } => {
                Err(RuntimeError::todo("RwLock expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }
            Expr::BarrierNew { .. } => {
                Err(RuntimeError::todo("Barrier expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }
            Expr::CondvarNew => {
                Err(RuntimeError::todo("Condvar expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }

            // v0.75: Await expression - not supported in interpreter
            Expr::Await { .. } => {
                Err(RuntimeError::todo("await expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }

            // v0.82: Select expression - not supported in interpreter
            Expr::Select { .. } => {
                Err(RuntimeError::todo("select expressions require native compilation; use 'bmb build' instead of 'bmb run'"))
            }

            Expr::Var(name) => {
                env.borrow()
                    .get(name)
                    .ok_or_else(|| RuntimeError::undefined_variable(name))
            }

            Expr::Binary { left, op, right } => {
                // Short-circuit evaluation for logical operators
                match op {
                    BinOp::And => {
                        let lval = self.eval(left, env)?;
                        if !lval.is_truthy() {
                            return Ok(Value::Bool(false));
                        }
                        let rval = self.eval(right, env)?;
                        Ok(Value::Bool(rval.is_truthy()))
                    }
                    BinOp::Or => {
                        let lval = self.eval(left, env)?;
                        if lval.is_truthy() {
                            return Ok(Value::Bool(true));
                        }
                        let rval = self.eval(right, env)?;
                        Ok(Value::Bool(rval.is_truthy()))
                    }
                    _ => {
                        let lval = self.eval(left, env)?;
                        let rval = self.eval(right, env)?;
                        self.eval_binary(*op, lval, rval)
                    }
                }
            }

            Expr::Unary { op, expr: inner } => {
                let val = self.eval(inner, env)?;
                self.eval_unary(*op, val)
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval(cond, env)?;
                if cond_val.is_truthy() {
                    self.eval(then_branch, env)
                } else {
                    self.eval(else_branch, env)
                }
            }

            // v0.34.2.3: Define in current scope (Block manages scope)
            // Fix: Don't create child env for Let - let Block handle scoping
            // This allows sequential let statements to share variables
            Expr::Let {
                name,
                mutable: _,
                ty: _,
                value,
                body,
            } => {
                let val = self.eval(value, env)?;
                env.borrow_mut().define(name.clone(), val);
                self.eval(body, env)
            }

            Expr::Assign { name, value } => {
                let val = self.eval(value, env)?;
                if !env.borrow_mut().set(name, val.clone()) {
                    return Err(RuntimeError::undefined_variable(name));
                }
                Ok(Value::Unit)
            }

            // v0.60.21: Uninitialized let binding for stack arrays
            // In interpreter, create array with default values
            Expr::LetUninit { name, mutable: _, ty, body } => {
                // Create uninitialized array based on type
                let val = match &ty.node {
                    crate::ast::Type::Array(_, size) => {
                        // Create array filled with zeros
                        Value::Array(vec![Value::Int(0); *size])
                    }
                    _ => Value::Unit, // Shouldn't happen - type checker ensures array type
                };
                env.borrow_mut().define(name.clone(), val);
                self.eval(body, env)
            }

            // v0.37: Invariant is for SMT verification, not runtime
            Expr::While { cond, invariant: _, body } => {
                while self.eval(cond, env)?.is_truthy() {
                    match self.eval(body, env) {
                        Ok(_) => {},
                        Err(e) if matches!(e.kind, ErrorKind::Continue) => continue,
                        Err(e) if matches!(e.kind, ErrorKind::Break(_)) => break,
                        Err(e) => return Err(e),
                    }
                }
                Ok(Value::Unit)
            }

            // v0.2: Range expression with kind
            Expr::Range { start, end, kind } => {
                let start_val = self.eval(start, env)?;
                let end_val = self.eval(end, env)?;
                match (&start_val, &end_val) {
                    (Value::Int(s), Value::Int(e)) => {
                        // For inclusive range (..=), add 1 to end for iteration purposes
                        let effective_end = match kind {
                            crate::ast::RangeKind::Inclusive => *e + 1,
                            crate::ast::RangeKind::Exclusive => *e,
                        };
                        Ok(Value::Range(*s, effective_end))
                    }
                    _ => Err(RuntimeError::type_error(
                        "integer",
                        &format!("{} {} {}", start_val.type_name(), kind, end_val.type_name()),
                    )),
                }
            }

            // v0.5 Phase 3: For loop
            Expr::For { var, iter, body } => {
                let iter_val = self.eval(iter, env)?;
                match iter_val {
                    Value::Range(start, end) => {
                        let child = child_env(env);
                        for i in start..end {
                            child.borrow_mut().define(var.clone(), Value::Int(i));
                            match self.eval(body, &child) {
                                Ok(_) => {},
                                Err(e) if matches!(e.kind, ErrorKind::Continue) => continue,
                                Err(e) if matches!(e.kind, ErrorKind::Break(_)) => break,
                                Err(e) => return Err(e),
                            }
                        }
                        Ok(Value::Unit)
                    }
                    // v0.90.33: Array iteration
                    Value::Array(elements) => {
                        let child = child_env(env);
                        for elem in elements {
                            child.borrow_mut().define(var.clone(), elem);
                            match self.eval(body, &child) {
                                Ok(_) => {},
                                Err(e) if matches!(e.kind, ErrorKind::Continue) => continue,
                                Err(e) if matches!(e.kind, ErrorKind::Break(_)) => break,
                                Err(e) => return Err(e),
                            }
                        }
                        Ok(Value::Unit)
                    }
                    _ => Err(RuntimeError::type_error("Range or Array", iter_val.type_name())),
                }
            }

            Expr::Call { func, args } => {
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval(a, env))
                    .collect::<InterpResult<Vec<_>>>()?;

                // v0.92: Check if func resolves to a closure value
                if let Some(Value::Closure { params, body, env: captured_env }) = env.borrow().get(func) {
                    return self.call_closure(&params, &body, &captured_env, arg_vals);
                }

                self.call(func, arg_vals)
            }

            Expr::Block(exprs) => {
                let child = child_env(env);
                let mut result = Value::Unit;
                for e in exprs {
                    result = self.eval(e, &child)?;
                }
                Ok(result)
            }

            Expr::Ret => {
                // Ret should only appear in post conditions, not in regular evaluation
                Err(RuntimeError::type_error("value", "ret"))
            }

            Expr::StructInit { name, fields } => {
                let mut field_values = HashMap::new();
                for (field_name, field_expr) in fields {
                    let val = self.eval(field_expr, env)?;
                    field_values.insert(field_name.node.clone(), val);
                }
                Ok(Value::Struct(name.clone(), field_values))
            }

            Expr::FieldAccess { expr: obj_expr, field } => {
                // Interpret the object expression
                let obj = { let o = obj_expr; self.eval(o, env)? };
                match obj {
                    Value::Struct(_, fields) => {
                        fields.get(&field.node).cloned()
                            .ok_or_else(|| RuntimeError::type_error("field", &field.node))
                    }
                    // v0.51.41: Typed pointer field access - look up struct in heap
                    Value::Int(ptr) if ptr != 0 => {
                        let heap = self.heap.borrow();
                        if let Some(Value::Struct(_, fields)) = heap.get(&ptr) {
                            fields.get(&field.node).cloned()
                                .ok_or_else(|| RuntimeError::type_error("field", &field.node))
                        } else {
                            Err(RuntimeError::type_error("struct at pointer", "uninitialized"))
                        }
                    }
                    _ => Err(RuntimeError::type_error("struct", obj.type_name())),
                }
            }

            // v0.51.23: Field assignment (v0.51.41: with typed pointer support)
            Expr::FieldAssign { object, field, value } => {
                // Get the new value first
                let new_val = { let v = value; self.eval(v, env)? };

                // The object must be a variable for field assignment to work
                if let Expr::Var(var_name) = &object.node {
                    // Get the value from environment
                    let obj_val = env.borrow().get(var_name)
                        .ok_or_else(|| RuntimeError::undefined_variable(var_name))?;

                    // Modify the field
                    match obj_val {
                        Value::Struct(struct_name, mut fields) => {
                            if !fields.contains_key(&field.node) {
                                return Err(RuntimeError::type_error("field", &field.node));
                            }
                            fields.insert(field.node.clone(), new_val);
                            // Store the modified struct back
                            env.borrow_mut().set(var_name, Value::Struct(struct_name, fields));
                            Ok(Value::Unit)
                        }
                        // v0.51.41: Typed pointer field assignment - store in heap
                        Value::Int(ptr) if ptr != 0 => {
                            let mut heap = self.heap.borrow_mut();
                            let entry = heap.entry(ptr).or_insert_with(|| {
                                Value::Struct("_heap_struct".to_string(), HashMap::new())
                            });
                            if let Value::Struct(_, fields) = entry {
                                fields.insert(field.node.clone(), new_val);
                            }
                            Ok(Value::Unit)
                        }
                        _ => Err(RuntimeError::type_error("struct", obj_val.type_name())),
                    }
                } else {
                    // For complex expressions, we need to recursively handle the assignment
                    // For now, return an error for unsupported cases
                    Err(RuntimeError::type_error("variable", "complex expression"))
                }
            }

            // v0.60.21: Dereference assignment: *ptr = value
            Expr::DerefAssign { ptr, value } => {
                let ptr_val = self.eval(ptr, env)?;
                let new_val = self.eval(value, env)?;

                match ptr_val {
                    Value::Int(addr) if addr != 0 => {
                        // Store value at the pointer address
                        self.heap.borrow_mut().insert(addr, new_val);
                        Ok(Value::Unit)
                    }
                    Value::Int(0) => Err(RuntimeError::type_error("non-null pointer", "null")),
                    _ => Err(RuntimeError::type_error("pointer", ptr_val.type_name())),
                }
            }

            // v0.43: Tuple field access
            Expr::TupleField { expr: tuple_expr, index } => {
                let tuple_val = self.eval(tuple_expr, env)?;
                match tuple_val {
                    Value::Tuple(elems) => {
                        elems.get(*index).cloned()
                            .ok_or_else(|| RuntimeError::index_out_of_bounds(*index as i64, elems.len()))
                    }
                    _ => Err(RuntimeError::type_error("tuple", tuple_val.type_name())),
                }
            }

            Expr::EnumVariant { enum_name, variant, args } => {
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval(a, env))
                    .collect::<InterpResult<Vec<_>>>()?;
                Ok(Value::Enum(enum_name.clone(), variant.clone(), arg_vals))
            }

            Expr::Match { expr: match_expr, arms } => {
                let val = self.eval(match_expr, env)?;

                for arm in arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern.node, &val) {
                        let child = child_env(env);
                        for (name, bound_val) in bindings {
                            child.borrow_mut().define(name, bound_val);
                        }
                        // v0.40: Check pattern guard if present
                        if let Some(guard) = &arm.guard {
                            let guard_result = self.eval(guard, &child)?;
                            if !guard_result.is_truthy() {
                                continue; // Guard failed, try next arm
                            }
                        }
                        return self.eval(&arm.body, &child);
                    }
                }

                Err(RuntimeError::type_error("matching arm", "no match found"))
            }

            // v0.5 Phase 5: References
            Expr::Ref(inner) => {
                let val = self.eval(inner, env)?;
                Ok(Value::Ref(std::rc::Rc::new(std::cell::RefCell::new(val))))
            }

            Expr::RefMut(inner) => {
                let val = self.eval(inner, env)?;
                Ok(Value::Ref(std::rc::Rc::new(std::cell::RefCell::new(val))))
            }

            Expr::Deref(inner) => {
                let val = self.eval(inner, env)?;
                match val {
                    Value::Ref(r) => Ok(r.borrow().clone()),
                    _ => Err(RuntimeError::type_error("reference", val.type_name())),
                }
            }

            // v0.5 Phase 6: Arrays
            Expr::ArrayLit(elems) => {
                let mut values = Vec::new();
                for elem in elems {
                    values.push(self.eval(elem, env)?);
                }
                Ok(Value::Array(values))
            }

            // v0.60.22: Array repeat [val; N]
            Expr::ArrayRepeat { value, count } => {
                let val = self.eval(value, env)?;
                let values = vec![val; *count];
                Ok(Value::Array(values))
            }

            // v0.42: Tuple expressions
            Expr::Tuple(elems) => {
                let mut values = Vec::new();
                for elem in elems {
                    values.push(self.eval(elem, env)?);
                }
                Ok(Value::Tuple(values))
            }

            Expr::Index { expr, index } => {
                let arr_val = self.eval(expr, env)?;
                let idx_val = self.eval(index, env)?;

                let idx = match idx_val {
                    Value::Int(n) => n as usize,
                    _ => return Err(RuntimeError::type_error("integer", idx_val.type_name())),
                };

                // v0.50.26: Dereference if indexing through a reference
                let derefed_val = match &arr_val {
                    Value::Ref(r) => r.borrow().clone(),
                    _ => arr_val,
                };

                match derefed_val {
                    Value::Array(arr) => {
                        if idx < arr.len() {
                            Ok(arr[idx].clone())
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, arr.len()))
                        }
                    }
                    Value::Str(s) => {
                        if idx < s.len() {
                            Ok(Value::Int(s.as_bytes()[idx] as i64))
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, s.len()))
                        }
                    }
                    // v0.93: Handle StringRope (lazy concatenated strings)
                    Value::StringRope(_) => {
                        let s = derefed_val.materialize_string()
                            .ok_or_else(|| RuntimeError::type_error("string", "invalid StringRope"))?;
                        if idx < s.len() {
                            Ok(Value::Int(s.as_bytes()[idx] as i64))
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, s.len()))
                        }
                    }
                    // v0.89.5: Typed pointer indexing (*i64) — ptr[i] does load_i64(ptr + i*8)
                    Value::Int(ptr) => {
                        if ptr == 0 {
                            return Err(RuntimeError::io_error("index: null pointer dereference"));
                        }
                        let addr = ptr + (idx as i64) * 8;
                        let value = unsafe {
                            let p = addr as *const i64;
                            *p
                        };
                        Ok(Value::Int(value))
                    }
                    _ => Err(RuntimeError::type_error("array, string, or pointer", derefed_val.type_name())),
                }
            }

            // v0.51: Index assignment: arr[i] = value
            Expr::IndexAssign { array, index, value } => {
                // Get the array identifier name for env lookup
                let arr_name = match &array.node {
                    Expr::Var(name) => name.clone(),
                    _ => return Err(RuntimeError::type_error("variable", "complex expression")),
                };

                let idx_val = self.eval(index, env)?;
                let new_val = self.eval(value, env)?;

                let idx = match idx_val {
                    Value::Int(n) => n as usize,
                    _ => return Err(RuntimeError::type_error("integer", idx_val.type_name())),
                };

                // v0.89.5: Check if this is a typed pointer (Value::Int) for ptr[i] = value
                // v0.90.24: Fix RefCell borrow conflict — bind lookup result so the
                // Ref<Env> temporary is dropped before entering the match body.
                // Previously, `match env.borrow().get(...)` kept the Ref alive for the
                // entire match, conflicting with `env.borrow_mut()` in the Array arm.
                let lookup = env.borrow().get(&arr_name);
                match lookup {
                    Some(Value::Int(ptr)) => {
                        if ptr == 0 {
                            return Err(RuntimeError::io_error("index assign: null pointer dereference"));
                        }
                        let write_val = match new_val {
                            Value::Int(v) => v,
                            _ => return Err(RuntimeError::type_error("i64", new_val.type_name())),
                        };
                        let addr = ptr + (idx as i64) * 8;
                        unsafe {
                            let p = addr as *mut i64;
                            *p = write_val;
                        }
                        Ok(Value::Unit)
                    }
                    Some(Value::Array(a)) => {
                        let mut arr = a;
                        if idx < arr.len() {
                            arr[idx] = new_val;
                            env.borrow_mut().set(&arr_name, Value::Array(arr));
                            Ok(Value::Unit)
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, arr.len()))
                        }
                    }
                    Some(other) => Err(RuntimeError::type_error("array or pointer", other.type_name())),
                    None => Err(RuntimeError::undefined_variable(&arr_name)),
                }
            }

            // v0.5 Phase 8: Method calls
            Expr::MethodCall { receiver, method, args } => {
                let recv_val = self.eval(receiver, env)?;
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval(a, env))
                    .collect::<InterpResult<Vec<_>>>()?;
                self.eval_method_call(recv_val, method, arg_vals)
            }

            // v0.2: State references (only valid in contracts, not runtime)
            Expr::StateRef { .. } => {
                Err(RuntimeError::type_error(
                    "contract expression",
                    "runtime expression (.pre/.post only valid in contracts)"
                ))
            }

            // v0.2: Refinement self-reference (only valid in refinement constraints)
            Expr::It => {
                Err(RuntimeError::type_error(
                    "refinement constraint",
                    "runtime expression ('it' only valid in type refinements)"
                ))
            }


            // v0.20.0 / v0.92: Closure expressions with proper environment capture
            Expr::Closure { params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.node.clone()).collect();
                Ok(Value::Closure {
                    params: param_names,
                    body: Box::new((**body).clone()),
                    env: Rc::clone(env),
                })
            }

            // v0.31: Todo expression - panics at runtime
            Expr::Todo { message } => {
                let msg = message.as_deref().unwrap_or("not yet implemented");
                Err(RuntimeError::todo(msg))
            }

            // v0.36: Additional control flow
            // Loop - infinite loop, exits only via break
            Expr::Loop { body } => {
                loop {
                    match self.eval(body, env) {
                        Ok(_) => continue,
                        Err(e) if matches!(e.kind, ErrorKind::Continue) => continue,
                        Err(e) if matches!(e.kind, ErrorKind::Break(_)) => {
                            if let ErrorKind::Break(val) = e.kind {
                                return Ok(val.map(|v| *v).unwrap_or(Value::Unit));
                            }
                            return Ok(Value::Unit);
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            Expr::Break { value } => {
                let val = match value {
                    Some(v) => Some(Box::new(self.eval(v, env)?)),
                    None => None,
                };
                Err(RuntimeError { kind: ErrorKind::Break(val), message: "break".to_string() })
            }

            Expr::Continue => {
                Err(RuntimeError { kind: ErrorKind::Continue, message: "continue".to_string() })
            }

            Expr::Return { value } => {
                let val = match value {
                    Some(v) => self.eval(v, env)?,
                    None => Value::Unit,
                };
                Err(RuntimeError { kind: ErrorKind::Return(Box::new(val)), message: "return".to_string() })
            }

            // v0.37: Quantifiers (verification-only, cannot be executed at runtime)
            Expr::Forall { .. } => {
                Err(RuntimeError::type_error(
                    "compile-time verification",
                    "forall expressions are for SMT verification only and cannot be evaluated at runtime"
                ))
            }
            Expr::Exists { .. } => {
                Err(RuntimeError::type_error(
                    "compile-time verification",
                    "exists expressions are for SMT verification only and cannot be evaluated at runtime"
                ))
            }

            // v0.39: Type cast
            Expr::Cast { expr, ty } => {
                let val = self.eval(expr, env)?;
                self.eval_cast(val, &ty.node)
            }
        }
    }

    /// Evaluate method call (v0.5 Phase 8, v0.30.283: StringRope support, v0.90.40: trait dispatch)
    fn eval_method_call(&mut self, receiver: Value, method: &str, args: Vec<Value>) -> InterpResult<Value> {
        match receiver {
            // v0.30.283: Handle StringRope by materializing
            Value::StringRope(_) => {
                let materialized = receiver.materialize_string()
                    .ok_or_else(|| RuntimeError::type_error("string", "invalid StringRope"))?;
                let s = Rc::new(materialized);
                self.eval_method_call(Value::Str(s), method, args)
            }
            // v0.90.34: Float methods
            Value::Float(f) => {
                match method {
                    "abs" => Ok(Value::Float(f.abs())),
                    "floor" => Ok(Value::Float(f.floor())),
                    "ceil" => Ok(Value::Float(f.ceil())),
                    "round" => Ok(Value::Float(f.round())),
                    "sqrt" => Ok(Value::Float(f.sqrt())),
                    "is_nan" => Ok(Value::Bool(f.is_nan())),
                    "is_infinite" => Ok(Value::Bool(f.is_infinite())),
                    "is_finite" => Ok(Value::Bool(f.is_finite())),
                    "min" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("min", 1, args.len()));
                        }
                        let other = match &args[0] {
                            Value::Float(n) => *n,
                            _ => return Err(RuntimeError::type_error("f64", args[0].type_name())),
                        };
                        Ok(Value::Float(f.min(other)))
                    }
                    "max" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("max", 1, args.len()));
                        }
                        let other = match &args[0] {
                            Value::Float(n) => *n,
                            _ => return Err(RuntimeError::type_error("f64", args[0].type_name())),
                        };
                        Ok(Value::Float(f.max(other)))
                    }
                    "to_int" => Ok(Value::Int(f as i64)),
                    "to_string" => Ok(Value::Str(Rc::new(f.to_string()))),
                    // v0.90.42: Math functions
                    "sin" => Ok(Value::Float(f.sin())),
                    "cos" => Ok(Value::Float(f.cos())),
                    "tan" => Ok(Value::Float(f.tan())),
                    "log" => Ok(Value::Float(f.ln())),
                    "log2" => Ok(Value::Float(f.log2())),
                    "log10" => Ok(Value::Float(f.log10())),
                    "exp" => Ok(Value::Float(f.exp())),
                    "sign" => Ok(Value::Float(if f > 0.0 { 1.0 } else if f < 0.0 { -1.0 } else { 0.0 })),
                    "is_positive" => Ok(Value::Bool(f > 0.0)),
                    "is_negative" => Ok(Value::Bool(f < 0.0)),
                    "is_zero" => Ok(Value::Bool(f == 0.0)),
                    // v0.90.47: trunc, fract, powi, powf
                    "trunc" => Ok(Value::Float(f.trunc())),
                    "fract" => Ok(Value::Float(f.fract())),
                    "powi" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("powi", 1, args.len()));
                        }
                        let exp = match &args[0] {
                            Value::Int(n) => *n as i32,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        Ok(Value::Float(f.powi(exp)))
                    }
                    "powf" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("powf", 1, args.len()));
                        }
                        let exp = match &args[0] {
                            Value::Float(e) => *e,
                            _ => return Err(RuntimeError::type_error("f64", args[0].type_name())),
                        };
                        Ok(Value::Float(f.powf(exp)))
                    }
                    _ => Err(RuntimeError::undefined_function(&format!("f64.{}", method))),
                }
            }
            Value::Str(s) => {
                match method {
                    "len" => Ok(Value::Int(s.len() as i64)),
                    // v0.67: Renamed from char_at for clarity (returns byte, not Unicode char)
                    "byte_at" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("byte_at", 1, args.len()));
                        }
                        let idx = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        if idx < s.len() {
                            Ok(Value::Int(s.as_bytes()[idx] as i64))
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, s.len()))
                        }
                    }
                    "slice" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("slice", 2, args.len()));
                        }
                        let start = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let end = match &args[1] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[1].type_name())),
                        };
                        if start > s.len() || end > s.len() || start > end {
                            return Err(RuntimeError::index_out_of_bounds(end as i64, s.len()));
                        }
                        Ok(Value::Str(Rc::new(s[start..end].to_string())))
                    }
                    "is_empty" => Ok(Value::Bool(s.is_empty())),
                    // v0.90.32: String methods
                    "to_upper" => Ok(Value::Str(Rc::new(s.to_uppercase()))),
                    "to_lower" => Ok(Value::Str(Rc::new(s.to_lowercase()))),
                    "trim" => Ok(Value::Str(Rc::new(s.trim().to_string()))),
                    "contains" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("contains", 1, args.len()));
                        }
                        let substr = match &args[0] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[0].type_name())),
                        };
                        Ok(Value::Bool(s.contains(&substr)))
                    }
                    "starts_with" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("starts_with", 1, args.len()));
                        }
                        let prefix = match &args[0] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[0].type_name())),
                        };
                        Ok(Value::Bool(s.starts_with(&prefix)))
                    }
                    "ends_with" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("ends_with", 1, args.len()));
                        }
                        let suffix = match &args[0] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[0].type_name())),
                        };
                        Ok(Value::Bool(s.ends_with(&suffix)))
                    }
                    "replace" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("replace", 2, args.len()));
                        }
                        let from = match &args[0] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[0].type_name())),
                        };
                        let to = match &args[1] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[1].type_name())),
                        };
                        Ok(Value::Str(Rc::new(s.replace(&from, &to))))
                    }
                    "repeat" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("repeat", 1, args.len()));
                        }
                        let count = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        Ok(Value::Str(Rc::new(s.repeat(count))))
                    }
                    "split" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("split", 1, args.len()));
                        }
                        let delimiter = match &args[0] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[0].type_name())),
                        };
                        let parts: Vec<Value> = s.split(&delimiter)
                            .map(|part| Value::Str(Rc::new(part.to_string())))
                            .collect();
                        Ok(Value::Array(parts))
                    }
                    "index_of" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("index_of", 1, args.len()));
                        }
                        let substr = match &args[0] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[0].type_name())),
                        };
                        match s.find(&substr) {
                            Some(idx) => Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![Value::Int(idx as i64)])),
                            None => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                        }
                    }
                    // v0.90.36: String parsing and utility methods
                    "to_int" => {
                        match s.parse::<i64>() {
                            Ok(n) => Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![Value::Int(n)])),
                            Err(_) => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                        }
                    }
                    "to_float" => {
                        match s.parse::<f64>() {
                            Ok(f) => Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![Value::Float(f)])),
                            Err(_) => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                        }
                    }
                    "chars" => {
                        let chars: Vec<Value> = s.chars()
                            .map(|c| Value::Str(Rc::new(c.to_string())))
                            .collect();
                        Ok(Value::Array(chars))
                    }
                    "reverse" => {
                        Ok(Value::Str(Rc::new(s.chars().rev().collect::<String>())))
                    }
                    // v0.90.41: lines() -> [String]
                    "lines" => {
                        let lines: Vec<Value> = s.lines()
                            .map(|l| Value::Str(Rc::new(l.to_string())))
                            .collect();
                        Ok(Value::Array(lines))
                    }
                    // v0.90.41: bytes() -> [i64]
                    "bytes" => {
                        let bytes: Vec<Value> = s.bytes()
                            .map(|b| Value::Int(b as i64))
                            .collect();
                        Ok(Value::Array(bytes))
                    }
                    // v0.90.41: char_at(i64) -> String
                    "char_at" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("char_at", 1, args.len()));
                        }
                        let idx = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        match s.chars().nth(idx) {
                            Some(c) => Ok(Value::Str(Rc::new(c.to_string()))),
                            None => Err(RuntimeError::index_out_of_bounds(idx as i64, s.len())),
                        }
                    }
                    // v0.90.41: strip_prefix(String) -> String?
                    "strip_prefix" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("strip_prefix", 1, args.len()));
                        }
                        let prefix = match &args[0] {
                            Value::Str(p) => p.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("String", args[0].type_name())),
                        };
                        match s.strip_prefix(prefix.as_str()) {
                            Some(rest) => Ok(Value::Enum("Option".to_string(), "Some".to_string(),
                                vec![Value::Str(Rc::new(rest.to_string()))])),
                            None => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                        }
                    }
                    // v0.90.41: strip_suffix(String) -> String?
                    "strip_suffix" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("strip_suffix", 1, args.len()));
                        }
                        let suffix = match &args[0] {
                            Value::Str(p) => p.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("String", args[0].type_name())),
                        };
                        match s.strip_suffix(suffix.as_str()) {
                            Some(rest) => Ok(Value::Enum("Option".to_string(), "Some".to_string(),
                                vec![Value::Str(Rc::new(rest.to_string()))])),
                            None => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                        }
                    }
                    // v0.90.41: pad_left(width, padding) -> String
                    "pad_left" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("pad_left", 2, args.len()));
                        }
                        let width = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let pad = match &args[1] {
                            Value::Str(p) => p.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("String", args[1].type_name())),
                        };
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        let current_len = s.chars().count();
                        if current_len >= width {
                            Ok(Value::Str(Rc::new(s.to_string())))
                        } else {
                            let padding: String = std::iter::repeat(pad_char).take(width - current_len).collect();
                            Ok(Value::Str(Rc::new(format!("{}{}", padding, s))))
                        }
                    }
                    // v0.90.41: pad_right(width, padding) -> String
                    "pad_right" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("pad_right", 2, args.len()));
                        }
                        let width = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let pad = match &args[1] {
                            Value::Str(p) => p.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("String", args[1].type_name())),
                        };
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        let current_len = s.chars().count();
                        if current_len >= width {
                            Ok(Value::Str(Rc::new(s.to_string())))
                        } else {
                            let padding: String = std::iter::repeat(pad_char).take(width - current_len).collect();
                            Ok(Value::Str(Rc::new(format!("{}{}", s, padding))))
                        }
                    }
                    // v0.90.44: trim_start() -> String
                    "trim_start" => Ok(Value::Str(Rc::new(s.trim_start().to_string()))),
                    // v0.90.44: trim_end() -> String
                    "trim_end" => Ok(Value::Str(Rc::new(s.trim_end().to_string()))),
                    // v0.90.44: char_count() -> i64
                    "char_count" => Ok(Value::Int(s.chars().count() as i64)),
                    // v0.90.44: count(substring) -> i64
                    "count" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("count", 1, args.len()));
                        }
                        let needle = match &args[0] {
                            Value::Str(n) => n.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("String", args[0].type_name())),
                        };
                        Ok(Value::Int(s.matches(&needle).count() as i64))
                    }
                    // v0.90.44: last_index_of(substring) -> i64?
                    "last_index_of" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("last_index_of", 1, args.len()));
                        }
                        let needle = match &args[0] {
                            Value::Str(n) => n.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("String", args[0].type_name())),
                        };
                        match s.rfind(&needle) {
                            Some(idx) => Ok(Value::Int(idx as i64)),
                            None => Ok(Value::Int(0)), // null
                        }
                    }
                    // v0.90.44: insert(index, substring) -> String
                    "insert" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("insert", 2, args.len()));
                        }
                        let idx = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let sub = match &args[1] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("String", args[1].type_name())),
                        };
                        let mut result = s.to_string();
                        let byte_idx = s.char_indices().nth(idx).map(|(i, _)| i).unwrap_or(s.len());
                        result.insert_str(byte_idx, &sub);
                        Ok(Value::Str(Rc::new(result)))
                    }
                    // v0.90.44: remove(start, end) -> String
                    "remove" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("remove", 2, args.len()));
                        }
                        let start = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let end = match &args[1] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[1].type_name())),
                        };
                        let chars: Vec<char> = s.chars().collect();
                        let start = start.min(chars.len());
                        let end = end.min(chars.len());
                        let result: String = chars[..start].iter().chain(chars[end..].iter()).collect();
                        Ok(Value::Str(Rc::new(result)))
                    }
                    // v0.90.50: map_chars(fn(String) -> String) -> String
                    "map_chars" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("map_chars", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result = String::new();
                                for c in s.chars() {
                                    let mapped = self.call_closure(&params, &body, &closure_env, vec![Value::Str(Rc::new(c.to_string()))])?;
                                    match mapped {
                                        Value::Str(s) => result.push_str(&s),
                                        _ => return Err(RuntimeError::type_error("String", mapped.type_name())),
                                    }
                                }
                                Ok(Value::Str(Rc::new(result)))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.50: filter_chars(fn(String) -> bool) -> String
                    "filter_chars" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("filter_chars", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result = String::new();
                                for c in s.chars() {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![Value::Str(Rc::new(c.to_string()))])?;
                                    if pred.is_truthy() {
                                        result.push(c);
                                    }
                                }
                                Ok(Value::Str(Rc::new(result)))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.50: any_char(fn(String) -> bool) -> bool
                    "any_char" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("any_char", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                for c in s.chars() {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![Value::Str(Rc::new(c.to_string()))])?;
                                    if pred.is_truthy() {
                                        return Ok(Value::Bool(true));
                                    }
                                }
                                Ok(Value::Bool(false))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.50: all_chars(fn(String) -> bool) -> bool
                    "all_chars" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("all_chars", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                for c in s.chars() {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![Value::Str(Rc::new(c.to_string()))])?;
                                    if !pred.is_truthy() {
                                        return Ok(Value::Bool(false));
                                    }
                                }
                                Ok(Value::Bool(true))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    _ => Err(RuntimeError::undefined_function(&format!("String.{}", method))),
                }
            }
            Value::Array(arr) => {
                match method {
                    "len" => Ok(Value::Int(arr.len() as i64)),
                    // v0.90.49: Array methods
                    "is_empty" => Ok(Value::Bool(arr.is_empty())),
                    "first" => {
                        match arr.first() {
                            Some(v) => Ok(v.clone()),
                            None => Err(RuntimeError::index_out_of_bounds(0, 0)),
                        }
                    }
                    "last" => {
                        match arr.last() {
                            Some(v) => Ok(v.clone()),
                            None => Err(RuntimeError::index_out_of_bounds(0, 0)),
                        }
                    }
                    "contains" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("contains", 1, args.len()));
                        }
                        Ok(Value::Bool(arr.contains(&args[0])))
                    }
                    "get" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("get", 1, args.len()));
                        }
                        let idx = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        match arr.get(idx) {
                            Some(v) => Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![v.clone()])),
                            None => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                        }
                    }
                    "reverse" => {
                        let mut reversed = arr;
                        reversed.reverse();
                        Ok(Value::Array(reversed))
                    }
                    // v0.90.37: Array functional methods
                    "push" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("push", 1, args.len()));
                        }
                        let mut new_arr = arr;
                        new_arr.push(args.into_iter().next().unwrap());
                        Ok(Value::Array(new_arr))
                    }
                    "pop" => {
                        if !args.is_empty() {
                            return Err(RuntimeError::arity_mismatch("pop", 0, args.len()));
                        }
                        let mut new_arr = arr;
                        new_arr.pop();
                        Ok(Value::Array(new_arr))
                    }
                    "concat" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("concat", 1, args.len()));
                        }
                        let other = match args.into_iter().next().unwrap() {
                            Value::Array(a) => a,
                            other => return Err(RuntimeError::type_error("Array", other.type_name())),
                        };
                        let mut new_arr = arr;
                        new_arr.extend(other);
                        Ok(Value::Array(new_arr))
                    }
                    "slice" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("slice", 2, args.len()));
                        }
                        let start = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let end = match &args[1] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[1].type_name())),
                        };
                        if start > arr.len() || end > arr.len() || start > end {
                            return Err(RuntimeError::index_out_of_bounds(end as i64, arr.len()));
                        }
                        Ok(Value::Array(arr[start..end].to_vec()))
                    }
                    "join" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("join", 1, args.len()));
                        }
                        let sep = match &args[0] {
                            Value::Str(s) => s.as_str().to_string(),
                            _ => return Err(RuntimeError::type_error("string", args[0].type_name())),
                        };
                        let parts: Vec<String> = arr.iter().map(|v| match v {
                            Value::Str(s) => s.to_string(),
                            Value::Int(n) => n.to_string(),
                            Value::Float(f) => f.to_string(),
                            Value::Bool(b) => b.to_string(),
                            other => format!("{:?}", other),
                        }).collect();
                        Ok(Value::Str(Rc::new(parts.join(&sep))))
                    }
                    // v0.90.38: map(fn(T) -> U) -> [U]
                    "map" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("map", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result = Vec::with_capacity(arr.len());
                                for elem in arr {
                                    let val = self.call_closure(&params, &body, &closure_env, vec![elem])?;
                                    result.push(val);
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.38: filter(fn(T) -> bool) -> [T]
                    "filter" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("filter", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result = Vec::new();
                                for elem in arr {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![elem.clone()])?;
                                    if pred.is_truthy() {
                                        result.push(elem);
                                    }
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.38: any(fn(T) -> bool) -> bool
                    "any" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("any", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                for elem in arr {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![elem])?;
                                    if pred.is_truthy() {
                                        return Ok(Value::Bool(true));
                                    }
                                }
                                Ok(Value::Bool(false))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.38: all(fn(T) -> bool) -> bool
                    "all" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("all", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                for elem in arr {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![elem])?;
                                    if !pred.is_truthy() {
                                        return Ok(Value::Bool(false));
                                    }
                                }
                                Ok(Value::Bool(true))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.38: for_each(fn(T) -> ()) -> ()
                    "for_each" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("for_each", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                for elem in arr {
                                    self.call_closure(&params, &body, &closure_env, vec![elem])?;
                                }
                                Ok(Value::Unit)
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.39: fold(init, fn(acc, elem) -> acc) -> acc
                    "fold" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("fold", 2, args.len()));
                        }
                        let mut args_iter = args.into_iter();
                        let init = args_iter.next().unwrap();
                        match args_iter.next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut acc = init;
                                for elem in arr {
                                    acc = self.call_closure(&params, &body, &closure_env, vec![acc, elem])?;
                                }
                                Ok(acc)
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.39: reduce(fn(acc, elem) -> acc) -> T?
                    "reduce" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("reduce", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                if arr.is_empty() {
                                    return Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![]));
                                }
                                let mut iter = arr.into_iter();
                                let mut acc = iter.next().unwrap();
                                for elem in iter {
                                    acc = self.call_closure(&params, &body, &closure_env, vec![acc, elem])?;
                                }
                                Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![acc]))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.39: find(fn(T) -> bool) -> T?
                    "find" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("find", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                for elem in arr {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![elem.clone()])?;
                                    if pred.is_truthy() {
                                        return Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![elem]));
                                    }
                                }
                                Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![]))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.39: position(fn(T) -> bool) -> i64?
                    "position" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("position", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                for (i, elem) in arr.into_iter().enumerate() {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![elem])?;
                                    if pred.is_truthy() {
                                        return Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![Value::Int(i as i64)]));
                                    }
                                }
                                Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![]))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.39: enumerate() -> [(i64, T)]
                    "enumerate" => {
                        if !args.is_empty() {
                            return Err(RuntimeError::arity_mismatch("enumerate", 0, args.len()));
                        }
                        let result: Vec<Value> = arr.into_iter().enumerate()
                            .map(|(i, v)| Value::Tuple(vec![Value::Int(i as i64), v]))
                            .collect();
                        Ok(Value::Array(result))
                    }
                    // v0.90.40: take(n) -> [T]
                    "take" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("take", 1, args.len()));
                        }
                        let n = match &args[0] {
                            Value::Int(n) => (*n).max(0) as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        Ok(Value::Array(arr.into_iter().take(n).collect()))
                    }
                    // v0.90.40: drop(n) -> [T]
                    "drop" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("drop", 1, args.len()));
                        }
                        let n = match &args[0] {
                            Value::Int(n) => (*n).max(0) as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        Ok(Value::Array(arr.into_iter().skip(n).collect()))
                    }
                    // v0.90.40: zip([U]) -> [(T, U)]
                    "zip" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("zip", 1, args.len()));
                        }
                        let other = match args.into_iter().next().unwrap() {
                            Value::Array(a) => a,
                            other => return Err(RuntimeError::type_error("Array", other.type_name())),
                        };
                        let result: Vec<Value> = arr.into_iter().zip(other)
                            .map(|(a, b)| Value::Tuple(vec![a, b]))
                            .collect();
                        Ok(Value::Array(result))
                    }
                    // v0.90.40: flatten() -> [T]
                    "flatten" => {
                        if !args.is_empty() {
                            return Err(RuntimeError::arity_mismatch("flatten", 0, args.len()));
                        }
                        let mut result = Vec::new();
                        for elem in arr {
                            match elem {
                                Value::Array(inner) => result.extend(inner),
                                _ => return Err(RuntimeError::type_error("Array", elem.type_name())),
                            }
                        }
                        Ok(Value::Array(result))
                    }
                    // v0.90.40: sort_by(fn(T, T) -> i64) -> [T]
                    "sort_by" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("sort_by", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result = arr;
                                let mut error = None;
                                result.sort_by(|a, b| {
                                    if error.is_some() {
                                        return std::cmp::Ordering::Equal;
                                    }
                                    match self.call_closure(&params, &body, &closure_env, vec![a.clone(), b.clone()]) {
                                        Ok(Value::Int(n)) => {
                                            if n < 0 { std::cmp::Ordering::Less }
                                            else if n > 0 { std::cmp::Ordering::Greater }
                                            else { std::cmp::Ordering::Equal }
                                        }
                                        Ok(_) => { error = Some(RuntimeError::type_error("integer", "non-integer")); std::cmp::Ordering::Equal }
                                        Err(e) => { error = Some(e); std::cmp::Ordering::Equal }
                                    }
                                });
                                if let Some(e) = error {
                                    return Err(e);
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.45: windows(size) -> [[T]]
                    "windows" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("windows", 1, args.len()));
                        }
                        let size = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        if size == 0 || size > arr.len() {
                            return Ok(Value::Array(vec![]));
                        }
                        let windows: Vec<Value> = arr.windows(size)
                            .map(|w| Value::Array(w.to_vec()))
                            .collect();
                        Ok(Value::Array(windows))
                    }
                    // v0.90.45: chunks(size) -> [[T]]
                    "chunks" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("chunks", 1, args.len()));
                        }
                        let size = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        if size == 0 {
                            return Ok(Value::Array(vec![]));
                        }
                        let chunks: Vec<Value> = arr.chunks(size)
                            .map(|c| Value::Array(c.to_vec()))
                            .collect();
                        Ok(Value::Array(chunks))
                    }
                    // v0.90.45: count(fn(T) -> bool) -> i64
                    "count" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("count", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut n = 0i64;
                                for elem in arr {
                                    let val = self.call_closure(&params, &body, &closure_env, vec![elem])?;
                                    if val.is_truthy() {
                                        n += 1;
                                    }
                                }
                                Ok(Value::Int(n))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.45: unique() -> [T]
                    "unique" => {
                        let mut result = Vec::new();
                        for elem in &arr {
                            if !result.contains(elem) {
                                result.push(elem.clone());
                            }
                        }
                        Ok(Value::Array(result))
                    }
                    // v0.90.43: sort() -> [T] (natural ordering)
                    "sort" => {
                        let mut result = arr;
                        result.sort_by(|a, b| {
                            match (a, b) {
                                (Value::Int(x), Value::Int(y)) => x.cmp(y),
                                (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                (Value::Str(x), Value::Str(y)) => x.cmp(y),
                                _ => std::cmp::Ordering::Equal,
                            }
                        });
                        Ok(Value::Array(result))
                    }
                    // v0.90.43: dedup() -> [T] (remove consecutive duplicates)
                    "dedup" => {
                        if arr.is_empty() {
                            return Ok(Value::Array(arr));
                        }
                        let mut result = vec![arr[0].clone()];
                        for elem in arr.iter().skip(1) {
                            if result.last() != Some(elem) {
                                result.push(elem.clone());
                            }
                        }
                        Ok(Value::Array(result))
                    }
                    // v0.90.43: sum() -> T
                    "sum" => {
                        if arr.is_empty() {
                            return Ok(Value::Int(0));
                        }
                        match &arr[0] {
                            Value::Int(_) => {
                                let total: i64 = arr.iter().map(|v| match v { Value::Int(n) => *n, _ => 0 }).sum();
                                Ok(Value::Int(total))
                            }
                            Value::Float(_) => {
                                let total: f64 = arr.iter().map(|v| match v { Value::Float(f) => *f, _ => 0.0 }).sum();
                                Ok(Value::Float(total))
                            }
                            _ => Err(RuntimeError::type_error("numeric array", "non-numeric array")),
                        }
                    }
                    // v0.90.43: product() -> T
                    "product" => {
                        if arr.is_empty() {
                            return Ok(Value::Int(1));
                        }
                        match &arr[0] {
                            Value::Int(_) => {
                                let total: i64 = arr.iter().map(|v| match v { Value::Int(n) => *n, _ => 1 }).product();
                                Ok(Value::Int(total))
                            }
                            Value::Float(_) => {
                                let total: f64 = arr.iter().map(|v| match v { Value::Float(f) => *f, _ => 1.0 }).product();
                                Ok(Value::Float(total))
                            }
                            _ => Err(RuntimeError::type_error("numeric array", "non-numeric array")),
                        }
                    }
                    // v0.90.43: min() -> T? (minimum element)
                    "min" => {
                        if arr.is_empty() {
                            return Ok(Value::Int(0)); // null for i64?
                        }
                        let mut min_val = arr[0].clone();
                        for elem in arr.iter().skip(1) {
                            let is_less = match (&min_val, elem) {
                                (Value::Int(a), Value::Int(b)) => b < a,
                                (Value::Float(a), Value::Float(b)) => b < a,
                                (Value::Str(a), Value::Str(b)) => b < a,
                                _ => false,
                            };
                            if is_less {
                                min_val = elem.clone();
                            }
                        }
                        Ok(min_val)
                    }
                    // v0.90.43: max() -> T? (maximum element)
                    "max" => {
                        if arr.is_empty() {
                            return Ok(Value::Int(0)); // null for i64?
                        }
                        let mut max_val = arr[0].clone();
                        for elem in arr.iter().skip(1) {
                            let is_greater = match (&max_val, elem) {
                                (Value::Int(a), Value::Int(b)) => b > a,
                                (Value::Float(a), Value::Float(b)) => b > a,
                                (Value::Str(a), Value::Str(b)) => b > a,
                                _ => false,
                            };
                            if is_greater {
                                max_val = elem.clone();
                            }
                        }
                        Ok(max_val)
                    }
                    // v0.90.43: flat_map(fn(T) -> [U]) -> [U]
                    "flat_map" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("flat_map", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result = Vec::new();
                                for elem in arr {
                                    let val = self.call_closure(&params, &body, &closure_env, vec![elem])?;
                                    match val {
                                        Value::Array(inner) => result.extend(inner),
                                        _ => return Err(RuntimeError::type_error("array", val.type_name())),
                                    }
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.49: scan(init, fn(acc, T) -> acc) -> [acc]
                    "scan" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("scan", 2, args.len()));
                        }
                        let mut args_iter = args.into_iter();
                        let init = args_iter.next().unwrap();
                        match args_iter.next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut acc = init;
                                let mut result = Vec::with_capacity(arr.len());
                                for elem in arr {
                                    acc = self.call_closure(&params, &body, &closure_env, vec![acc, elem])?;
                                    result.push(acc.clone());
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.49: partition(fn(T) -> bool) -> [T] (matching elements)
                    "partition" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("partition", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut matching = Vec::new();
                                for elem in arr {
                                    let val = self.call_closure(&params, &body, &closure_env, vec![elem.clone()])?;
                                    if val.is_truthy() {
                                        matching.push(elem);
                                    }
                                }
                                Ok(Value::Array(matching))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.49: skip_while(fn(T) -> bool) -> [T]
                    "skip_while" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("skip_while", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut skipping = true;
                                let mut result = Vec::new();
                                for elem in arr {
                                    if skipping {
                                        let val = self.call_closure(&params, &body, &closure_env, vec![elem.clone()])?;
                                        if !val.is_truthy() {
                                            skipping = false;
                                            result.push(elem);
                                        }
                                    } else {
                                        result.push(elem);
                                    }
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.49: take_while(fn(T) -> bool) -> [T]
                    "take_while" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("take_while", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result = Vec::new();
                                for elem in arr {
                                    let val = self.call_closure(&params, &body, &closure_env, vec![elem.clone()])?;
                                    if val.is_truthy() {
                                        result.push(elem);
                                    } else {
                                        break;
                                    }
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    // v0.90.46: swap(i, j) -> [T]
                    "swap" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("swap", 2, args.len()));
                        }
                        let i = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let j = match &args[1] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[1].type_name())),
                        };
                        let mut result = arr;
                        if i < result.len() && j < result.len() {
                            result.swap(i, j);
                        }
                        Ok(Value::Array(result))
                    }
                    // v0.90.46: rotate_left(n) -> [T]
                    "rotate_left" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("rotate_left", 1, args.len()));
                        }
                        let n = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let mut result = arr;
                        if !result.is_empty() {
                            let n = n % result.len();
                            result.rotate_left(n);
                        }
                        Ok(Value::Array(result))
                    }
                    // v0.90.46: rotate_right(n) -> [T]
                    "rotate_right" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("rotate_right", 1, args.len()));
                        }
                        let n = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let mut result = arr;
                        if !result.is_empty() {
                            let n = n % result.len();
                            result.rotate_right(n);
                        }
                        Ok(Value::Array(result))
                    }
                    // v0.90.46: fill(value) -> [T]
                    "fill" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("fill", 1, args.len()));
                        }
                        let val = args.into_iter().next().unwrap();
                        let result = vec![val; arr.len()];
                        Ok(Value::Array(result))
                    }
                    // v0.90.46: index_of(value) -> i64?
                    "index_of" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("index_of", 1, args.len()));
                        }
                        let target = &args[0];
                        match arr.iter().position(|x| x == target) {
                            Some(idx) => Ok(Value::Int(idx as i64)),
                            None => Ok(Value::Int(0)), // null
                        }
                    }
                    // v0.90.51: zip_with(other, fn(T, U) -> V) -> [V]
                    "zip_with" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("zip_with", 2, args.len()));
                        }
                        let mut args_iter = args.into_iter();
                        let other = args_iter.next().unwrap();
                        let closure = args_iter.next().unwrap();
                        match (other, closure) {
                            (Value::Array(other_arr), Value::Closure { params, body, env: closure_env }) => {
                                let mut result = Vec::new();
                                let len = arr.len().min(other_arr.len());
                                for i in 0..len {
                                    let a = arr[i].clone();
                                    let b = other_arr[i].clone();
                                    let val = self.call_closure(&params, &body, &closure_env, vec![a, b])?;
                                    result.push(val);
                                }
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("array and closure", "other")),
                        }
                    }
                    // v0.90.51: each_cons(n) -> [[T]]
                    "each_cons" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("each_cons", 1, args.len()));
                        }
                        let n = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        if n == 0 || n > arr.len() {
                            return Ok(Value::Array(Vec::new()));
                        }
                        let mut result = Vec::new();
                        for i in 0..=(arr.len() - n) {
                            result.push(Value::Array(arr[i..i + n].to_vec()));
                        }
                        Ok(Value::Array(result))
                    }
                    // v0.90.51: step_by(n) -> [T]
                    "step_by" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("step_by", 1, args.len()));
                        }
                        let n = match &args[0] {
                            Value::Int(n) => *n as usize,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        if n == 0 {
                            return Err(RuntimeError::type_error("positive integer", "zero"));
                        }
                        let result: Vec<Value> = arr.iter().step_by(n).cloned().collect();
                        Ok(Value::Array(result))
                    }
                    // v0.90.51: chunk_by(fn(T) -> K) -> [[T]]
                    "chunk_by" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("chunk_by", 1, args.len()));
                        }
                        match args.into_iter().next().unwrap() {
                            Value::Closure { params, body, env: closure_env } => {
                                let mut result: Vec<Value> = Vec::new();
                                if arr.is_empty() {
                                    return Ok(Value::Array(result));
                                }
                                let mut current_chunk = vec![arr[0].clone()];
                                let mut current_key = self.call_closure(&params, &body, &closure_env, vec![arr[0].clone()])?;
                                for item in arr.iter().skip(1) {
                                    let key = self.call_closure(&params, &body, &closure_env, vec![item.clone()])?;
                                    if key == current_key {
                                        current_chunk.push(item.clone());
                                    } else {
                                        result.push(Value::Array(current_chunk));
                                        current_chunk = vec![item.clone()];
                                        current_key = key;
                                    }
                                }
                                result.push(Value::Array(current_chunk));
                                Ok(Value::Array(result))
                            }
                            _ => Err(RuntimeError::type_error("closure", "non-closure")),
                        }
                    }
                    _ => Err(RuntimeError::undefined_function(&format!("Array.{}", method))),
                }
            }
            // v0.18: Option<T> methods
            Value::Enum(enum_name, variant, values) if enum_name == "Option" => {
                match method {
                    "is_some" => Ok(Value::Bool(variant == "Some")),
                    "is_none" => Ok(Value::Bool(variant == "None")),
                    "unwrap_or" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("unwrap_or", 1, args.len()));
                        }
                        match variant.as_str() {
                            "Some" => Ok(values.first().cloned().unwrap_or(Value::Unit)),
                            "None" => Ok(args.into_iter().next().unwrap()),
                            _ => Err(RuntimeError::type_error("Option variant", &variant)),
                        }
                    }
                    // v0.90.42: map(fn(T) -> U) -> U?
                    "map" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("map", 1, args.len()));
                        }
                        match variant.as_str() {
                            "Some" => {
                                let inner = values.into_iter().next().unwrap_or(Value::Unit);
                                match args.into_iter().next().unwrap() {
                                    Value::Closure { params, body, env: closure_env } => {
                                        let result = self.call_closure(&params, &body, &closure_env, vec![inner])?;
                                        Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![result]))
                                    }
                                    _ => Err(RuntimeError::type_error("closure", "non-closure")),
                                }
                            }
                            "None" => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                            _ => Err(RuntimeError::type_error("Option variant", &variant)),
                        }
                    }
                    // v0.90.42: and_then(fn(T) -> U?) -> U?
                    "and_then" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("and_then", 1, args.len()));
                        }
                        match variant.as_str() {
                            "Some" => {
                                let inner = values.into_iter().next().unwrap_or(Value::Unit);
                                match args.into_iter().next().unwrap() {
                                    Value::Closure { params, body, env: closure_env } => {
                                        self.call_closure(&params, &body, &closure_env, vec![inner])
                                    }
                                    _ => Err(RuntimeError::type_error("closure", "non-closure")),
                                }
                            }
                            "None" => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                            _ => Err(RuntimeError::type_error("Option variant", &variant)),
                        }
                    }
                    // v0.90.42: filter(fn(T) -> bool) -> T?
                    "filter" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("filter", 1, args.len()));
                        }
                        match variant.as_str() {
                            "Some" => {
                                let inner = values.first().cloned().unwrap_or(Value::Unit);
                                match args.into_iter().next().unwrap() {
                                    Value::Closure { params, body, env: closure_env } => {
                                        let pred = self.call_closure(&params, &body, &closure_env, vec![inner.clone()])?;
                                        if pred.is_truthy() {
                                            Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![inner]))
                                        } else {
                                            Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![]))
                                        }
                                    }
                                    _ => Err(RuntimeError::type_error("closure", "non-closure")),
                                }
                            }
                            "None" => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                            _ => Err(RuntimeError::type_error("Option variant", &variant)),
                        }
                    }
                    // v0.90.42: unwrap() -> T (panics if None)
                    "unwrap" => {
                        match variant.as_str() {
                            "Some" => Ok(values.into_iter().next().unwrap_or(Value::Unit)),
                            "None" => Err(RuntimeError::type_error("Some", "None (unwrap on None)")),
                            _ => Err(RuntimeError::type_error("Option variant", &variant)),
                        }
                    }
                    _ => Err(RuntimeError::undefined_function(&format!("Option.{}", method))),
                }
            }
            // v0.18: Result<T, E> methods
            Value::Enum(enum_name, variant, values) if enum_name == "Result" => {
                match method {
                    "is_ok" => Ok(Value::Bool(variant == "Ok")),
                    "is_err" => Ok(Value::Bool(variant == "Err")),
                    "unwrap_or" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("unwrap_or", 1, args.len()));
                        }
                        match variant.as_str() {
                            "Ok" => Ok(values.first().cloned().unwrap_or(Value::Unit)),
                            "Err" => Ok(args.into_iter().next().unwrap()),
                            _ => Err(RuntimeError::type_error("Result variant", &variant)),
                        }
                    }
                    // v0.90.42: unwrap() -> T (panics if Err)
                    "unwrap" => {
                        match variant.as_str() {
                            "Ok" => Ok(values.into_iter().next().unwrap_or(Value::Unit)),
                            "Err" => Err(RuntimeError::type_error("Ok", "Err (unwrap on Err)")),
                            _ => Err(RuntimeError::type_error("Result variant", &variant)),
                        }
                    }
                    _ => Err(RuntimeError::undefined_function(&format!("Result.{}", method))),
                }
            }
            // v0.90.36: Bool methods
            Value::Bool(b) => {
                match method {
                    "to_string" => Ok(Value::Str(Rc::new(if b { "true" } else { "false" }.to_string()))),
                    _ => Err(RuntimeError::undefined_function(&format!("bool.{}", method))),
                }
            }
            // v0.89.17: Nullable<T> (T?) methods on integer values + v0.90.35: Integer methods
            Value::Int(n) => {
                match method {
                    "is_some" => Ok(Value::Bool(n != 0)),
                    "is_none" => Ok(Value::Bool(n == 0)),
                    "unwrap_or" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("unwrap_or", 1, args.len()));
                        }
                        if n != 0 {
                            Ok(Value::Int(n))
                        } else {
                            Ok(args.into_iter().next().unwrap())
                        }
                    }
                    // v0.90.35: Integer methods
                    "abs" => Ok(Value::Int(n.abs())),
                    "min" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("min", 1, args.len()));
                        }
                        let other = match &args[0] {
                            Value::Int(m) => *m,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        Ok(Value::Int(n.min(other)))
                    }
                    "max" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("max", 1, args.len()));
                        }
                        let other = match &args[0] {
                            Value::Int(m) => *m,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        Ok(Value::Int(n.max(other)))
                    }
                    "clamp" => {
                        if args.len() != 2 {
                            return Err(RuntimeError::arity_mismatch("clamp", 2, args.len()));
                        }
                        let lo = match &args[0] {
                            Value::Int(m) => *m,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let hi = match &args[1] {
                            Value::Int(m) => *m,
                            _ => return Err(RuntimeError::type_error("integer", args[1].type_name())),
                        };
                        Ok(Value::Int(n.clamp(lo, hi)))
                    }
                    "pow" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("pow", 1, args.len()));
                        }
                        let exp = match &args[0] {
                            Value::Int(m) => *m as u32,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        Ok(Value::Int(n.pow(exp)))
                    }
                    "to_float" => Ok(Value::Float(n as f64)),
                    "to_string" => Ok(Value::Str(Rc::new(n.to_string()))),
                    // v0.90.42: map(fn(T) -> U) -> U? — nullable closure methods
                    "map" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("map", 1, args.len()));
                        }
                        if n != 0 {
                            match args.into_iter().next().unwrap() {
                                Value::Closure { params, body, env: closure_env } => {
                                    self.call_closure(&params, &body, &closure_env, vec![Value::Int(n)])
                                }
                                _ => Err(RuntimeError::type_error("closure", "non-closure")),
                            }
                        } else {
                            Ok(Value::Int(0))
                        }
                    }
                    "and_then" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("and_then", 1, args.len()));
                        }
                        if n != 0 {
                            match args.into_iter().next().unwrap() {
                                Value::Closure { params, body, env: closure_env } => {
                                    self.call_closure(&params, &body, &closure_env, vec![Value::Int(n)])
                                }
                                _ => Err(RuntimeError::type_error("closure", "non-closure")),
                            }
                        } else {
                            Ok(Value::Int(0))
                        }
                    }
                    "filter" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("filter", 1, args.len()));
                        }
                        if n != 0 {
                            match args.into_iter().next().unwrap() {
                                Value::Closure { params, body, env: closure_env } => {
                                    let pred = self.call_closure(&params, &body, &closure_env, vec![Value::Int(n)])?;
                                    if pred.is_truthy() {
                                        Ok(Value::Int(n))
                                    } else {
                                        Ok(Value::Int(0))
                                    }
                                }
                                _ => Err(RuntimeError::type_error("closure", "non-closure")),
                            }
                        } else {
                            Ok(Value::Int(0))
                        }
                    }
                    "unwrap" => {
                        if n != 0 {
                            Ok(Value::Int(n))
                        } else {
                            Err(RuntimeError::type_error("non-null", "null (unwrap on null)"))
                        }
                    }
                    // v0.90.42: sign() -> i64 (-1, 0, or 1)
                    "sign" => Ok(Value::Int(if n > 0 { 1 } else if n < 0 { -1 } else { 0 })),
                    "is_positive" => Ok(Value::Bool(n > 0)),
                    "is_negative" => Ok(Value::Bool(n < 0)),
                    "is_zero" => Ok(Value::Bool(n == 0)),
                    // v0.90.42: gcd(other) -> i64
                    "gcd" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("gcd", 1, args.len()));
                        }
                        let other = match &args[0] {
                            Value::Int(m) => *m,
                            _ => return Err(RuntimeError::type_error("integer", args[0].type_name())),
                        };
                        let mut a = n.abs();
                        let mut b = other.abs();
                        while b != 0 {
                            let t = b;
                            b = a % b;
                            a = t;
                        }
                        Ok(Value::Int(a))
                    }
                    // v0.90.47: to_hex() -> String
                    "to_hex" => Ok(Value::Str(Rc::new(format!("{:x}", n)))),
                    // v0.90.47: to_binary() -> String
                    "to_binary" => Ok(Value::Str(Rc::new(format!("{:b}", n)))),
                    // v0.90.47: to_octal() -> String
                    "to_octal" => Ok(Value::Str(Rc::new(format!("{:o}", n)))),
                    // v0.90.47: digits() -> [i64]
                    "digits" => {
                        let mut digits = Vec::new();
                        let mut num = n.abs();
                        if num == 0 {
                            digits.push(Value::Int(0));
                        } else {
                            while num > 0 {
                                digits.push(Value::Int(num % 10));
                                num /= 10;
                            }
                            digits.reverse();
                        }
                        Ok(Value::Array(digits))
                    }
                    _ => Err(RuntimeError::type_error("object with methods", receiver.type_name())),
                }
            }
            // v0.90.40: Trait method dispatch for struct instances
            Value::Struct(ref type_name, _) => {
                let key = (type_name.clone(), method.to_string());
                if let Some(fn_def) = self.impl_methods.get(&key).cloned() {
                    // Prepend receiver as 'self' parameter
                    let mut all_args = vec![receiver];
                    all_args.extend(args);
                    self.call_function(&fn_def, &all_args)
                } else {
                    Err(RuntimeError::undefined_function(&format!("{}.{}", type_name, method)))
                }
            }
            // v0.90.47: Trait method dispatch for enum instances
            Value::Enum(ref enum_name, _, _) => {
                let key = (enum_name.clone(), method.to_string());
                if let Some(fn_def) = self.impl_methods.get(&key).cloned() {
                    let mut all_args = vec![receiver];
                    all_args.extend(args);
                    self.call_function(&fn_def, &all_args)
                } else {
                    Err(RuntimeError::undefined_function(&format!("{}.{}", enum_name, method)))
                }
            }
            _ => Err(RuntimeError::type_error("object with methods", receiver.type_name())),
        }
    }

    /// Try to match a value against a pattern, returning bindings if successful
    fn match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(vec![]),

            Pattern::Var(name) => Some(vec![(name.clone(), value.clone())]),

            Pattern::Literal(lit) => {
                match (lit, value) {
                    (crate::ast::LiteralPattern::Int(n), Value::Int(v)) if *n == *v => Some(vec![]),
                    (crate::ast::LiteralPattern::Float(f), Value::Float(v)) if *f == *v => Some(vec![]),
                    (crate::ast::LiteralPattern::Bool(b), Value::Bool(v)) if *b == *v => Some(vec![]),
                    (crate::ast::LiteralPattern::String(s), Value::Str(v)) if s == v.as_ref() => Some(vec![]),
                    // v0.30.283: StringRope support for pattern matching
                    (crate::ast::LiteralPattern::String(s), Value::StringRope(r)) => {
                        let materialized: String = r.borrow().iter().map(|f| f.as_str()).collect();
                        if s == &materialized { Some(vec![]) } else { None }
                    }
                    _ => None,
                }
            }

            // v0.41: Nested patterns in enum bindings
            Pattern::EnumVariant { enum_name, variant, bindings } => {
                match value {
                    Value::Enum(e_name, v_name, args) if e_name == enum_name && v_name == variant => {
                        if bindings.len() != args.len() {
                            return None;
                        }
                        let mut result = vec![];
                        for (binding, arg) in bindings.iter().zip(args.iter()) {
                            // Recursively match nested patterns
                            if let Some(inner_bindings) = self.match_pattern(&binding.node, arg) {
                                result.extend(inner_bindings);
                            } else {
                                return None;
                            }
                        }
                        Some(result)
                    }
                    _ => None,
                }
            }

            Pattern::Struct { name, fields } => {
                match value {
                    Value::Struct(s_name, s_fields) if s_name == name => {
                        let mut result = vec![];
                        for (field_name, field_pat) in fields {
                            if let Some(field_val) = s_fields.get(&field_name.node) {
                                if let Some(inner_bindings) = self.match_pattern(&field_pat.node, field_val) {
                                    result.extend(inner_bindings);
                                } else {
                                    return None;
                                }
                            } else {
                                return None;
                            }
                        }
                        Some(result)
                    }
                    _ => None,
                }
            }
            // v0.39: Range pattern
            Pattern::Range { start, end, inclusive } => {
                let val_int = match value {
                    Value::Int(n) => *n,
                    _ => return None,
                };
                let start_int = match start {
                    LiteralPattern::Int(n) => *n,
                    _ => return None,
                };
                let end_int = match end {
                    LiteralPattern::Int(n) => *n,
                    _ => return None,
                };
                let in_range = if *inclusive {
                    val_int >= start_int && val_int <= end_int
                } else {
                    val_int >= start_int && val_int < end_int
                };
                if in_range { Some(vec![]) } else { None }
            }
            // v0.40: Or-pattern: try each alternative
            Pattern::Or(alts) => {
                for alt in alts {
                    if let Some(bindings) = self.match_pattern(&alt.node, value) {
                        return Some(bindings);
                    }
                }
                None
            }
            // v0.41: Binding pattern: name @ pattern
            Pattern::Binding { name, pattern } => {
                // First match the inner pattern
                if let Some(mut inner_bindings) = self.match_pattern(&pattern.node, value) {
                    // Add the binding for the entire value
                    inner_bindings.push((name.clone(), value.clone()));
                    Some(inner_bindings)
                } else {
                    None
                }
            }
            // v0.42: Tuple pattern
            Pattern::Tuple(patterns) => {
                if let Value::Tuple(values) = value {
                    if patterns.len() != values.len() {
                        return None;
                    }
                    let mut bindings = Vec::new();
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if let Some(sub_bindings) = self.match_pattern(&pat.node, val) {
                            bindings.extend(sub_bindings);
                        } else {
                            return None;
                        }
                    }
                    Some(bindings)
                } else {
                    None
                }
            }
            // v0.44: Array pattern
            Pattern::Array(patterns) => {
                if let Value::Array(values) = value {
                    if patterns.len() != values.len() {
                        return None;
                    }
                    let mut bindings = Vec::new();
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if let Some(sub_bindings) = self.match_pattern(&pat.node, val) {
                            bindings.extend(sub_bindings);
                        } else {
                            return None;
                        }
                    }
                    Some(bindings)
                } else {
                    None
                }
            }
            // v0.45: Array rest pattern - matches arrays with prefix..suffix
            // The ".." skips zero or more elements in the middle (non-capturing)
            Pattern::ArrayRest { prefix, suffix } => {
                if let Value::Array(values) = value {
                    let required_len = prefix.len() + suffix.len();
                    // Array must have at least enough elements for prefix + suffix
                    if values.len() < required_len {
                        return None;
                    }

                    let mut bindings = Vec::new();

                    // Match prefix elements from the start
                    for (pat, val) in prefix.iter().zip(values.iter()) {
                        if let Some(sub_bindings) = self.match_pattern(&pat.node, val) {
                            bindings.extend(sub_bindings);
                        } else {
                            return None;
                        }
                    }

                    // Match suffix elements from the end
                    for (pat, val) in suffix.iter().zip(values.iter().skip(values.len() - suffix.len())) {
                        if let Some(sub_bindings) = self.match_pattern(&pat.node, val) {
                            bindings.extend(sub_bindings);
                        } else {
                            return None;
                        }
                    }

                    Some(bindings)
                } else {
                    None
                }
            }
        }
    }

    /// Call a function by name
    fn call(&mut self, name: &str, args: Vec<Value>) -> InterpResult<Value> {
        // Check builtins first
        if let Some(builtin) = self.builtins.get(name) {
            return builtin(&args);
        }

        // Then user-defined functions
        if let Some(fn_def) = self.functions.get(name).cloned() {
            return self.call_function(&fn_def, &args);
        }

        Err(RuntimeError::undefined_function(name))
    }

    /// v0.92: Call a closure value with captured environment
    fn call_closure(
        &mut self,
        params: &[String],
        body: &Spanned<Expr>,
        captured_env: &EnvRef,
        args: Vec<Value>,
    ) -> InterpResult<Value> {
        if params.len() != args.len() {
            return Err(RuntimeError::arity_mismatch(
                "<closure>",
                params.len(),
                args.len(),
            ));
        }

        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            self.recursion_depth -= 1;
            return Err(RuntimeError::stack_overflow());
        }

        // Create child env from the captured environment (lexical scoping)
        let closure_env = child_env(captured_env);

        // Bind parameters
        for (param, arg) in params.iter().zip(args.into_iter()) {
            closure_env.borrow_mut().define(param.clone(), arg);
        }

        let result = self.eval(body, &closure_env);
        self.recursion_depth -= 1;
        result
    }

    /// Call a user-defined function with automatic stack growth
    fn call_function(&mut self, fn_def: &FnDef, args: &[Value]) -> InterpResult<Value> {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROW_SIZE, || {
            self.call_function_inner(fn_def, args)
        })
    }

    /// Inner function call implementation
    fn call_function_inner(&mut self, fn_def: &FnDef, args: &[Value]) -> InterpResult<Value> {
        // Check arity
        if fn_def.params.len() != args.len() {
            return Err(RuntimeError::arity_mismatch(
                &fn_def.name.node,
                fn_def.params.len(),
                args.len(),
            ));
        }

        // Check recursion depth
        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            self.recursion_depth -= 1;
            return Err(RuntimeError::stack_overflow());
        }

        // Create new environment for function body
        let func_env = child_env(&self.global_env);

        // Bind parameters
        for (param, arg) in fn_def.params.iter().zip(args.iter()) {
            func_env
                .borrow_mut()
                .define(param.name.node.clone(), arg.clone());
        }

        // Evaluate pre-condition if present
        if let Some(pre) = &fn_def.pre {
            let pre_val = self.eval(pre, &func_env)?;
            if !pre_val.is_truthy() {
                self.recursion_depth -= 1;
                return Err(RuntimeError::pre_condition_failed(&fn_def.name.node));
            }
        }

        // Evaluate body — catch Return control flow
        let result = match self.eval(&fn_def.body, &func_env) {
            Ok(v) => Ok(v),
            Err(e) if matches!(e.kind, ErrorKind::Return(_)) => {
                if let ErrorKind::Return(val) = e.kind {
                    Ok(*val)
                } else {
                    unreachable!()
                }
            }
            Err(e) => Err(e),
        };
        self.recursion_depth -= 1;
        result
    }

    /// v0.39: Evaluate type cast
    fn eval_cast(&self, val: Value, target_ty: &Type) -> InterpResult<Value> {
        match (&val, target_ty) {
            // i64 casts
            (Value::Int(n), Type::I64) => Ok(Value::Int(*n)),
            (Value::Int(n), Type::I32) => Ok(Value::Int(*n as i32 as i64)),
            (Value::Int(n), Type::U32) => Ok(Value::Int(*n as u32 as i64)),
            (Value::Int(n), Type::U64) => Ok(Value::Int(*n as u64 as i64)),
            (Value::Int(n), Type::F64) => Ok(Value::Float(*n as f64)),
            (Value::Int(n), Type::Bool) => Ok(Value::Bool(*n != 0)),
            // f64 casts
            (Value::Float(f), Type::I64) => Ok(Value::Int(*f as i64)),
            (Value::Float(f), Type::I32) => Ok(Value::Int(*f as i32 as i64)),
            (Value::Float(f), Type::U32) => Ok(Value::Int(*f as u32 as i64)),
            (Value::Float(f), Type::U64) => Ok(Value::Int(*f as u64 as i64)),
            (Value::Float(f), Type::F64) => Ok(Value::Float(*f)),
            (Value::Float(f), Type::Bool) => Ok(Value::Bool(*f != 0.0)),
            // bool casts
            (Value::Bool(b), Type::I64) => Ok(Value::Int(if *b { 1 } else { 0 })),
            (Value::Bool(b), Type::I32) => Ok(Value::Int(if *b { 1 } else { 0 })),
            (Value::Bool(b), Type::U32) => Ok(Value::Int(if *b { 1 } else { 0 })),
            (Value::Bool(b), Type::U64) => Ok(Value::Int(if *b { 1 } else { 0 })),
            (Value::Bool(b), Type::F64) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
            (Value::Bool(b), Type::Bool) => Ok(Value::Bool(*b)),
            // v0.51.41: Pointer casts for malloc/free
            // i64 -> *T (malloc result to typed pointer)
            (Value::Int(n), Type::Ptr(_)) => Ok(Value::Int(*n)),
            _ => Err(RuntimeError::type_error(
                &format!("{}", target_ty),
                &format!("cannot cast {} to {}", val.type_name(), target_ty),
            )),
        }
    }

    /// Evaluate binary operation
    fn eval_binary(&self, op: BinOp, left: Value, right: Value) -> InterpResult<Value> {
        match op {
            // Arithmetic
            BinOp::Add => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                // String concatenation (v0.30.283: StringRope for lazy concat)
                (Value::Str(_), Value::Str(_)) |
                (Value::Str(_), Value::StringRope(_)) |
                (Value::StringRope(_), Value::Str(_)) |
                (Value::StringRope(_), Value::StringRope(_)) => {
                    Value::concat_strings(&left, &right).ok_or_else(|| {
                        RuntimeError::type_error("string", "invalid string concat")
                    })
                }
                _ => Err(RuntimeError::type_error(
                    "numeric or string",
                    &format!("{} + {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::Sub => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(RuntimeError::type_error(
                    "numeric",
                    &format!("{} - {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::Mul => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err(RuntimeError::type_error(
                    "numeric",
                    &format!("{} * {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::Div => match (&left, &right) {
                (Value::Int(_), Value::Int(0)) => Err(RuntimeError::division_by_zero()),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / *b as f64)),
                _ => Err(RuntimeError::type_error(
                    "numeric",
                    &format!("{} / {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::Mod => match (&left, &right) {
                (Value::Int(_), Value::Int(0)) => Err(RuntimeError::division_by_zero()),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
                _ => Err(RuntimeError::type_error("int", left.type_name())),
            },

            // v0.37: Wrapping arithmetic (no overflow panic)
            BinOp::AddWrap => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_add(*b))),
                _ => Err(RuntimeError::type_error(
                    "int +% int",
                    &format!("{} +% {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::SubWrap => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_sub(*b))),
                _ => Err(RuntimeError::type_error(
                    "int -% int",
                    &format!("{} -% {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::MulWrap => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.wrapping_mul(*b))),
                _ => Err(RuntimeError::type_error(
                    "int *% int",
                    &format!("{} *% {}", left.type_name(), right.type_name()),
                )),
            },

            // v0.38: Checked arithmetic (returns Some(value) or None on overflow)
            // For now, wrap in Option-like Enum. Full Option support needs more work.
            BinOp::AddChecked => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => {
                    match a.checked_add(*b) {
                        Some(v) => Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![Value::Int(v)])),
                        None => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                    }
                }
                _ => Err(RuntimeError::type_error(
                    "int +? int",
                    &format!("{} +? {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::SubChecked => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => {
                    match a.checked_sub(*b) {
                        Some(v) => Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![Value::Int(v)])),
                        None => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                    }
                }
                _ => Err(RuntimeError::type_error(
                    "int -? int",
                    &format!("{} -? {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::MulChecked => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => {
                    match a.checked_mul(*b) {
                        Some(v) => Ok(Value::Enum("Option".to_string(), "Some".to_string(), vec![Value::Int(v)])),
                        None => Ok(Value::Enum("Option".to_string(), "None".to_string(), vec![])),
                    }
                }
                _ => Err(RuntimeError::type_error(
                    "int *? int",
                    &format!("{} *? {}", left.type_name(), right.type_name()),
                )),
            },

            // v0.38: Saturating arithmetic (clamps to min/max on overflow)
            BinOp::AddSat => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.saturating_add(*b))),
                _ => Err(RuntimeError::type_error(
                    "int +| int",
                    &format!("{} +| {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::SubSat => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.saturating_sub(*b))),
                _ => Err(RuntimeError::type_error(
                    "int -| int",
                    &format!("{} -| {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::MulSat => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a.saturating_mul(*b))),
                _ => Err(RuntimeError::type_error(
                    "int *| int",
                    &format!("{} *| {}", left.type_name(), right.type_name()),
                )),
            },

            // Comparison
            // v0.89.4: Cross-type numeric coercion for Eq/Ne (matches Ge/Le/Lt/Gt behavior)
            // sqrt(4) == 2 was returning false because Int(2) != Float(2.0)
            BinOp::Eq => {
                match (&left, &right) {
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(*a as f64 == *b)),
                    (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a == *b as f64)),
                    _ => Ok(Value::Bool(left == right)),
                }
            }
            BinOp::Ne => {
                match (&left, &right) {
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(*a as f64 != *b)),
                    (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a != *b as f64)),
                    _ => Ok(Value::Bool(left != right)),
                }
            }
            BinOp::Lt => self.compare_values(&left, &right, |a, b| a < b),
            BinOp::Gt => self.compare_values(&left, &right, |a, b| a > b),
            BinOp::Le => self.compare_values(&left, &right, |a, b| a <= b),
            BinOp::Ge => self.compare_values(&left, &right, |a, b| a >= b),

            // Logical
            BinOp::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),

            // v0.32: Shift operators
            BinOp::Shl => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a << b)),
                _ => Err(RuntimeError::type_error(
                    "int << int",
                    &format!("{} << {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::Shr => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a >> b)),
                _ => Err(RuntimeError::type_error(
                    "int >> int",
                    &format!("{} >> {}", left.type_name(), right.type_name()),
                )),
            },

            // v0.36: Bitwise operators
            BinOp::Band => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                _ => Err(RuntimeError::type_error(
                    "int band int",
                    &format!("{} band {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::Bor => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                _ => Err(RuntimeError::type_error(
                    "int bor int",
                    &format!("{} bor {}", left.type_name(), right.type_name()),
                )),
            },
            BinOp::Bxor => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
                _ => Err(RuntimeError::type_error(
                    "int bxor int",
                    &format!("{} bxor {}", left.type_name(), right.type_name()),
                )),
            },

            // v0.36: Logical implication (P implies Q = not P or Q)
            BinOp::Implies => Ok(Value::Bool(!left.is_truthy() || right.is_truthy())),
        }
    }

    /// Compare two values
    fn compare_values<F>(&self, left: &Value, right: &Value, f: F) -> InterpResult<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(f(*a as f64, *b as f64))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(f(*a, *b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(f(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(f(*a, *b as f64))),
            // v0.64: Character comparison (by Unicode codepoint)
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(f(*a as u32 as f64, *b as u32 as f64))),
            _ => Err(RuntimeError::type_error(
                "numeric",
                &format!("{} cmp {}", left.type_name(), right.type_name()),
            )),
        }
    }

    /// Evaluate unary operation
    fn eval_unary(&self, op: UnOp, val: Value) -> InterpResult<Value> {
        match op {
            UnOp::Neg => match val {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(RuntimeError::type_error("numeric", val.type_name())),
            },
            UnOp::Not => Ok(Value::Bool(!val.is_truthy())),
            // v0.36: Bitwise not
            UnOp::Bnot => match val {
                Value::Int(n) => Ok(Value::Int(!n)),
                _ => Err(RuntimeError::type_error("int", val.type_name())),
            },
        }
    }

    /// Get the global environment (for REPL)
    pub fn global_env(&self) -> &EnvRef {
        &self.global_env
    }

    /// Define a function (for REPL)
    pub fn define_function(&mut self, fn_def: FnDef) {
        self.functions.insert(fn_def.name.node.clone(), fn_def);
    }

    // ============ v0.30.280: ScopeStack-based Fast Evaluation ============

    /// Evaluate an expression using ScopeStack for efficient memory
    fn eval_fast(&mut self, expr: &Spanned<Expr>) -> InterpResult<Value> {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROW_SIZE, || self.eval_fast_inner(expr))
    }

    /// Inner fast eval implementation using ScopeStack
    fn eval_fast_inner(&mut self, expr: &Spanned<Expr>) -> InterpResult<Value> {
        match &expr.node {
            Expr::IntLit(n) => Ok(Value::Int(*n)),
            Expr::FloatLit(f) => Ok(Value::Float(*f)),
            Expr::BoolLit(b) => Ok(Value::Bool(*b)),
            Expr::StringLit(s) => Ok(Value::Str(self.intern_string(s))),
            // v0.64: Character literal evaluation
            Expr::CharLit(c) => Ok(Value::Char(*c)),
            Expr::Unit => Ok(Value::Unit),

            Expr::Var(name) => {
                self.scope_stack
                    .get(name)
                    .ok_or_else(|| RuntimeError::undefined_variable(name))
            }

            Expr::Binary { left, op, right } => {
                match op {
                    BinOp::And => {
                        let lval = self.eval_fast(left)?;
                        if !lval.is_truthy() {
                            return Ok(Value::Bool(false));
                        }
                        let rval = self.eval_fast(right)?;
                        Ok(Value::Bool(rval.is_truthy()))
                    }
                    BinOp::Or => {
                        let lval = self.eval_fast(left)?;
                        if lval.is_truthy() {
                            return Ok(Value::Bool(true));
                        }
                        let rval = self.eval_fast(right)?;
                        Ok(Value::Bool(rval.is_truthy()))
                    }
                    _ => {
                        let lval = self.eval_fast(left)?;
                        let rval = self.eval_fast(right)?;
                        self.eval_binary(*op, lval, rval)
                    }
                }
            }

            Expr::Unary { op, expr: inner } => {
                let val = self.eval_fast(inner)?;
                self.eval_unary(*op, val)
            }

            Expr::If { cond, then_branch, else_branch } => {
                let cond_val = self.eval_fast(cond)?;
                if cond_val.is_truthy() {
                    self.eval_fast(then_branch)
                } else {
                    self.eval_fast(else_branch)
                }
            }

            // v0.34.2.3: Define in current scope (Block manages scope)
            // Fix: Don't push/pop scope for Let - let Block handle scoping
            // This allows sequential let statements to share variables
            Expr::Let { name, value, body, .. } => {
                let val = self.eval_fast(value)?;
                self.scope_stack.define(name.clone(), val);
                self.eval_fast(body)
            }

            // v0.60.21: Uninitialized let binding for stack arrays (fast path)
            Expr::LetUninit { name, mutable: _, ty, body } => {
                // Create uninitialized array based on type
                let val = match &ty.node {
                    crate::ast::Type::Array(_, size) => {
                        // Create array filled with zeros
                        Value::Array(vec![Value::Int(0); *size])
                    }
                    _ => Value::Unit, // Shouldn't happen - type checker ensures array type
                };
                self.scope_stack.define(name.clone(), val);
                self.eval_fast(body)
            }

            Expr::Call { func, args } => {
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_fast(a))
                    .collect::<InterpResult<Vec<_>>>()?;
                self.call_fast(func, arg_vals)
            }

            Expr::MethodCall { receiver, method, args } => {
                let recv_val = self.eval_fast(receiver)?;
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_fast(a))
                    .collect::<InterpResult<Vec<_>>>()?;
                self.eval_method_call(recv_val, method, arg_vals)
            }

            // v0.30.280: Block expression - immediate scope deallocation
            Expr::Block(exprs) => {
                self.scope_stack.push_scope();
                let mut result = Value::Unit;
                for e in exprs {
                    result = self.eval_fast(e)?;
                }
                self.scope_stack.pop_scope();
                Ok(result)
            }

            // v0.30.280: Assignment using ScopeStack
            Expr::Assign { name, value } => {
                let val = self.eval_fast(value)?;
                if !self.scope_stack.set(name, val.clone()) {
                    return Err(RuntimeError::undefined_variable(name));
                }
                Ok(Value::Unit)
            }

            // v0.30.280: While loop using ScopeStack
            // v0.37: Invariant is for SMT verification, not runtime
            Expr::While { cond, invariant: _, body } => {
                while self.eval_fast(cond)?.is_truthy() {
                    match self.eval_fast(body) {
                        Ok(_) => {},
                        Err(e) if matches!(e.kind, ErrorKind::Continue) => continue,
                        Err(e) if matches!(e.kind, ErrorKind::Break(_)) => break,
                        Err(e) => return Err(e),
                    }
                }
                Ok(Value::Unit)
            }

            // v0.30.280: Match expression using ScopeStack
            Expr::Match { expr: match_expr, arms } => {
                let val = self.eval_fast(match_expr)?;
                for arm in arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern.node, &val) {
                        self.scope_stack.push_scope();
                        for (name, bound_val) in bindings {
                            self.scope_stack.define(name, bound_val);
                        }
                        // v0.40: Check pattern guard if present
                        if let Some(guard) = &arm.guard {
                            let guard_result = self.eval_fast(guard)?;
                            if !guard_result.is_truthy() {
                                self.scope_stack.pop_scope();
                                continue; // Guard failed, try next arm
                            }
                        }
                        let result = self.eval_fast(&arm.body);
                        self.scope_stack.pop_scope();
                        return result;
                    }
                }
                Err(RuntimeError::type_error("matching arm", "no match found"))
            }

            // v0.30.280: Struct support
            Expr::StructInit { name, fields } => {
                let mut field_values = std::collections::HashMap::new();
                for (field_name, field_expr) in fields {
                    let val = self.eval_fast(field_expr)?;
                    field_values.insert(field_name.node.clone(), val);
                }
                Ok(Value::Struct(name.clone(), field_values))
            }

            Expr::FieldAccess { expr: obj_expr, field } => {
                let obj = self.eval_fast(obj_expr)?;
                match obj {
                    Value::Struct(_, fields) => {
                        fields.get(&field.node).cloned()
                            .ok_or_else(|| RuntimeError::type_error("field", &field.node))
                    }
                    _ => Err(RuntimeError::type_error("struct", obj.type_name())),
                }
            }

            // v0.51.23: Field assignment (eval_fast version)
            Expr::FieldAssign { object, field, value } => {
                // Get the new value first
                let new_val = self.eval_fast(value)?;

                // The object must be a variable for field assignment to work
                if let Expr::Var(var_name) = &object.node {
                    // Get the struct from the scope stack
                    let struct_val = self.scope_stack.get(var_name)
                        .ok_or_else(|| RuntimeError::undefined_variable(var_name))?;

                    // Modify the field
                    match struct_val {
                        Value::Struct(struct_name, mut fields) => {
                            if !fields.contains_key(&field.node) {
                                return Err(RuntimeError::type_error("field", &field.node));
                            }
                            fields.insert(field.node.clone(), new_val);
                            // Store the modified struct back
                            self.scope_stack.set(var_name, Value::Struct(struct_name, fields));
                            Ok(Value::Unit)
                        }
                        _ => Err(RuntimeError::type_error("struct", struct_val.type_name())),
                    }
                } else {
                    Err(RuntimeError::type_error("variable", "complex expression"))
                }
            }

            // v0.60.21: Dereference assignment (eval_fast version)
            Expr::DerefAssign { ptr, value } => {
                let ptr_val = self.eval_fast(ptr)?;
                let new_val = self.eval_fast(value)?;

                match ptr_val {
                    Value::Int(addr) if addr != 0 => {
                        self.heap.borrow_mut().insert(addr, new_val);
                        Ok(Value::Unit)
                    }
                    Value::Int(0) => Err(RuntimeError::type_error("non-null pointer", "null")),
                    _ => Err(RuntimeError::type_error("pointer", ptr_val.type_name())),
                }
            }

            // v0.43: Tuple field access
            Expr::TupleField { expr: tuple_expr, index } => {
                let tuple_val = self.eval_fast(tuple_expr)?;
                match tuple_val {
                    Value::Tuple(elems) => {
                        elems.get(*index).cloned()
                            .ok_or_else(|| RuntimeError::index_out_of_bounds(*index as i64, elems.len()))
                    }
                    _ => Err(RuntimeError::type_error("tuple", tuple_val.type_name())),
                }
            }

            // v0.30.280: Enum support
            Expr::EnumVariant { enum_name, variant, args } => {
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_fast(a))
                    .collect::<InterpResult<Vec<_>>>()?;
                Ok(Value::Enum(enum_name.clone(), variant.clone(), arg_vals))
            }

            // v0.30.280: Array support
            Expr::ArrayLit(elems) => {
                let mut values = Vec::new();
                for elem in elems {
                    values.push(self.eval_fast(elem)?);
                }
                Ok(Value::Array(values))
            }

            // v0.60.22: Array repeat [val; N]
            Expr::ArrayRepeat { value, count } => {
                let val = self.eval_fast(value)?;
                let values = vec![val; *count];
                Ok(Value::Array(values))
            }

            // v0.42: Tuple expressions
            Expr::Tuple(elems) => {
                let mut values = Vec::new();
                for elem in elems {
                    values.push(self.eval_fast(elem)?);
                }
                Ok(Value::Tuple(values))
            }

            Expr::Index { expr, index } => {
                let arr_val = self.eval_fast(expr)?;
                let idx_val = self.eval_fast(index)?;
                let idx = match idx_val {
                    Value::Int(n) => n as usize,
                    _ => return Err(RuntimeError::type_error("integer", idx_val.type_name())),
                };

                // v0.50.26: Dereference if indexing through a reference
                let derefed_val = match &arr_val {
                    Value::Ref(r) => r.borrow().clone(),
                    _ => arr_val,
                };

                match derefed_val {
                    Value::Array(arr) => {
                        if idx < arr.len() {
                            Ok(arr[idx].clone())
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, arr.len()))
                        }
                    }
                    Value::Str(s) => {
                        if idx < s.len() {
                            Ok(Value::Int(s.as_bytes()[idx] as i64))
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, s.len()))
                        }
                    }
                    // v0.93: Handle StringRope (lazy concatenated strings)
                    Value::StringRope(_) => {
                        let s = derefed_val.materialize_string()
                            .ok_or_else(|| RuntimeError::type_error("string", "invalid StringRope"))?;
                        if idx < s.len() {
                            Ok(Value::Int(s.as_bytes()[idx] as i64))
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, s.len()))
                        }
                    }
                    // v0.89.5: Typed pointer indexing (*i64) — ptr[i] does load_i64(ptr + i*8)
                    Value::Int(ptr) => {
                        if ptr == 0 {
                            return Err(RuntimeError::io_error("index: null pointer dereference"));
                        }
                        let addr = ptr + (idx as i64) * 8;
                        let value = unsafe {
                            let p = addr as *const i64;
                            *p
                        };
                        Ok(Value::Int(value))
                    }
                    _ => Err(RuntimeError::type_error("array, string, or pointer", derefed_val.type_name())),
                }
            }

            // v0.90.24: IndexAssign in eval_fast (scope-stack path)
            Expr::IndexAssign { array, index, value } => {
                let arr_name = match &array.node {
                    Expr::Var(name) => name.clone(),
                    _ => return Err(RuntimeError::type_error("variable", "complex expression")),
                };

                let idx_val = self.eval_fast(index)?;
                let new_val = self.eval_fast(value)?;

                let idx = match idx_val {
                    Value::Int(n) => n as usize,
                    _ => return Err(RuntimeError::type_error("integer", idx_val.type_name())),
                };

                let lookup = self.scope_stack.get(&arr_name);
                match lookup {
                    Some(Value::Int(ptr)) => {
                        if ptr == 0 {
                            return Err(RuntimeError::io_error("index assign: null pointer dereference"));
                        }
                        let write_val = match new_val {
                            Value::Int(v) => v,
                            _ => return Err(RuntimeError::type_error("i64", new_val.type_name())),
                        };
                        let addr = ptr + (idx as i64) * 8;
                        unsafe {
                            let p = addr as *mut i64;
                            *p = write_val;
                        }
                        Ok(Value::Unit)
                    }
                    Some(Value::Array(a)) => {
                        let mut arr = a;
                        if idx < arr.len() {
                            arr[idx] = new_val;
                            self.scope_stack.set(&arr_name, Value::Array(arr));
                            Ok(Value::Unit)
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, arr.len()))
                        }
                    }
                    Some(other) => Err(RuntimeError::type_error("array or pointer", other.type_name())),
                    None => Err(RuntimeError::undefined_variable(&arr_name)),
                }
            }

            Expr::Loop { body } => {
                loop {
                    match self.eval_fast(body) {
                        Ok(_) => continue,
                        Err(e) if matches!(e.kind, ErrorKind::Continue) => continue,
                        Err(e) if matches!(e.kind, ErrorKind::Break(_)) => {
                            if let ErrorKind::Break(val) = e.kind {
                                return Ok(val.map(|v| *v).unwrap_or(Value::Unit));
                            }
                            return Ok(Value::Unit);
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            Expr::Break { value } => {
                let val = match value {
                    Some(v) => Some(Box::new(self.eval_fast(v)?)),
                    None => None,
                };
                Err(RuntimeError { kind: ErrorKind::Break(val), message: "break".to_string() })
            }

            Expr::Continue => {
                Err(RuntimeError { kind: ErrorKind::Continue, message: "continue".to_string() })
            }

            Expr::Return { value } => {
                let val = match value {
                    Some(v) => self.eval_fast(v)?,
                    None => Value::Unit,
                };
                Err(RuntimeError { kind: ErrorKind::Return(Box::new(val)), message: "return".to_string() })
            }

            // For unsupported expressions, return error (force explicit handling)
            _ => Err(RuntimeError::type_error(
                "supported expression in fast path",
                "unsupported expression (Range, For, Ref, Closure, etc.)"
            ))
        }
    }

    /// Call a function by name using ScopeStack
    fn call_fast(&mut self, name: &str, args: Vec<Value>) -> InterpResult<Value> {
        if let Some(builtin) = self.builtins.get(name) {
            return builtin(&args);
        }
        if let Some(fn_def) = self.functions.get(name).cloned() {
            return self.call_function_fast(&fn_def, &args);
        }
        Err(RuntimeError::undefined_function(name))
    }

    /// Call a user-defined function using ScopeStack
    fn call_function_fast(&mut self, fn_def: &FnDef, args: &[Value]) -> InterpResult<Value> {
        if fn_def.params.len() != args.len() {
            return Err(RuntimeError::arity_mismatch(
                &fn_def.name.node,
                fn_def.params.len(),
                args.len(),
            ));
        }

        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            self.recursion_depth -= 1;
            return Err(RuntimeError::stack_overflow());
        }

        self.scope_stack.push_scope();
        for (param, arg) in fn_def.params.iter().zip(args.iter()) {
            self.scope_stack.define(param.name.node.clone(), arg.clone());
        }

        // Catch Return control flow
        let result = match self.eval_fast(&fn_def.body) {
            Ok(v) => Ok(v),
            Err(e) if matches!(e.kind, ErrorKind::Return(_)) => {
                if let ErrorKind::Return(val) = e.kind {
                    Ok(*val)
                } else {
                    unreachable!()
                }
            }
            Err(e) => Err(e),
        };
        self.scope_stack.pop_scope();
        self.recursion_depth -= 1;
        result
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Built-in Functions ============

fn builtin_print(args: &[Value]) -> InterpResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{arg}");
    }
    io::stdout().flush().map_err(|e| RuntimeError::io_error(&e.to_string()))?;
    Ok(Value::Unit)
}

fn builtin_println(args: &[Value]) -> InterpResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{arg}");
    }
    println!();
    Ok(Value::Unit)
}

/// print_str(s: String) -> i64
/// Prints a string without newline. Returns 0 on success.
/// v0.31.21: Added for gotgan string output
fn builtin_print_str(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("print_str", 1, args.len()));
    }
    // v0.35.5: Handle both Value::Str and Value::StringRope using materialize_string
    if let Some(s) = args[0].materialize_string() {
        print!("{}", s);
        io::stdout().flush().map_err(|e| RuntimeError::io_error(&e.to_string()))?;
        Ok(Value::Int(0))
    } else {
        Err(RuntimeError::type_error("String", args[0].type_name()))
    }
}

/// println_str(s: String) -> Unit
/// Prints a string with newline.
/// v0.100: Added for string output consistency
fn builtin_println_str(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("println_str", 1, args.len()));
    }
    if let Some(s) = args[0].materialize_string() {
        println!("{}", s);
        Ok(Value::Unit)
    } else {
        Err(RuntimeError::type_error("String", args[0].type_name()))
    }
}

/// println_f64(f: f64) -> Unit
/// Prints a float with newline (9 decimal places).
/// v0.60.44: Added for spectral_norm, n_body benchmarks
fn builtin_println_f64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("println_f64", 1, args.len()));
    }
    match &args[0] {
        Value::Float(f) => {
            println!("{:.9}", f);
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("f64", args[0].type_name())),
    }
}

/// print_f64(f: f64) -> Unit
/// Prints a float without newline (9 decimal places).
/// v0.60.44: Added for spectral_norm, n_body benchmarks
fn builtin_print_f64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("print_f64", 1, args.len()));
    }
    match &args[0] {
        Value::Float(f) => {
            print!("{:.9}", f);
            io::stdout().flush().map_err(|e| RuntimeError::io_error(&e.to_string()))?;
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("f64", args[0].type_name())),
    }
}

fn builtin_assert(args: &[Value]) -> InterpResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::arity_mismatch("assert", 1, 0));
    }
    if !args[0].is_truthy() {
        return Err(RuntimeError::assertion_failed(None));
    }
    Ok(Value::Unit)
}

fn builtin_read_int(_args: &[Value]) -> InterpResult<Value> {
    let stdin = io::stdin();
    let line = stdin
        .lock()
        .lines()
        .next()
        .ok_or_else(|| RuntimeError::io_error("end of input"))?
        .map_err(|e| RuntimeError::io_error(&e.to_string()))?;

    line.trim()
        .parse::<i64>()
        .map(Value::Int)
        .map_err(|_| RuntimeError::type_error("integer", "invalid input"))
}

fn builtin_abs(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("abs", 1, args.len()));
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(RuntimeError::type_error("numeric", args[0].type_name())),
    }
}

fn builtin_min(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("min", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        _ => Err(RuntimeError::type_error("numeric", "mixed types")),
    }
}

fn builtin_max(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("max", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        _ => Err(RuntimeError::type_error("numeric", "mixed types")),
    }
}

// ============ v0.34: Math Intrinsics for Phase 34.4 Benchmark Gate ============

/// sqrt(x: f64) -> f64
/// Returns the square root of a floating-point number.
/// Returns NaN for negative inputs.
fn builtin_sqrt(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("sqrt", 1, args.len()));
    }
    match &args[0] {
        Value::Float(f) => Ok(Value::Float(f.sqrt())),
        Value::Int(n) => Ok(Value::Float((*n as f64).sqrt())),
        _ => Err(RuntimeError::type_error("f64", args[0].type_name())),
    }
}

/// i64_to_f64(x: i64) -> f64
/// Converts an integer to a floating-point number.
fn builtin_i64_to_f64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("i64_to_f64", 1, args.len()));
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// f64_to_i64(x: f64) -> i64
/// Converts a floating-point number to an integer (truncates toward zero).
fn builtin_f64_to_i64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("f64_to_i64", 1, args.len()));
    }
    match &args[0] {
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        _ => Err(RuntimeError::type_error("f64", args[0].type_name())),
    }
}

/// v0.51.47: i32_to_f64(x: i32) -> f64
/// Converts a 32-bit integer to a floating-point number.
/// In the interpreter, integers are stored as i64, but we treat them as i32.
fn builtin_i32_to_f64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("i32_to_f64", 1, args.len()));
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Float((*n as i32) as f64)),
        _ => Err(RuntimeError::type_error("i32", args[0].type_name())),
    }
}

/// v0.51.47: i32_to_i64(x: i32) -> i64
/// Sign-extends a 32-bit integer to 64-bit.
/// In the interpreter, this is a no-op since we store all ints as i64.
fn builtin_i32_to_i64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("i32_to_i64", 1, args.len()));
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Int((*n as i32) as i64)), // sign-extend
        _ => Err(RuntimeError::type_error("i32", args[0].type_name())),
    }
}

/// v0.51.47: i64_to_i32(x: i64) -> i32
/// Truncates a 64-bit integer to 32-bit.
/// In the interpreter, we still return i64 but truncated value.
fn builtin_i64_to_i32(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("i64_to_i32", 1, args.len()));
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Int((*n as i32) as i64)), // truncate then sign-extend back
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

// ============ v0.34.2: Memory Allocation Builtins for Phase 34.2 Dynamic Collections ============

/// malloc(size: i64) -> i64 (pointer as integer)
/// Allocates `size` bytes and returns the pointer as an i64.
/// In the interpreter, we use Rust's allocator.
fn builtin_malloc(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("malloc", 1, args.len()));
    }
    match &args[0] {
        Value::Int(size) => {
            if *size <= 0 {
                return Ok(Value::Int(0)); // NULL for invalid size
            }
            let layout = std::alloc::Layout::from_size_align(*size as usize, 8)
                .map_err(|_| RuntimeError::io_error("malloc: invalid allocation size"))?;
            let ptr = unsafe { std::alloc::alloc(layout) };
            if ptr.is_null() {
                Ok(Value::Int(0)) // NULL
            } else {
                Ok(Value::Int(ptr as i64))
            }
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// free(ptr: i64) -> i64
/// Frees memory allocated by malloc. Returns 0.
/// v0.89.4: Changed from Unit to i64(0) so free() can be used as expression
/// Note: In the interpreter, we intentionally leak memory for safety.
/// Native compilation uses real libc free.
fn builtin_free(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("free", 1, args.len()));
    }
    match &args[0] {
        Value::Int(_ptr) => {
            // Intentionally do nothing in interpreter for memory safety
            // Real free happens in native compiled code via libc
            Ok(Value::Int(0))
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// realloc(ptr: i64, new_size: i64) -> i64
/// Reallocates memory to new_size. In interpreter, allocates new and leaks old.
fn builtin_realloc(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("realloc", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(_old_ptr), Value::Int(new_size)) => {
            // For interpreter simplicity, just allocate new memory
            // Native compilation uses real libc realloc
            if *new_size <= 0 {
                return Ok(Value::Int(0)); // NULL
            }
            let layout = std::alloc::Layout::from_size_align(*new_size as usize, 8)
                .map_err(|_| RuntimeError::io_error("realloc: invalid allocation size"))?;
            let ptr = unsafe { std::alloc::alloc(layout) };
            if ptr.is_null() {
                Ok(Value::Int(0)) // NULL
            } else {
                Ok(Value::Int(ptr as i64))
            }
        }
        _ => Err(RuntimeError::type_error("i64, i64", "other")),
    }
}

/// calloc(count: i64, size: i64) -> i64
/// Allocates zeroed memory for count elements of size bytes each.
fn builtin_calloc(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("calloc", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(count), Value::Int(size)) => {
            let total = (*count as usize).saturating_mul(*size as usize);
            if total == 0 {
                return Ok(Value::Int(0)); // NULL for zero size
            }
            let layout = std::alloc::Layout::from_size_align(total, 8)
                .map_err(|_| RuntimeError::io_error("calloc: invalid allocation size"))?;
            let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
            if ptr.is_null() {
                Ok(Value::Int(0)) // NULL
            } else {
                Ok(Value::Int(ptr as i64))
            }
        }
        _ => Err(RuntimeError::type_error("i64, i64", "other")),
    }
}

/// store_i64(ptr: i64, value: i64) -> i64
/// Stores an i64 value at the given memory address. Returns 0.
/// v0.89.4: Changed from Unit to i64(0) for expression use
fn builtin_store_i64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("store_i64", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(ptr), Value::Int(value)) => {
            if *ptr == 0 {
                return Err(RuntimeError::io_error("store_i64: null pointer dereference"));
            }
            unsafe {
                let p = *ptr as *mut i64;
                *p = *value;
            }
            Ok(Value::Int(0))
        }
        _ => Err(RuntimeError::type_error("i64, i64", "other")),
    }
}

/// load_i64(ptr: i64) -> i64
/// Loads an i64 value from the given memory address.
fn builtin_load_i64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("load_i64", 1, args.len()));
    }
    match &args[0] {
        Value::Int(ptr) => {
            if *ptr == 0 {
                return Err(RuntimeError::io_error("load_i64: null pointer dereference"));
            }
            let value = unsafe {
                let p = *ptr as *const i64;
                *p
            };
            Ok(Value::Int(value))
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// store_f64(ptr: i64, value: f64) -> i64
/// Stores an f64 value at the given memory address. Returns 0.
/// v0.89.4: Changed from Unit to i64(0) for expression use
fn builtin_store_f64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("store_f64", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(ptr), Value::Float(value)) => {
            if *ptr == 0 {
                return Err(RuntimeError::io_error("store_f64: null pointer dereference"));
            }
            unsafe {
                let p = *ptr as *mut f64;
                *p = *value;
            }
            Ok(Value::Int(0))
        }
        _ => Err(RuntimeError::type_error("i64, f64", "other")),
    }
}

/// load_f64(ptr: i64) -> f64
/// Loads an f64 value from the given memory address.
/// v0.51.5: Added for numerical benchmark fairness (n_body, spectral_norm)
fn builtin_load_f64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("load_f64", 1, args.len()));
    }
    match &args[0] {
        Value::Int(ptr) => {
            if *ptr == 0 {
                return Err(RuntimeError::io_error("load_f64: null pointer dereference"));
            }
            let value = unsafe {
                let p = *ptr as *const f64;
                *p
            };
            Ok(Value::Float(value))
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// box_new_i64(value: i64) -> i64
/// Allocates 8 bytes on the heap, stores the value, and returns the pointer.
/// This is a convenience wrapper: malloc(8) + store_i64(ptr, value)
fn builtin_box_new_i64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("box_new_i64", 1, args.len()));
    }
    match &args[0] {
        Value::Int(value) => {
            // Allocate 8 bytes for one i64
            let layout = std::alloc::Layout::from_size_align(8, 8)
                .map_err(|_| RuntimeError::io_error("box_new_i64: invalid allocation size"))?;
            let ptr = unsafe { std::alloc::alloc(layout) };
            if ptr.is_null() {
                return Ok(Value::Int(0)); // NULL
            }
            // Store the value
            unsafe {
                let p = ptr as *mut i64;
                *p = *value;
            }
            Ok(Value::Int(ptr as i64))
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

// ============ v0.34.2.3: Vec<i64> Dynamic Array Builtins (RFC-0007) ============
//
// Memory Layout:
// Vec header (24 bytes, heap-allocated):
//   offset 0: ptr (i64) - pointer to data array
//   offset 8: len (i64) - current number of elements
//   offset 16: cap (i64) - allocated capacity
//
// Data array (heap-allocated, cap * 8 bytes):
//   [i64; cap] - actual element storage
//

/// vec_new() -> i64: Create empty vector, returns header pointer
fn builtin_vec_new(args: &[Value]) -> InterpResult<Value> {
    if !args.is_empty() {
        return Err(RuntimeError::arity_mismatch("vec_new", 0, args.len()));
    }
    // Allocate 24 bytes for header (ptr, len, cap)
    let layout = std::alloc::Layout::from_size_align(24, 8)
        .map_err(|_| RuntimeError::io_error("vec_new: invalid allocation size"))?;
    let header = unsafe { std::alloc::alloc_zeroed(layout) };
    if header.is_null() {
        return Ok(Value::Int(0)); // NULL
    }
    // Header is already zeroed: ptr=0, len=0, cap=0
    Ok(Value::Int(header as i64))
}

/// vec_with_capacity(cap: i64) -> i64: Create vector with pre-allocated capacity
fn builtin_vec_with_capacity(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("vec_with_capacity", 1, args.len()));
    }
    match &args[0] {
        Value::Int(cap) => {
            if *cap < 0 {
                return Err(RuntimeError::io_error("vec_with_capacity: negative capacity"));
            }
            // Allocate header
            let header_layout = std::alloc::Layout::from_size_align(24, 8)
                .map_err(|_| RuntimeError::io_error("vec_with_capacity: invalid header size"))?;
            let header = unsafe { std::alloc::alloc(header_layout) };
            if header.is_null() {
                return Ok(Value::Int(0));
            }

            // Allocate data array if capacity > 0
            let data_ptr = if *cap > 0 {
                let data_layout = std::alloc::Layout::from_size_align((*cap as usize) * 8, 8)
                    .map_err(|_| RuntimeError::io_error("vec_with_capacity: invalid data size"))?;
                let data = unsafe { std::alloc::alloc(data_layout) };
                if data.is_null() {
                    // Free header and return NULL
                    unsafe { std::alloc::dealloc(header, header_layout) };
                    return Ok(Value::Int(0));
                }
                data as i64
            } else {
                0i64
            };

            // Initialize header: ptr, len=0, cap
            unsafe {
                let h = header as *mut i64;
                *h = data_ptr;           // offset 0: ptr
                *h.add(1) = 0;           // offset 8: len
                *h.add(2) = *cap;        // offset 16: cap
            }
            Ok(Value::Int(header as i64))
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// vec_push(vec: i64, value: i64) -> Unit: Append element with auto-grow
fn builtin_vec_push(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("vec_push", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(vec_ptr), Value::Int(value)) => {
            if *vec_ptr == 0 {
                return Err(RuntimeError::io_error("vec_push: null vector"));
            }
            unsafe {
                let header = *vec_ptr as *mut i64;
                let ptr = *header;           // data pointer
                let len = *header.add(1);    // current length
                let cap = *header.add(2);    // capacity

                // Check if we need to grow
                if len >= cap {
                    // Grow strategy: 0 -> 4 -> 8 -> 16 -> 32 -> ...
                    let new_cap = if cap == 0 { 4 } else { cap * 2 };
                    let new_size = (new_cap as usize) * 8;

                    let new_data = if ptr == 0 {
                        // First allocation
                        let layout = std::alloc::Layout::from_size_align(new_size, 8)
                            .map_err(|_| RuntimeError::io_error("vec_push: allocation failed"))?;
                        std::alloc::alloc(layout)
                    } else {
                        // Realloc existing
                        let old_layout = std::alloc::Layout::from_size_align((cap as usize) * 8, 8)
                            .map_err(|_| RuntimeError::io_error("vec_push: invalid old layout"))?;
                        std::alloc::realloc(ptr as *mut u8, old_layout, new_size)
                    };

                    if new_data.is_null() {
                        return Err(RuntimeError::io_error("vec_push: out of memory"));
                    }

                    // Update header
                    *header = new_data as i64;
                    *header.add(2) = new_cap;
                }

                // Store value at data[len]
                let data = *header as *mut i64;
                let len = *header.add(1);
                *data.add(len as usize) = *value;

                // Increment length
                *header.add(1) = len + 1;
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("i64, i64", "other")),
    }
}

/// vec_pop(vec: i64) -> i64: Remove and return last element
fn builtin_vec_pop(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("vec_pop", 1, args.len()));
    }
    match &args[0] {
        Value::Int(vec_ptr) => {
            if *vec_ptr == 0 {
                return Err(RuntimeError::io_error("vec_pop: null vector"));
            }
            unsafe {
                let header = *vec_ptr as *mut i64;
                let ptr = *header;
                let len = *header.add(1);

                if len <= 0 {
                    return Err(RuntimeError::io_error("vec_pop: empty vector"));
                }

                // Get last element
                let data = ptr as *const i64;
                let value = *data.add((len - 1) as usize);

                // Decrement length
                *header.add(1) = len - 1;

                Ok(Value::Int(value))
            }
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// vec_get(vec: i64, index: i64) -> i64: Read element at index
fn builtin_vec_get(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("vec_get", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(vec_ptr), Value::Int(index)) => {
            if *vec_ptr == 0 {
                return Err(RuntimeError::io_error("vec_get: null vector"));
            }
            unsafe {
                let header = *vec_ptr as *const i64;
                let ptr = *header;
                let len = *header.add(1);

                if *index < 0 || *index >= len {
                    return Err(RuntimeError::io_error(&format!(
                        "vec_get: index {} out of bounds (len={})", index, len
                    )));
                }

                let data = ptr as *const i64;
                Ok(Value::Int(*data.add(*index as usize)))
            }
        }
        _ => Err(RuntimeError::type_error("i64, i64", "other")),
    }
}

/// vec_set(vec: i64, index: i64, value: i64) -> Unit: Write element at index
fn builtin_vec_set(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::arity_mismatch("vec_set", 3, args.len()));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::Int(vec_ptr), Value::Int(index), Value::Int(value)) => {
            if *vec_ptr == 0 {
                return Err(RuntimeError::io_error("vec_set: null vector"));
            }
            unsafe {
                let header = *vec_ptr as *mut i64;
                let ptr = *header;
                let len = *header.add(1);

                if *index < 0 || *index >= len {
                    return Err(RuntimeError::io_error(&format!(
                        "vec_set: index {} out of bounds (len={})", index, len
                    )));
                }

                let data = ptr as *mut i64;
                *data.add(*index as usize) = *value;
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("i64, i64, i64", "other")),
    }
}

/// vec_len(vec: i64) -> i64: Get current length
fn builtin_vec_len(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("vec_len", 1, args.len()));
    }
    match &args[0] {
        Value::Int(vec_ptr) => {
            if *vec_ptr == 0 {
                return Ok(Value::Int(0)); // NULL vec has len 0
            }
            unsafe {
                let header = *vec_ptr as *const i64;
                Ok(Value::Int(*header.add(1)))
            }
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// vec_cap(vec: i64) -> i64: Get capacity
fn builtin_vec_cap(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("vec_cap", 1, args.len()));
    }
    match &args[0] {
        Value::Int(vec_ptr) => {
            if *vec_ptr == 0 {
                return Ok(Value::Int(0)); // NULL vec has cap 0
            }
            unsafe {
                let header = *vec_ptr as *const i64;
                Ok(Value::Int(*header.add(2)))
            }
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// vec_free(vec: i64) -> Unit: Deallocate vector and its data
fn builtin_vec_free(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("vec_free", 1, args.len()));
    }
    match &args[0] {
        Value::Int(vec_ptr) => {
            if *vec_ptr == 0 {
                return Ok(Value::Unit); // NULL is no-op
            }
            unsafe {
                let header = *vec_ptr as *mut i64;
                let ptr = *header;
                let cap = *header.add(2);

                // Free data array if allocated
                if ptr != 0 && cap > 0 {
                    let data_layout = std::alloc::Layout::from_size_align((cap as usize) * 8, 8)
                        .map_err(|_| RuntimeError::io_error("vec_free: invalid data layout"))?;
                    std::alloc::dealloc(ptr as *mut u8, data_layout);
                }

                // Free header
                let header_layout = std::alloc::Layout::from_size_align(24, 8)
                    .map_err(|_| RuntimeError::io_error("vec_free: invalid header layout"))?;
                std::alloc::dealloc(*vec_ptr as *mut u8, header_layout);
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// vec_clear(vec: i64) -> Unit: Set length to 0 without deallocating
fn builtin_vec_clear(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("vec_clear", 1, args.len()));
    }
    match &args[0] {
        Value::Int(vec_ptr) => {
            if *vec_ptr == 0 {
                return Err(RuntimeError::io_error("vec_clear: null vector"));
            }
            unsafe {
                let header = *vec_ptr as *mut i64;
                // Set len to 0, keep capacity
                *header.add(1) = 0;
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

// ============ v0.34.24: Hash Builtins ============

/// hash_i64(x: i64) -> i64: Hash function for integers
/// Uses FNV-1a style multiplication hash
fn builtin_hash_i64(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("hash_i64", 1, args.len()));
    }
    match &args[0] {
        Value::Int(x) => {
            // FNV-1a inspired hash: multiply by prime, xor with shifted value
            let h = (*x as u64).wrapping_mul(0x517cc1b727220a95);
            let result = (h ^ (h >> 32)) as i64;
            Ok(Value::Int(result))
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

// ============ v0.34.24: HashMap Builtins ============
// Layout: [count: i64, capacity: i64, keys_ptr: i64, values_ptr: i64, states_ptr: i64]
// Header: 40 bytes (5 * 8)
// States: 0=empty, 1=occupied, 2=deleted (tombstone)

const HASHMAP_HEADER_SIZE: usize = 40;
const HASHMAP_STATE_EMPTY: i64 = 0;
const HASHMAP_STATE_OCCUPIED: i64 = 1;
const HASHMAP_STATE_DELETED: i64 = 2;
const HASHMAP_DEFAULT_CAPACITY: i64 = 16;

/// Helper: Hash and find slot for key
fn hashmap_find_slot(keys_ptr: *const i64, states_ptr: *const i64, capacity: i64, key: i64) -> (i64, bool) {
    let hash = {
        let h = (key as u64).wrapping_mul(0x517cc1b727220a95);
        (h ^ (h >> 32)) as i64
    };
    let mask = capacity - 1;
    let mut idx = hash & mask;
    let mut first_deleted: Option<i64> = None;

    unsafe {
        for _ in 0..capacity {
            let state = *states_ptr.add(idx as usize);
            if state == HASHMAP_STATE_EMPTY {
                // Empty slot - key not found
                let insert_idx = first_deleted.unwrap_or(idx);
                return (insert_idx, false);
            } else if state == HASHMAP_STATE_DELETED {
                // Remember first deleted slot for insertion
                if first_deleted.is_none() {
                    first_deleted = Some(idx);
                }
            } else if *keys_ptr.add(idx as usize) == key {
                // Found the key
                return (idx, true);
            }
            // Linear probing
            idx = (idx + 1) & mask;
        }
    }
    // Table is full (shouldn't happen with proper load factor)
    (first_deleted.unwrap_or(0), false)
}

/// hashmap_new() -> i64: Create empty hashmap with default capacity
fn builtin_hashmap_new(args: &[Value]) -> InterpResult<Value> {
    if !args.is_empty() {
        return Err(RuntimeError::arity_mismatch("hashmap_new", 0, args.len()));
    }

    unsafe {
        // Allocate header
        let header_layout = std::alloc::Layout::from_size_align(HASHMAP_HEADER_SIZE, 8)
            .map_err(|_| RuntimeError::io_error("hashmap_new: invalid header layout"))?;
        let header = std::alloc::alloc_zeroed(header_layout) as *mut i64;
        if header.is_null() {
            return Err(RuntimeError::io_error("hashmap_new: out of memory"));
        }

        // Allocate keys array
        let keys_layout = std::alloc::Layout::from_size_align((HASHMAP_DEFAULT_CAPACITY as usize) * 8, 8)
            .map_err(|_| RuntimeError::io_error("hashmap_new: invalid keys layout"))?;
        let keys = std::alloc::alloc_zeroed(keys_layout) as *mut i64;
        if keys.is_null() {
            std::alloc::dealloc(header as *mut u8, header_layout);
            return Err(RuntimeError::io_error("hashmap_new: out of memory"));
        }

        // Allocate values array
        let values_layout = std::alloc::Layout::from_size_align((HASHMAP_DEFAULT_CAPACITY as usize) * 8, 8)
            .map_err(|_| RuntimeError::io_error("hashmap_new: invalid values layout"))?;
        let values = std::alloc::alloc_zeroed(values_layout) as *mut i64;
        if values.is_null() {
            std::alloc::dealloc(keys as *mut u8, keys_layout);
            std::alloc::dealloc(header as *mut u8, header_layout);
            return Err(RuntimeError::io_error("hashmap_new: out of memory"));
        }

        // Allocate states array (all zeros = empty)
        let states_layout = std::alloc::Layout::from_size_align((HASHMAP_DEFAULT_CAPACITY as usize) * 8, 8)
            .map_err(|_| RuntimeError::io_error("hashmap_new: invalid states layout"))?;
        let states = std::alloc::alloc_zeroed(states_layout) as *mut i64;
        if states.is_null() {
            std::alloc::dealloc(values as *mut u8, values_layout);
            std::alloc::dealloc(keys as *mut u8, keys_layout);
            std::alloc::dealloc(header as *mut u8, header_layout);
            return Err(RuntimeError::io_error("hashmap_new: out of memory"));
        }

        // Initialize header: [count, capacity, keys_ptr, values_ptr, states_ptr]
        *header = 0; // count
        *header.add(1) = HASHMAP_DEFAULT_CAPACITY; // capacity
        *header.add(2) = keys as i64;
        *header.add(3) = values as i64;
        *header.add(4) = states as i64;

        Ok(Value::Int(header as i64))
    }
}

/// hashmap_insert(map: i64, key: i64, value: i64) -> i64
/// Returns previous value if key existed, or 0 if new
fn builtin_hashmap_insert(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 3 {
        return Err(RuntimeError::arity_mismatch("hashmap_insert", 3, args.len()));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::Int(map_ptr), Value::Int(key), Value::Int(value)) => {
            if *map_ptr == 0 {
                return Err(RuntimeError::io_error("hashmap_insert: null map"));
            }
            unsafe {
                let header = *map_ptr as *mut i64;
                let count = *header;
                let capacity = *header.add(1);
                let keys_ptr = *header.add(2) as *mut i64;
                let values_ptr = *header.add(3) as *mut i64;
                let states_ptr = *header.add(4) as *mut i64;

                // Check load factor (> 70% triggers resize)
                if count * 10 > capacity * 7 {
                    // Resize: double capacity
                    let new_capacity = capacity * 2;

                    // Allocate new arrays
                    let new_keys_layout = std::alloc::Layout::from_size_align((new_capacity as usize) * 8, 8)
                        .map_err(|_| RuntimeError::io_error("hashmap_insert: resize failed"))?;
                    let new_keys = std::alloc::alloc_zeroed(new_keys_layout) as *mut i64;

                    let new_values_layout = std::alloc::Layout::from_size_align((new_capacity as usize) * 8, 8)
                        .map_err(|_| RuntimeError::io_error("hashmap_insert: resize failed"))?;
                    let new_values = std::alloc::alloc_zeroed(new_values_layout) as *mut i64;

                    let new_states_layout = std::alloc::Layout::from_size_align((new_capacity as usize) * 8, 8)
                        .map_err(|_| RuntimeError::io_error("hashmap_insert: resize failed"))?;
                    let new_states = std::alloc::alloc_zeroed(new_states_layout) as *mut i64;

                    if new_keys.is_null() || new_values.is_null() || new_states.is_null() {
                        return Err(RuntimeError::io_error("hashmap_insert: out of memory"));
                    }

                    // Rehash existing entries
                    for i in 0..capacity {
                        if *states_ptr.add(i as usize) == HASHMAP_STATE_OCCUPIED {
                            let k = *keys_ptr.add(i as usize);
                            let v = *values_ptr.add(i as usize);
                            let (idx, _) = hashmap_find_slot(new_keys, new_states, new_capacity, k);
                            *new_keys.add(idx as usize) = k;
                            *new_values.add(idx as usize) = v;
                            *new_states.add(idx as usize) = HASHMAP_STATE_OCCUPIED;
                        }
                    }

                    // Free old arrays
                    let old_keys_layout = std::alloc::Layout::from_size_align((capacity as usize) * 8, 8).unwrap();
                    let old_values_layout = std::alloc::Layout::from_size_align((capacity as usize) * 8, 8).unwrap();
                    let old_states_layout = std::alloc::Layout::from_size_align((capacity as usize) * 8, 8).unwrap();
                    std::alloc::dealloc(keys_ptr as *mut u8, old_keys_layout);
                    std::alloc::dealloc(values_ptr as *mut u8, old_values_layout);
                    std::alloc::dealloc(states_ptr as *mut u8, old_states_layout);

                    // Update header
                    *header.add(1) = new_capacity;
                    *header.add(2) = new_keys as i64;
                    *header.add(3) = new_values as i64;
                    *header.add(4) = new_states as i64;
                }

                // Re-read pointers after potential resize
                let capacity = *header.add(1);
                let keys_ptr = *header.add(2) as *mut i64;
                let values_ptr = *header.add(3) as *mut i64;
                let states_ptr = *header.add(4) as *mut i64;

                let (idx, found) = hashmap_find_slot(keys_ptr, states_ptr, capacity, *key);
                let old_value = if found {
                    *values_ptr.add(idx as usize)
                } else {
                    *header += 1; // increment count
                    0
                };

                *keys_ptr.add(idx as usize) = *key;
                *values_ptr.add(idx as usize) = *value;
                *states_ptr.add(idx as usize) = HASHMAP_STATE_OCCUPIED;

                Ok(Value::Int(old_value))
            }
        }
        _ => Err(RuntimeError::type_error("(i64, i64, i64)", "other")),
    }
}

/// hashmap_get(map: i64, key: i64) -> i64
/// Returns value if found, or -9223372036854775808 (i64::MIN) if not found
fn builtin_hashmap_get(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("hashmap_get", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(map_ptr), Value::Int(key)) => {
            if *map_ptr == 0 {
                return Err(RuntimeError::io_error("hashmap_get: null map"));
            }
            unsafe {
                let header = *map_ptr as *const i64;
                let capacity = *header.add(1);
                let keys_ptr = *header.add(2) as *const i64;
                let values_ptr = *header.add(3) as *const i64;
                let states_ptr = *header.add(4) as *const i64;

                let (idx, found) = hashmap_find_slot(keys_ptr, states_ptr, capacity, *key);
                if found {
                    Ok(Value::Int(*values_ptr.add(idx as usize)))
                } else {
                    Ok(Value::Int(i64::MIN)) // sentinel for not found
                }
            }
        }
        _ => Err(RuntimeError::type_error("(i64, i64)", "other")),
    }
}

/// hashmap_contains(map: i64, key: i64) -> i64
/// Returns 1 if key exists, 0 otherwise
fn builtin_hashmap_contains(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("hashmap_contains", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(map_ptr), Value::Int(key)) => {
            if *map_ptr == 0 {
                return Err(RuntimeError::io_error("hashmap_contains: null map"));
            }
            unsafe {
                let header = *map_ptr as *const i64;
                let capacity = *header.add(1);
                let keys_ptr = *header.add(2) as *const i64;
                let states_ptr = *header.add(4) as *const i64;

                let (_, found) = hashmap_find_slot(keys_ptr, states_ptr, capacity, *key);
                Ok(Value::Int(if found { 1 } else { 0 }))
            }
        }
        _ => Err(RuntimeError::type_error("(i64, i64)", "other")),
    }
}

/// hashmap_remove(map: i64, key: i64) -> i64
/// Returns removed value if found, or i64::MIN if not found
fn builtin_hashmap_remove(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("hashmap_remove", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(map_ptr), Value::Int(key)) => {
            if *map_ptr == 0 {
                return Err(RuntimeError::io_error("hashmap_remove: null map"));
            }
            unsafe {
                let header = *map_ptr as *mut i64;
                let capacity = *header.add(1);
                let keys_ptr = *header.add(2) as *mut i64;
                let values_ptr = *header.add(3) as *mut i64;
                let states_ptr = *header.add(4) as *mut i64;

                let (idx, found) = hashmap_find_slot(keys_ptr, states_ptr, capacity, *key);
                if found {
                    let old_value = *values_ptr.add(idx as usize);
                    *states_ptr.add(idx as usize) = HASHMAP_STATE_DELETED;
                    *header -= 1; // decrement count
                    Ok(Value::Int(old_value))
                } else {
                    Ok(Value::Int(i64::MIN)) // not found
                }
            }
        }
        _ => Err(RuntimeError::type_error("(i64, i64)", "other")),
    }
}

/// hashmap_len(map: i64) -> i64
/// Returns number of entries in the map
fn builtin_hashmap_len(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("hashmap_len", 1, args.len()));
    }
    match &args[0] {
        Value::Int(map_ptr) => {
            if *map_ptr == 0 {
                return Ok(Value::Int(0));
            }
            unsafe {
                let header = *map_ptr as *const i64;
                Ok(Value::Int(*header))
            }
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// hashmap_free(map: i64) -> Unit
/// Deallocate hashmap and all its arrays
fn builtin_hashmap_free(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("hashmap_free", 1, args.len()));
    }
    match &args[0] {
        Value::Int(map_ptr) => {
            if *map_ptr == 0 {
                return Ok(Value::Unit);
            }
            unsafe {
                let header = *map_ptr as *mut i64;
                let capacity = *header.add(1);
                let keys_ptr = *header.add(2) as *mut u8;
                let values_ptr = *header.add(3) as *mut u8;
                let states_ptr = *header.add(4) as *mut u8;

                // Free arrays
                let arr_layout = std::alloc::Layout::from_size_align((capacity as usize) * 8, 8)
                    .map_err(|_| RuntimeError::io_error("hashmap_free: invalid layout"))?;
                if !keys_ptr.is_null() {
                    std::alloc::dealloc(keys_ptr, arr_layout);
                }
                if !values_ptr.is_null() {
                    std::alloc::dealloc(values_ptr, arr_layout);
                }
                if !states_ptr.is_null() {
                    std::alloc::dealloc(states_ptr, arr_layout);
                }

                // Free header
                let header_layout = std::alloc::Layout::from_size_align(HASHMAP_HEADER_SIZE, 8)
                    .map_err(|_| RuntimeError::io_error("hashmap_free: invalid header layout"))?;
                std::alloc::dealloc(*map_ptr as *mut u8, header_layout);
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

// ============ v0.34.24: HashSet Builtins ============
// HashSet is a thin wrapper around HashMap with value always = 1

/// hashset_new() -> i64: Create empty hashset
fn builtin_hashset_new(args: &[Value]) -> InterpResult<Value> {
    builtin_hashmap_new(args)
}

/// hashset_insert(set: i64, value: i64) -> i64
/// Returns 1 if newly inserted, 0 if already existed
fn builtin_hashset_insert(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("hashset_insert", 2, args.len()));
    }
    // Insert with value=1
    let insert_args = vec![args[0].clone(), args[1].clone(), Value::Int(1)];
    let result = builtin_hashmap_insert(&insert_args)?;
    // Return 1 if new (old_value was 0), 0 if existed (old_value was 1)
    match result {
        Value::Int(old) => Ok(Value::Int(if old == 0 { 1 } else { 0 })),
        _ => Ok(result),
    }
}

/// hashset_contains(set: i64, value: i64) -> i64
/// Returns 1 if value exists, 0 otherwise
fn builtin_hashset_contains(args: &[Value]) -> InterpResult<Value> {
    builtin_hashmap_contains(args)
}

/// hashset_remove(set: i64, value: i64) -> i64
/// Returns 1 if removed, 0 if not found
fn builtin_hashset_remove(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("hashset_remove", 2, args.len()));
    }
    let result = builtin_hashmap_remove(args)?;
    match result {
        Value::Int(v) => Ok(Value::Int(if v == i64::MIN { 0 } else { 1 })),
        _ => Ok(result),
    }
}

/// hashset_len(set: i64) -> i64
fn builtin_hashset_len(args: &[Value]) -> InterpResult<Value> {
    builtin_hashmap_len(args)
}

/// hashset_free(set: i64) -> Unit
fn builtin_hashset_free(args: &[Value]) -> InterpResult<Value> {
    builtin_hashmap_free(args)
}

// ============ v0.31.10: File I/O Builtins for Phase 32.0 Bootstrap Infrastructure ============

/// Helper: Extract string from Value (handles both Str and StringRope)
fn extract_string(val: &Value) -> Option<String> {
    val.materialize_string()
}

/// read_file(path: String) -> String
/// Reads entire file contents as a string. Returns error on failure.
fn builtin_read_file(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("read_file", 1, args.len()));
    }
    match extract_string(&args[0]) {
        Some(path) => {
            match fs::read_to_string(&path) {
                Ok(content) => Ok(Value::Str(Rc::new(content))),
                Err(e) => Err(RuntimeError::io_error(&format!("read_file '{}': {}", path, e))),
            }
        }
        None => Err(RuntimeError::type_error("string", args[0].type_name())),
    }
}

/// write_file(path: String, content: String) -> i64
/// Writes content to file. Returns 0 on success, -1 on error.
fn builtin_write_file(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("write_file", 2, args.len()));
    }
    match (extract_string(&args[0]), extract_string(&args[1])) {
        (Some(path), Some(content)) => {
            match fs::write(&path, &content) {
                Ok(()) => Ok(Value::Int(0)),
                Err(e) => {
                    eprintln!("write_file error: {}", e);
                    Ok(Value::Int(-1))
                }
            }
        }
        _ => Err(RuntimeError::type_error("(string, string)", "other")),
    }
}

/// append_file(path: String, content: String) -> i64
/// Appends content to file. Returns 0 on success, -1 on error.
fn builtin_append_file(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("append_file", 2, args.len()));
    }
    match (extract_string(&args[0]), extract_string(&args[1])) {
        (Some(path), Some(content)) => {
            use std::fs::OpenOptions;
            match OpenOptions::new().create(true).append(true).open(&path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(()) => Ok(Value::Int(0)),
                        Err(e) => {
                            eprintln!("append_file write error: {}", e);
                            Ok(Value::Int(-1))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("append_file open error: {}", e);
                    Ok(Value::Int(-1))
                }
            }
        }
        _ => Err(RuntimeError::type_error("(string, string)", "other")),
    }
}

/// file_exists(path: String) -> i64
/// Returns 1 if file exists, 0 otherwise.
fn builtin_file_exists(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("file_exists", 1, args.len()));
    }
    match extract_string(&args[0]) {
        Some(path) => {
            let exists = Path::new(&path).exists();
            Ok(Value::Int(if exists { 1 } else { 0 }))
        }
        None => Err(RuntimeError::type_error("string", args[0].type_name())),
    }
}

/// file_size(path: String) -> i64
/// Returns file size in bytes, or -1 on error.
fn builtin_file_size(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("file_size", 1, args.len()));
    }
    match extract_string(&args[0]) {
        Some(path) => {
            match fs::metadata(&path) {
                Ok(meta) => Ok(Value::Int(meta.len() as i64)),
                Err(_) => Ok(Value::Int(-1)),
            }
        }
        None => Err(RuntimeError::type_error("string", args[0].type_name())),
    }
}

// ============ v0.31.11: Process Execution Builtins for Phase 32.0.2 Bootstrap Infrastructure ============

/// Helper: Parse command arguments string into Vec<String>
/// Simple split on whitespace, handles quoted strings
fn parse_args(args_str: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for c in args_str.chars() {
        if in_quote {
            if c == quote_char {
                in_quote = false;
            } else {
                current.push(c);
            }
        } else if c == '"' || c == '\'' {
            in_quote = true;
            quote_char = c;
        } else if c.is_whitespace() {
            if !current.is_empty() {
                result.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// exec(command: String, args: String) -> i64
/// Execute a command with arguments, returns exit code (0 = success, -1 = error).
fn builtin_exec(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("exec", 2, args.len()));
    }
    match (extract_string(&args[0]), extract_string(&args[1])) {
        (Some(command), Some(args_str)) => {
            let parsed_args = parse_args(&args_str);
            match Command::new(&command).args(&parsed_args).status() {
                Ok(status) => {
                    Ok(Value::Int(status.code().unwrap_or(-1) as i64))
                }
                Err(e) => {
                    eprintln!("exec error: {}", e);
                    Ok(Value::Int(-1))
                }
            }
        }
        _ => Err(RuntimeError::type_error("(string, string)", "other")),
    }
}

/// exec_output(command: String, args: String) -> String
/// Execute a command and capture stdout.
fn builtin_exec_output(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("exec_output", 2, args.len()));
    }
    match (extract_string(&args[0]), extract_string(&args[1])) {
        (Some(command), Some(args_str)) => {
            let parsed_args = parse_args(&args_str);
            match Command::new(&command).args(&parsed_args).output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    Ok(Value::Str(Rc::new(stdout)))
                }
                Err(e) => {
                    eprintln!("exec_output error: {}", e);
                    Ok(Value::Str(Rc::new(String::new())))
                }
            }
        }
        _ => Err(RuntimeError::type_error("(string, string)", "other")),
    }
}

/// system(command: String) -> i64
/// Execute a shell command, returns exit code.
fn builtin_system(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("system", 1, args.len()));
    }
    match extract_string(&args[0]) {
        Some(command) => {
            // Use platform-specific shell
            #[cfg(windows)]
            let result = Command::new("cmd").args(["/C", &command]).status();
            #[cfg(not(windows))]
            let result = Command::new("sh").args(["-c", &command]).status();

            match result {
                Ok(status) => Ok(Value::Int(status.code().unwrap_or(-1) as i64)),
                Err(e) => {
                    eprintln!("system error: {}", e);
                    Ok(Value::Int(-1))
                }
            }
        }
        None => Err(RuntimeError::type_error("string", args[0].type_name())),
    }
}

/// getenv(name: String) -> String
/// Get environment variable value, or empty string if not set.
fn builtin_getenv(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("getenv", 1, args.len()));
    }
    match extract_string(&args[0]) {
        Some(name) => {
            let value = env::var(&name).unwrap_or_default();
            Ok(Value::Str(Rc::new(value)))
        }
        None => Err(RuntimeError::type_error("string", args[0].type_name())),
    }
}

// v0.63: Timing builtin for bmb-bench benchmark framework
/// time_ns() -> i64
/// Returns nanoseconds since UNIX epoch (for benchmarking)
fn builtin_time_ns(args: &[Value]) -> InterpResult<Value> {
    if !args.is_empty() {
        return Err(RuntimeError::arity_mismatch("time_ns", 0, args.len()));
    }
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    Ok(Value::Int(now.as_nanos() as i64))
}

// ============ v0.31.22: Command-line Argument Builtins for Phase 32.3.D ============
// Provides CLI argument access for standalone BMB compiler
// v0.46: Updated to use thread-local storage for program arguments

/// arg_count() -> i64
/// Returns the number of command-line arguments.
/// v0.46: Uses thread-local PROGRAM_ARGS instead of env::args()
fn builtin_arg_count(_args: &[Value]) -> InterpResult<Value> {
    let count = get_program_arg_count() as i64;
    Ok(Value::Int(count))
}

/// get_arg(n: i64) -> String
/// Returns the nth command-line argument (0 = program name).
/// Returns empty string if index is out of bounds.
/// v0.46: Uses thread-local PROGRAM_ARGS instead of env::args()
fn builtin_get_arg(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("get_arg", 1, args.len()));
    }
    match &args[0] {
        Value::Int(n) => {
            let idx = *n as usize;
            let arg = get_program_arg(idx);
            Ok(Value::Str(Rc::new(arg)))
        }
        _ => Err(RuntimeError::type_error("integer", args[0].type_name())),
    }
}

// ============ v0.31.13: StringBuilder Builtins for Phase 32.0.4 ============
// Provides O(1) amortized string append operations to fix O(n²) concatenation
// in Bootstrap compiler's MIR generation.

use std::cell::RefCell as SbRefCell;

thread_local! {
    /// Thread-local string builder storage. Each builder is identified by an i64 ID.
    static STRING_BUILDERS: SbRefCell<HashMap<i64, Vec<String>>> = SbRefCell::new(HashMap::new());
    /// Counter for generating unique builder IDs
    static SB_COUNTER: SbRefCell<i64> = const { SbRefCell::new(0) };
}

/// sb_new() -> i64
/// Creates a new string builder, returns its ID.
fn builtin_sb_new(args: &[Value]) -> InterpResult<Value> {
    if !args.is_empty() {
        return Err(RuntimeError::arity_mismatch("sb_new", 0, args.len()));
    }
    let id = SB_COUNTER.with(|counter| {
        let mut c = counter.borrow_mut();
        let id = *c;
        *c += 1;
        id
    });

    STRING_BUILDERS.with(|builders| {
        builders.borrow_mut().insert(id, Vec::new());
    });

    Ok(Value::Int(id))
}

/// sb_with_capacity(capacity: i64) -> i64
/// Creates a new string builder with pre-allocated capacity.
/// v0.51.45: P0-E optimization to avoid reallocations.
fn builtin_sb_with_capacity(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("sb_with_capacity", 1, args.len()));
    }
    let capacity = match &args[0] {
        Value::Int(n) => *n as usize,
        _ => 64,  // Default capacity if not an integer
    };

    let id = SB_COUNTER.with(|counter| {
        let mut c = counter.borrow_mut();
        let id = *c;
        *c += 1;
        id
    });

    STRING_BUILDERS.with(|builders| {
        // Pre-allocate with capacity hint (Vec::with_capacity approximation)
        // Note: The interpreter uses Vec<String> so capacity hint is ignored,
        // but the native runtime (bmb_runtime.c) does use real capacity.
        let v = Vec::with_capacity(capacity);
        builders.borrow_mut().insert(id, v);
    });

    Ok(Value::Int(id))
}

/// sb_push(id: i64, str: String) -> i64
/// Appends a string to the builder. Returns the same ID for chaining.
fn builtin_sb_push(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("sb_push", 2, args.len()));
    }
    match (&args[0], extract_string(&args[1])) {
        (Value::Int(id), Some(s)) => {
            STRING_BUILDERS.with(|builders| {
                let mut map = builders.borrow_mut();
                if let Some(builder) = map.get_mut(id) {
                    builder.push(s);
                    Ok(Value::Int(*id))
                } else {
                    Err(RuntimeError::io_error(&format!("Invalid string builder ID: {}", id)))
                }
            })
        }
        _ => Err(RuntimeError::type_error("(i64, string)", "other")),
    }
}

/// sb_push_char(id: i64, char_code: i64) -> i64
/// Appends a single character (by code point) to the builder. Returns the same ID for chaining.
fn builtin_sb_push_char(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("sb_push_char", 2, args.len()));
    }
    match (&args[0], &args[1]) {
        (Value::Int(id), Value::Int(char_code)) => {
            STRING_BUILDERS.with(|builders| {
                let mut map = builders.borrow_mut();
                if let Some(builder) = map.get_mut(id) {
                    // Convert char code to single-char string
                    if let Some(c) = char::from_u32(*char_code as u32) {
                        builder.push(c.to_string());
                        Ok(Value::Int(*id))
                    } else {
                        Err(RuntimeError::io_error(&format!("Invalid char code: {}", char_code)))
                    }
                } else {
                    Err(RuntimeError::io_error(&format!("Invalid string builder ID: {}", id)))
                }
            })
        }
        _ => Err(RuntimeError::type_error("(i64, i64)", "other")),
    }
}

/// sb_build(id: i64) -> String
/// Materializes the builder into a single string and removes the builder.
fn builtin_sb_build(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("sb_build", 1, args.len()));
    }
    match &args[0] {
        Value::Int(id) => {
            STRING_BUILDERS.with(|builders| {
                let mut map = builders.borrow_mut();
                if let Some(fragments) = map.remove(id) {
                    let total_len: usize = fragments.iter().map(|s| s.len()).sum();
                    let mut result = String::with_capacity(total_len);
                    for frag in fragments {
                        result.push_str(&frag);
                    }
                    Ok(Value::Str(Rc::new(result)))
                } else {
                    Err(RuntimeError::io_error(&format!("Invalid string builder ID: {}", id)))
                }
            })
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// sb_len(id: i64) -> i64
/// Returns the total length of all strings in the builder.
fn builtin_sb_len(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("sb_len", 1, args.len()));
    }
    match &args[0] {
        Value::Int(id) => {
            STRING_BUILDERS.with(|builders| {
                let map = builders.borrow();
                if let Some(fragments) = map.get(id) {
                    let total_len: i64 = fragments.iter().map(|s| s.len() as i64).sum();
                    Ok(Value::Int(total_len))
                } else {
                    Err(RuntimeError::io_error(&format!("Invalid string builder ID: {}", id)))
                }
            })
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// sb_clear(id: i64) -> i64
/// Clears the builder contents without removing it. Returns same ID.
fn builtin_sb_clear(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("sb_clear", 1, args.len()));
    }
    match &args[0] {
        Value::Int(id) => {
            STRING_BUILDERS.with(|builders| {
                let mut map = builders.borrow_mut();
                if let Some(builder) = map.get_mut(id) {
                    builder.clear();
                    Ok(Value::Int(*id))
                } else {
                    Err(RuntimeError::io_error(&format!("Invalid string builder ID: {}", id)))
                }
            })
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// sb_println(id: i64) -> i64
/// Prints the builder contents directly without allocating a new string.
/// v0.60.64: Added for zero-allocation output
fn builtin_sb_println(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("sb_println", 1, args.len()));
    }
    match &args[0] {
        Value::Int(id) => {
            STRING_BUILDERS.with(|builders| {
                let map = builders.borrow();
                if let Some(fragments) = map.get(id) {
                    for frag in fragments {
                        print!("{}", frag);
                    }
                    println!();
                    Ok(Value::Int(0))
                } else {
                    Err(RuntimeError::io_error(&format!("Invalid string builder ID: {}", id)))
                }
            })
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// puts_cstr(ptr: i64) -> i64
/// Prints a null-terminated C string from a raw memory address.
/// v0.60.65: Added for low-level I/O
fn builtin_puts_cstr(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("puts_cstr", 1, args.len()));
    }
    // In interpreter mode, we can't really handle raw pointers
    // Just print a placeholder for now
    match &args[0] {
        Value::Int(_ptr) => {
            println!("[puts_cstr: interpreter cannot access raw memory]");
            Ok(Value::Int(0))
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// chr(code: i64) -> char
/// Converts a Unicode codepoint to a character.
/// v0.31.21: Added for gotgan string handling
/// v0.65: Updated to return char type with full Unicode support
/// v0.50.18: Changed to return String to match C runtime behavior (bmb_chr returns char*)
fn builtin_chr(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("chr", 1, args.len()));
    }
    match &args[0] {
        Value::Int(code) => {
            if *code < 0 {
                Err(RuntimeError::io_error(&format!("chr: negative code {}", code)))
            } else if let Some(c) = char::from_u32(*code as u32) {
                // Return single-char String to match C runtime behavior
                Ok(Value::Str(std::rc::Rc::new(c.to_string())))
            } else {
                Err(RuntimeError::io_error(&format!("chr: invalid Unicode codepoint {}", code)))
            }
        }
        _ => Err(RuntimeError::type_error("i64", args[0].type_name())),
    }
}

/// ord(c: char) -> i64
/// Returns the Unicode codepoint of a character.
/// v0.31.21: Added for gotgan string handling
/// v0.65: Updated to accept char type with full Unicode support
fn builtin_ord(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("ord", 1, args.len()));
    }
    match &args[0] {
        Value::Char(c) => Ok(Value::Int(*c as u32 as i64)),
        _ => Err(RuntimeError::type_error("char", args[0].type_name())),
    }
}

/// char_at(s: String, idx: i64) -> char
/// Returns the character at the given index (Unicode-aware).
/// v0.66: Added for string-char interop
/// v0.92: Fixed to handle StringRope (lazy concatenated strings)
fn builtin_char_at(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch("char_at", 2, args.len()));
    }
    // v0.92: Use materialize_string to handle both Str and StringRope
    let s = args[0]
        .materialize_string()
        .ok_or_else(|| RuntimeError::type_error("String", args[0].type_name()))?;
    match &args[1] {
        Value::Int(idx) => {
            let idx = *idx;
            if idx < 0 {
                return Err(RuntimeError::io_error(&format!(
                    "char_at: negative index {}",
                    idx
                )));
            }
            let idx = idx as usize;
            match s.chars().nth(idx) {
                Some(c) => Ok(Value::Char(c)),
                None => Err(RuntimeError::io_error(&format!(
                    "char_at: index {} out of bounds (string has {} characters)",
                    idx,
                    s.chars().count()
                ))),
            }
        }
        other => Err(RuntimeError::type_error("i64", other.type_name())),
    }
}

/// char_to_string(c: char) -> String
/// Converts a character to a single-character string.
/// v0.66: Added for string-char interop
fn builtin_char_to_string(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("char_to_string", 1, args.len()));
    }
    match &args[0] {
        Value::Char(c) => Ok(Value::Str(Rc::new(c.to_string()))),
        _ => Err(RuntimeError::type_error("char", args[0].type_name())),
    }
}

/// str_len(s: String) -> i64
/// Returns the Unicode character count of a string.
/// Note: This is O(n) for UTF-8. Use s.len() for O(1) byte length.
/// v0.67: Added for string utilities
fn builtin_str_len(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("str_len", 1, args.len()));
    }
    match &args[0] {
        Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
        Value::StringRope(fragments) => {
            let count: usize = fragments.borrow().iter().map(|s| s.chars().count()).sum();
            Ok(Value::Int(count as i64))
        }
        _ => Err(RuntimeError::type_error("String", args[0].type_name())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned {
            node,
            span: Span { start: 0, end: 0 },
        }
    }

    #[test]
    fn test_eval_literals() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        assert_eq!(
            interp.eval(&spanned(Expr::IntLit(42)), &env).unwrap(),
            Value::Int(42)
        );
        assert_eq!(
            interp.eval(&spanned(Expr::BoolLit(true)), &env).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_eval_binary() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        let add_expr = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(2))),
            op: BinOp::Add,
            right: Box::new(spanned(Expr::IntLit(3))),
        };
        assert_eq!(
            interp.eval(&spanned(add_expr), &env).unwrap(),
            Value::Int(5)
        );
    }

    #[test]
    fn test_eval_if() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        let if_expr = Expr::If {
            cond: Box::new(spanned(Expr::BoolLit(true))),
            then_branch: Box::new(spanned(Expr::IntLit(1))),
            else_branch: Box::new(spanned(Expr::IntLit(2))),
        };
        assert_eq!(
            interp.eval(&spanned(if_expr), &env).unwrap(),
            Value::Int(1)
        );
    }

    #[test]
    fn test_eval_let() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        let let_expr = Expr::Let {
            name: "x".to_string(),
            mutable: false,
            ty: None,
            value: Box::new(spanned(Expr::IntLit(10))),
            body: Box::new(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Var("x".to_string()))),
                op: BinOp::Mul,
                right: Box::new(spanned(Expr::IntLit(2))),
            })),
        };
        assert_eq!(
            interp.eval(&spanned(let_expr), &env).unwrap(),
            Value::Int(20)
        );
    }

    #[test]
    fn test_division_by_zero() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        let div_expr = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(10))),
            op: BinOp::Div,
            right: Box::new(spanned(Expr::IntLit(0))),
        };
        let result = interp.eval(&spanned(div_expr), &env);
        assert!(result.is_err());
    }


    #[test]
    fn test_eval_string() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        assert_eq!(
            interp.eval(&spanned(Expr::StringLit("hello".to_string())), &env).unwrap(),
            Value::Str(Rc::new("hello".to_string()))
        );
    }

    #[test]
    fn test_string_concat() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        let concat_expr = Expr::Binary {
            left: Box::new(spanned(Expr::StringLit("hello".to_string()))),
            op: BinOp::Add,
            right: Box::new(spanned(Expr::StringLit(" world".to_string()))),
        };
        assert_eq!(
            interp.eval(&spanned(concat_expr), &env).unwrap(),
            Value::Str(Rc::new("hello world".to_string()))
        );
    }

    #[test]
    fn test_short_circuit_and() {
        // Test: false and <error> should return false without evaluating right side
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        // false and (1/0) - if short-circuit works, no division by zero error
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::BoolLit(false))),
            op: BinOp::And,
            right: Box::new(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::IntLit(1))),
                op: BinOp::Div,
                right: Box::new(spanned(Expr::IntLit(0))),
            })),
        };
        // Should succeed with false (short-circuit prevents division by zero)
        assert_eq!(
            interp.eval(&spanned(expr), &env).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_short_circuit_or() {
        // Test: true or <error> should return true without evaluating right side
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        // true or (1/0) - if short-circuit works, no division by zero error
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::BoolLit(true))),
            op: BinOp::Or,
            right: Box::new(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::IntLit(1))),
                op: BinOp::Div,
                right: Box::new(spanned(Expr::IntLit(0))),
            })),
        };
        // Should succeed with true (short-circuit prevents division by zero)
        assert_eq!(
            interp.eval(&spanned(expr), &env).unwrap(),
            Value::Bool(true)
        );
    }

    // ================================================================
    // Unary Operations Tests
    // ================================================================

    #[test]
    fn test_unary_neg() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Unary {
            op: UnOp::Neg,
            expr: Box::new(spanned(Expr::IntLit(42))),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Int(-42));
    }

    #[test]
    fn test_unary_not() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Unary {
            op: UnOp::Not,
            expr: Box::new(spanned(Expr::BoolLit(true))),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Bool(false));
    }

    // ================================================================
    // Comparison & Arithmetic Tests
    // ================================================================

    #[test]
    fn test_comparison_ops() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();

        let lt = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(3))),
            op: BinOp::Lt,
            right: Box::new(spanned(Expr::IntLit(5))),
        };
        assert_eq!(interp.eval(&spanned(lt), &env).unwrap(), Value::Bool(true));

        let ge = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(5))),
            op: BinOp::Ge,
            right: Box::new(spanned(Expr::IntLit(5))),
        };
        assert_eq!(interp.eval(&spanned(ge), &env).unwrap(), Value::Bool(true));

        let eq = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(5))),
            op: BinOp::Eq,
            right: Box::new(spanned(Expr::IntLit(5))),
        };
        assert_eq!(interp.eval(&spanned(eq), &env).unwrap(), Value::Bool(true));

        let ne = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(5))),
            op: BinOp::Ne,
            right: Box::new(spanned(Expr::IntLit(3))),
        };
        assert_eq!(interp.eval(&spanned(ne), &env).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_modulo() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(10))),
            op: BinOp::Mod,
            right: Box::new(spanned(Expr::IntLit(3))),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Int(1));
    }

    #[test]
    fn test_multiply() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(7))),
            op: BinOp::Mul,
            right: Box::new(spanned(Expr::IntLit(6))),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_subtract() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(10))),
            op: BinOp::Sub,
            right: Box::new(spanned(Expr::IntLit(3))),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Int(7));
    }

    // ================================================================
    // F64 Tests
    // ================================================================

    #[test]
    fn test_f64_literal() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        assert_eq!(
            interp.eval(&spanned(Expr::FloatLit(1.234)), &env).unwrap(),
            Value::Float(1.234)
        );
    }

    #[test]
    fn test_f64_arithmetic() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::FloatLit(1.5))),
            op: BinOp::Add,
            right: Box::new(spanned(Expr::FloatLit(2.5))),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Float(4.0));
    }

    // ================================================================
    // If-Else Branch Tests
    // ================================================================

    #[test]
    fn test_if_false_branch() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let if_expr = Expr::If {
            cond: Box::new(spanned(Expr::BoolLit(false))),
            then_branch: Box::new(spanned(Expr::IntLit(1))),
            else_branch: Box::new(spanned(Expr::IntLit(2))),
        };
        assert_eq!(interp.eval(&spanned(if_expr), &env).unwrap(), Value::Int(2));
    }

    // ================================================================
    // Nested Expression Tests
    // ================================================================

    #[test]
    fn test_nested_let() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        // let x = 5; let y = x * 2; y + 1 = 11
        let expr = Expr::Let {
            name: "x".to_string(),
            mutable: false,
            ty: None,
            value: Box::new(spanned(Expr::IntLit(5))),
            body: Box::new(spanned(Expr::Let {
                name: "y".to_string(),
                mutable: false,
                ty: None,
                value: Box::new(spanned(Expr::Binary {
                    left: Box::new(spanned(Expr::Var("x".to_string()))),
                    op: BinOp::Mul,
                    right: Box::new(spanned(Expr::IntLit(2))),
                })),
                body: Box::new(spanned(Expr::Binary {
                    left: Box::new(spanned(Expr::Var("y".to_string()))),
                    op: BinOp::Add,
                    right: Box::new(spanned(Expr::IntLit(1))),
                })),
            })),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Int(11));
    }

    #[test]
    fn test_nested_arithmetic() {
        // (2 + 3) * (4 - 1) = 15
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::IntLit(2))),
                op: BinOp::Add,
                right: Box::new(spanned(Expr::IntLit(3))),
            })),
            op: BinOp::Mul,
            right: Box::new(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::IntLit(4))),
                op: BinOp::Sub,
                right: Box::new(spanned(Expr::IntLit(1))),
            })),
        };
        assert_eq!(interp.eval(&spanned(expr), &env).unwrap(), Value::Int(15));
    }

    // ================================================================
    // Full Program Interpretation Tests
    // ================================================================

    fn run_source(source: &str) -> InterpResult<Value> {
        let tokens = crate::lexer::tokenize(source).expect("tokenize failed");
        let program = crate::parser::parse("test.bmb", source, tokens).expect("parse failed");
        let mut tc = crate::types::TypeChecker::new();
        tc.check_program(&program).expect("type-check failed");
        let mut interp = Interpreter::new();
        interp.run(&program)
    }

    #[test]
    fn test_run_simple_function() {
        assert_eq!(run_source("fn main() -> i64 = 42;").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_run_function_with_params() {
        let result = run_source("fn add(a: i64, b: i64) -> i64 = a + b; fn main() -> i64 = add(3, 4);");
        assert_eq!(result.unwrap(), Value::Int(7));
    }

    #[test]
    fn test_run_recursive_function() {
        let result = run_source(
            "fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) }; fn main() -> i64 = fact(5);"
        );
        assert_eq!(result.unwrap(), Value::Int(120));
    }

    #[test]
    fn test_run_if_else_expression() {
        assert_eq!(
            run_source("fn main() -> i64 = if 3 > 2 { 10 } else { 20 };").unwrap(),
            Value::Int(10)
        );
    }

    #[test]
    fn test_run_string_operations() {
        let result = run_source(r#"fn main() -> String = "hello" + " " + "world";"#);
        assert_eq!(result.unwrap(), Value::Str(Rc::new("hello world".to_string())));
    }

    #[test]
    fn test_run_bool_operations() {
        assert_eq!(
            run_source("fn main() -> bool = true and false or true;").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_run_nested_calls() {
        let result = run_source(
            "fn double(x: i64) -> i64 = x * 2; fn main() -> i64 = double(double(3));"
        );
        assert_eq!(result.unwrap(), Value::Int(12));
    }

    // ================================================================
    // Error Handling Tests
    // ================================================================

    #[test]
    fn test_modulo_by_zero() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let expr = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(10))),
            op: BinOp::Mod,
            right: Box::new(spanned(Expr::IntLit(0))),
        };
        assert!(interp.eval(&spanned(expr), &env).is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        assert!(interp.eval(&spanned(Expr::Var("undefined_var".to_string())), &env).is_err());
    }

    // ====================================================================
    // Source-based interpreter integration tests (v0.89.6, Cycle 55)
    // ====================================================================

    fn run_program(source: &str) -> Value {
        let tokens = crate::lexer::tokenize(source).expect("tokenize failed");
        let program = crate::parser::parse("<test>", source, tokens).expect("parse failed");
        let mut interp = Interpreter::new();
        interp.run(&program).expect("run failed")
    }

    // v0.89.5: Float/int equality coercion (Cycle 42 fix)
    #[test]
    fn test_float_int_equality() {
        // sqrt(4) == 2 should be true (float/int cross-type comparison)
        assert_eq!(run_program("fn main() -> i64 = if 4.0 == 4 { 1 } else { 0 };"), Value::Int(1));
        assert_eq!(run_program("fn main() -> i64 = if 3.0 != 4 { 1 } else { 0 };"), Value::Int(1));
        assert_eq!(run_program("fn main() -> i64 = if 2 == 2.0 { 1 } else { 0 };"), Value::Int(1));
    }

    // v0.89.5: free() returns i64 (Cycle 42 fix)
    #[test]
    fn test_free_returns_i64() {
        let result = run_program("fn main() -> i64 = { let p = malloc(8); let r = free(p); r };");
        assert_eq!(result, Value::Int(0));
    }

    // v0.89.6: Assignments in if-branches (Cycle 52 fix)
    #[test]
    fn test_assign_in_if_branch() {
        let result = run_program(
            "fn main() -> i64 = { let mut x = 0; let _r = if true { x = 42; 0 } else { 0 }; x };"
        );
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_assign_in_else_branch() {
        let result = run_program(
            "fn main() -> i64 = { let mut x = 0; let _r = if false { 0 } else { x = 99; 0 }; x };"
        );
        assert_eq!(result, Value::Int(99));
    }

    // v0.89.6: Let bindings in if-branches (Cycle 52 fix)
    #[test]
    fn test_let_in_if_branch() {
        let result = run_program(
            "fn main() -> i64 = if true { let x = 42; x } else { 0 };"
        );
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_let_in_else_branch() {
        let result = run_program(
            "fn main() -> i64 = if false { 0 } else { let y = 100; y };"
        );
        assert_eq!(result, Value::Int(100));
    }

    // While loop with mutable state
    #[test]
    fn test_while_loop_accumulator() {
        let result = run_program(
            "fn main() -> i64 = { let mut sum = 0; let mut i = 1; while i <= 10 { sum = sum + i; i = i + 1; 0 }; sum };"
        );
        assert_eq!(result, Value::Int(55)); // 1+2+...+10 = 55
    }

    // v0.89.6: Multiple assignments in if-branch
    #[test]
    fn test_multi_assign_in_if_branch() {
        let result = run_program(
            "fn main() -> i64 = { let mut a = 0; let mut b = 0; let _r = if true { a = 10; b = 20; 0 } else { 0 }; a + b };"
        );
        assert_eq!(result, Value::Int(30));
    }

    // If-else if chain with assignments
    #[test]
    fn test_elseif_chain_with_assigns() {
        let result = run_program(
            "fn main() -> i64 = { let mut x = 0; let v = 5; let _r = if v < 3 { x = 1; 0 } else if v < 7 { x = 2; 0 } else { x = 3; 0 }; x };"
        );
        assert_eq!(result, Value::Int(2)); // 5 < 7 so x = 2
    }

    // Recursive function
    #[test]
    fn test_recursive_factorial() {
        let result = run_program(
            "fn fact(n: i64) -> i64 = if n <= 1 { 1 } else { n * fact(n - 1) }; fn main() -> i64 = fact(5);"
        );
        assert_eq!(result, Value::Int(120));
    }

    // Nested function calls
    #[test]
    fn test_nested_calls() {
        let result = run_program(
            "fn double(x: i64) -> i64 = x * 2; fn add1(x: i64) -> i64 = x + 1; fn main() -> i64 = add1(double(5));"
        );
        assert_eq!(result, Value::Int(11));
    }

    // Boolean operations
    #[test]
    fn test_boolean_and_or() {
        assert_eq!(run_program("fn main() -> i64 = if true and true { 1 } else { 0 };"), Value::Int(1));
        assert_eq!(run_program("fn main() -> i64 = if true and false { 1 } else { 0 };"), Value::Int(0));
        assert_eq!(run_program("fn main() -> i64 = if false or true { 1 } else { 0 };"), Value::Int(1));
        assert_eq!(run_program("fn main() -> i64 = if false or false { 1 } else { 0 };"), Value::Int(0));
    }

    // String operations
    #[test]
    fn test_string_len() {
        let result = run_program("fn main() -> i64 = \"hello\".len();");
        assert_eq!(result, Value::Int(5));
    }

    // ================================================================
    // Cycle 82: Extended interpreter tests
    // ================================================================

    #[test]
    fn test_for_loop_sum() {
        let result = run_program(
            "fn main() -> i64 = { let mut sum = 0; for i in 0..5 { sum = sum + i }; sum };"
        );
        assert_eq!(result, Value::Int(10)); // 0+1+2+3+4 = 10
    }

    #[test]
    fn test_struct_creation_and_access() {
        let result = run_program(
            "struct Point { x: i64, y: i64 }
             fn main() -> i64 = { let p = new Point { x: 3, y: 4 }; p.x + p.y };"
        );
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_enum_match() {
        let result = run_program(
            "enum Color { Red, Green, Blue }
             fn to_int(c: Color) -> i64 = match c { Color::Red => 1, Color::Green => 2, Color::Blue => 3 };
             fn main() -> i64 = to_int(Color::Green);"
        );
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_tuple_creation() {
        let result = run_program(
            "fn first(t: (i64, i64)) -> i64 = t.0;
             fn main() -> i64 = first((42, 99));"
        );
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_f64_comparison() {
        assert_eq!(run_program("fn main() -> i64 = if 3.14 > 2.71 { 1 } else { 0 };"), Value::Int(1));
        assert_eq!(run_program("fn main() -> i64 = if 1.0 < 2.0 { 1 } else { 0 };"), Value::Int(1));
        assert_eq!(run_program("fn main() -> i64 = if 1.5 >= 1.5 { 1 } else { 0 };"), Value::Int(1));
        assert_eq!(run_program("fn main() -> i64 = if 1.5 <= 1.5 { 1 } else { 0 };"), Value::Int(1));
    }

    #[test]
    fn test_f64_arithmetic_full() {
        let result = run_program("fn main() -> f64 = 10.0 / 3.0;");
        if let Value::Float(f) = result {
            assert!((f - 3.3333333333333335).abs() < 1e-10);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_mutual_recursion() {
        let result = run_program(
            "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
             fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };
             fn main() -> i64 = if is_even(4) { 1 } else { 0 };"
        );
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_match_wildcard() {
        let result = run_program(
            "fn classify(x: i64) -> i64 = match x { 0 => 100, 1 => 200, _ => 0 };
             fn main() -> i64 = classify(5);"
        );
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_nested_struct_access() {
        let result = run_program(
            "struct Inner { val: i64 }
             struct Outer { inner: Inner }
             fn main() -> i64 = { let o = new Outer { inner: new Inner { val: 42 } }; o.inner.val };"
        );
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_while_with_break_condition() {
        let result = run_program(
            "fn main() -> i64 = { let mut i = 0; while i < 100 { i = i + 7; 0 }; i };"
        );
        // i goes 0,7,14,...,98,105 — stops when 105 >= 100
        assert_eq!(result, Value::Int(105));
    }

    #[test]
    fn test_string_concat_chain() {
        let result = run_program(r#"fn main() -> String = "a" + "b" + "c";"#);
        assert_eq!(result, Value::Str(Rc::new("abc".to_string())));
    }

    #[test]
    fn test_comparison_chained() {
        // Chaining via function calls
        let result = run_program(
            "fn max(a: i64, b: i64) -> i64 = if a > b { a } else { b };
             fn main() -> i64 = max(max(3, 7), max(5, 2));"
        );
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_let_binding_shadowing() {
        let result = run_program(
            "fn main() -> i64 = { let x = 10; let x = x + 5; x };"
        );
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_complex_block_expression() {
        let result = run_program(
            "fn main() -> i64 = { let a = 1; let b = 2; let c = { let d = a + b; d * 2 }; c + 1 };"
        );
        assert_eq!(result, Value::Int(7)); // d = 3, c = 6, c + 1 = 7
    }

    #[test]
    fn test_bitwise_operations() {
        assert_eq!(run_program("fn main() -> i64 = 5 band 3;"), Value::Int(1));   // 101 & 011 = 001
        assert_eq!(run_program("fn main() -> i64 = 5 bor 3;"), Value::Int(7));    // 101 | 011 = 111
        assert_eq!(run_program("fn main() -> i64 = 5 bxor 3;"), Value::Int(6));   // 101 ^ 011 = 110
    }

    #[test]
    fn test_shift_operations() {
        assert_eq!(run_program("fn main() -> i64 = 1 << 4;"), Value::Int(16));
        assert_eq!(run_program("fn main() -> i64 = 16 >> 2;"), Value::Int(4));
    }

    #[test]
    fn test_multiple_function_calls() {
        let result = run_program(
            "fn sq(x: i64) -> i64 = x * x;
             fn cube(x: i64) -> i64 = x * sq(x);
             fn main() -> i64 = cube(3);"
        );
        assert_eq!(result, Value::Int(27));
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(run_program("fn main() -> i64 = 0 - 42;"), Value::Int(-42));
        assert_eq!(run_program("fn main() -> i64 = (0 - 5) * (0 - 3);"), Value::Int(15));
    }

    // --- Cycle 90: Extended interpreter tests ---

    #[test]
    fn test_as_cast_i64_to_f64() {
        let result = run_program("fn main() -> f64 = 42 as f64;");
        assert_eq!(result, Value::Float(42.0));
    }

    #[test]
    fn test_as_cast_f64_to_i64() {
        let result = run_program("fn main() -> i64 = 3.7 as i64;");
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_generic_identity() {
        assert_eq!(
            run_program("fn id<T>(x: T) -> T = x; fn main() -> i64 = id(42);"),
            Value::Int(42)
        );
    }

    #[test]
    fn test_nested_if_else() {
        let result = run_program(
            "fn clamp(x: i64) -> i64 = if x < 0 { 0 } else { if x > 100 { 100 } else { x } };
             fn main() -> i64 = clamp(150);"
        );
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_block_returns_last() {
        assert_eq!(
            run_program("fn main() -> i64 = { 1; 2; 3 };"),
            Value::Int(3)
        );
    }

    #[test]
    fn test_block_with_let_bindings() {
        assert_eq!(
            run_program("fn main() -> i64 = { let a: i64 = 10; let b: i64 = 20; a + b };"),
            Value::Int(30)
        );
    }

    #[test]
    fn test_enum_variant_no_data() {
        let result = run_program(
            "enum Dir { N, S, E, W }
             fn is_n(d: Dir) -> bool = match d { Dir::N => true, _ => false };
             fn main() -> bool = is_n(Dir::N);"
        );
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_char_literal() {
        let result = run_program("fn main() -> char = 'x';");
        assert_eq!(result, Value::Char('x'));
    }

    #[test]
    fn test_i32_arithmetic() {
        assert_eq!(
            run_program("fn main() -> i32 = { let a: i32 = 10; let b: i32 = 20; a + b };"),
            Value::Int(30)
        );
    }

    #[test]
    fn test_u32_literal() {
        assert_eq!(
            run_program("fn main() -> u32 = 42;"),
            Value::Int(42)
        );
    }

    #[test]
    fn test_extern_function_declaration() {
        // extern fns should be registered without error, even if not callable
        let tokens = crate::lexer::tokenize("extern fn ext(x: i64) -> i64; fn main() -> i64 = 0;").expect("tokenize");
        let program = crate::parser::parse("<test>", "extern fn ext(x: i64) -> i64; fn main() -> i64 = 0;", tokens).expect("parse");
        let mut interp = Interpreter::new();
        let result = interp.run(&program).expect("run");
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_type_alias() {
        assert_eq!(
            run_program("type Num = i64; fn main() -> Num = 99;"),
            Value::Int(99)
        );
    }

    #[test]
    fn test_unit_return() {
        assert_eq!(
            run_program("fn noop() -> () = (); fn main() -> () = noop();"),
            Value::Unit
        );
    }

    #[test]
    fn test_nested_function_calls_deep() {
        let result = run_program(
            "fn a(x: i64) -> i64 = x + 1;
             fn b(x: i64) -> i64 = a(x) + 1;
             fn c(x: i64) -> i64 = b(x) + 1;
             fn main() -> i64 = c(0);"
        );
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_pre_condition_passes() {
        let result = run_program(
            "fn safe_div(a: i64, b: i64) -> i64
                 pre b != 0
             = a / b;
             fn main() -> i64 = safe_div(10, 2);"
        );
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_wrapping_add() {
        assert_eq!(
            run_program("fn main() -> i64 = 100 +% 200;"),
            Value::Int(300)
        );
    }

    #[test]
    fn test_tuple_creation_and_access() {
        assert_eq!(
            run_program("fn main() -> (i64, bool) = (42, true);"),
            Value::Tuple(vec![Value::Int(42), Value::Bool(true)])
        );
    }

    #[test]
    fn test_match_integer_patterns() {
        assert_eq!(
            run_program("fn classify(x: i64) -> i64 = match x { 0 => 100, 1 => 200, _ => 300 }; fn main() -> i64 = classify(1);"),
            Value::Int(200)
        );
    }

    #[test]
    fn test_match_wildcard_default() {
        assert_eq!(
            run_program("fn classify(x: i64) -> i64 = match x { 0 => 100, _ => 999 }; fn main() -> i64 = classify(42);"),
            Value::Int(999)
        );
    }

    #[test]
    fn test_string_equality() {
        assert_eq!(
            run_program(r#"fn main() -> bool = "hello" == "hello";"#),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_string_inequality() {
        assert_eq!(
            run_program(r#"fn main() -> bool = "hello" != "world";"#),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_bool_equality() {
        assert_eq!(
            run_program("fn main() -> bool = true == true;"),
            Value::Bool(true)
        );
        assert_eq!(
            run_program("fn main() -> bool = true == false;"),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_f64_neg() {
        assert_eq!(
            run_program("fn main() -> f64 = -3.5;"),
            Value::Float(-3.5)
        );
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(
            run_program(
                "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };
                 fn main() -> i64 = fib(10);"
            ),
            Value::Int(55)
        );
    }

    #[test]
    fn test_for_loop_with_accumulator() {
        assert_eq!(
            run_program(
                "fn main() -> i64 = { let mut sum: i64 = 0; for i in 0..5 { sum = sum + i }; sum };"
            ),
            Value::Int(10) // 0+1+2+3+4 = 10
        );
    }

    // ====================================================================
    // Cycle 93: Closure capture tests
    // ====================================================================

    #[test]
    fn test_closure_creation() {
        let mut interp = Interpreter::new();
        let env = interp.global_env.clone();
        let closure_expr = Expr::Closure {
            params: vec![crate::ast::ClosureParam {
                name: spanned("x".to_string()),
                ty: None,
            }],
            ret_ty: None,
            body: Box::new(spanned(Expr::Var("x".to_string()))),
        };
        let result = interp.eval(&spanned(closure_expr), &env).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    #[test]
    fn test_closure_type_name() {
        assert_eq!(
            Value::Closure {
                params: vec!["x".to_string()],
                body: Box::new(spanned(Expr::IntLit(0))),
                env: crate::interp::Environment::new().into_ref(),
            }.type_name(),
            "closure"
        );
    }

    #[test]
    fn test_closure_display() {
        let c = Value::Closure {
            params: vec!["a".to_string(), "b".to_string()],
            body: Box::new(spanned(Expr::IntLit(0))),
            env: crate::interp::Environment::new().into_ref(),
        };
        assert_eq!(format!("{}", c), "<closure(a, b)>");
    }

    #[test]
    fn test_closure_is_truthy() {
        let c = Value::Closure {
            params: vec![],
            body: Box::new(spanned(Expr::IntLit(0))),
            env: crate::interp::Environment::new().into_ref(),
        };
        assert!(c.is_truthy());
    }

    #[test]
    fn test_closure_not_equal() {
        let env = crate::interp::Environment::new().into_ref();
        let a = Value::Closure {
            params: vec!["x".to_string()],
            body: Box::new(spanned(Expr::IntLit(0))),
            env: env.clone(),
        };
        let b = Value::Closure {
            params: vec!["x".to_string()],
            body: Box::new(spanned(Expr::IntLit(0))),
            env,
        };
        assert_ne!(a, b); // Closures use identity semantics
    }

    // ====================================================================
    // Cycle 105: Nullable type interpreter tests
    // ====================================================================

    #[test]
    fn test_nullable_maybe_function() {
        // Test: fn maybe(x: i64) -> i64? = if x > 0 { x } else { null };
        let result_positive = run_program(
            "fn maybe(x: i64) -> i64? = if x > 0 { x } else { null };
             fn main() -> i64 = maybe(42);"
        );
        assert_eq!(result_positive, Value::Int(42));

        let result_zero = run_program(
            "fn maybe(x: i64) -> i64? = if x > 0 { x } else { null };
             fn main() -> i64 = maybe(0);"
        );
        assert_eq!(result_zero, Value::Int(0)); // null is represented as 0
    }

    #[test]
    fn test_nullable_unwrap_or() {
        // Test nullable .unwrap_or() method
        let result_some = run_program(
            "fn get_or_default(x: i64) -> i64 = {
               let val: i64? = if x > 0 { x } else { null };
               val.unwrap_or(99)
             };
             fn main() -> i64 = get_or_default(42);"
        );
        assert_eq!(result_some, Value::Int(42));

        let result_none = run_program(
            "fn get_or_default(x: i64) -> i64 = {
               let val: i64? = if x <= 0 { null } else { x };
               val.unwrap_or(99)
             };
             fn main() -> i64 = get_or_default(0);"
        );
        assert_eq!(result_none, Value::Int(99));
    }

    #[test]
    fn test_nullable_is_some() {
        // Test nullable .is_some() method
        let result_true = run_program(
            "fn has_value(x: i64) -> bool = {
               let val: i64? = if x > 0 { x } else { null };
               val.is_some()
             };
             fn main() -> bool = has_value(42);"
        );
        assert_eq!(result_true, Value::Bool(true));

        let result_false = run_program(
            "fn has_value(x: i64) -> bool = {
               let val: i64? = if x <= 0 { null } else { x };
               val.is_some()
             };
             fn main() -> bool = has_value(0);"
        );
        assert_eq!(result_false, Value::Bool(false));
    }

    #[test]
    fn test_nullable_is_none() {
        // Test nullable .is_none() method
        let result_false = run_program(
            "fn is_missing(x: i64) -> bool = {
               let val: i64? = if x > 0 { x } else { null };
               val.is_none()
             };
             fn main() -> bool = is_missing(42);"
        );
        assert_eq!(result_false, Value::Bool(false));

        let result_true = run_program(
            "fn is_missing(x: i64) -> bool = {
               let val: i64? = if x <= 0 { null } else { x };
               val.is_none()
             };
             fn main() -> bool = is_missing(0);"
        );
        assert_eq!(result_true, Value::Bool(true));
    }

    // ====================================================================
    // Cycle 106: Extended interpreter edge-case tests
    // ====================================================================

    #[test]
    fn test_deeply_nested_struct_field_access() {
        // Three levels of struct nesting
        let result = run_program(
            "struct A { v: i64 }
             struct B { a: A }
             struct C { b: B }
             fn main() -> i64 = {
                 let c = new C { b: new B { a: new A { v: 777 } } };
                 c.b.a.v
             };"
        );
        assert_eq!(result, Value::Int(777));
    }

    #[test]
    fn test_array_literal_and_indexing() {
        // Create an array literal and index into it
        let result = run_program(
            "fn main() -> i64 = {
                 let arr = [10, 20, 30, 40, 50];
                 arr[2]
             };"
        );
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_array_len_method() {
        let result = run_program(
            "fn main() -> i64 = {
                 let arr = [1, 2, 3, 4];
                 arr.len()
             };"
        );
        assert_eq!(result, Value::Int(4));
    }

    #[test]
    fn test_string_len_method() {
        let result = run_program(
            r#"fn main() -> i64 = "hello world".len();"#
        );
        assert_eq!(result, Value::Int(11));
    }

    #[test]
    fn test_string_concat_operator() {
        // BMB uses ++ for string concat, but + also works in the interpreter
        let result = run_program(
            r#"fn main() -> String = "foo" + "bar" + "baz";"#
        );
        assert_eq!(result, Value::Str(Rc::new("foobarbaz".to_string())));
    }

    #[test]
    fn test_multiple_if_else_return_paths() {
        // Four distinct return paths through nested if/else
        let result = run_program(
            "fn categorize(x: i64) -> i64 =
                 if x < 0 { 1 }
                 else { if x == 0 { 2 }
                 else { if x < 100 { 3 }
                 else { 4 } } };
             fn main() -> i64 = categorize(0 - 5) + categorize(0) * 10 + categorize(50) * 100 + categorize(200) * 1000;"
        );
        // 1 + 20 + 300 + 4000 = 4321
        assert_eq!(result, Value::Int(4321));
    }

    #[test]
    fn test_mutual_recursion_is_odd() {
        // Verify the odd path of mutual recursion
        let result = run_program(
            "fn is_even(n: i64) -> bool = if n == 0 { true } else { is_odd(n - 1) };
             fn is_odd(n: i64) -> bool = if n == 0 { false } else { is_even(n - 1) };
             fn main() -> i64 = if is_odd(7) { 1 } else { 0 };"
        );
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_negative_number_arithmetic() {
        // Operations on negative numbers
        let result = run_program(
            "fn main() -> i64 = {
                 let a = 0 - 10;
                 let b = 0 - 3;
                 a * b + a / b + a % b
             };"
        );
        // (-10)*(-3) = 30, (-10)/(-3) = 3, (-10)%(-3) = -1
        // 30 + 3 + (-1) = 32
        assert_eq!(result, Value::Int(32));
    }

    #[test]
    fn test_deeply_nested_function_calls() {
        // f(g(h(x))) style triple nesting
        let result = run_program(
            "fn double(x: i64) -> i64 = x * 2;
             fn add_one(x: i64) -> i64 = x + 1;
             fn square(x: i64) -> i64 = x * x;
             fn main() -> i64 = square(add_one(double(3)));"
        );
        // double(3) = 6, add_one(6) = 7, square(7) = 49
        assert_eq!(result, Value::Int(49));
    }

    #[test]
    fn test_match_multiple_literal_patterns() {
        // Match with many integer arms
        let result = run_program(
            "fn day_type(d: i64) -> i64 = match d {
                 1 => 10,
                 2 => 20,
                 3 => 30,
                 4 => 40,
                 5 => 50,
                 6 => 60,
                 7 => 70,
                 _ => 0
             };
             fn main() -> i64 = day_type(4) + day_type(7) + day_type(99);"
        );
        // 40 + 70 + 0 = 110
        assert_eq!(result, Value::Int(110));
    }

    #[test]
    fn test_while_loop_countdown() {
        // While loop that counts down from 10
        let result = run_program(
            "fn main() -> i64 = {
                 let mut n = 10;
                 let mut sum = 0;
                 while n > 0 {
                     sum = sum + n;
                     n = n - 1;
                     0
                 };
                 sum
             };"
        );
        // 10+9+8+7+6+5+4+3+2+1 = 55
        assert_eq!(result, Value::Int(55));
    }

    #[test]
    fn test_for_loop_product() {
        // For loop computing factorial(5) = 120
        let result = run_program(
            "fn main() -> i64 = {
                 let mut product: i64 = 1;
                 for i in 1..6 {
                     product = product * i
                 };
                 product
             };"
        );
        assert_eq!(result, Value::Int(120));
    }

    #[test]
    fn test_multiple_let_bindings_chain() {
        // Chain of let bindings each depending on the previous
        let result = run_program(
            "fn main() -> i64 = {
                 let a = 1;
                 let b = a + 2;
                 let c = b * 3;
                 let d = c - 1;
                 let e = d / 2;
                 e
             };"
        );
        // a=1, b=3, c=9, d=8, e=4
        assert_eq!(result, Value::Int(4));
    }

    #[test]
    fn test_variable_shadowing_across_functions() {
        // Variable shadowing: each function has its own scope
        let result = run_program(
            "fn compute(x: i64) -> i64 = { let x = x * 2; x + 1 };
             fn main() -> i64 = {
                 let x = 10;
                 let y = compute(x);
                 x + y
             };"
        );
        // compute(10): x is shadowed to 20, returns 21
        // main: x is still 10 => 10 + 21 = 31
        assert_eq!(result, Value::Int(31));
    }

    #[test]
    fn test_expression_block_as_function_arg() {
        // A block expression used directly as a function argument
        let result = run_program(
            "fn add(a: i64, b: i64) -> i64 = a + b;
             fn main() -> i64 = add({ let x = 3; x * x }, { let y = 4; y * y });"
        );
        // 9 + 16 = 25
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_recursive_power() {
        // Recursive exponentiation: power(2, 10) = 1024
        let result = run_program(
            "fn power(base: i64, exp: i64) -> i64 =
                 if exp == 0 { 1 }
                 else { base * power(base, exp - 1) };
             fn main() -> i64 = power(2, 10);"
        );
        assert_eq!(result, Value::Int(1024));
    }

    #[test]
    fn test_struct_passed_to_function() {
        // Create struct, pass to function, access fields
        let result = run_program(
            "struct Rect { w: i64, h: i64 }
             fn area(r: Rect) -> i64 = r.w * r.h;
             fn perimeter(r: Rect) -> i64 = 2 * (r.w + r.h);
             fn main() -> i64 = {
                 let r = new Rect { w: 5, h: 3 };
                 area(r) + perimeter(r)
             };"
        );
        // area = 15, perimeter = 16 => 31
        assert_eq!(result, Value::Int(31));
    }

    #[test]
    fn test_bitwise_combined_operations() {
        // Combined bitwise and shift operations
        let result = run_program(
            "fn main() -> i64 = {
                 let a = 255;
                 let b = a band 15;
                 let c = b << 4;
                 let d = c bor 5;
                 d
             };"
        );
        // a = 255, b = 15, c = 240, d = 240 | 5 = 245
        assert_eq!(result, Value::Int(245));
    }

    // ================================================================
    // Cycle 124: Extended Interpreter Tests
    // ================================================================

    #[test]
    fn test_nested_function_calls_arithmetic() {
        // f(g(h(1))) where each function doubles
        let result = run_program(
            "fn h(x: i64) -> i64 = x * 2;
             fn g(x: i64) -> i64 = x * 2;
             fn f(x: i64) -> i64 = x * 2;
             fn main() -> i64 = f(g(h(1)));"
        );
        assert_eq!(result, Value::Int(8));
    }

    #[test]
    fn test_fibonacci_15() {
        let result = run_program(
            "fn fib(n: i64) -> i64 = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) };
             fn main() -> i64 = fib(15);"
        );
        assert_eq!(result, Value::Int(610));
    }

    #[test]
    fn test_gcd_euclidean() {
        let result = run_program(
            "fn gcd(a: i64, b: i64) -> i64 = if b == 0 { a } else { gcd(b, a % b) };
             fn main() -> i64 = gcd(48, 18);"
        );
        assert_eq!(result, Value::Int(6));
    }

    #[test]
    fn test_enum_match_with_data() {
        let result = run_program(
            "enum Op { Add(i64), Mul(i64), Nop }
             fn apply(x: i64, op: Op) -> i64 = match op {
                 Op::Add(n) => x + n,
                 Op::Mul(n) => x * n,
                 Op::Nop => x,
             };
             fn main() -> i64 = {
                 let r1 = apply(10, Op::Add(5));
                 let r2 = apply(r1, Op::Mul(2));
                 apply(r2, Op::Nop)
             };"
        );
        // 10+5=15, 15*2=30, 30
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_ackermann_function() {
        // Ackermann function: deeply recursive
        let result = run_program(
            "fn ack(m: i64, n: i64) -> i64 =
                 if m == 0 { n + 1 }
                 else if n == 0 { ack(m - 1, 1) }
                 else { ack(m - 1, ack(m, n - 1)) };
             fn main() -> i64 = ack(2, 3);"
        );
        // ack(2, 3) = 9
        assert_eq!(result, Value::Int(9));
    }

    #[test]
    fn test_while_loop_sum_1_to_100() {
        let result = run_program(
            "fn main() -> i64 = {
                 let mut s = 0;
                 let mut i = 1;
                 while i <= 100 {
                     s = s + i;
                     i = i + 1
                 };
                 s
             };"
        );
        assert_eq!(result, Value::Int(5050));
    }

    #[test]
    fn test_for_loop_factorial() {
        let result = run_program(
            "fn main() -> i64 = {
                 let mut result = 1;
                 for i in 1..11 {
                     result = result * i
                 };
                 result
             };"
        );
        // 10! = 3628800
        assert_eq!(result, Value::Int(3628800));
    }

    #[test]
    fn test_nested_structs_with_methods() {
        let result = run_program(
            "struct Vec2 { x: i64, y: i64 }
             fn dot(a: Vec2, b: Vec2) -> i64 = a.x * b.x + a.y * b.y;
             fn main() -> i64 = {
                 let v1 = new Vec2 { x: 3, y: 4 };
                 let v2 = new Vec2 { x: 5, y: 6 };
                 dot(v1, v2)
             };"
        );
        // 3*5 + 4*6 = 15 + 24 = 39
        assert_eq!(result, Value::Int(39));
    }

    #[test]
    fn test_string_concatenation_chain() {
        let result = run_program(
            r#"fn main() -> i64 = {
                 let s = "a" + "b" + "c" + "d";
                 s.len()
             };"#
        );
        assert_eq!(result, Value::Int(4));
    }

    #[test]
    fn test_boolean_short_circuit_and() {
        // a && b: if a is false, b should not be evaluated
        let result = run_program(
            "fn main() -> i64 = {
                 let x = false && true;
                 if x { 1 } else { 0 }
             };"
        );
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_boolean_short_circuit_or() {
        let result = run_program(
            "fn main() -> i64 = {
                 let x = true || false;
                 if x { 1 } else { 0 }
             };"
        );
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_nested_if_else_chain() {
        let result = run_program(
            "fn classify(x: i64) -> i64 =
                 if x > 100 { 4 }
                 else if x > 50 { 3 }
                 else if x > 10 { 2 }
                 else if x > 0 { 1 }
                 else { 0 };
             fn main() -> i64 = classify(75);"
        );
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_modulo_operations() {
        let result = run_program(
            "fn main() -> i64 = {
                 let a = 17 % 5;
                 let b = 100 % 7;
                 a + b
             };"
        );
        // 17 % 5 = 2, 100 % 7 = 2 => 4
        assert_eq!(result, Value::Int(4));
    }

    #[test]
    fn test_complex_expression_evaluation() {
        let result = run_program(
            "fn main() -> i64 = (3 + 4) * (10 - 2) / 2;"
        );
        // (7) * (8) / 2 = 56 / 2 = 28
        assert_eq!(result, Value::Int(28));
    }

    #[test]
    fn test_power_of_two_check() {
        let result = run_program(
            "fn is_pow2(n: i64) -> i64 =
                 if n <= 0 { 0 }
                 else if n band (n - 1) == 0 { 1 }
                 else { 0 };
             fn main() -> i64 = is_pow2(16) + is_pow2(15) + is_pow2(8);"
        );
        // 16 is pow2(1), 15 is not(0), 8 is pow2(1) => 2
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_tuple_three_element_sum() {
        let result = run_program(
            "fn main() -> i64 = {
                 let t = (10, 20, 30);
                 t.0 + t.1 + t.2
             };"
        );
        assert_eq!(result, Value::Int(60));
    }

    // ================================================================
    // Cycle 210: Extended edge-case interpreter tests
    // ================================================================

    #[test]
    fn test_loop_break_exits() {
        // loop with break should terminate and yield Unit
        let result = run_program(
            "fn main() -> i64 = {
                 let mut count = 0;
                 loop {
                     count = count + 1;
                     if count == 5 { break } else { () };
                     0
                 };
                 count
             };"
        );
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_while_continue_skips_iteration() {
        // continue in while loop should skip the rest of the body
        let result = run_program(
            "fn main() -> i64 = {
                 let mut sum = 0;
                 let mut i = 0;
                 while i < 10 {
                     { i = i + 1 };
                     if i % 2 == 0 { continue } else { () };
                     { sum = sum + i }
                 };
                 sum
             };"
        );
        // odd numbers 1..=9: 1+3+5+7+9 = 25
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_return_from_function_body() {
        // Early return should short-circuit the rest of the function
        let result = run_program(
            "fn early(x: i64) -> i64 = {
                 if x > 10 { return 999 } else { () };
                 x + 1
             };
             fn main() -> i64 = early(50) + early(5);"
        );
        // early(50) returns 999, early(5) returns 6 => 1005
        assert_eq!(result, Value::Int(1005));
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        // Accessing beyond array length should produce an error
        let result = run_source(
            "fn main() -> i64 = {
                 let arr = [10, 20, 30];
                 arr[5]
             };"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_array_index_assign() {
        // Mutable array element assignment via set a[i] = val
        let result = run_program(
            "fn main() -> i64 = {
                 let mut arr = [1, 2, 3, 4, 5];
                 set arr[2] = 99;
                 arr[2]
             };"
        );
        assert_eq!(result, Value::Int(99));
    }

    #[test]
    fn test_array_repeat_syntax() {
        // [val; N] should create N-element array filled with val
        let result = run_program(
            "fn main() -> i64 = {
                 let arr = [0; 5];
                 arr.len()
             };"
        );
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_string_slice_method() {
        let result = run_program(
            r#"fn main() -> String = "hello world".slice(0, 5);"#
        );
        assert_eq!(result, Value::Str(Rc::new("hello".to_string())));
    }

    #[test]
    fn test_string_is_empty_method() {
        assert_eq!(
            run_program(r#"fn main() -> bool = "".is_empty();"#),
            Value::Bool(true)
        );
        assert_eq!(
            run_program(r#"fn main() -> bool = "abc".is_empty();"#),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_pre_condition_failure() {
        // Calling a function that violates its pre-condition should error
        let result = run_source(
            "fn safe_div(a: i64, b: i64) -> i64
                 pre b != 0
             = a / b;
             fn main() -> i64 = safe_div(10, 0);"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_wrapping_sub_and_mul() {
        // Wrapping subtraction and multiplication should work normally
        assert_eq!(
            run_program("fn main() -> i64 = 10 -% 3;"),
            Value::Int(7)
        );
        assert_eq!(
            run_program("fn main() -> i64 = 6 *% 7;"),
            Value::Int(42)
        );
    }

    #[test]
    fn test_saturating_arithmetic() {
        // Saturating add/sub should clamp at i64 boundaries
        assert_eq!(
            run_program("fn main() -> i64 = 100 +| 200;"),
            Value::Int(300)
        );
        assert_eq!(
            run_program("fn main() -> i64 = 5 -| 10;"),
            Value::Int(-5)
        );
        assert_eq!(
            run_program("fn main() -> i64 = 3 *| 4;"),
            Value::Int(12)
        );
    }

    #[test]
    fn test_cast_bool_to_i64() {
        assert_eq!(
            run_program("fn main() -> i64 = true as i64;"),
            Value::Int(1)
        );
        assert_eq!(
            run_program("fn main() -> i64 = false as i64;"),
            Value::Int(0)
        );
    }

    #[test]
    fn test_cast_i64_to_bool() {
        assert_eq!(
            run_program("fn main() -> bool = 42 as bool;"),
            Value::Bool(true)
        );
        assert_eq!(
            run_program("fn main() -> bool = 0 as bool;"),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_cast_f64_to_bool() {
        assert_eq!(
            run_program("fn main() -> bool = 1.5 as bool;"),
            Value::Bool(true)
        );
        assert_eq!(
            run_program("fn main() -> bool = 0.0 as bool;"),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_f64_subtraction_and_multiplication() {
        let result = run_program("fn main() -> f64 = (10.5 - 3.5) * 2.0;");
        assert_eq!(result, Value::Float(14.0));
    }

    #[test]
    fn test_for_loop_inclusive_range() {
        // ..= inclusive range should include the end value
        let result = run_program(
            "fn main() -> i64 = {
                 let mut sum = 0;
                 for i in 1..=5 { sum = sum + i };
                 sum
             };"
        );
        // 1+2+3+4+5 = 15
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_struct_field_assign() {
        // Mutable struct field assignment via set obj.field = val
        let result = run_program(
            "struct Point { x: i64, y: i64 }
             fn main() -> i64 = {
                 let mut p = new Point { x: 1, y: 2 };
                 set p.x = 10;
                 set p.y = 20;
                 p.x + p.y
             };"
        );
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_for_loop_continue() {
        // continue inside for loop should skip to next iteration
        let result = run_program(
            "fn main() -> i64 = {
                 let mut sum = 0;
                 for i in 0..10 {
                     if i % 3 == 0 { continue } else { () };
                     { sum = sum + i }
                 };
                 sum
             };"
        );
        // Skip 0,3,6,9 => sum = 1+2+4+5+7+8 = 27
        assert_eq!(result, Value::Int(27));
    }
}
