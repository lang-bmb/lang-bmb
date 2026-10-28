//! Ownership of values that move (ADR-0031 §2, ADR-0047 §2, backlogs
//! 2G-02 and 2O-01): a `Vec`, a `Box`, and the data types that hold one
//! (`check::is_move`).
//!
//! Such a local is moved by a value use: a `let` initializer, the right
//! side of an assignment, a call argument, a block's value, the scrutinee
//! of a `match` that binds an owned payload. Reading through it (`v.len()`,
//! `v[i]`, `&v`, `*b`, `b.f`) is not a move. A use of a moved local is
//! E0413; assigning it again makes it owned again.
//!
//! A field of a local struct can be moved out on its own (a partial move,
//! backlog 2O-03): the state is kept per field path (`s.buf`), the moved
//! field cannot be used until it is assigned again, the whole local cannot
//! be used meanwhile (E0433), and at the end of the block only the fields
//! still owned are dropped. Moving out of a place behind a reference or in
//! a `Box` (`*b`, `r.f`) is E0413.
//!
//! At every join (the branches of `if` and `match`, the right side of `&&`,
//! `||` and `==>`, and a loop head against the end of its body) a local is
//! owned on all paths or moved on all paths; "maybe moved" is E0414. So the
//! place where each value is dropped is fixed by the program text, and the
//! pass records it for code generation: the locals still owned at the end
//! of each block and arm, and the assignments that replace an owned value.
//!
//! A `return` (ADR-0049 §4) ends its path: the values still owned there
//! are dropped on that exit (`MoveInfo::exits`), and a join takes only the
//! paths that reach it.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::check::{Adt, is_move_in};
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
    /// Assignments (by statement span) to a partially moved place: the
    /// field paths still owned, dropped after the right side is evaluated.
    pub partial_replaces: BTreeMap<Key, Vec<String>>,
    /// Per `return` (by its span): the owned locals and parameters (or
    /// their fields still owned) dropped on that exit.
    pub exits: BTreeMap<Key, Vec<String>>,
}

/// A loop a `break` or `continue` may leave.
struct MvLoop {
    label: Option<String>,
    /// The locals and parameters live before it (the ones it keeps).
    outer: BTreeSet<String>,
    /// The states its `break`s leave with (joined at its exit).
    breaks: Vec<BTreeMap<String, St>>,
    /// The states its `continue`s go back with (checked at its head).
    continues: Vec<BTreeMap<String, St>>,
}

struct Mv<'a> {
    /// Canonical types of the function's parameters and locals (names are
    /// unique in a function, E0303).
    locals: &'a BTreeMap<String, TypeKind>,
    types: &'a BTreeMap<Key, TypeKind>,
    adts: &'a BTreeMap<String, Adt>,
    /// The type parameters without `Copy` (their values move).
    moving: &'a BTreeSet<String>,
    state: BTreeMap<String, St>,
    /// The current path has left the function or a loop (`return`,
    /// `break`, `continue`): what follows on it does not run, and a join
    /// ignores it.
    dead: bool,
    /// The loops around the code being checked, innermost last.
    loops: Vec<MvLoop>,
    diags: Vec<Diag>,
    info: MoveInfo,
    /// Inside a `proof { .. }` block (ADR-0062): ghost code reads values
    /// and moves none (it is erased).
    ghost: bool,
}

/// Checks the moves of an executable function and returns its diagnostics
/// and drop points.
pub fn check(
    f: &FnDef,
    locals: &BTreeMap<String, TypeKind>,
    types: &BTreeMap<Key, TypeKind>,
    adts: &BTreeMap<String, Adt>,
    moving: &BTreeSet<String>,
) -> (Vec<Diag>, MoveInfo) {
    let mut m = Mv {
        locals,
        types,
        adts,
        moving,
        state: BTreeMap::new(),
        dead: false,
        loops: Vec::new(),
        diags: Vec::new(),
        info: MoveInfo::default(),
        ghost: false,
    };
    for p in &f.params {
        if m.owns(&p.name.name) {
            m.state.insert(p.name.name.clone(), St::Owned);
        }
    }
    m.block(&f.body);
    if m.dead {
        return (m.diags, m.info);
    }
    let roots: Vec<String> = m
        .state
        .keys()
        .filter(|n| !n.contains('.'))
        .cloned()
        .collect();
    let params: Vec<String> = roots.iter().flat_map(|n| m.drop_paths(n)).collect();
    if !params.is_empty() {
        m.info
            .drops
            .entry(key(f.body.span))
            .or_default()
            .extend(params);
    }
    (m.diags, m.info)
}

/// The state of field path `p` in `state`: moved when it or a prefix is.
fn status_in(state: &BTreeMap<String, St>, p: &str) -> Option<St> {
    let mut cur = String::new();
    let mut found = None;
    for (i, part) in p.split('.').enumerate() {
        if i > 0 {
            cur.push('.');
        }
        cur.push_str(part);
        match state.get(&cur) {
            Some(St::Moved) => return Some(St::Moved),
            Some(St::Owned) => found = Some(St::Owned),
            None if i == 0 => return None,
            None => {}
        }
    }
    found
}

/// The type of field `f` of struct type `t`.
fn field_ty(t: &TypeKind, f: &str, adts: &BTreeMap<String, Adt>) -> Option<TypeKind> {
    let TypeKind::Named { path, .. } = t else {
        return None;
    };
    let a = adts.get(&path.join("::"))?;
    let crate::check::AdtKind::Struct(fs) = &a.kind else {
        return None;
    };
    let m = crate::check::adt_subst(a, t);
    fs.iter()
        .find(|(n, _)| n == f)
        .map(|(_, ft)| crate::check::subst(ft, &m))
}

impl Mv<'_> {
    /// Whether local `name` holds a value that moves.
    fn owns(&self, name: &str) -> bool {
        self.locals
            .get(name)
            .is_some_and(|t| is_move_in(t, self.adts, self.moving))
    }

    /// Whether the value of expression `e` moves.
    fn moves_ty(&self, e: &Expr) -> bool {
        self.types
            .get(&key(e.span))
            .is_some_and(|t| is_move_in(t, self.adts, self.moving))
    }

    /// E0413 for a move out of a place that is not a whole local.
    fn move_out(&mut self, e: &Expr, msg: String, fix: String, replacement: Option<String>) {
        self.diags
            .push(Diag::new("E0413", e.span, msg).fix(fix, replacement));
    }

    /// The state of field path `p` (a local, or `s.f.g`): moved when it or
    /// a prefix is; `None` for an untracked local.
    fn status(&self, p: &str) -> Option<St> {
        status_in(&self.state, p)
    }

    /// A moved field path under `p` (`p.f..`), if any.
    fn moved_part(&self, p: &str) -> Option<String> {
        let pre = format!("{p}.");
        self.state
            .iter()
            .find(|(k, s)| k.starts_with(&pre) && matches!(s, St::Moved))
            .map(|(k, _)| k.clone())
    }

    /// The field path of `e` when it names a part of a local struct by
    /// value: `s`, `s.f`, `s.f.g` (not through a reference or a `Box`).
    fn path_of(&self, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Var(id) => self.locals.contains_key(&id.name).then(|| id.name.clone()),
            ExprKind::Field { base, name } => {
                let bp = self.path_of(base)?;
                matches!(self.path_ty(&bp)?, TypeKind::Named { .. })
                    .then(|| format!("{bp}.{}", name.name))
            }
            _ => None,
        }
    }

    /// The type of field path `p`.
    fn path_ty(&self, p: &str) -> Option<TypeKind> {
        let mut parts = p.split('.');
        let mut t = self.locals.get(parts.next()?)?.clone();
        for f in parts {
            t = field_ty(&t, f, self.adts)?;
        }
        Some(t)
    }

    /// The paths to drop for the owned local or field `p`: itself, or, when
    /// parts of it were moved, its fields still owned that own memory.
    fn drop_paths(&self, p: &str) -> Vec<String> {
        match self.status(p) {
            Some(St::Owned) => {}
            _ => return Vec::new(),
        }
        if self.moved_part(p).is_none() {
            return vec![p.to_string()];
        }
        let Some(TypeKind::Named { path, .. }) = self.path_ty(p) else {
            return Vec::new();
        };
        let names: Vec<String> = match self.adts.get(&path.join("::")).map(|a| &a.kind) {
            Some(crate::check::AdtKind::Struct(fs)) => fs.iter().map(|(n, _)| n.clone()).collect(),
            _ => Vec::new(),
        };
        names
            .iter()
            .map(|n| format!("{p}.{n}"))
            .filter(|q| {
                self.path_ty(q)
                    .is_some_and(|t| is_move_in(&t, self.adts, self.moving))
            })
            .flat_map(|q| self.drop_paths(&q))
            .collect()
    }

    /// Forgets the field states under `p` (it is assigned whole).
    fn clear(&mut self, p: &str) {
        let pre = format!("{p}.");
        self.state.retain(|k, _| !k.starts_with(&pre));
    }

    /// E0433: `p` is used whole after its field `part` was moved.
    fn partly_moved(&mut self, p: &str, part: &str, span: Span) {
        self.diags.push(
            Diag::new(
                "E0433",
                span,
                format!("`{p}` is used after its field `{part}` was moved"),
            )
            .fix(
                format!("assign `{part}` again before this use, or use the other fields"),
                None,
            ),
        );
        // One report: the use counts as whole from here.
        self.clear(p);
    }

    /// A use of field path `p` (a local or a field of one), moving it when
    /// `moves`.
    fn use_path(&mut self, p: &str, span: Span, moves: bool) {
        match self.status(p) {
            Some(St::Owned) => {
                if let Some(part) = self.moved_part(p) {
                    self.partly_moved(p, &part, span);
                }
                if moves {
                    self.state.insert(p.to_string(), St::Moved);
                }
            }
            Some(St::Moved) => {
                self.diags.push(
                    Diag::new("E0413", span, format!("`{p}` is used after it was moved"))
                        .fix(format!("assign `{p}` again before this use"), None),
                );
                let st = if moves { St::Moved } else { St::Owned };
                self.state.insert(p.to_string(), st);
            }
            None => {}
        }
    }

    fn use_var(&mut self, id: &Ident, span: Span, moves: bool) {
        let moves = moves && !self.ghost;
        if let Some(part) = self.moved_part(&id.name)
            && matches!(self.status(&id.name), Some(St::Owned))
        {
            self.partly_moved(&id.name, &part, span);
        }
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
        let keys: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
        for n in keys {
            let root = n.split('.').next().unwrap_or(n);
            if !(a.contains_key(root) && b.contains_key(root)) {
                continue;
            }
            let (Some(sa), Some(sb)) = (status_in(&a, n), status_in(&b, n)) else {
                continue;
            };
            let st = if sa.same(sb) {
                sa
            } else {
                self.maybe_moved(n, span, "on one path and not on another");
                St::Moved
            };
            // A field is kept only when moved (owned is inherited).
            if !n.contains('.') || matches!(st, St::Moved) {
                out.insert(n.clone(), st);
            }
        }
        self.state = out;
    }

    /// The join of two paths, each with its state and whether it left the
    /// function: a path that left does not reach the join.
    fn join(
        &mut self,
        a: (BTreeMap<String, St>, bool),
        b: (BTreeMap<String, St>, bool),
        span: Span,
    ) {
        match (a, b) {
            ((sa, true), (_, true)) => {
                self.state = sa;
                self.dead = true;
            }
            ((_, true), (sb, false)) => {
                self.state = sb;
                self.dead = false;
            }
            ((sa, false), (_, true)) => {
                self.state = sa;
                self.dead = false;
            }
            ((sa, false), (sb, false)) => {
                self.dead = false;
                self.merge(sa, sb, span);
            }
        }
    }

    /// The states that go back to the head of a loop (the end of its body
    /// when reached, and each `continue`) against the head: a local owned on
    /// one and moved on the other is E0414 (and moved after the loop).
    fn loop_backs(
        &mut self,
        head: &BTreeMap<String, St>,
        mut backs: Vec<BTreeMap<String, St>>,
        out: &mut BTreeMap<String, St>,
        span: Span,
    ) {
        // A body that left the loop or the function does not loop back.
        let end = std::mem::take(&mut self.state);
        if !std::mem::replace(&mut self.dead, false) {
            backs.push(end);
        }
        for back in backs {
            let keys: BTreeSet<String> = head.keys().chain(back.keys()).cloned().collect();
            for n in &keys {
                let root = n.split('.').next().unwrap_or(n);
                if !head.contains_key(root) {
                    continue;
                }
                if let (Some(s0), Some(s1)) = (status_in(head, n), status_in(&back, n))
                    && !s0.same(s1)
                {
                    let how = if matches!(s0, St::Owned) {
                        "in the loop and not assigned again before the next iteration"
                    } else {
                        "before the loop and assigned only inside it"
                    };
                    self.maybe_moved(n, span, how);
                    out.insert(n.clone(), St::Moved);
                }
            }
        }
    }

    /// A `break` or `continue` at `e` to loop `k`: the locals declared in the
    /// loop and still owned are dropped on that exit; the state of the rest
    /// goes to the loop's exit or head.
    fn jump(&mut self, e: &Expr, label: &Option<Ident>, brk: bool) {
        let Some(k) = (match label {
            Some(l) => self
                .loops
                .iter()
                .rposition(|c| c.label.as_deref() == Some(l.name.as_str())),
            None => self.loops.len().checked_sub(1),
        }) else {
            return;
        };
        let outer = self.loops[k].outer.clone();
        let inner: Vec<String> = self
            .state
            .keys()
            .filter(|n| !n.contains('.') && !outer.contains(*n))
            .cloned()
            .collect();
        let owned: Vec<String> = inner.iter().flat_map(|n| self.drop_paths(n)).collect();
        if !owned.is_empty() {
            self.info.exits.insert(key(e.span), owned);
        }
        let kept: BTreeMap<String, St> = self
            .state
            .iter()
            .filter(|(n, _)| outer.contains(n.split('.').next().unwrap_or(n)))
            .map(|(n, s)| (n.clone(), *s))
            .collect();
        if brk {
            self.loops[k].breaks.push(kept);
        } else {
            self.loops[k].continues.push(kept);
        }
        self.dead = true;
    }

    /// The owned paths of every live local and parameter (a `return`).
    fn owned_now(&self) -> Vec<String> {
        let roots: Vec<String> = self
            .state
            .keys()
            .filter(|n| !n.contains('.'))
            .cloned()
            .collect();
        roots.iter().flat_map(|n| self.drop_paths(n)).collect()
    }

    /// Whether place `e` is reached through a `&mut` parameter (not
    /// through a `Box` of a local, which can be moved whole).
    fn behind_mut(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Var(id) => matches!(
                self.locals.get(&id.name),
                Some(TypeKind::Ref { mutable: true, .. })
            ),
            ExprKind::Field { base, .. } | ExprKind::Deref(base) | ExprKind::Index { base, .. } => {
                self.behind_mut(base)
            }
            _ => false,
        }
    }

    /// E0239: an owned value moved out from behind a `&mut`; the fix puts
    /// a new value in its place with `std::mem::replace` (ADR-0047 §2).
    fn out_of_mut(&mut self, e: &Expr) {
        let place = crate::fmt::expr_text(e);
        let fresh = match self.types.get(&key(e.span)) {
            Some(TypeKind::Vec(_)) => Some("Vec::new()"),
            Some(TypeKind::Named { path, .. }) if path.len() == 1 && path[0] == "Option" => {
                Some("None")
            }
            _ => None,
        };
        let d = Diag::new(
            "E0239",
            e.span,
            format!("cannot move `{place}` out from behind a `&mut`"),
        );
        let d = match fresh {
            Some(v) => d.fix(
                format!("take it and leave `{v}` in its place"),
                Some(format!("std::mem::replace(&mut {place}, {v})")),
            ),
            None => d.fix(
                format!("take it with `std::mem::replace(&mut {place}, new)`, or exchange it with `std::mem::swap`"),
                None,
            ),
        };
        self.diags.push(d);
    }

    /// The drop of an assignment to owned place `p` at `span`: the old
    /// value whole, or its fields still owned when parts were moved.
    fn replace_at(&mut self, p: &str, span: Span) {
        if !matches!(self.status(p), Some(St::Owned)) {
            return;
        }
        if self.moved_part(p).is_none() {
            self.info.replaces.insert(key(span));
        } else {
            let rest = self.drop_paths(p);
            if !rest.is_empty() {
                self.info.partial_replaces.insert(key(span), rest);
            }
        }
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
        // What follows a `return` does not run.
        for s in &b.stmts {
            if self.dead {
                break;
            }
            self.stmt(s);
        }
        if let Some(t) = &b.tail
            && !self.dead
        {
            self.expr(t, true);
        }
        let inner: Vec<String> = self
            .state
            .keys()
            .filter(|n| !n.contains('.') && !outer.contains(*n))
            .cloned()
            .collect();
        let mut owned = Vec::new();
        for n in inner {
            owned.extend(self.drop_paths(&n));
            self.clear(&n);
            self.state.remove(&n);
        }
        if !owned.is_empty() && !self.dead {
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
                        self.replace_at(&id.name, *span);
                        self.clear(&id.name);
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
                    // An owned element: the old one is dropped.
                    ExprKind::Index { base, index } => {
                        if self.moves_ty(target) {
                            self.info.replaces.insert(key(*span));
                        }
                        self.expr(base, false);
                        self.expr(index, true);
                    }
                    // A field of a local: its own state (it may have been
                    // moved out, and the struct may not be).
                    ExprKind::Field { .. }
                        if self.path_of(target).is_some_and(|p| {
                            let root = p.split('.').next().unwrap_or(&p).to_string();
                            self.status(&root).is_some()
                        }) =>
                    {
                        let p = self.path_of(target).expect("checked");
                        let parent = &p[..p.rfind('.').expect("a field path")];
                        if let Some(St::Moved) = self.status(parent) {
                            self.diags.push(
                                Diag::new(
                                    "E0413",
                                    target.span,
                                    format!("`{parent}` is moved: its field cannot be assigned"),
                                )
                                .fix(format!("assign `{parent}` whole"), None),
                            );
                        } else if self.moves_ty(target) {
                            self.replace_at(&p, *span);
                            self.clear(&p);
                            self.state.remove(&p);
                        }
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
                label,
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
                self.loops.push(MvLoop {
                    label: label.as_ref().map(|l| l.name.clone()),
                    outer: head.keys().cloned().collect(),
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                self.block(body);
                let lp = self.loops.pop().expect("pushed");
                let mut out = exit;
                self.loop_backs(&head, lp.continues, &mut out, *span);
                self.state = out;
                for b in lp.breaks {
                    let cur = std::mem::take(&mut self.state);
                    self.merge(cur, b, *span);
                }
            }
            // As `while` (ADR-0049 §2); the bounds are integers.
            Stmt::For {
                label,
                lo,
                hi,
                invariants,
                body,
                span,
                ..
            } => {
                self.expr(lo, false);
                self.expr(hi, false);
                let head = self.state.clone();
                for inv in invariants {
                    self.expr(inv, false);
                }
                self.loops.push(MvLoop {
                    label: label.as_ref().map(|l| l.name.clone()),
                    outer: head.keys().cloned().collect(),
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                self.block(body);
                let lp = self.loops.pop().expect("pushed");
                let mut out = head.clone();
                self.loop_backs(&head, lp.continues, &mut out, *span);
                self.state = out;
                for b in lp.breaks {
                    let cur = std::mem::take(&mut self.state);
                    self.merge(cur, b, *span);
                }
            }
            Stmt::Assert { cond, .. } => self.expr(cond, false),
            Stmt::Expr(e) => self.expr(e, true),
            Stmt::Proof { body, .. } => {
                let ghost = std::mem::replace(&mut self.ghost, true);
                self.block(body);
                self.ghost = ghost;
            }
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
                if moves && self.moves_ty(e) {
                    self.move_out(
                        e,
                        format!("cannot move out of `{}`", crate::fmt::expr_text(e)),
                        "read through it (`v[i].len()`, `&v[i]`), take it with `pop`, or exchange it with `std::mem::replace(&mut v[i], new)`".into(),
                        None,
                    );
                }
                self.expr(base, false);
                self.expr(index, true);
            }
            ExprKind::Unary(_, x) => self.expr(x, true),
            ExprKind::Binary(BinOp::And | BinOp::Or | BinOp::Implies, l, r) => {
                self.expr(l, true);
                let skip = self.state.clone();
                self.expr(r, true);
                let taken = std::mem::take(&mut self.state);
                let dead = std::mem::replace(&mut self.dead, false);
                self.join((skip, false), (taken, dead), e.span);
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
                let then_dead = std::mem::replace(&mut self.dead, false);
                if let Some(el) = els {
                    self.expr(el, true);
                }
                let after_else = std::mem::take(&mut self.state);
                let else_dead = std::mem::replace(&mut self.dead, false);
                self.join((after_then, then_dead), (after_else, else_dead), e.span);
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
                if moves && self.moves_ty(e) && self.behind_mut(e) {
                    self.out_of_mut(e);
                } else if moves && self.moves_ty(e) {
                    let fix = if matches!(self.types.get(&key(e.span)), Some(TypeKind::Vec(_))) {
                        format!("move `{}.clone()` instead", crate::fmt::expr_text(x))
                    } else {
                        "read through it (`*b` of a copyable value, `b.f`) or move the whole value"
                            .into()
                    };
                    self.move_out(
                        e,
                        format!("cannot move out of `{}`", crate::fmt::expr_text(e)),
                        fix,
                        None,
                    );
                }
                self.expr(x, false);
            }
            // A field of a local struct: moved on its own, or read.
            ExprKind::Field { .. }
                if self.path_of(e).is_some_and(|p| {
                    let root = p.split('.').next().unwrap_or(&p).to_string();
                    self.status(&root).is_some()
                }) =>
            {
                let p = self.path_of(e).expect("checked");
                let owned = self.moves_ty(e);
                self.use_path(&p, e.span, moves && owned);
            }
            ExprKind::Field { base, .. } => {
                if moves && self.moves_ty(e) && self.behind_mut(e) {
                    self.out_of_mut(e);
                } else if moves && self.moves_ty(e) {
                    self.move_out(
                        e,
                        format!(
                            "cannot move the field `{}` out of a place behind a shared reference or in a `Box`",
                            crate::fmt::expr_text(e)
                        ),
                        "read through it, or move the whole value".into(),
                        None,
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
                // Binding owned payloads out of a place behind a reference or
                // out of a field moves out of it: match a reference instead
                // (Rust's default binding mode, ADR-0055). A field of a local
                // struct moves on its own (a partial move).
                let local_field = self.path_of(scrut).is_some_and(|p| {
                    let root = p.split('.').next().unwrap_or(&p).to_string();
                    self.status(&root).is_some()
                });
                match &scrut.kind {
                    ExprKind::Deref(x) | ExprKind::Field { base: x, .. }
                        if binds_owned && self.moves_ty(scrut) && !local_field =>
                    {
                        let by_ref = matches!(&scrut.kind, ExprKind::Deref(_))
                            && matches!(self.types.get(&key(x.span)), Some(TypeKind::Ref { .. }));
                        let fix = if by_ref {
                            crate::fmt::expr_text(x)
                        } else {
                            format!("&{}", crate::fmt::expr_text(scrut))
                        };
                        self.move_out(
                            scrut,
                            format!(
                                "the patterns move owned payloads out of `{}`",
                                crate::fmt::expr_text(scrut)
                            ),
                            format!("match a reference, `match {fix}`: each binding refers to its payload"),
                            Some(fix.clone()),
                        );
                        self.expr(scrut, false);
                    }
                    _ => self.expr(scrut, binds_owned),
                }
                let before = self.state.clone();
                let mut joined: Option<(BTreeMap<String, St>, bool)> = None;
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
                    let dead = std::mem::replace(&mut self.dead, false);
                    if !owned.is_empty() && !dead {
                        self.info.arm_drops.insert(key(arm.body.span), owned);
                    }
                    let after = std::mem::take(&mut self.state);
                    joined = Some(match joined {
                        None => (after, dead),
                        Some(j) => {
                            self.join(j, (after, dead), e.span);
                            let d = std::mem::replace(&mut self.dead, false);
                            (std::mem::take(&mut self.state), d)
                        }
                    });
                }
                match joined {
                    Some((st, dead)) => {
                        self.state = st;
                        self.dead = dead;
                    }
                    None => self.state = before,
                }
            }
            ExprKind::Loop {
                label,
                invariants,
                body,
                ..
            } => {
                let head = self.state.clone();
                for inv in invariants {
                    self.expr(inv, false);
                }
                self.loops.push(MvLoop {
                    label: label.as_ref().map(|l| l.name.clone()),
                    outer: head.keys().cloned().collect(),
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                self.block(body);
                let lp = self.loops.pop().expect("pushed");
                let mut ignored = head.clone();
                self.loop_backs(&head, lp.continues, &mut ignored, e.span);
                // Only the `break`s leave a `loop`; with none it does not
                // finish.
                let mut breaks = lp.breaks.into_iter();
                match breaks.next() {
                    None => {
                        self.state = ignored;
                        self.dead = true;
                    }
                    Some(first) => {
                        // The head's verdicts (E0414 marks) hold at the exit.
                        self.state = first;
                        for (n, st) in &ignored {
                            if matches!(st, St::Moved)
                                && self.state.contains_key(n.split('.').next().unwrap_or(n))
                            {
                                self.state.insert(n.clone(), St::Moved);
                            }
                        }
                        for b in breaks {
                            let cur = std::mem::take(&mut self.state);
                            self.merge(cur, b, e.span);
                        }
                    }
                }
            }
            ExprKind::Break { label, value } => {
                if let Some(v) = value {
                    self.expr(v, true);
                }
                self.jump(e, label, true);
            }
            ExprKind::Continue { label } => self.jump(e, label, false),
            // `x?` (ADR-0049 §4): the operand moves (its payload is the
            // value, or goes out in the returned `None`/`Err`); on the
            // return edge everything else still owned is dropped.
            ExprKind::Try(x) => {
                if self.moves_ty(x) {
                    self.info.consumed.insert(key(e.span));
                }
                self.expr(x, true);
                let owned = self.owned_now();
                if !owned.is_empty() {
                    self.info.exits.insert(key(e.span), owned);
                }
            }
            // The value moves out; everything else still owned is dropped
            // on this exit.
            ExprKind::Return(x) => {
                if let Some(x) = x {
                    self.expr(x, true);
                }
                let owned = self.owned_now();
                if !owned.is_empty() {
                    self.info.exits.insert(key(e.span), owned);
                }
                self.dead = true;
            }
            ExprKind::Int(..)
            | ExprKind::ByteLit(_)
            | ExprKind::ByteStr(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit
            | ExprKind::Path(_)
            | ExprKind::Error => {}
        }
    }
}
