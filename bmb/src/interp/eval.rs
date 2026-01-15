//! Expression evaluator

use super::env::{child_env, EnvRef, Environment};
use super::error::{InterpResult, RuntimeError};
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
        };
        interp.register_builtins();
        interp
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

        // v0.31.22: Command-line argument builtins for Phase 32.3.D CLI Independence
        self.builtins.insert("arg_count".to_string(), builtin_arg_count);
        self.builtins.insert("get_arg".to_string(), builtin_get_arg);

        // v0.31.13: StringBuilder builtins for Phase 32.0.4 O(n²) fix
        self.builtins.insert("sb_new".to_string(), builtin_sb_new);
        self.builtins.insert("sb_push".to_string(), builtin_sb_push);
        self.builtins.insert("sb_build".to_string(), builtin_sb_build);
        self.builtins.insert("sb_len".to_string(), builtin_sb_len);
        self.builtins.insert("sb_clear".to_string(), builtin_sb_clear);

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

        // v0.34.2: Memory allocation for Phase 34.2 Dynamic Collections
        self.builtins.insert("malloc".to_string(), builtin_malloc);
        self.builtins.insert("free".to_string(), builtin_free);
        self.builtins.insert("realloc".to_string(), builtin_realloc);
        self.builtins.insert("calloc".to_string(), builtin_calloc);
        self.builtins.insert("store_i64".to_string(), builtin_store_i64);
        self.builtins.insert("load_i64".to_string(), builtin_load_i64);
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
                // v0.20.1: Trait system not yet supported in interpreter
                crate::ast::Item::TraitDef(_) => {}
                crate::ast::Item::ImplBlock(_) => {}
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

            // v0.37: Invariant is for SMT verification, not runtime
            Expr::While { cond, invariant: _, body } => {
                while self.eval(cond, env)?.is_truthy() {
                    self.eval(body, env)?;
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
                            self.eval(body, &child)?;
                        }
                        Ok(Value::Unit)
                    }
                    _ => Err(RuntimeError::type_error("Range", iter_val.type_name())),
                }
            }

            Expr::Call { func, args } => {
                let arg_vals: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval(a, env))
                    .collect::<InterpResult<Vec<_>>>()?;

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
                let obj = self.eval(obj_expr, env)?;
                match obj {
                    Value::Struct(_, fields) => {
                        fields.get(&field.node).cloned()
                            .ok_or_else(|| RuntimeError::type_error("field", &field.node))
                    }
                    _ => Err(RuntimeError::type_error("struct", obj.type_name())),
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

                match arr_val {
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
                        let s = arr_val.materialize_string()
                            .ok_or_else(|| RuntimeError::type_error("string", "invalid StringRope"))?;
                        if idx < s.len() {
                            Ok(Value::Int(s.as_bytes()[idx] as i64))
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, s.len()))
                        }
                    }
                    _ => Err(RuntimeError::type_error("array or string", arr_val.type_name())),
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


            // v0.20.0: Closure expressions
            // TODO: Implement closure evaluation with proper capture
            Expr::Closure { body, .. } => {
                // For now, just evaluate the body directly
                // Full closure semantics (capture, delayed execution) will be implemented later
                self.eval(body, env)
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
                        Err(e) => return Err(e),
                    }
                }
            }

            // Break - not yet fully implemented (needs loop context)
            Expr::Break { .. } => {
                Err(RuntimeError::type_error("loop context", "break outside loop"))
            }

            // Continue - not yet fully implemented (needs loop context)
            Expr::Continue => {
                Err(RuntimeError::type_error("loop context", "continue outside loop"))
            }

            // Return - early return from function
            Expr::Return { value } => {
                match value {
                    Some(v) => self.eval(v, env),
                    None => Ok(Value::Unit),
                }
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

    /// Evaluate method call (v0.5 Phase 8, v0.30.283: StringRope support)
    fn eval_method_call(&self, receiver: Value, method: &str, args: Vec<Value>) -> InterpResult<Value> {
        match receiver {
            // v0.30.283: Handle StringRope by materializing
            Value::StringRope(_) => {
                let materialized = receiver.materialize_string()
                    .ok_or_else(|| RuntimeError::type_error("string", "invalid StringRope"))?;
                let s = Rc::new(materialized);
                self.eval_method_call(Value::Str(s), method, args)
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
                    _ => Err(RuntimeError::undefined_function(&format!("String.{}", method))),
                }
            }
            Value::Array(arr) => {
                match method {
                    "len" => Ok(Value::Int(arr.len() as i64)),
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
                    _ => Err(RuntimeError::undefined_function(&format!("Result.{}", method))),
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

        // Evaluate body
        let result = self.eval(&fn_def.body, &func_env);
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
            _ => Err(RuntimeError::type_error(
                &format!("{:?}", target_ty),
                &format!("cannot cast {} to {:?}", val.type_name(), target_ty),
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
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Ne => Ok(Value::Bool(left != right)),
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
                    self.eval_fast(body)?;
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
                match arr_val {
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
                        let s = arr_val.materialize_string()
                            .ok_or_else(|| RuntimeError::type_error("string", "invalid StringRope"))?;
                        if idx < s.len() {
                            Ok(Value::Int(s.as_bytes()[idx] as i64))
                        } else {
                            Err(RuntimeError::index_out_of_bounds(idx as i64, s.len()))
                        }
                    }
                    _ => Err(RuntimeError::type_error("array or string", arr_val.type_name())),
                }
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

        let result = self.eval_fast(&fn_def.body);
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

/// free(ptr: i64) -> unit
/// Frees memory allocated by malloc.
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
            Ok(Value::Unit)
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

/// store_i64(ptr: i64, value: i64) -> ()
/// Stores an i64 value at the given memory address.
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
            Ok(Value::Unit)
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

/// chr(code: i64) -> char
/// Converts a Unicode codepoint to a character.
/// v0.31.21: Added for gotgan string handling
/// v0.65: Updated to return char type with full Unicode support
fn builtin_chr(args: &[Value]) -> InterpResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch("chr", 1, args.len()));
    }
    match &args[0] {
        Value::Int(code) => {
            if *code < 0 {
                Err(RuntimeError::io_error(&format!("chr: negative code {}", code)))
            } else if let Some(c) = char::from_u32(*code as u32) {
                Ok(Value::Char(c))
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
}
