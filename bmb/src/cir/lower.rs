//! AST to CIR Lowering
//!
//! Transforms the typed AST into Contract IR, extracting contracts
//! and converting them to logical propositions.

use std::collections::HashMap;

use crate::ast::{self, BinOp as AstBinOp, Expr, FnDef, Item, Program, Type, UnOp as AstUnOp};

use super::{
    BinOp, CirExpr, CirExternFn, CirFunction, CirParam, CirProgram, CirStruct, CirType,
    CompareOp, EffectSet, LoopInvariant, NamedProposition, Proposition, UnaryOp,
};

/// Lower a typed AST program to CIR
pub fn lower_to_cir(program: &Program) -> CirProgram {
    let mut lowerer = CirLowerer::new();
    lowerer.lower_program(program)
}

/// CIR lowering context
struct CirLowerer {
    /// Struct definitions for reference
    structs: HashMap<String, CirStruct>,
    /// Loop counter for generating unique loop IDs
    loop_counter: usize,
    /// Current function name (for error context)
    current_function: Option<String>,
}

impl CirLowerer {
    fn new() -> Self {
        Self {
            structs: HashMap::new(),
            loop_counter: 0,
            current_function: None,
        }
    }

    fn lower_program(&mut self, program: &Program) -> CirProgram {
        let mut functions = Vec::new();
        let mut extern_fns = Vec::new();
        let type_invariants = HashMap::new();

        // First pass: collect struct definitions
        for item in &program.items {
            if let Item::StructDef(struct_def) = item {
                let cir_struct = self.lower_struct_def(struct_def);
                self.structs.insert(cir_struct.name.clone(), cir_struct);
            }
        }

        // Second pass: lower functions and extern declarations
        for item in &program.items {
            match item {
                Item::FnDef(fn_def) => {
                    if let Some(cir_fn) = self.lower_fn_def(fn_def) {
                        functions.push(cir_fn);
                    }
                }
                Item::ExternFn(extern_fn) => {
                    extern_fns.push(self.lower_extern_fn(extern_fn));
                }
                _ => {}
            }
        }

        CirProgram {
            functions,
            extern_fns,
            structs: self.structs.clone(),
            type_invariants,
        }
    }

    fn lower_struct_def(&self, struct_def: &ast::StructDef) -> CirStruct {
        let fields = struct_def
            .fields
            .iter()
            .map(|f| (f.name.node.clone(), self.lower_type(&f.ty.node)))
            .collect();

        CirStruct {
            name: struct_def.name.node.clone(),
            fields,
            invariants: Vec::new(), // TODO: Extract struct invariants from @invariant attributes
        }
    }

    fn lower_fn_def(&mut self, fn_def: &FnDef) -> Option<CirFunction> {
        self.current_function = Some(fn_def.name.node.clone());

        // Extract type parameters
        let type_params: Vec<String> = fn_def
            .type_params
            .iter()
            .map(|tp| tp.name.clone())
            .collect();

        // Lower parameters
        let params: Vec<CirParam> = fn_def
            .params
            .iter()
            .map(|p| CirParam {
                name: p.name.node.clone(),
                ty: self.lower_type(&p.ty.node),
                constraints: Vec::new(),
            })
            .collect();

        // Return type and name
        let ret_ty = self.lower_type(&fn_def.ret_ty.node);
        let ret_name = fn_def
            .ret_name
            .as_ref()
            .map(|s| s.node.clone())
            .unwrap_or_else(|| "ret".to_string());

        // Extract preconditions and postconditions
        let mut preconditions = Vec::new();
        let mut postconditions = Vec::new();

        // Legacy pre/post
        if let Some(pre) = &fn_def.pre {
            if let Some(prop) = self.expr_to_proposition(&pre.node) {
                preconditions.push(NamedProposition {
                    name: None,
                    proposition: prop,
                });
            }
        }

        if let Some(post) = &fn_def.post {
            if let Some(prop) = self.expr_to_proposition(&post.node) {
                postconditions.push(NamedProposition {
                    name: None,
                    proposition: prop,
                });
            }
        }

        // Named contracts
        for contract in &fn_def.contracts {
            if let Some(prop) = self.expr_to_proposition(&contract.condition.node) {
                // Determine if this is a pre or post condition by checking for 'ret' reference
                let named_prop = NamedProposition {
                    name: contract.name.as_ref().map(|n| n.node.clone()),
                    proposition: prop,
                };

                // For now, assume contracts without 'ret' are preconditions
                if self.references_return_value(&contract.condition.node, &ret_name) {
                    postconditions.push(named_prop);
                } else {
                    preconditions.push(named_prop);
                }
            }
        }

        // Analyze effects from attributes
        let effects = self.analyze_effects(fn_def);

        // Lower body
        let body = self.lower_expr(&fn_def.body.node);

        // Extract loop invariants from body
        let loop_invariants = self.extract_loop_invariants(&fn_def.body.node);

        self.current_function = None;

        Some(CirFunction {
            name: fn_def.name.node.clone(),
            type_params,
            params,
            ret_ty,
            ret_name,
            preconditions,
            postconditions,
            loop_invariants,
            effects,
            body,
        })
    }

    fn lower_extern_fn(&self, extern_fn: &ast::ExternFn) -> CirExternFn {
        let module = extern_fn
            .link_name
            .clone()
            .unwrap_or_else(|| "env".to_string());

        let params: Vec<CirType> = extern_fn
            .params
            .iter()
            .map(|p| self.lower_type(&p.ty.node))
            .collect();

        let ret_ty = self.lower_type(&extern_fn.ret_ty.node);

        // External functions are assumed to have side effects
        let effects = EffectSet::impure();

        CirExternFn {
            module,
            name: extern_fn.name.node.clone(),
            params,
            ret_ty,
            effects,
        }
    }

    /// Convert an expression to a proposition (for contract conditions)
    fn expr_to_proposition(&self, expr: &Expr) -> Option<Proposition> {
        match expr {
            // Boolean literals
            Expr::BoolLit(true) => Some(Proposition::True),
            Expr::BoolLit(false) => Some(Proposition::False),

            // Comparison operators
            Expr::Binary { op, left, right } => {
                let cir_lhs = self.lower_expr(&left.node);
                let cir_rhs = self.lower_expr(&right.node);

                match op {
                    AstBinOp::Lt => Some(Proposition::compare(cir_lhs, CompareOp::Lt, cir_rhs)),
                    AstBinOp::Le => Some(Proposition::compare(cir_lhs, CompareOp::Le, cir_rhs)),
                    AstBinOp::Gt => Some(Proposition::compare(cir_lhs, CompareOp::Gt, cir_rhs)),
                    AstBinOp::Ge => Some(Proposition::compare(cir_lhs, CompareOp::Ge, cir_rhs)),
                    AstBinOp::Eq => Some(Proposition::compare(cir_lhs, CompareOp::Eq, cir_rhs)),
                    AstBinOp::Ne => Some(Proposition::compare(cir_lhs, CompareOp::Ne, cir_rhs)),

                    // Logical operators
                    AstBinOp::And => {
                        let lhs_prop = self.expr_to_proposition(&left.node)?;
                        let rhs_prop = self.expr_to_proposition(&right.node)?;
                        Some(Proposition::And(vec![lhs_prop, rhs_prop]))
                    }
                    AstBinOp::Or => {
                        let lhs_prop = self.expr_to_proposition(&left.node)?;
                        let rhs_prop = self.expr_to_proposition(&right.node)?;
                        Some(Proposition::Or(vec![lhs_prop, rhs_prop]))
                    }

                    // Implication
                    AstBinOp::Implies => {
                        let lhs_prop = self.expr_to_proposition(&left.node)?;
                        let rhs_prop = self.expr_to_proposition(&right.node)?;
                        Some(Proposition::Implies(Box::new(lhs_prop), Box::new(rhs_prop)))
                    }

                    // Arithmetic operations are not propositions by themselves
                    _ => None,
                }
            }

            // Unary NOT
            Expr::Unary { op: AstUnOp::Not, expr: operand } => {
                let inner = self.expr_to_proposition(&operand.node)?;
                Some(Proposition::not(inner))
            }

            // Function calls (predicates)
            Expr::Call { func, args } => {
                let cir_args: Vec<CirExpr> = args
                    .iter()
                    .map(|a| self.lower_expr(&a.node))
                    .collect();

                Some(Proposition::Predicate {
                    name: func.clone(),
                    args: cir_args,
                })
            }

            // Variable (treat as boolean variable)
            Expr::Var(name) => {
                Some(Proposition::Compare {
                    lhs: Box::new(CirExpr::Var(name.clone())),
                    op: CompareOp::Ne,
                    rhs: Box::new(CirExpr::IntLit(0)),
                })
            }

            // Forall quantifier
            Expr::Forall { var, ty, body } => {
                let prop = self.expr_to_proposition(&body.node)?;
                Some(Proposition::Forall {
                    var: var.node.clone(),
                    ty: self.lower_type(&ty.node),
                    body: Box::new(prop),
                })
            }

            // Exists quantifier
            Expr::Exists { var, ty, body } => {
                let prop = self.expr_to_proposition(&body.node)?;
                Some(Proposition::Exists {
                    var: var.node.clone(),
                    ty: self.lower_type(&ty.node),
                    body: Box::new(prop),
                })
            }

            _ => None,
        }
    }

    /// Check if expression references the return value
    fn references_return_value(&self, expr: &Expr, ret_name: &str) -> bool {
        match expr {
            Expr::Var(name) => name == ret_name || name == "ret",
            Expr::Ret => true,
            Expr::Binary { left, right, .. } => {
                self.references_return_value(&left.node, ret_name)
                    || self.references_return_value(&right.node, ret_name)
            }
            Expr::Unary { expr: operand, .. } => self.references_return_value(&operand.node, ret_name),
            Expr::Call { args, .. } => args
                .iter()
                .any(|a| self.references_return_value(&a.node, ret_name)),
            Expr::MethodCall { receiver, args, .. } => {
                self.references_return_value(&receiver.node, ret_name)
                    || args.iter().any(|a| self.references_return_value(&a.node, ret_name))
            }
            Expr::FieldAccess { expr, .. } => self.references_return_value(&expr.node, ret_name),
            Expr::Index { expr, index } => {
                self.references_return_value(&expr.node, ret_name)
                    || self.references_return_value(&index.node, ret_name)
            }
            _ => false,
        }
    }

    /// Analyze function attributes to determine effects
    fn analyze_effects(&self, fn_def: &FnDef) -> EffectSet {
        let mut effects = EffectSet::impure();

        for attr in &fn_def.attributes {
            match attr {
                ast::Attribute::Simple { name, .. } => {
                    match name.node.as_str() {
                        "pure" => {
                            effects.is_pure = true;
                            effects.reads = false;
                            effects.writes = false;
                            effects.io = false;
                        }
                        "const" => {
                            effects.is_pure = true;
                            effects.is_const = true;
                            effects.reads = false;
                            effects.writes = false;
                            effects.io = false;
                        }
                        _ => {}
                    }
                }
                ast::Attribute::WithArgs { name, .. } => {
                    // Handle @inline(always) etc.
                    if name.node == "pure" {
                        effects.is_pure = true;
                    }
                }
                _ => {}
            }
        }

        effects
    }

    /// Extract loop invariants from an expression
    fn extract_loop_invariants(&mut self, expr: &Expr) -> Vec<LoopInvariant> {
        let mut invariants = Vec::new();
        self.collect_loop_invariants(expr, &mut invariants);
        invariants
    }

    fn collect_loop_invariants(&mut self, expr: &Expr, invariants: &mut Vec<LoopInvariant>) {
        match expr {
            Expr::While { cond, body, invariant } => {
                let loop_id = self.loop_counter;
                self.loop_counter += 1;

                // If there's an explicit invariant annotation
                if let Some(inv_expr) = invariant {
                    if let Some(prop) = self.expr_to_proposition(&inv_expr.node) {
                        invariants.push(LoopInvariant {
                            loop_id,
                            invariant: prop,
                        });
                    }
                }

                // The loop condition is also a (weak) invariant - it's true at loop entry
                if let Some(cond_prop) = self.expr_to_proposition(&cond.node) {
                    invariants.push(LoopInvariant {
                        loop_id,
                        invariant: cond_prop,
                    });
                }

                // Recurse into body
                self.collect_loop_invariants(&body.node, invariants);
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_loop_invariants(&stmt.node, invariants);
                }
            }
            Expr::If { then_branch, else_branch, .. } => {
                self.collect_loop_invariants(&then_branch.node, invariants);
                self.collect_loop_invariants(&else_branch.node, invariants);
            }
            Expr::Let { body, .. } => {
                self.collect_loop_invariants(&body.node, invariants);
            }
            // v0.60.21: Uninitialized let binding
            Expr::LetUninit { body, .. } => {
                self.collect_loop_invariants(&body.node, invariants);
            }
            Expr::Loop { body } => {
                let loop_id = self.loop_counter;
                self.loop_counter += 1;
                // Infinite loops don't have conditions
                // Recurse into body
                self.collect_loop_invariants(&body.node, invariants);
                let _ = loop_id; // suppress unused warning
            }
            Expr::For { body, .. } => {
                self.collect_loop_invariants(&body.node, invariants);
            }
            _ => {}
        }
    }

    /// Lower an expression to CIR
    fn lower_expr(&self, expr: &Expr) -> CirExpr {
        match expr {
            Expr::IntLit(n) => CirExpr::IntLit(*n),
            Expr::FloatLit(f) => CirExpr::FloatLit(f.to_bits()),
            Expr::BoolLit(b) => CirExpr::BoolLit(*b),
            Expr::StringLit(s) => CirExpr::StringLit(s.clone()),
            Expr::CharLit(c) => CirExpr::IntLit(*c as i64),
            Expr::Null => CirExpr::IntLit(0), // Null as 0
            Expr::Unit => CirExpr::Unit,

            Expr::Var(name) => CirExpr::Var(name.clone()),
            Expr::Ret => CirExpr::Var("ret".to_string()),
            Expr::It => CirExpr::Var("it".to_string()),

            Expr::Binary { op, left, right } => {
                let cir_lhs = self.lower_expr(&left.node);
                let cir_rhs = self.lower_expr(&right.node);
                let cir_op = self.lower_binop(*op);
                CirExpr::BinOp {
                    op: cir_op,
                    lhs: Box::new(cir_lhs),
                    rhs: Box::new(cir_rhs),
                }
            }

            Expr::Unary { op, expr: operand } => {
                let cir_operand = self.lower_expr(&operand.node);
                let cir_op = self.lower_unaryop(*op);
                CirExpr::UnaryOp {
                    op: cir_op,
                    operand: Box::new(cir_operand),
                }
            }

            Expr::Call { func, args } => {
                let cir_args: Vec<CirExpr> = args
                    .iter()
                    .map(|a| self.lower_expr(&a.node))
                    .collect();
                CirExpr::Call {
                    func: func.clone(),
                    args: cir_args,
                }
            }

            Expr::Index { expr: base, index } => {
                let cir_base = self.lower_expr(&base.node);
                let cir_index = self.lower_expr(&index.node);
                CirExpr::Index {
                    base: Box::new(cir_base),
                    index: Box::new(cir_index),
                }
            }

            Expr::FieldAccess { expr: base, field } => {
                let cir_base = self.lower_expr(&base.node);
                CirExpr::Field {
                    base: Box::new(cir_base),
                    field: field.node.clone(),
                }
            }

            Expr::TupleField { expr: base, index } => {
                let cir_base = self.lower_expr(&base.node);
                CirExpr::Field {
                    base: Box::new(cir_base),
                    field: index.to_string(),
                }
            }

            Expr::If { cond, then_branch, else_branch } => {
                let cir_cond = self.lower_expr(&cond.node);
                let cir_then = self.lower_expr(&then_branch.node);
                let cir_else = self.lower_expr(&else_branch.node);
                CirExpr::If {
                    cond: Box::new(cir_cond),
                    then_branch: Box::new(cir_then),
                    else_branch: Box::new(cir_else),
                }
            }

            Expr::While { cond, body, invariant } => {
                let cir_cond = self.lower_expr(&cond.node);
                let cir_body = self.lower_expr(&body.node);
                let cir_invariant = invariant
                    .as_ref()
                    .and_then(|inv| self.expr_to_proposition(&inv.node));
                CirExpr::While {
                    cond: Box::new(cir_cond),
                    body: Box::new(cir_body),
                    invariant: cir_invariant,
                }
            }

            Expr::Let { name, mutable, ty, value, body } => {
                let cir_ty = ty.as_ref()
                    .map(|t| self.lower_type(&t.node))
                    .unwrap_or(CirType::Unit);
                let cir_value = self.lower_expr(&value.node);
                let cir_body = self.lower_expr(&body.node);
                if *mutable {
                    CirExpr::LetMut {
                        name: name.clone(),
                        ty: cir_ty,
                        value: Box::new(cir_value),
                        body: Box::new(cir_body),
                    }
                } else {
                    CirExpr::Let {
                        name: name.clone(),
                        ty: cir_ty,
                        value: Box::new(cir_value),
                        body: Box::new(cir_body),
                    }
                }
            }

            // v0.60.21: Uninitialized let binding for stack arrays
            Expr::LetUninit { name, mutable, ty, body } => {
                let cir_ty = self.lower_type(&ty.node);
                let cir_body = self.lower_expr(&body.node);
                // Treat as a mutable binding with undefined value
                if *mutable {
                    CirExpr::LetMut {
                        name: name.clone(),
                        ty: cir_ty,
                        value: Box::new(CirExpr::Unit), // undefined
                        body: Box::new(cir_body),
                    }
                } else {
                    CirExpr::Let {
                        name: name.clone(),
                        ty: cir_ty,
                        value: Box::new(CirExpr::Unit), // undefined
                        body: Box::new(cir_body),
                    }
                }
            }

            Expr::Assign { name, value } => {
                let cir_value = self.lower_expr(&value.node);
                CirExpr::Assign {
                    target: name.clone(),
                    value: Box::new(cir_value),
                }
            }

            Expr::IndexAssign { array, index, value } => {
                // IndexAssign as special form
                let cir_array = self.lower_expr(&array.node);
                let cir_index = self.lower_expr(&index.node);
                let cir_value = self.lower_expr(&value.node);
                CirExpr::IndexAssign {
                    array: Box::new(cir_array),
                    index: Box::new(cir_index),
                    value: Box::new(cir_value),
                }
            }

            Expr::FieldAssign { object, field, value } => {
                let cir_object = self.lower_expr(&object.node);
                let cir_value = self.lower_expr(&value.node);
                CirExpr::FieldAssign {
                    object: Box::new(cir_object),
                    field: field.node.clone(),
                    value: Box::new(cir_value),
                }
            }

            // v0.60.21: Dereference assignment (CIR just passes through as deref + store)
            Expr::DerefAssign { ptr, value } => {
                let cir_ptr = self.lower_expr(&ptr.node);
                let cir_value = self.lower_expr(&value.node);
                CirExpr::DerefStore {
                    ptr: Box::new(cir_ptr),
                    value: Box::new(cir_value),
                }
            }

            Expr::Block(stmts) => {
                let cir_stmts: Vec<CirExpr> = stmts
                    .iter()
                    .map(|s| self.lower_expr(&s.node))
                    .collect();
                if cir_stmts.is_empty() {
                    CirExpr::Unit
                } else if cir_stmts.len() == 1 {
                    cir_stmts.into_iter().next().unwrap()
                } else {
                    CirExpr::Block(cir_stmts)
                }
            }

            Expr::StructInit { name, fields } => {
                let cir_fields: Vec<(String, CirExpr)> = fields
                    .iter()
                    .map(|(fname, fval)| (fname.node.clone(), self.lower_expr(&fval.node)))
                    .collect();
                CirExpr::Struct {
                    name: name.clone(),
                    fields: cir_fields,
                }
            }

            Expr::MethodCall { receiver, method, args } => {
                // Convert method call to regular call with receiver as first arg
                let mut cir_args = vec![self.lower_expr(&receiver.node)];
                cir_args.extend(args.iter().map(|a| self.lower_expr(&a.node)));
                CirExpr::Call {
                    func: method.clone(),
                    args: cir_args,
                }
            }

            Expr::Return { value } => {
                // Return is lowered as the return value
                value.as_ref()
                    .map(|v| self.lower_expr(&v.node))
                    .unwrap_or(CirExpr::Unit)
            }

            Expr::Match { expr: scrutinee, arms } => {
                // Lower match as the scrutinee for now
                // Proper match lowering would need pattern matching
                let scrutinee_expr = self.lower_expr(&scrutinee.node);

                if arms.is_empty() {
                    return CirExpr::Unit;
                }

                // For now, just return the scrutinee - proper match lowering TODO
                scrutinee_expr
            }

            Expr::ArrayLit(elems) => {
                let cir_elems: Vec<CirExpr> = elems
                    .iter()
                    .map(|e| self.lower_expr(&e.node))
                    .collect();
                CirExpr::Array(cir_elems)
            }

            // v0.60.22: Array repeat [val; N]
            Expr::ArrayRepeat { value, count } => {
                let val_expr = self.lower_expr(&value.node);
                let cir_elems: Vec<CirExpr> = (0..*count)
                    .map(|_| val_expr.clone())
                    .collect();
                CirExpr::Array(cir_elems)
            }

            Expr::Tuple(elems) => {
                let cir_elems: Vec<CirExpr> = elems
                    .iter()
                    .map(|e| self.lower_expr(&e.node))
                    .collect();
                CirExpr::Tuple(cir_elems)
            }

            Expr::Ref(inner) => {
                let cir_inner = self.lower_expr(&inner.node);
                CirExpr::Ref(Box::new(cir_inner))
            }

            Expr::RefMut(inner) => {
                let cir_inner = self.lower_expr(&inner.node);
                CirExpr::RefMut(Box::new(cir_inner))
            }

            Expr::Deref(inner) => {
                let cir_inner = self.lower_expr(&inner.node);
                CirExpr::Deref(Box::new(cir_inner))
            }

            Expr::Loop { body } => {
                let cir_body = self.lower_expr(&body.node);
                CirExpr::Loop {
                    body: Box::new(cir_body),
                }
            }

            Expr::For { var, iter, body } => {
                let cir_iter = self.lower_expr(&iter.node);
                let cir_body = self.lower_expr(&body.node);
                CirExpr::For {
                    var: var.clone(),
                    iter: Box::new(cir_iter),
                    body: Box::new(cir_body),
                }
            }

            Expr::Break { value } => {
                let cir_value = value.as_ref()
                    .map(|v| self.lower_expr(&v.node))
                    .unwrap_or(CirExpr::Unit);
                CirExpr::Break(Box::new(cir_value))
            }

            Expr::Continue => CirExpr::Continue,

            Expr::Range { start, end, kind } => {
                let cir_start = self.lower_expr(&start.node);
                let cir_end = self.lower_expr(&end.node);
                let inclusive = matches!(kind, ast::RangeKind::Inclusive);
                CirExpr::Range {
                    start: Box::new(cir_start),
                    end: Box::new(cir_end),
                    inclusive,
                }
            }

            Expr::EnumVariant { enum_name, variant, args } => {
                let cir_args: Vec<CirExpr> = args
                    .iter()
                    .map(|a| self.lower_expr(&a.node))
                    .collect();
                CirExpr::EnumVariant {
                    enum_name: enum_name.clone(),
                    variant: variant.clone(),
                    args: cir_args,
                }
            }

            Expr::StateRef { expr: inner, state } => {
                let cir_inner = self.lower_expr(&inner.node);
                let is_pre = matches!(state, ast::StateKind::Pre);
                CirExpr::StateRef {
                    expr: Box::new(cir_inner),
                    is_pre,
                }
            }

            Expr::Closure { params, body, .. } => {
                let param_names: Vec<String> = params
                    .iter()
                    .map(|p| p.name.node.clone())
                    .collect();
                let cir_body = self.lower_expr(&body.node);
                CirExpr::Closure {
                    params: param_names,
                    body: Box::new(cir_body),
                }
            }

            Expr::Cast { expr: inner, ty } => {
                let cir_inner = self.lower_expr(&inner.node);
                let cir_ty = self.lower_type(&ty.node);
                CirExpr::Cast {
                    expr: Box::new(cir_inner),
                    ty: cir_ty,
                }
            }

            Expr::Sizeof { ty } => {
                let cir_ty = self.lower_type(&ty.node);
                CirExpr::Sizeof(cir_ty)
            }

            // v0.70: Spawn expression - lower the body
            Expr::Spawn { body } => {
                let cir_body = self.lower_expr(&body.node);
                // Represent spawn as a call (for CIR purposes)
                CirExpr::Call {
                    func: "__spawn".to_string(),
                    args: vec![cir_body],
                }
            }

            // v0.72: Atomic creation expression
            Expr::AtomicNew { value } => {
                let val = self.lower_expr(&value.node);
                CirExpr::Call {
                    func: "__atomic_new".to_string(),
                    args: vec![val],
                }
            }

            // v0.71: Mutex creation expression
            Expr::MutexNew { value } => {
                let val = self.lower_expr(&value.node);
                CirExpr::Call {
                    func: "__mutex_new".to_string(),
                    args: vec![val],
                }
            }

            // v0.73: Channel creation expression
            Expr::ChannelNew { capacity, .. } => {
                let cap = self.lower_expr(&capacity.node);
                CirExpr::Call {
                    func: "__channel_new".to_string(),
                    args: vec![cap],
                }
            }

            // v0.74: RwLock, Barrier, Condvar creation expressions
            Expr::RwLockNew { value } => {
                let val = self.lower_expr(&value.node);
                CirExpr::Call {
                    func: "__rwlock_new".to_string(),
                    args: vec![val],
                }
            }
            Expr::BarrierNew { count } => {
                let cnt = self.lower_expr(&count.node);
                CirExpr::Call {
                    func: "__barrier_new".to_string(),
                    args: vec![cnt],
                }
            }
            Expr::CondvarNew => {
                CirExpr::Call {
                    func: "__condvar_new".to_string(),
                    args: vec![],
                }
            }

            // v0.75: Await expression - lower as a call to __await
            Expr::Await { future } => {
                let f = self.lower_expr(&future.node);
                CirExpr::Call {
                    func: "__await".to_string(),
                    args: vec![f],
                }
            }

            Expr::Forall { var, ty, body } => {
                // Forall as boolean expression
                let cir_body = self.lower_expr(&body.node);
                CirExpr::Forall {
                    var: var.node.clone(),
                    ty: self.lower_type(&ty.node),
                    body: Box::new(cir_body),
                }
            }

            Expr::Exists { var, ty, body } => {
                // Exists as boolean expression
                let cir_body = self.lower_expr(&body.node);
                CirExpr::Exists {
                    var: var.node.clone(),
                    ty: self.lower_type(&ty.node),
                    body: Box::new(cir_body),
                }
            }

            Expr::Todo { message } => {
                CirExpr::Todo(message.clone())
            }
        }
    }

    fn lower_binop(&self, op: AstBinOp) -> BinOp {
        match op {
            AstBinOp::Add => BinOp::Add,
            AstBinOp::Sub => BinOp::Sub,
            AstBinOp::Mul => BinOp::Mul,
            AstBinOp::Div => BinOp::Div,
            AstBinOp::Mod => BinOp::Mod,
            AstBinOp::AddWrap => BinOp::AddWrap,
            AstBinOp::SubWrap => BinOp::SubWrap,
            AstBinOp::MulWrap => BinOp::MulWrap,
            AstBinOp::AddChecked => BinOp::AddChecked,
            AstBinOp::SubChecked => BinOp::SubChecked,
            AstBinOp::MulChecked => BinOp::MulChecked,
            AstBinOp::AddSat => BinOp::AddSat,
            AstBinOp::SubSat => BinOp::SubSat,
            AstBinOp::MulSat => BinOp::MulSat,
            AstBinOp::Lt => BinOp::Lt,
            AstBinOp::Le => BinOp::Le,
            AstBinOp::Gt => BinOp::Gt,
            AstBinOp::Ge => BinOp::Ge,
            AstBinOp::Eq => BinOp::Eq,
            AstBinOp::Ne => BinOp::Ne,
            AstBinOp::And => BinOp::And,
            AstBinOp::Or => BinOp::Or,
            AstBinOp::Band => BinOp::BitAnd,
            AstBinOp::Bor => BinOp::BitOr,
            AstBinOp::Bxor => BinOp::BitXor,
            AstBinOp::Shl => BinOp::Shl,
            AstBinOp::Shr => BinOp::Shr,
            AstBinOp::Implies => BinOp::Implies,
        }
    }

    fn lower_unaryop(&self, op: AstUnOp) -> UnaryOp {
        match op {
            AstUnOp::Neg => UnaryOp::Neg,
            AstUnOp::Not => UnaryOp::Not,
            AstUnOp::Bnot => UnaryOp::BitNot,
        }
    }

    fn lower_type(&self, ty: &Type) -> CirType {
        match ty {
            Type::Unit => CirType::Unit,
            Type::Bool => CirType::Bool,
            Type::I32 => CirType::I32,
            Type::I64 => CirType::I64,
            Type::U32 => CirType::U32,
            Type::U64 => CirType::U64,
            Type::F64 => CirType::F64,
            Type::Char => CirType::Char,
            Type::String => CirType::String,
            Type::Named(name) => CirType::Struct(name.clone()),
            Type::TypeVar(name) => CirType::TypeParam(name.clone()),
            Type::Array(elem_ty, size) => {
                let cir_elem = self.lower_type(elem_ty);
                CirType::Array(Box::new(cir_elem), *size)
            }
            Type::Ref(inner) => {
                let cir_inner = self.lower_type(inner);
                CirType::Ref(Box::new(cir_inner))
            }
            Type::RefMut(inner) => {
                let cir_inner = self.lower_type(inner);
                CirType::RefMut(Box::new(cir_inner))
            }
            Type::Ptr(inner) => {
                let cir_inner = self.lower_type(inner);
                CirType::Ptr(Box::new(cir_inner))
            }
            // v0.70: Thread type - represented as i64 handle
            Type::Thread(_) => CirType::I64,
            // v0.71: Mutex type - represented as i64 handle
            Type::Mutex(_) => CirType::I64,
            // v0.72: Arc and Atomic types - represented as i64 handle
            Type::Arc(_) => CirType::I64,
            Type::Atomic(_) => CirType::I64,
            // v0.73: Sender and Receiver types - represented as i64 handle
            Type::Sender(_) => CirType::I64,
            Type::Receiver(_) => CirType::I64,
            // v0.74: RwLock, Barrier, Condvar - represented as i64 handle
            Type::RwLock(_) => CirType::I64,
            Type::Barrier => CirType::I64,
            Type::Condvar => CirType::I64,
            // v0.75: Future type - use i64 for handle
            Type::Future(_) => CirType::I64,
            Type::Nullable(inner) => {
                let cir_inner = self.lower_type(inner);
                CirType::Option(Box::new(cir_inner))
            }
            Type::Range(elem_ty) => {
                let cir_elem = self.lower_type(elem_ty);
                CirType::Range(Box::new(cir_elem))
            }
            Type::Generic { name, type_args } => {
                let cir_args: Vec<CirType> = type_args
                    .iter()
                    .map(|a| self.lower_type(a))
                    .collect();
                CirType::Generic(name.clone(), cir_args)
            }
            Type::Fn { params, ret } => {
                let cir_params: Vec<CirType> = params
                    .iter()
                    .map(|p| self.lower_type(p))
                    .collect();
                let cir_ret = self.lower_type(ret);
                CirType::Fn {
                    params: cir_params,
                    ret: Box::new(cir_ret),
                }
            }
            Type::Tuple(elems) => {
                let cir_elems: Vec<CirType> = elems
                    .iter()
                    .map(|e| self.lower_type(e))
                    .collect();
                CirType::Tuple(cir_elems)
            }
            Type::Struct { name, .. } => CirType::Struct(name.clone()),
            Type::Enum { name, .. } => CirType::Enum(name.clone()),
            Type::Never => CirType::Never,
            Type::Refined { base, constraints: _ } => {
                // For now, strip the refinement
                self.lower_type(base)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_empty_program() {
        let program = Program {
            header: None,
            items: vec![],
        };
        let cir = lower_to_cir(&program);
        assert!(cir.functions.is_empty());
    }
}
