//! Type invariants (ADR-0060 §2): the deep validity `valid_T(x)` of a
//! value and where the encoder assumes and proves it.
//!
//! `valid_T(x)` is `T`'s invariant (`#[invariant(p)]`, the opaque atom
//! `p(x)`) together with the validity of every field, payload, `Box`
//! content and element (quantified, triggered on the element read). A type
//! parameter's validity is an uninterpreted predicate `|'T.valid|`, so a
//! generic definition is proved once for every instance. A datatype on a
//! cycle (a recursive enum or struct) has a validity function
//! `|T.'valid|` defined by structural recursion; a query gets its ground
//! instances for the applications it holds, two levels deep (as a
//! recursive spec fn's ground instances, ADR-0023): no quantified
//! definition, so no matching loop.
//!
//! New values are assumed valid: parameters at entry, a call's result and
//! the new values behind its `&mut` arguments, locals a loop havocs at its
//! head. Values the function lets go are obligations (E0823): call
//! arguments, the result and `&mut` parameters at every exit, and the
//! havocked locals on entry to a loop and at the end of each iteration.
//! Specifications and ghost code assume nothing (a `proof fn` is ghost,
//! as a `spec fn`: ADR-0060 §2, ADR-0062).

use std::collections::{BTreeMap, BTreeSet};

use super::*;

/// The types a value of `t` directly holds (fields, payloads, elements,
/// what a reference points to), in the encoder's view.
fn holds(t: &TypeKind, tb: &Tables) -> Vec<TypeKind> {
    if is_array(t) {
        return vec![elem_ty(t)];
    }
    if let TypeKind::Ref { inner, .. } = t {
        return vec![inner.kind.clone()];
    }
    if let Some(fs) = struct_fields(t, tb) {
        return fs.into_iter().map(|(_, ft)| ft).collect();
    }
    if let Some(vs) = enum_variants(t, tb) {
        return vs.into_iter().flat_map(|(_, ps)| ps).collect();
    }
    Vec::new()
}

/// The predicate instance of `t`'s invariant: its path, or the instance
/// key `p<A, ..>` of a generic type's (the predicate's parameters are the
/// type's, in order: E0254).
fn inv_target(t: &TypeKind, tb: &Tables) -> Option<String> {
    let TypeKind::Named { path, args } = value_ty(t) else {
        return None;
    };
    let p = tb.adts.get(&path.join("::"))?.inv.as_ref()?;
    if args.is_empty() {
        Some(p.clone())
    } else {
        let args: Vec<TypeKind> = args.iter().map(|a| a.kind.clone()).collect();
        Some(inst_key(p, &args))
    }
}

/// Every generic invariant predicate instance a value of one of `types`
/// can need: (key, predicate, substitution).
pub(super) fn instances(
    tb: &Tables,
    types: &[TypeKind],
) -> Vec<(String, String, BTreeMap<String, TypeKind>)> {
    let mut seen = BTreeSet::new();
    let mut todo: Vec<TypeKind> = types.to_vec();
    let mut out = Vec::new();
    while let Some(t) = todo.pop() {
        if type_depth(&t) > MAX_INST_DEPTH || !seen.insert(crate::check::ty_str(&t)) {
            continue;
        }
        if let TypeKind::Named { path, args } = &t
            && !args.is_empty()
            && let Some(p) = tb.adts.get(&path.join("::")).and_then(|a| a.inv.as_ref())
            && let Some(sig) = tb.fns.get(p)
        {
            let args: Vec<TypeKind> = args.iter().map(|a| a.kind.clone()).collect();
            let sub = sig
                .generics
                .iter()
                .map(|g| g.name.clone())
                .zip(args.iter().cloned())
                .collect();
            out.push((inst_key(p, &args), p.clone(), sub));
        }
        todo.extend(holds(&t, tb));
    }
    out
}

impl Prog<'_> {
    /// Whether a value of `t` has a validity other than `true`: some type it
    /// holds, itself included, has an invariant or is a type parameter.
    pub(super) fn needs_valid(&self, t: &TypeKind) -> bool {
        let key = crate::check::ty_str(t);
        if let Some(&b) = self.valid_memo.borrow().get(&key) {
            return b;
        }
        let mut seen = BTreeSet::new();
        let mut todo = vec![t.clone()];
        let mut found = false;
        while let Some(u) = todo.pop() {
            if matches!(u, TypeKind::Param(_)) || inv_target(&u, self.t).is_some() {
                found = true;
                break;
            }
            if seen.insert(crate::check::ty_str(&u)) {
                todo.extend(holds(&u, self.t));
            }
        }
        self.valid_memo.borrow_mut().insert(key, found);
        found
    }

    /// Whether `t` is a datatype (an enum, or a struct that holds itself
    /// through structs) that holds itself: its validity is a function.
    fn rec_valid(&self, t: &TypeKind) -> bool {
        let t = value_ty(t);
        if !dt_struct(t, self.t) && enum_variants(t, self.t).is_none() {
            return false;
        }
        let me = crate::check::ty_str(t);
        let mut seen = BTreeSet::new();
        let mut todo = holds(t, self.t);
        while let Some(u) = todo.pop() {
            let k = crate::check::ty_str(value_ty(&u));
            if k == me {
                return true;
            }
            if seen.insert(crate::check::ty_str(&u)) {
                todo.extend(holds(&u, self.t));
            }
        }
        false
    }
}

/// Where a value leaves the function (an E0823 obligation), for its message.
pub(super) enum Boundary<'a> {
    Arg(&'a Expr, &'a str),
    Result(&'a str),
    MutParam(&'a str, &'a str),
    LoopEntry(&'a str),
    LoopBack(&'a str),
}

impl<'p, 'q> Enc<'p, 'q> {
    /// The conjuncts of `valid_T(v)` for `v` of type `t` (none when `t`
    /// has no invariant anywhere).
    pub(super) fn valid(&mut self, v: &Val, t: &TypeKind) -> Vec<String> {
        if !self.prog.needs_valid(t) {
            return Vec::new();
        }
        let tb = self.prog.t;
        if let TypeKind::Ref { inner, .. } = t
            && !is_array(t)
        {
            return self.valid(v, &inner.kind.clone());
        }
        if let TypeKind::Param(n) = t {
            let f = sym(&format!("'{n}.valid"));
            if self.valid_decl.insert(f.clone()) {
                self.decls
                    .push(format!("(declare-fun {f} ({}) Bool)", sort_of(t)));
            }
            return vec![format!("({f} {})", v.s())];
        }
        if is_array(t) {
            let e = elem_ty(t);
            // Bound by nesting depth: the same value has the same text, so
            // a fact already on the path is not proved again.
            let k = sym(&format!("valid!k{}", self.valid_depth));
            let (len, elem, pats) = match v {
                Val::A { elems, len } => {
                    let x = format!("(select {elems} {k})");
                    (len.clone(), Val::S(x.clone()), vec![x])
                }
                Val::Arr { elem, len } => {
                    let mut ls = Vec::new();
                    flatten_elem(elem, &mut ls);
                    let pats = ls.iter().map(|l| format!("(select {l} {k})")).collect();
                    (len.clone(), select_val(elem, &k), pats)
                }
                _ => return Vec::new(),
            };
            self.valid_depth += 1;
            let body = self.valid(&elem, &e);
            self.valid_depth -= 1;
            if body.is_empty() {
                return Vec::new();
            }
            let pats: Vec<String> = pats.iter().map(|p| format!(":pattern ({p})")).collect();
            return vec![format!(
                "(forall (({k} Int)) (! (=> (and (<= 0 {k}) (< {k} {len})) {}) {}))",
                and(&body),
                pats.join(" ")
            )];
        }
        if let (Some(fs), Val::Struct(xs)) = (flat_fields(t, tb), v) {
            let mut out = self.inv_app(v, t);
            for ((_, ft), x) in fs.iter().zip(xs) {
                out.extend(self.valid(x, ft));
            }
            return out;
        }
        let Val::S(c) = v else {
            return Vec::new();
        };
        if self.prog.rec_valid(t) {
            let f = self.valid_fn(t);
            return vec![format!("({f} {c})")];
        }
        self.valid_dt(c, t)
    }

    /// The invariant atom of `v` of type `t`, if `t` has one.
    fn inv_app(&mut self, v: &Val, t: &TypeKind) -> Vec<String> {
        let Some(target) = inv_target(t, self.prog.t) else {
            return Vec::new();
        };
        assert!(
            self.prog.t.fns.contains_key(&target),
            "invariant instance `{target}` not prepared"
        );
        let mut flat = Vec::new();
        flatten(v, &mut flat);
        let f = sym(&target);
        vec![if flat.is_empty() {
            f
        } else {
            format!("({f} {})", flat.join(" "))
        }]
    }

    /// The validity of datatype value `c` of type `t`, one level: its
    /// invariant, and each variant's payloads (a recursive struct's fields)
    /// when it is that variant.
    fn valid_dt(&mut self, c: &str, t: &TypeKind) -> Vec<String> {
        let tb = self.prog.t;
        let mut out = self.inv_app(&Val::S(c.to_string()), t);
        if dt_struct(t, tb) {
            let fs = struct_fields(t, tb).expect("a struct");
            for (x, (_, ft)) in dt_fields(value_ty(t), c, tb).iter().zip(&fs) {
                out.extend(self.valid(x, ft));
            }
            return out;
        }
        let vs = enum_variants(t, tb).unwrap_or_default();
        // A value written with its constructor (`None`, `Some(x)`) is that
        // variant: its payloads are the arguments.
        let built = vs.iter().position(|(name, _)| {
            let k = ctor(t, name);
            c == k || c.starts_with(&format!("({k} "))
        });
        for (i, (name, ps)) in vs.iter().enumerate() {
            if built.is_some_and(|b| b != i) || !ps.iter().any(|p| self.prog.needs_valid(p)) {
                continue;
            }
            // The payloads of a constructor application are its arguments
            // (its leaves, in `flatten` order); otherwise the selectors.
            let args: Option<Vec<String>> = (built == Some(i))
                .then(|| match smt::parse(c) {
                    Some(smt::Sexp::List(xs)) => Some(xs[1..].iter().map(smt::print).collect()),
                    _ => None,
                })
                .flatten();
            let vals: Vec<Val> = match args {
                Some(args) => {
                    let mut it = args.into_iter();
                    ps.iter().map(|pt| build(pt, tb, &mut it)).collect()
                }
                None => self.payload(t, name, c).0,
            };
            let mut inner = Vec::new();
            for (pv, pt) in vals.iter().zip(ps) {
                inner.extend(self.valid(pv, pt));
            }
            if inner.is_empty() {
                continue;
            }
            if built == Some(i) {
                out.extend(inner);
            } else {
                out.push(format!(
                    "(=> ((_ is {}) {c}) {})",
                    ctor(t, name),
                    and(&inner)
                ));
            }
        }
        out
    }

    /// The validity function of recursive datatype `t`, declared and
    /// defined (for ground instances) on first use.
    fn valid_fn(&mut self, t: &TypeKind) -> String {
        let t = value_ty(t).clone();
        let f = sym(&format!("{}.'valid", crate::check::ty_str(&t)));
        if self.valid_decl.insert(f.clone()) {
            self.decls
                .push(format!("(declare-fun {f} ({}) Bool)", sort_of(&t)));
            let x = sym("valid!x");
            let body = self.valid_dt(&x, &t);
            self.valid_defs.push(AxiomParts {
                sym: f.clone(),
                params: vec![x],
                guard: "true".into(),
                body: and(&body),
            });
        }
        f
    }

    /// Assumes `v` (of type `t`) valid: a new value of code.
    pub(super) fn assume_valid(&mut self, v: &Val, t: &TypeKind) {
        if self.ghost {
            return;
        }
        let facts = self.valid(v, t);
        self.pc.extend(facts);
    }

    /// The obligation that `v` (of type `t`) is valid where it leaves the
    /// function at `span` (E0823); the counterexample shows `v`.
    pub(super) fn oblige_valid(&mut self, span: Span, at: Boundary, v: &Val, t: &TypeKind) {
        if self.ghost {
            return;
        }
        let goals: Vec<String> = self
            .valid(v, t)
            .into_iter()
            .filter(|g| !self.pc.contains(g))
            .collect();
        if goals.is_empty() {
            return;
        }
        let ty = crate::check::ty_str(value_ty(t));
        let (what, shown) = match at {
            Boundary::Arg(a, callee) => (
                format!("argument `{}` of `{callee}`", crate::fmt::expr_text(a)),
                "argument".to_string(),
            ),
            Boundary::Result(how) => (format!("the result {how}"), "result".to_string()),
            Boundary::MutParam(p, how) => (
                format!("`*{p}` (a `&mut` parameter) {how}"),
                format!("*{p}"),
            ),
            Boundary::LoopEntry(x) => (
                format!("`{x}` on entry to the loop"),
                format!("{x} on entry"),
            ),
            Boundary::LoopBack(x) => (
                format!("`{x}` when the loop goes back to its head"),
                format!("{x} after the iteration"),
            ),
        };
        let n = self.obls.len();
        self.oblige(
            "E0823",
            span,
            format!("type invariant of `{ty}` may not hold for {what}"),
            and(&goals),
        );
        if let Some(o) = self.obls.get_mut(n) {
            o.shown = Some((shown, v.clone(), t.clone()));
        }
    }

    /// The obligations at an exit of the function (ADR-0060 §2, with the
    /// postconditions): its result and the current value of each `&mut`
    /// parameter are valid.
    pub(super) fn exit_valid(&mut self, result: &Val, span: Span, how: &str) {
        let f = self.prog.defs[&self.path].f;
        if f.kind != FnKind::Exec {
            return;
        }
        let ret = crate::check::subst(&self.prog.t.fns[&self.path].ret, &self.tsub);
        self.oblige_valid(span, Boundary::Result(how), result, &ret);
        for p in &f.params {
            if let TypeKind::Ref { mutable: true, .. } = p.ty.kind
                && let Some(v) = self.vars.get(&p.name.name).cloned()
            {
                let ty = self.local_ty(&p.name.name);
                self.oblige_valid(span, Boundary::MutParam(&p.name.name, how), &v, &ty);
            }
        }
    }

    /// The locals a loop havocs that have a validity, with their types.
    pub(super) fn loop_valid_vars(&self, modified: &BTreeSet<String>) -> Vec<(String, TypeKind)> {
        if self.ghost {
            return Vec::new();
        }
        modified
            .iter()
            .filter(|v| self.vars.contains_key(*v))
            .map(|v| (v.clone(), self.local_ty(v)))
            .filter(|(_, t)| self.prog.needs_valid(t))
            .collect()
    }
}
