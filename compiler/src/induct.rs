//! Induction lemma drafts (ADR-0062 §5, ADR-0135, backlog T1-24): for an
//! executable function whose postcondition the gate does not prove, the
//! `proof fn` that takes a recursive predicate to a predicate of the same
//! parameters, and where to call it.
//!
//! The shape is mechanical. A postcondition, read through the definitions
//! of the spec fns it names, applies some `Q(args)`. A recursive `R` of the
//! same unit with the same parameter types, a `bool` result and `decreases
//! p` for an unsigned parameter `p` gives the draft
//!
//! ```text
//! proof fn R_gives_Q(params)
//!     requires R(params),
//!     ensures Q(params),
//!     decreases p,
//! {
//!     if p > 0 && R(params at p - 1) {
//!         R_gives_Q(params at p - 1);
//!     }
//! }
//! ```
//!
//! and the call `proof { R_gives_Q(args); }` before the function's result,
//! with the result's name replaced by the result. This module only writes
//! candidates, in a fixed order; the caller keeps the first one the gate
//! proves together with the function (`fix_sources_opts`), so a draft that
//! is shown is a lemma that holds and that closes the proof.

use crate::ast::{Expr, ExprKind, FnDef, FnKind, Mod, Program, TypeKind};
use crate::fmt;
use std::collections::BTreeMap;

/// One draft: two insertions in one file.
pub struct Draft {
    /// The function the draft is for, and the lemma's path.
    pub def: String,
    pub lemma: String,
    pub file: usize,
    /// Where the `proof fn` goes (the start of the function's first line)
    /// and its text, a blank line after it.
    pub lemma_at: usize,
    pub lemma_text: String,
    /// Where the call goes (the start of the line of the function's
    /// result) and its text.
    pub call_at: usize,
    pub call_text: String,
}

impl Draft {
    /// `text` with the draft written in.
    pub fn apply(&self, text: &str) -> String {
        let mut out = text.to_string();
        // The later position first: the earlier one stays where it is.
        out.insert_str(self.call_at, &self.call_text);
        out.insert_str(self.lemma_at, &self.lemma_text);
        out
    }
}

/// How deep a postcondition is read through non-recursive definitions.
const DEPTH: usize = 4;

/// `e` with each variable of `m` replaced, all at once. `None` for an
/// expression this does not read (a block, a `match`, a closure): the
/// draft is not written then.
fn subst(e: &Expr, m: &BTreeMap<String, Expr>) -> Option<Expr> {
    let b = |x: &Expr| subst(x, m).map(Box::new);
    let all = |xs: &[Expr]| xs.iter().map(|x| subst(x, m)).collect::<Option<Vec<_>>>();
    let kind = match &e.kind {
        ExprKind::Var(id) => return Some(m.get(&id.name).cloned().unwrap_or_else(|| e.clone())),
        ExprKind::Int(..)
        | ExprKind::Bool(_)
        | ExprKind::ByteLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::Unit
        | ExprKind::Path(_) => return Some(e.clone()),
        ExprKind::Call {
            path,
            targs,
            args,
            method,
        } => ExprKind::Call {
            path: path.clone(),
            targs: targs.clone(),
            args: all(args)?,
            method: *method,
        },
        ExprKind::Method { recv, name, args } => ExprKind::Method {
            recv: b(recv)?,
            name: name.clone(),
            args: all(args)?,
        },
        ExprKind::Index { base, index } => ExprKind::Index {
            base: b(base)?,
            index: b(index)?,
        },
        ExprKind::Unary(op, x) => ExprKind::Unary(*op, b(x)?),
        ExprKind::Binary(op, x, y) => ExprKind::Binary(*op, b(x)?, b(y)?),
        ExprKind::Ref { mutable, expr } => ExprKind::Ref {
            mutable: *mutable,
            expr: b(expr)?,
        },
        ExprKind::Deref(x) => ExprKind::Deref(b(x)?),
        ExprKind::Field { base, name } => ExprKind::Field {
            base: b(base)?,
            name: name.clone(),
        },
        ExprKind::Trigger(x) => ExprKind::Trigger(b(x)?),
        ExprKind::Quant { q, binders, body } => {
            // The bound names hide the outer ones.
            let mut inner = m.clone();
            for p in binders {
                inner.remove(&p.name.name);
            }
            ExprKind::Quant {
                q: *q,
                binders: binders.clone(),
                body: Box::new(subst(body, &inner)?),
            }
        }
        _ => return None,
    };
    Some(Expr { kind, span: e.span })
}

/// The applications of the unit's spec fns in `e`, outside quantifiers (an
/// argument that names a bound variable is no argument of a call), in
/// order.
fn apps<'a>(e: &'a Expr, unit: &str, out: &mut Vec<(&'a str, &'a [Expr])>) {
    match &e.kind {
        ExprKind::Call { path, args, .. } => {
            let local = match path.as_slice() {
                [f] => Some(f.name.as_str()),
                [u, f] if u.name == unit => Some(f.name.as_str()),
                _ => None,
            };
            if let Some(f) = local {
                out.push((f, args));
            }
            args.iter().for_each(|a| apps(a, unit, out));
        }
        ExprKind::Method { recv, args, .. } => {
            apps(recv, unit, out);
            args.iter().for_each(|a| apps(a, unit, out));
        }
        ExprKind::Index { base, index } => {
            apps(base, unit, out);
            apps(index, unit, out);
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Deref(x)
        | ExprKind::Field { base: x, .. }
        | ExprKind::Trigger(x) => apps(x, unit, out),
        ExprKind::Binary(_, x, y) => {
            apps(x, unit, out);
            apps(y, unit, out);
        }
        _ => {}
    }
}

fn calls_itself(f: &FnDef) -> bool {
    fn in_expr(e: &Expr, name: &str) -> bool {
        let mut found = Vec::new();
        apps(e, "", &mut found);
        if found.iter().any(|(f, _)| *f == name) {
            return true;
        }
        match &e.kind {
            ExprKind::If { cond, then, els } => {
                in_expr(cond, name)
                    || then.tail.as_deref().is_some_and(|t| in_expr(t, name))
                    || els.as_deref().is_some_and(|x| in_expr(x, name))
            }
            ExprKind::Block(b) => b.tail.as_deref().is_some_and(|t| in_expr(t, name)),
            ExprKind::Binary(_, x, y) => in_expr(x, name) || in_expr(y, name),
            ExprKind::Unary(_, x) => in_expr(x, name),
            _ => false,
        }
    }
    f.body
        .tail
        .as_deref()
        .is_some_and(|t| in_expr(t, &f.name.name))
}

/// A spec fn of plain shape: no generics, a `bool` result, a body that is
/// one expression.
fn predicate(f: &FnDef) -> bool {
    f.kind == FnKind::Spec
        && f.generics.is_empty()
        && f.ret.ty.kind == TypeKind::Bool
        && f.body.stmts.is_empty()
        && f.body.tail.is_some()
}

/// The parameter `R` descends on, when that is an unsigned integer
/// parameter named by `decreases` alone.
fn descent(r: &FnDef) -> Option<usize> {
    let ExprKind::Var(p) = &r.decreases.as_ref()?.kind else {
        return None;
    };
    r.params.iter().position(|q| {
        q.name.name == p.name && matches!(&q.ty.kind, TypeKind::Int(t) if !t.signed())
    })
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

/// The drafts for `def` (`unit::name`, an executable function of a
/// top-level unit), in the order they are tried: postconditions in order,
/// the applications each reaches in order, the unit's recursive predicates
/// in order.
pub fn drafts(p: &Program, texts: &[(String, String)], def: &str) -> Vec<Draft> {
    let Some((unit, name)) = def.split_once("::") else {
        return Vec::new();
    };
    let Some(m) = p.mods.iter().find(|m| m.name.name == unit) else {
        return Vec::new();
    };
    let Some(f) = m
        .fns
        .iter()
        .find(|f| f.name.name == name && f.kind == FnKind::Exec && f.generics.is_empty())
    else {
        return Vec::new();
    };
    let file = f.span.file as usize;
    let Some((_, text)) = texts.get(file) else {
        return Vec::new();
    };
    // The result, as the call names it: the function's last expression,
    // when that is a variable (anything else would be evaluated twice).
    let Some(tail) = f.body.tail.as_deref() else {
        return Vec::new();
    };
    if !matches!(tail.kind, ExprKind::Var(_)) {
        return Vec::new();
    }
    let mut result = BTreeMap::new();
    if let Some(r) = &f.ret.name {
        result.insert(r.name.clone(), tail.clone());
    }
    let first = f
        .attrs
        .iter()
        .map(|a| a.span.lo)
        .chain([f.span.lo])
        .min()
        .unwrap_or(f.span.lo) as usize;
    let lemma_at = line_start(text, first);
    let indent = &text[lemma_at..first];
    let call_at = line_start(text, tail.span.lo as usize);
    let call_indent = &text[call_at..tail.span.lo as usize];
    if !indent.trim().is_empty() || !call_indent.trim().is_empty() {
        return Vec::new();
    }

    let mut out: Vec<Draft> = Vec::new();
    for clause in &f.ensures {
        let mut reached = Vec::new();
        reach(clause, m, DEPTH, &mut reached);
        for (q, args) in reached {
            let Some(qf) = m.fns.iter().find(|g| g.name.name == q && predicate(g)) else {
                continue;
            };
            if calls_itself(qf) {
                continue;
            }
            let Some(args) = args
                .iter()
                .map(|a| subst(a, &result))
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            for r in m.fns.iter().filter(|r| predicate(r) && calls_itself(r)) {
                let Some(at) = descent(r) else { continue };
                let same = r.params.len() == qf.params.len()
                    && r.params
                        .iter()
                        .zip(&qf.params)
                        .all(|(a, b)| fmt::type_str(&a.ty) == fmt::type_str(&b.ty));
                let lemma = format!("{}_gives_{}", r.name.name, q);
                if !same
                    || m.fns.iter().any(|g| g.name.name == lemma)
                    || out.iter().any(|d| d.lemma == format!("{unit}::{lemma}"))
                {
                    continue;
                }
                let names: Vec<&str> = r.params.iter().map(|p| p.name.name.as_str()).collect();
                let params: Vec<String> = r
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name.name, fmt::type_str(&p.ty)))
                    .collect();
                let below: Vec<String> = names
                    .iter()
                    .enumerate()
                    .map(|(k, n)| {
                        if k == at {
                            format!("{n} - 1")
                        } else {
                            n.to_string()
                        }
                    })
                    .collect();
                let (rn, pn) = (&r.name.name, names[at]);
                let lemma_text = [
                    format!("proof fn {lemma}({})", params.join(", ")),
                    format!("    requires {rn}({}),", names.join(", ")),
                    format!("    ensures {q}({}),", names.join(", ")),
                    format!("    decreases {pn},"),
                    "{".to_string(),
                    format!("    if {pn} > 0 && {rn}({}) {{", below.join(", ")),
                    format!("        {lemma}({});", below.join(", ")),
                    "    }".to_string(),
                    "}".to_string(),
                ]
                .iter()
                .map(|l| format!("{indent}{l}\n"))
                .collect::<String>()
                    + "\n";
                let shown: Vec<String> = args.iter().map(fmt::expr_text).collect();
                let call_text = format!(
                    "{call_indent}proof {{\n{call_indent}    {lemma}({});\n{call_indent}}}\n",
                    shown.join(", ")
                );
                out.push(Draft {
                    def: def.to_string(),
                    lemma: format!("{unit}::{lemma}"),
                    file,
                    lemma_at,
                    lemma_text,
                    call_at,
                    call_text,
                });
            }
        }
    }
    out
}

/// The applications `e` makes and, through the bodies of the unit's
/// non-recursive predicates, those they make, with the arguments in the
/// terms of `e`.
fn reach(e: &Expr, m: &Mod, depth: usize, out: &mut Vec<(String, Vec<Expr>)>) {
    let mut here = Vec::new();
    apps(e, &m.name.name, &mut here);
    for (f, args) in here {
        out.push((f.to_string(), args.to_vec()));
        let Some(g) = m.fns.iter().find(|g| g.name.name == f && predicate(g)) else {
            continue;
        };
        if depth == 0 || calls_itself(g) || g.params.len() != args.len() {
            continue;
        }
        let map: BTreeMap<String, Expr> = g
            .params
            .iter()
            .zip(args)
            .map(|(p, a)| (p.name.name.clone(), a.clone()))
            .collect();
        if let Some(body) = g.body.tail.as_deref().and_then(|t| subst(t, &map)) {
            reach(&body, m, depth - 1, out);
        }
    }
}
