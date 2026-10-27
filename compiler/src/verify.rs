//! Proof gate (backlog 1B-02 .. 1B-11, semantics in ADR-0017).
//!
//! Each executable function is symbolically executed over its checked syntax
//! tree. Local variables become single-assignment SMT constants; loops are cut
//! by their invariants; calls assume only the callee's `ensures`. Every implicit
//! and explicit proof obligation is checked separately by Z3. Only `unsat`
//! proves an obligation.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::cache::ProofCache;
use crate::check::FnSig;
use crate::diag::{Diag, Diags};
use crate::smt::{self, Answer, Solver};
use crate::source::Span;

const I64_MIN: &str = "(- 9223372036854775808)";
const I64_MAX: &str = "9223372036854775807";
const U64_MAX: &str = "18446744073709551615";
const TWO_64: &str = "18446744073709551616";
const TWO_63: &str = "9223372036854775808";
/// `isize::MAX / size_of(T)` (ADR-0017 §3).
const LEN_MAX_WORD: &str = "1152921504606846975";
const LEN_MAX_BYTE: &str = "9223372036854775807";

/// Tables produced by the checker that the verifier needs.
pub struct Tables {
    pub fns: BTreeMap<String, FnSig>,
    pub types: BTreeMap<(u32, u32, u32), TypeKind>,
    pub recursive: BTreeMap<String, usize>,
    /// Declared structs and enums by full path.
    pub adts: BTreeMap<String, crate::check::Adt>,
    /// Canonical type of every parameter and local, by definition path.
    pub locals: BTreeMap<String, BTreeMap<String, TypeKind>>,
    /// Where each executable function frees its `Vec` values (ADR-0031 §2).
    pub moves: BTreeMap<String, crate::moves::MoveInfo>,
    /// The type arguments of every generic call, by the call's span.
    pub targs: BTreeMap<(u32, u32, u32), Vec<TypeKind>>,
    /// The `impl Trait for T` blocks (ADR-0046 §2), for `check::dispatch`.
    pub trait_impls: Vec<crate::check::TraitImplInfo>,
    /// The instances of generic spec fns the program reaches (ADR-0046 §4),
    /// by key `path<A, B>`: the generic path and the substitution of its
    /// parameters. Filled for the encoder (`prepare`).
    pub insts: BTreeMap<String, (String, BTreeMap<String, TypeKind>)>,
    /// The types instances and substituted contracts mention, for the
    /// datatype declarations.
    pub inst_types: Vec<TypeKind>,
}

impl Tables {
    /// The tables as the encoder sees them (ADR-0031 §3): a `Vec<T>` is a
    /// slice value, the pair of its element array and length, and
    /// `&Vec<T>` / `&mut Vec<T>` are slice references. The operations that
    /// change a length (`push`, `pop`, `truncate`, `Vec::new`) and a callee
    /// taking `&mut Vec<T>` are told apart by the program text. A `Box<T>`
    /// is its value `T` everywhere (ADR-0047 §3), also in fields, payloads
    /// and type arguments.
    fn slice_view(&self) -> Tables {
        fn view(t: &TypeKind) -> TypeKind {
            let sub = |x: &Type| Type {
                kind: view(&x.kind),
                span: x.span,
            };
            match t {
                TypeKind::Box(e) => view(&e.kind),
                TypeKind::Named { path, args } => TypeKind::Named {
                    path: path.clone(),
                    args: args.iter().map(sub).collect(),
                },
                TypeKind::Array(e, n) => TypeKind::Array(Box::new(sub(e)), *n),
                TypeKind::Vec(e) => TypeKind::Ref {
                    mutable: false,
                    inner: Box::new(Type {
                        kind: TypeKind::Slice(Box::new(sub(e))),
                        span: e.span,
                    }),
                },
                TypeKind::Ref { mutable, inner } => match &inner.kind {
                    TypeKind::Vec(e) => TypeKind::Ref {
                        mutable: *mutable,
                        inner: Box::new(Type {
                            kind: TypeKind::Slice(Box::new(sub(e))),
                            span: e.span,
                        }),
                    },
                    _ => TypeKind::Ref {
                        mutable: *mutable,
                        inner: Box::new(sub(inner)),
                    },
                },
                TypeKind::Slice(e) => TypeKind::Slice(Box::new(sub(e))),
                _ => t.clone(),
            }
        }
        let mut adts = self.adts.clone();
        for a in adts.values_mut() {
            match &mut a.kind {
                crate::check::AdtKind::Struct(fs) => {
                    for (_, ft) in fs.iter_mut() {
                        *ft = view(ft);
                    }
                }
                crate::check::AdtKind::Enum(vs) => {
                    for (_, ps) in vs.iter_mut() {
                        for pt in ps.iter_mut() {
                            *pt = view(pt);
                        }
                    }
                }
            }
        }
        let mut trait_impls = self.trait_impls.clone();
        for i in &mut trait_impls {
            i.head = view(&i.head);
        }
        let mut fns = self.fns.clone();
        for s in fns.values_mut() {
            s.params = s.params.iter().map(view).collect();
            s.ret = view(&s.ret);
        }
        Tables {
            fns,
            types: self.types.iter().map(|(k, t)| (*k, view(t))).collect(),
            recursive: self.recursive.clone(),
            adts,
            locals: self
                .locals
                .iter()
                .map(|(p, m)| {
                    (
                        p.clone(),
                        m.iter().map(|(n, t)| (n.clone(), view(t))).collect(),
                    )
                })
                .collect(),
            moves: BTreeMap::new(),
            targs: self
                .targs
                .iter()
                .map(|(k, ts)| (*k, ts.iter().map(view).collect()))
                .collect(),
            trait_impls,
            insts: BTreeMap::new(),
            inst_types: Vec::new(),
        }
    }
}

/// The key of an instance of generic definition `path`.
pub(crate) fn inst_key(path: &str, args: &[TypeKind]) -> String {
    format!(
        "{path}<{}>",
        args.iter()
            .map(crate::check::ty_str)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// The definition a call `path` from unit `mod_path` names.
fn resolve_in(t: &Tables, path: &[Ident], mod_path: &str) -> String {
    let name = path
        .iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::");
    let local = format!("{mod_path}::{name}");
    let local_type = path.len() == 2
        && (t
            .adts
            .contains_key(&format!("{mod_path}::{}", path[0].name))
            || t.fns.get(&local).is_some_and(|s| s.trait_of.is_some()));
    if (path.len() == 1 || local_type) && t.fns.contains_key(&local) {
        return local;
    }
    name
}

/// For a trait impl's function `path`: the trait method it implements and
/// the substitution that states the trait's contract for it (`Self` by the
/// implementing type, the method's own type parameters by the impl
/// function's), ADR-0046 §4.
pub(crate) fn refinement_of(
    t: &Tables,
    path: &str,
) -> Option<(String, BTreeMap<String, TypeKind>)> {
    let sig = t.fns.get(path)?;
    if !(sig.in_impl && path.starts_with('<')) {
        return None;
    }
    let name = path.rsplit("::").next()?;
    let i = t.trait_impls.iter().find(|i| i.fn_path(name) == path)?;
    let decl = format!("{}::{name}", i.trait_path);
    let dsig = t.fns.get(&decl)?;
    let mut sub = BTreeMap::from([("Self".to_string(), i.head.clone())]);
    for (a, b) in dsig.generics[dsig.impl_generics..]
        .iter()
        .zip(&sig.generics[sig.impl_generics..])
    {
        sub.insert(a.name.clone(), TypeKind::Param(b.name.clone()));
    }
    Some((decl, sub))
}

/// The definition a call of `target` with type arguments `args` runs, and
/// its type arguments: for a trait method with a known `Self`, its impl's
/// function (ADR-0046 §2).
fn dispatched(t: &Tables, target: String, args: Vec<TypeKind>) -> (String, Vec<TypeKind>) {
    crate::check::dispatch(&t.fns, &t.trait_impls, &target, &args).unwrap_or((target, args))
}

/// Every expression node of `e`, specification parts included.
fn each_expr<'e>(e: &'e Expr, f: &mut dyn FnMut(&'e Expr)) {
    f(e);
    let mut sub = |x: &'e Expr| each_expr(x, f);
    match &e.kind {
        ExprKind::Call { args, .. } | ExprKind::Array(args) => args.iter().for_each(&mut sub),
        ExprKind::Method { recv, args, .. } => {
            sub(recv);
            args.iter().for_each(sub);
        }
        ExprKind::Index { base, index } => {
            sub(base);
            sub(index);
        }
        ExprKind::Binary(_, l, r) => {
            sub(l);
            sub(r);
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Deref(x)
        | ExprKind::Trigger(x)
        | ExprKind::Repeat { value: x, .. }
        | ExprKind::Field { base: x, .. }
        | ExprKind::Forall { body: x, .. } => sub(x),
        ExprKind::If { cond, then, els } => {
            sub(cond);
            each_block_expr(then, f);
            if let Some(x) = els {
                each_expr(x, f);
            }
        }
        ExprKind::Block(b) => each_block_expr(b, f),
        ExprKind::StructLit { fields, rest, .. } => {
            fields.iter().for_each(|(_, x)| each_expr(x, f));
            if let Some(r) = rest {
                each_expr(r, f);
            }
        }
        ExprKind::Match { scrut, arms } => {
            each_expr(scrut, f);
            arms.iter().for_each(|a| each_expr(&a.body, f));
        }
        _ => {}
    }
}

fn each_block_expr<'e>(b: &'e Block, f: &mut dyn FnMut(&'e Expr)) {
    for s in &b.stmts {
        match s {
            Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => each_expr(init, f),
            Stmt::Assign { target, value, .. } => {
                each_expr(target, f);
                each_expr(value, f);
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                each_expr(cond, f);
                invariants.iter().for_each(|x| each_expr(x, f));
                if let Some(d) = decreases {
                    each_expr(d, f);
                }
                each_block_expr(body, f);
            }
            Stmt::Assert { cond, .. } => each_expr(cond, f),
            Stmt::Expr(e) => each_expr(e, f),
        }
    }
    if let Some(t) = &b.tail {
        each_expr(t, f);
    }
}

/// How deep type arguments may nest before instantiation stops (a generic
/// call that wraps its own parameter would never end; the checker reports
/// it).
pub(crate) const MAX_INST_DEPTH: usize = 8;

pub(crate) fn type_depth(t: &TypeKind) -> usize {
    match t {
        TypeKind::Array(e, _) | TypeKind::Slice(e) | TypeKind::Vec(e) | TypeKind::Box(e) => {
            1 + type_depth(&e.kind)
        }
        TypeKind::Ref { inner, .. } => 1 + type_depth(&inner.kind),
        TypeKind::Named { args, .. } => {
            1 + args.iter().map(|a| type_depth(&a.kind)).max().unwrap_or(0)
        }
        _ => 0,
    }
}

/// The encoder's tables (ADR-0031 §3 slice view) with the instances of
/// generic spec fns the program reaches (ADR-0046 §4): each definition's
/// generic calls, with type arguments in its own terms (a generic
/// definition is proved once over its parameters as sorts), then each
/// instance's body and each generic exec callee's contract under the
/// substitution of the call. Instances get signatures of their own.
fn prepare(p: &Program, t: &Tables) -> Tables {
    let mut view = t.slice_view();
    let defs = collect(p, t);
    let mut insts: BTreeMap<String, (String, BTreeMap<String, TypeKind>)> = BTreeMap::new();
    let mut seen_exec: BTreeSet<String> = BTreeSet::new();
    let mut types: Vec<TypeKind> = Vec::new();
    let mut work: Vec<(String, BTreeMap<String, TypeKind>, bool)> = defs
        .keys()
        .map(|k| (k.clone(), BTreeMap::new(), true))
        .collect();
    while let Some((path, sub, whole)) = work.pop() {
        // A trait impl's function is also checked against the trait's
        // contract, stated for it (its refinement obligations).
        if whole
            && sub.is_empty()
            && let Some((decl, rsub)) = refinement_of(&view, &path)
            && defs.contains_key(&decl)
        {
            work.push((decl, rsub, false));
        }
        let d = &defs[&path];
        let mut roots: Vec<&Expr> = d.f.requires.iter().chain(&d.f.ensures).collect();
        if whole {
            roots.extend(&d.f.decreases);
        }
        let mut nodes: Vec<&Expr> = Vec::new();
        for r in roots {
            each_expr(r, &mut |e| nodes.push(e));
        }
        if whole {
            each_block_expr(&d.f.body, &mut |e| nodes.push(e));
        }
        for e in nodes {
            let k = (e.span.file, e.span.lo, e.span.hi);
            if !sub.is_empty()
                && let Some(ty) = view.types.get(&k)
            {
                types.push(crate::check::subst(ty, &sub));
            }
            let ExprKind::Call { path: cp, .. } = &e.kind else {
                continue;
            };
            let Some(targs) = view.targs.get(&k) else {
                continue;
            };
            let args: Vec<TypeKind> = targs.iter().map(|a| crate::check::subst(a, &sub)).collect();
            if args.iter().any(|a| type_depth(a) > MAX_INST_DEPTH) {
                continue;
            }
            let (target, args) = dispatched(&view, resolve_in(&view, cp, &d.mod_path), args);
            // A trait call that runs a non-generic impl function.
            if args.is_empty() {
                continue;
            }
            let Some(sig) = view.fns.get(&target) else {
                continue;
            };
            let csub: BTreeMap<String, TypeKind> = sig
                .generics
                .iter()
                .map(|g| g.name.clone())
                .zip(args.iter().cloned())
                .collect();
            let key = inst_key(&target, &args);
            if sig.kind == FnKind::Spec {
                if let std::collections::btree_map::Entry::Vacant(v) = insts.entry(key) {
                    v.insert((target.clone(), csub.clone()));
                    work.push((target, csub, true));
                }
            } else if seen_exec.insert(key) {
                work.push((target, csub, false));
            }
        }
    }
    for (key, (base, sub)) in &insts {
        let mut sig = view.fns[base].clone();
        sig.path = key.clone();
        sig.params = sig
            .params
            .iter()
            .map(|x| crate::check::subst(x, sub))
            .collect();
        sig.ret = crate::check::subst(&sig.ret, sub);
        sig.generics = Vec::new();
        sig.impl_generics = 0;
        types.extend(sig.params.iter().cloned());
        types.push(sig.ret.clone());
        view.fns.insert(key.clone(), sig);
    }
    view.insts = insts;
    view.inst_types = types;
    view
}

/// The definitions of `p` and the spec fn instances of `t` (`prepare`).
fn prog_of<'p>(p: &'p Program, t: &'p Tables) -> Prog<'p> {
    let mut defs = collect(p, t);
    for (key, (base, sub)) in &t.insts {
        let d = &defs[base];
        let inst = Def {
            f: d.f,
            mod_path: d.mod_path.clone(),
            tsub: sub.clone(),
            inst: true,
        };
        defs.insert(key.clone(), inst);
    }
    let (_, dts) = enum_dts(t);
    let selectors = dts
        .values()
        .flat_map(|d| {
            d.ctors
                .iter()
                .flat_map(|(c, fs)| fs.iter().map(move |(sel, _)| (sel.clone(), c.clone())))
        })
        .collect();
    Prog { defs, t, selectors }
}

/// The sorts of a generic definition's type parameters (ADR-0046 §4) and a
/// value of each (the base of an empty array). A parameter bounded by
/// `Ord` has an uninterpreted total order `le` (reflexive, antisymmetric,
/// transitive, total: the contract of the built-in `Ord`).
fn sort_decls(sig: &FnSig) -> String {
    sig.generics
        .iter()
        .map(|g| {
            let s = sort_of(&TypeKind::Param(g.name.clone()));
            let mut d = format!(
                "(declare-sort {s} 0)\n(declare-const {} {s})\n",
                sym(&format!("'{}.default", g.name))
            );
            if g.bounds.iter().any(|b| b == "Ord") {
                let le = sym(&format!("'{}.le", g.name));
                d.push_str(&format!(
                    "(declare-fun {le} ({s} {s}) Bool)\n\
                     (assert (forall ((a {s})) (! ({le} a a) :pattern (({le} a a)))))\n\
                     (assert (forall ((a {s}) (b {s})) (! (=> (and ({le} a b) ({le} b a)) (= a b)) :pattern (({le} a b) ({le} b a)))))\n\
                     (assert (forall ((a {s}) (b {s}) (c {s})) (! (=> (and ({le} a b) ({le} b c)) ({le} a c)) :pattern (({le} a b) ({le} b c)))))\n\
                     (assert (forall ((a {s}) (b {s})) (! (or ({le} a b) ({le} b a)) :pattern (({le} a b)))))\n"
                ));
            }
            d
        })
        .collect()
}

/// An ordering comparison of `a` and `b` of type `t` (Rust `Ord`): the
/// sort's total order for a type parameter, `false < true` for `bool`,
/// integer order otherwise.
fn ord_term(op: BinOp, t: &TypeKind, a: &str, b: &str) -> String {
    match t {
        TypeKind::Param(n) => {
            let le = sym(&format!("'{n}.le"));
            match op {
                BinOp::Le => format!("({le} {a} {b})"),
                BinOp::Ge => format!("({le} {b} {a})"),
                BinOp::Lt => format!("(and ({le} {a} {b}) (not (= {a} {b})))"),
                _ => format!("(and ({le} {b} {a}) (not (= {a} {b})))"),
            }
        }
        TypeKind::Bool => match op {
            BinOp::Le => format!("(=> {a} {b})"),
            BinOp::Ge => format!("(=> {b} {a})"),
            BinOp::Lt => format!("(and (not {a}) {b})"),
            _ => format!("(and {a} (not {b}))"),
        },
        _ => {
            let o = match op {
                BinOp::Lt => "<",
                BinOp::Le => "<=",
                BinOp::Gt => ">",
                _ => ">=",
            };
            format!("({o} {a} {b})")
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnStatus {
    pub path: String,
    pub obligations: usize,
    pub failed: usize,
    /// Solver resource units over all obligations, and the largest single one
    /// (compare with the pinned rlimit to see how close a proof is to failing).
    pub rlimit_total: u64,
    pub rlimit_max: u64,
}

/// An obligation's identity: diagnostic code and source span `(file, lo, hi)`.
pub type ObKey = (&'static str, u32, u32, u32);

/// What proving found. The default is the outcome of not proving: no
/// statuses, nothing proven.
#[derive(Default)]
pub struct Outcome {
    pub statuses: Vec<FnStatus>,
    /// Obligations proved at every symbolic visit; code generation may drop
    /// the corresponding runtime check (DESIGN.md principle 9).
    pub proven: BTreeSet<ObKey>,
    /// SMT-LIB script per definition, as sent to the solver (without model queries).
    pub scripts: BTreeMap<String, String>,
    /// Under `vacuous_ends` only: the points of each definition whose path
    /// condition the solver refutes (backlog 1B-44).
    pub vacuous: Vec<(String, Span, &'static str)>,
}

thread_local! {
    /// Set by `vacuous_ends`: record reach points, check them instead of the
    /// obligations.
    static REACH: Cell<bool> = const { Cell::new(false) };
}

/// The points of a definition that its own encoding makes unreachable
/// (backlog 1B-44): each loop exit and the end of each executable body,
/// with the path condition there (contracts, revealed definitions, loop
/// invariants, callee postconditions) refuted by the solver. In a verified
/// function with satisfiable preconditions this cannot happen honestly:
/// proved obligations never trap, every loop has a decreasing measure, so
/// the end is reached. A refuted point means its proofs hold vacuously:
/// contradictory contracts, or a fault in the encoding (2G-05 found one).
/// An undecided point is not reported.
pub fn vacuous_ends(p: &Program, t: &Tables) -> Vec<(String, Span, &'static str)> {
    let prev = REACH.with(|r| r.replace(true));
    let o = verify_impl(p, t, &mut Diags::default(), 0, None, None, false, None);
    REACH.with(|r| r.set(prev));
    o.vacuous
}

#[derive(Debug, Clone, PartialEq)]
enum Val {
    S(String),
    A {
        elems: String,
        len: String,
    },
    Unit,
    /// A struct, field by field in declaration order (ADR-0027 §1).
    Struct(Vec<Val>),
    /// A slice of structs, one SMT array per leaf (ADR-0027 §2): `elem` has
    /// the element's shape with an array term at each leaf.
    Arr {
        elem: Box<Val>,
        len: String,
    },
}

impl Val {
    fn s(&self) -> String {
        match self {
            Val::S(t) => t.clone(),
            Val::Unit => "true".into(),
            Val::A { .. } => panic!("array used as scalar"),
            Val::Struct(_) | Val::Arr { .. } => panic!("data value used as scalar"),
        }
    }
}

/// Element `i` of a slice of structs.
fn select_val(elem: &Val, i: &str) -> Val {
    match elem {
        Val::S(a) => Val::S(format!("(select {a} {i})")),
        Val::Struct(fs) => Val::Struct(fs.iter().map(|f| select_val(f, i)).collect()),
        // A fixed-size array inside the element (backlog 2V-04b): its leaves
        // are arrays over the outer index; the length is a constant.
        Val::A { elems, len } => Val::A {
            elems: format!("(select {elems} {i})"),
            len: len.clone(),
        },
        Val::Arr { elem, len } => Val::Arr {
            elem: Box::new(select_val(elem, i)),
            len: len.clone(),
        },
        v => panic!("not an element template {v:?}"),
    }
}

/// The element arrays of a slice of structs with element `i` set to `v`.
fn store_val(elem: &Val, i: &str, v: &Val) -> Val {
    match (elem, v) {
        (Val::S(a), Val::S(x)) => Val::S(format!("(store {a} {i} {x})")),
        (Val::Struct(fs), Val::Struct(xs)) => {
            Val::Struct(fs.iter().zip(xs).map(|(f, x)| store_val(f, i, x)).collect())
        }
        (Val::A { elems, len }, Val::A { elems: x, .. }) => Val::A {
            elems: format!("(store {elems} {i} {x})"),
            len: len.clone(),
        },
        (Val::Arr { elem, len }, Val::Arr { elem: x, .. }) => Val::Arr {
            elem: Box::new(store_val(elem, i, x)),
            len: len.clone(),
        },
        (e, v) => panic!("element {v:?} does not match {e:?}"),
    }
}

/// The SMT terms a value is passed as (spec fn arguments): its scalars, an
/// array's elements and length, a struct's fields in order, recursively.
fn flatten(v: &Val, out: &mut Vec<String>) {
    match v {
        Val::S(t) => out.push(t.clone()),
        Val::A { elems, len } => {
            out.push(elems.clone());
            out.push(len.clone());
        }
        Val::Struct(fs) => fs.iter().for_each(|f| flatten(f, out)),
        Val::Arr { elem, len } => {
            flatten_elem(elem, out);
            out.push(len.clone());
        }
        Val::Unit => {}
    }
}

/// Some value of type `t` (backlog 2V-05: the base of an empty array): the
/// default scalar, each field's, the first variant with default payloads
/// that does not need a value of an enum being built (a recursive type,
/// ADR-0047: the checker ensures one exists, E0421).
fn default_val(t: &TypeKind, tb: &Tables) -> Val {
    default_in(t, tb, &mut Vec::new()).expect("a type with a finite value")
}

fn default_in(t: &TypeKind, tb: &Tables, building: &mut Vec<String>) -> Option<Val> {
    if let Some(fs) = struct_fields(t, tb) {
        let mut out = Vec::new();
        for (_, ft) in &fs {
            out.push(default_in(ft, tb, building)?);
        }
        return Some(Val::Struct(out));
    }
    if let Some(vs) = enum_variants(t, tb) {
        let ts = crate::check::ty_str(t);
        if building.contains(&ts) {
            return None;
        }
        building.push(ts);
        let found = vs.iter().find_map(|(v, ps)| {
            let mut leaves = Vec::new();
            for pt in ps {
                flatten(&default_in(pt, tb, building)?, &mut leaves);
            }
            let c = ctor(t, v);
            Some(Val::S(if leaves.is_empty() {
                c
            } else {
                format!("({c} {})", leaves.join(" "))
            }))
        });
        building.pop();
        return found;
    }
    Some(match t {
        TypeKind::Param(n) => Val::S(sym(&format!("'{n}.default"))),
        TypeKind::Array(e, n) => match const_elems(&e.kind, &default_val(&e.kind, tb), tb) {
            Val::S(elems) => Val::A {
                elems,
                len: n.to_string(),
            },
            elem => Val::Arr {
                elem: Box::new(elem),
                len: n.to_string(),
            },
        },
        TypeKind::Unit => Val::Unit,
        _ => Val::S(default_of(t).to_string()),
    })
}

/// The element arrays of an array whose every element is `v` (of type
/// `e`): one constant array, or one per leaf for an aggregate element.
fn const_elems(e: &TypeKind, v: &Val, tb: &Tables) -> Val {
    if !agg_elem(e, tb) {
        return Val::S(format!("((as const (Array Int {})) {})", sort_of(e), v.s()));
    }
    let mut ls = Vec::new();
    elem_leaves(e, tb, "", &mut ls);
    let mut terms = Vec::new();
    flatten_elem(v, &mut terms);
    let consts: Vec<String> = ls
        .iter()
        .zip(terms)
        .map(|((_, sort), x)| format!("((as const (Array Int {sort})) {x})"))
        .collect();
    elem_template(e, tb, &mut consts.into_iter())
}

/// The longest fixed array whose `==` is written element by element.
const UNROLL_EQ: usize = 16;

/// The leaf arrays of an element template (`elem_leaves` order): inner
/// fixed lengths are constants, not terms.
fn flatten_elem(v: &Val, out: &mut Vec<String>) {
    match v {
        Val::S(t) => out.push(t.clone()),
        Val::A { elems, .. } => out.push(elems.clone()),
        Val::Arr { elem, .. } => flatten_elem(elem, out),
        Val::Struct(fs) => fs.iter().for_each(|f| flatten_elem(f, out)),
        Val::Unit => {}
    }
}

/// Whether an array of `e` holds its elements as one array per leaf
/// (ADR-0027 §2): structs and fixed-size arrays (backlog 2V-04b).
fn agg_elem(e: &TypeKind, tb: &Tables) -> bool {
    struct_fields(e, tb).is_some() || matches!(e, TypeKind::Array(..))
}

/// The leaves of an element of type `e`, each with the sort of its value
/// for one element: a struct's fields in order, an inner array's leaves
/// one array level deeper (its fixed length is a constant).
fn elem_leaves(e: &TypeKind, tb: &Tables, suffix: &str, out: &mut Vec<(String, String)>) {
    if let Some(fs) = struct_fields(e, tb) {
        for (n, ft) in &fs {
            elem_leaves(ft, tb, &format!("{suffix}.{n}"), out);
        }
        return;
    }
    match e {
        TypeKind::Unit => {}
        TypeKind::Array(inner, _) => {
            let mut el = Vec::new();
            elem_leaves(&inner.kind, tb, &format!("{suffix}.e"), &mut el);
            out.extend(el.into_iter().map(|(n, s)| (n, format!("(Array Int {s})"))));
        }
        t => out.push((suffix.to_string(), sort_of(t))),
    }
}

/// The element template of type `e` whose leaf arrays are `terms` (the
/// inverse of `flatten_elem`).
fn elem_template(e: &TypeKind, tb: &Tables, terms: &mut impl Iterator<Item = String>) -> Val {
    if let Some(fs) = struct_fields(e, tb) {
        return Val::Struct(
            fs.iter()
                .map(|(_, ft)| elem_template(ft, tb, terms))
                .collect(),
        );
    }
    match e {
        TypeKind::Unit => Val::Unit,
        TypeKind::Array(inner, n) if agg_elem(&inner.kind, tb) => Val::Arr {
            elem: Box::new(elem_template(&inner.kind, tb, terms)),
            len: n.to_string(),
        },
        TypeKind::Array(_, n) => Val::A {
            elems: terms.next().expect("array leaf"),
            len: n.to_string(),
        },
        _ => Val::S(terms.next().expect("scalar leaf")),
    }
}

/// One SMT leaf of a value: a scalar, or an array's elements or length
/// (`Some(n)` for a fixed-size array).
enum Leaf {
    Scalar(TypeKind),
    Elems(TypeKind),
    /// One leaf array of an array of aggregates: the sort of one element's leaf.
    Leaves(String),
    Len(TypeKind, Option<u64>),
}

impl Leaf {
    fn sort(&self) -> String {
        match self {
            Leaf::Scalar(t) => sort_of(t),
            Leaf::Elems(e) => format!("(Array Int {})", sort_of(e)),
            Leaf::Leaves(s) => format!("(Array Int {s})"),
            Leaf::Len(..) => "Int".into(),
        }
    }

    /// What every value of the leaf satisfies (ADR-0017 §3).
    fn fact(&self, x: &str) -> Option<String> {
        match self {
            Leaf::Scalar(t) => range(t, x),
            Leaf::Elems(_) | Leaf::Leaves(_) => None,
            Leaf::Len(_, Some(n)) => Some(format!("(= {x} {n})")),
            Leaf::Len(e, None) => Some(format!("(and (<= 0 {x}) (<= {x} {}))", len_max(e))),
        }
    }
}

/// The fields of the struct `t` denotes (through a reference), in
/// declaration order.
/// A generic struct's field types are those of the instance `t`.
pub(crate) fn struct_fields(t: &TypeKind, tb: &Tables) -> Option<Vec<(String, TypeKind)>> {
    let t = match t {
        TypeKind::Ref { inner, .. } => &inner.kind,
        t => t,
    };
    let TypeKind::Named { path, .. } = t else {
        return None;
    };
    match tb.adts.get(&path.join("::")) {
        Some(
            a @ crate::check::Adt {
                kind: crate::check::AdtKind::Struct(fs),
                ..
            },
        ) => {
            let m = crate::check::adt_subst(a, t);
            Some(
                fs.iter()
                    .map(|(n, ft)| (n.clone(), crate::check::subst(ft, &m)))
                    .collect(),
            )
        }
        _ => None,
    }
}

/// The variants of the enum `t` denotes (through a reference), in
/// declaration order, with their payload types. `Option` is `None, Some`,
/// `Result` is `Ok, Err` (ADR-0026).
pub(crate) fn enum_variants(t: &TypeKind, tb: &Tables) -> Option<Vec<(String, Vec<TypeKind>)>> {
    let t = match t {
        TypeKind::Ref { inner, .. } => &inner.kind,
        t => t,
    };
    let TypeKind::Named { path, args } = t else {
        return None;
    };
    match (path.join("::").as_str(), args.as_slice()) {
        ("Option", [a]) => Some(vec![
            ("None".into(), Vec::new()),
            ("Some".into(), vec![a.kind.clone()]),
        ]),
        ("Result", [a, b]) => Some(vec![
            ("Ok".into(), vec![a.kind.clone()]),
            ("Err".into(), vec![b.kind.clone()]),
        ]),
        (p, _) => match tb.adts.get(p) {
            Some(
                a @ crate::check::Adt {
                    kind: crate::check::AdtKind::Enum(vs),
                    ..
                },
            ) => {
                let m = crate::check::adt_subst(a, t);
                Some(
                    vs.iter()
                        .map(|(n, ps)| {
                            (
                                n.clone(),
                                ps.iter().map(|p| crate::check::subst(p, &m)).collect(),
                            )
                        })
                        .collect(),
                )
            }
            _ => None,
        },
    }
}

/// The type without a reference around it.
pub(crate) fn value_ty(t: &TypeKind) -> &TypeKind {
    match t {
        TypeKind::Ref { inner, .. } if !matches!(inner.kind, TypeKind::Slice(_)) => &inner.kind,
        t => t,
    }
}

/// The constructor symbol of variant `v` of enum `t` (ADR-0027 §3).
fn ctor(t: &TypeKind, v: &str) -> String {
    sym(&format!("{}::{v}", crate::check::ty_str(value_ty(t))))
}

/// The selector leaves of payload position `k` of variant `v`.
fn payload_leaves(
    t: &TypeKind,
    v: &str,
    k: usize,
    pt: &TypeKind,
    tb: &Tables,
) -> Vec<(String, Leaf)> {
    let mut ls = Vec::new();
    let base = format!("{}::{v}.{k}", crate::check::ty_str(value_ty(t)));
    leaves(pt, tb, &base, &mut ls);
    ls
}

/// Whether a value of type `t` holds an array anywhere (its SMT equality
/// would compare elements past the length).
fn has_array(t: &TypeKind, tb: &Tables) -> bool {
    fn go(t: &TypeKind, tb: &Tables, seen: &mut BTreeSet<String>) -> bool {
        if is_array(t) {
            return true;
        }
        // A recursive type is visited once.
        if !seen.insert(crate::check::ty_str(t)) {
            return false;
        }
        if let Some(fs) = struct_fields(t, tb) {
            return fs.iter().any(|(_, ft)| go(ft, tb, seen));
        }
        enum_variants(t, tb).is_some_and(|vs| {
            vs.iter()
                .any(|(_, ps)| ps.iter().any(|pt| go(pt, tb, seen)))
        })
    }
    go(t, tb, &mut BTreeSet::new())
}

/// An enum instance as the encoder declares it: constructor symbols and
/// their fields (selector symbol, sort).
struct Dt {
    ctors: Vec<(String, Vec<(String, String)>)>,
}

/// Every enum instance a program uses, by type text, and the order that
/// lists each one after the enums its payloads name.
fn enum_dts(tb: &Tables) -> (Vec<String>, BTreeMap<String, Dt>) {
    fn visit(t: &TypeKind, tb: &Tables, order: &mut Vec<String>, dts: &mut BTreeMap<String, Dt>) {
        match t {
            TypeKind::Array(e, _) | TypeKind::Slice(e) => return visit(&e.kind, tb, order, dts),
            TypeKind::Ref { inner, .. } => return visit(&inner.kind, tb, order, dts),
            TypeKind::Named { .. } => {}
            _ => return,
        }
        if let Some(fs) = struct_fields(t, tb) {
            for (_, ft) in &fs {
                visit(ft, tb, order, dts);
            }
            return;
        }
        let Some(vs) = enum_variants(t, tb) else {
            return;
        };
        let ts = crate::check::ty_str(t);
        if dts.contains_key(&ts) {
            return;
        }
        dts.insert(ts.clone(), Dt { ctors: Vec::new() });
        let mut ctors = Vec::new();
        for (v, ps) in &vs {
            let mut fields = Vec::new();
            for (k, pt) in ps.iter().enumerate() {
                visit(pt, tb, order, dts);
                for (name, leaf) in payload_leaves(t, v, k, pt, tb) {
                    fields.push((sym(&name), leaf.sort()));
                }
            }
            ctors.push((ctor(t, v), fields));
        }
        dts.get_mut(&ts).expect("inserted").ctors = ctors;
        order.push(ts);
    }
    let mut all: Vec<&TypeKind> = tb.types.values().collect();
    all.extend(tb.inst_types.iter());
    all.extend(tb.locals.values().flat_map(|m| m.values()));
    for sig in tb.fns.values() {
        all.extend(sig.params.iter());
        all.push(&sig.ret);
    }
    let mut order = Vec::new();
    let mut dts: BTreeMap<String, Dt> = BTreeMap::new();
    for t in all {
        visit(t, tb, &mut order, &mut dts);
    }
    (order, dts)
}

/// Every enum instance a program uses, as (constructor prefix, sort,
/// `declare-datatypes`), dependencies first. Recursive enums (ADR-0047 §3,
/// through a `Box`, which the encoder does not see) are declared together
/// with the enums of their cycle, under the first one's entry (the others'
/// entries name it).
fn datatypes(tb: &Tables) -> Vec<(String, String, String)> {
    let (order, dts) = enum_dts(tb);
    // The enums each one's fields name (by sort symbol).
    let deps: BTreeMap<String, Vec<String>> = dts
        .iter()
        .map(|(ts, d)| {
            let sorts: Vec<&String> = d
                .ctors
                .iter()
                .flat_map(|(_, fs)| fs.iter().map(|(_, s)| s))
                .collect();
            let named = order
                .iter()
                .filter(|o| {
                    let s = sym(o);
                    sorts.iter().any(|x| x.contains(s.as_str()))
                })
                .cloned()
                .collect();
            (ts.clone(), named)
        })
        .collect();
    // Tarjan's components, dependencies first.
    struct Tarjan<'a> {
        deps: &'a BTreeMap<String, Vec<String>>,
        index: BTreeMap<String, usize>,
        low: BTreeMap<String, usize>,
        stack: Vec<String>,
        on: BTreeSet<String>,
        out: Vec<Vec<String>>,
    }
    impl Tarjan<'_> {
        fn go(&mut self, v: &str) {
            let i = self.index.len();
            self.index.insert(v.to_string(), i);
            self.low.insert(v.to_string(), i);
            self.stack.push(v.to_string());
            self.on.insert(v.to_string());
            for w in self.deps[v].clone() {
                if !self.index.contains_key(&w) {
                    self.go(&w);
                    let lw = self.low[&w];
                    let lv = self.low.get_mut(v).expect("visited");
                    *lv = (*lv).min(lw);
                } else if self.on.contains(&w) {
                    let iw = self.index[&w];
                    let lv = self.low.get_mut(v).expect("visited");
                    *lv = (*lv).min(iw);
                }
            }
            if self.low[v] == self.index[v] {
                let mut comp = Vec::new();
                while let Some(w) = self.stack.pop() {
                    self.on.remove(&w);
                    let done = w == v;
                    comp.push(w);
                    if done {
                        break;
                    }
                }
                comp.reverse();
                self.out.push(comp);
            }
        }
    }
    let mut tj = Tarjan {
        deps: &deps,
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        stack: Vec::new(),
        on: BTreeSet::new(),
        out: Vec::new(),
    };
    for ts in &order {
        if !tj.index.contains_key(ts) {
            tj.go(ts);
        }
    }
    let ctor_text = |c: &String, fs: &Vec<(String, String)>| {
        if fs.is_empty() {
            format!("({c})")
        } else {
            let f: Vec<String> = fs.iter().map(|(n, s)| format!("({n} {s})")).collect();
            format!("({c} {})", f.join(" "))
        }
    };
    let mut out = Vec::new();
    for comp in tj.out {
        let heads: Vec<String> = comp.iter().map(|ts| format!("({} 0)", sym(ts))).collect();
        let bodies: Vec<String> = comp
            .iter()
            .map(|ts| {
                let cs: Vec<String> = dts[ts]
                    .ctors
                    .iter()
                    .map(|(c, fs)| ctor_text(c, fs))
                    .collect();
                format!("({})", cs.join(" "))
            })
            .collect();
        let decl = format!(
            "(declare-datatypes ({}) ({}))",
            heads.join(" "),
            bodies.join(" ")
        );
        let first = sym(&comp[0]);
        out.push((format!("|{}::", comp[0]), first.clone(), decl));
        for ts in &comp[1..] {
            out.push((
                format!("|{ts}::"),
                sym(ts),
                format!("; {} is declared with {first}", sym(ts)),
            ));
        }
    }
    out
}

/// The SMT functions of spec fn `path`: one, or one per leaf of a struct
/// result (`|geo::mid.x|`), with their result sorts.
fn spec_funs(path: &str, tb: &Tables) -> Vec<(String, String)> {
    let ret = &tb.fns[path].ret;
    if struct_fields(ret, tb).is_none() {
        return vec![(sym(path), sort_of(ret))];
    }
    let mut ls = Vec::new();
    leaves(ret, tb, path, &mut ls);
    ls.iter().map(|(n, l)| (sym(n), l.sort())).collect()
}

/// What marks spec fn `path` in a query text: its symbol, or the prefix of
/// its leaf symbols.
fn spec_key(path: &str, tb: &Tables) -> String {
    if struct_fields(&tb.fns[path].ret, tb).is_some() {
        format!("|{path}.")
    } else {
        sym(path)
    }
}

fn field_index(t: &TypeKind, tb: &Tables, name: &str) -> usize {
    struct_fields(t, tb)
        .and_then(|fs| fs.iter().position(|(n, _)| n == name))
        .unwrap_or_else(|| panic!("no field `{name}` in {t:?}"))
}

/// The leaves of a value of type `t`, named `base` + a suffix, in the order
/// `flatten` writes them.
fn leaves(t: &TypeKind, tb: &Tables, base: &str, out: &mut Vec<(String, Leaf)>) {
    if let Some(fs) = struct_fields(t, tb) {
        for (n, ft) in &fs {
            leaves(ft, tb, &format!("{base}.{n}"), out);
        }
        return;
    }
    match t {
        TypeKind::Unit => {}
        _ if is_array(t) => {
            let e = elem_ty(t);
            let n = match t {
                TypeKind::Array(_, n) => Some(*n),
                _ => None,
            };
            // A struct or array element: one array per leaf (ADR-0027 §2).
            if agg_elem(&e, tb) {
                let mut el = Vec::new();
                elem_leaves(&e, tb, "", &mut el);
                for (suf, sort) in el {
                    out.push((format!("{base}.e{suf}"), Leaf::Leaves(sort)));
                }
            } else {
                out.push((format!("{base}.e"), Leaf::Elems(e.clone())));
            }
            out.push((format!("{base}.len"), Leaf::Len(e, n)));
        }
        TypeKind::Ref { inner, .. } => leaves(&inner.kind, tb, base, out),
        _ => out.push((base.to_string(), Leaf::Scalar(t.clone()))),
    }
}

/// The value of type `t` whose leaves are `terms` (the inverse of `flatten`).
fn build(t: &TypeKind, tb: &Tables, terms: &mut impl Iterator<Item = String>) -> Val {
    if let Some(fs) = struct_fields(t, tb) {
        return Val::Struct(fs.iter().map(|(_, ft)| build(ft, tb, terms)).collect());
    }
    match t {
        TypeKind::Unit => Val::Unit,
        _ if is_array(t) => {
            let e = elem_ty(t);
            if agg_elem(&e, tb) {
                let elem = Box::new(elem_template(&e, tb, terms));
                let len = terms.next().expect("length leaf");
                return Val::Arr { elem, len };
            }
            let elems = terms.next().expect("array leaf");
            let len = terms.next().expect("length leaf");
            Val::A { elems, len }
        }
        TypeKind::Ref { inner, .. } => build(&inner.kind, tb, terms),
        _ => Val::S(terms.next().expect("scalar leaf")),
    }
}

/// A step from a variable to the place an assignment writes.
enum Step {
    Field(usize),
    Index(String),
}

/// `v` with the place at `steps` replaced by `new`.
fn set_at(v: Val, steps: &[Step], new: Val) -> Val {
    match (steps.split_first(), v) {
        (None, _) => new,
        (Some((Step::Field(k), rest)), Val::Struct(mut fs)) => {
            let f = std::mem::replace(&mut fs[*k], Val::Unit);
            fs[*k] = set_at(f, rest, new);
            Val::Struct(fs)
        }
        (Some((Step::Index(i), rest)), Val::Arr { elem, len }) => {
            let cur = select_val(&elem, i);
            let new = set_at(cur, rest, new);
            Val::Arr {
                elem: Box::new(store_val(&elem, i, &new)),
                len,
            }
        }
        (Some((Step::Index(i), rest)), Val::A { elems, len }) => {
            assert!(rest.is_empty(), "array elements are scalars");
            Val::A {
                elems: format!("(store {elems} {i} {})", new.s()),
                len,
            }
        }
        (_, v) => panic!("place does not match value {v:?}"),
    }
}

/// `ite(c, a, b)` on values of one type, leaf by leaf.
fn ite_val(c: &str, a: &Val, b: &Val) -> Val {
    match (a, b) {
        (Val::S(x), Val::S(y)) => Val::S(ite(c, x, y)),
        (Val::A { elems: e1, len: l1 }, Val::A { elems: e2, len: l2 }) => Val::A {
            elems: ite(c, e1, e2),
            len: ite(c, l1, l2),
        },
        (Val::Struct(xs), Val::Struct(ys)) => {
            Val::Struct(xs.iter().zip(ys).map(|(x, y)| ite_val(c, x, y)).collect())
        }
        (Val::Arr { elem: e1, len: l1 }, Val::Arr { elem: e2, len: l2 }) => Val::Arr {
            elem: Box::new(ite_val(c, e1, e2)),
            len: ite(c, l1, l2),
        },
        _ => Val::Unit,
    }
}

struct Obl {
    code: &'static str,
    span: Span,
    msg: String,
    assumptions: Vec<String>,
    goal: String,
}

struct Def<'p> {
    f: &'p FnDef,
    mod_path: String,
    /// An instance of a generic spec fn: the substitution of its type
    /// parameters (empty otherwise: a generic definition is proved over its
    /// parameters as sorts).
    tsub: BTreeMap<String, TypeKind>,
    /// An instance (`Tables::insts`): declared and revealed, not proved.
    inst: bool,
}

struct Prog<'p> {
    defs: BTreeMap<String, Def<'p>>,
    t: &'p Tables,
    /// The constructor of every enum selector symbol (structural
    /// `decreases`, ADR-0047 §3).
    selectors: BTreeMap<String, String>,
}

/// Symbolic state of one definition.
struct Enc<'p, 'q> {
    prog: &'q Prog<'p>,
    path: String,
    mod_path: String,
    /// The substitution of the type parameters of the definition whose text
    /// is being translated (a callee's contract, an instance's body) into
    /// the query's types.
    tsub: BTreeMap<String, TypeKind>,
    /// Unit whose names the specification being translated refers to.
    spec_mod: String,
    decls: Vec<String>,
    counters: BTreeMap<String, u32>,
    pc: Vec<String>,
    vars: BTreeMap<String, Val>,
    old: BTreeMap<String, Val>,
    entry_measure: Option<(String, bool)>,
    obls: Vec<Obl>,
    /// Entry constants of the parameters, for counterexamples: (name, terms,
    /// type).
    inputs: Vec<(String, Val, TypeKind)>,
    /// Spec translation: enclosing conditions, and the recursion check context.
    spec_pc: Vec<String>,
    term_check: Option<(usize, String, bool)>,
    /// The `forall`s around the specification being translated: bound
    /// declarations, their range facts, and `spec_pc`'s length on entry.
    foralls: Vec<(Vec<String>, Vec<String>, usize)>,
    /// Per enclosing `forall`, its `#[trigger]` terms: each one a list of
    /// alternatives (a struct-valued trigger has one per leaf).
    triggers: Vec<Vec<Vec<String>>>,
    bound: u32,
    /// Under `vacuous_ends`: each reach point and its path condition.
    reach: Vec<(Span, &'static str, Vec<String>)>,
}

/// `(f a)` as (`f`, `a`): an application of a symbol to one argument (a
/// selector application in a measure term).
fn sel_app(t: &str) -> Option<(&str, &str)> {
    let inner = t.strip_prefix('(')?.strip_suffix(')')?;
    let head_end = if let Some(q) = inner.strip_prefix('|') {
        q.find('|')? + 2
    } else {
        inner.find(' ')?
    };
    let (head, rest) = inner.split_at(head_end);
    let arg = rest.strip_prefix(' ')?;
    // One balanced argument.
    let mut depth = 0i32;
    let mut quoted = false;
    for (i, c) in arg.char_indices() {
        match c {
            '|' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            ' ' if !quoted && depth == 0 && i + 1 < arg.len() => return None,
            _ => {}
        }
    }
    Some((head, arg))
}

fn num(v: i128) -> String {
    if v < 0 {
        format!("(- {})", -v)
    } else {
        v.to_string()
    }
}

fn sort_of(t: &TypeKind) -> String {
    match t {
        TypeKind::Bool => "Bool".into(),
        // A type parameter is an uninterpreted sort (ADR-0046 §4).
        TypeKind::Param(n) => sym(&format!("'{n}")),
        // An enum is a datatype named after its type (ADR-0027 §3).
        TypeKind::Named { .. } => sym(&crate::check::ty_str(t)),
        TypeKind::Ref { inner, .. } if !matches!(inner.kind, TypeKind::Slice(_)) => {
            sort_of(&inner.kind)
        }
        _ => "Int".into(),
    }
}

fn elem_ty(t: &TypeKind) -> TypeKind {
    match t {
        TypeKind::Array(e, _) => e.kind.clone(),
        TypeKind::Ref { inner, .. } => match &inner.kind {
            TypeKind::Slice(e) => e.kind.clone(),
            other => other.clone(),
        },
        TypeKind::Slice(e) => e.kind.clone(),
        other => other.clone(),
    }
}

fn is_array(t: &TypeKind) -> bool {
    matches!(t, TypeKind::Array(..))
        || matches!(t, TypeKind::Ref { inner, .. } if matches!(inner.kind, TypeKind::Slice(_)))
}

fn range(t: &TypeKind, x: &str) -> Option<String> {
    match t {
        TypeKind::I64 => Some(format!("(and (<= {I64_MIN} {x}) (<= {x} {I64_MAX}))")),
        TypeKind::U64 | TypeKind::Usize => Some(format!("(and (<= 0 {x}) (<= {x} {U64_MAX}))")),
        TypeKind::U8 => Some(format!("(and (<= 0 {x}) (<= {x} 255))")),
        _ => None,
    }
}

fn len_max(elem: &TypeKind) -> &'static str {
    // A data type may be one byte (a fieldless enum, a struct of a bool):
    // only `isize::MAX` is known without its layout.
    if matches!(
        elem,
        TypeKind::Bool | TypeKind::U8 | TypeKind::Named { .. } | TypeKind::Param(_)
    ) {
        LEN_MAX_BYTE
    } else {
        LEN_MAX_WORD
    }
}

fn and(xs: &[String]) -> String {
    match xs.len() {
        0 => "true".into(),
        1 => xs[0].clone(),
        _ => format!("(and {})", xs.join(" ")),
    }
}

fn ite(c: &str, a: &str, b: &str) -> String {
    if a == b {
        a.to_string()
    } else {
        format!("(ite {c} {a} {b})")
    }
}

fn wrap(t: &TypeKind, x: &str) -> String {
    match t {
        TypeKind::I64 => format!("(- (mod (+ {x} {TWO_63}) {TWO_64}) {TWO_63})"),
        TypeKind::U8 => format!("(mod {x} 256)"),
        _ => format!("(mod {x} {TWO_64})"),
    }
}

/// A built-in integer method `a.op(b)` on type `t` (ADR-0028 §3, §5).
enum IntMethod {
    /// `wrapping_*`, `saturating_*`: a value of `t`.
    Value(String),
    /// `checked_*`: `Some(v)` exactly when `ok`; `lemmas` are the division
    /// facts of `v` (true for every divisor).
    Checked {
        ok: String,
        v: String,
        lemmas: Vec<String>,
    },
}

fn int_method(op: &str, t: &TypeKind, a: &str, b: &str) -> IntMethod {
    let signed = *t == TypeKind::I64;
    let (kind, base) = op.split_once('_').expect("integer method");
    let (v, lemmas) = match base {
        "add" => (format!("(+ {a} {b})"), Vec::new()),
        "sub" => (format!("(- {a} {b})"), Vec::new()),
        "mul" => (format!("(* {a} {b})"), Vec::new()),
        "div" | "rem" => {
            let (q, r, ls) = if signed {
                (
                    format!("(bmb_tdiv {a} {b})"),
                    format!("(bmb_trem {a} {b})"),
                    SIGNED_DIV_LEMMAS,
                )
            } else {
                (
                    format!("(div {a} {b})"),
                    format!("(mod {a} {b})"),
                    UNSIGNED_DIV_LEMMAS,
                )
            };
            let lemmas = ls.iter().map(|l| instantiate(l, a, b, &q, &r)).collect();
            (if base == "div" { q } else { r }, lemmas)
        }
        _ => unreachable!("integer method {op}"),
    };
    let (min, max) = match t {
        TypeKind::I64 => (I64_MIN, I64_MAX),
        TypeKind::U8 => ("0", "255"),
        _ => ("0", U64_MAX),
    };
    match kind {
        "wrapping" => IntMethod::Value(wrap(t, &v)),
        "saturating" => IntMethod::Value(format!(
            "(ite (> {v} {max}) {max} (ite (< {v} {min}) {min} {v}))"
        )),
        _ => {
            let in_range = range(t, &v).expect("integer type");
            let ok = match base {
                "div" => format!("(and (not (= {b} 0)) {in_range})"),
                // `i64::MIN % -1` is 0 in mathematics but overflows in Rust.
                "rem" if signed => {
                    format!("(and (not (= {b} 0)) (not (and (= {a} {I64_MIN}) (= {b} (- 1)))))")
                }
                "rem" => format!("(not (= {b} 0))"),
                _ => in_range,
            };
            IntMethod::Checked { ok, v, lemmas }
        }
    }
}

/// The SMT array of a byte string (ADR-0044 §2): elements past the end
/// are 0, as the encoding of any array's tail is unconstrained by use.
fn byte_array(v: &[u8]) -> String {
    let mut a = "((as const (Array Int Int)) 0)".to_string();
    for (k, b) in v.iter().enumerate() {
        a = format!("(store {a} {k} {b})");
    }
    a
}

fn sym(name: &str) -> String {
    format!("|{name}|")
}

impl<'p, 'q> Enc<'p, 'q> {
    /// `Some(v)` when `ok`, else `None`, of `e`'s `Option` type.
    fn checked(&mut self, e: &Expr, ok: &str, v: String) -> Val {
        let opt = self.ty(e);
        let some = self.variant_value(&opt, "Some", &[Val::S(v)]).s();
        let none = self.variant_value(&opt, "None", &[]).s();
        Val::S(ite(ok, &some, &none))
    }

    fn new(prog: &'q Prog<'p>, path: &str, mod_path: &str) -> Self {
        Enc {
            prog,
            path: path.to_string(),
            mod_path: mod_path.to_string(),
            tsub: prog
                .defs
                .get(path)
                .map(|d| d.tsub.clone())
                .unwrap_or_default(),
            spec_mod: mod_path.to_string(),
            decls: Vec::new(),
            counters: BTreeMap::new(),
            pc: Vec::new(),
            vars: BTreeMap::new(),
            old: BTreeMap::new(),
            entry_measure: None,
            obls: Vec::new(),
            inputs: Vec::new(),
            spec_pc: Vec::new(),
            term_check: None,
            foralls: Vec::new(),
            triggers: Vec::new(),
            bound: 0,
            reach: Vec::new(),
        }
    }

    /// A function's `decreases` measure `d`: its term, and whether it is
    /// the value of a recursive enum (structural, ADR-0047 §3) rather than
    /// an integer.
    fn measure(
        &mut self,
        d: &Expr,
        env: &BTreeMap<String, Val>,
        old: &BTreeMap<String, Val>,
    ) -> (String, bool) {
        let v = self.spec(d, env, old).s();
        let t = self.ty(d);
        (v, enum_variants(value_ty(&t), self.prog.t).is_some())
    }

    /// The goal that measure `m1` of a call is below the caller's `m0`. An
    /// integer measure is non-negative and smaller. A structural one is a
    /// part of the caller's value: a chain of selectors from `m0`, each
    /// applied to a value of its constructor (so a strict subterm of a
    /// finite datatype value); any other term fails (it may not be smaller).
    fn decrease_goal(&self, m0: &(String, bool), m1: &(String, bool)) -> String {
        match (m0, m1) {
            ((a, false), (b, false)) => format!("(and (<= 0 {a}) (< {b} {a}))"),
            ((a, true), (b, true)) => {
                let mut testers = Vec::new();
                let mut cur = b.as_str();
                while let Some((head, arg)) = sel_app(cur) {
                    let Some(c) = self.prog.selectors.get(head) else {
                        break;
                    };
                    testers.push(format!("((_ is {c}) {arg})"));
                    cur = arg;
                    if cur == a {
                        return and(&testers);
                    }
                }
                "false".into()
            }
            _ => "false".into(),
        }
    }

    fn ty(&self, e: &Expr) -> TypeKind {
        let t = self
            .prog
            .t
            .types
            .get(&(e.span.file, e.span.lo, e.span.hi))
            .cloned()
            .unwrap_or(TypeKind::Error);
        crate::check::subst(&t, &self.tsub)
    }

    /// The type arguments of the generic call at `span`, in the query's
    /// types, and the substitution of the callee's parameters by them.
    fn call_subst(&self, target: &str, span: Span) -> BTreeMap<String, TypeKind> {
        let Some(targs) = self.prog.t.targs.get(&(span.file, span.lo, span.hi)) else {
            return BTreeMap::new();
        };
        self.prog.t.fns[target]
            .generics
            .iter()
            .map(|g| g.name.clone())
            .zip(targs.iter().map(|a| crate::check::subst(a, &self.tsub)))
            .collect()
    }

    /// The spec fn a call names: an instance for a generic one.
    fn spec_target(&self, target: String, span: Span) -> String {
        let sub = self.call_subst(&target, span);
        if sub.is_empty() {
            return target;
        }
        let args: Vec<TypeKind> = self.prog.t.fns[&target]
            .generics
            .iter()
            .map(|g| sub[&g.name].clone())
            .collect();
        // A trait's spec method with a known `Self` is its impl's.
        let (target, args) = dispatched(self.prog.t, target, args);
        if args.is_empty() {
            target
        } else {
            inst_key(&target, &args)
        }
    }

    fn fresh(&mut self, base: &str, sort: &str) -> String {
        let n = self.counters.entry(base.to_string()).or_insert(0);
        let name = sym(&format!("{base}@{n}"));
        *n += 1;
        self.decls.push(format!("(declare-const {name} {sort})"));
        name
    }

    /// A fresh array of elements of type `e`. Every element is within its
    /// type's range, also where only a specification reads it (backlog 1B-30).
    /// `i64` elements get the fact only when `signed` is set: for parameters
    /// and declared locals, where a spec fn over an element unfolds only when
    /// its argument is known to be in range (1B-37, ADR-0017 §4). After a call
    /// or loop havoc the callee's contract or the invariant states the range
    /// it needs; the extra quantifier there cost proofs their rlimit (1B-37).
    fn fresh_elems(&mut self, base: &str, e: &TypeKind, signed: bool) -> String {
        let elems = self.fresh(base, &format!("(Array Int {})", sort_of(e)));
        let wanted = matches!(e, TypeKind::U64 | TypeKind::Usize | TypeKind::U8)
            || (signed && matches!(e, TypeKind::I64));
        if wanted && let Some(r) = range(e, &format!("(select {elems} k)")) {
            self.pc.push(format!(
                "(forall ((k Int)) (! {r} :pattern ((select {elems} k))))"
            ));
        }
        elems
    }

    /// Fresh element arrays of a slice of structs: one per leaf (element
    /// ranges are assumed where an element is read).
    fn fresh_struct_elems(&mut self, base: &str, e: &TypeKind) -> Val {
        let mut ls = Vec::new();
        elem_leaves(e, self.prog.t, &format!("{base}.e"), &mut ls);
        let terms: Vec<String> = ls
            .iter()
            .map(|(name, sort)| self.fresh(name, &format!("(Array Int {sort})")))
            .collect();
        elem_template(e, self.prog.t, &mut terms.into_iter())
    }

    fn fresh_val(&mut self, base: &str, t: &TypeKind) -> Val {
        if let Some(fs) = struct_fields(t, self.prog.t) {
            let fs = fs.to_vec();
            return Val::Struct(
                fs.iter()
                    .map(|(n, ft)| self.fresh_val(&format!("{base}.{n}"), ft))
                    .collect(),
            );
        }
        if let TypeKind::Ref { inner, .. } = t
            && !is_array(t)
        {
            return self.fresh_val(base, &inner.kind);
        }
        if is_array(t) && agg_elem(&elem_ty(t), self.prog.t) {
            let e = elem_ty(t);
            let elem = Box::new(self.fresh_struct_elems(base, &e));
            let len = match t {
                TypeKind::Array(_, n) => n.to_string(),
                _ => {
                    let l = self.fresh(&format!("{base}.len"), "Int");
                    self.pc
                        .push(format!("(and (<= 0 {l}) (<= {l} {}))", len_max(&e)));
                    l
                }
            };
            return Val::Arr { elem, len };
        }
        if is_array(t) {
            let e = elem_ty(t);
            let elems = self.fresh_elems(base, &e, true);
            let len = match t {
                TypeKind::Array(_, n) => n.to_string(),
                _ => {
                    let l = self.fresh(&format!("{base}.len"), "Int");
                    self.pc
                        .push(format!("(and (<= 0 {l}) (<= {l} {}))", len_max(&e)));
                    l
                }
            };
            Val::A { elems, len }
        } else if *t == TypeKind::Unit {
            Val::Unit
        } else {
            let c = self.fresh(base, &sort_of(t));
            if let Some(r) = range(t, &c) {
                self.pc.push(r);
            }
            Val::S(c)
        }
    }

    /// Records the path condition here as a reach point (`vacuous_ends`).
    fn mark_reach(&mut self, span: Span, what: &'static str) {
        if REACH.with(|r| r.get()) {
            let mut pc = self.pc.clone();
            pc.extend(self.spec_pc.iter().cloned());
            self.reach.push((span, what, pc));
        }
    }

    /// Facts about the product of `a` and `b` when neither is a constant
    /// (backlog 2V-02): its zero and sign, and for each pair of upper bounds
    /// the path condition states for the operands, the bound of a nonnegative
    /// product. Without them the solver's nonlinear search took a bounded
    /// `i * i` past three million steps.
    fn mul_facts(&mut self, a: &str, b: &str) {
        if is_numeral(a) || is_numeral(b) {
            return;
        }
        let p = format!("(* {a} {b})");
        let mut facts: Vec<String> = MUL_LEMMAS
            .iter()
            .map(|l| instantiate_mul(l, a, b, &p, "", ""))
            .collect();
        let xs = upper_bounds(&self.pc, a);
        let ys = if a == b {
            xs.clone()
        } else {
            upper_bounds(&self.pc, b)
        };
        for x in &xs {
            for y in &ys {
                facts.push(instantiate_mul(MUL_BOUND_LEMMA, a, b, &p, x, y));
            }
        }
        self.pc.extend(facts);
    }

    fn oblige(&mut self, code: &'static str, span: Span, msg: impl Into<String>, goal: String) {
        if goal != "true" {
            let mut assumptions = self.pc.clone();
            assumptions.extend(self.spec_pc.iter().cloned());
            self.obls.push(Obl {
                code,
                span,
                msg: msg.into(),
                assumptions,
                goal: goal.clone(),
            });
        }
        if self.term_check.is_none() {
            self.pc.push(goal);
        }
    }

    /// The full path of a function named from unit `mod_path`, as the
    /// checker resolves it: `f`, `T::f` (a type of the unit) and
    /// `Trait::f` (a trait of the unit) are local, anything else is a full
    /// path.
    fn resolve(&self, path: &[Ident], mod_path: &str) -> String {
        resolve_in(self.prog.t, path, mod_path)
    }

    // ---------- specification terms ----------

    fn spec(&mut self, e: &Expr, env: &BTreeMap<String, Val>, old: &BTreeMap<String, Val>) -> Val {
        match &e.kind {
            ExprKind::Int(v) => Val::S(num(*v as i128)),
            ExprKind::ByteLit(v) => Val::S(v.to_string()),
            // A byte string: its length and every element are known.
            ExprKind::ByteStr(v) => Val::A {
                elems: byte_array(v),
                len: v.len().to_string(),
            },
            ExprKind::Bool(b) => Val::S(b.to_string()),
            ExprKind::Unit => Val::Unit,
            ExprKind::Var(id) if id.name == "None" && !env.contains_key("None") => {
                self.variant_value(&self.ty(e), "None", &[])
            }
            ExprKind::Var(id) => env
                .get(&id.name)
                .cloned()
                .unwrap_or_else(|| panic!("unbound `{}` in {}", id.name, self.path)),
            ExprKind::Unary(UnOp::Neg, x) => {
                if let ExprKind::Int(v) = x.kind {
                    return Val::S(num(-(v as i128)));
                }
                Val::S(format!("(- {})", self.spec(x, env, old).s()))
            }
            ExprKind::Unary(UnOp::Not, x) => {
                Val::S(format!("(not {})", self.spec(x, env, old).s()))
            }
            // Structural `==` (ADR-0026 §2): field by field; a value that
            // holds an array compares its elements below the length only,
            // also inside enum payloads (backlog 2V-05).
            ExprKind::Binary(op @ (BinOp::Eq | BinOp::Ne), l, r)
                if struct_fields(&self.ty(l), self.prog.t).is_some()
                    || has_array(&self.ty(l), self.prog.t) =>
            {
                let t = self.ty(l);
                let a = self.spec(l, env, old);
                let b = self.spec(r, env, old);
                let q = self.ext_eq(&a, &b, &t);
                if *op == BinOp::Eq {
                    Val::S(q)
                } else {
                    Val::S(format!("(not {q})"))
                }
            }
            ExprKind::Binary(op, l, r) => {
                let lt = self.spec(l, env, old).s();
                let guard = match op {
                    BinOp::And | BinOp::Implies => Some(lt.clone()),
                    BinOp::Or => Some(format!("(not {lt})")),
                    _ => None,
                };
                if let Some(g) = &guard {
                    self.spec_pc.push(g.clone());
                }
                let rt = self.spec(r, env, old).s();
                if guard.is_some() {
                    self.spec_pc.pop();
                }
                let signed = self.ty(l) == TypeKind::I64;
                Val::S(match op {
                    BinOp::Add => format!("(+ {lt} {rt})"),
                    BinOp::Sub => format!("(- {lt} {rt})"),
                    BinOp::Mul => format!("(* {lt} {rt})"),
                    BinOp::Div if signed => format!("(bmb_tdiv {lt} {rt})"),
                    BinOp::Div => format!("(div {lt} {rt})"),
                    BinOp::Rem if signed => format!("(bmb_trem {lt} {rt})"),
                    BinOp::Rem => format!("(mod {lt} {rt})"),
                    BinOp::Eq => format!("(= {lt} {rt})"),
                    BinOp::Ne => format!("(not (= {lt} {rt}))"),
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        ord_term(*op, &self.ty(l), &lt, &rt)
                    }
                    BinOp::And => format!("(and {lt} {rt})"),
                    BinOp::Or => format!("(or {lt} {rt})"),
                    BinOp::Implies => format!("(=> {lt} {rt})"),
                })
            }
            ExprKind::Index { base, index } => {
                let bv = self.spec(base, env, old);
                let i = self.spec(index, env, old).s();
                match bv {
                    Val::A { elems, .. } => Val::S(format!("(select {elems} {i})")),
                    Val::Arr { elem, .. } => select_val(&elem, &i),
                    _ => panic!("index of non-array"),
                }
            }
            ExprKind::Method { recv, name, args } => {
                let rv = self.spec(recv, env, old);
                match name.name.as_str() {
                    "len" => match rv {
                        Val::A { len, .. } | Val::Arr { len, .. } => Val::S(len),
                        _ => panic!("len of scalar"),
                    },
                    "as_slice" | "as_mut_slice" => rv,
                    op => {
                        let b = self.spec(&args[0], env, old).s();
                        match int_method(op, &self.ty(recv), &rv.s(), &b) {
                            IntMethod::Value(v) => Val::S(v),
                            IntMethod::Checked { ok, v, .. } => self.checked(e, &ok, v),
                        }
                    }
                }
            }
            // `Box::new(v)` is `v` (ADR-0047 §3).
            ExprKind::Call { path, args, .. } if crate::check::is_box_new(path) => {
                self.spec(&args[0], env, old)
            }
            ExprKind::Call { path, args, .. } if self.is_variant(path) => {
                let vals: Vec<Val> = args.iter().map(|a| self.spec(a, env, old)).collect();
                self.variant_value(&self.ty(e), &path.last().unwrap().name, &vals)
            }
            ExprKind::Path(segs) if crate::check::int_const(segs).is_some() => {
                Val::S(num(crate::check::int_const(segs).expect("checked").1))
            }
            ExprKind::Path(segs) => {
                self.variant_value(&self.ty(e), &segs.last().unwrap().name, &[])
            }
            ExprKind::Match { scrut, arms } => self.spec_match(scrut, arms, env, old),
            ExprKind::Call { path, args, .. } => {
                let target = self.resolve(path, &self.spec_mod.clone());
                let target = self.spec_target(target, e.span);
                let mut flat = Vec::new();
                let mut argv = Vec::new();
                for a in args {
                    let v = self.spec(a, env, old);
                    flatten(&v, &mut flat);
                    argv.push(v);
                }
                self.check_spec_recursion(&target, &argv, e.span);
                let apps: Vec<String> = spec_funs(&target, self.prog.t)
                    .into_iter()
                    .map(|(f, _)| {
                        if flat.is_empty() {
                            f
                        } else {
                            format!("({f} {})", flat.join(" "))
                        }
                    })
                    .collect();
                let ret = self.prog.t.fns[&target].ret.clone();
                build(&ret, self.prog.t, &mut apps.into_iter())
            }
            ExprKind::Old(x) => {
                let ExprKind::Var(id) = &x.kind else {
                    panic!("old of non-variable")
                };
                old.get(&id.name).cloned().expect("old of unknown variable")
            }
            ExprKind::Forall { binders, body } => {
                let mut env2 = env.clone();
                let mut decl = Vec::new();
                let mut facts = Vec::new();
                for b in binders {
                    let s = sym(&format!("{}!{}", b.name.name, self.bound));
                    self.bound += 1;
                    let bt = self.local_ty(&b.name.name);
                    let sort = if matches!(bt, TypeKind::Param(_)) {
                        sort_of(&bt)
                    } else {
                        "Int".to_string()
                    };
                    decl.push(format!("({s} {sort})"));
                    if let Some(r) = range(&b.ty.kind, &s) {
                        facts.push(r);
                    }
                    env2.insert(b.name.name.clone(), Val::S(s));
                }
                self.foralls
                    .push((decl.clone(), facts.clone(), self.spec_pc.len()));
                self.triggers.push(Vec::new());
                let bt = self.spec(body, &env2, old).s();
                let pats = self.triggers.pop().unwrap();
                self.foralls.pop();
                let inner = if facts.is_empty() {
                    bt
                } else {
                    format!("(=> {} {bt})", and(&facts))
                };
                // One multi-pattern per choice of an alternative from each
                // trigger (a struct trigger offers each of its leaves).
                let mut combos: Vec<Vec<String>> = vec![Vec::new()];
                for alts in &pats {
                    combos = combos
                        .iter()
                        .flat_map(|c| {
                            alts.iter().map(move |a| {
                                let mut c = c.clone();
                                c.push(a.clone());
                                c
                            })
                        })
                        .take(16)
                        .collect();
                }
                let patterns: Vec<String> = combos
                    .iter()
                    .map(|c| format!(":pattern ({})", c.join(" ")))
                    .collect();
                Val::S(format!(
                    "(forall ({}) (! {inner} {}))",
                    decl.join(" "),
                    patterns.join(" ")
                ))
            }
            ExprKind::Trigger(x) => {
                let v = self.spec(x, env, old);
                if let Some(top) = self.triggers.last_mut() {
                    let mut leaves = Vec::new();
                    flatten(&v, &mut leaves);
                    top.push(leaves);
                }
                v
            }
            ExprKind::If { cond, then, els } => {
                let c = self.spec(cond, env, old).s();
                self.spec_pc.push(c.clone());
                let a = self.spec_block(then, env, old);
                self.spec_pc.pop();
                self.spec_pc.push(format!("(not {c})"));
                let b = match els {
                    Some(x) => self.spec(x, env, old),
                    None => Val::Unit,
                };
                self.spec_pc.pop();
                ite_val(&c, &a, &b)
            }
            ExprKind::Block(b) => self.spec_block(b, env, old),
            // `p(&a)` on a local array: the reference denotes the array.
            ExprKind::Ref { expr, .. } => self.spec(expr, env, old),
            ExprKind::StructLit { fields, .. } => {
                let t = self.ty(e);
                let mut vals: Vec<(usize, Val)> = fields
                    .iter()
                    .map(|(n, x)| {
                        (
                            field_index(&t, self.prog.t, &n.name),
                            self.spec(x, env, old),
                        )
                    })
                    .collect();
                vals.sort_by_key(|(k, _)| *k);
                Val::Struct(vals.into_iter().map(|(_, v)| v).collect())
            }
            ExprKind::Field { base, name } => {
                let bt = self.ty(base);
                match self.spec(base, env, old) {
                    Val::Struct(mut fs) => {
                        fs.swap_remove(field_index(&bt, self.prog.t, &name.name))
                    }
                    v => panic!("field of non-struct {v:?}"),
                }
            }
            ExprKind::Deref(x) => self.spec(x, env, old),
            // Array values in specifications (backlog 2V-05): a constant
            // array of the first element (or the default scalar) with every
            // element stored; a repeat is a constant array per leaf.
            ExprKind::Array(items) => {
                let elem = elem_ty(&self.ty(e));
                let vals: Vec<Val> = items.iter().map(|it| self.spec(it, env, old)).collect();
                let base = match vals.first() {
                    Some(v) => const_elems(&elem, v, self.prog.t),
                    // No element: any value of the element type will do.
                    None => const_elems(&elem, &default_val(&elem, self.prog.t), self.prog.t),
                };
                let mut arr = base;
                for (k, v) in vals.iter().enumerate() {
                    arr = store_val(&arr, &k.to_string(), v);
                }
                let len = items.len().to_string();
                match arr {
                    Val::S(elems) => Val::A { elems, len },
                    elem => Val::Arr {
                        elem: Box::new(elem),
                        len,
                    },
                }
            }
            ExprKind::Repeat { value, count, .. } => {
                let elem = elem_ty(&self.ty(e));
                let v = self.spec(value, env, old);
                match const_elems(&elem, &v, self.prog.t) {
                    Val::S(elems) => Val::A {
                        elems,
                        len: count.to_string(),
                    },
                    t => Val::Arr {
                        elem: Box::new(t),
                        len: count.to_string(),
                    },
                }
            }
            ExprKind::Error => unreachable!("rejected by the checker"),
        }
    }

    fn spec_block(
        &mut self,
        b: &Block,
        env: &BTreeMap<String, Val>,
        old: &BTreeMap<String, Val>,
    ) -> Val {
        let mut env2 = env.clone();
        for s in &b.stmts {
            match s {
                Stmt::Let { name, init, .. } => {
                    let v = self.spec(init, &env2, old);
                    env2.insert(name.name.clone(), v);
                }
                Stmt::Expr(_) => {}
                _ => {
                    unreachable!("other statements in a specification are rejected by the checker")
                }
            }
        }
        match &b.tail {
            Some(t) => self.spec(t, &env2, old),
            None => Val::Unit,
        }
    }

    /// When checking a recursive spec fn's termination, every call into the
    /// same component must decrease the measure.
    fn check_spec_recursion(&mut self, target: &str, args: &[Val], span: Span) {
        let Some((comp, caller_measure, data)) = self.term_check.clone() else {
            return;
        };
        let base = self
            .prog
            .t
            .insts
            .get(target)
            .map_or(target, |(b, _)| b.as_str());
        if self.prog.t.recursive.get(base) != Some(&comp) {
            return;
        }
        let def = &self.prog.defs[target];
        let Some(d) = &def.f.decreases else { return };
        let env = bind_params(def.f, args);
        let saved = self.term_check.take();
        let callee_measure = self.in_def(&def.mod_path.clone(), def.tsub.clone(), |s| {
            s.measure(d, &env, &BTreeMap::new())
        });
        self.term_check = saved;
        let goal = self.decrease_goal(&(caller_measure, data), &callee_measure);
        let msg = format!("recursive call to `{target}` may not decrease its `decreases` measure");
        // Inside `forall`: for every value of the bound variables that
        // reaches the call (backlog 2V-05), as Dafny checks it.
        if let Some((_, _, start)) = self.foralls.first() {
            let start = *start;
            let decls: Vec<String> = self.foralls.iter().flat_map(|f| f.0.clone()).collect();
            let mut inner: Vec<String> = self.foralls.iter().flat_map(|f| f.1.clone()).collect();
            inner.extend(self.spec_pc[start..].iter().cloned());
            let mut assumptions = self.pc.clone();
            assumptions.extend(self.spec_pc[..start].iter().cloned());
            self.obls.push(Obl {
                code: "E0813",
                span,
                msg,
                assumptions,
                goal: format!("(forall ({}) (=> {} {goal}))", decls.join(" "), and(&inner)),
            });
            return;
        }
        self.oblige("E0813", span, msg, goal);
    }

    // ---------- executable code ----------

    fn eval(&mut self, e: &Expr) -> Val {
        match &e.kind {
            ExprKind::Int(v) => Val::S(num(*v as i128)),
            ExprKind::ByteLit(v) => Val::S(v.to_string()),
            // A byte string: its length and every element are known.
            ExprKind::ByteStr(v) => Val::A {
                elems: byte_array(v),
                len: v.len().to_string(),
            },
            ExprKind::Bool(b) => Val::S(b.to_string()),
            ExprKind::Unit => Val::Unit,
            ExprKind::Var(id) if id.name == "None" && !self.vars.contains_key("None") => {
                self.variant_value(&self.ty(e), "None", &[])
            }
            ExprKind::Var(id) => self.vars[&id.name].clone(),
            ExprKind::Unary(UnOp::Neg, x) => {
                if let ExprKind::Int(v) = x.kind {
                    return Val::S(num(-(v as i128)));
                }
                let t = format!("(- {})", self.eval(x).s());
                let ty = self.ty(e);
                if let Some(r) = range(&ty, &t) {
                    self.oblige("E0803", e.span, "negation may overflow", r);
                }
                Val::S(t)
            }
            ExprKind::Unary(UnOp::Not, x) => Val::S(format!("(not {})", self.eval(x).s())),
            ExprKind::Binary(op @ (BinOp::And | BinOp::Or), l, r) => {
                // Short-circuit: the right operand runs only when it matters.
                let lt = self.eval(l).s();
                let n = self.pc.len();
                let (guard, name) = if *op == BinOp::And {
                    (lt.clone(), "and")
                } else {
                    (format!("(not {lt})"), "or")
                };
                self.pc.push(guard.clone());
                let rt = self.eval(r).s();
                // Facts learned on the right (a callee's `ensures`, division
                // lemmas, element ranges) hold whenever it ran (1B-29).
                let facts: Vec<String> = self.pc[n + 1..].to_vec();
                self.pc.truncate(n);
                if !facts.is_empty() {
                    self.pc.push(format!("(=> {guard} {})", and(&facts)));
                }
                Val::S(format!("({name} {lt} {rt})"))
            }
            ExprKind::Binary(op, l, r) => {
                let lt = self.eval(l).s();
                let rt = self.eval(r).s();
                let ty = self.ty(l);
                let signed = ty == TypeKind::I64;
                let t = match op {
                    BinOp::Add => format!("(+ {lt} {rt})"),
                    BinOp::Sub => format!("(- {lt} {rt})"),
                    BinOp::Mul => {
                        self.mul_facts(&lt, &rt);
                        format!("(* {lt} {rt})")
                    }
                    BinOp::Div | BinOp::Rem => {
                        self.oblige(
                            "E0804",
                            r.span,
                            "divisor may be zero",
                            format!("(not (= {rt} 0))"),
                        );
                        if *op == BinOp::Rem && signed {
                            self.oblige(
                                "E0803",
                                e.span,
                                "`i64::MIN % -1` may occur",
                                format!("(not (and (= {lt} {I64_MIN}) (= {rt} (- 1))))"),
                            );
                        }
                        let (q, r, lemmas) = if signed {
                            (
                                format!("(bmb_tdiv {lt} {rt})"),
                                format!("(bmb_trem {lt} {rt})"),
                                SIGNED_DIV_LEMMAS,
                            )
                        } else {
                            (
                                format!("(div {lt} {rt})"),
                                format!("(mod {lt} {rt})"),
                                UNSIGNED_DIV_LEMMAS,
                            )
                        };
                        for l in lemmas {
                            self.pc.push(instantiate(l, &lt, &rt, &q, &r));
                        }
                        if *op == BinOp::Div { q } else { r }
                    }
                    BinOp::Eq => return Val::S(format!("(= {lt} {rt})")),
                    BinOp::Ne => return Val::S(format!("(not (= {lt} {rt}))")),
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        return Val::S(ord_term(*op, &self.ty(l), &lt, &rt));
                    }
                    BinOp::And | BinOp::Or | BinOp::Implies => unreachable!(),
                };
                if let Some(rg) = range(&ty, &t) {
                    self.oblige(
                        "E0803",
                        e.span,
                        format!(
                            "`{}` may overflow `{}`",
                            op.symbol(),
                            crate::check::ty_str(&ty)
                        ),
                        rg,
                    );
                }
                Val::S(t)
            }
            ExprKind::Index { base, index } => {
                let bv = self.eval(base);
                let i = self.eval(index).s();
                let len = match &bv {
                    Val::A { len, .. } | Val::Arr { len, .. } => len.clone(),
                    _ => panic!("index of non-array"),
                };
                self.oblige(
                    "E0805",
                    e.span,
                    "index may be out of bounds",
                    format!("(and (<= 0 {i}) (< {i} {len}))"),
                );
                let elems = match bv {
                    Val::Arr { elem, .. } => {
                        // Element ranges hold where an element is read.
                        let v = select_val(&elem, &i);
                        let et = self.ty(e);
                        let mut ls = Vec::new();
                        leaves(&et, self.prog.t, "", &mut ls);
                        let mut terms = Vec::new();
                        flatten(&v, &mut terms);
                        for ((_, leaf), x) in ls.iter().zip(&terms) {
                            if let Some(f) = leaf.fact(x) {
                                self.pc.push(f);
                            }
                        }
                        return v;
                    }
                    Val::A { elems, .. } => elems,
                    _ => unreachable!(),
                };
                let t = format!("(select {elems} {i})");
                if let Some(r) = range(&self.ty(e), &t) {
                    self.pc.push(r);
                }
                Val::S(t)
            }
            // `T::try_from(x).unwrap()` / `.ok()` (ADR-0028 §1, §5).
            ExprKind::Method { recv, name, .. } if crate::check::conv_call(recv).is_some() => {
                let (target, args) = crate::check::conv_call(recv).unwrap();
                let v = self.eval(&args[0]).s();
                let st = self.ty(&args[0]);
                let r = range(&target, &v).expect("integer target");
                if name.name == "unwrap" {
                    if crate::check::conv_obligation(&st, &target) {
                        self.oblige(
                            "E0815",
                            e.span,
                            format!(
                                "conversion to `{}` may be out of range",
                                crate::check::ty_str(&target)
                            ),
                            r,
                        );
                    }
                    return Val::S(v);
                }
                self.checked(e, &r, v)
            }
            ExprKind::Method { recv, name, args } => {
                let rv = self.eval(recv);
                match name.name.as_str() {
                    "len" => match rv {
                        Val::A { len, .. } | Val::Arr { len, .. } => Val::S(len),
                        _ => panic!("len of scalar"),
                    },
                    "as_slice" | "as_mut_slice" | "clone" => rv,
                    m @ ("push" | "pop" | "truncate") => self.vec_change(e, recv, rv, m, args),
                    op => {
                        let b = self.eval(&args[0]).s();
                        if op.ends_with("_mul") {
                            self.mul_facts(&rv.s(), &b);
                        }
                        match int_method(op, &self.ty(recv), &rv.s(), &b) {
                            IntMethod::Value(v) => Val::S(v),
                            IntMethod::Checked { ok, v, lemmas } => {
                                self.pc.extend(lemmas);
                                self.checked(e, &ok, v)
                            }
                        }
                    }
                }
            }
            ExprKind::Call { path, args, .. }
                if path.len() == 2
                    && path[0].name == "Vec"
                    && matches!(path[1].name.as_str(), "new" | "with_capacity") =>
            {
                self.vec_new(e, path, args)
            }
            ExprKind::Call { path, args, .. } if crate::check::is_box_new(path) => {
                self.eval(&args[0])
            }
            ExprKind::Call { path, args, .. } if self.is_variant(path) => {
                let vals: Vec<Val> = args.iter().map(|a| self.eval(a)).collect();
                self.variant_value(&self.ty(e), &path.last().unwrap().name, &vals)
            }
            ExprKind::Call { path, args, .. } => self.call(path, args, e.span),
            ExprKind::If { cond, then, els } => {
                let c = self.eval(cond).s();
                let t = self.ty(e);
                self.branch(
                    &c,
                    &t,
                    |s| s.block(then),
                    |s| match els {
                        Some(x) => s.eval(x),
                        None => Val::Unit,
                    },
                )
            }
            ExprKind::Block(b) => self.block(b),
            // An array of structs or enums (2B-14): fresh element arrays
            // with each item stored, so no default value is needed.
            ExprKind::Array(items)
                if matches!(elem_ty(&self.ty(e)), TypeKind::Named { .. })
                    || agg_elem(&elem_ty(&self.ty(e)), self.prog.t) =>
            {
                let elem = elem_ty(&self.ty(e));
                let vals: Vec<Val> = items.iter().map(|it| self.eval(it)).collect();
                let mut arr = if agg_elem(&elem, self.prog.t) {
                    self.fresh_struct_elems("arr", &elem)
                } else {
                    Val::S(self.fresh("arr.e", &format!("(Array Int {})", sort_of(&elem))))
                };
                for (k, v) in vals.iter().enumerate() {
                    arr = store_val(&arr, &k.to_string(), v);
                }
                let len = items.len().to_string();
                match arr {
                    Val::S(elems) => Val::A { elems, len },
                    elem => Val::Arr {
                        elem: Box::new(elem),
                        len,
                    },
                }
            }
            ExprKind::Array(items) => {
                let elem = elem_ty(&self.ty(e));
                let mut arr = format!(
                    "((as const (Array Int {})) {})",
                    sort_of(&elem),
                    default_of(&elem)
                );
                for (k, it) in items.iter().enumerate() {
                    let v = self.eval(it).s();
                    arr = format!("(store {arr} {k} {v})");
                }
                Val::A {
                    elems: arr,
                    len: items.len().to_string(),
                }
            }
            ExprKind::Repeat { value, count, .. }
                if agg_elem(&elem_ty(&self.ty(e)), self.prog.t) =>
            {
                // One constant array per leaf of the element value (2B-14,
                // 2V-04b).
                let elem = elem_ty(&self.ty(e));
                let v = self.eval(value);
                Val::Arr {
                    elem: Box::new(const_elems(&elem, &v, self.prog.t)),
                    len: count.to_string(),
                }
            }
            ExprKind::Repeat { value, count, .. } => {
                let elem = elem_ty(&self.ty(e));
                let v = self.eval(value).s();
                Val::A {
                    elems: format!("((as const (Array Int {})) {v})", sort_of(&elem)),
                    len: count.to_string(),
                }
            }
            ExprKind::Ref { expr, .. } => self.eval(expr),
            ExprKind::Old(_) | ExprKind::Forall { .. } | ExprKind::Trigger(_) | ExprKind::Error => {
                panic!("specification construct in executable code")
            }
            // Fields are evaluated as written; the value is in declaration order.
            ExprKind::StructLit { fields, .. } => {
                let t = self.ty(e);
                let mut vals: Vec<(usize, Val)> = Vec::new();
                for (n, x) in fields {
                    let v = self.eval(x);
                    vals.push((field_index(&t, self.prog.t, &n.name), v));
                }
                vals.sort_by_key(|(k, _)| *k);
                Val::Struct(vals.into_iter().map(|(_, v)| v).collect())
            }
            ExprKind::Field { base, name } => {
                // A place under `&`/`&mut` has no recorded type.
                let bt = match self.ty(base) {
                    TypeKind::Error => self.place_ty(base),
                    t => t,
                };
                match self.eval(base) {
                    Val::Struct(mut fs) => {
                        fs.swap_remove(field_index(&bt, self.prog.t, &name.name))
                    }
                    v => panic!("field of non-struct {v:?}"),
                }
            }
            // A reference denotes its value (references are second-class).
            ExprKind::Deref(x) => self.eval(x),
            ExprKind::Path(segs) if crate::check::int_const(segs).is_some() => {
                Val::S(num(crate::check::int_const(segs).expect("checked").1))
            }
            ExprKind::Path(segs) => {
                self.variant_value(&self.ty(e), &segs.last().unwrap().name, &[])
            }
            ExprKind::Match { scrut, arms } => {
                let sv = self.eval(scrut).s();
                let st = self.ty(scrut);
                let t = self.ty(e);
                self.match_arms(&sv, &st, arms, 0, &t)
            }
        }
    }

    /// Runs two branches from the same state and merges them under `c`.
    fn branch(
        &mut self,
        c: &str,
        ty: &TypeKind,
        then_f: impl FnOnce(&mut Self) -> Val,
        else_f: impl FnOnce(&mut Self) -> Val,
    ) -> Val {
        let base_vars = self.vars.clone();
        let n = self.pc.len();
        self.pc.push(c.to_string());
        let tv = then_f(self);
        let then_vars = std::mem::replace(&mut self.vars, base_vars.clone());
        let then_facts: Vec<String> = self.pc[n + 1..].to_vec();
        self.pc.truncate(n);
        self.pc.push(format!("(not {c})"));
        let ev = else_f(self);
        let else_vars = std::mem::take(&mut self.vars);
        let else_facts: Vec<String> = self.pc[n + 1..].to_vec();
        self.pc.truncate(n);
        let mut merged = BTreeMap::new();
        for k in base_vars.keys() {
            let (a, b) = (&then_vars[k], &else_vars[k]);
            let t = if a == b {
                TypeKind::Error
            } else {
                self.local_ty(k)
            };
            let v = self.merge(c, a, b, &t);
            merged.insert(k.clone(), v);
        }
        self.vars = merged;
        if !then_facts.is_empty() {
            self.pc.push(format!("(=> {c} {})", and(&then_facts)));
        }
        if !else_facts.is_empty() {
            self.pc.push(format!("(=> (not {c}) {})", and(&else_facts)));
        }
        self.merge(c, &tv, &ev, ty)
    }

    /// `ite(c, a, b)`. A large merged term is bound to a fresh constant so
    /// that later merges refer to it instead of copying it: copying doubles
    /// the term at every sequential `if` (backlog 1B-26).
    fn merge(&mut self, c: &str, a: &Val, b: &Val, t: &TypeKind) -> Val {
        const BIND_ABOVE: usize = 200;
        match (a, b) {
            (Val::S(x), Val::S(y)) => {
                let term = ite(c, x, y);
                if term.len() > BIND_ABOVE
                    && !matches!(t, TypeKind::Error | TypeKind::Unit)
                    && !is_array(t)
                {
                    let v = self.fresh("merge", &sort_of(t));
                    self.pc.push(format!("(= {v} {term})"));
                    Val::S(v)
                } else {
                    Val::S(term)
                }
            }
            (Val::Struct(xs), Val::Struct(ys)) => {
                let tys: Vec<TypeKind> = match struct_fields(t, self.prog.t) {
                    Some(fs) => fs.iter().map(|(_, ft)| ft.clone()).collect(),
                    None => vec![TypeKind::Error; xs.len()],
                };
                Val::Struct(
                    xs.iter()
                        .zip(ys)
                        .zip(&tys)
                        .map(|((x, y), ft)| {
                            let ft = if x == y { TypeKind::Error } else { ft.clone() };
                            self.merge(c, x, y, &ft)
                        })
                        .collect(),
                )
            }
            (Val::Arr { .. }, Val::Arr { .. }) => ite_val(c, a, b),
            (Val::A { elems: e1, len: l1 }, Val::A { elems: e2, len: l2 }) => {
                let mut elems = ite(c, e1, e2);
                if elems.len() > BIND_ABOVE && is_array(t) {
                    let v = self.fresh("merge", &format!("(Array Int {})", sort_of(&elem_ty(t))));
                    self.pc.push(format!("(= {v} {elems})"));
                    elems = v;
                }
                Val::A {
                    elems,
                    len: ite(c, l1, l2),
                }
            }
            _ => a.clone(),
        }
    }

    fn block(&mut self, b: &Block) -> Val {
        for s in &b.stmts {
            self.stmt(s);
        }
        match &b.tail {
            Some(t) => self.eval(t),
            None => Val::Unit,
        }
    }

    fn cur_env(&self) -> (BTreeMap<String, Val>, BTreeMap<String, Val>) {
        (self.vars.clone(), self.old.clone())
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::LetFields { .. } => unreachable!("rejected by the checker (E0230)"),
            Stmt::Let { name, init, .. } => {
                let v = self.eval(init);
                self.vars.insert(name.name.clone(), v);
            }
            Stmt::Assign { target, value, .. } => {
                // Rust evaluates the value, then the place (and its bounds).
                let v = self.eval(value);
                let mut steps = Vec::new();
                let root = self.place(target, &mut steps, true);
                let cur = self.vars[&root].clone();
                self.vars.insert(root, set_at(cur, &steps, v));
            }
            Stmt::Assert { cond, span } => {
                let (env, old) = self.cur_env();
                let g = self.spec(cond, &env, &old).s();
                self.oblige("E0810", *span, "assertion may not hold", g);
            }
            Stmt::Expr(e) => {
                self.eval(e);
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                ..
            } => self.while_loop(cond, invariants, decreases.as_ref(), body),
        }
    }

    fn while_loop(
        &mut self,
        cond: &Expr,
        invariants: &[Expr],
        decreases: Option<&Expr>,
        body: &Block,
    ) {
        for inv in invariants {
            let (env, old) = self.cur_env();
            let g = self.spec(inv, &env, &old).s();
            self.oblige("E0806", inv.span, "loop invariant may not hold on entry", g);
        }
        // Havoc everything the loop may change, then assume the invariants.
        let mut modified = BTreeSet::new();
        assigned_in_block(body, &mut modified);
        assigned_in_expr(cond, &mut modified);
        let mut_params: BTreeSet<String> = self.prog.defs[&self.path]
            .f
            .params
            .iter()
            .filter(|p| matches!(p.ty.kind, TypeKind::Ref { mutable: true, .. }))
            .map(|p| p.name.name.clone())
            .collect();
        let modified: BTreeSet<String> = modified
            .into_iter()
            .filter_map(|v| match v.strip_prefix('?') {
                Some(n) => mut_params.contains(n).then(|| n.to_string()),
                None => Some(v),
            })
            .collect();
        for v in &modified {
            let Some(cur) = self.vars.get(v).cloned() else {
                continue;
            };
            let fresh = match cur {
                Val::S(_) => {
                    let ty = self.local_ty(v);
                    self.fresh_val(v, &ty)
                }
                Val::A { len, .. } => {
                    let ty = self.local_ty(v);
                    let e = elem_ty(&ty);
                    Val::A {
                        elems: self.fresh_elems(v, &e, false),
                        len: self.loop_len(v, len, &e),
                    }
                }
                Val::Struct(_) => {
                    let ty = self.local_ty(v);
                    self.fresh_val(v, &ty)
                }
                // The elements may change; the length only of a `Vec`.
                Val::Arr { len, .. } => {
                    let e = elem_ty(&self.local_ty(v));
                    Val::Arr {
                        elem: Box::new(self.fresh_struct_elems(v, &e)),
                        len: self.loop_len(v, len, &e),
                    }
                }
                Val::Unit => Val::Unit,
            };
            self.vars.insert(v.clone(), fresh);
        }
        // Earlier facts mention only earlier constants, so they stay valid.
        for inv in invariants {
            let (env, old) = self.cur_env();
            let g = self.spec(inv, &env, &old).s();
            self.pc.push(g);
        }
        let c = self.eval(cond).s();
        let after_cond_vars = self.vars.clone();
        let n = self.pc.len();
        // One arbitrary iteration.
        self.pc.push(c.clone());
        let measure = decreases.map(|d| {
            let (env, old) = self.cur_env();
            self.spec(d, &env, &old).s()
        });
        self.block(body);
        for inv in invariants {
            let (env, old) = self.cur_env();
            let g = self.spec(inv, &env, &old).s();
            self.oblige("E0807", inv.span, "loop invariant may not be preserved", g);
        }
        if let (Some(d0), Some(d)) = (measure, decreases) {
            let (env, old) = self.cur_env();
            let d1 = self.spec(d, &env, &old).s();
            self.oblige(
                "E0808",
                d.span,
                "`decreases` may not decrease or may be negative",
                format!("(and (<= 0 {d0}) (< {d1} {d0}))"),
            );
        }
        // Exit.
        self.pc.truncate(n);
        self.vars = after_cond_vars;
        self.pc.push(format!("(not {c})"));
        self.mark_reach(cond.span, "exit of the loop");
    }

    /// The length of `v` after a loop that changes it: an array's or a
    /// slice's stays, a `Vec`'s is any value in its range (backlog 2G-05:
    /// keeping it made the code after a `push` loop unreachable).
    fn loop_len(&mut self, v: &str, len: String, e: &TypeKind) -> String {
        if !self.is_vec_local(v) {
            return len;
        }
        let l = self.fresh(&format!("{v}.len"), "Int");
        self.pc
            .push(format!("(and (<= 0 {l}) (<= {l} {}))", len_max(e)));
        l
    }

    /// Whether local or parameter `name` is a `Vec`, `&Vec` or `&mut Vec`
    /// (the encoder's tables show them as slices; the program text does not).
    fn is_vec_local(&self, name: &str) -> bool {
        fn surface(t: &TypeKind) -> bool {
            match t {
                TypeKind::Named { path, .. } => path.len() == 1 && path[0] == "Vec",
                TypeKind::Ref { inner, .. } => surface(&inner.kind),
                _ => false,
            }
        }
        let def = &self.prog.defs[&self.path];
        if let Some(p) = def.f.params.iter().find(|p| p.name.name == name) {
            return surface(&p.ty.kind);
        }
        let mut found = None;
        find_let(&def.f.body, name, &mut found);
        found.is_some_and(|t| surface(&t))
    }

    /// `push`, `pop` and `truncate` on a `Vec` (ADR-0031 §3): the new pair
    /// is written back to the receiver's place; `push` may not pass the
    /// maximum length (E0816); `pop` is `None` exactly on an empty `Vec`.
    fn vec_change(&mut self, e: &Expr, recv: &Expr, rv: Val, m: &str, args: &[Expr]) -> Val {
        let el = elem_ty(&self.ty(recv));
        let (len, elems) = match rv {
            Val::A { elems, len } => (len, Val::S(elems)),
            Val::Arr { elem, len } => (len, *elem),
            v => panic!("{m} of {v:?}"),
        };
        let (result, new_len, new_elems) = match m {
            "push" => {
                let x = self.eval(&args[0]);
                self.oblige(
                    "E0816",
                    e.span,
                    "`push` may exceed the maximum length of a `Vec`",
                    format!("(< {len} {})", len_max(&el)),
                );
                let stored = store_val(&elems, &len, &x);
                (Val::Unit, format!("(+ {len} 1)"), stored)
            }
            "pop" => {
                let empty = format!("(= {len} 0)");
                let last = select_val(&elems, &format!("(- {len} 1)"));
                let opt = self.ty(e);
                let some = self.variant_value(&opt, "Some", &[last]).s();
                let none = self.variant_value(&opt, "None", &[]).s();
                (
                    Val::S(ite(&empty, &none, &some)),
                    ite(&empty, "0", &format!("(- {len} 1)")),
                    elems,
                )
            }
            _ => {
                let n = self.eval(&args[0]).s();
                (Val::Unit, ite(&format!("(< {n} {len})"), &n, &len), elems)
            }
        };
        let new = match new_elems {
            Val::S(a) => Val::A {
                elems: a,
                len: new_len,
            },
            other => Val::Arr {
                elem: Box::new(other),
                len: new_len,
            },
        };
        let mut steps = Vec::new();
        let root = self.place(recv, &mut steps, false);
        let cur = self.vars[&root].clone();
        self.vars.insert(root, set_at(cur, &steps, new));
        result
    }

    /// `Vec::new()` / `Vec::with_capacity(n)`: length 0, elements unknown;
    /// a capacity above the maximum length is E0816 (Rust panics).
    fn vec_new(&mut self, e: &Expr, path: &[Ident], args: &[Expr]) -> Val {
        let el = elem_ty(&self.ty(e));
        if path[1].name == "with_capacity" {
            let n = self.eval(&args[0]).s();
            self.oblige(
                "E0816",
                e.span,
                "the capacity may exceed the maximum length of a `Vec`",
                format!("(<= {n} {})", len_max(&el)),
            );
        }
        if struct_fields(&el, self.prog.t).is_some() {
            Val::Arr {
                elem: Box::new(self.fresh_struct_elems("vec", &el)),
                len: "0".into(),
            }
        } else {
            Val::A {
                elems: self.fresh_elems("vec", &el, false),
                len: "0".into(),
            }
        }
    }

    /// Whether a checked call builds a variant: `Some(x)`, `Ok(x)`, `Err(e)`
    /// or `E::V(..)` (a path that names no function).
    fn is_variant(&self, path: &[Ident]) -> bool {
        if path.len() == 1 {
            return matches!(path[0].name.as_str(), "Some" | "Ok" | "Err")
                && !self
                    .prog
                    .t
                    .fns
                    .contains_key(&format!("{}::{}", self.spec_mod, path[0].name));
        }
        !self
            .prog
            .t
            .fns
            .contains_key(&self.resolve(path, &self.spec_mod.clone()))
    }

    /// Variant `v` of enum `t` holding `payload`.
    fn variant_value(&self, t: &TypeKind, v: &str, payload: &[Val]) -> Val {
        let mut leaves = Vec::new();
        for p in payload {
            flatten(p, &mut leaves);
        }
        let c = ctor(t, v);
        Val::S(if leaves.is_empty() {
            c
        } else {
            format!("({c} {})", leaves.join(" "))
        })
    }

    /// The payload of `sv`, read as variant `v` of enum `t`: one value per
    /// position, and the range facts of its leaves (true when `sv` is a `v`,
    /// ADR-0027 §5).
    fn payload(&self, t: &TypeKind, v: &str, sv: &str) -> (Vec<Val>, Vec<String>) {
        let vs = enum_variants(t, self.prog.t).expect("match on an enum");
        let ps = &vs.iter().find(|(n, _)| n == v).expect("checked variant").1;
        let mut vals = Vec::new();
        let mut facts = Vec::new();
        for (k, pt) in ps.iter().enumerate() {
            let ls = payload_leaves(t, v, k, pt, self.prog.t);
            let terms: Vec<String> = ls
                .iter()
                .map(|(n, _)| format!("({} {sv})", sym(n)))
                .collect();
            for ((_, leaf), x) in ls.iter().zip(&terms) {
                facts.extend(leaf.fact(x));
            }
            vals.push(build(pt, self.prog.t, &mut terms.into_iter()));
        }
        (vals, facts)
    }

    /// Rust `==` on values `a`, `b` of type `t` (backlog 2V-05): SMT
    /// equality for scalars and enums without arrays; field by field for
    /// structs; equal lengths and elements below the length for arrays;
    /// for an enum with an array in a payload, the same variant with equal
    /// payloads (the SMT equality of its datatype would compare the array
    /// elements past the length).
    fn ext_eq(&mut self, a: &Val, b: &Val, t: &TypeKind) -> String {
        let tb = self.prog.t;
        if let TypeKind::Ref { inner, .. } = t
            && !is_array(t)
        {
            return self.ext_eq(a, b, &inner.kind.clone());
        }
        if let Some(fs) = struct_fields(t, tb) {
            let fs = fs.to_vec();
            let (Val::Struct(xs), Val::Struct(ys)) = (a, b) else {
                panic!("struct values expected")
            };
            let parts: Vec<String> = fs
                .iter()
                .zip(xs.iter().zip(ys))
                .map(|((_, ft), (x, y))| self.ext_eq(x, y, ft))
                .collect();
            return and(&parts);
        }
        if let Some(vs) = enum_variants(t, tb)
            && has_array(t, tb)
        {
            let (sa, sb) = (a.s(), b.s());
            let mut alts = Vec::new();
            for (v, ps) in &vs {
                let c = ctor(t, v);
                let (pa, _) = self.payload(t, v, &sa);
                let (pb, _) = self.payload(t, v, &sb);
                let mut parts = vec![format!("((_ is {c}) {sa})"), format!("((_ is {c}) {sb})")];
                for ((x, y), pt) in pa.iter().zip(&pb).zip(ps) {
                    parts.push(self.ext_eq(x, y, pt));
                }
                alts.push(and(&parts));
            }
            return format!("(or {})", alts.join(" "));
        }
        if is_array(t) {
            let e = elem_ty(t);
            // A fixed length up to UNROLL_EQ: element by element, without a
            // quantifier (the solver decides it, and a counterexample shows it).
            let fixed = match value_ty(t) {
                TypeKind::Array(_, n) => usize::try_from(*n).ok(),
                _ => None,
            };
            if let Some(n) = fixed.filter(|n| *n <= UNROLL_EQ) {
                let mut parts = Vec::new();
                for k in 0..n {
                    let (x, y) = match (a, b) {
                        (Val::A { elems: e1, .. }, Val::A { elems: e2, .. }) => (
                            Val::S(format!("(select {e1} {k})")),
                            Val::S(format!("(select {e2} {k})")),
                        ),
                        (Val::Arr { elem: e1, .. }, Val::Arr { elem: e2, .. }) => (
                            select_val(e1, &k.to_string()),
                            select_val(e2, &k.to_string()),
                        ),
                        _ => panic!("array values expected"),
                    };
                    parts.push(self.ext_eq(&x, &y, &e));
                }
                // Both lengths are the type's `n`.
                return and(&parts);
            }
            let k = sym(&format!("eq!{}", self.bound));
            self.bound += 1;
            let (len_a, len_b, body, pat) = match (a, b) {
                (Val::A { elems: e1, len: l1 }, Val::A { elems: e2, len: l2 }) => {
                    let x = Val::S(format!("(select {e1} {k})"));
                    let y = Val::S(format!("(select {e2} {k})"));
                    let body = self.ext_eq(&x, &y, &e);
                    (l1, l2, body, format!("({}) :pattern ({})", x.s(), y.s()))
                }
                (Val::Arr { elem: e1, len: l1 }, Val::Arr { elem: e2, len: l2 }) => {
                    let x = select_val(e1, &k);
                    let y = select_val(e2, &k);
                    let body = self.ext_eq(&x, &y, &e);
                    let (mut ts, mut us) = (Vec::new(), Vec::new());
                    flatten_elem(&x, &mut ts);
                    flatten_elem(&y, &mut us);
                    let pat = format!("({}) :pattern ({})", ts[0], us[0]);
                    (l1, l2, body, pat)
                }
                _ => panic!("array values expected"),
            };
            return format!(
                "(and (= {len_a} {len_b}) (forall (({k} Int)) (! (=> (and (<= 0 {k}) (< {k} {len_a})) {body}) :pattern {pat})))"
            );
        }
        if *t == TypeKind::Unit {
            return "true".into();
        }
        format!("(= {} {})", a.s(), b.s())
    }

    /// Arms `i..` of an executable `match` on `sv`: each arm runs under its
    /// tester (the last one under the others' negations: arms are
    /// exhaustive), with its bindings and their range facts.
    fn match_arms(&mut self, sv: &str, st: &TypeKind, arms: &[Arm], i: usize, t: &TypeKind) -> Val {
        let arm = &arms[i];
        let v = arm.pat.path.last().unwrap().name.clone();
        let run = |s: &mut Self| {
            let (vals, facts) = s.payload(st, &v, sv);
            s.pc.extend(facts);
            for (b, val) in arm.pat.binds.iter().zip(vals) {
                if let Some(id) = b {
                    s.vars.insert(id.name.clone(), val);
                }
            }
            s.eval(&arm.body)
        };
        if i + 1 == arms.len() {
            return run(self);
        }
        let c = format!("((_ is {}) {sv})", ctor(st, &v));
        self.branch(&c, t, run, |s| s.match_arms(sv, st, arms, i + 1, t))
    }

    /// A `match` in a specification: an `ite` over the testers.
    fn spec_match(
        &mut self,
        scrut: &Expr,
        arms: &[Arm],
        env: &BTreeMap<String, Val>,
        old: &BTreeMap<String, Val>,
    ) -> Val {
        let sv = self.spec(scrut, env, old).s();
        let st = self.ty(scrut);
        let mut parts = Vec::new();
        for arm in arms {
            let v = arm.pat.path.last().unwrap().name.clone();
            let c = format!("((_ is {}) {sv})", ctor(&st, &v));
            let (vals, _) = self.payload(&st, &v, &sv);
            let mut env2 = env.clone();
            for (b, val) in arm.pat.binds.iter().zip(vals) {
                if let Some(id) = b {
                    env2.insert(id.name.clone(), val);
                }
            }
            self.spec_pc.push(c.clone());
            let x = self.spec(&arm.body, &env2, old);
            self.spec_pc.pop();
            parts.push((c, x));
        }
        let (_, mut out) = parts.pop().expect("a match has arms");
        while let Some((c, x)) = parts.pop() {
            out = ite_val(&c, &x, &out);
        }
        out
    }

    /// The variable a place expression writes, and the steps from it to the
    /// place: fields, `*p` (the reference is its value) and array elements,
    /// whose index is checked against the length when `check` is set.
    fn place(&mut self, e: &Expr, steps: &mut Vec<Step>, check: bool) -> String {
        match &e.kind {
            ExprKind::Var(id) => id.name.clone(),
            ExprKind::Deref(x) | ExprKind::Ref { expr: x, .. } => self.place(x, steps, check),
            ExprKind::Method { recv, .. } => self.place(recv, steps, check),
            ExprKind::Field { base, name } => {
                let root = self.place(base, steps, check);
                let bt = self.place_ty(base);
                steps.push(Step::Field(field_index(&bt, self.prog.t, &name.name)));
                root
            }
            ExprKind::Index { base, index } => {
                let root = self.place(base, steps, check);
                let (Val::A { len, .. } | Val::Arr { len, .. }) = self.eval(base) else {
                    panic!("index of non-array")
                };
                let i = self.eval(index).s();
                if check {
                    self.oblige(
                        "E0805",
                        e.span,
                        "index may be out of bounds",
                        format!("(and (<= 0 {i}) (< {i} {len}))"),
                    );
                }
                steps.push(Step::Index(i));
                root
            }
            _ => panic!("not a place"),
        }
    }

    /// The type of a place expression (the checker does not record them).
    fn place_ty(&self, e: &Expr) -> TypeKind {
        match &e.kind {
            ExprKind::Var(id) => self.local_ty(&id.name),
            ExprKind::Deref(x) | ExprKind::Ref { expr: x, .. } => match self.place_ty(x) {
                TypeKind::Ref { inner, .. } => inner.kind,
                t => t,
            },
            ExprKind::Field { base, name } => {
                let bt = self.place_ty(base);
                let fs = struct_fields(&bt, self.prog.t).expect("field of non-struct");
                fs[field_index(&bt, self.prog.t, &name.name)].1.clone()
            }
            ExprKind::Index { base, .. } => elem_ty(&self.place_ty(base)),
            _ => self.ty(e),
        }
    }

    fn local_ty(&self, name: &str) -> TypeKind {
        let base = self
            .prog
            .t
            .insts
            .get(&self.path)
            .map_or(&self.path, |(b, _)| b);
        if let Some(t) = self.prog.t.locals.get(base).and_then(|m| m.get(name)) {
            return crate::check::subst(t, &self.tsub);
        }
        let def = &self.prog.defs[&self.path];
        if let Some(p) = def.f.params.iter().find(|p| p.name.name == name) {
            return p.ty.kind.clone();
        }
        let mut found = None;
        find_let(&def.f.body, name, &mut found);
        found.unwrap_or(TypeKind::Error)
    }

    /// Translates a specification written in another unit.
    fn in_unit<T>(&mut self, unit: &str, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = std::mem::replace(&mut self.spec_mod, unit.to_string());
        let r = f(self);
        self.spec_mod = saved;
        r
    }

    /// Translates a specification of another definition: its unit, and its
    /// type parameters substituted by `tsub`.
    fn in_def<T>(
        &mut self,
        unit: &str,
        tsub: BTreeMap<String, TypeKind>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let saved = std::mem::replace(&mut self.tsub, tsub);
        let r = self.in_unit(unit, f);
        self.tsub = saved;
        r
    }

    /// The refinement obligations of trait impl function `f` (ADR-0046
    /// §4, E0817), on its entry values: the trait method `decl`'s
    /// `requires` imply each of `f`'s, and the trait's `requires` with
    /// `f`'s `ensures` imply each of the trait's `ensures` (on a new
    /// result and new values behind `&mut` parameters). Parameters match
    /// by position; `tsub` states the trait's contract for `f`. Nothing
    /// is assumed afterwards.
    fn refines(&mut self, f: &FnDef, decl: &str, tsub: BTreeMap<String, TypeKind>) {
        let Some(td) = self.prog.defs.get(decl) else {
            return;
        };
        let (tf, tunit) = (td.f, td.mod_path.clone());
        if tf.requires.is_empty() && tf.ensures.is_empty() && f.requires.is_empty() {
            return;
        }
        let saved = self.pc.clone();
        let (env, old) = self.cur_env();
        let by_trait = |m: &BTreeMap<String, Val>| -> BTreeMap<String, Val> {
            tf.params
                .iter()
                .zip(&f.params)
                .filter_map(|(tp, ip)| {
                    m.get(&ip.name.name)
                        .map(|v| (tp.name.name.clone(), v.clone()))
                })
                .collect()
        };
        let (tenv, told) = (by_trait(&env), by_trait(&old));
        let treq: Vec<String> = tf
            .requires
            .iter()
            .map(|r| self.in_def(&tunit, tsub.clone(), |s| s.spec(r, &tenv, &told).s()))
            .collect();
        let pre = and(&treq);
        for r in &f.requires {
            let g = self.spec(r, &env, &old).s();
            self.oblige(
                "E0817",
                r.span,
                format!(
                    "precondition `{}` is not implied by the preconditions of `{decl}`",
                    pred_name(r)
                ),
                format!("(=> {pre} {g})"),
            );
            self.pc = saved.clone();
        }
        if !tf.ensures.is_empty() {
            // The state after the call: a new result, new values behind
            // `&mut` parameters (their entry values are `old`).
            let mut post = env.clone();
            for p in &f.params {
                if let TypeKind::Ref { mutable: true, .. } = p.ty.kind {
                    let ty = self.local_ty(&p.name.name);
                    let v = self.fresh_val(&format!("{}.post", p.name.name), &ty);
                    post.insert(p.name.name.clone(), v);
                }
            }
            let ret = self.prog.t.fns[&self.path].ret.clone();
            let ret = crate::check::subst(&ret, &self.tsub);
            let result = self.fresh_val("result.post", &ret);
            let mut ipost = post.clone();
            if let Some(n) = &f.ret.name {
                ipost.insert(n.name.clone(), result.clone());
            }
            let mut assumed = treq.clone();
            for en in &f.ensures {
                assumed.push(self.spec(en, &ipost, &old).s());
            }
            let mut tpost = by_trait(&post);
            if let Some(n) = &tf.ret.name {
                tpost.insert(n.name.clone(), result.clone());
            }
            let hyp = and(&assumed);
            // With the range facts of the new values.
            let base = self.pc.clone();
            for en in &tf.ensures {
                let g = self.in_def(&tunit, tsub.clone(), |s| s.spec(en, &tpost, &told).s());
                self.oblige(
                    "E0817",
                    f.name.span,
                    format!("postcondition `{}` of `{decl}` may not hold", pred_name(en)),
                    format!("(=> {hyp} {g})"),
                );
                self.pc = base.clone();
            }
        }
        self.pc = saved;
    }

    fn call(&mut self, path: &[Ident], args: &[Expr], span: Span) -> Val {
        let mut target = self.resolve(path, &self.mod_path.clone());
        // A generic callee's contract under the call's type arguments; a
        // trait method's is its impl's when `Self` is known.
        let mut csub = self.call_subst(&target, span);
        if let Some(sig) = self.prog.t.fns.get(&target)
            && sig.trait_of.is_some()
        {
            let targs: Vec<TypeKind> = sig.generics.iter().map(|g| csub[&g.name].clone()).collect();
            let (t, a) = dispatched(self.prog.t, target, targs);
            csub = self.prog.t.fns[&t]
                .generics
                .iter()
                .map(|g| g.name.clone())
                .zip(a)
                .collect();
            target = t;
        }
        let mut argv = Vec::new();
        for a in args {
            argv.push(self.eval(a));
        }
        let Some(def) = self.prog.defs.get(&target) else {
            // The one built-in function has no contract; anything else
            // unresolved would drop a callee's effects.
            assert!(
                matches!(target.as_str(), "print_i64" | "print_u64" | "print_bytes"),
                "unresolved call in {}",
                self.path
            );
            return Val::Unit;
        };
        let f = def.f;
        let pre_env = bind_params(f, &argv);
        let unit = def.mod_path.clone();
        for r in &f.requires {
            let g = self.in_def(&unit, csub.clone(), |s| {
                s.spec(r, &pre_env, &BTreeMap::new()).s()
            });
            self.oblige(
                "E0801",
                span,
                format!("precondition `{}` of `{target}` may not hold", pred_name(r)),
                g,
            );
        }
        if let (Some(cc), Some(d)) = (self.prog.t.recursive.get(&target), &f.decreases)
            && self.prog.t.recursive.get(&self.path) == Some(cc)
            && let Some(m0) = self.entry_measure.clone()
        {
            let m1 = self.in_def(&unit, csub.clone(), |s| {
                s.measure(d, &pre_env, &BTreeMap::new())
            });
            self.oblige(
                "E0809",
                span,
                format!("recursive call to `{target}` may not decrease its measure"),
                self.decrease_goal(&m0, &m1),
            );
        }
        // `&mut` arguments: the callee may change the place, the elements
        // of a slice but never its length; a `&mut Vec<T>` also its length.
        let mut post_args = argv.clone();
        let mut old_env = BTreeMap::new();
        let param_tys: Vec<TypeKind> = self.prog.t.fns[&target]
            .params
            .iter()
            .map(|t| crate::check::subst(t, &csub))
            .collect();
        for (i, p) in f.params.iter().enumerate() {
            if let TypeKind::Ref { mutable: true, .. } = p.ty.kind
                && root_var(&args[i]).is_some()
            {
                let mut steps = Vec::new();
                let root = self.place(&args[i], &mut steps, false);
                let grows = matches!(&p.ty.kind, TypeKind::Ref { inner, .. }
                    if matches!(&inner.kind, TypeKind::Named { path, .. } if path.len() == 1 && path[0] == "Vec"));
                let e = elem_ty(&param_tys[i]);
                let new_len = |s: &mut Self, len: &String| {
                    if !grows {
                        return len.clone();
                    }
                    let l = s.fresh(&format!("{root}.len"), "Int");
                    s.pc.push(format!("(and (<= 0 {l}) (<= {l} {}))", len_max(&e)));
                    l
                };
                let nv = match &argv[i] {
                    Val::A { len, .. } => Val::A {
                        elems: self.fresh_elems(&root, &e, false),
                        len: new_len(self, len),
                    },
                    Val::Arr { len, .. } => Val::Arr {
                        elem: Box::new(self.fresh_struct_elems(&root, &e)),
                        len: new_len(self, len),
                    },
                    _ => self.fresh_val(&root, &param_tys[i]),
                };
                old_env.insert(p.name.name.clone(), argv[i].clone());
                let cur = self.vars[&root].clone();
                self.vars.insert(root, set_at(cur, &steps, nv.clone()));
                post_args[i] = nv;
            }
        }
        let mut post_env = bind_params(f, &post_args);
        let ret_ty = crate::check::subst(&self.prog.t.fns[&target].ret, &csub);
        let result = self.fresh_val(&format!("{target}.result"), &ret_ty);
        if let Some(n) = &f.ret.name {
            post_env.insert(n.name.clone(), result.clone());
        }
        for en in &f.ensures {
            let g = self.in_def(&unit, csub.clone(), |s| s.spec(en, &post_env, &old_env).s());
            self.pc.push(g);
        }
        result
    }
}

fn default_of(t: &TypeKind) -> &'static str {
    if *t == TypeKind::Bool { "false" } else { "0" }
}

fn bind_params(f: &FnDef, args: &[Val]) -> BTreeMap<String, Val> {
    f.params
        .iter()
        .zip(args)
        .map(|(p, v)| (p.name.name.clone(), v.clone()))
        .collect()
}

fn pred_name(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Call { path, .. } => path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::"),
        _ => "..".into(),
    }
}

fn root_var(e: &Expr) -> Option<String> {
    match &e.kind {
        ExprKind::Var(id) => Some(id.name.clone()),
        // `v.as_mut_slice()` / `v.as_slice()` as an argument.
        ExprKind::Method { recv, name, .. }
            if matches!(name.name.as_str(), "as_mut_slice" | "as_slice") =>
        {
            root_var(recv)
        }
        ExprKind::Ref { expr: x, .. }
        | ExprKind::Deref(x)
        | ExprKind::Field { base: x, .. }
        | ExprKind::Index { base: x, .. } => root_var(x),
        _ => None,
    }
}

/// The declared type of local `name`, wherever its `let` is: names are
/// unique within a function (no shadowing), so the first match is the one.
fn find_let(b: &Block, name: &str, out: &mut Option<TypeKind>) {
    for s in &b.stmts {
        if out.is_some() {
            return;
        }
        match s {
            Stmt::Let {
                name: n, ty, init, ..
            } => {
                if n.name == name {
                    *out = Some(ty.kind.clone());
                    return;
                }
                find_let_expr(init, name, out);
            }
            Stmt::Assign { target, value, .. } => {
                find_let_expr(target, name, out);
                find_let_expr(value, name, out);
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                find_let_expr(cond, name, out);
                invariants.iter().for_each(|e| find_let_expr(e, name, out));
                if let Some(d) = decreases {
                    find_let_expr(d, name, out);
                }
                find_let(body, name, out);
            }
            Stmt::Assert { cond, .. } => find_let_expr(cond, name, out),
            Stmt::LetFields { init, .. } => find_let_expr(init, name, out),
            Stmt::Expr(e) => find_let_expr(e, name, out),
        }
    }
    if let Some(t) = &b.tail {
        find_let_expr(t, name, out);
    }
}

fn find_let_expr(e: &Expr, name: &str, out: &mut Option<TypeKind>) {
    if out.is_some() {
        return;
    }
    match &e.kind {
        ExprKind::If { cond, then, els } => {
            find_let_expr(cond, name, out);
            find_let(then, name, out);
            if let Some(x) = els {
                find_let_expr(x, name, out);
            }
        }
        ExprKind::Block(b) => find_let(b, name, out),
        ExprKind::Call { args, .. } => args.iter().for_each(|a| find_let_expr(a, name, out)),
        ExprKind::Method { recv, args, .. } => {
            find_let_expr(recv, name, out);
            args.iter().for_each(|a| find_let_expr(a, name, out));
        }
        ExprKind::Index { base, index } => {
            find_let_expr(base, name, out);
            find_let_expr(index, name, out);
        }
        ExprKind::Binary(_, l, r) => {
            find_let_expr(l, name, out);
            find_let_expr(r, name, out);
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Trigger(x) => find_let_expr(x, name, out),
        ExprKind::Repeat { value, .. } => find_let_expr(value, name, out),
        ExprKind::Array(items) => items.iter().for_each(|a| find_let_expr(a, name, out)),
        ExprKind::Forall { body, .. } => find_let_expr(body, name, out),
        ExprKind::Field { base, .. } | ExprKind::Deref(base) => find_let_expr(base, name, out),
        ExprKind::StructLit { fields, .. } => {
            fields.iter().for_each(|(_, x)| find_let_expr(x, name, out))
        }
        ExprKind::Match { scrut, arms } => {
            find_let_expr(scrut, name, out);
            arms.iter().for_each(|a| find_let_expr(&a.body, name, out));
        }
        ExprKind::Int(_)
        | ExprKind::ByteLit(_)
        | ExprKind::ByteStr(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Var(_)
        | ExprKind::Path(_)
        | ExprKind::Error => {}
    }
}

fn assigned_in_block(b: &Block, out: &mut BTreeSet<String>) {
    for s in &b.stmts {
        match s {
            Stmt::Assign { target, value, .. } => {
                if let Some(r) = root_var(match &target.kind {
                    ExprKind::Index { base, .. } => base,
                    _ => target,
                }) {
                    out.insert(r);
                }
                assigned_in_expr(value, out);
            }
            Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => assigned_in_expr(init, out),
            Stmt::While { cond, body, .. } => {
                assigned_in_expr(cond, out);
                assigned_in_block(body, out);
            }
            Stmt::Expr(e) => assigned_in_expr(e, out),
            Stmt::Assert { .. } => {}
        }
    }
    if let Some(t) = &b.tail {
        assigned_in_expr(t, out);
    }
}

fn assigned_in_expr(e: &Expr, out: &mut BTreeSet<String>) {
    match &e.kind {
        ExprKind::Call { args, .. } => {
            for a in args {
                if let ExprKind::Ref {
                    mutable: true,
                    expr,
                } = &a.kind
                    && let Some(r) = root_var(expr)
                {
                    out.insert(r);
                }
                // A reference parameter passed on may be modified by the callee;
                // `?name` marks it until the caller knows which names are `&mut`.
                if let ExprKind::Var(id) = &a.kind {
                    out.insert(format!("?{}", id.name));
                }
                assigned_in_expr(a, out);
            }
        }
        ExprKind::If { cond, then, els } => {
            assigned_in_expr(cond, out);
            assigned_in_block(then, out);
            if let Some(x) = els {
                assigned_in_expr(x, out);
            }
        }
        ExprKind::Block(b) => assigned_in_block(b, out),
        ExprKind::Binary(_, l, r) => {
            assigned_in_expr(l, out);
            assigned_in_expr(r, out);
        }
        ExprKind::Unary(_, x) | ExprKind::Ref { expr: x, .. } => assigned_in_expr(x, out),
        ExprKind::Index { base, index } => {
            assigned_in_expr(base, out);
            assigned_in_expr(index, out);
        }
        ExprKind::Method { recv, name, args } => {
            if matches!(
                name.name.as_str(),
                "push" | "pop" | "truncate" | "as_mut_slice"
            ) && let Some(r) = root_var(recv)
            {
                out.insert(r);
            }
            assigned_in_expr(recv, out);
            args.iter().for_each(|a| assigned_in_expr(a, out));
        }
        _ => {}
    }
}

// ---------- program-level driver ----------

/// The definitions of `p` by path (a later duplicate is dropped), and the
/// trait method declarations, which are declared, not proved (ADR-0046 §4).
fn collect<'p>(p: &'p Program, t: &Tables) -> BTreeMap<String, Def<'p>> {
    let mut out = BTreeMap::new();
    for (path, mod_path, f) in crate::check::defs_of(p, &t.fns) {
        out.entry(path).or_insert(Def {
            f,
            mod_path,
            tsub: BTreeMap::new(),
            inst: false,
        });
    }
    for (path, mod_path, f) in crate::check::trait_decls(p) {
        out.entry(path).or_insert(Def {
            f,
            mod_path,
            tsub: BTreeMap::new(),
            inst: true,
        });
    }
    out
}

fn param_sorts(params: &[TypeKind], tb: &Tables) -> Vec<String> {
    let mut ls = Vec::new();
    for t in params {
        leaves(t, tb, "", &mut ls);
    }
    ls.iter().map(|(_, l)| l.sort()).collect()
}

/// Facts about truncating division, instantiated at every `/` and `%` so the
/// solver needs no nonlinear reasoning for range obligations. Each is proved
/// from the `PRELUDE` definitions by `tests/lemmas.rs`. Placeholders: `{a}`,
/// `{b}` operands, `{q}` quotient term, `{r}` remainder term.
pub const SIGNED_DIV_LEMMAS: &[&str] = &[
    "(=> (= {b} 1) (= {q} {a}))",
    "(=> (= {b} (- 1)) (= {q} (- {a})))",
    "(=> (> (bmb_abs {b}) 1) (<= (* 2 (bmb_abs {q})) (bmb_abs {a})))",
    "(=> (not (= {b} 0)) (< (bmb_abs {r}) (bmb_abs {b})))",
    "(=> (not (= {b} 0)) (or (= {r} 0) (= (>= {r} 0) (>= {a} 0))))",
    // The quotient's sign (backlog 1B-42): without it, `a / (x / g) >= 0`
    // for positive operands took the solver past its limit.
    "(=> (not (= {b} 0)) (or (= {q} 0) (= (> {q} 0) (= (> {a} 0) (> {b} 0)))))",
    // Nonnegative operands divide as SMT `div` and `mod` do, which `u64`
    // specifications use (1B-42: `a % b` against `gcd(b, a % b)`).
    "(=> (and (>= {a} 0) (> {b} 0)) (and (= {q} (div {a} {b})) (= {r} (mod {a} {b}))))",
];

/// The same for unsigned operands, where `/` and `%` are SMT `div` and `mod`.
pub const UNSIGNED_DIV_LEMMAS: &[&str] = &[
    "(=> (and (>= {a} 0) (> {b} 0)) (and (<= 0 {q}) (<= {q} {a})))",
    "(=> (> {b} 0) (and (<= 0 {r}) (< {r} {b})))",
];

/// Facts about a product `{p}` of `{a}` and `{b}` (backlog 2V-02), proved by
/// `tests/lemmas.rs`.
pub const MUL_LEMMAS: &[&str] = &[
    "(= (= {p} 0) (or (= {a} 0) (= {b} 0)))",
    "(=> (and (not (= {a} 0)) (not (= {b} 0))) (= (> {p} 0) (= (> {a} 0) (> {b} 0))))",
];

/// A nonnegative product is at most the product of upper bounds `{x}` of
/// `{a}` and `{y}` of `{b}` (monotonicity).
pub const MUL_BOUND_LEMMA: &str =
    "(=> (and (<= 0 {a}) (<= {a} {x}) (<= 0 {b}) (<= {b} {y})) (<= {p} (* {x} {y})))";

pub fn instantiate_mul(template: &str, a: &str, b: &str, p: &str, x: &str, y: &str) -> String {
    template
        .replace("{a}", a)
        .replace("{b}", b)
        .replace("{p}", p)
        .replace("{x}", x)
        .replace("{y}", y)
}

/// The most upper bounds of one term `mul_facts` takes from the path condition.
const MAX_BOUNDS: usize = 2;

/// Upper bounds of `term` that top-level conjuncts of the path condition
/// state (`term < t`, `term <= t`, `t > term`, `t >= term`), newest first.
fn upper_bounds(pc: &[String], term: &str) -> Vec<String> {
    fn walk(e: &smt::Sexp, term: &str, out: &mut Vec<String>) {
        let smt::Sexp::List(xs) = e else { return };
        let head = match xs.first() {
            Some(smt::Sexp::Atom(h)) => h.as_str(),
            _ => return,
        };
        match (head, xs.len()) {
            ("and", _) => xs[1..].iter().for_each(|x| walk(x, term, out)),
            ("<" | "<=", 3) if smt::print(&xs[1]) == term => out.push(smt::print(&xs[2])),
            (">" | ">=", 3) if smt::print(&xs[2]) == term => out.push(smt::print(&xs[1])),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for f in pc.iter().rev() {
        if let Some(e) = smt::parse(f) {
            walk(&e, term, &mut out);
        }
        if out.len() >= MAX_BOUNDS {
            break;
        }
    }
    out.truncate(MAX_BOUNDS);
    out.dedup();
    out
}

fn is_numeral(t: &str) -> bool {
    let t = t
        .strip_prefix("(- ")
        .and_then(|x| x.strip_suffix(')'))
        .unwrap_or(t);
    !t.is_empty() && t.bytes().all(|c| c.is_ascii_digit())
}

pub fn instantiate(template: &str, a: &str, b: &str, q: &str, r: &str) -> String {
    template
        .replace("{a}", a)
        .replace("{b}", b)
        .replace("{q}", q)
        .replace("{r}", r)
}

pub const PRELUDE: &str = "(define-fun bmb_abs ((x Int)) Int (ite (>= x 0) x (- x)))
(define-fun bmb_tdiv ((a Int) (b Int)) Int (ite (>= a 0) (ite (> b 0) (div a b) (- (div a (- b)))) (ite (> b 0) (- (div (- a) b)) (div (- a) (- b)))))
(define-fun bmb_trem ((a Int) (b Int)) Int (- a (* b (bmb_tdiv a b))))";

impl<'p> Prog<'p> {
    /// Declarations of every spec fn, as (path, SMT symbol, declaration).
    fn spec_decls(&self) -> Vec<(String, String, String)> {
        self.defs
            .iter()
            .filter(|(p, d)| d.f.kind == FnKind::Spec && self.t.fns[*p].generics.is_empty())
            .map(|(path, _)| {
                let sig = &self.t.fns[path];
                let params = param_sorts(&sig.params, self.t).join(" ");
                let decl = spec_funs(path, self.t)
                    .iter()
                    .map(|(f, sort)| format!("(declare-fun {f} ({params}) {sort})"))
                    .collect::<Vec<_>>()
                    .join("\n");
                (path.clone(), spec_key(path, self.t), decl)
            })
            .collect()
    }

    /// The spec fns `f`'s `#[uses]` reveals: a generic one by each of its
    /// instances.
    fn revealed(&self, f: &FnDef, mod_path: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut named = Vec::new();
        for u in uses_of(f, mod_path, &self.t.fns) {
            // A trait's spec method: every impl's definition of it.
            match &self.t.fns[&u].trait_of {
                Some(tr) => {
                    let m = u.rsplit("::").next().unwrap_or_default();
                    named.extend(
                        self.t
                            .trait_impls
                            .iter()
                            .filter(|i| &i.trait_path == tr)
                            .map(|i| i.fn_path(m))
                            .filter(|p| self.t.fns.contains_key(p)),
                    );
                    // The default body, for each `Self` whose impl leaves it
                    // out (`Wrap<T>` too; never a bare type parameter: no
                    // impl is known for it, and its impl may define it).
                    let sig = &self.t.fns[&u];
                    out.extend(
                        self.t
                            .insts
                            .iter()
                            .filter(|(_, (b, _))| *b == u)
                            .filter(|(_, (_, sub))| {
                                let args: Vec<TypeKind> =
                                    sig.generics.iter().map(|g| sub[&g.name].clone()).collect();
                                crate::check::dispatch(&self.t.fns, &self.t.trait_impls, &u, &args)
                                    .is_some_and(|(p, _)| p == u)
                            })
                            .map(|(k, _)| k.clone()),
                    );
                }
                None => named.push(u),
            }
        }
        for u in named {
            if self.t.fns[&u].generics.is_empty() {
                out.push(u);
            } else {
                out.extend(
                    self.t
                        .insts
                        .iter()
                        .filter(|(_, (b, _))| *b == u)
                        .map(|(k, _)| k.clone()),
                );
            }
        }
        out
    }

    /// Definitional axiom of a spec fn (ADR-0017 §4).
    fn axiom(&self, path: &str) -> (String, AxiomParts) {
        let d = &self.defs[path];
        let mut enc = Enc::new(self, path, &d.mod_path);
        let mut binders = Vec::new();
        let mut facts = Vec::new();
        let mut env = BTreeMap::new();
        let mut app_args = Vec::new();
        let sig = &self.t.fns[path];
        for (p, t) in d.f.params.iter().zip(&sig.params) {
            let mut ls = Vec::new();
            leaves(t, self.t, &format!("{}!p", p.name.name), &mut ls);
            let mut names = Vec::new();
            for (name, leaf) in &ls {
                let s = sym(name);
                binders.push(format!("({s} {})", leaf.sort()));
                if let Some(f) = leaf.fact(&s) {
                    facts.push(f);
                }
                app_args.push(s.clone());
                names.push(s);
            }
            env.insert(
                p.name.name.clone(),
                build(t, self.t, &mut names.into_iter()),
            );
        }
        let value = enc.spec_block(&d.f.body, &env, &BTreeMap::new());
        let mut bodies = Vec::new();
        flatten(&value, &mut bodies);
        let guard = and(&facts);
        // One definition per SMT function: a struct result has one per leaf.
        let mut ax = Vec::new();
        for ((f, _), body) in spec_funs(path, self.t).iter().zip(&bodies) {
            let app = if app_args.is_empty() {
                f.clone()
            } else {
                format!("({f} {})", app_args.join(" "))
            };
            ax.push(if binders.is_empty() {
                format!("(assert (= {app} {body}))")
            } else {
                format!(
                    "(assert (forall ({}) (! (=> {guard} (= {app} {body})) :pattern ({app}))))",
                    binders.join(" "),
                )
            });
        }
        // Ground instances (ADR-0023) cover single-result spec fns.
        let parts = AxiomParts {
            sym: spec_key(path, self.t),
            params: app_args,
            guard,
            body: bodies.first().cloned().unwrap_or_default(),
        };
        (ax.join("\n"), parts)
    }
}

pub(crate) fn uses_of(f: &FnDef, mod_path: &str, fns: &BTreeMap<String, FnSig>) -> Vec<String> {
    let mut out = BTreeSet::new();
    for a in &f.attrs {
        if a.name.name == "uses" {
            for arg in &a.args {
                if let AttrArg::Ident(i) = arg
                    && let Some(full) = crate::check::uses_target(&i.name, mod_path, fns)
                {
                    out.insert(full);
                }
            }
        }
    }
    out.into_iter().collect()
}

/// Verifies every definition of a checked program. Failures go to `diags`.
pub fn verify_program(p: &Program, t: &Tables, diags: &mut Diags) -> Outcome {
    verify_program_seeded(p, t, diags, 0)
}

/// As `verify_program` with another solver seed. Seed 0 is the pinned
/// configuration; other seeds exist to measure proof stability (backlog
/// 1B-19): a proof whose verdict depends on the seed is brittle.
pub fn verify_program_seeded(p: &Program, t: &Tables, diags: &mut Diags, seed: u64) -> Outcome {
    verify_program_with(p, t, diags, seed, None)
}

/// As `verify_program_seeded`, skipping obligations the proof cache already
/// holds as proved (backlog 2D-01). Output is the same with or without it.
pub fn verify_program_with(
    p: &Program,
    t: &Tables,
    diags: &mut Diags,
    seed: u64,
    cache: Option<&mut ProofCache>,
) -> Outcome {
    verify_impl(p, t, diags, seed, cache, None, false, None)
}

/// As `verify_program_with` (seed 0), reusing the stored result of every
/// definition whose incremental key is unchanged (backlog 2D-10): such a
/// definition is not encoded at all. Its obligations count as proof cache
/// hits. Output is the same as without it, except that `scripts` holds only
/// the definitions that were encoded.
pub fn verify_program_incremental(
    p: &Program,
    t: &Tables,
    diags: &mut Diags,
    cache: Option<&mut ProofCache>,
    incr: &mut crate::incremental::Session,
) -> Outcome {
    verify_impl(p, t, diags, 0, cache, None, false, Some(incr))
}

/// Every solver query starts with this text: options (seed 0, pinned
/// `rlimit`) and the prelude. The declarations of the spec fns the query
/// mentions follow it (`needed_decls`).
pub fn query_header() -> String {
    header(0).join("\n")
}

/// The SMT declaration of every spec fn of a program, by full path. A query
/// holds the declarations of exactly the spec fns its text mentions (backlog
/// 2D-14), so the incremental key takes these for the spec fns a definition
/// can reach.
pub fn spec_decl_map(p: &Program, t: &Tables) -> BTreeMap<String, String> {
    let view = prepare(p, t);
    let prog = prog_of(p, &view);
    prog.spec_decls()
        .into_iter()
        .map(|(path, _, decl)| (path, decl))
        .collect()
}

/// As `verify_program_with` (seed 0), proving only the definitions in
/// `only` (full paths). Every other definition is still seen through its
/// contract and `#[uses]`, exactly as in a whole-program run, so a listed
/// definition gets the same verdict as there. Used by `bmb lint` to
/// re-verify one definition or its callers after a contract change
/// (backlog 2D-08). Lint only asks whether every listed definition verifies
/// completely, so the run stops at the first obligation that fails
/// (backlog 2D-15): after a failure, statuses and diagnostics cover only
/// what ran.
pub fn verify_defs(
    p: &Program,
    t: &Tables,
    diags: &mut Diags,
    only: &BTreeSet<String>,
    cache: Option<&mut ProofCache>,
) -> Outcome {
    verify_impl(p, t, diags, 0, cache, Some(only), true, None)
}

/// Proves only the definitions in `only` (full paths), through the
/// incremental cache when given, without stopping at a failure (backlog
/// 3A-04). Each listed definition gets the status and proved obligations
/// of a whole-program run (`verify_program_incremental` with the same
/// caches): a definition's queries are built from its own text and what it
/// sees of other definitions (contracts, `#[uses]`), never from another
/// definition's proof, and every obligation runs on a reset solver.
pub fn verify_only(
    p: &Program,
    t: &Tables,
    diags: &mut Diags,
    only: &BTreeSet<String>,
    cache: Option<&mut ProofCache>,
    incr: Option<&mut crate::incremental::Session>,
) -> Outcome {
    verify_impl(p, t, diags, 0, cache, Some(only), false, incr)
}

fn header(seed: u64) -> Vec<String> {
    vec![
        "(set-option :produce-models true)".to_string(),
        format!("(set-option :smt.random_seed {seed})"),
        format!("(set-option :sat.random_seed {seed})"),
        format!("(set-option :rlimit {})", smt::rlimit()),
        PRELUDE.to_string(),
    ]
}

/// The declarations of the spec fns whose symbol occurs in `text`, one per
/// line, in path order (backlog 2D-14). `text` is everything the query
/// asserts after them (revealed definitions, local declarations, ground
/// instances, assumptions and goal); revealed definitions are already closed
/// over the spec fns their bodies name (`relevant_axioms`), so no symbol of
/// the query is left undeclared. Symbols are `|path|`, delimited, so a
/// substring test cannot confuse two of them.
fn needed_decls(
    spec_decls: &[(String, String, String)],
    dts: &[(String, String, String)],
    text: &str,
) -> String {
    let mut out = String::new();
    for (_, s, decl) in spec_decls {
        if text.contains(s.as_str()) {
            out.push_str(decl);
            out.push('\n');
        }
    }
    // The datatypes the query or those declarations name, with the ones
    // their constructors name; `dts` lists dependencies first.
    let mut wanted: Vec<bool> = dts
        .iter()
        .map(|(pre, sort, _)| {
            text.contains(sort.as_str())
                || text.contains(pre.as_str())
                || out.contains(sort.as_str())
        })
        .collect();
    for i in (0..dts.len()).rev() {
        if wanted[i] {
            for j in 0..i {
                if dts[i].2.contains(dts[j].1.as_str()) {
                    wanted[j] = true;
                }
            }
        }
    }
    let mut head = String::new();
    for ((_, _, decl), w) in dts.iter().zip(wanted) {
        if w {
            head.push_str(decl);
            head.push('\n');
        }
    }
    head + &out
}

/// Whether the `requires` clauses of every executable function that has
/// any can hold together (backlog 2D-08). The query is the prover's own
/// encoding of the function's entry state: parameter range facts, the
/// definitions its `#[uses]` reveals, and the clauses, under the pinned seed
/// and `rlimit`. Per function with clauses, in path order: its path, the
/// span from the first to the last clause, and `Some(true)` (satisfiable),
/// `Some(false)` (unsatisfiable: the function can never be called and every
/// proof about it is vacuous) or `None` (undecided, unsupported construct,
/// or the solver could not run).
pub fn preconditions_satisfiable(p: &Program, t: &Tables) -> Vec<(String, Span, Option<bool>)> {
    let view = prepare(p, t);
    let t = &view;
    let prog = prog_of(p, t);
    let spec_decls = prog.spec_decls();
    let dts = datatypes(t);
    let mut solver: Option<Solver> = None;
    let mut solver_down = crate::toolchain::check_z3().is_err();
    let mut out = Vec::new();
    for (path, d) in &prog.defs {
        if d.inst || d.f.kind != FnKind::Exec || d.f.requires.is_empty() {
            continue;
        }
        let span = d.f.requires[0]
            .span
            .to(d.f.requires[d.f.requires.len() - 1].span);
        let mut enc = Enc::new(&prog, path, &d.mod_path);
        let mut axioms: Vec<(String, String)> = Vec::new();
        let mut axiom_parts: Vec<AxiomParts> = Vec::new();
        for u in prog.revealed(d.f, &d.mod_path) {
            let (ax, parts) = prog.axiom(&u);
            axioms.push((spec_key(&u, t), ax));
            axiom_parts.push(parts);
        }
        for prm in &d.f.params {
            let ty = enc.local_ty(&prm.name.name);
            let v = enc.fresh_val(&prm.name.name, &ty);
            enc.vars.insert(prm.name.name.clone(), v);
        }
        let (env, old) = enc.cur_env();
        for r in &d.f.requires {
            let g = enc.spec(r, &env, &old).s();
            enc.pc.push(g);
        }
        if solver_down {
            out.push((path.clone(), span, None));
            continue;
        }
        let q = enc
            .pc
            .iter()
            .map(|a| format!("(assert {a})"))
            .collect::<Vec<_>>()
            .join("\n");
        let relevant = relevant_axioms(&axioms, &q);
        let mut base = header(0).join("\n");
        base.push('\n');
        base.push_str(&sort_decls(&t.fns[path]));
        let decls = enc.decls.join("\n");
        let used = needed_decls(&spec_decls, &dts, &format!("{relevant}{decls}\n{q}"));
        let full = format!("{base}{used}{relevant}{decls}");
        if solver.as_ref().is_some_and(|s| !s.reusable()) {
            solver = None;
        }
        if solver.is_none() {
            match Solver::start() {
                Ok(s) => solver = Some(s),
                Err(_) => {
                    solver_down = true;
                    out.push((path.clone(), span, None));
                    continue;
                }
            }
        }
        let sol = solver.as_mut().expect("solver started");
        let mut answer = sol
            .run("(reset)")
            .and_then(|_| sol.run(&full))
            .and_then(|_| sol.run(&q))
            .and_then(|_| sol.check_sat());
        // Undecided with quantified definitions: ground instances follow from
        // the definitions, so `unsat` with them is `unsat` (as in the prover).
        if matches!(answer, Ok(Answer::Unknown(_))) && !axiom_parts.is_empty() {
            let (inst, _) = ground_instances(&axiom_parts, &q, 3);
            if !inst.is_empty() {
                let used = needed_decls(&spec_decls, &dts, &format!("{decls}\n{inst}{q}"));
                let head = format!("{base}{used}{decls}");
                let retry = sol
                    .run("(reset)")
                    .and_then(|_| sol.run(&head))
                    .and_then(|_| sol.run(&format!("{inst}{q}")))
                    .and_then(|_| sol.check_sat());
                if matches!(retry, Ok(Answer::Unsat)) {
                    answer = retry;
                }
            }
        }
        let verdict = match answer {
            Ok(Answer::Unsat) => Some(false),
            Ok(Answer::Sat) => Some(true),
            Ok(Answer::Unknown(_)) => None,
            Err(_) => {
                solver = None;
                None
            }
        };
        out.push((path.clone(), span, verdict));
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn verify_impl(
    p: &Program,
    t: &Tables,
    diags: &mut Diags,
    seed: u64,
    mut cache: Option<&mut ProofCache>,
    only: Option<&BTreeSet<String>>,
    stop_at_failure: bool,
    mut incr: Option<&mut crate::incremental::Session>,
) -> Outcome {
    let view = prepare(p, t);
    let t = &view;
    let prog = prog_of(p, t);
    let mut statuses = Vec::new();
    let mut scripts = BTreeMap::new();
    let mut proved_keys: BTreeSet<ObKey> = BTreeSet::new();
    let mut failed_keys: BTreeSet<ObKey> = BTreeSet::new();
    let mut vacuous = Vec::new();
    let spec_decls = prog.spec_decls();
    let dts = datatypes(t);
    let first_span = prog
        .defs
        .values()
        .next()
        .map(|d| d.f.name.span)
        .unwrap_or_default();

    if let Err(e) = crate::toolchain::check_z3() {
        diags.push(Diag::new(
            "E0812",
            first_span,
            format!("proof gate cannot run: {e}"),
        ));
    }

    // Every spec fn by SMT symbol, for proof-context suggestions (backlog 1D-13).
    let spec_syms: Vec<(String, String)> = prog
        .defs
        .iter()
        .filter(|(_, d)| d.f.kind == FnKind::Spec && !d.inst)
        .map(|(p, _)| (p.clone(), sym(p)))
        .collect();
    // One solver process serves every definition (backlog 1D-18): each
    // obligation starts with `(reset)` and re-sends its whole base, options
    // included (ADR-0018 §1), so nothing carries over between obligations.
    // Started on the first obligation the cache does not hold, replaced when
    // too old to take on another definition (`Solver::reusable`).
    let mut solver: Option<Solver> = None;
    let mut stopped = false;
    for (path, d) in &prog.defs {
        if stopped {
            break;
        }
        if d.inst {
            continue;
        }
        if only.is_some_and(|o| !o.contains(path)) {
            continue;
        }
        // Incremental build (backlog 2D-10): an unchanged, fully proved
        // definition is not encoded again.
        let hit = incr.as_ref().and_then(|s| s.proof_lookup(path, d.f.span));
        if let Some(h) = &hit
            && !incr.as_ref().is_some_and(|s| s.audit())
        {
            if let Some(c) = cache.as_mut() {
                c.hits += h.status.obligations;
            }
            proved_keys.extend(h.proved.iter().cloned());
            statuses.push(h.status.clone());
            if let Some(s) = incr.as_mut() {
                s.stats.reused.push(path.clone());
            }
            continue;
        }
        let mut enc = Enc::new(&prog, path, &d.mod_path);
        let header = header(seed);
        // Definitional axioms of revealed spec fns, filtered per obligation.
        let mut axioms: Vec<(String, String)> = Vec::new();
        let mut axiom_parts: Vec<AxiomParts> = Vec::new();

        match d.f.kind {
            FnKind::Spec => {
                // Only recursive spec fns carry obligations (termination).
                let Some(comp) = t.recursive.get(path).copied() else {
                    continue;
                };
                let Some(dec) = &d.f.decreases else { continue };
                let mut env = BTreeMap::new();
                for prm in &d.f.params {
                    let ty = enc.local_ty(&prm.name.name);
                    let v = enc.fresh_val(&prm.name.name, &ty);
                    env.insert(prm.name.name.clone(), v);
                }
                let (m, data) = enc.measure(dec, &env, &BTreeMap::new());
                enc.term_check = Some((comp, m, data));
                enc.spec_block(&d.f.body, &env, &BTreeMap::new());
                enc.term_check = None;
            }
            FnKind::Exec => {
                for u in prog.revealed(d.f, &d.mod_path) {
                    let (ax, parts) = prog.axiom(&u);
                    axioms.push((spec_key(&u, t), ax));
                    axiom_parts.push(parts);
                }
                for prm in &d.f.params {
                    let ty = enc.local_ty(&prm.name.name);
                    let v = enc.fresh_val(&prm.name.name, &ty);
                    if let TypeKind::Ref { mutable: true, .. } = prm.ty.kind {
                        enc.old.insert(prm.name.name.clone(), v.clone());
                    }
                    enc.inputs.push((prm.name.name.clone(), v.clone(), ty));
                    enc.vars.insert(prm.name.name.clone(), v);
                }
                if let Some((decl, rsub)) = refinement_of(t, path) {
                    enc.refines(d.f, &decl, rsub);
                }
                let (env, old) = enc.cur_env();
                for r in &d.f.requires {
                    let g = enc.spec(r, &env, &old).s();
                    enc.pc.push(g);
                }
                if let Some(dec) = &d.f.decreases {
                    enc.entry_measure = Some(enc.measure(dec, &env, &old));
                }
                let result = enc.block(&d.f.body);
                enc.mark_reach(d.f.name.span, "end of the body");
                let (mut env, old) = enc.cur_env();
                if let Some(n) = &d.f.ret.name {
                    env.insert(n.name.clone(), result);
                }
                for en in &d.f.ensures {
                    let g = enc.spec(en, &env, &old).s();
                    enc.oblige(
                        "E0802",
                        en.span,
                        format!("postcondition `{}` may not hold", pred_name(en)),
                        g,
                    );
                }
            }
        }

        if REACH.with(|r| r.get()) {
            let mut prefix = header.join("\n");
            prefix.push('\n');
            prefix.push_str(&sort_decls(&t.fns[path]));
            let decls = enc.decls.join("\n");
            for (span, what, pc) in &enc.reach {
                // A refutation comes fast; a model may take long: a small
                // budget, and undecided is not reported.
                let q = format!(
                    "(set-option :rlimit 1000000)\n{}",
                    pc.iter()
                        .map(|a| format!("(assert {a})"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                let relevant = relevant_axioms(&axioms, &q);
                let used = needed_decls(&spec_decls, &dts, &format!("{relevant}{decls}\n{q}"));
                let base = format!("{prefix}{used}{relevant}{decls}");
                if solver.as_ref().is_some_and(|s| !s.reusable()) {
                    solver = None;
                }
                if solver.is_none() {
                    solver = Solver::start().ok();
                }
                let Some(sol) = solver.as_mut() else { break };
                let answer = sol
                    .run("(reset)")
                    .and_then(|_| sol.run(&base))
                    .and_then(|_| sol.run(&q))
                    .and_then(|_| sol.check_sat());
                if matches!(answer, Ok(Answer::Unsat)) {
                    vacuous.push((path.clone(), *span, *what));
                }
            }
            continue;
        }

        // Assemble and solve. Every obligation runs on a reset solver holding only
        // the definition's base (options, prelude, axioms, declarations), so one
        // obligation's result never depends on another's (ADR-0017 §5).
        let mut prefix = header.join("\n");
        prefix.push('\n');
        prefix.push_str(&sort_decls(&t.fns[path]));
        let decls = enc.decls.join("\n");
        let mut script = String::new();
        let mut failed = 0usize;
        let (mut rlimit_total, mut rlimit_max) = (0u64, 0u64);
        if solver.as_ref().is_some_and(|s| !s.reusable()) {
            solver = None;
        }
        let mut solver_down = false;
        let mut def_proved: Vec<ObKey> = Vec::new();
        for (k, o) in enc.obls.iter().enumerate() {
            if stop_at_failure && failed > 0 {
                break;
            }
            let q = format!(
                "{}\n(assert (not {}))",
                o.assumptions
                    .iter()
                    .map(|a| format!("(assert {a})"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                o.goal
            );
            // Only definitions the obligation can reach (DESIGN.md §4.1:
            // unrelated context is the main source of solver instability).
            let relevant = relevant_axioms(&axioms, &q);
            let relevant_text = relevant.clone();
            let hint =
                hidden_predicates(&spec_syms, &axioms, &format!("{q}{relevant}"), &d.mod_path);
            // Only the spec fns the query mentions are declared (backlog
            // 2D-14): an unrelated spec fn does not change the query text.
            let used = needed_decls(&spec_decls, &dts, &format!("{relevant}{decls}\n{q}"));
            let base = format!("{prefix}{used}{relevant}{decls}");
            script.push_str(&format!(
                "; obligation {k}: {} {}\n(reset)\n{base}\n{q}\n(check-sat)\n",
                o.code, o.msg
            ));
            let key = (o.code, o.span.file, o.span.lo, o.span.hi);
            let cache_key = cache
                .as_ref()
                .map(|_| ProofCache::key(&format!("{base}\n{q}")));
            // The ground-instance retry of an obligation: the same text
            // whether it runs or comes from the proof cache (2V-03).
            let retry_script = |q: &str| {
                let (inst, complete) = ground_instances(&axiom_parts, q, 3);
                if inst.is_empty() {
                    return None;
                }
                let used = needed_decls(&spec_decls, &dts, &format!("{decls}\n{inst}{q}"));
                Some((format!("{prefix}{used}{decls}"), inst, complete))
            };
            if let (Some(c), Some(ck)) = (cache.as_mut(), cache_key.as_ref())
                && let Some((total, max, retried)) = c.get(ck)
            {
                rlimit_total += total;
                rlimit_max = rlimit_max.max(max);
                if retried && let Some((head, inst, _)) = retry_script(&q) {
                    script.push_str(&format!(
                        "; retry of obligation {k} with ground instances\n(reset)\n{head}\n{inst}{q}\n(check-sat)\n"
                    ));
                }
                proved_keys.insert(key);
                def_proved.push(key);
                continue;
            }
            if solver.is_none() && !solver_down {
                match Solver::start() {
                    Ok(s) => solver = Some(s),
                    Err(e) => {
                        diags.push(Diag::new(
                            "E0812",
                            d.f.name.span,
                            format!("solver error: {e}"),
                        ));
                        solver_down = true;
                    }
                }
            }
            let Some(sol) = solver.as_mut() else {
                failed += 1;
                continue;
            };
            let total_before = rlimit_total;
            let mut first_used = 0;
            let mut retried = false;
            let mut answer = sol
                .run("(reset)")
                .and_then(|_| sol.run(&base))
                .and_then(|_| sol.run(&q))
                .and_then(|_| sol.check_sat());
            if answer.is_ok()
                && let Ok(used) = sol.rlimit_used()
            {
                rlimit_total += used;
                rlimit_max = rlimit_max.max(used);
                first_used = used;
            }
            // Undecided with quantified definitions: retry quantifier-free with
            // ground instances (sound for both answers, see `ground_instances`).
            if matches!(answer, Ok(Answer::Unknown(_)))
                && !axiom_parts.is_empty()
                && let Some((head, inst, complete)) = retry_script(&q)
            {
                script.push_str(&format!(
                        "; retry of obligation {k} with ground instances\n(reset)\n{head}\n{inst}{q}\n(check-sat)\n"
                    ));
                let retry = sol
                    .run("(reset)")
                    .and_then(|_| sol.run(&head))
                    .and_then(|_| sol.run(&format!("{inst}{q}")))
                    .and_then(|_| sol.check_sat());
                if retry.is_ok()
                    && let Ok(used) = sol.rlimit_used()
                {
                    rlimit_total += used;
                }
                match retry {
                    Ok(Answer::Unsat) => {
                        answer = retry;
                        retried = true;
                    }
                    Ok(Answer::Sat) if complete => answer = retry,
                    Ok(Answer::Sat) => {
                        answer = Ok(Answer::Unknown(
                            "a model exists for the instantiated definitions only".into(),
                        ))
                    }
                    _ => {}
                }
            }
            failed += 1;
            if matches!(answer, Ok(Answer::Unsat)) {
                proved_keys.insert(key);
                def_proved.push(key);
                if let (Some(c), Some(ck)) = (cache.as_mut(), cache_key) {
                    c.insert(ck, rlimit_total - total_before, first_used, retried);
                }
            } else {
                failed_keys.insert(key);
            }
            match answer {
                Ok(Answer::Unsat) => failed -= 1,
                Ok(Answer::Sat) => {
                    let cex = counterexample(sol, &enc.inputs, t);
                    let mut msg = format!("{} in `{path}`", o.msg);
                    if !cex.is_empty() {
                        msg.push_str(&format!("; counterexample: {cex}"));
                    }
                    diags.push(with_hint(Diag::new(o.code, o.span, msg), &hint, false));
                }
                Ok(Answer::Unknown(why)) => {
                    let why = if why.contains("canceled") || why.contains("resource") {
                        format!("resource limit reached, rlimit {}", smt::rlimit())
                    } else {
                        why
                    };
                    let mut msg = format!("{} in `{path}`: solver could not decide ({why})", o.msg);
                    // Z3 often keeps a candidate model; show it, marked as unconfirmed.
                    let cex = counterexample(sol, &enc.inputs, t);
                    if !cex.is_empty() {
                        msg.push_str(&format!("; unconfirmed counterexample: {cex}"));
                    }
                    diags.push(with_hint(
                        Diag::new("E0811", o.span, msg),
                        &hint,
                        has_nonlinear_product(&format!("{q}{relevant_text}")),
                    ));
                }
                // The resource limit can also stop a command before `check-sat`.
                Err(e) if e.contains("canceled") => diags.push(Diag::new(
                    "E0811",
                    o.span,
                    format!(
                        "{} in `{path}`: solver could not decide (resource limit reached, rlimit {})",
                        o.msg,
                        smt::rlimit()
                    ),
                )),
                Err(e) => {
                    diags.push(Diag::new("E0812", o.span, format!("solver error: {e}")));
                    solver = None;
                    solver_down = true;
                }
            }
        }
        stopped = stop_at_failure && failed > 0;
        let status = FnStatus {
            path: path.clone(),
            obligations: enc.obls.len(),
            failed,
            rlimit_total,
            rlimit_max,
        };
        if let Some(s) = incr.as_mut() {
            s.stats.encoded.push(path.clone());
            if let Some(h) = &hit {
                s.proof_audit(h, &status, &def_proved, &script);
            }
            s.proof_record(&status, d.f.span, &def_proved, &script);
        }
        statuses.push(status);
        scripts.insert(path.clone(), script);
    }
    let proven = proved_keys.difference(&failed_keys).cloned().collect();
    Outcome {
        statuses,
        proven,
        scripts,
        vacuous,
    }
}

/// A spec fn's definition, kept apart for ground instantiation.
struct AxiomParts {
    sym: String,
    /// Bound parameter symbols, in application order.
    params: Vec<String>,
    guard: String,
    body: String,
}

/// Ground instances of the definitions for the applications occurring in
/// `text`, to `depth` levels of nesting (backlog 1B-14). Instances follow from
/// the quantified definitions, so a proof with them is a proof. The second
/// result is true when every application got its instance and no quantifier
/// remains, which makes a model of the instances a model of the definitions.
fn ground_instances(axioms: &[AxiomParts], text: &str, depth: usize) -> (String, bool) {
    let mut seen = BTreeSet::new();
    let mut out = String::new();
    let mut frontier = text.to_string();
    let mut complete = true;
    for level in 0..=depth {
        let mut next = String::new();
        let mut apps = Vec::new();
        for cmd in smt::split_commands(&frontier) {
            if let Some(e) = smt::parse(&cmd) {
                collect_apps(&e, axioms, &mut apps);
            }
        }
        // Zero-arity definitions appear as bare symbols.
        for a in axioms.iter().filter(|a| a.params.is_empty()) {
            if frontier.contains(&a.sym) {
                apps.push((a.sym.clone(), a.sym.clone(), Vec::new()));
            }
        }
        for (app, symbol, args) in apps {
            if !seen.insert(app.clone()) {
                continue;
            }
            if level == depth {
                complete = false;
                break;
            }
            let ax = axioms.iter().find(|a| a.sym == symbol).unwrap();
            let subst = |t: &str| {
                let mut t = t.to_string();
                for (p, a) in ax.params.iter().zip(&args) {
                    t = t.replace(p.as_str(), a);
                }
                t
            };
            let body = subst(&ax.body);
            if body.contains("(forall") {
                complete = false;
            }
            out.push_str(&format!(
                "(assert (=> {} (= {app} {body})))\n",
                subst(&ax.guard)
            ));
            next.push_str(&body);
            next.push(' ');
        }
        if next.is_empty() {
            break;
        }
        frontier = format!("(assert (and {next} true))");
    }
    (out, complete)
}

fn collect_apps(
    e: &smt::Sexp,
    axioms: &[AxiomParts],
    out: &mut Vec<(String, String, Vec<String>)>,
) {
    if let smt::Sexp::List(xs) = e {
        if let Some(smt::Sexp::Atom(head)) = xs.first()
            && axioms
                .iter()
                .any(|a| &a.sym == head && a.params.len() == xs.len() - 1)
        {
            let args: Vec<String> = xs[1..].iter().map(smt::print).collect();
            out.push((smt::print(e), head.clone(), args));
        }
        // Bound variables of a quantifier cannot be instantiated from outside.
        if xs.first() != Some(&smt::Sexp::Atom("forall".into())) {
            xs.iter().for_each(|x| collect_apps(x, axioms, out));
        }
    }
}

/// Spec fns the obligation mentions by name but whose definition the proof
/// cannot see (not in `#[uses]`), as names to add to `#[uses(..)]`.
fn hidden_predicates(
    spec_syms: &[(String, String)],
    revealed: &[(String, String)],
    text: &str,
    unit: &str,
) -> Vec<String> {
    spec_syms
        .iter()
        .filter(|(_, s)| text.contains(s.as_str()) && !revealed.iter().any(|(r, _)| r == s))
        .map(|(p, _)| match p.strip_prefix(&format!("{unit}::")) {
            Some(local) if !local.contains("::") => local.to_string(),
            _ => p.clone(),
        })
        .collect()
}

/// Attaches the deterministic hints that apply to a failed obligation.
fn with_hint(d: Diag, hidden: &[String], nonlinear: bool) -> Diag {
    let mut parts = Vec::new();
    if !hidden.is_empty() {
        parts.push(format!(
            "the proof sees {} only by name; if the failure depends on {}, add to #[uses({})]",
            hidden
                .iter()
                .map(|h| format!("`{h}`"))
                .collect::<Vec<_>>()
                .join(", "),
            if hidden.len() == 1 {
                "its definition"
            } else {
                "their definitions"
            },
            hidden.join(", ")
        ));
    }
    if nonlinear {
        parts.push(
            "the obligation multiplies variables (nonlinear arithmetic), which the solver cannot \
             always decide; state the needed fact as a precondition or as the postcondition of a \
             helper, or bound the operands"
                .to_string(),
        );
    }
    if parts.is_empty() {
        d
    } else {
        d.fix(parts.join("; also, "), None)
    }
}

/// True if some term multiplies two non-constant factors (backlog 1B-16).
pub fn has_nonlinear_product(text: &str) -> bool {
    fn constant(e: &smt::Sexp) -> bool {
        match e {
            smt::Sexp::Atom(a) => a.chars().all(|c| c.is_ascii_digit()),
            smt::Sexp::List(xs) => {
                xs.len() == 2 && xs[0] == smt::Sexp::Atom("-".into()) && constant(&xs[1])
            }
        }
    }
    fn walk(e: &smt::Sexp) -> bool {
        match e {
            smt::Sexp::Atom(_) => false,
            smt::Sexp::List(xs) => {
                let product = xs.first() == Some(&smt::Sexp::Atom("*".into()))
                    && xs[1..].iter().filter(|x| !constant(x)).count() >= 2;
                product || xs.iter().any(walk)
            }
        }
    }
    smt::split_commands(text)
        .iter()
        .filter_map(|c| smt::parse(c))
        .any(|e| walk(&e))
}

/// Axioms whose symbol occurs in `text`, closed over the axioms' own bodies.
fn relevant_axioms(axioms: &[(String, String)], text: &str) -> String {
    let mut chosen = vec![false; axioms.len()];
    let mut haystack = text.to_string();
    loop {
        let mut grew = false;
        for (k, (symbol, ax)) in axioms.iter().enumerate() {
            if !chosen[k] && haystack.contains(symbol.as_str()) {
                chosen[k] = true;
                haystack.push_str(ax);
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    let mut out = String::new();
    for (k, (_, ax)) in axioms.iter().enumerate() {
        if chosen[k] {
            out.push_str(ax);
            out.push('\n');
        }
    }
    out
}

/// `x = 5, a = [1, 2] (len 2), p = Point { x: 1, y: 2 }` from the current
/// model.
fn counterexample(sol: &mut Solver, inputs: &[(String, Val, TypeKind)], tb: &Tables) -> String {
    let mut terms = Vec::new();
    for (_, v, _) in inputs {
        model_terms(v, &mut terms);
    }
    let Ok(vals) = sol.get_values(&terms) else {
        return String::new();
    };
    let mut it = vals.into_iter();
    let mut parts = Vec::new();
    for (name, v, t) in inputs {
        if *v != Val::Unit {
            parts.push(format!("{name} = {}", show(v, t, tb, &mut it)));
        }
    }
    parts.join(", ")
}

/// Elements of an array shown in a counterexample.
const SHOW: usize = 4;

/// A datatype value from the model in Rust notation: `Some(992)`,
/// `Shape::Rect(Point { x: 1, y: 2 }, ..)` (ADR-0026 §4).
fn show_variant(x: &smt::Sexp, t: &TypeKind, tb: &Tables) -> String {
    let (head, args) = match x {
        smt::Sexp::Atom(a) => (a.clone(), &[][..]),
        smt::Sexp::List(xs) if !xs.is_empty() => match &xs[0] {
            smt::Sexp::Atom(a) => (a.clone(), &xs[1..]),
            _ => return smt::render_value(x),
        },
        _ => return smt::render_value(x),
    };
    let ts = crate::check::ty_str(value_ty(t));
    let Some(v) = head
        .strip_prefix(&format!("|{ts}::"))
        .and_then(|r| r.strip_suffix('|'))
    else {
        return smt::render_value(x);
    };
    let vs = enum_variants(t, tb).unwrap_or_default();
    let ps = vs
        .iter()
        .find(|(n, _)| n == v)
        .map(|(_, ps)| ps.clone())
        .unwrap_or_default();
    let name = if matches!(ts.split('<').next(), Some("Option" | "Result")) {
        v.to_string()
    } else {
        format!("{}::{v}", ts.rsplit("::").next().unwrap_or(&ts))
    };
    if ps.is_empty() {
        return name;
    }
    let mut it = args.iter();
    let shown: Vec<String> = ps.iter().map(|pt| show_leaves(pt, tb, &mut it)).collect();
    format!("{name}({})", shown.join(", "))
}

/// A value of type `t` from its model leaves (constructor arguments).
fn show_leaves<'a>(
    t: &TypeKind,
    tb: &Tables,
    it: &mut impl Iterator<Item = &'a smt::Sexp>,
) -> String {
    if let Some(fs) = struct_fields(t, tb) {
        let short = crate::check::ty_str(value_ty(t));
        let short = short.rsplit("::").next().unwrap_or(&short).to_string();
        let shown: Vec<String> = fs
            .iter()
            .map(|(n, ft)| format!("{n}: {}", show_leaves(ft, tb, it)))
            .collect();
        return format!("{short} {{ {} }}", shown.join(", "));
    }
    if is_array(t) {
        // A payload array of scalars: its elements read from the model's
        // array value (2V-05); anything else is elided.
        let (elems, len) = (it.next(), it.next());
        let e = elem_ty(t);
        let n = len
            .map(smt::render_value)
            .and_then(|l| l.parse::<i64>().ok());
        if let (Some(a), Some(n)) = (elems, n)
            && (is_scalar_ty(&e) || enum_variants(&e, tb).is_some())
        {
            let shown: Option<Vec<String>> = (0..n.min(SHOW as i64))
                .map(|k| {
                    model_select(a, k).map(|x| match enum_variants(&e, tb) {
                        Some(_) => show_variant(&x, &e, tb),
                        None => smt::render_value(&x),
                    })
                })
                .collect();
            if let Some(xs) = shown {
                let more = if n > SHOW as i64 { ", .." } else { "" };
                return format!("[{}{more}]", xs.join(", "));
            }
        }
        return "[..]".into();
    }
    match it.next() {
        Some(x) if enum_variants(t, tb).is_some() => show_variant(x, t, tb),
        Some(x) => smt::render_value(x),
        None => String::new(),
    }
}

fn is_scalar_ty(t: &TypeKind) -> bool {
    matches!(
        t,
        TypeKind::I64 | TypeKind::U64 | TypeKind::Usize | TypeKind::U8 | TypeKind::Bool
    )
}

/// Element `k` of an array value as Z3 prints it in a model: a constant
/// array, `store`s over one, or a `lambda` whose body is an `ite` chain on
/// the index. `None` for any other form.
fn model_select(a: &smt::Sexp, k: i64) -> Option<smt::Sexp> {
    use smt::Sexp::{Atom, List};
    let num = |x: &smt::Sexp| smt::render_value(x).parse::<i64>().ok();
    let List(xs) = a else { return None };
    match xs.as_slice() {
        // ((as const (Array Int T)) v)
        [List(h), v] if matches!(h.first(), Some(Atom(x)) if x == "as") => Some(v.clone()),
        [Atom(s), arr, i, v] if s == "store" => {
            if num(i)? == k {
                Some(v.clone())
            } else {
                model_select(arr, k)
            }
        }
        [Atom(l), List(params), body] if l == "lambda" => {
            let List(p) = params.first()? else {
                return None;
            };
            let Atom(x) = p.first()? else { return None };
            let mut e = body.clone();
            loop {
                match &e {
                    List(ys) if ys.len() == 4 && ys[0] == Atom("ite".into()) => {
                        let List(c) = &ys[1] else { return None };
                        let hit = c.len() == 3
                            && c[0] == Atom("=".into())
                            && ((c[1] == Atom(x.clone()) && num(&c[2]) == Some(k))
                                || (c[2] == Atom(x.clone()) && num(&c[1]) == Some(k)));
                        e = if hit { ys[2].clone() } else { ys[3].clone() };
                    }
                    other => return Some(other.clone()),
                }
            }
        }
        _ => None,
    }
}

/// The terms whose model values show `v`, in the order `show` reads them.
fn model_terms(v: &Val, terms: &mut Vec<String>) {
    match v {
        Val::S(t) => terms.push(t.clone()),
        Val::A { elems, len } => {
            terms.push(len.clone());
            for k in 0..SHOW {
                terms.push(format!("(select {elems} {k})"));
            }
        }
        Val::Struct(fs) => fs.iter().for_each(|f| model_terms(f, terms)),
        Val::Arr { elem, len } => {
            terms.push(len.clone());
            for k in 0..SHOW {
                model_terms(&select_val(elem, &k.to_string()), terms);
            }
        }
        Val::Unit => {}
    }
}

/// `v` in Rust notation from its model values.
fn show(v: &Val, t: &TypeKind, tb: &Tables, it: &mut impl Iterator<Item = String>) -> String {
    match v {
        Val::S(_) if enum_variants(t, tb).is_some() => {
            let raw = it.next().unwrap_or_default();
            match smt::parse(&raw) {
                Some(x) => show_variant(&x, t, tb),
                None => raw,
            }
        }
        Val::S(_) => it.next().unwrap_or_default(),
        Val::A { .. } => {
            let len = it.next().unwrap_or_default();
            let n: usize = len.parse().unwrap_or(SHOW).min(SHOW);
            // Consume all SHOW values even when fewer are shown: the ones
            // after belong to the next input.
            let shown: Vec<String> = (0..SHOW).filter_map(|_| it.next()).collect();
            let e = elem_ty(t);
            let elems: Vec<String> = shown
                .into_iter()
                .take(n)
                .map(|x| match smt::parse(&x) {
                    Some(v) if enum_variants(&e, tb).is_some() => show_variant(&v, &e, tb),
                    _ => x,
                })
                .collect();
            let more = if len.parse::<usize>().map(|l| l > SHOW).unwrap_or(false) {
                ", .."
            } else {
                ""
            };
            format!("[{}{more}] (len {len})", elems.join(", "))
        }
        Val::Struct(fs) => {
            let decl = struct_fields(t, tb).map(|f| f.to_vec()).unwrap_or_default();
            let name = match t {
                TypeKind::Ref { inner, .. } => crate::check::ty_str(&inner.kind),
                t => crate::check::ty_str(t),
            };
            let short = name.rsplit("::").next().unwrap_or(&name).to_string();
            let shown: Vec<String> = fs
                .iter()
                .zip(&decl)
                .map(|(f, (n, ft))| format!("{n}: {}", show(f, ft, tb, it)))
                .collect();
            format!("{short} {{ {} }}", shown.join(", "))
        }
        Val::Arr { elem, .. } => {
            let len = it.next().unwrap_or_default();
            let n: usize = len.parse().unwrap_or(SHOW).min(SHOW);
            let e = elem_ty(t);
            let shown: Vec<String> = (0..SHOW)
                .map(|k| show(&select_val(elem, &k.to_string()), &e, tb, it))
                .take(SHOW)
                .collect();
            let more = if len.parse::<usize>().map(|l| l > SHOW).unwrap_or(false) {
                ", .."
            } else {
                ""
            };
            let elems: Vec<String> = shown.into_iter().take(n).collect();
            format!("[{}{more}] (len {len})", elems.join(", "))
        }
        Val::Unit => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ground instances (ADR-0023) take datatype terms as arguments: the
    /// applications are found and substituted whole (backlog 2B-05d).
    #[test]
    fn ground_instances_take_datatype_arguments() {
        let ax = AxiomParts {
            sym: "|m::is_some|".into(),
            params: vec!["|o!p|".into()],
            guard: "true".into(),
            body: "((_ is |Option<i64>::Some|) |o!p|)".into(),
        };
        let text = "(assert (not (|m::is_some| (|Option<i64>::Some| (- 5)))))";
        let (inst, complete) = ground_instances(&[ax], text, 3);
        assert!(complete, "{inst}");
        assert_eq!(
            inst,
            "(assert (=> true (= (|m::is_some| (|Option<i64>::Some| (- 5))) ((_ is |Option<i64>::Some|) (|Option<i64>::Some| (- 5))))))\n"
        );
    }
}
