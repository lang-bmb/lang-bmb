//! Ownership of values that move (ADR-0031 §2, ADR-0047 §2, backlogs
//! 2G-02 and 2O-01): a `Vec`, a `Box`, and the data types that hold one
//! (`check::is_move`).
//!
//! Such a local is moved by a value use: a `let` initializer, the right
//! side of an assignment, a call argument, a block's value, the scrutinee
//! of a `match` that binds an owned payload. Reading through it (`v.len()`,
//! `v[i]`, `&v`, `*b`, `b.f`) is not a move. A use of a moved local is
//! E0413; assigning it again makes it owned again. Moving out of a field
//! or out of `*b` is E0413 until partial moves (backlog 2O-03).
//!
//! At every join (the branches of `if` and `match`, the right side of `&&`,
//! `||` and `==>`, and a loop head against the end of its body) a local is
//! owned on all paths or moved on all paths; "maybe moved" is E0414. So the
//! place where each value is dropped is fixed by the program text, and the
//! pass records it for code generation: the locals still owned at the end
//! of each block and arm, and the assignments that replace an owned value.
//!
//! The language has no `return`, `break` or `continue`, so blocks, branches
//! and loops are the whole control flow.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::check::{Adt, is_move};
use crate::diag::Diag;
use crate::source::Span;

type Key = (u32, u32, u32);

fn key(s: Span) -> Key {
    (s.file, s.lo, s.hi)
}

#[derive(Clone, Copy, Debug)]
enum St {
    Owned,
    Moved,
}

impl St {
    fn same(self, other: St) -> bool {
        matches!(
            (self, other),
            (St::Owned, St::Owned) | (St::Moved, St::Moved)
        )
    }
}

/// Where the owned values of one function are dropped.
#[derive(Debug, Clone, Default)]
pub struct MoveInfo {
    /// Per block (by span): the owned locals declared in it, or the owned
    /// parameters for the function body, still owned at its end.
    pub drops: BTreeMap<Key, Vec<String>>,
    /// Assignments (by statement span) that replace an owned value: the old
    /// value is dropped after the right side is evaluated.
    pub replaces: BTreeSet<Key>,
    /// Per `match` arm (by its body's span): the bindings still owned at
    /// its end.
    pub arm_drops: BTreeMap<Key, Vec<String>>,
    /// The `match`es (by span) that move their scrutinee: some arm binds an
    /// owned payload. Each arm drops the owned payloads it does not bind.
    pub consumed: BTreeSet<Key>,
}

struct Mv<'a> {
    /// Canonical types of the function's parameters and locals (names are
    /// unique in a function, E0303).
    locals: &'a BTreeMap<String, TypeKind>,
    types: &'a BTreeMap<Key, TypeKind>,
    adts: &'a BTreeMap<String, Adt>,
    state: BTreeMap<String, St>,
    diags: Vec<Diag>,
    info: MoveInfo,
}

/// Checks the moves of an executable function and returns its diagnostics
/// and drop points.
pub fn check(
    f: &FnDef,
    locals: &BTreeMap<String, TypeKind>,
    types: &BTreeMap<Key, TypeKind>,
    adts: &BTreeMap<String, Adt>,
) -> (Vec<Diag>, MoveInfo) {
    let mut m = Mv {
        locals,
        types,
        adts,
        state: BTreeMap::new(),
        diags: Vec::new(),
        info: MoveInfo::default(),
    };
    for p in &f.params {
        if m.owns(&p.name.name) {
            m.state.insert(p.name.name.clone(), St::Owned);
        }
    }
    m.block(&f.body);
    let params: Vec<String> = m
        .state
        .iter()
        .filter(|(_, s)| matches!(s, St::Owned))
        .map(|(n, _)| n.clone())
        .collect();
    if !params.is_empty() {
        m.info
            .drops
            .entry(key(f.body.span))
            .or_default()
            .extend(params);
    }
    (m.diags, m.info)
}

impl Mv<'_> {
    /// Whether local `name` holds a value that moves.
    fn owns(&self, name: &str) -> bool {
        self.locals.get(name).is_some_and(|t| is_move(t, self.adts))
    }

    /// Whether the value of expression `e` moves.
    fn moves_ty(&self, e: &Expr) -> bool {
        self.types
            .get(&key(e.span))
            .is_some_and(|t| is_move(t, self.adts))
    }

    /// E0413 for a move out of a place that is not a whole local.
    fn move_out(&mut self, e: &Expr, msg: String, fix: String) {
        self.diags
            .push(Diag::new("E0413", e.span, msg).fix(fix, None));
    }

    fn use_var(&mut self, id: &Ident, span: Span, moves: bool) {
        match self.state.get(&id.name) {
            Some(St::Owned) => {
                if moves {
                    self.state.insert(id.name.clone(), St::Moved);
                }
            }
            Some(St::Moved) => {
                self.diags.push(
                    Diag::new(
                        "E0413",
                        span,
                        format!("`{}` is used after it was moved", id.name),
                    )
                    .fix(
                        format!(
                            "move `{0}.clone()` instead, or assign `{0}` again before this use",
                            id.name
                        ),
                        None,
                    ),
                );
                // One report per move: the use counts as owned from here.
                let st = if moves { St::Moved } else { St::Owned };
                self.state.insert(id.name.clone(), st);
            }
            None => {}
        }
    }

    /// The join of two states: a local owned on one path and moved on the
    /// other is E0414 (and moved from here on).
    fn merge(&mut self, a: BTreeMap<String, St>, b: BTreeMap<String, St>, span: Span) {
        let mut out = BTreeMap::new();
        for (n, sa) in &a {
            let Some(sb) = b.get(n) else { continue };
            if sa.same(*sb) {
                out.insert(n.clone(), *sa);
            } else {
                self.maybe_moved(n, span, "on one path and not on another");
                out.insert(n.clone(), St::Moved);
            }
        }
        self.state = out;
    }

    fn maybe_moved(&mut self, name: &str, span: Span, how: &str) {
        self.diags.push(
            Diag::new("E0414", span, format!("`{name}` is moved {how}")).fix(
                format!(
                    "move or assign `{name}` on every path: an owned value is dropped at a point fixed by the program text (ADR-0031 §2)"
                ),
                None,
            ),
        );
    }

    /// A block: its value (the tail) is moved out, as in Rust, also where
    /// the block is only read (`{ v }.len()`).
    fn block(&mut self, b: &Block) {
        let outer: BTreeSet<String> = self.state.keys().cloned().collect();
        for s in &b.stmts {
            self.stmt(s);
        }
        if let Some(t) = &b.tail {
            self.expr(t, true);
        }
        let inner: Vec<String> = self
            .state
            .keys()
            .filter(|n| !outer.contains(*n))
            .cloned()
            .collect();
        let mut owned = Vec::new();
        for n in inner {
            if let Some(St::Owned) = self.state.remove(&n) {
                owned.push(n);
            }
        }
        if !owned.is_empty() {
            self.info.drops.insert(key(b.span), owned);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let { name, init, .. } => {
                self.expr(init, true);
                if self.owns(&name.name) {
                    self.state.insert(name.name.clone(), St::Owned);
                }
            }
            Stmt::LetFields { init, .. } => self.expr(init, false),
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                self.expr(value, true);
                match &target.kind {
                    ExprKind::Var(id) if self.owns(&id.name) => {
                        if let Some(St::Owned) = self.state.get(&id.name) {
                            self.info.replaces.insert(key(*span));
                        }
                        self.state.insert(id.name.clone(), St::Owned);
                    }
                    // `*v = w` for `v: &mut Vec<T>` or `v: Box<T>`, `s.f = w`:
                    // the value behind a reference, in a `Box` or in a field
                    // of a usable local is owned.
                    ExprKind::Deref(x) => {
                        if self.moves_ty(target) {
                            self.info.replaces.insert(key(*span));
                        }
                        self.expr(x, false);
                    }
                    ExprKind::Index { base, index } => {
                        self.expr(base, false);
                        self.expr(index, true);
                    }
                    ExprKind::Field { base, .. } => {
                        if self.moves_ty(target) {
                            self.info.replaces.insert(key(*span));
                        }
                        self.expr(base, false);
                    }
                    _ => {}
                }
            }
            Stmt::While {
                cond,
                invariants,
                body,
                span,
                ..
            } => {
                let head = self.state.clone();
                for inv in invariants {
                    self.expr(inv, false);
                }
                self.expr(cond, true);
                let exit = self.state.clone();
                self.block(body);
                let mut out = exit;
                for (n, s0) in &head {
                    if let Some(s1) = self.state.get(n)
                        && !s0.same(*s1)
                    {
                        let how = if matches!(s0, St::Owned) {
                            "in the loop and not assigned again before the next iteration"
                        } else {
                            "before the loop and assigned only inside it"
                        };
                        self.maybe_moved(n, *span, how);
                        out.insert(n.clone(), St::Moved);
                    }
                }
                self.state = out;
            }
            Stmt::Assert { cond, .. } => self.expr(cond, false),
            Stmt::Expr(e) => self.expr(e, true),
        }
    }

    fn expr(&mut self, e: &Expr, moves: bool) {
        match &e.kind {
            ExprKind::Var(id) => {
                if self.owns(&id.name) {
                    self.use_var(id, e.span, moves);
                }
            }
            ExprKind::Call { args, .. } => {
                for a in args {
                    self.expr(a, true);
                }
            }
            ExprKind::Method { recv, args, .. } => {
                self.expr(recv, false);
                for a in args {
                    self.expr(a, true);
                }
            }
            ExprKind::Index { base, index } => {
                self.expr(base, false);
                self.expr(index, true);
            }
            ExprKind::Unary(_, x) => self.expr(x, true),
            ExprKind::Binary(BinOp::And | BinOp::Or | BinOp::Implies, l, r) => {
                self.expr(l, true);
                let skip = self.state.clone();
                self.expr(r, true);
                let taken = std::mem::take(&mut self.state);
                self.merge(skip, taken, e.span);
            }
            ExprKind::Binary(_, l, r) => {
                self.expr(l, false);
                self.expr(r, false);
            }
            ExprKind::Ref { expr, .. } => self.expr(expr, false),
            ExprKind::If { cond, then, els } => {
                self.expr(cond, true);
                let before = self.state.clone();
                self.block(then);
                let after_then = std::mem::replace(&mut self.state, before);
                if let Some(el) = els {
                    self.expr(el, true);
                }
                let after_else = std::mem::take(&mut self.state);
                self.merge(after_then, after_else, e.span);
            }
            ExprKind::Block(b) => self.block(b),
            ExprKind::Array(items) => {
                for it in items {
                    self.expr(it, true);
                }
            }
            ExprKind::Repeat { value, .. } => self.expr(value, true),
            ExprKind::Old(x) | ExprKind::Trigger(x) => self.expr(x, false),
            ExprKind::Forall { body, .. } => self.expr(body, false),
            ExprKind::StructLit { fields, rest, .. } => {
                for (_, v) in fields {
                    self.expr(v, true);
                }
                if let Some(r) = rest {
                    self.expr(r, false);
                }
            }
            ExprKind::Deref(x) => {
                if moves && self.moves_ty(e) {
                    let fix = if matches!(self.types.get(&key(e.span)), Some(TypeKind::Vec(_))) {
                        format!("move `{}.clone()` instead", crate::fmt::expr_text(x))
                    } else {
                        "read through it (`*b` of a copyable value, `b.f`) or move the whole value; moving out comes with backlog 2O-03".into()
                    };
                    self.move_out(
                        e,
                        format!("cannot move out of `{}`", crate::fmt::expr_text(e)),
                        fix,
                    );
                }
                self.expr(x, false);
            }
            ExprKind::Field { base, .. } => {
                if moves && self.moves_ty(e) {
                    self.move_out(
                        e,
                        format!(
                            "cannot move the field `{}` out of its struct",
                            crate::fmt::expr_text(e)
                        ),
                        "move the whole struct; moving a field out comes with backlog 2O-03".into(),
                    );
                }
                self.expr(base, false)
            }
            ExprKind::Match { scrut, arms } => {
                // The scrutinee moves when an arm binds an owned payload
                // (Rust moves what the pattern binds; the rest of the value
                // is dropped with it, here at the start of the arm).
                let binds_owned = arms
                    .iter()
                    .any(|a| a.pat.binds.iter().flatten().any(|id| self.owns(&id.name)));
                if binds_owned {
                    self.info.consumed.insert(key(e.span));
                }
                self.expr(scrut, binds_owned);
                let before = self.state.clone();
                let mut joined: Option<BTreeMap<String, St>> = None;
                for arm in arms {
                    self.state = before.clone();
                    let binds: Vec<String> = arm
                        .pat
                        .binds
                        .iter()
                        .flatten()
                        .filter(|id| self.owns(&id.name))
                        .map(|id| id.name.clone())
                        .collect();
                    for b in &binds {
                        self.state.insert(b.clone(), St::Owned);
                    }
                    self.expr(&arm.body, true);
                    let mut owned = Vec::new();
                    for b in binds {
                        if let Some(St::Owned) = self.state.remove(&b) {
                            owned.push(b);
                        }
                    }
                    if !owned.is_empty() {
                        self.info.arm_drops.insert(key(arm.body.span), owned);
                    }
                    let after = std::mem::take(&mut self.state);
                    joined = Some(match joined {
                        None => after,
                        Some(j) => {
                            self.merge(j, after, e.span);
                            std::mem::take(&mut self.state)
                        }
                    });
                }
                self.state = joined.unwrap_or(before);
            }
            ExprKind::Int(_)
            | ExprKind::ByteLit(_)
            | ExprKind::ByteStr(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit
            | ExprKind::Path(_)
            | ExprKind::Error => {}
        }
    }
}
