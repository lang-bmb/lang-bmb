//! Expression evaluator

use super::env::{child_env, EnvRef, Environment};
use super::error::{InterpResult, RuntimeError};
use super::value::Value;
use crate::ast::{BinOp, EnumDef, Expr, FnDef, Pattern, Program, Spanned, StructDef, UnOp};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::rc::Rc;

/// Maximum recursion depth (v0.30.248: increased for bootstrap compiler Stage 3 verification)
const MAX_RECURSION_DEPTH: usize = 100000;

/// Stack growth parameters for deep recursion
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
        };
        interp.register_builtins();
        interp
    }

    /// Register built-in functions
    fn register_builtins(&mut self) {
        self.builtins.insert("print".to_string(), builtin_print);
        self.builtins.insert("println".to_string(), builtin_println);
        self.builtins.insert("assert".to_string(), builtin_assert);
        self.builtins.insert("read_int".to_string(), builtin_read_int);
        self.builtins.insert("abs".to_string(), builtin_abs);
        self.builtins.insert("min".to_string(), builtin_min);
        self.builtins.insert("max".to_string(), builtin_max);
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
            Expr::StringLit(s) => Ok(Value::Str(Rc::new(s.clone()))),
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

            Expr::Let {
                name,
                mutable: _,
                ty: _,
                value,
                body,
            } => {
                let val = self.eval(value, env)?;
                let child = child_env(env);
                child.borrow_mut().define(name.clone(), val);
                self.eval(body, &child)
            }

            Expr::Assign { name, value } => {
                let val = self.eval(value, env)?;
                if !env.borrow_mut().set(name, val.clone()) {
                    return Err(RuntimeError::undefined_variable(name));
                }
                Ok(Value::Unit)
            }

            Expr::While { cond, body } => {
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

            // v0.13.2: Try block - evaluate body and wrap result
            Expr::Try { body } => {
                // For now, try blocks just evaluate the body
                // Full Result type support will be added with generic type checking
                self.eval(body, env)
            }

            // v0.13.2: Question mark operator - propagate errors
            Expr::Question { expr: inner } => {
                // For now, just evaluate the inner expression
                // Full error propagation will be added with Result type support
                self.eval(inner, env)
            }

            // v0.20.0: Closure expressions
            // TODO: Implement closure evaluation with proper capture
            Expr::Closure { body, .. } => {
                // For now, just evaluate the body directly
                // Full closure semantics (capture, delayed execution) will be implemented later
                self.eval(body, env)
            }
        }
    }

    /// Evaluate method call (v0.5 Phase 8)
    fn eval_method_call(&self, receiver: Value, method: &str, args: Vec<Value>) -> InterpResult<Value> {
        match receiver {
            Value::Str(s) => {
                match method {
                    "len" => Ok(Value::Int(s.len() as i64)),
                    "char_at" => {
                        if args.len() != 1 {
                            return Err(RuntimeError::arity_mismatch("char_at", 1, args.len()));
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
                    _ => None,
                }
            }

            Pattern::EnumVariant { enum_name, variant, bindings } => {
                match value {
                    Value::Enum(e_name, v_name, args) if e_name == enum_name && v_name == variant => {
                        if bindings.len() != args.len() {
                            return None;
                        }
                        let mut result = vec![];
                        for (binding, arg) in bindings.iter().zip(args.iter()) {
                            result.push((binding.node.clone(), arg.clone()));
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

    /// Evaluate binary operation
    fn eval_binary(&self, op: BinOp, left: Value, right: Value) -> InterpResult<Value> {
        match op {
            // Arithmetic
            BinOp::Add => match (&left, &right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                // String concatenation (v0.30.256: pre-allocated capacity, v0.30.268: Rc<String>)
                (Value::Str(a), Value::Str(b)) => {
                    let mut result = String::with_capacity(a.len() + b.len());
                    result.push_str(a);
                    result.push_str(b);
                    Ok(Value::Str(Rc::new(result)))
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
