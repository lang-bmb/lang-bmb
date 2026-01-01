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
    /// Current function return type (for `ret` keyword)
    current_ret_ty: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            functions: HashMap::new(),
            current_ret_ty: None,
        }
    }

    /// Check entire program
    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect function signatures
        for item in &program.items {
            match item {
                Item::FnDef(f) => {
                    let param_tys: Vec<_> = f.params.iter().map(|p| p.ty.node.clone()).collect();
                    self.functions
                        .insert(f.name.node.clone(), (param_tys, f.ret_ty.node.clone()));
                }
            }
        }

        // Second pass: type check function bodies
        for item in &program.items {
            match item {
                Item::FnDef(f) => self.check_fn(f)?,
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
            Expr::IntLit(_) => Ok(Type::I32),
            Expr::FloatLit(_) => Ok(Type::F64),
            Expr::BoolLit(_) => Ok(Type::Bool),
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
        }
    }

    /// Check binary operation types
    fn check_binary_op(&self, op: BinOp, left: &Type, right: &Type, span: Span) -> Result<Type> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                self.unify(left, right, span)?;
                match left {
                    Type::I32 | Type::I64 | Type::F64 => Ok(left.clone()),
                    _ => Err(CompileError::type_error(
                        format!("arithmetic operator requires numeric type, got {left}"),
                        span,
                    )),
                }
            }

            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                self.unify(left, right, span)?;
                match left {
                    Type::I32 | Type::I64 | Type::F64 | Type::Bool => Ok(Type::Bool),
                    _ => Err(CompileError::type_error(
                        format!("comparison operator requires comparable type, got {left}"),
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
