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

        // v0.94: Extract @invariant attributes as propositions
        let mut invariants = Vec::new();
        for attr in &struct_def.attributes {
            if let ast::Attribute::WithArgs { name, args, .. } = attr
                && name.node == "invariant"
            {
                invariants.extend(args.iter().filter_map(|arg| self.expr_to_proposition(&arg.node)));
            }
        }

        CirStruct {
            name: struct_def.name.node.clone(),
            fields,
            invariants,
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
        if let Some(pre) = &fn_def.pre
            && let Some(prop) = self.expr_to_proposition(&pre.node) {
                preconditions.push(NamedProposition {
                    name: None,
                    proposition: prop,
                });
            }

        if let Some(post) = &fn_def.post
            && let Some(prop) = self.expr_to_proposition(&post.node) {
                postconditions.push(NamedProposition {
                    name: None,
                    proposition: prop,
                });
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
                if let Some(inv_expr) = invariant
                    && let Some(prop) = self.expr_to_proposition(&inv_expr.node) {
                        invariants.push(LoopInvariant {
                            loop_id,
                            invariant: prop,
                        });
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
                let scrutinee_expr = self.lower_expr(&scrutinee.node);

                if arms.is_empty() {
                    return CirExpr::Unit;
                }

                // Lower match to nested If chains with a shared scrutinee binding
                let scrutinee_name = "__match_scrutinee".to_string();
                let body = self.lower_match_arms(&scrutinee_name, arms);
                CirExpr::Let {
                    name: scrutinee_name,
                    ty: CirType::Infer,
                    value: Box::new(scrutinee_expr),
                    body: Box::new(body),
                }
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

            // v0.82: Select expression - not supported in CIR (use native compilation)
            Expr::Select { .. } => {
                CirExpr::Call {
                    func: "__select_unsupported".to_string(),
                    args: vec![],
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

    /// Lower match arms to nested If/Let chains
    fn lower_match_arms(&self, scrutinee_name: &str, arms: &[ast::MatchArm]) -> CirExpr {
        if arms.is_empty() {
            return CirExpr::Unit;
        }

        let arm = &arms[0];
        let rest = &arms[1..];

        // Check for wildcard or variable (terminal patterns)
        match &arm.pattern.node {
            ast::Pattern::Wildcard => {
                // Wildcard matches everything - just return the body
                // Apply guard if present
                let body = self.lower_expr(&arm.body.node);
                if let Some(guard) = &arm.guard {
                    let guard_expr = self.lower_expr(&guard.node);
                    let else_branch = self.lower_match_arms(scrutinee_name, rest);
                    CirExpr::If {
                        cond: Box::new(guard_expr),
                        then_branch: Box::new(body),
                        else_branch: Box::new(else_branch),
                    }
                } else {
                    body
                }
            }
            ast::Pattern::Var(var_name) => {
                // Variable pattern: bind scrutinee to var_name, then evaluate body
                let body = self.lower_expr(&arm.body.node);
                let inner = if let Some(guard) = &arm.guard {
                    let guard_expr = self.lower_expr(&guard.node);
                    let else_branch = self.lower_match_arms(scrutinee_name, rest);
                    CirExpr::If {
                        cond: Box::new(guard_expr),
                        then_branch: Box::new(body),
                        else_branch: Box::new(else_branch),
                    }
                } else {
                    body
                };
                CirExpr::Let {
                    name: var_name.clone(),
                    ty: CirType::Infer,
                    value: Box::new(CirExpr::Var(scrutinee_name.to_string())),
                    body: Box::new(inner),
                }
            }
            _ => {
                // Generate condition from pattern
                let cond = self.pattern_to_condition(scrutinee_name, &arm.pattern.node);
                let body = self.lower_expr(&arm.body.node);

                // Wrap body with pattern bindings
                let body_with_bindings = self.wrap_with_pattern_bindings(
                    scrutinee_name, &arm.pattern.node, body
                );

                // Apply guard if present
                let full_cond = if let Some(guard) = &arm.guard {
                    let guard_expr = self.lower_expr(&guard.node);
                    CirExpr::BinOp {
                        op: BinOp::And,
                        lhs: Box::new(cond),
                        rhs: Box::new(guard_expr),
                    }
                } else {
                    cond
                };

                let else_branch = self.lower_match_arms(scrutinee_name, rest);

                CirExpr::If {
                    cond: Box::new(full_cond),
                    then_branch: Box::new(body_with_bindings),
                    else_branch: Box::new(else_branch),
                }
            }
        }
    }

    /// Convert a pattern to a CIR condition expression
    fn pattern_to_condition(&self, scrutinee: &str, pattern: &ast::Pattern) -> CirExpr {
        let scrut = CirExpr::Var(scrutinee.to_string());
        match pattern {
            ast::Pattern::Wildcard => CirExpr::BoolLit(true),
            ast::Pattern::Var(_) => CirExpr::BoolLit(true),
            ast::Pattern::Literal(lit) => {
                let lit_expr = self.literal_pattern_to_expr(lit);
                CirExpr::BinOp {
                    op: BinOp::Eq,
                    lhs: Box::new(scrut),
                    rhs: Box::new(lit_expr),
                }
            }
            ast::Pattern::EnumVariant { enum_name, variant, .. } => {
                // Check variant tag equality
                CirExpr::Call {
                    func: format!("__is_{}_{}", enum_name, variant),
                    args: vec![scrut],
                }
            }
            ast::Pattern::Range { start, end, inclusive } => {
                let start_expr = self.literal_pattern_to_expr(start);
                let end_expr = self.literal_pattern_to_expr(end);
                let ge = CirExpr::BinOp {
                    op: BinOp::Ge,
                    lhs: Box::new(scrut.clone()),
                    rhs: Box::new(start_expr),
                };
                let upper = if *inclusive {
                    CirExpr::BinOp {
                        op: BinOp::Le,
                        lhs: Box::new(scrut),
                        rhs: Box::new(end_expr),
                    }
                } else {
                    CirExpr::BinOp {
                        op: BinOp::Lt,
                        lhs: Box::new(scrut),
                        rhs: Box::new(end_expr),
                    }
                };
                CirExpr::BinOp {
                    op: BinOp::And,
                    lhs: Box::new(ge),
                    rhs: Box::new(upper),
                }
            }
            ast::Pattern::Or(alternatives) => {
                // Or-pattern: any of the alternatives matches
                let conditions: Vec<CirExpr> = alternatives
                    .iter()
                    .map(|alt| self.pattern_to_condition(scrutinee, &alt.node))
                    .collect();
                conditions.into_iter().reduce(|acc, c| {
                    CirExpr::BinOp {
                        op: BinOp::Or,
                        lhs: Box::new(acc),
                        rhs: Box::new(c),
                    }
                }).unwrap_or(CirExpr::BoolLit(false))
            }
            ast::Pattern::Binding { pattern: inner, .. } => {
                // The condition comes from the inner pattern
                self.pattern_to_condition(scrutinee, &inner.node)
            }
            ast::Pattern::Tuple(pats) => {
                // All sub-patterns must match their respective fields
                let conditions: Vec<CirExpr> = pats.iter().enumerate().map(|(i, p)| {
                    let field_scrut = format!("{}_{}", scrutinee, i);
                    self.pattern_to_condition(&field_scrut, &p.node)
                }).collect();
                conditions.into_iter().reduce(|acc, c| {
                    CirExpr::BinOp {
                        op: BinOp::And,
                        lhs: Box::new(acc),
                        rhs: Box::new(c),
                    }
                }).unwrap_or(CirExpr::BoolLit(true))
            }
            ast::Pattern::Struct { .. } => {
                // Struct patterns always match the right type (type checker ensures this)
                CirExpr::BoolLit(true)
            }
            ast::Pattern::Array(_) | ast::Pattern::ArrayRest { .. } => {
                // Array patterns: type-checked for size already
                CirExpr::BoolLit(true)
            }
        }
    }

    /// Convert a literal pattern to a CIR expression
    fn literal_pattern_to_expr(&self, lit: &ast::LiteralPattern) -> CirExpr {
        match lit {
            ast::LiteralPattern::Int(n) => CirExpr::IntLit(*n),
            ast::LiteralPattern::Float(f) => CirExpr::FloatLit(f.to_bits()),
            ast::LiteralPattern::Bool(b) => CirExpr::BoolLit(*b),
            ast::LiteralPattern::String(s) => CirExpr::StringLit(s.clone()),
        }
    }

    /// Wrap an expression with Let bindings for pattern variables
    fn wrap_with_pattern_bindings(&self, scrutinee: &str, pattern: &ast::Pattern, body: CirExpr) -> CirExpr {
        match pattern {
            ast::Pattern::Wildcard | ast::Pattern::Literal(_) | ast::Pattern::Range { .. } => body,
            ast::Pattern::Var(name) => {
                CirExpr::Let {
                    name: name.clone(),
                    ty: CirType::Infer,
                    value: Box::new(CirExpr::Var(scrutinee.to_string())),
                    body: Box::new(body),
                }
            }
            ast::Pattern::Binding { name, pattern: inner } => {
                let inner_body = self.wrap_with_pattern_bindings(scrutinee, &inner.node, body);
                CirExpr::Let {
                    name: name.clone(),
                    ty: CirType::Infer,
                    value: Box::new(CirExpr::Var(scrutinee.to_string())),
                    body: Box::new(inner_body),
                }
            }
            ast::Pattern::EnumVariant { bindings, .. } => {
                // Bind each destructured field
                let mut result = body;
                for (i, binding) in bindings.iter().enumerate().rev() {
                    let field_expr = CirExpr::Call {
                        func: "__enum_field".to_string(),
                        args: vec![CirExpr::Var(scrutinee.to_string()), CirExpr::IntLit(i as i64)],
                    };
                    result = self.wrap_with_pattern_bindings(
                        &format!("__enum_field_{}_{}", scrutinee, i),
                        &binding.node,
                        result,
                    );
                    if let ast::Pattern::Var(var_name) = &binding.node {
                        result = CirExpr::Let {
                            name: var_name.clone(),
                            ty: CirType::Infer,
                            value: Box::new(field_expr),
                            body: Box::new(result),
                        };
                    }
                }
                result
            }
            ast::Pattern::Struct { fields, .. } => {
                let mut result = body;
                for (field_name, field_pat) in fields.iter().rev() {
                    let field_expr = CirExpr::Field {
                        base: Box::new(CirExpr::Var(scrutinee.to_string())),
                        field: field_name.node.clone(),
                    };
                    if let ast::Pattern::Var(var_name) = &field_pat.node {
                        result = CirExpr::Let {
                            name: var_name.clone(),
                            ty: CirType::Infer,
                            value: Box::new(field_expr),
                            body: Box::new(result),
                        };
                    }
                }
                result
            }
            ast::Pattern::Or(_) => body, // Or-patterns don't introduce bindings at CIR level
            ast::Pattern::Tuple(pats) => {
                let mut result = body;
                for (i, pat) in pats.iter().enumerate().rev() {
                    if let ast::Pattern::Var(var_name) = &pat.node {
                        let field_expr = CirExpr::Field {
                            base: Box::new(CirExpr::Var(scrutinee.to_string())),
                            field: i.to_string(),
                        };
                        result = CirExpr::Let {
                            name: var_name.clone(),
                            ty: CirType::Infer,
                            value: Box::new(field_expr),
                            body: Box::new(result),
                        };
                    }
                }
                result
            }
            ast::Pattern::Array(pats) => {
                let mut result = body;
                for (i, pat) in pats.iter().enumerate().rev() {
                    if let ast::Pattern::Var(var_name) = &pat.node {
                        let idx_expr = CirExpr::Index {
                            base: Box::new(CirExpr::Var(scrutinee.to_string())),
                            index: Box::new(CirExpr::IntLit(i as i64)),
                        };
                        result = CirExpr::Let {
                            name: var_name.clone(),
                            ty: CirType::Infer,
                            value: Box::new(idx_expr),
                            body: Box::new(result),
                        };
                    }
                }
                result
            }
            ast::Pattern::ArrayRest { prefix, suffix } => {
                let mut result = body;
                for (i, pat) in prefix.iter().enumerate().rev() {
                    if let ast::Pattern::Var(var_name) = &pat.node {
                        let idx_expr = CirExpr::Index {
                            base: Box::new(CirExpr::Var(scrutinee.to_string())),
                            index: Box::new(CirExpr::IntLit(i as i64)),
                        };
                        result = CirExpr::Let {
                            name: var_name.clone(),
                            ty: CirType::Infer,
                            value: Box::new(idx_expr),
                            body: Box::new(result),
                        };
                    }
                }
                // Suffix bindings use negative-offset-from-end semantics via Len
                for (i, pat) in suffix.iter().enumerate().rev() {
                    if let ast::Pattern::Var(var_name) = &pat.node {
                        let len_expr = CirExpr::Len(Box::new(CirExpr::Var(scrutinee.to_string())));
                        let offset = CirExpr::IntLit((suffix.len() - i) as i64);
                        let idx = CirExpr::BinOp {
                            op: BinOp::Sub,
                            lhs: Box::new(len_expr),
                            rhs: Box::new(offset),
                        };
                        let idx_expr = CirExpr::Index {
                            base: Box::new(CirExpr::Var(scrutinee.to_string())),
                            index: Box::new(idx),
                        };
                        result = CirExpr::Let {
                            name: var_name.clone(),
                            ty: CirType::Infer,
                            value: Box::new(idx_expr),
                            body: Box::new(result),
                        };
                    }
                }
                result
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
            // v0.83: AsyncFile - use i64 for file handle
            Type::AsyncFile => CirType::I64,
            // v0.83.1: AsyncSocket - use i64 for socket handle
            Type::AsyncSocket => CirType::I64,
            // v0.84: ThreadPool - use i64 for pool handle
            Type::ThreadPool => CirType::I64,
            // v0.85: Scope - use i64 for scope handle
            Type::Scope => CirType::I64,
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

    /// Helper: parse + type-check + lower to CIR
    fn source_to_cir(source: &str) -> CirProgram {
        let tokens = crate::lexer::tokenize(source).expect("tokenize failed");
        let ast = crate::parser::parse("test.bmb", source, tokens).expect("parse failed");
        let mut tc = crate::types::TypeChecker::new();
        tc.check_program(&ast).expect("type-check failed");
        lower_to_cir(&ast)
    }

    #[test]
    fn test_lower_empty_program() {
        let program = Program {
            header: None,
            items: vec![],
        };
        let cir = lower_to_cir(&program);
        assert!(cir.functions.is_empty());
    }

    #[test]
    fn test_lower_simple_function() {
        let cir = source_to_cir("fn add(a: i64, b: i64) -> i64 = a + b;");
        assert_eq!(cir.functions.len(), 1);
        let func = &cir.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].name, "a");
        assert_eq!(func.params[1].name, "b");
        assert_eq!(func.ret_ty, CirType::I64);
    }

    #[test]
    fn test_lower_function_with_precondition() {
        let cir = source_to_cir(
            "fn safe_div(a: i64, b: i64) -> i64
               pre b != 0
             = a / b;"
        );
        let func = &cir.functions[0];
        assert_eq!(func.name, "safe_div");
        assert!(!func.preconditions.is_empty(), "Should have preconditions");
    }

    #[test]
    fn test_lower_function_with_postcondition() {
        let cir = source_to_cir(
            "fn abs(x: i64) -> i64
               post ret >= 0
             = if x >= 0 { x } else { 0 - x };"
        );
        let func = &cir.functions[0];
        assert_eq!(func.name, "abs");
        assert!(!func.postconditions.is_empty(), "Should have postconditions");
    }

    #[test]
    fn test_lower_multiple_functions() {
        let cir = source_to_cir(
            "fn foo(x: i64) -> i64 = x;
             fn bar(y: i64) -> i64 = y + 1;"
        );
        assert_eq!(cir.functions.len(), 2);
        assert_eq!(cir.functions[0].name, "foo");
        assert_eq!(cir.functions[1].name, "bar");
    }

    #[test]
    fn test_lower_bool_return_type() {
        let cir = source_to_cir("fn is_zero(x: i64) -> bool = x == 0;");
        let func = &cir.functions[0];
        assert_eq!(func.ret_ty, CirType::Bool);
    }

    #[test]
    fn test_lower_struct_definition() {
        let cir = source_to_cir(
            "struct Point { x: i64, y: i64 }
             fn origin() -> Point = new Point { x: 0, y: 0 };"
        );
        assert!(cir.structs.contains_key("Point"), "Should contain Point struct");
        let point = &cir.structs["Point"];
        assert_eq!(point.fields.len(), 2);
    }

    #[test]
    fn test_lower_extern_fn() {
        let cir = source_to_cir(
            "extern fn my_func(x: i64) -> i64;"
        );
        assert_eq!(cir.extern_fns.len(), 1);
        assert_eq!(cir.extern_fns[0].name, "my_func");
    }

    #[test]
    fn test_lower_pre_and_post() {
        let cir = source_to_cir(
            "fn bounded(x: i64) -> i64
               pre x >= 0
               post ret >= 0
             = x;"
        );
        let func = &cir.functions[0];
        assert!(!func.preconditions.is_empty(), "Should have pre");
        assert!(!func.postconditions.is_empty(), "Should have post");
    }

    #[test]
    fn test_lower_pure_function_effects() {
        let cir = source_to_cir("@pure fn square(x: i64) -> i64 = x * x;");
        let func = &cir.functions[0];
        assert!(func.effects.is_pure, "Pure function should have pure effects");
    }

    // ---- Cycle 67: expr_to_proposition tests ----

    #[test]
    fn test_expr_to_proposition_bool_literals() {
        let lowerer = CirLowerer::new();
        assert_eq!(lowerer.expr_to_proposition(&Expr::BoolLit(true)), Some(Proposition::True));
        assert_eq!(lowerer.expr_to_proposition(&Expr::BoolLit(false)), Some(Proposition::False));
    }

    #[test]
    fn test_expr_to_proposition_comparison_operators() {
        let cir = source_to_cir("fn f(a: i64, b: i64) -> i64 pre a < b = a;");
        let func = &cir.functions[0];
        assert_eq!(func.preconditions.len(), 1);
        // The proposition should be a Compare with Lt
        match &func.preconditions[0].proposition {
            Proposition::Compare { op, .. } => assert_eq!(*op, CompareOp::Lt),
            other => panic!("Expected Compare, got {:?}", other),
        }
    }

    #[test]
    fn test_expr_to_proposition_logical_and() {
        let cir = source_to_cir("fn f(x: i64) -> i64 pre x > 0 && x < 100 = x;");
        let func = &cir.functions[0];
        match &func.preconditions[0].proposition {
            Proposition::And(parts) => assert_eq!(parts.len(), 2),
            other => panic!("Expected And, got {:?}", other),
        }
    }

    #[test]
    fn test_expr_to_proposition_logical_or() {
        let cir = source_to_cir("fn f(x: i64) -> i64 pre x == 0 || x == 1 = x;");
        let func = &cir.functions[0];
        match &func.preconditions[0].proposition {
            Proposition::Or(parts) => assert_eq!(parts.len(), 2),
            other => panic!("Expected Or, got {:?}", other),
        }
    }

    #[test]
    fn test_expr_to_proposition_not() {
        let cir = source_to_cir("fn f(x: i64) -> i64 pre !(x == 0) = x;");
        let func = &cir.functions[0];
        match &func.preconditions[0].proposition {
            Proposition::Not(_) => {}
            other => panic!("Expected Not, got {:?}", other),
        }
    }

    #[test]
    fn test_expr_to_proposition_var_as_boolean() {
        let lowerer = CirLowerer::new();
        let prop = lowerer.expr_to_proposition(&Expr::Var("flag".to_string()));
        // Variable treated as boolean: flag != 0
        match prop {
            Some(Proposition::Compare { op: CompareOp::Ne, .. }) => {}
            other => panic!("Expected Compare(Ne), got {:?}", other),
        }
    }

    #[test]
    fn test_expr_to_proposition_arithmetic_returns_none() {
        let lowerer = CirLowerer::new();
        use crate::ast::Spanned;
        let span = crate::ast::Span { start: 0, end: 1 };
        let expr = Expr::Binary {
            op: AstBinOp::Add,
            left: Box::new(Spanned { node: Expr::IntLit(1), span }),
            right: Box::new(Spanned { node: Expr::IntLit(2), span }),
        };
        assert!(lowerer.expr_to_proposition(&expr).is_none());
    }

    // ---- lower_expr edge cases ----

    #[test]
    fn test_lower_expr_char_lit() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::CharLit('A'));
        assert_eq!(result, CirExpr::IntLit(65));
    }

    #[test]
    fn test_lower_expr_null() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::Null);
        assert_eq!(result, CirExpr::IntLit(0));
    }

    #[test]
    fn test_lower_expr_unit() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::Unit);
        assert_eq!(result, CirExpr::Unit);
    }

    #[test]
    fn test_lower_expr_ret() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::Ret);
        assert_eq!(result, CirExpr::Var("ret".to_string()));
    }

    #[test]
    fn test_lower_expr_it() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::It);
        assert_eq!(result, CirExpr::Var("it".to_string()));
    }

    #[test]
    fn test_lower_expr_continue() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::Continue);
        assert_eq!(result, CirExpr::Continue);
    }

    #[test]
    fn test_lower_expr_todo_with_message() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::Todo { message: Some("later".to_string()) });
        assert_eq!(result, CirExpr::Todo(Some("later".to_string())));
    }

    // ---- lower_type tests ----

    #[test]
    fn test_lower_type_primitives() {
        let lowerer = CirLowerer::new();
        assert_eq!(lowerer.lower_type(&Type::Unit), CirType::Unit);
        assert_eq!(lowerer.lower_type(&Type::Bool), CirType::Bool);
        assert_eq!(lowerer.lower_type(&Type::I32), CirType::I32);
        assert_eq!(lowerer.lower_type(&Type::I64), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::U32), CirType::U32);
        assert_eq!(lowerer.lower_type(&Type::U64), CirType::U64);
        assert_eq!(lowerer.lower_type(&Type::F64), CirType::F64);
        assert_eq!(lowerer.lower_type(&Type::Char), CirType::Char);
        assert_eq!(lowerer.lower_type(&Type::String), CirType::String);
    }

    #[test]
    fn test_lower_type_array() {
        let lowerer = CirLowerer::new();
        let ty = Type::Array(Box::new(Type::I64), 10);
        let expected = CirType::Array(Box::new(CirType::I64), 10);
        assert_eq!(lowerer.lower_type(&ty), expected);
    }

    #[test]
    fn test_lower_type_ref_and_ptr() {
        let lowerer = CirLowerer::new();
        assert_eq!(
            lowerer.lower_type(&Type::Ref(Box::new(Type::I64))),
            CirType::Ref(Box::new(CirType::I64))
        );
        assert_eq!(
            lowerer.lower_type(&Type::RefMut(Box::new(Type::I64))),
            CirType::RefMut(Box::new(CirType::I64))
        );
        assert_eq!(
            lowerer.lower_type(&Type::Ptr(Box::new(Type::I64))),
            CirType::Ptr(Box::new(CirType::I64))
        );
    }

    #[test]
    fn test_lower_type_concurrency() {
        let lowerer = CirLowerer::new();
        // All concurrency types map to I64
        assert_eq!(lowerer.lower_type(&Type::Thread(Box::new(Type::Unit))), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::Mutex(Box::new(Type::I64))), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::Arc(Box::new(Type::I64))), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::Atomic(Box::new(Type::I64))), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::Future(Box::new(Type::I64))), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::Barrier), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::Condvar), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::AsyncFile), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::ThreadPool), CirType::I64);
        assert_eq!(lowerer.lower_type(&Type::Scope), CirType::I64);
    }

    #[test]
    fn test_lower_type_nullable() {
        let lowerer = CirLowerer::new();
        let ty = Type::Nullable(Box::new(Type::I64));
        assert_eq!(lowerer.lower_type(&ty), CirType::Option(Box::new(CirType::I64)));
    }

    #[test]
    fn test_lower_type_tuple() {
        let lowerer = CirLowerer::new();
        let ty = Type::Tuple(vec![Box::new(Type::I64), Box::new(Type::Bool)]);
        assert_eq!(lowerer.lower_type(&ty), CirType::Tuple(vec![CirType::I64, CirType::Bool]));
    }

    #[test]
    fn test_lower_type_never() {
        let lowerer = CirLowerer::new();
        assert_eq!(lowerer.lower_type(&Type::Never), CirType::Never);
    }

    // ---- references_return_value tests ----

    #[test]
    fn test_references_return_value_var() {
        let lowerer = CirLowerer::new();
        assert!(lowerer.references_return_value(&Expr::Var("ret".to_string()), "ret"));
        assert!(lowerer.references_return_value(&Expr::Var("result".to_string()), "result"));
        assert!(!lowerer.references_return_value(&Expr::Var("x".to_string()), "ret"));
    }

    #[test]
    fn test_references_return_value_ret_expr() {
        let lowerer = CirLowerer::new();
        assert!(lowerer.references_return_value(&Expr::Ret, "anything"));
    }

    #[test]
    fn test_references_return_value_nested() {
        let lowerer = CirLowerer::new();
        let span = crate::ast::Span { start: 0, end: 1 };
        let expr = Expr::Binary {
            op: AstBinOp::Gt,
            left: Box::new(crate::ast::Spanned { node: Expr::Var("ret".to_string()), span }),
            right: Box::new(crate::ast::Spanned { node: Expr::IntLit(0), span }),
        };
        assert!(lowerer.references_return_value(&expr, "ret"));
    }

    // ---- Integration: lowering with contracts ----

    #[test]
    fn test_lower_const_function_effects() {
        let cir = source_to_cir("@const fn zero() -> i64 = 0;");
        let func = &cir.functions[0];
        assert!(func.effects.is_const, "Should have const effect");
        assert!(func.effects.is_pure, "Const implies pure");
    }

    #[test]
    fn test_lower_function_ret_name_default() {
        let cir = source_to_cir("fn id(x: i64) -> i64 = x;");
        assert_eq!(cir.functions[0].ret_name, "ret");
    }

    #[test]
    fn test_lower_function_with_postcondition_uses_ret() {
        let cir = source_to_cir("fn abs(x: i64) -> i64 post ret >= 0 = if x >= 0 { x } else { 0 - x };");
        let func = &cir.functions[0];
        assert_eq!(func.ret_name, "ret");
        assert!(!func.postconditions.is_empty());
    }

    // ---- binop/unaryop lowering ----

    #[test]
    fn test_lower_all_binops() {
        let lowerer = CirLowerer::new();
        assert_eq!(lowerer.lower_binop(AstBinOp::Add), BinOp::Add);
        assert_eq!(lowerer.lower_binop(AstBinOp::Sub), BinOp::Sub);
        assert_eq!(lowerer.lower_binop(AstBinOp::Mul), BinOp::Mul);
        assert_eq!(lowerer.lower_binop(AstBinOp::Div), BinOp::Div);
        assert_eq!(lowerer.lower_binop(AstBinOp::Mod), BinOp::Mod);
        assert_eq!(lowerer.lower_binop(AstBinOp::Lt), BinOp::Lt);
        assert_eq!(lowerer.lower_binop(AstBinOp::Le), BinOp::Le);
        assert_eq!(lowerer.lower_binop(AstBinOp::Gt), BinOp::Gt);
        assert_eq!(lowerer.lower_binop(AstBinOp::Ge), BinOp::Ge);
        assert_eq!(lowerer.lower_binop(AstBinOp::Eq), BinOp::Eq);
        assert_eq!(lowerer.lower_binop(AstBinOp::Ne), BinOp::Ne);
        assert_eq!(lowerer.lower_binop(AstBinOp::And), BinOp::And);
        assert_eq!(lowerer.lower_binop(AstBinOp::Or), BinOp::Or);
        assert_eq!(lowerer.lower_binop(AstBinOp::Band), BinOp::BitAnd);
        assert_eq!(lowerer.lower_binop(AstBinOp::Bor), BinOp::BitOr);
        assert_eq!(lowerer.lower_binop(AstBinOp::Bxor), BinOp::BitXor);
        assert_eq!(lowerer.lower_binop(AstBinOp::Shl), BinOp::Shl);
        assert_eq!(lowerer.lower_binop(AstBinOp::Shr), BinOp::Shr);
        assert_eq!(lowerer.lower_binop(AstBinOp::Implies), BinOp::Implies);
    }

    #[test]
    fn test_lower_all_unaryops() {
        let lowerer = CirLowerer::new();
        assert_eq!(lowerer.lower_unaryop(AstUnOp::Neg), UnaryOp::Neg);
        assert_eq!(lowerer.lower_unaryop(AstUnOp::Not), UnaryOp::Not);
        assert_eq!(lowerer.lower_unaryop(AstUnOp::Bnot), UnaryOp::BitNot);
    }

    // ---- Integration: struct fields ----

    #[test]
    fn test_lower_struct_fields_types() {
        let cir = source_to_cir(
            "struct Pair { a: i64, b: bool }
             fn make() -> Pair = new Pair { a: 1, b: true };"
        );
        let pair = &cir.structs["Pair"];
        assert_eq!(pair.fields[0], ("a".to_string(), CirType::I64));
        assert_eq!(pair.fields[1], ("b".to_string(), CirType::Bool));
    }

    // ---- Block lowering ----

    #[test]
    fn test_lower_empty_block() {
        let lowerer = CirLowerer::new();
        let result = lowerer.lower_expr(&Expr::Block(vec![]));
        assert_eq!(result, CirExpr::Unit);
    }

    // ---- Cycle 92: CIR match lowering tests ----

    #[test]
    fn test_lower_match_literal_arms() {
        let cir = source_to_cir(
            "fn classify(x: i64) -> i64 = match x { 0 => 10, 1 => 20, _ => 30 };"
        );
        let func = &cir.functions[0];
        // Should be a Let binding for __match_scrutinee wrapping nested If
        match &func.body {
            CirExpr::Let { name, .. } => {
                assert_eq!(name, "__match_scrutinee");
            }
            other => panic!("Expected Let for match scrutinee, got {:?}", other),
        }
    }

    #[test]
    fn test_lower_match_wildcard_only() {
        let cir = source_to_cir(
            "fn identity(x: i64) -> i64 = match x { _ => x };"
        );
        let func = &cir.functions[0];
        // Wildcard-only match should still wrap in Let
        match &func.body {
            CirExpr::Let { name, .. } => {
                assert_eq!(name, "__match_scrutinee");
            }
            other => panic!("Expected Let for match scrutinee, got {:?}", other),
        }
    }

    #[test]
    fn test_lower_match_var_binding() {
        let cir = source_to_cir(
            "fn double(x: i64) -> i64 = match x { n => n + n };"
        );
        let func = &cir.functions[0];
        match &func.body {
            CirExpr::Let { name, body, .. } => {
                assert_eq!(name, "__match_scrutinee");
                // Body should be a Let binding for 'n'
                match body.as_ref() {
                    CirExpr::Let { name: inner_name, .. } => {
                        assert_eq!(inner_name, "n");
                    }
                    other => panic!("Expected inner Let for var binding, got {:?}", other),
                }
            }
            other => panic!("Expected Let, got {:?}", other),
        }
    }

    #[test]
    fn test_lower_match_bool_patterns() {
        let cir = source_to_cir(
            "fn to_int(b: bool) -> i64 = match b { true => 1, false => 0 };"
        );
        let func = &cir.functions[0];
        match &func.body {
            CirExpr::Let { name, .. } => {
                assert_eq!(name, "__match_scrutinee");
            }
            other => panic!("Expected Let for match scrutinee, got {:?}", other),
        }
    }

    #[test]
    fn test_lower_match_empty_arms() {
        // Empty match should produce Unit
        let lowerer = CirLowerer::new();
        let span = crate::ast::Span { start: 0, end: 1 };
        let expr = Expr::Match {
            expr: Box::new(crate::ast::Spanned { node: Expr::IntLit(0), span }),
            arms: vec![],
        };
        let result = lowerer.lower_expr(&expr);
        assert_eq!(result, CirExpr::Unit);
    }

    #[test]
    fn test_lower_match_multiple_literals() {
        let cir = source_to_cir(
            "fn grade(x: i64) -> i64 = match x { 1 => 100, 2 => 90, 3 => 80, _ => 0 };"
        );
        let func = &cir.functions[0];
        // Verify the structure has If chains
        match &func.body {
            CirExpr::Let { body, .. } => {
                match body.as_ref() {
                    CirExpr::If { .. } => {} // Should be nested Ifs
                    other => panic!("Expected If chain, got {:?}", other),
                }
            }
            other => panic!("Expected Let, got {:?}", other),
        }
    }

    // ---- Cycle 94: Struct invariant tests ----

    #[test]
    fn test_lower_struct_with_invariant() {
        let cir = source_to_cir(
            "@invariant(self.min <= self.max)
             struct Range { min: i64, max: i64 }
             fn make(a: i64, b: i64) -> Range = new Range { min: a, max: b };"
        );
        let range_struct = &cir.structs["Range"];
        assert!(!range_struct.invariants.is_empty(), "Should have extracted invariant");
    }

    #[test]
    fn test_lower_struct_no_invariant() {
        let cir = source_to_cir(
            "struct Point { x: i64, y: i64 }
             fn origin() -> Point = new Point { x: 0, y: 0 };"
        );
        let point = &cir.structs["Point"];
        assert!(point.invariants.is_empty(), "No invariant declared");
    }

    // =====================================================================
    // Match lowering tests
    // =====================================================================

    #[test]
    fn test_match_literal_pattern() {
        let cir = source_to_cir(
            "fn classify(x: i64) -> i64 = match x { 0 => 10, 1 => 20, _ => 30 };"
        );
        assert_eq!(cir.functions.len(), 1);
        let body = &cir.functions[0].body;
        // Match should lower to Let + nested Ifs
        assert!(matches!(body, CirExpr::Let { .. }),
            "Match should lower to Let binding for scrutinee");
    }

    #[test]
    fn test_match_wildcard_pattern() {
        let cir = source_to_cir(
            "fn always(x: i64) -> i64 = match x { _ => 42 };"
        );
        let body = &cir.functions[0].body;
        // Wildcard match should lower to Let + body (no If needed)
        assert!(matches!(body, CirExpr::Let { .. }),
            "Wildcard match should lower to Let");
    }

    #[test]
    fn test_match_variable_binding() {
        let cir = source_to_cir(
            "fn identity(x: i64) -> i64 = match x { y => y };"
        );
        let body = &cir.functions[0].body;
        // Variable pattern should create Let binding
        assert!(matches!(body, CirExpr::Let { .. }),
            "Variable match should create Let binding");
    }

    #[test]
    fn test_match_multiple_literals() {
        let cir = source_to_cir(
            "fn multi(x: i64) -> i64 = match x { 1 => 10, 2 => 20, 3 => 30, _ => 0 };"
        );
        let body = &cir.functions[0].body;
        // Should create nested If chain
        if let CirExpr::Let { body: inner, .. } = body {
            assert!(matches!(inner.as_ref(), CirExpr::If { .. }),
                "Multiple literal patterns should create nested Ifs");
        } else {
            panic!("Expected Let wrapping If chain");
        }
    }

    #[test]
    fn test_match_range_pattern() {
        let cir = source_to_cir(
            "fn in_range(x: i64) -> i64 = match x { 0..10 => 1, _ => 0 };"
        );
        let body = &cir.functions[0].body;
        assert!(matches!(body, CirExpr::Let { .. }),
            "Range pattern match should lower to Let + If");
    }

    #[test]
    fn test_match_guard() {
        let cir = source_to_cir(
            "fn guarded(x: i64) -> i64 = match x { y if y > 0 => y, _ => 0 };"
        );
        let body = &cir.functions[0].body;
        assert!(matches!(body, CirExpr::Let { .. }),
            "Guarded match should produce Let binding");
    }

    #[test]
    fn test_match_enum_variant() {
        let cir = source_to_cir(
            "enum Color { Red, Green, Blue }
             fn is_red(c: Color) -> i64 = match c { Color::Red => 1, _ => 0 };"
        );
        let body = &cir.functions[0].body;
        // Should use __is_Color_Red call for variant check
        assert!(matches!(body, CirExpr::Let { .. }),
            "Enum match should lower to Let + If with variant check");
    }

    #[test]
    fn test_match_bool_literal() {
        let cir = source_to_cir(
            "fn flip(b: bool) -> i64 = match b { true => 0, false => 1 };"
        );
        assert_eq!(cir.functions.len(), 1);
        let body = &cir.functions[0].body;
        assert!(matches!(body, CirExpr::Let { .. }),
            "Bool match should lower to Let + If");
    }

    #[test]
    fn test_match_nested_in_function() {
        let cir = source_to_cir(
            "fn abs(x: i64) -> i64 = {
                 let sign = match x { 0 => 0, _ => 1 };
                 sign
             };"
        );
        assert_eq!(cir.functions.len(), 1);
    }

    #[test]
    fn test_match_lowering_not_stub() {
        // Verify match is NOT just returning scrutinee (the old stub behavior)
        let cir = source_to_cir(
            "fn classify(x: i64) -> i64 = match x { 0 => 100, _ => 200 };"
        );
        let body = &cir.functions[0].body;
        // The old stub returned the scrutinee directly; new code should have Let+If
        if let CirExpr::Let { body: inner, .. } = body {
            // Inner should be an If, not just a Var
            assert!(!matches!(inner.as_ref(), CirExpr::Var(_)),
                "Match should NOT be a stub returning scrutinee");
        }
    }
}
