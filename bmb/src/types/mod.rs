//! Type checking

use std::collections::HashMap;

use crate::ast::*;
use crate::error::{CompileError, Result};

/// Type checker
pub struct TypeChecker {
    /// Variable environment
    env: HashMap<String, Type>,
    /// Function signatures
    functions: HashMap<String, (Vec<Type>, Type)>,
    /// Struct definitions: name -> field types
    structs: HashMap<String, Vec<(String, Type)>>,
    /// Enum definitions: name -> variant info (variant_name, field types)
    enums: HashMap<String, Vec<(String, Vec<Type>)>>,
    /// Current function return type (for `ret` keyword)
    current_ret_ty: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        // Register built-in functions
        // print(x) -> Unit
        functions.insert("print".to_string(), (vec![Type::I64], Type::Unit));
        // println(x) -> Unit
        functions.insert("println".to_string(), (vec![Type::I64], Type::Unit));
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

        Self {
            env: HashMap::new(),
            functions,
            structs: HashMap::new(),
            enums: HashMap::new(),
            current_ret_ty: None,
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
                    self.structs.insert(s.name.node.clone(), fields);
                }
                Item::EnumDef(e) => {
                    let variants: Vec<_> = e.variants.iter()
                        .map(|v| (v.name.node.clone(), v.fields.iter().map(|f| f.node.clone()).collect()))
                        .collect();
                    self.enums.insert(e.name.node.clone(), variants);
                }
                Item::FnDef(_) => {}
            }
        }

        // Second pass: collect function signatures
        for item in &program.items {
            match item {
                Item::FnDef(f) => {
                    let param_tys: Vec<_> = f.params.iter().map(|p| p.ty.node.clone()).collect();
                    self.functions
                        .insert(f.name.node.clone(), (param_tys, f.ret_ty.node.clone()));
                }
                Item::StructDef(_) | Item::EnumDef(_) => {}
            }
        }

        // Third pass: type check function bodies
        for item in &program.items {
            match item {
                Item::FnDef(f) => self.check_fn(f)?,
                Item::StructDef(_) | Item::EnumDef(_) => {}
            }
        }

        Ok(())
    }

    /// Check function definition
    fn check_fn(&mut self, f: &FnDef) -> Result<()> {
        // Clear environment and add parameters
        self.env.clear();
        for param in &f.params {
            self.env.insert(param.name.node.clone(), param.ty.node.clone());
        }

        // Set current return type for `ret` keyword
        self.current_ret_ty = Some(f.ret_ty.node.clone());

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
        self.unify(&f.ret_ty.node, &body_ty, f.body.span)?;

        self.current_ret_ty = None;
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

            // v0.5 Phase 3: Range expression
            Expr::Range { start, end } => {
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
                let (param_tys, ret_ty) = self.functions.get(func).cloned().ok_or_else(|| {
                    CompileError::type_error(format!("undefined function: {func}"), span)
                })?;

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

                Ok(ret_ty)
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
                let struct_fields = self.structs.get(name).cloned().ok_or_else(|| {
                    CompileError::type_error(format!("undefined struct: {name}"), span)
                })?;

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

                Ok(Type::Named(name.clone()))
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
                    _ => Err(CompileError::type_error(
                        format!("field access on non-struct type: {obj_ty}"),
                        span,
                    )),
                }
            }

            Expr::EnumVariant { enum_name, variant, args } => {
                let variants = self.enums.get(enum_name).cloned().ok_or_else(|| {
                    CompileError::type_error(format!("undefined enum: {enum_name}"), span)
                })?;

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

                Ok(Type::Named(enum_name.clone()))
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
    fn check_binary_op(&self, op: BinOp, left: &Type, right: &Type, span: Span) -> Result<Type> {
        match op {
            BinOp::Add => {
                self.unify(left, right, span)?;
                match left {
                    Type::I32 | Type::I64 | Type::F64 => Ok(left.clone()),
                    Type::String => Ok(Type::String), // String concatenation
                    _ => Err(CompileError::type_error(
                        format!("+ operator requires numeric or String type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                self.unify(left, right, span)?;
                match left {
                    Type::I32 | Type::I64 | Type::F64 => Ok(left.clone()),
                    _ => Err(CompileError::type_error(
                        format!("arithmetic operator requires numeric type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Eq | BinOp::Ne => {
                self.unify(left, right, span)?;
                match left {
                    Type::I32 | Type::I64 | Type::F64 | Type::Bool | Type::String => Ok(Type::Bool),
                    _ => Err(CompileError::type_error(
                        format!("equality operator requires comparable type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                self.unify(left, right, span)?;
                match left {
                    Type::I32 | Type::I64 | Type::F64 => Ok(Type::Bool),
                    _ => Err(CompileError::type_error(
                        format!("comparison operator requires numeric type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::And | BinOp::Or => {
                self.unify(&Type::Bool, left, span)?;
                self.unify(&Type::Bool, right, span)?;
                Ok(Type::Bool)
            }
        }
    }

    /// Check unary operation types
    fn check_unary_op(&self, op: UnOp, ty: &Type, span: Span) -> Result<Type> {
        match op {
            UnOp::Neg => match ty {
                Type::I32 | Type::I64 | Type::F64 => Ok(ty.clone()),
                _ => Err(CompileError::type_error(
                    format!("negation requires numeric type, got {ty}"),
                    span,
                )),
            },
            UnOp::Not => {
                self.unify(&Type::Bool, ty, span)?;
                Ok(Type::Bool)
            }
        }
    }

    /// Unify two types
    fn unify(&self, expected: &Type, actual: &Type, span: Span) -> Result<()> {
        if expected == actual {
            Ok(())
        } else {
            Err(CompileError::type_error(
                format!("expected {expected}, got {actual}"),
                span,
            ))
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
