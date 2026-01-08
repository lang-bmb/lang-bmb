//! Type checking

use std::collections::HashMap;

use crate::ast::*;
use crate::error::{CompileError, Result};
use crate::resolver::Module;

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
        // getenv(name: String) -> String (env var value)
        functions.insert("getenv".to_string(), (vec![Type::String], Type::String));

        // v0.31.22: Command-line argument builtins for Phase 32.3.D CLI Independence
        // arg_count() -> i64 (number of arguments including program name)
        functions.insert("arg_count".to_string(), (vec![], Type::I64));
        // get_arg(n: i64) -> String (nth argument, 0 = program name)
        functions.insert("get_arg".to_string(), (vec![Type::I64], Type::String));

        // v0.31.13: StringBuilder builtins for Phase 32.0.4 O(n²) fix
        // sb_new() -> i64 (builder ID)
        functions.insert("sb_new".to_string(), (vec![], Type::I64));
        // sb_push(id: i64, str: String) -> i64 (same ID for chaining)
        functions.insert("sb_push".to_string(), (vec![Type::I64, Type::String], Type::I64));
        // sb_build(id: i64) -> String (final string)
        functions.insert("sb_build".to_string(), (vec![Type::I64], Type::String));
        // sb_len(id: i64) -> i64 (total length)
        functions.insert("sb_len".to_string(), (vec![Type::I64], Type::I64));
        // sb_clear(id: i64) -> i64 (same ID)
        functions.insert("sb_clear".to_string(), (vec![Type::I64], Type::I64));

        // v0.31.21: Character conversion builtins for gotgan string handling
        // chr(code: i64) -> String (ASCII code to single-char string)
        functions.insert("chr".to_string(), (vec![Type::I64], Type::String));
        // ord(s: String) -> i64 (first char to ASCII code)
        functions.insert("ord".to_string(), (vec![Type::String], Type::I64));

        // v0.34: Math intrinsics for Phase 34.4 Benchmark Gate (n_body, mandelbrot_fp)
        // sqrt(x: f64) -> f64 (square root)
        functions.insert("sqrt".to_string(), (vec![Type::F64], Type::F64));
        // i64_to_f64(x: i64) -> f64 (type conversion)
        functions.insert("i64_to_f64".to_string(), (vec![Type::I64], Type::F64));
        // f64_to_i64(x: f64) -> i64 (type conversion, truncates toward zero)
        functions.insert("f64_to_i64".to_string(), (vec![Type::F64], Type::I64));

        Self {
            env: HashMap::new(),
            functions,
            generic_functions: HashMap::new(),
            generic_structs: HashMap::new(),
            structs: HashMap::new(),
            generic_enums: HashMap::new(),
            enums: HashMap::new(),
            current_ret_ty: None,
            type_param_env: HashMap::new(),
            traits: HashMap::new(),
            impls: HashMap::new(),
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
                        self.functions.insert(f.name.node.clone(), (param_tys, f.ret_ty.node.clone()));
                    } else {
                        let type_param_names: Vec<_> = f.type_params.iter().map(|tp| tp.name.as_str()).collect();
                        let param_tys: Vec<_> = f.params.iter()
                            .map(|p| self.resolve_type_vars(&p.ty.node, &type_param_names))
                            .collect();
                        let ret_ty = self.resolve_type_vars(&f.ret_ty.node, &type_param_names);
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

    /// Check entire program
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect type definitions (structs and enums)
        for item in &program.items {
            match item {
                Item::StructDef(s) => {
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
                }
                Item::EnumDef(e) => {
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
                }
                // v0.20.1: ImplBlocks are processed in a later pass
                Item::ImplBlock(_) => {}
            }
        }

        // Second pass: collect function signatures (including extern fn)
        for item in &program.items {
            match item {
                Item::FnDef(f) => {
                    // v0.15: Handle generic functions separately
                    if f.type_params.is_empty() {
                        let param_tys: Vec<_> = f.params.iter().map(|p| p.ty.node.clone()).collect();
                        self.functions
                            .insert(f.name.node.clone(), (param_tys, f.ret_ty.node.clone()));
                    } else {
                        // Convert Named types that match type params to TypeVar
                        let type_param_names: Vec<_> = f.type_params.iter().map(|tp| tp.name.as_str()).collect();
                        let param_tys: Vec<_> = f.params.iter()
                            .map(|p| self.resolve_type_vars(&p.ty.node, &type_param_names))
                            .collect();
                        let ret_ty = self.resolve_type_vars(&f.ret_ty.node, &type_param_names);
                        self.generic_functions.insert(
                            f.name.node.clone(),
                            (f.type_params.clone(), param_tys, ret_ty)
                        );
                    }
                }
                // v0.13.0: Register extern function signatures
                Item::ExternFn(e) => {
                    let param_tys: Vec<_> = e.params.iter().map(|p| p.ty.node.clone()).collect();
                    self.functions
                        .insert(e.name.node.clone(), (param_tys, e.ret_ty.node.clone()));
                }
                Item::StructDef(_) | Item::EnumDef(_) | Item::Use(_) => {}
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
            }
        }

        // v0.31: Validate module header exports (RFC-0002)
        if let Some(header) = &program.header {
            self.validate_module_exports(header, program)?;
        }

        Ok(())
    }

    /// Check function definition
    fn check_fn(&mut self, f: &FnDef) -> Result<()> {
        // Clear environment and add parameters
        self.env.clear();
        self.type_param_env.clear();

        // v0.15: Register type parameters for generic functions
        let type_param_names: Vec<_> = f.type_params.iter().map(|tp| tp.name.as_str()).collect();
        for tp in &f.type_params {
            self.type_param_env.insert(tp.name.clone(), tp.bounds.clone());
        }

        // v0.15: Convert Named types that match type params to TypeVar for env
        for param in &f.params {
            let resolved_ty = if f.type_params.is_empty() {
                param.ty.node.clone()
            } else {
                self.resolve_type_vars(&param.ty.node, &type_param_names)
            };
            self.env.insert(param.name.node.clone(), resolved_ty);
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
            Expr::Unit => Ok(Type::Unit),

            Expr::Ret => self.current_ret_ty.clone().ok_or_else(|| {
                CompileError::type_error("'ret' used outside function", span)
            }),

            Expr::Var(name) => self.env.get(name).cloned().ok_or_else(|| {
                CompileError::type_error(format!("undefined variable: {name}"), span)
            }),

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
                self.unify(&then_ty, &else_ty, else_branch.span)?;

                Ok(then_ty)
            }

            Expr::Let {
                name,
                mutable: _,
                ty,
                value,
                body,
            } => {
                let value_ty = self.infer(&value.node, value.span)?;

                if let Some(ann_ty) = ty {
                    self.unify(&ann_ty.node, &value_ty, value.span)?;
                }

                self.env.insert(name.clone(), value_ty);
                self.infer(&body.node, body.span)
            }

            Expr::Assign { name, value } => {
                // Check that variable exists
                let var_ty = self.env.get(name).cloned().ok_or_else(|| {
                    CompileError::type_error(format!("undefined variable: {name}"), span)
                })?;

                // Check that value type matches variable type
                let value_ty = self.infer(&value.node, value.span)?;
                self.unify(&var_ty, &value_ty, value.span)?;

                // Assignment returns unit
                Ok(Type::Unit)
            }

            Expr::While { cond, body } => {
                // Condition must be bool
                let cond_ty = self.infer(&cond.node, cond.span)?;
                self.unify(&Type::Bool, &cond_ty, cond.span)?;

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
                    Type::I32 | Type::I64 => Ok(Type::Range(Box::new(start_ty))),
                    _ => Err(CompileError::type_error(
                        format!("range requires integer types, got {start_ty}"),
                        span,
                    )),
                }
            }

            // v0.5 Phase 3: For loop
            Expr::For { var, iter, body } => {
                let iter_ty = self.infer(&iter.node, iter.span)?;

                // Iterator must be a Range type
                let elem_ty = match &iter_ty {
                    Type::Range(elem) => (**elem).clone(),
                    _ => {
                        return Err(CompileError::type_error(
                            format!("for loop requires Range type, got {iter_ty}"),
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

                // v0.15: Try generic functions
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
                    for tp in &type_params {
                        if !type_subst.contains_key(&tp.name) {
                            return Err(CompileError::type_error(
                                format!("could not infer type for type parameter {}", tp.name),
                                span,
                            ));
                        }
                    }

                    // Substitute type parameters in return type
                    let instantiated_ret_ty = self.substitute_type(&ret_ty, &type_subst);
                    return Ok(instantiated_ret_ty);
                }

                Err(CompileError::type_error(format!("undefined function: {func}"), span))
            }

            Expr::Block(exprs) => {
                if exprs.is_empty() {
                    return Ok(Type::Unit);
                }

                let mut last_ty = Type::Unit;
                for expr in exprs {
                    last_ty = self.infer(&expr.node, expr.span)?;
                }
                Ok(last_ty)
            }

            // v0.5: Struct and Enum expressions
            Expr::StructInit { name, fields } => {
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

                Err(CompileError::type_error(format!("undefined struct: {name}"), span))
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

                        Err(CompileError::type_error(
                            format!("unknown field: {}", field.node),
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

                            return Err(CompileError::type_error(
                                format!("unknown field: {}", field.node),
                                span,
                            ));
                        }
                        Err(CompileError::type_error(
                            format!("not a struct: {struct_name}"),
                            span,
                        ))
                    }
                    _ => Err(CompileError::type_error(
                        format!("field access on non-struct type: {obj_ty}"),
                        span,
                    )),
                }
            }

            Expr::EnumVariant { enum_name, variant, args } => {
                // v0.16: First try non-generic enums
                if let Some(variants) = self.enums.get(enum_name).cloned() {
                    let variant_fields = variants.iter()
                        .find(|(name, _)| name == variant)
                        .map(|(_, fields)| fields.clone())
                        .ok_or_else(|| {
                            CompileError::type_error(format!("unknown variant: {variant}"), span)
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
                            CompileError::type_error(format!("unknown variant: {variant}"), span)
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

                Err(CompileError::type_error(format!("undefined enum: {enum_name}"), span))
            }

            Expr::Match { expr: match_expr, arms } => {
                let match_ty = self.infer(&match_expr.node, match_expr.span)?;

                if arms.is_empty() {
                    return Ok(Type::Unit);
                }

                // All arms must have the same result type
                let mut result_ty: Option<Type> = None;

                for arm in arms {
                    // Check pattern against match expression type
                    self.check_pattern(&arm.pattern.node, &match_ty, arm.pattern.span)?;

                    // Infer body type with pattern bindings
                    let body_ty = self.infer(&arm.body.node, arm.body.span)?;

                    match &result_ty {
                        None => result_ty = Some(body_ty),
                        Some(expected) => self.unify(expected, &body_ty, arm.body.span)?,
                    }
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
                    _ => Err(CompileError::type_error(format!("Cannot dereference non-reference type: {}", inner_ty), span)),
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

            Expr::Index { expr, index } => {
                let expr_ty = self.infer(&expr.node, expr.span)?;
                let index_ty = self.infer(&index.node, index.span)?;

                // Index must be an integer (v0.2: handle refined types)
                match index_ty.base_type() {
                    Type::I32 | Type::I64 => {}
                    _ => return Err(CompileError::type_error(format!("Array index must be integer, got: {}", index_ty), index.span)),
                }

                // Expression must be an array
                match expr_ty {
                    Type::Array(elem_ty, _) => Ok(*elem_ty),
                    Type::String => Ok(Type::I64), // String indexing returns char code
                    _ => Err(CompileError::type_error(format!("Cannot index into type: {}", expr_ty), expr.span)),
                }
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

            // v0.13.2: Try block - type is the body's type wrapped in Result
            Expr::Try { body } => {
                // For now, just return the body's type
                // Full Result<T, E> type inference will be added later
                self.infer(&body.node, body.span)
            }

            // v0.13.2: Question mark operator - unwraps Result/Option
            Expr::Question { expr: inner } => {
                // For now, just return the inner expression's type
                // Full Result/Option unwrapping will be added later
                self.infer(&inner.node, inner.span)
            }

            // v0.20.0: Closure expressions
            Expr::Closure { params, ret_ty, body } => {
                // Save current environment for capture analysis
                let outer_env = self.env.clone();

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
                }

                // Infer body type
                let body_ty = self.infer(&body.node, body.span)?;

                // Check against explicit return type if provided
                if let Some(explicit_ret) = ret_ty {
                    self.unify(&explicit_ret.node, &body_ty, body.span)?;
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
        }
    }

    /// Check method call types (v0.5 Phase 8)
    fn check_method_call(&mut self, receiver_ty: &Type, method: &str, args: &[Spanned<Expr>], span: Span) -> Result<Type> {
        match receiver_ty {
            Type::String => {
                match method {
                    // len() -> i64
                    "len" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("len() takes no arguments", span));
                        }
                        Ok(Type::I64)
                    }
                    // char_at(index: i64) -> i64
                    "char_at" => {
                        if args.len() != 1 {
                            return Err(CompileError::type_error("char_at() takes 1 argument", span));
                        }
                        let arg_ty = self.infer(&args[0].node, args[0].span)?;
                        match arg_ty {
                            Type::I32 | Type::I64 => Ok(Type::I64),
                            _ => Err(CompileError::type_error(
                                format!("char_at() requires integer argument, got {}", arg_ty),
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
                                Type::I32 | Type::I64 => {}
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
                    _ => Err(CompileError::type_error(
                        format!("unknown method '{}' for String", method),
                        span,
                    )),
                }
            }
            Type::Array(_, _) => {
                match method {
                    // len() -> i64
                    "len" => {
                        if !args.is_empty() {
                            return Err(CompileError::type_error("len() takes no arguments", span));
                        }
                        Ok(Type::I64)
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
            // v0.18: Result<T, E> methods
            Type::Named(name) if name == "Result" => {
                self.check_result_method(method, args, None, None, span)
            }
            Type::Generic { name, type_args } if name == "Result" => {
                let ok_ty = type_args.first().map(|t| t.as_ref().clone());
                let err_ty = type_args.get(1).map(|t| t.as_ref().clone());
                self.check_result_method(method, args, ok_ty, err_ty, span)
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

    /// Check pattern validity
    fn check_pattern(&mut self, pattern: &crate::ast::Pattern, expected_ty: &Type, span: Span) -> Result<()> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Wildcard => Ok(()),
            Pattern::Var(name) => {
                // Bind the variable to the expected type
                self.env.insert(name.clone(), expected_ty.clone());
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
                // Check that pattern matches expected type
                match expected_ty {
                    Type::Named(name) if name == enum_name => {
                        // Non-generic enum pattern matching
                        let variants = self.enums.get(enum_name).ok_or_else(|| {
                            CompileError::type_error(format!("undefined enum: {enum_name}"), span)
                        })?;

                        let variant_fields = variants.iter()
                            .find(|(n, _)| n == variant)
                            .map(|(_, fields)| fields.clone())
                            .ok_or_else(|| {
                                CompileError::type_error(format!("unknown variant: {variant}"), span)
                            })?;

                        if bindings.len() != variant_fields.len() {
                            return Err(CompileError::type_error(
                                format!("expected {} bindings, got {}", variant_fields.len(), bindings.len()),
                                span,
                            ));
                        }

                        // Bind pattern variables
                        for (binding, field_ty) in bindings.iter().zip(variant_fields.iter()) {
                            self.env.insert(binding.node.clone(), field_ty.clone());
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
                                CompileError::type_error(format!("unknown variant: {variant}"), span)
                            })?;

                        if bindings.len() != variant_fields.len() {
                            return Err(CompileError::type_error(
                                format!("expected {} bindings, got {}", variant_fields.len(), bindings.len()),
                                span,
                            ));
                        }

                        // Build type substitution from type_params to type_args
                        let mut type_subst: HashMap<String, Type> = HashMap::new();
                        for (tp, arg) in type_params.iter().zip(type_args.iter()) {
                            type_subst.insert(tp.name.clone(), (**arg).clone());
                        }

                        // Bind pattern variables with substituted types
                        for (binding, field_ty) in bindings.iter().zip(variant_fields.iter()) {
                            // Convert Named types to TypeVar, then substitute
                            let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();
                            let resolved_ty = self.resolve_type_vars(field_ty, &type_param_names);
                            let substituted_ty = self.substitute_type(&resolved_ty, &type_subst);
                            self.env.insert(binding.node.clone(), substituted_ty);
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
                                    CompileError::type_error(format!("unknown variant: {variant}"), span)
                                })?;

                            if bindings.len() != variant_fields.len() {
                                return Err(CompileError::type_error(
                                    format!("expected {} bindings, got {}", variant_fields.len(), bindings.len()),
                                    span,
                                ));
                            }

                            // Bind pattern variables with the original field types (may contain type params)
                            let type_param_names: Vec<_> = type_params.iter().map(|tp| tp.name.as_str()).collect();
                            for (binding, field_ty) in bindings.iter().zip(variant_fields.iter()) {
                                let resolved_ty = self.resolve_type_vars(field_ty, &type_param_names);
                                self.env.insert(binding.node.clone(), resolved_ty);
                            }

                            Ok(())
                        } else {
                            Err(CompileError::type_error(format!("undefined enum: {enum_name}"), span))
                        }
                    }
                    _ => Err(CompileError::type_error(
                        format!("expected {}, got enum pattern", expected_ty),
                        span,
                    )),
                }
            }
            Pattern::Struct { name, fields } => {
                match expected_ty {
                    Type::Named(expected_name) if expected_name == name => {
                        let struct_fields = self.structs.get(name).cloned().ok_or_else(|| {
                            CompileError::type_error(format!("undefined struct: {name}"), span)
                        })?;

                        for (field_name, field_pat) in fields {
                            let field_ty = struct_fields.iter()
                                .find(|(n, _)| n == &field_name.node)
                                .map(|(_, ty)| ty.clone())
                                .ok_or_else(|| {
                                    CompileError::type_error(format!("unknown field: {}", field_name.node), span)
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
        }
    }

    /// Check binary operation types
    /// v0.2: Uses base_type() to handle refined types correctly
    fn check_binary_op(&self, op: BinOp, left: &Type, right: &Type, span: Span) -> Result<Type> {
        // v0.2: Extract base types for refined types
        let left_base = left.base_type();
        let right_base = right.base_type();

        match op {
            BinOp::Add => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    Type::I32 | Type::I64 | Type::F64 => Ok(left_base.clone()),
                    Type::String => Ok(Type::String), // String concatenation
                    _ => Err(CompileError::type_error(
                        format!("+ operator requires numeric or String type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    Type::I32 | Type::I64 | Type::F64 => Ok(left_base.clone()),
                    _ => Err(CompileError::type_error(
                        format!("arithmetic operator requires numeric type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Eq | BinOp::Ne => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    Type::I32 | Type::I64 | Type::F64 | Type::Bool | Type::String => Ok(Type::Bool),
                    _ => Err(CompileError::type_error(
                        format!("equality operator requires comparable type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                self.unify(left_base, right_base, span)?;
                match left_base {
                    Type::I32 | Type::I64 | Type::F64 => Ok(Type::Bool),
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
        }
    }

    /// Unify two types
    /// v0.15: Updated to handle TypeVar in generic function body checking
    fn unify(&self, expected: &Type, actual: &Type, span: Span) -> Result<()> {
        // v0.15: TypeVar in function body context matches any type
        // When type checking a generic function body, TypeVar acts as a placeholder
        if let Type::TypeVar(name) = expected
            && self.type_param_env.contains_key(name)
        {
            // TypeVar is bound in current generic context - accept any type
            return Ok(());
        }
        if let Type::TypeVar(name) = actual
            && self.type_param_env.contains_key(name)
        {
            // TypeVar is bound in current generic context - accept any type
            return Ok(());
        }

        // Both are TypeVar with same name
        if let (Type::TypeVar(a), Type::TypeVar(b)) = (expected, actual)
            && a == b
        {
            return Ok(());
        }

        // v0.16: Handle Generic types with TypeVar in type_args
        // e.g., unify Option<i64> with Option<T> where T is a type parameter
        if let (Type::Generic { name: n1, type_args: a1 }, Type::Generic { name: n2, type_args: a2 }) = (expected, actual)
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
        if let Type::TypeVar(_) = expected {
            // Allow any type to match an unbound TypeVar
            return Ok(());
        }
        if let Type::TypeVar(_) = actual {
            // Allow unbound TypeVar to match any expected type
            return Ok(());
        }

        if expected == actual {
            Ok(())
        } else {
            Err(CompileError::type_error(
                format!("expected {expected}, got {actual}"),
                span,
            ))
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
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "String".to_string(),
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
