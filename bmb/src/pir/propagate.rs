//! Proof Propagation
//!
//! Phase 2.2: Convert CIR to PIR while propagating proofs through the program.
//!
//! Propagation rules:
//! 1. Preconditions are facts at function entry
//! 2. Branch conditions become facts in their respective branches
//! 3. Loop conditions are facts inside the loop body
//! 4. Let bindings transfer facts from value to variable
//! 5. Function calls add postconditions as facts after the call

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::cir::{
    CirProgram, CirFunction, CirExpr, Proposition, BinOp, UnaryOp, CompareOp,
};

#[cfg(test)]
use crate::cir::CirType;
use crate::verify::ProofDatabase;

use super::{
    PirProgram, PirFunction, PirExpr, PirExprKind, PirType, PirParam,
    PirBinOp, PirUnaryOp, ProvenFact,
};

/// Counter for generating unique fact IDs
static FACT_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_fact_id() -> u32 {
    FACT_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Propagation rules applied during CIR → PIR conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationRule {
    /// Preconditions become facts at function entry
    PreconditionToFact,

    /// In `if (cond) { ... }`, cond is true in then branch
    BranchCondition,

    /// In `while (cond) { ... }`, cond is true inside loop
    LoopCondition,

    /// `let x = e` transfers facts from e to x
    LetBinding,

    /// After function call, postconditions become facts
    PostconditionAfterCall,
}

/// Context for proof propagation
#[derive(Debug)]
struct PropagationContext {
    /// Currently proven facts
    facts: Vec<ProvenFact>,

    /// Variable -> facts map
    var_facts: HashMap<String, Vec<ProvenFact>>,

    /// Function signatures for postcondition lookup
    function_signatures: HashMap<String, FunctionSignature>,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    preconditions: Vec<Proposition>,
    postconditions: Vec<Proposition>,
}

impl PropagationContext {
    fn new() -> Self {
        Self {
            facts: Vec::new(),
            var_facts: HashMap::new(),
            function_signatures: HashMap::new(),
        }
    }

    fn with_facts(facts: Vec<ProvenFact>) -> Self {
        Self {
            facts,
            var_facts: HashMap::new(),
            function_signatures: HashMap::new(),
        }
    }

    fn add_fact(&mut self, fact: ProvenFact) {
        self.facts.push(fact);
    }

    fn add_var_fact(&mut self, var: &str, fact: ProvenFact) {
        self.var_facts
            .entry(var.to_string())
            .or_default()
            .push(fact);
    }

    fn get_var_facts(&self, var: &str) -> &[ProvenFact] {
        self.var_facts.get(var).map(|v| v.as_slice()).unwrap_or(&[])
    }

    fn current_facts(&self) -> Vec<ProvenFact> {
        self.facts.clone()
    }

    /// Create a child context with additional facts
    fn with_additional_facts(&self, additional: Vec<ProvenFact>) -> Self {
        let mut new_facts = self.facts.clone();
        new_facts.extend(additional);

        Self {
            facts: new_facts,
            var_facts: self.var_facts.clone(),
            function_signatures: self.function_signatures.clone(),
        }
    }
}

/// Convert CIR program to PIR with proof propagation
pub fn propagate_proofs(cir: &CirProgram, db: &ProofDatabase) -> PirProgram {
    let mut pir = PirProgram::new();

    // Build function signature map
    let mut signatures: HashMap<String, FunctionSignature> = HashMap::new();
    for func in &cir.functions {
        signatures.insert(
            func.name.clone(),
            FunctionSignature {
                preconditions: func.preconditions.iter().map(|p| p.proposition.clone()).collect(),
                postconditions: func.postconditions.iter().map(|p| p.proposition.clone()).collect(),
            },
        );
    }

    // Convert each function
    for func in &cir.functions {
        let pir_func = propagate_function(func, &signatures);
        pir.functions.push(pir_func);
    }

    // Copy type invariants
    pir.type_invariants = cir.type_invariants.clone();

    pir
}

/// Propagate proofs through a single function
fn propagate_function(
    func: &CirFunction,
    signatures: &HashMap<String, FunctionSignature>,
) -> PirFunction {
    // Start with preconditions as entry facts
    let entry_facts: Vec<ProvenFact> = func
        .preconditions
        .iter()
        .map(|pre| ProvenFact::from_precondition(pre.proposition.clone(), next_fact_id()))
        .collect();

    // Build context
    let mut ctx = PropagationContext::with_facts(entry_facts.clone());
    ctx.function_signatures = signatures.clone();

    // Add parameter facts
    for (i, param) in func.params.iter().enumerate() {
        // Add any constraints from preconditions that mention this parameter
        for fact in &entry_facts {
            if mentions_var(&fact.proposition, &param.name) {
                ctx.add_var_fact(&param.name, fact.clone());
            }
        }
    }

    // Convert body
    let body = propagate_expr(&func.body, &mut ctx);

    // Exit facts from postconditions
    let exit_facts: Vec<ProvenFact> = func
        .postconditions
        .iter()
        .map(|post| ProvenFact::from_precondition(post.proposition.clone(), next_fact_id()))
        .collect();

    PirFunction {
        name: func.name.clone(),
        params: func.params.iter().map(|p| PirParam {
            name: p.name.clone(),
            ty: PirType::from_cir(&p.ty),
            constraints: ctx.get_var_facts(&p.name).to_vec(),
        }).collect(),
        ret_ty: PirType::from_cir(&func.ret_ty),
        body,
        entry_facts,
        exit_facts,
    }
}

/// Propagate proofs through an expression
fn propagate_expr(expr: &CirExpr, ctx: &mut PropagationContext) -> PirExpr {
    let current_facts = ctx.current_facts();

    match expr {
        // === Literals ===
        CirExpr::IntLit(n) => {
            PirExpr::new(PirExprKind::IntLit(*n), PirType::I64)
                .with_proven(current_facts)
        }

        CirExpr::FloatLit(bits) => {
            PirExpr::new(PirExprKind::FloatLit(*bits), PirType::F64)
                .with_proven(current_facts)
        }

        CirExpr::BoolLit(b) => {
            PirExpr::new(PirExprKind::BoolLit(*b), PirType::Bool)
                .with_proven(current_facts)
        }

        CirExpr::StringLit(s) => {
            PirExpr::new(PirExprKind::StringLit(s.clone()), PirType::String)
                .with_proven(current_facts)
        }

        CirExpr::Unit => {
            PirExpr::new(PirExprKind::Unit, PirType::Unit)
                .with_proven(current_facts)
        }

        // === Variables ===
        CirExpr::Var(name) => {
            let var_facts = ctx.get_var_facts(name).to_vec();
            let mut all_facts = current_facts;
            all_facts.extend(var_facts);

            PirExpr::new(PirExprKind::Var(name.clone()), PirType::Infer)
                .with_proven(all_facts)
        }

        // === Binary operations ===
        CirExpr::BinOp { op, lhs, rhs } => {
            let pir_lhs = propagate_expr(lhs, ctx);
            let pir_rhs = propagate_expr(rhs, ctx);

            // Special handling for division
            if matches!(op, BinOp::Div) {
                let nonzero_proof = find_nonzero_proof(&current_facts, rhs);

                return PirExpr::new(
                    PirExprKind::Div {
                        lhs: Box::new(pir_lhs),
                        rhs: Box::new(pir_rhs),
                        nonzero_proof,
                    },
                    PirType::I64,
                ).with_proven(current_facts);
            }

            let pir_op = convert_binop(*op);
            PirExpr::new(
                PirExprKind::BinOp {
                    op: pir_op,
                    lhs: Box::new(pir_lhs),
                    rhs: Box::new(pir_rhs),
                },
                infer_binop_type(*op),
            ).with_proven(current_facts)
        }

        // === Unary operations ===
        CirExpr::UnaryOp { op, operand } => {
            let pir_operand = propagate_expr(operand, ctx);
            let pir_op = convert_unaryop(*op);

            PirExpr::new(
                PirExprKind::UnaryOp {
                    op: pir_op,
                    operand: Box::new(pir_operand),
                },
                PirType::Infer,
            ).with_proven(current_facts)
        }

        // === Array indexing ===
        CirExpr::Index { base, index } => {
            let pir_base = propagate_expr(base, ctx);
            let pir_index = propagate_expr(index, ctx);

            // Check if bounds are proven
            let bounds_proof = find_bounds_proof(&current_facts, index, base);

            PirExpr::new(
                PirExprKind::Index {
                    array: Box::new(pir_base),
                    index: Box::new(pir_index),
                    bounds_proof,
                },
                PirType::Infer,
            ).with_proven(current_facts)
        }

        // === Field access ===
        CirExpr::Field { base, field } => {
            let pir_base = propagate_expr(base, ctx);

            // Check if non-null is proven
            let null_proof = find_null_proof(&current_facts, base);

            PirExpr::new(
                PirExprKind::Field {
                    base: Box::new(pir_base),
                    field: field.clone(),
                    null_proof,
                },
                PirType::Infer,
            ).with_proven(current_facts)
        }

        // === If expression ===
        CirExpr::If { cond, then_branch, else_branch } => {
            let pir_cond = propagate_expr(cond, ctx);

            // Convert condition to proposition for then branch
            let then_prop = expr_to_proposition(cond);
            let then_fact = ProvenFact::from_control_flow(then_prop.clone(), next_fact_id());
            let then_facts = vec![then_fact.clone()];

            // Negate for else branch
            let else_prop = Proposition::Not(Box::new(then_prop));
            let else_fact = ProvenFact::from_control_flow(else_prop, next_fact_id());
            let else_facts = vec![else_fact.clone()];

            // Propagate through branches with additional facts
            let mut then_ctx = ctx.with_additional_facts(then_facts.clone());
            let pir_then = propagate_expr(then_branch, &mut then_ctx);

            let mut else_ctx = ctx.with_additional_facts(else_facts.clone());
            let pir_else = propagate_expr(else_branch, &mut else_ctx);

            PirExpr::new(
                PirExprKind::If {
                    cond: Box::new(pir_cond),
                    then_branch: Box::new(pir_then),
                    else_branch: Box::new(pir_else),
                    then_facts,
                    else_facts,
                },
                PirType::Infer,
            ).with_proven(current_facts)
        }

        // === While loop ===
        CirExpr::While { cond, body, invariant } => {
            let pir_cond = propagate_expr(cond, ctx);

            // Condition is true inside loop body
            let loop_prop = expr_to_proposition(cond);
            let loop_fact = ProvenFact::from_control_flow(loop_prop, next_fact_id());
            let mut invariant_facts = vec![loop_fact];

            // Add explicit invariant if present
            if let Some(inv) = invariant {
                invariant_facts.push(ProvenFact::from_precondition(inv.clone(), next_fact_id()));
            }

            let mut body_ctx = ctx.with_additional_facts(invariant_facts.clone());
            let pir_body = propagate_expr(body, &mut body_ctx);

            PirExpr::new(
                PirExprKind::While {
                    cond: Box::new(pir_cond),
                    body: Box::new(pir_body),
                    invariant_facts,
                },
                PirType::Unit,
            ).with_proven(current_facts)
        }

        // === Loop ===
        CirExpr::Loop { body } => {
            let pir_body = propagate_expr(body, ctx);

            PirExpr::new(
                PirExprKind::Loop {
                    body: Box::new(pir_body),
                    invariant_facts: vec![],
                },
                PirType::Never,
            ).with_proven(current_facts)
        }

        // === For loop ===
        CirExpr::For { var, iter, body } => {
            let pir_iter = propagate_expr(iter, ctx);

            // Add facts about iteration variable (e.g., 0 <= var < len)
            let iter_facts = generate_iterator_facts(var, iter);

            let mut body_ctx = ctx.with_additional_facts(iter_facts.clone());
            // Add variable to context
            for fact in &iter_facts {
                body_ctx.add_var_fact(var, fact.clone());
            }

            let pir_body = propagate_expr(body, &mut body_ctx);

            PirExpr::new(
                PirExprKind::For {
                    var: var.clone(),
                    iter: Box::new(pir_iter),
                    body: Box::new(pir_body),
                    iter_facts,
                },
                PirType::Unit,
            ).with_proven(current_facts)
        }

        // === Let binding ===
        CirExpr::Let { name, ty, value, body } => {
            let pir_value = propagate_expr(value, ctx);

            // Transfer value's result facts to variable
            let value_facts = pir_value.result_facts.clone();
            for fact in &value_facts {
                ctx.add_var_fact(name, fact.clone());
            }

            let pir_body = propagate_expr(body, ctx);

            PirExpr::new(
                PirExprKind::Let {
                    name: name.clone(),
                    ty: PirType::from_cir(ty),
                    value: Box::new(pir_value),
                    body: Box::new(pir_body),
                    value_facts,
                },
                PirType::Infer,
            ).with_proven(current_facts)
        }

        // === LetMut ===
        CirExpr::LetMut { name, ty, value, body } => {
            let pir_value = propagate_expr(value, ctx);
            let pir_body = propagate_expr(body, ctx);

            PirExpr::new(
                PirExprKind::LetMut {
                    name: name.clone(),
                    ty: PirType::from_cir(ty),
                    value: Box::new(pir_value),
                    body: Box::new(pir_body),
                },
                PirType::Infer,
            ).with_proven(current_facts)
        }

        // === Assign ===
        CirExpr::Assign { target, value } => {
            let pir_value = propagate_expr(value, ctx);

            PirExpr::new(
                PirExprKind::Assign {
                    target: target.clone(),
                    value: Box::new(pir_value),
                },
                PirType::Unit,
            ).with_proven(current_facts)
        }

        // === Function call ===
        CirExpr::Call { func, args } => {
            let pir_args: Vec<PirExpr> = args.iter()
                .map(|a| propagate_expr(a, ctx))
                .collect();

            // Get postcondition facts from callee
            let postcondition_facts = if let Some(sig) = ctx.function_signatures.get(func) {
                sig.postconditions.iter()
                    .map(|p| ProvenFact::from_precondition(p.clone(), next_fact_id()))
                    .collect()
            } else {
                vec![]
            };

            PirExpr::new(
                PirExprKind::Call {
                    func: func.clone(),
                    args: pir_args,
                    postcondition_facts: postcondition_facts.clone(),
                },
                PirType::Infer,
            ).with_proven(current_facts)
            .with_result_facts(postcondition_facts)
        }

        // === Block ===
        CirExpr::Block(exprs) => {
            let pir_exprs: Vec<PirExpr> = exprs.iter()
                .map(|e| propagate_expr(e, ctx))
                .collect();

            PirExpr::new(
                PirExprKind::Block(pir_exprs),
                PirType::Infer,
            ).with_proven(current_facts)
        }

        // === Struct ===
        CirExpr::Struct { name, fields } => {
            let pir_fields: Vec<(String, PirExpr)> = fields.iter()
                .map(|(n, e)| (n.clone(), propagate_expr(e, ctx)))
                .collect();

            PirExpr::new(
                PirExprKind::Struct {
                    name: name.clone(),
                    fields: pir_fields,
                },
                PirType::Struct(name.clone()),
            ).with_proven(current_facts)
        }

        // === Break ===
        CirExpr::Break(expr) => {
            let pir_expr = propagate_expr(expr, ctx);

            PirExpr::new(
                PirExprKind::Break(Box::new(pir_expr)),
                PirType::Never,
            ).with_proven(current_facts)
        }

        // === Continue ===
        CirExpr::Continue => {
            PirExpr::new(PirExprKind::Continue, PirType::Never)
                .with_proven(current_facts)
        }

        // === Other expressions (simplified) ===
        CirExpr::Array(exprs) => {
            let pir_exprs: Vec<PirExpr> = exprs.iter()
                .map(|e| propagate_expr(e, ctx))
                .collect();

            PirExpr::new(
                PirExprKind::Array(pir_exprs),
                PirType::Infer,
            ).with_proven(current_facts)
        }

        CirExpr::Tuple(exprs) => {
            let pir_exprs: Vec<PirExpr> = exprs.iter()
                .map(|e| propagate_expr(e, ctx))
                .collect();

            PirExpr::new(
                PirExprKind::Tuple(pir_exprs),
                PirType::Infer,
            ).with_proven(current_facts)
        }

        CirExpr::Ref(expr) => {
            let pir_expr = propagate_expr(expr, ctx);
            PirExpr::new(
                PirExprKind::Ref(Box::new(pir_expr)),
                PirType::Infer,
            ).with_proven(current_facts)
        }

        CirExpr::RefMut(expr) => {
            let pir_expr = propagate_expr(expr, ctx);
            PirExpr::new(
                PirExprKind::RefMut(Box::new(pir_expr)),
                PirType::Infer,
            ).with_proven(current_facts)
        }

        CirExpr::Deref(expr) => {
            let pir_expr = propagate_expr(expr, ctx);
            PirExpr::new(
                PirExprKind::Deref(Box::new(pir_expr)),
                PirType::Infer,
            ).with_proven(current_facts)
        }

        CirExpr::Len(expr) => {
            let pir_expr = propagate_expr(expr, ctx);
            PirExpr::new(
                PirExprKind::Len(Box::new(pir_expr)),
                PirType::I64,
            ).with_proven(current_facts)
        }

        CirExpr::Cast { expr, ty } => {
            let pir_expr = propagate_expr(expr, ctx);
            let pir_ty = PirType::from_cir(ty);

            PirExpr::new(
                PirExprKind::Cast {
                    expr: Box::new(pir_expr),
                    ty: pir_ty.clone(),
                },
                pir_ty,
            ).with_proven(current_facts)
        }

        // Default case for other expressions
        _ => {
            PirExpr::new(PirExprKind::Unit, PirType::Unit)
                .with_proven(current_facts)
        }
    }
}

// === Helper functions ===

fn convert_binop(op: BinOp) -> PirBinOp {
    match op {
        BinOp::Add | BinOp::AddWrap | BinOp::AddChecked | BinOp::AddSat => PirBinOp::Add,
        BinOp::Sub | BinOp::SubWrap | BinOp::SubChecked | BinOp::SubSat => PirBinOp::Sub,
        BinOp::Mul | BinOp::MulWrap | BinOp::MulChecked | BinOp::MulSat => PirBinOp::Mul,
        BinOp::Div => PirBinOp::Add, // Division is handled separately
        BinOp::Mod => PirBinOp::Mod,
        BinOp::Lt => PirBinOp::Lt,
        BinOp::Le => PirBinOp::Le,
        BinOp::Gt => PirBinOp::Gt,
        BinOp::Ge => PirBinOp::Ge,
        BinOp::Eq => PirBinOp::Eq,
        BinOp::Ne => PirBinOp::Ne,
        BinOp::And => PirBinOp::And,
        BinOp::Or => PirBinOp::Or,
        BinOp::Implies => PirBinOp::Or, // a => b is !a || b
        BinOp::BitAnd => PirBinOp::BitAnd,
        BinOp::BitOr => PirBinOp::BitOr,
        BinOp::BitXor => PirBinOp::BitXor,
        BinOp::Shl => PirBinOp::Shl,
        BinOp::Shr => PirBinOp::Shr,
    }
}

fn convert_unaryop(op: UnaryOp) -> PirUnaryOp {
    match op {
        UnaryOp::Neg => PirUnaryOp::Neg,
        UnaryOp::Not => PirUnaryOp::Not,
        UnaryOp::BitNot => PirUnaryOp::BitNot,
    }
}

fn infer_binop_type(op: BinOp) -> PirType {
    match op {
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge |
        BinOp::Eq | BinOp::Ne | BinOp::And | BinOp::Or | BinOp::Implies => PirType::Bool,
        _ => PirType::I64,
    }
}

/// Convert a CIR expression to a proposition (for conditions)
fn expr_to_proposition(expr: &CirExpr) -> Proposition {
    match expr {
        CirExpr::BoolLit(true) => Proposition::True,
        CirExpr::BoolLit(false) => Proposition::False,
        CirExpr::BinOp { op, lhs, rhs } => {
            match op {
                BinOp::Lt => Proposition::Compare {
                    lhs: lhs.clone(),
                    op: CompareOp::Lt,
                    rhs: rhs.clone(),
                },
                BinOp::Le => Proposition::Compare {
                    lhs: lhs.clone(),
                    op: CompareOp::Le,
                    rhs: rhs.clone(),
                },
                BinOp::Gt => Proposition::Compare {
                    lhs: lhs.clone(),
                    op: CompareOp::Gt,
                    rhs: rhs.clone(),
                },
                BinOp::Ge => Proposition::Compare {
                    lhs: lhs.clone(),
                    op: CompareOp::Ge,
                    rhs: rhs.clone(),
                },
                BinOp::Eq => Proposition::Compare {
                    lhs: lhs.clone(),
                    op: CompareOp::Eq,
                    rhs: rhs.clone(),
                },
                BinOp::Ne => Proposition::Compare {
                    lhs: lhs.clone(),
                    op: CompareOp::Ne,
                    rhs: rhs.clone(),
                },
                BinOp::And => Proposition::And(vec![
                    expr_to_proposition(lhs),
                    expr_to_proposition(rhs),
                ]),
                BinOp::Or => Proposition::Or(vec![
                    expr_to_proposition(lhs),
                    expr_to_proposition(rhs),
                ]),
                _ => Proposition::True, // Default for non-boolean ops
            }
        }
        CirExpr::UnaryOp { op: UnaryOp::Not, operand } => {
            Proposition::Not(Box::new(expr_to_proposition(operand)))
        }
        _ => Proposition::True, // Default
    }
}

/// Check if a proposition mentions a variable
fn mentions_var(prop: &Proposition, var: &str) -> bool {
    match prop {
        Proposition::Compare { lhs, rhs, .. } => {
            expr_mentions_var(lhs, var) || expr_mentions_var(rhs, var)
        }
        Proposition::Not(inner) => mentions_var(inner, var),
        Proposition::And(props) | Proposition::Or(props) => {
            props.iter().any(|p| mentions_var(p, var))
        }
        Proposition::Implies(lhs, rhs) => {
            mentions_var(lhs, var) || mentions_var(rhs, var)
        }
        Proposition::Forall { body, .. } |
        Proposition::Exists { body, .. } => mentions_var(body, var),
        Proposition::InBounds { index, array } => {
            expr_mentions_var(index, var) || expr_mentions_var(array, var)
        }
        Proposition::NonNull(expr) => expr_mentions_var(expr, var),
        Proposition::Predicate { args, .. } => {
            args.iter().any(|a| expr_mentions_var(a, var))
        }
        Proposition::Old(expr, inner) => {
            expr_mentions_var(expr, var) || mentions_var(inner, var)
        }
        Proposition::True | Proposition::False => false,
    }
}

fn expr_mentions_var(expr: &CirExpr, var: &str) -> bool {
    match expr {
        CirExpr::Var(name) => name == var,
        CirExpr::BinOp { lhs, rhs, .. } => {
            expr_mentions_var(lhs, var) || expr_mentions_var(rhs, var)
        }
        CirExpr::UnaryOp { operand, .. } => expr_mentions_var(operand, var),
        CirExpr::Index { base, index } => {
            expr_mentions_var(base, var) || expr_mentions_var(index, var)
        }
        CirExpr::Field { base, .. } => expr_mentions_var(base, var),
        CirExpr::Call { args, .. } => args.iter().any(|a| expr_mentions_var(a, var)),
        CirExpr::Len(e) => expr_mentions_var(e, var),
        _ => false,
    }
}

/// Find bounds proof for array access
fn find_bounds_proof(facts: &[ProvenFact], index: &CirExpr, array: &CirExpr) -> Option<ProvenFact> {
    facts.iter().find(|f| {
        matches!(&f.proposition, Proposition::InBounds { .. }) ||
        // Also check for explicit comparison facts
        matches!(&f.proposition, Proposition::Compare { op: CompareOp::Lt, .. })
    }).cloned()
}

/// Find non-null proof for pointer access
fn find_null_proof(facts: &[ProvenFact], expr: &CirExpr) -> Option<ProvenFact> {
    facts.iter().find(|f| {
        matches!(&f.proposition, Proposition::NonNull(_))
    }).cloned()
}

/// Find non-zero proof for division
fn find_nonzero_proof(facts: &[ProvenFact], divisor: &CirExpr) -> Option<ProvenFact> {
    facts.iter().find(|f| {
        match &f.proposition {
            Proposition::Compare { op: CompareOp::Ne, rhs, .. } => {
                matches!(rhs.as_ref(), CirExpr::IntLit(0))
            }
            _ => false,
        }
    }).cloned()
}

/// Generate facts about iterator variable in for loops
fn generate_iterator_facts(var: &str, iter: &CirExpr) -> Vec<ProvenFact> {
    let mut facts = vec![];

    // For range iterations, add bounds facts
    if let CirExpr::Range { start, end, .. } = iter {
        // var >= start
        facts.push(ProvenFact::from_control_flow(
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var(var.to_string())),
                op: CompareOp::Ge,
                rhs: start.clone(),
            },
            next_fact_id(),
        ));

        // var < end
        facts.push(ProvenFact::from_control_flow(
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var(var.to_string())),
                op: CompareOp::Lt,
                rhs: end.clone(),
            },
            next_fact_id(),
        ));
    }

    facts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::{CirProgram, CirFunction, CirParam, NamedProposition, EffectSet};
    use std::collections::HashMap;

    fn make_test_function() -> CirFunction {
        CirFunction {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![
                CirParam {
                    name: "x".to_string(),
                    ty: CirType::I64,
                    constraints: vec![],
                },
            ],
            ret_name: "result".to_string(),
            ret_ty: CirType::I64,
            preconditions: vec![
                NamedProposition {
                    name: None,
                    proposition: Proposition::Compare {
                        lhs: Box::new(CirExpr::Var("x".to_string())),
                        op: CompareOp::Gt,
                        rhs: Box::new(CirExpr::IntLit(0)),
                    },
                },
            ],
            postconditions: vec![],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
            body: CirExpr::Var("x".to_string()),
        }
    }

    #[test]
    fn test_propagate_proofs_basic() {
        let cir = CirProgram {
            functions: vec![make_test_function()],
            extern_fns: vec![],
            structs: HashMap::new(),
            type_invariants: HashMap::new(),
        };

        let db = ProofDatabase::new();
        let pir = propagate_proofs(&cir, &db);

        assert_eq!(pir.functions.len(), 1);
        assert_eq!(pir.functions[0].entry_facts.len(), 1);
    }

    #[test]
    fn test_expr_to_proposition() {
        let expr = CirExpr::BinOp {
            op: BinOp::Lt,
            lhs: Box::new(CirExpr::Var("x".to_string())),
            rhs: Box::new(CirExpr::IntLit(10)),
        };

        let prop = expr_to_proposition(&expr);
        assert!(matches!(prop, Proposition::Compare { op: CompareOp::Lt, .. }));
    }

    #[test]
    fn test_mentions_var() {
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Gt,
            rhs: Box::new(CirExpr::IntLit(0)),
        };

        assert!(mentions_var(&prop, "x"));
        assert!(!mentions_var(&prop, "y"));
    }

    #[test]
    fn test_branch_fact_propagation() {
        let func = CirFunction {
            name: "branch_test".to_string(),
            type_params: vec![],
            params: vec![CirParam {
                name: "x".to_string(),
                ty: CirType::I64,
                constraints: vec![],
            }],
            ret_name: "result".to_string(),
            ret_ty: CirType::I64,
            preconditions: vec![],
            postconditions: vec![],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
            body: CirExpr::If {
                cond: Box::new(CirExpr::BinOp {
                    op: BinOp::Gt,
                    lhs: Box::new(CirExpr::Var("x".to_string())),
                    rhs: Box::new(CirExpr::IntLit(0)),
                }),
                then_branch: Box::new(CirExpr::Var("x".to_string())),
                else_branch: Box::new(CirExpr::IntLit(0)),
            },
        };

        let signatures = HashMap::new();
        let pir_func = propagate_function(&func, &signatures);

        // Check that if expression has branch facts
        if let PirExprKind::If { then_facts, else_facts, .. } = &pir_func.body.kind {
            assert!(!then_facts.is_empty(), "Then branch should have facts");
            assert!(!else_facts.is_empty(), "Else branch should have facts");
        } else {
            panic!("Expected If expression");
        }
    }
}
