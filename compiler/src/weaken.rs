//! Contract weakening between two versions of a program (backlog 2D-19,
//! DESIGN.md §8.2 "계약 조정", hypothesis H-GAME): `bmb weaken OLD NEW`.
//!
//! **Matching.** Executable functions and `proof fn`s (free and in `impl T`
//! blocks) are matched by path; the rest by identity `id` (ADR-0059: a pure
//! rename or move); the rest by the identity of the program with every
//! `requires`, `ensures` and `#[uses]` removed (a rename together with a
//! contract change: the `id` holds the contract, so it changes with it).
//! Each of the last two only where the hash is unique on both sides.
//!
//! **Verdicts.** Old contract (R0, E0), new (R1, E1). Forward, by the
//! prover's refinement obligations (`verify::contract_refinements`, the
//! trait refinement of ADR-0046 §4): R0 ⇒ each clause of R1, and
//! R0 ∧ E1 ⇒ each clause of E0, over the parameters, the result and the
//! values behind `&mut` parameters. Backward: R1 ⇒ each clause of R0, and
//! R0 ∧ E0 ⇒ each clause of E1. `weakened`: a forward clause is refuted, or
//! the effect went from `pure` to `io`. `undecided`: no forward clause is
//! refuted and one is not decided (G1: never a guess). `same`: all four
//! proved and the same effect. `refined`: forward proved and a backward
//! clause refuted, or `io` became `pure` (the commutation relation's
//! non-conflicting change, collaboration review §4). `compatible`: forward
//! proved, backward not decided. `signature`: the parameters, result or
//! type parameters differ (by type identity, so a renamed type is the
//! same). Every refuted clause carries the solver's counterexample.
//!
//! **One program.** Both versions are encoded in one query: the old units
//! under fresh names (`old_u`), with every old definition and type whose
//! `id` the new version also has shared with it (references rewritten to
//! the new one, `refs.rs`), so an unchanged `spec fn` is the same function
//! on both sides, opaque or revealed alike, and an unchanged type the same
//! sort. When several new items have the `id`, only the one of the same
//! path is taken; a trait and its `impl` functions only under the same
//! path. The changed `impl` functions of a shared type become functions of
//! the old unit (`T__f`); a type whose generic `impl` changed is not
//! shared. A compared definition always keeps its old copy, and anything
//! that refers to an unshared item is unshared too. The aligned
//! program must check and every old item must keep its `id` there; else
//! the compared definitions are `undecided` with the reason.
//!
//! **What the solver sees.** The spec fns each side's `#[uses]` reveals
//! (as the ordinary proofs see them), and every spec fn the two contracts
//! reach through revealed bodies whose `id` changed between the versions:
//! a contract that names a changed spec fn is compared through its
//! bodies, not its name. The pinned `rlimit`, no time limit (DESIGN.md
//! §8.2): the same input gives the same bytes.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::check::{Effect, FnSig};
use crate::diag::{Diag, Diags, json_str};
use crate::identity::{Ids, Key};
use crate::refs::{Rewrite, Site};
use crate::verify::{ClauseAnswer, RefineJob};

/// The warning a weakened contract gets.
pub const CODE: &str = "W0908";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Same,
    Refined,
    Compatible,
    Weakened,
    Undecided,
    Signature,
    Added,
    Removed,
}

impl Verdict {
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Same => "same",
            Verdict::Refined => "refined",
            Verdict::Compatible => "compatible",
            Verdict::Weakened => "weakened",
            Verdict::Undecided => "undecided",
            Verdict::Signature => "signature",
            Verdict::Added => "added",
            Verdict::Removed => "removed",
        }
    }
}

/// The new clauses (or effect) against the old: `weaker` requires and
/// `stronger` ensures refine, `stronger` requires and `weaker` ensures
/// weaken.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rel {
    Same,
    Weaker,
    Stronger,
    Incomparable,
    Undecided,
}

impl Rel {
    pub fn name(self) -> &'static str {
        match self {
            Rel::Same => "same",
            Rel::Weaker => "weaker",
            Rel::Stronger => "stronger",
            Rel::Incomparable => "incomparable",
            Rel::Undecided => "undecided",
        }
    }
}

/// A clause one version promises that the other does not, with the
/// version it is from (`new requires p(x)`, `old ensures q(x, r)`,
/// `effects pure`), and the counterexample, or why it was not decided.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub clause: String,
    pub detail: String,
    /// The spec fns the refuting query saw only by name (neither side's
    /// `#[uses]` reveals them and their `id` did not change): the model
    /// interprets them freely, as a caller's proof that names them may.
    pub opaque: Vec<String>,
}

/// One compared definition.
#[derive(Clone, Debug)]
pub struct Row {
    pub def: Option<String>,
    pub old: Option<String>,
    /// How the two were matched: `path`, `id` or `body`.
    pub matched: Option<&'static str>,
    pub verdict: Verdict,
    pub requires: Option<Rel>,
    pub ensures: Option<Rel>,
    pub effects: Option<Rel>,
    /// What the old contract promised that the new one does not (refuted
    /// forward clauses, a lost `pure`).
    pub lost: Vec<Clause>,
    /// What the new contract promises that the old one did not (refuted
    /// backward clauses).
    pub gained: Vec<Clause>,
    /// Clauses the solver did not decide.
    pub undecided: Vec<Clause>,
    pub why: Option<String>,
}

impl Row {
    fn new(def: Option<&str>, old: Option<&str>, verdict: Verdict) -> Row {
        Row {
            def: def.map(String::from),
            old: old.map(String::from),
            matched: None,
            verdict,
            requires: None,
            ensures: None,
            effects: None,
            lost: Vec::new(),
            gained: Vec::new(),
            undecided: Vec::new(),
            why: None,
        }
    }

    /// The row as one JSON object (every key present, `null` where it does
    /// not apply).
    pub fn json(&self) -> String {
        let opt = |s: &Option<String>| s.as_deref().map_or("null".into(), json_str);
        let rel = |r: Option<Rel>| r.map_or("null".into(), |r| json_str(r.name()));
        let list = |cs: &[Clause], refuted: bool| {
            let xs: Vec<String> = cs
                .iter()
                .map(|c| {
                    let opaque: Vec<String> = c.opaque.iter().map(|o| json_str(o)).collect();
                    if refuted {
                        format!(
                            "{{\"clause\":{},\"counterexample\":{},\"opaque\":[{}]}}",
                            json_str(&c.clause),
                            json_str(&c.detail),
                            opaque.join(",")
                        )
                    } else {
                        format!(
                            "{{\"clause\":{},\"why\":{}}}",
                            json_str(&c.clause),
                            json_str(&c.detail)
                        )
                    }
                })
                .collect();
            format!("[{}]", xs.join(","))
        };
        format!(
            "{{\"def\":{},\"old\":{},\"match\":{},\"verdict\":{},\"requires\":{},\"ensures\":{},\"effects\":{},\"lost\":{},\"gained\":{},\"undecided\":{},\"why\":{}}}",
            opt(&self.def),
            opt(&self.old),
            self.matched.map_or("null".into(), json_str),
            json_str(self.verdict.name()),
            rel(self.requires),
            rel(self.ensures),
            rel(self.effects),
            list(&self.lost, true),
            list(&self.gained, true),
            list(&self.undecided, false),
            opt(&self.why),
        )
    }
}

#[derive(Debug)]
pub struct WeakenOutput {
    /// One row per definition of either version, by path.
    pub rows: Vec<Row>,
    /// `W0908` for each weakened definition, at the new version (JSONL).
    pub diagnostics: String,
    /// Solver obligations run.
    pub queries: usize,
}

impl WeakenOutput {
    fn count(&self, v: Verdict) -> usize {
        self.rows.iter().filter(|r| r.verdict == v).count()
    }

    /// No contract got weaker and every comparison was decided.
    pub fn ok(&self) -> bool {
        self.count(Verdict::Weakened) == 0 && self.count(Verdict::Undecided) == 0
    }

    /// JSONL: the rows, the `W0908` diagnostics, then
    /// `{"summary":true,"defs","same","refined","compatible","weakened","undecided","signature","added","removed","queries"}`.
    pub fn report(&self) -> String {
        let mut out = String::new();
        for r in &self.rows {
            out.push_str(&r.json());
            out.push('\n');
        }
        out.push_str(&self.diagnostics);
        out.push_str(&format!(
            "{{\"summary\":true,\"defs\":{},\"same\":{},\"refined\":{},\"compatible\":{},\"weakened\":{},\"undecided\":{},\"signature\":{},\"added\":{},\"removed\":{},\"queries\":{}}}\n",
            self.rows.len(),
            self.count(Verdict::Same),
            self.count(Verdict::Refined),
            self.count(Verdict::Compatible),
            self.count(Verdict::Weakened),
            self.count(Verdict::Undecided),
            self.count(Verdict::Signature),
            self.count(Verdict::Added),
            self.count(Verdict::Removed),
            self.queries
        ));
        out
    }
}

/// A compared definition of one version.
struct Def<'p> {
    f: &'p FnDef,
    sig: &'p FnSig,
    id: String,
}

/// The executable functions and `proof fn`s of a checked program (free and
/// in `impl T` blocks; trait impl functions and default bodies refine their
/// trait's contract, which the gate proves, E0817), with their `id`s.
fn defs<'p>(p: &'p Program, t: &'p crate::verify::Tables, ids: &Ids) -> BTreeMap<String, Def<'p>> {
    crate::check::defs_of(p, &t.fns)
        .into_iter()
        .filter(|(path, _, f)| {
            f.kind != FnKind::Spec && !path.starts_with('<') && t.fns[path].trait_of.is_none()
        })
        .filter_map(|(path, _, f)| {
            let id = ids.def(&path)?.id;
            Some((
                path.clone(),
                Def {
                    f,
                    sig: &t.fns[&path],
                    id,
                },
            ))
        })
        .collect()
}

/// The identity of each definition in the program with every `requires`,
/// `ensures` and `#[uses]` removed: what a rename that also changes the
/// contract keeps. Empty when that program does not check.
fn body_ids(p: &Program) -> BTreeMap<String, String> {
    fn strip(m: &mut Mod) {
        let fns = m
            .fns
            .iter_mut()
            .chain(m.impls.iter_mut().flat_map(|i| &mut i.fns))
            .chain(m.trait_impls.iter_mut().flat_map(|i| &mut i.fns))
            .chain(
                m.traits
                    .iter_mut()
                    .flat_map(|t| t.fns.iter_mut().map(|tf| &mut tf.f)),
            );
        for f in fns {
            f.requires.clear();
            f.ensures.clear();
            f.attrs.retain(|a| a.name.name != "uses");
        }
        m.mods.iter_mut().for_each(strip);
    }
    let mut q = p.clone();
    q.mods.iter_mut().for_each(strip);
    let mut d = Diags::default();
    let mut c = crate::check::Checker::new(&mut d);
    c.check_program(&q);
    let t = c.into_tables();
    if d.has_errors() {
        return BTreeMap::new();
    }
    let ids = Ids::new(&q, &t);
    crate::check::defs_of(&q, &t.fns)
        .into_iter()
        .filter_map(|(path, _, _)| Some((path.clone(), ids.def(&path)?.id)))
        .collect()
}

/// A type with named types by identity and type parameters by position:
/// equal for two versions exactly when the type is the same up to names.
fn ty_key(t: &TypeKind, ids: &Ids, tparams: &[String]) -> String {
    let k = |x: &Type| ty_key(&x.kind, ids, tparams);
    match t {
        TypeKind::Named { path, args } => {
            let full = path.join("::");
            let head = ids.get(&Key::Type(full.clone())).map_or(full, |h| h.id);
            let a: Vec<String> = args.iter().map(k).collect();
            format!("{head}<{}>", a.join(", "))
        }
        TypeKind::Param(n) => match tparams.iter().position(|p| p == n) {
            Some(i) => format!("'{i}"),
            None => format!("'{n}"),
        },
        TypeKind::Array(e, n) => format!("[{}; {n}]", k(e)),
        TypeKind::Slice(e) => format!("[{}]", k(e)),
        TypeKind::Vec(e) => format!("Vec<{}>", k(e)),
        TypeKind::Box(e) => format!("Box<{}>", k(e)),
        TypeKind::Ref { mutable, inner } => {
            format!("&{}{}", if *mutable { "mut " } else { "" }, k(inner))
        }
        t => crate::check::ty_str(t),
    }
}

/// How the signatures differ (parameters, result, type parameters), if
/// they do.
fn signature_change(o: &Def, io: &Ids, n: &Def, inw: &Ids) -> Option<String> {
    let names = |s: &FnSig| {
        s.generics
            .iter()
            .map(|g| g.name.clone())
            .collect::<Vec<_>>()
    };
    let (go, gn) = (names(o.sig), names(n.sig));
    if go.len() != gn.len() {
        return Some(format!("type parameters: {} -> {}", go.len(), gn.len()));
    }
    if o.sig.params.len() != n.sig.params.len() {
        return Some(format!(
            "parameters: {} -> {}",
            o.sig.params.len(),
            n.sig.params.len()
        ));
    }
    for (k, (a, b)) in o.sig.params.iter().zip(&n.sig.params).enumerate() {
        if ty_key(a, io, &go) != ty_key(b, inw, &gn) {
            return Some(format!(
                "parameter {}: `{}` -> `{}`",
                k + 1,
                crate::check::ty_str(a),
                crate::check::ty_str(b)
            ));
        }
    }
    if ty_key(&o.sig.ret, io, &go) != ty_key(&n.sig.ret, inw, &gn) {
        return Some(format!(
            "result: `{}` -> `{}`",
            crate::check::ty_str(&o.sig.ret),
            crate::check::ty_str(&n.sig.ret)
        ));
    }
    None
}

/// The effect of the new version against the old: `io` promises less than
/// `pure`.
fn effects_rel(o: &FnSig, n: &FnSig) -> Rel {
    match (o.effect, n.effect) {
        (a, b) if a == b => Rel::Same,
        (Effect::Pure, Effect::Io) => Rel::Weaker,
        _ => Rel::Stronger,
    }
}

/// Matches the definitions of the two versions: (old, new, how).
fn match_defs(
    od: &BTreeMap<String, Def>,
    nd: &BTreeMap<String, Def>,
    ob: &BTreeMap<String, String>,
    nb: &BTreeMap<String, String>,
) -> Vec<(Option<String>, Option<String>, Option<&'static str>)> {
    let mut out = Vec::new();
    let mut old_left: BTreeSet<&String> = od.keys().collect();
    let mut new_left: BTreeSet<&String> = BTreeSet::new();
    for (p, n) in nd {
        match od.get(p) {
            Some(o) if o.f.kind == n.f.kind => {
                old_left.remove(p);
                out.push((Some(p.clone()), Some(p.clone()), Some("path")));
            }
            _ => {
                new_left.insert(p);
            }
        }
    }
    // Then by a hash unique on both sides.
    let mut by = |how: &'static str,
                  oh: &dyn Fn(&String) -> Option<String>,
                  nh: &dyn Fn(&String) -> Option<String>| {
        let group = |left: &BTreeSet<&String>, h: &dyn Fn(&String) -> Option<String>| {
            let mut g: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for p in left {
                if let Some(x) = h(p) {
                    g.entry(x).or_default().push((*p).clone());
                }
            }
            g
        };
        let (go, gn) = (group(&old_left, oh), group(&new_left, nh));
        for (h, os) in &go {
            if let (Some(ns), [o]) = (gn.get(h), os.as_slice())
                && let [n] = ns.as_slice()
                && od[o].f.kind == nd[n].f.kind
            {
                old_left.remove(o);
                new_left.remove(n);
                out.push((Some(o.clone()), Some(n.clone()), Some(how)));
            }
        }
    };
    by("id", &|p| Some(od[p].id.clone()), &|p| {
        Some(nd[p].id.clone())
    });
    by("body", &|p| ob.get(p).cloned(), &|p| nb.get(p).cloned());
    out.extend(old_left.into_iter().map(|p| (Some(p.clone()), None, None)));
    out.extend(new_left.into_iter().map(|p| (None, Some(p.clone()), None)));
    out
}

/// The definition a path written at `site` in `unit` names, as the
/// checker resolves it (`verify::resolve_in`, `check::uses_target`): its
/// full path and how many segments name it (a variant's type leaves the
/// variant after it).
fn resolve(
    t: &crate::verify::Tables,
    traits: &BTreeSet<String>,
    segs: &[String],
    site: Site,
    unit: &[String],
    tparams: &[String],
) -> Option<(String, usize)> {
    let first = segs.first()?;
    if tparams.contains(first) || first == "Self" {
        return None;
    }
    let n = segs.len();
    let u = unit.join("::");
    let local = |k: usize| format!("{u}::{}", segs[..k].join("::"));
    let abs = |k: usize| segs[..k].join("::");
    let is_fn = |p: &str| t.fns.get(p).is_some_and(|s| !s.builtin);
    let is_ty = |p: &str| t.adts.contains_key(p);
    match site {
        Site::Type | Site::Struct => {
            let p = if n == 1 { local(1) } else { abs(n) };
            is_ty(&p).then_some((p, n))
        }
        Site::Bound => {
            let p = if n == 1 { local(1) } else { abs(n) };
            traits.contains(&p).then_some((p, n))
        }
        Site::Call | Site::Variant => {
            if n == 1 {
                return (site == Site::Call && is_fn(&local(1))).then(|| (local(1), 1));
            }
            if n == 2 && (is_ty(&local(1)) || traits.contains(&local(1))) {
                if is_fn(&local(2)) {
                    return Some((local(2), 2));
                }
                return is_ty(&local(1)).then(|| (local(1), 1));
            }
            if is_fn(&abs(n)) {
                return Some((abs(n), n));
            }
            is_ty(&abs(n - 1)).then(|| (abs(n - 1), n - 1))
        }
    }
}

/// The old program's references, rewritten to the aligned program's paths.
struct Align<'a> {
    t: &'a crate::verify::Tables,
    traits: &'a BTreeSet<String>,
    to: &'a dyn Fn(&str) -> String,
}

impl Rewrite for Align<'_> {
    fn path(&mut self, segs: &mut Vec<String>, site: Site, unit: &[String], tparams: &[String]) {
        if let Some((e, k)) = resolve(self.t, self.traits, segs, site, unit, tparams) {
            let mut out: Vec<String> = (self.to)(&e).split("::").map(String::from).collect();
            out.extend(segs[k..].iter().cloned());
            *segs = out;
        }
    }
}

/// The two versions as one program.
struct Aligned {
    files: Vec<(String, String)>,
    /// Each compared old definition's path there.
    at: BTreeMap<String, String>,
    /// The private names of the old units (old -> aligned).
    units: BTreeMap<String, String>,
    /// The changed `impl` functions of shared types, by old path: where
    /// they are (functions of the old unit).
    moved: BTreeMap<String, String>,
}

/// Which old items the aligned program shares with the new version: old
/// path -> new path (see the module comment).
fn shared(
    ot: &crate::verify::Tables,
    io: &Ids,
    inw: &Ids,
    nt: &crate::verify::Tables,
    compared: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    // A trait, its `impl` functions and its default bodies are named by
    // the trait's path (`<T as Tr>::m`, `Tr::m`): shared only under the same
    // path.
    let in_trait = |k: &Key, t: &crate::verify::Tables| match k {
        Key::Def(p) => p.starts_with('<') || t.fns.get(p).is_some_and(|s| s.trait_of.is_some()),
        Key::Type(_) => false,
        Key::Trait(_) => true,
    };
    let item = |k: &Key, t: &crate::verify::Tables| match k {
        Key::Def(p) => t.fns.contains_key(p),
        _ => true,
    };
    let kind = |k: &Key| match k {
        Key::Def(_) => 0u8,
        Key::Type(_) => 1,
        Key::Trait(_) => 2,
    };
    // New items by identity. Two declarations can have one `id` (two
    // structs of the same fields and no trait `impl`s): an old item is
    // shared with the new item of its own path and `id`, else with the one
    // new item of its `id`, else not at all.
    let mut by_id: BTreeMap<(u8, String), Vec<String>> = BTreeMap::new();
    for k in inw.keys().filter(|k| item(k, nt)) {
        if let Some(h) = inw.get(k) {
            by_id
                .entry((kind(k), h.id))
                .or_default()
                .push(k.path().to_string());
        }
    }
    let mut share: BTreeMap<String, String> = BTreeMap::new();
    let mut keys: BTreeMap<String, Key> = BTreeMap::new();
    for k in io.keys().filter(|k| item(k, ot)) {
        let Some(h) = io.get(k) else { continue };
        let Some(ns) = by_id.get(&(kind(k), h.id)) else {
            continue;
        };
        let np = if ns.iter().any(|n| n == k.path()) {
            k.path().to_string()
        } else if let [n] = ns.as_slice()
            && !in_trait(k, ot)
        {
            n.clone()
        } else {
            continue;
        };
        share.insert(k.path().to_string(), np);
        keys.insert(k.path().to_string(), k.clone());
    }
    // A compared definition keeps its old copy even when a new definition
    // has its `id`: the backward comparison is made on that copy.
    for c in compared {
        share.remove(c);
    }
    // The `impl T` functions of each old type, and whether their block
    // has type parameters.
    let impl_fns: Vec<(String, String, bool)> = ot
        .fns
        .iter()
        .filter(|(p, s)| s.in_impl && !p.starts_with('<') && !s.builtin)
        .filter_map(|(p, s)| {
            Some((
                p.rsplit_once("::")?.0.to_string(),
                p.clone(),
                s.impl_generics > 0,
            ))
        })
        .collect();
    loop {
        let before = share.len();
        // An `impl` function goes with its type, into the shared type's
        // `impl`. A changed one of a shared type becomes a function of the
        // old unit (`align`), which a generic `impl` cannot give.
        for (ty, f, generic) in &impl_fns {
            let ok = match (share.get(ty), share.get(f)) {
                (Some(nt), Some(nf)) => nf.rsplit_once("::").is_some_and(|(h, _)| h == nt),
                _ => false,
            };
            if !ok {
                share.remove(f);
                if *generic {
                    share.remove(ty);
                }
            }
        }
        // Nothing that refers to an unshared item.
        let gone: Vec<String> = share
            .keys()
            .filter(|p| {
                io.refs(&keys[*p])
                    .iter()
                    .any(|r| !share.contains_key(r.path()))
            })
            .cloned()
            .collect();
        for p in gone {
            share.remove(&p);
        }
        if share.len() == before {
            break;
        }
    }
    share
}

/// Builds the aligned program (see the module comment).
fn align(
    old: &crate::Frontend,
    io: &Ids,
    new_files: &[(String, String)],
    new: &crate::Frontend,
    inw: &Ids,
    compared: &BTreeSet<String>,
) -> Aligned {
    let share = shared(&old.tables, io, inw, &new.tables, compared);
    let taken: BTreeSet<String> = old
        .program
        .mods
        .iter()
        .chain(&new.program.mods)
        .map(|m| m.name.name.clone())
        .collect();
    let mut units: BTreeMap<String, String> = BTreeMap::new();
    for m in &old.program.mods {
        let mut pu = format!("old_{}", m.name.name);
        while taken.contains(&pu) || units.values().any(|v| v == &pu) {
            pu = format!("old_{pu}");
        }
        units.insert(m.name.name.clone(), pu);
    }
    let private = |p: &str| match p.split_once("::") {
        Some((u, rest)) => format!("{}::{rest}", units.get(u).map_or(u, String::as_str)),
        None => p.to_string(),
    };
    let traits = crate::workspace::trait_paths(&old.program);
    let mut prog = old.program.clone();
    // Drops the shared items (and `main`, which the new version has). The
    // changed `impl` functions of a shared type become functions of their
    // unit, `T__f` (`conv`: old path -> that path), with `Self` written `T`
    // and the receiver `self` renamed.
    fn prune(
        m: &mut Mod,
        prefix: &str,
        share: &BTreeMap<String, String>,
        conv: &mut BTreeMap<String, String>,
    ) {
        let mp = format!("{prefix}{}", m.name.name);
        m.fns.retain(|f| {
            !share.contains_key(&format!("{mp}::{}", f.name.name))
                && !(f.kind == FnKind::Exec && f.name.name == "main")
        });
        m.types
            .retain(|t| !share.contains_key(&format!("{mp}::{}", t.name.name)));
        // A shared trait goes with its `impl`s (they are part of its `id`).
        m.traits
            .retain(|t| !share.contains_key(&format!("{mp}::{}", t.name.name)));
        m.trait_impls.retain(|i| {
            let names: Vec<&str> = i.trait_path.iter().map(|s| s.name.as_str()).collect();
            let tr = match names.as_slice() {
                [one] => format!("{mp}::{one}"),
                more => more.join("::"),
            };
            !share.contains_key(&tr)
        });
        let unit: Vec<String> = mp.split("::").map(String::from).collect();
        let mut kept = Vec::new();
        for mut i in std::mem::take(&mut m.impls) {
            let ty = i.ty.name.clone();
            if !share.contains_key(&format!("{mp}::{ty}")) {
                kept.push(i);
                continue;
            }
            for mut f in std::mem::take(&mut i.fns) {
                let path = format!("{mp}::{ty}::{}", f.name.name);
                if share.contains_key(&path) {
                    continue;
                }
                let taken = |n: &str| {
                    m.fns.iter().any(|g| g.name.name == n)
                        || m.types.iter().any(|t| t.name.name == n)
                };
                let mut name = format!("{ty}__{}", f.name.name);
                while taken(&name) {
                    name.push('_');
                }
                let mut vars = Vars(BTreeSet::new());
                crate::refs::walk_fn(&mut f, &unit, &mut vars);
                let mut this = "this".to_string();
                while vars.0.contains(&this) || f.params.iter().any(|p| p.name.name == this) {
                    this.push('_');
                }
                for p in f.params.iter_mut().filter(|p| p.name.name == "self") {
                    p.name.name = this.clone();
                }
                crate::refs::walk_fn(
                    &mut f,
                    &unit,
                    &mut ImplToFree {
                        ty: &ty,
                        this: &this,
                    },
                );
                f.name.name = name.clone();
                conv.insert(path, format!("{mp}::{name}"));
                m.fns.push(f);
            }
        }
        m.impls = kept;
        for s in &mut m.mods {
            prune(s, &format!("{mp}::"), share, conv);
        }
    }
    let mut conv = BTreeMap::new();
    for m in &mut prog.mods {
        prune(m, "", &share, &mut conv);
    }
    // A method of a shared trait keeps its path (traits are shared only
    // under their own path).
    let to = |p: &str| match conv.get(p) {
        Some(c) => private(c),
        None => match share.get(p) {
            Some(n) => n.clone(),
            None if p
                .rsplit_once("::")
                .is_some_and(|(tr, _)| share.contains_key(tr) && traits.contains(tr)) =>
            {
                p.to_string()
            }
            None => private(p),
        },
    };
    let mut a = Align {
        t: &old.tables,
        traits: &traits,
        to: &to,
    };
    crate::refs::walk(&mut prog, &mut a);
    for m in &mut prog.mods {
        m.name.name = units[&m.name.name].clone();
    }
    let mut files = new_files.to_vec();
    for (unit, text) in crate::fmt::format_units(&prog) {
        files.push((format!("<old>/{unit}.bmb"), text));
    }
    let at = compared.iter().map(|p| (p.clone(), to(p))).collect();
    let moved = conv.iter().map(|(p, c)| (p.clone(), private(c))).collect();
    Aligned {
        files,
        at,
        units,
        moved,
    }
}

/// The variables a function reads.
struct Vars(BTreeSet<String>);

impl Rewrite for Vars {
    fn path(&mut self, _: &mut Vec<String>, _: Site, _: &[String], _: &[String]) {}
    fn var(&mut self, name: &mut String) {
        self.0.insert(name.clone());
    }
}

/// An `impl T` function made a function of `T`'s unit: `Self` is `T`, the
/// receiver `self` is `this`.
struct ImplToFree<'a> {
    ty: &'a str,
    this: &'a str,
}

impl Rewrite for ImplToFree<'_> {
    fn path(&mut self, segs: &mut Vec<String>, _: Site, _: &[String], _: &[String]) {
        if segs.first().is_some_and(|s| s == "Self") {
            segs[0] = self.ty.to_string();
        }
    }
    fn var(&mut self, name: &mut String) {
        if name == "self" {
            *name = self.this.to_string();
        }
    }
}

/// Every old item the aligned program holds keeps its `id` there (the
/// alignment changed no meaning), or the first that does not.
fn audit(old: &Ids, aligned: &Ids, al: &Aligned) -> Result<(), String> {
    for k in old.keys() {
        let p = k.path();
        if p.starts_with('<') {
            continue;
        }
        let Some((u, rest)) = p.split_once("::") else {
            continue;
        };
        let q = match al.moved.get(p) {
            Some(m) => m.clone(),
            None => format!("{}::{rest}", al.units[u]),
        };
        let ak = match k {
            Key::Def(_) => Key::Def(q),
            Key::Type(_) => Key::Type(q),
            Key::Trait(_) => Key::Trait(q),
        };
        let Some(a) = aligned.get(&ak) else {
            continue; // shared with the new version, or `main`
        };
        if old.get(k).map(|h| h.id) != Some(a.id) {
            return Err(format!(
                "aligning the versions changed the meaning of `{p}`"
            ));
        }
    }
    Ok(())
}

/// The function at `path` in `p` (free or in an `impl T` block).
fn def_mut<'p>(p: &'p mut Program, path: &str) -> Option<&'p mut FnDef> {
    fn walk<'p>(m: &'p mut Mod, prefix: &str, path: &str) -> Option<&'p mut FnDef> {
        let mp = format!("{prefix}{}", m.name.name);
        let rest = path.strip_prefix(&format!("{mp}::"))?;
        if let Some(i) = m.fns.iter().position(|f| f.name.name == rest) {
            return Some(&mut m.fns[i]);
        }
        if let Some((ty, name)) = rest.split_once("::")
            && let Some(i) = m.impls.iter().position(|i| i.ty.name == ty)
            && let Some(k) = m.impls[i].fns.iter().position(|f| f.name.name == name)
        {
            return Some(&mut m.impls[i].fns[k]);
        }
        let sub = format!("{mp}::");
        m.mods.iter_mut().find_map(|s| walk(s, &sub, path))
    }
    p.mods.iter_mut().find_map(|m| walk(m, "", path))
}

/// The spec fns a comparison reveals: both sides' `#[uses]`, and every spec
/// fn the two surfaces reach through revealed bodies whose identity changed
/// between the versions.
fn reveal(
    ids: &Ids,
    t: &crate::verify::Tables,
    p: &Program,
    sides: [&str; 2],
    changed: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    for path in sides {
        for (q, unit, f) in crate::check::defs_of(p, &t.fns) {
            if q == path {
                out.extend(crate::verify::uses_of(f, &unit, &t.fns));
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut work: Vec<Key> = sides
        .iter()
        .flat_map(|s| ids.surface_refs(&Key::Def(s.to_string())))
        .collect();
    work.extend(out.iter().map(|s| Key::Def(s.clone())));
    while let Some(k) = work.pop() {
        let Key::Def(s) = &k else { continue };
        if !seen.insert(s.clone()) || t.fns.get(s).is_none_or(|g| g.kind != FnKind::Spec) {
            continue;
        }
        if changed(s) {
            out.insert(s.clone());
        }
        if out.contains(s) {
            work.extend(ids.surface_refs(&k));
        }
    }
    out.into_iter().collect()
}

/// The answers to one side's clauses, by clause position. A clause the
/// comparison made no obligation for holds trivially (`Enc::refines` makes
/// none when there is nothing to imply); a comparison that did not run
/// decides nothing.
fn answers(
    results: Option<&Vec<(crate::source::Span, ClauseAnswer)>>,
    clauses: &[Expr],
) -> Vec<ClauseAnswer> {
    let Some(r) = results else {
        return vec![ClauseAnswer::Undecided("not compared".into()); clauses.len().max(1)];
    };
    clauses
        .iter()
        .map(|c| {
            r.iter()
                .find(|(s, _)| *s == c.span)
                .map_or(ClauseAnswer::Proved, |(_, a)| a.clone())
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    Proved,
    Refuted,
    Undecided,
}

fn dir(xs: &[ClauseAnswer]) -> Dir {
    if xs.iter().any(|a| matches!(a, ClauseAnswer::Refuted { .. })) {
        Dir::Refuted
    } else if xs.iter().any(|a| matches!(a, ClauseAnswer::Undecided(_))) {
        Dir::Undecided
    } else {
        Dir::Proved
    }
}

/// The relation of the new clauses to the old from the two directions:
/// `fwd` (the old side implies the new clauses) and `back`. For
/// `requires`, a refuted forward direction is a stronger precondition; for
/// `ensures`, a weaker postcondition.
fn relation(fwd: Dir, back: Dir, requires: bool) -> Rel {
    let (f_fails, b_fails) = match (fwd, back) {
        (Dir::Undecided, _) | (_, Dir::Undecided) => return Rel::Undecided,
        (f, b) => (f == Dir::Refuted, b == Dir::Refuted),
    };
    match (f_fails, b_fails, requires) {
        (false, false, _) => Rel::Same,
        (true, true, _) => Rel::Incomparable,
        (true, false, true) | (false, true, false) => Rel::Stronger,
        (false, true, true) | (true, false, false) => Rel::Weaker,
    }
}

/// The verdict of a comparison the solver could not make: `weakened`
/// still when the effect alone weakens the contract.
fn undecided(r: &Row) -> Verdict {
    if r.effects == Some(Rel::Weaker) {
        Verdict::Weakened
    } else {
        Verdict::Undecided
    }
}

/// Compares the contracts of two versions of a program (see the module
/// comment). `Err` carries the diagnostics when a version does not check.
pub fn weaken_sources(
    old: &[(String, String)],
    new: &[(String, String)],
) -> Result<WeakenOutput, String> {
    let fo = crate::frontend(old);
    if fo.diags.has_errors() {
        return Err(fo.diags.to_jsonl(&fo.sm));
    }
    let fnw = crate::frontend(new);
    if fnw.diags.has_errors() {
        return Err(fnw.diags.to_jsonl(&fnw.sm));
    }
    let io = Ids::new(&fo.program, &fo.tables);
    let inw = Ids::new(&fnw.program, &fnw.tables);
    let od = defs(&fo.program, &fo.tables, &io);
    let nd = defs(&fnw.program, &fnw.tables, &inw);
    let pairs = match_defs(&od, &nd, &body_ids(&fo.program), &body_ids(&fnw.program));

    let mut rows: Vec<Row> = Vec::new();
    // Pairs that need the solver: (row index, old, new).
    let mut asks: Vec<(usize, String, String)> = Vec::new();
    for (op, np, how) in pairs {
        let (o, n) = match (&op, &np) {
            (Some(o), Some(n)) => (o, n),
            (Some(o), None) => {
                rows.push(Row::new(None, Some(o), Verdict::Removed));
                continue;
            }
            (None, Some(n)) => {
                rows.push(Row::new(Some(n), None, Verdict::Added));
                continue;
            }
            (None, None) => continue,
        };
        let (od1, nd1) = (&od[o], &nd[n]);
        let mut row = Row::new(Some(n), Some(o), Verdict::Same);
        row.matched = how;
        if let Some(why) = signature_change(od1, &io, nd1, &inw) {
            row.verdict = Verdict::Signature;
            row.why = Some(why);
            rows.push(row);
            continue;
        }
        let eff = effects_rel(od1.sig, nd1.sig);
        row.effects = Some(eff);
        if eff == Rel::Weaker {
            row.lost.push(Clause {
                clause: "effects pure".into(),
                detail: String::new(),
                opaque: Vec::new(),
            });
        }
        let empty = |f: &FnDef| f.requires.is_empty() && f.ensures.is_empty();
        if od1.id == nd1.id || (empty(od1.f) && empty(nd1.f)) {
            row.requires = Some(Rel::Same);
            row.ensures = Some(Rel::Same);
            row.verdict = match eff {
                Rel::Same => Verdict::Same,
                Rel::Weaker => Verdict::Weakened,
                _ => Verdict::Refined,
            };
            rows.push(row);
            continue;
        }
        asks.push((rows.len(), o.clone(), n.clone()));
        rows.push(row);
    }

    let mut queries = 0;
    if !asks.is_empty() {
        let compared: BTreeSet<String> = asks.iter().map(|(_, o, _)| o.clone()).collect();
        let al = align(&fo, &io, new, &fnw, &inw, &compared);
        let mut fc = crate::frontend(&al.files);
        let ic = Ids::new(&fc.program, &fc.tables);
        let ready = if fc.diags.has_errors() {
            let first = fc
                .diags
                .items
                .iter()
                .find(|d| crate::diag::severity(d.code) == "error")
                .map(|d| format!("{}: {}", d.code, d.message))
                .unwrap_or_default();
            Err(format!(
                "the two versions could not be checked as one program ({first})"
            ))
        } else {
            audit(&io, &ic, &al)
        };
        match ready {
            Err(why) => {
                for (k, _, _) in &asks {
                    rows[*k].verdict = undecided(&rows[*k]);
                    rows[*k].requires = Some(Rel::Undecided);
                    rows[*k].ensures = Some(Rel::Undecided);
                    rows[*k].why = Some(why.clone());
                }
            }
            Ok(()) => {
                let old_ids: BTreeSet<String> =
                    io.keys().filter_map(|k| io.get(k)).map(|h| h.id).collect();
                let private: BTreeSet<&String> = al.units.values().collect();
                let changed = |s: &str| {
                    let unit = s.split("::").next().unwrap_or_default().to_string();
                    private.contains(&unit) || ic.def(s).is_none_or(|h| !old_ids.contains(&h.id))
                };
                let mut jobs: BTreeMap<String, RefineJob> = BTreeMap::new();
                let mut uses: Vec<(String, Vec<String>)> = Vec::new();
                let tsub = |from: &str, to: &str| -> BTreeMap<String, TypeKind> {
                    let (a, b) = (&fc.tables.fns[from], &fc.tables.fns[to]);
                    a.generics
                        .iter()
                        .zip(&b.generics)
                        .map(|(x, y)| (x.name.clone(), TypeKind::Param(y.name.clone())))
                        .collect()
                };
                for (k, o, n) in &asks {
                    let oc = &al.at[o];
                    // The aligned signatures: a type the alignment could not
                    // share makes them differ.
                    let (so, sn) = (&fc.tables.fns[oc], &fc.tables.fns[n]);
                    let sub = tsub(oc, n);
                    let tys = |s: &FnSig, m: &BTreeMap<String, TypeKind>| -> Vec<String> {
                        s.params
                            .iter()
                            .chain([&s.ret])
                            .map(|t| crate::check::ty_str(&crate::check::subst(t, m)))
                            .collect()
                    };
                    if tys(so, &sub) != tys(sn, &BTreeMap::new()) {
                        rows[*k].verdict = undecided(&rows[*k]);
                        rows[*k].requires = Some(Rel::Undecided);
                        rows[*k].ensures = Some(Rel::Undecided);
                        rows[*k].why = Some(format!(
                            "a type of `{o}`'s signature could not be shared between the versions"
                        ));
                        continue;
                    }
                    let list = reveal(&ic, &fc.tables, &fc.program, [n, oc], &changed);
                    uses.push((n.clone(), list.clone()));
                    uses.push((oc.clone(), list));
                    jobs.insert(
                        n.clone(),
                        RefineJob {
                            decl: oc.clone(),
                            tsub: sub,
                            own_domain: false,
                        },
                    );
                }
                for (_, o, n) in &asks {
                    let oc = &al.at[o];
                    if jobs.contains_key(n) && !jobs.contains_key(oc) {
                        jobs.insert(
                            oc.clone(),
                            RefineJob {
                                decl: n.clone(),
                                tsub: tsub(n, oc),
                                own_domain: true,
                            },
                        );
                    }
                }
                drop(ic);
                // Each side reveals what the comparison needs (`reveal`).
                for (path, list) in uses {
                    if let Some(f) = def_mut(&mut fc.program, &path) {
                        f.attrs.retain(|a| a.name.name != "uses");
                        if !list.is_empty() {
                            f.attrs.push(Attr {
                                name: Ident {
                                    name: "uses".into(),
                                    span: f.name.span,
                                },
                                args: list
                                    .iter()
                                    .map(|s| {
                                        AttrArg::Ident(Ident {
                                            name: s.clone(),
                                            span: f.name.span,
                                        })
                                    })
                                    .collect(),
                                span: f.name.span,
                            });
                        }
                    }
                }
                let results =
                    crate::verify::contract_refinements(&fc.program, &fc.tables, jobs.clone());
                queries = results.values().map(Vec::len).sum();
                let find = |p: &str| -> Option<(&FnDef, String)> {
                    crate::check::defs_of(&fc.program, &fc.tables.fns)
                        .into_iter()
                        .find(|(q, _, _)| q == p)
                        .map(|(_, u, f)| (f, u))
                };
                // A spec fn the prover named from `unit`, by its full path
                // in the version it is from.
                let back_units: BTreeMap<&String, &String> =
                    al.units.iter().map(|(o, a)| (a, o)).collect();
                let full = |name: &str, unit: &str| -> String {
                    let q = if name.contains("::") {
                        name.to_string()
                    } else {
                        format!("{unit}::{name}")
                    };
                    match q.split_once("::") {
                        Some((u, rest)) if back_units.contains_key(&u.to_string()) => {
                            format!("{}::{rest}", back_units[&u.to_string()])
                        }
                        _ => q,
                    }
                };
                for (k, o, n) in &asks {
                    if !jobs.contains_key(n) {
                        continue;
                    }
                    let oc = &al.at[o];
                    let (Some((fo_c, uo)), Some((fn_c, un))) = (find(oc), find(n)) else {
                        continue;
                    };
                    let fwd_req = answers(results.get(n), &fn_c.requires);
                    let fwd_ens = answers(results.get(n), &fo_c.ensures);
                    let backward = jobs.get(oc).is_some_and(|j| &j.decl == n);
                    let (back_req, back_ens) = if backward {
                        (
                            answers(results.get(oc), &fo_c.requires),
                            answers(results.get(oc), &fn_c.ensures),
                        )
                    } else {
                        let u = |cs: &[Expr]| {
                            vec![ClauseAnswer::Undecided("not compared".into()); cs.len().max(1)]
                        };
                        (u(&fo_c.requires), u(&fn_c.ensures))
                    };
                    let (of, nf) = (od[o].f, nd[n].f);
                    let text = |kw: &str, e: &Expr| format!("{kw} {}", crate::fmt::expr_text(e));
                    let row = &mut rows[*k];
                    let mut put = |xs: &[ClauseAnswer], cs: &[Expr], kw: &str, fwd: bool| {
                        // Forward obligations are the new version's, backward
                        // ones the old version's (named from its unit).
                        let unit = if fwd { &un } else { &uo };
                        for (a, c) in xs.iter().zip(cs) {
                            match a {
                                ClauseAnswer::Proved => {}
                                ClauseAnswer::Refuted {
                                    counterexample,
                                    opaque,
                                } => {
                                    let c = Clause {
                                        clause: text(kw, c),
                                        detail: counterexample.clone(),
                                        opaque: opaque.iter().map(|o| full(o, unit)).collect(),
                                    };
                                    if fwd {
                                        row.lost.push(c);
                                    } else {
                                        row.gained.push(c);
                                    }
                                }
                                ClauseAnswer::Undecided(why) if fwd => row.undecided.push(Clause {
                                    clause: text(kw, c),
                                    detail: why.clone(),
                                    opaque: Vec::new(),
                                }),
                                ClauseAnswer::Undecided(_) => {}
                            }
                        }
                    };
                    // Forward: the new requires the old ones must imply, the
                    // old ensures the new ones must imply. Backward: the
                    // other way.
                    put(&fwd_req, &nf.requires, "new requires", true);
                    put(&fwd_ens, &of.ensures, "old ensures", true);
                    if backward {
                        put(&back_req, &of.requires, "old requires", false);
                        put(&back_ens, &nf.ensures, "new ensures", false);
                    }
                    let (fr, fe) = (dir(&fwd_req), dir(&fwd_ens));
                    let (br, be) = (dir(&back_req), dir(&back_ens));
                    row.requires = Some(relation(fr, br, true));
                    row.ensures = Some(relation(fe, be, false));
                    let eff = row.effects.unwrap_or(Rel::Same);
                    row.verdict = if fr == Dir::Refuted || fe == Dir::Refuted || eff == Rel::Weaker
                    {
                        Verdict::Weakened
                    } else if fr == Dir::Undecided || fe == Dir::Undecided {
                        Verdict::Undecided
                    } else if br == Dir::Refuted || be == Dir::Refuted || eff == Rel::Stronger {
                        Verdict::Refined
                    } else if br == Dir::Proved && be == Dir::Proved {
                        Verdict::Same
                    } else {
                        Verdict::Compatible
                    };
                }
            }
        }
    }

    rows.sort_by(|a, b| {
        let key = |r: &Row| (r.def.clone().or(r.old.clone()), r.old.clone());
        key(a).cmp(&key(b))
    });

    // W0908 at the new version: the first lost clause.
    let mut diags = Diags::default();
    for r in rows.iter().filter(|r| r.verdict == Verdict::Weakened) {
        let (Some(n), Some(o)) = (&r.def, &r.old) else {
            continue;
        };
        let f = nd[n].f;
        let first = &r.lost[0];
        let (span, what) = if let Some(c) = first.clause.strip_prefix("new requires ") {
            let at = f
                .requires
                .iter()
                .find(|e| crate::fmt::expr_text(e) == c)
                .map_or(f.name.span, |e| e.span);
            (
                at,
                format!("precondition `{c}` is not implied by the old preconditions"),
            )
        } else if let Some(c) = first.clause.strip_prefix("old ensures ") {
            let at = match (f.ensures.first(), f.ensures.last()) {
                (Some(a), Some(b)) => a.span.to(b.span),
                _ => f.name.span,
            };
            (
                at,
                format!("old postcondition `{c}` is not implied by the new contract"),
            )
        } else {
            (
                f.name.span,
                "the effect went from `pure` to `io`".to_string(),
            )
        };
        let mut msg = format!("the contract of `{n}` is weaker than the old one (`{o}`): {what}");
        if !first.detail.is_empty() && first.clause != "effects pure" {
            msg.push_str(&format!("; counterexample: {}", first.detail));
        }
        if !first.opaque.is_empty() {
            let names: Vec<String> = first.opaque.iter().map(|o| format!("`{o}`")).collect();
            msg.push_str(&format!(
                ", with {} seen only by name (neither version reveals it)",
                names.join(", ")
            ));
        }
        if r.lost.len() > 1 {
            msg.push_str(&format!(" (and {} more)", r.lost.len() - 1));
        }
        diags.push(Diag::new(CODE, span, msg).fix(
            "callers may rely on the old contract: keep it, or change it only by weakening `requires` or strengthening `ensures` (a refinement)",
            None,
        ));
    }
    Ok(WeakenOutput {
        rows,
        diagnostics: diags.to_jsonl(&fnw.sm),
        queries,
    })
}
