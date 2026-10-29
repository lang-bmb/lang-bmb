//! Proof gate (backlog 1B-02 .. 1B-11, semantics in ADR-0017).
//!
//! Each executable function is symbolically executed over its checked syntax
//! tree. Local variables become single-assignment SMT constants; loops are cut
//! by their invariants; calls assume only the callee's `ensures`. Every implicit
//! and explicit proof obligation is checked separately by Z3. Only `unsat`
//! proves an obligation.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

mod invariants;
use invariants::Boundary;

use crate::ast::*;
use crate::cache::ProofCache;
use crate::check::FnSig;
use crate::diag::{Diag, Diags};
use crate::smt::{self, Answer, Solver};
use crate::source::Span;

/// `isize::MAX / size_of(T)` (ADR-0017 §3).
const LEN_MAX_WORD: &str = "1152921504606846975";
const LEN_MAX_BYTE: &str = "9223372036854775807";
/// `isize::MAX / 2` and `isize::MAX / 4`: elements of 16 and 32 bits.
const LEN_MAX_HALF: &str = "4611686018427387903";
const LEN_MAX_QUARTER: &str = "2305843009213693951";

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
        | ExprKind::Try(x)
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
        ExprKind::Return(Some(x)) | ExprKind::Break { value: Some(x), .. } => sub(x),
        ExprKind::Loop {
            invariants,
            decreases,
            body,
            ..
        } => {
            invariants.iter().for_each(|x| each_expr(x, f));
            if let Some(d) = decreases {
                each_expr(d, f);
            }
            each_block_expr(body, f);
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
            Stmt::For {
                lo,
                hi,
                invariants,
                body,
                ..
            } => {
                each_expr(lo, f);
                each_expr(hi, f);
                invariants.iter().for_each(|x| each_expr(x, f));
                each_block_expr(body, f);
            }
            Stmt::Assert { cond, .. } => each_expr(cond, f),
            Stmt::Expr(e) => each_expr(e, f),
            Stmt::Proof { body, .. } => each_block_expr(body, f),
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
    loop {
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
            // An instance's values have their types under `sub` (their
            // invariants' instances, below).
            if !sub.is_empty() {
                if let Some(ls) = view.locals.get(&path) {
                    types.extend(ls.values().map(|t| crate::check::subst(t, &sub)));
                }
                if let Some(sig) = view.fns.get(&path) {
                    types.extend(sig.params.iter().map(|t| crate::check::subst(t, &sub)));
                    types.push(crate::check::subst(&sig.ret, &sub));
                }
            }
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
                let args: Vec<TypeKind> =
                    targs.iter().map(|a| crate::check::subst(a, &sub)).collect();
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
        // The invariant predicates of generic types (ADR-0060 §2) at every
        // type a value of the program has: an instance's body may call further
        // generic spec fns.
        let mut all: Vec<TypeKind> = view.types.values().cloned().collect();
        all.extend(view.locals.values().flat_map(|m| m.values().cloned()));
        for s in view.fns.values() {
            all.extend(s.params.iter().cloned());
            all.push(s.ret.clone());
        }
        all.extend(types.iter().cloned());
        let mut grew = false;
        for (key, base, sub) in invariants::instances(&view, &all) {
            if let std::collections::btree_map::Entry::Vacant(v) = insts.entry(key) {
                v.insert((base.clone(), sub.clone()));
                work.push((base, sub, true));
                grew = true;
            }
        }
        if !grew {
            break;
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
    let struct_ctors = dts
        .values()
        .filter(|d| d.arb.is_some())
        .filter_map(|d| d.ctors.first().map(|(c, _)| c.clone()))
        .collect();
    let struct_sels = dts.into_values().flat_map(|d| d.leaves).collect();
    Prog {
        defs,
        t,
        selectors,
        struct_ctors,
        struct_sels,
        valid_memo: RefCell::default(),
    }
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
    /// Set by `contract_refinements`: the comparison of each listed
    /// definition, encoded instead of its body; the obligations each one
    /// made are recorded.
    static REFINE: RefCell<Option<Refining>> = const { RefCell::new(None) };
}

/// Where the obligations of `Enc::refines` stand.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RefineAt {
    /// A trait impl function against its trait (E0817): the `ensures`
    /// obligations at the function's name.
    Trait,
    /// Two versions of a definition (backlog 2D-19): each obligation at
    /// its clause, the result shown in a counterexample.
    Version { own_domain: bool },
}

/// One contract comparison of `contract_refinements`: the definition it
/// is listed under (the refiner) against `decl`, as a trait impl
/// function's contract against its trait's (ADR-0046 §4): `decl`'s
/// `requires` imply each of the refiner's, and `decl`'s `requires` with
/// the refiner's `ensures` imply each of `decl`'s `ensures`.
#[derive(Clone, Debug)]
pub struct RefineJob {
    pub decl: String,
    /// `decl`'s type parameters by the refiner's (by position).
    pub tsub: BTreeMap<String, TypeKind>,
    /// The `ensures` obligations assume the refiner's own `requires` in
    /// place of `decl`'s: both sides' `ensures` compared on the
    /// refiner's domain.
    pub own_domain: bool,
}

struct Refining {
    jobs: BTreeMap<String, RefineJob>,
    /// The obligations each refiner made: (code, span).
    made: Vec<(String, Vec<(&'static str, Span)>)>,
}

/// The answer to one obligation of a contract comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClauseAnswer {
    Proved,
    /// The solver found a model: the counterexample (inputs, and the result
    /// for an `ensures` clause), as Rust values, and the spec fns the query
    /// saw only by name (the model interprets them freely, as a caller's
    /// proof that does not reveal them may).
    Refuted {
        counterexample: String,
        opaque: Vec<String>,
    },
    /// Unknown, resource limit, or no solver (G1: never a guess).
    Undecided(String),
}

/// The contract comparisons `jobs` (backlog 2D-19, `weaken.rs`), each by the
/// prover's own encoding: the refiner's parameters on entry, the spec fns
/// its `#[uses]` reveals, every obligation on a reset solver with the
/// pinned `rlimit` and the ground-instance retry. No body is encoded.
/// Returns, per refiner, each obligation's span and answer, in the order
/// made: `requires` obligations at the refiner's clauses, `ensures`
/// obligations at `decl`'s.
pub fn contract_refinements(
    p: &Program,
    t: &Tables,
    jobs: BTreeMap<String, RefineJob>,
) -> BTreeMap<String, Vec<(Span, ClauseAnswer)>> {
    let only: BTreeSet<String> = jobs.keys().cloned().collect();
    let prev = REFINE.with(|r| {
        r.replace(Some(Refining {
            jobs,
            made: Vec::new(),
        }))
    });
    let mut diags = Diags::default();
    let o = verify_impl(p, t, &mut diags, 0, None, Some(&only), false, None);
    let made = REFINE
        .with(|r| std::mem::replace(&mut *r.borrow_mut(), prev))
        .map(|r| r.made)
        .unwrap_or_default();
    let mut out = BTreeMap::new();
    for (path, obls) in made {
        let answers = obls
            .into_iter()
            .map(|(code, span)| {
                let key = (code, span.file, span.lo, span.hi);
                let at = |d: &&Diag| d.span == span;
                let answer = if o.proven.contains(&key) {
                    ClauseAnswer::Proved
                } else if let Some(d) = diags.items.iter().find(|d| at(d) && d.code == code) {
                    let cex = d
                        .message
                        .split_once("; counterexample: ")
                        .map_or("", |(_, c)| c);
                    // The hint `with_hint` attaches names them.
                    let opaque = d
                        .fix
                        .as_ref()
                        .and_then(|f| f.message.strip_prefix("the proof sees "))
                        .and_then(|m| m.split_once(" only by name"))
                        .map(|(names, _)| {
                            names
                                .split(", ")
                                .map(|n| n.trim_matches('`').to_string())
                                .collect()
                        })
                        .unwrap_or_default();
                    ClauseAnswer::Refuted {
                        counterexample: cex.to_string(),
                        opaque,
                    }
                } else {
                    let why = diags
                        .items
                        .iter()
                        .find(at)
                        .or_else(|| diags.items.iter().find(|d| d.code == "E0812"))
                        .map_or("not decided".to_string(), |d| {
                            // The reason, without the prover's own names
                            // and an unconfirmed model.
                            let m = &d.message;
                            let m = m
                                .find("solver could not decide")
                                .map_or(m.as_str(), |k| &m[k..]);
                            m.split("; unconfirmed counterexample")
                                .next()
                                .unwrap_or(m)
                                .to_string()
                        });
                    ClauseAnswer::Undecided(why)
                };
                (span, answer)
            })
            .collect();
        out.insert(path, answers);
    }
    out
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
        // An array inside the element (backlog 2V-04b): its leaves are
        // arrays over the outer index; a fixed length is a constant, an
        // inner `Vec`'s is one more array (backlog 2O-03b).
        Val::A { elems, len } => Val::A {
            elems: format!("(select {elems} {i})"),
            len: select_len(len, i),
        },
        Val::Arr { elem, len } => Val::Arr {
            elem: Box::new(select_val(elem, i)),
            len: select_len(len, i),
        },
        v => panic!("not an element template {v:?}"),
    }
}

/// The range of every inner `Vec` length in element template `v` of type
/// `e` (its lengths are arrays over the outer index), at the first level.
fn inner_len_facts(v: &Val, e: &TypeKind, tb: &Tables, out: &mut Vec<String>) {
    if let (Some(fs), Val::Struct(xs)) = (flat_fields(e, tb), v) {
        for ((_, ft), x) in fs.iter().zip(xs) {
            inner_len_facts(x, ft, tb, out);
        }
        return;
    }
    if let (TypeKind::Ref { .. }, Val::A { len, .. } | Val::Arr { len, .. }) = (e, v)
        && !fixed_len(len)
    {
        let x = format!("(select {len} k)");
        out.push(format!(
            "(forall ((k Int)) (! (and (<= 0 {x}) (<= {x} {})) :pattern ({x})))",
            len_max(&elem_ty(e))
        ));
    }
}

/// Whether an element template's length is a fixed constant (a numeral),
/// not the array of an inner `Vec`'s lengths.
fn fixed_len(len: &str) -> bool {
    len.parse::<u64>().is_ok()
}

fn select_len(len: &str, i: &str) -> String {
    if fixed_len(len) {
        len.to_string()
    } else {
        format!("(select {len} {i})")
    }
}

fn store_len(len: &str, i: &str, x: &str) -> String {
    if fixed_len(len) {
        len.to_string()
    } else {
        format!("(store {len} {i} {x})")
    }
}

/// The element arrays of a slice of structs with element `i` set to `v`.
fn store_val(elem: &Val, i: &str, v: &Val) -> Val {
    match (elem, v) {
        (Val::S(a), Val::S(x)) => Val::S(format!("(store {a} {i} {x})")),
        (Val::Struct(fs), Val::Struct(xs)) => {
            Val::Struct(fs.iter().zip(xs).map(|(f, x)| store_val(f, i, x)).collect())
        }
        (Val::A { elems, len }, Val::A { elems: x, len: xl }) => Val::A {
            elems: format!("(store {elems} {i} {x})"),
            len: store_len(len, i, xl),
        },
        (Val::Arr { elem, len }, Val::Arr { elem: x, len: xl }) => Val::Arr {
            elem: Box::new(store_val(elem, i, x)),
            len: store_len(len, i, xl),
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
    if dt_struct(t, tb) {
        return Some(Val::S(arb(t)));
    }
    if let Some(fs) = flat_fields(t, tb) {
        let mut out = Vec::new();
        for (_, ft) in &fs {
            out.push(default_in(ft, tb, building)?);
        }
        return Some(Val::Struct(out));
    }
    if let Some(vs) = variants(t, tb) {
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
        TypeKind::Array(e, n) => match const_elems(&e.kind, &elem_default(&e.kind, tb), tb) {
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
        // An empty `Vec` (backlog 2O-03b: an inner one's default).
        _ if is_array(t) => {
            let e = elem_ty(t);
            match const_elems(&e, &elem_default(&e, tb), tb) {
                Val::S(elems) => Val::A {
                    elems,
                    len: "0".into(),
                },
                elem => Val::Arr {
                    elem: Box::new(elem),
                    len: "0".into(),
                },
            }
        }
        _ => Val::S(default_of(t).to_string()),
    })
}

/// The value an array of `e` holds at every element by default: the
/// default value, or for a recursive struct (whose default holds such an
/// array) an arbitrary one, `arb` (backlog 2O-03d).
fn elem_default(e: &TypeKind, tb: &Tables) -> Val {
    if dt_struct(e, tb) {
        Val::S(arb(e))
    } else {
        default_val(e, tb)
    }
}

/// The arbitrary constant of a recursive struct's datatype.
fn arb(t: &TypeKind) -> String {
    sym(&format!("{}::'arb", crate::check::ty_str(value_ty(t))))
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
    flatten_elem_of(v, e, tb, &mut terms);
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
        Val::A { elems, len } => {
            out.push(elems.clone());
            if !fixed_len(len) {
                out.push(len.clone());
            }
        }
        Val::Arr { elem, len } => {
            flatten_elem(elem, out);
            if !fixed_len(len) {
                out.push(len.clone());
            }
        }
        Val::Struct(fs) => fs.iter().for_each(|f| flatten_elem(f, out)),
        Val::Unit => {}
    }
}

/// The leaf terms of one element value `v` of type `e`, in
/// `elem_leaves` order (an inner `Vec`'s length is a leaf even when it is
/// a numeral).
fn flatten_elem_of(v: &Val, e: &TypeKind, tb: &Tables, out: &mut Vec<String>) {
    if let Some(fs) = flat_fields(e, tb) {
        if let Val::Struct(xs) = v {
            for ((_, ft), x) in fs.iter().zip(xs) {
                flatten_elem_of(x, ft, tb, out);
            }
        }
        return;
    }
    match (e, v) {
        (TypeKind::Array(inner, _), Val::A { elems, .. }) if !agg_elem(&inner.kind, tb) => {
            out.push(elems.clone())
        }
        (TypeKind::Array(inner, _), Val::Arr { elem, .. }) => {
            flatten_elem_of(elem, &inner.kind, tb, out)
        }
        (TypeKind::Ref { inner, .. }, Val::A { elems, len }) => {
            let _ = inner;
            out.push(elems.clone());
            out.push(len.clone());
        }
        (TypeKind::Ref { inner, .. }, Val::Arr { elem, len }) => {
            if let TypeKind::Slice(el) = &inner.kind {
                flatten_elem_of(elem, &el.kind, tb, out);
            }
            out.push(len.clone());
        }
        (_, Val::Unit) => {}
        (_, v) => flatten_elem(v, out),
    }
}

/// Whether an array of `e` holds its elements as one array per leaf
/// (ADR-0027 §2): structs and fixed-size arrays (backlog 2V-04b).
fn agg_elem(e: &TypeKind, tb: &Tables) -> bool {
    flat_fields(e, tb).is_some() || is_array(e)
}

/// The leaves of an element of type `e`, each with the sort of its value
/// for one element: a struct's fields in order, an inner array's leaves
/// one array level deeper (its fixed length is a constant).
fn elem_leaves(e: &TypeKind, tb: &Tables, suffix: &str, out: &mut Vec<(String, String)>) {
    if let Some(fs) = flat_fields(e, tb) {
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
        // An inner `Vec` (backlog 2O-03b): its leaves one level deeper and
        // its length.
        TypeKind::Ref { inner, .. } if matches!(inner.kind, TypeKind::Slice(_)) => {
            let el = elem_ty(e);
            let mut l = Vec::new();
            elem_leaves(&el, tb, &format!("{suffix}.e"), &mut l);
            out.extend(l.into_iter().map(|(n, s)| (n, format!("(Array Int {s})"))));
            out.push((format!("{suffix}.len"), "Int".into()));
        }
        t => out.push((suffix.to_string(), sort_of(t))),
    }
}

/// The element template of type `e` whose leaf arrays are `terms` (the
/// inverse of `flatten_elem`).
fn elem_template(e: &TypeKind, tb: &Tables, terms: &mut impl Iterator<Item = String>) -> Val {
    if let Some(fs) = flat_fields(e, tb) {
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
        TypeKind::Ref { inner, .. } if matches!(inner.kind, TypeKind::Slice(_)) => {
            let el = elem_ty(e);
            if agg_elem(&el, tb) {
                let elem = Box::new(elem_template(&el, tb, terms));
                Val::Arr {
                    elem,
                    len: terms.next().expect("length leaf"),
                }
            } else {
                let elems = terms.next().expect("array leaf");
                Val::A {
                    elems,
                    len: terms.next().expect("length leaf"),
                }
            }
        }
        _ => Val::S(terms.next().expect("scalar leaf")),
    }
}

/// One SMT leaf of a value: a scalar, or an array's elements or length
/// (`Some(n)` for a fixed-size array).
enum Leaf {
    Scalar(TypeKind),
    /// A recursive struct (`dt_struct`): a value of its sort.
    Dt(TypeKind),
    Elems(TypeKind),
    /// One leaf array of an array of aggregates: the sort of one element's leaf.
    Leaves(String),
    Len(TypeKind, Option<u64>),
}

impl Leaf {
    fn sort(&self) -> String {
        match self {
            Leaf::Scalar(t) | Leaf::Dt(t) => sort_of(t),
            Leaf::Elems(e) => format!("(Array Int {})", sort_of(e)),
            Leaf::Leaves(s) => format!("(Array Int {s})"),
            Leaf::Len(..) => "Int".into(),
        }
    }

    /// What every value of the leaf satisfies (ADR-0017 §3).
    fn fact(&self, x: &str) -> Option<String> {
        match self {
            Leaf::Scalar(t) => range(t, x),
            Leaf::Dt(_) => None,
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

/// Whether `t` (through a reference) is a struct that contains itself
/// through structs alone: a datatype of one constructor, not a tuple of
/// leaves (ADR-0047 §3, backlog 2O-03d).
fn dt_struct(t: &TypeKind, tb: &Tables) -> bool {
    struct_fields(t, tb).is_some() && crate::check::struct_cycle(t, &tb.adts)
}

/// The fields of a struct the encoder holds as a tuple of leaves: every
/// struct but a recursive one (`dt_struct`).
fn flat_fields(t: &TypeKind, tb: &Tables) -> Option<Vec<(String, TypeKind)>> {
    if dt_struct(t, tb) {
        return None;
    }
    struct_fields(t, tb)
}

/// The constructor name of a recursive struct's datatype: the struct's.
fn dt_ctor_name(t: &TypeKind) -> String {
    match value_ty(t) {
        TypeKind::Named { path, .. } => path.last().cloned().unwrap_or_default(),
        _ => String::new(),
    }
}

/// The variants of the datatype of `t`: an enum's, or the one constructor
/// of a recursive struct with its fields as the payload.
fn variants(t: &TypeKind, tb: &Tables) -> Option<Vec<(String, Vec<TypeKind>)>> {
    if dt_struct(t, tb) {
        let fs = struct_fields(t, tb)?;
        return Some(vec![(
            dt_ctor_name(t),
            fs.into_iter().map(|(_, ft)| ft).collect(),
        )]);
    }
    enum_variants(t, tb)
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
        if let Some(fs) = flat_fields(t, tb) {
            return fs.iter().any(|(_, ft)| go(ft, tb, seen));
        }
        variants(t, tb).is_some_and(|vs| {
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
    /// A recursive struct's arbitrary constant (`arb`).
    arb: Option<String>,
    /// A recursive struct's selectors with their leaves.
    leaves: Vec<(String, Leaf)>,
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
        if let Some(fs) = flat_fields(t, tb) {
            for (_, ft) in &fs {
                visit(ft, tb, order, dts);
            }
            return;
        }
        let Some(vs) = variants(t, tb) else {
            return;
        };
        let ts = crate::check::ty_str(t);
        if dts.contains_key(&ts) {
            return;
        }
        dts.insert(
            ts.clone(),
            Dt {
                ctors: Vec::new(),
                arb: dt_struct(t, tb).then(|| arb(t)),
                leaves: Vec::new(),
            },
        );
        let mut ctors = Vec::new();
        let mut kept = Vec::new();
        let is_struct = dt_struct(t, tb);
        for (v, ps) in &vs {
            let mut fields = Vec::new();
            for (k, pt) in ps.iter().enumerate() {
                visit(pt, tb, order, dts);
                for (name, leaf) in payload_leaves(t, v, k, pt, tb) {
                    fields.push((sym(&name), leaf.sort()));
                    if is_struct {
                        kept.push((sym(&name), leaf));
                    }
                }
            }
            ctors.push((ctor(t, v), fields));
        }
        let d = dts.get_mut(&ts).expect("inserted");
        d.ctors = ctors;
        d.leaves = kept;
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
    let (all, dts) = enum_dts(tb);
    // Recursive structs (backlog 2O-03d) are handles, `Int` under the
    // struct's sort name, declared before every datatype, with their
    // selector functions after them. Not datatypes: Z3 4.15.4 does not end
    // satisfiable queries over a datatype nested in an array whose every
    // constructor holds it, and not uninterpreted sorts: with a quantified
    // spec fn its model search ran to the resource limit where `Int`
    // handles answer at once (`dev-docs/measurements/z3-nested-datatypes`).
    let structs: Vec<String> = all
        .iter()
        .filter(|ts| dts[*ts].arb.is_some())
        .cloned()
        .collect();
    let order: Vec<String> = all
        .iter()
        .filter(|ts| dts[*ts].arb.is_none())
        .cloned()
        .collect();
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
    let mut out: Vec<(String, String, String)> = structs
        .iter()
        .map(|ts| {
            (
                format!("|{ts}::"),
                sym(ts),
                format!("(define-sort {} () Int)", sym(ts)),
            )
        })
        .collect();
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
    for ts in &structs {
        let d = &dts[ts];
        let mut decl = String::new();
        for (_, fs) in &d.ctors {
            for (sel, sort) in fs {
                decl.push_str(&format!("(declare-fun {sel} ({}) {sort})\n", sym(ts)));
            }
        }
        decl.push_str(&format!(
            "(declare-const {} {})",
            d.arb.as_ref().expect("a recursive struct"),
            sym(ts)
        ));
        out.push((format!("|{ts}::"), sym(ts), decl));
    }
    out
}

/// The SMT functions of spec fn `path`: one, or one per leaf of a struct
/// result (`|geo::mid.x|`), with their result sorts.
fn spec_funs(path: &str, tb: &Tables) -> Vec<(String, String)> {
    let ret = &tb.fns[path].ret;
    if flat_fields(ret, tb).is_none() {
        return vec![(sym(path), sort_of(ret))];
    }
    let mut ls = Vec::new();
    leaves(ret, tb, path, &mut ls);
    ls.iter().map(|(n, l)| (sym(n), l.sort())).collect()
}

/// What marks spec fn `path` in a query text: its symbol, or the prefix of
/// its leaf symbols.
fn spec_key(path: &str, tb: &Tables) -> String {
    if flat_fields(&tb.fns[path].ret, tb).is_some() {
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
    if let Some(fs) = flat_fields(t, tb) {
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
        _ if dt_struct(t, tb) => out.push((base.to_string(), Leaf::Dt(t.clone()))),
        _ => out.push((base.to_string(), Leaf::Scalar(t.clone()))),
    }
}

/// The value of type `t` whose leaves are `terms` (the inverse of `flatten`).
fn build(t: &TypeKind, tb: &Tables, terms: &mut impl Iterator<Item = String>) -> Val {
    if let Some(fs) = flat_fields(t, tb) {
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
    /// Field `k` of a recursive struct of type `t` (`dt_struct`).
    DtField(usize, TypeKind),
    Index(String),
}

/// The fields of the recursive struct value `x` of type `t`, one value per
/// field (its selectors applied to `x`).
fn dt_fields(t: &TypeKind, x: &str, tb: &Tables) -> Vec<Val> {
    let name = dt_ctor_name(t);
    let fs = struct_fields(t, tb).expect("a struct");
    fs.iter()
        .enumerate()
        .map(|(k, (_, ft))| {
            let terms: Vec<String> = payload_leaves(t, &name, k, ft, tb)
                .iter()
                .map(|(n, _)| format!("({} {x})", sym(n)))
                .collect();
            build(ft, tb, &mut terms.into_iter())
        })
        .collect()
}

/// `v` with the place at `steps` replaced by `new`; `mk` builds a
/// recursive struct value from its fields (`Enc::mk_struct`).
fn set_at(
    v: Val,
    steps: &[Step],
    new: Val,
    tb: &Tables,
    mk: &mut dyn FnMut(&TypeKind, Vec<Val>) -> Val,
) -> Val {
    match (steps.split_first(), v) {
        (None, _) => new,
        (Some((Step::Field(k), rest)), Val::Struct(mut fs)) => {
            let f = std::mem::replace(&mut fs[*k], Val::Unit);
            fs[*k] = set_at(f, rest, new, tb, mk);
            Val::Struct(fs)
        }
        (Some((Step::DtField(k, t), rest)), Val::S(x)) => {
            let mut fs = dt_fields(t, &x, tb);
            let f = std::mem::replace(&mut fs[*k], Val::Unit);
            fs[*k] = set_at(f, rest, new, tb, mk);
            mk(t, fs)
        }
        (Some((Step::Index(i), rest)), Val::Arr { elem, len }) => {
            let cur = select_val(&elem, i);
            let new = set_at(cur, rest, new, tb, mk);
            Val::Arr {
                elem: Box::new(store_val(&elem, i, &new)),
                len,
            }
        }
        // A scalar element, or a recursive struct one (backlog 2O-03d).
        (Some((Step::Index(i), rest)), Val::A { elems, len }) => {
            let cur = Val::S(format!("(select {elems} {i})"));
            let new = set_at(cur, rest, new, tb, mk);
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
    /// A sub-proof (`assert(e) by (bit_vector)`, ADR-0048 §4): its own
    /// query in place of the assumptions and goal, or why `e` has none
    /// (and the spec fns it would unfold that `#[uses]` does not reveal).
    sub: Option<Result<crate::subproof::Query, (String, Vec<String>)>>,
    /// A value the counterexample shows besides the inputs: the one an
    /// invariant obligation is about (E0823, ADR-0060 §4).
    shown: Option<(String, Val, TypeKind)>,
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
    /// The constructors of recursive structs (`dt_struct`): every program
    /// value of such a type is one.
    struct_ctors: BTreeSet<String>,
    /// The leaf of every recursive struct selector (`Prog::struct_facts`).
    struct_sels: BTreeMap<String, Leaf>,
    /// Whether a type has a validity (`needs_valid`), by its text.
    valid_memo: RefCell<BTreeMap<String, bool>>,
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
    /// The current path has returned or left a loop (ADR-0049 §3): what
    /// follows on it does not run, and a join takes the other paths.
    dead: bool,
    /// The loops around the code being encoded, innermost last.
    loops: Vec<VLoop>,
    /// Encoding ghost code (ADR-0062 §2): a `proof fn` body or a `proof`
    /// block. Its expressions are specifications (no overflow or index
    /// obligations); its effects are the facts it proves.
    ghost: bool,
    /// Type invariants (ADR-0060 §2, `invariants.rs`): the validity
    /// symbols declared, the nesting depth of element quantifiers (their
    /// bound index: the same value gets the same text), and the definitions
    /// of the recursive types' validity functions (ground instances).
    valid_decl: BTreeSet<String>,
    valid_depth: u32,
    valid_defs: Vec<AxiomParts>,
}

/// A loop a `break` or `continue` may leave (ADR-0049 §3).
struct VLoop {
    label: Option<String>,
    /// The path condition's length at the head of the iteration: what was
    /// assumed in the iteration up to a `break` follows it.
    start: usize,
    invariants: Vec<Expr>,
    decreases: Option<Expr>,
    /// The measure at the head of the iteration.
    measure: Option<String>,
    /// Each `break`: the iteration's path condition there, the state, and
    /// the value it carries.
    breaks: Vec<(Vec<String>, BTreeMap<String, Val>, Val)>,
    /// A range `for`'s variable and bounds: the end of an iteration (the
    /// body's or a `continue`) steps it (ADR-0049 §2).
    step: Option<ForStep>,
    /// The loop, and the locals it havocs that have a validity: valid
    /// again when an iteration goes back to the head (ADR-0060 §2).
    span: Span,
    valid_vars: Vec<(String, TypeKind)>,
}

/// A range `for`: `var` from `lo` to `hi` (excluded, or included), the
/// bounds evaluated once on entry.
#[derive(Clone)]
struct ForStep {
    var: String,
    ty: TypeKind,
    lo: String,
    hi: String,
    inclusive: bool,
    span: Span,
}

/// A range `for`'s head invariant at `i`: `lo <= i && (i <= hi || i ==
/// lo)` (the second part also holds on entry to an empty range).
fn for_bounds(st: &ForStep, i: &str) -> String {
    format!(
        "(and {} (or {} (= {i} {})))",
        ord_term(BinOp::Le, &st.ty, &st.lo, i),
        ord_term(BinOp::Le, &st.ty, i, &st.hi),
        st.lo
    )
}

/// `(select a i)` as (`a`, `i`).
fn select_app(t: &str) -> Option<(&str, &str)> {
    let inner = t.strip_prefix("(select ")?.strip_suffix(')')?;
    let (mut depth, mut quoted) = (0i32, false);
    for (k, c) in inner.char_indices() {
        match c {
            '|' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            ' ' if !quoted && depth == 0 => return Some((&inner[..k], &inner[k + 1..])),
            _ => {}
        }
    }
    None
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
        TypeKind::F64 => "Float64".into(),
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
    t.int().map(|i| range_int(i, x))
}

fn range_int(i: IntTy, x: &str) -> String {
    format!(
        "(and (<= {} {x}) (<= {x} {}))",
        num(i.min_value()),
        num(i.max_value())
    )
}

fn len_max(elem: &TypeKind) -> &'static str {
    // A data type may be one byte (a fieldless enum, a struct of a bool):
    // only `isize::MAX` is known without its layout.
    match elem {
        TypeKind::Bool | TypeKind::Named { .. } | TypeKind::Param(_) => LEN_MAX_BYTE,
        TypeKind::Int(i) => match i.bits() {
            8 => LEN_MAX_BYTE,
            16 => LEN_MAX_HALF,
            32 => LEN_MAX_QUARTER,
            _ => LEN_MAX_WORD,
        },
        _ => LEN_MAX_WORD,
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

/// `x` reduced into the range of `t` (two's complement).
fn wrap(t: &TypeKind, x: &str) -> String {
    wrap_int(t.int().expect("integer type"), x)
}

fn wrap_int(i: IntTy, x: &str) -> String {
    let size = num(1i128 << i.bits());
    if i.signed() {
        let half = num(1i128 << (i.bits() - 1));
        format!("(- (mod (+ {x} {half}) {size}) {half})")
    } else {
        format!("(mod {x} {size})")
    }
}

/// The value of a numeral term (`5`, `(- 5)`).
fn numeral(t: &str) -> Option<i128> {
    if !is_numeral(t) {
        return None;
    }
    match t.strip_prefix("(- ").and_then(|x| x.strip_suffix(')')) {
        Some(x) => x.parse::<i128>().ok().map(|v| -v),
        None => t.parse().ok(),
    }
}

/// `x` as a bitvector of `i`'s width: its value modulo `2^w`, so a negative
/// value is its two's complement (ADR-0048 §4).
fn to_bv(i: IntTy, x: &str) -> String {
    format!("((_ int2bv {}) {x})", i.bits())
}

/// The value of a bitvector `v` of `i`'s width as an integer of `i`:
/// unsigned, or two's complement (flipping the sign bit adds `2^(w-1)`
/// modulo `2^w`, so one `bv2nat` suffices).
fn from_bv(i: IntTy, v: &str) -> String {
    if i.signed() {
        let half = 1i128 << (i.bits() - 1);
        format!("(- (bv2nat (bvxor {v} (_ bv{half} {}))) {half})", i.bits())
    } else {
        format!("(bv2nat {v})")
    }
}

/// `!a` on integer type `i`: every bit flipped, which is linear
/// (`-a - 1`, or `MAX - a` unsigned).
pub fn not_term(i: IntTy, a: &str) -> String {
    if i.signed() {
        format!("(- (- {a}) 1)")
    } else {
        format!("(- {} {a})", num(i.max_value()))
    }
}

/// A bit operation `a op b` on integer type `i` (ADR-0048 §4): its term and
/// the facts about the term that hold when the operands are in `i`'s range.
/// Integers stay `Int`; only the term goes through a bitvector of the
/// operand width (`int2bv`, the operation, back with `bv2nat`). Forms that
/// are linear stay linear (`linear_bit`). The facts (checked by
/// `tests/lemmas.rs`) bound the result by the operands, so range obligations
/// need no bitvector reasoning. A shift amount outside `0..w` is the
/// caller's obligation (E0803). Code names its operands' bitvectors instead
/// (`Enc::operand_bv`): the solver finds models far sooner that way.
pub fn bit_op(op: BinOp, i: IntTy, a: &str, b: &str) -> (String, Vec<String>) {
    if let Some(t) = linear_bit(op, i, a, b) {
        return (t, Vec::new());
    }
    let r = bv_term(op, i, &to_bv(i, a), &to_bv(i, b));
    let facts = bit_facts(op, i, a, b, &r);
    (r, facts)
}

/// The linear form of `a op b` on `i`, where there is one: a constant shift
/// amount is a multiplication (wrapped into the range) or a floor division,
/// and `a & m` with `m = 2^k - 1` is `a mod 2^k`.
pub fn linear_bit(op: BinOp, i: IntTy, a: &str, b: &str) -> Option<String> {
    let w = i.bits() as i128;
    // `a & (2^k - 1)` keeps the low `k` bits.
    let mask = |m: Option<i128>| {
        m.filter(|m| *m >= 0 && *m <= i.max_value() && (m + 1) & m == 0)
            .map(|m| (m + 1).trailing_zeros())
    };
    match op {
        BinOp::BitAnd => match (mask(numeral(b)), mask(numeral(a))) {
            (Some(k), _) => Some(format!("(mod {a} {})", num(1i128 << k))),
            (_, Some(k)) => Some(format!("(mod {b} {})", num(1i128 << k))),
            _ => None,
        },
        BinOp::Shl | BinOp::Shr if numeral(b) == Some(0) => Some(a.to_string()),
        BinOp::Shl => numeral(b)
            .filter(|k| 0 < *k && *k < w)
            .map(|k| wrap_int(i, &format!("(* {a} {})", num(1i128 << k)))),
        // Floor division: `>>` rounds toward negative infinity on signed
        // values (arithmetic shift), as SMT `div` does.
        BinOp::Shr => numeral(b)
            .filter(|k| 0 < *k && *k < w)
            .map(|k| format!("(div {a} {})", num(1i128 << k))),
        _ => None,
    }
}

/// The term of `a op b` on `i` over bitvectors `xa`, `xb` of `i`'s width.
pub fn bv_term(op: BinOp, i: IntTy, xa: &str, xb: &str) -> String {
    let f = match op {
        BinOp::BitAnd => "bvand",
        BinOp::BitOr => "bvor",
        BinOp::BitXor => "bvxor",
        BinOp::Shl => "bvshl",
        BinOp::Shr if i.signed() => "bvashr",
        BinOp::Shr => "bvlshr",
        _ => unreachable!("not a bit operation: {op:?}"),
    };
    from_bv(i, &format!("({f} {xa} {xb})"))
}

/// The unsigned value of the bitvector that stands for `a`, a value of
/// `i` in range: `a` itself, or its two's complement below zero. Code
/// declares a bitvector with this value for each operand (`Enc::operand_bv`).
pub fn bv_value(i: IntTy, a: &str) -> String {
    if i.signed() {
        format!("(ite (< {a} 0) (+ {a} {}) {a})", num(1i128 << i.bits()))
    } else {
        a.to_string()
    }
}

/// `bit_facts_of`, each under the range of the operands (a shift amount's
/// range is its own obligation).
fn bit_facts(op: BinOp, i: IntTy, a: &str, b: &str, r: &str) -> Vec<String> {
    let ra = range_int(i, a);
    let pre = if op.is_shift() {
        ra
    } else {
        format!("(and {ra} {})", range_int(i, b))
    };
    bit_facts_of(op, i, a, b, r)
        .into_iter()
        .map(|f| format!("(=> {pre} {f})"))
        .collect()
}

/// What a bitvector term `r = a op b` is known to satisfy when `a` (and
/// `b`, but for a shift amount) are in `i`'s range: the range of `i`, and
/// its order against the operands (clearing bits makes a value smaller,
/// setting bits larger, in two's complement as well).
fn bit_facts_of(op: BinOp, i: IntTy, a: &str, b: &str, r: &str) -> Vec<String> {
    let mut fs = vec![range_int(i, r)];
    let s = i.signed();
    match op {
        BinOp::BitAnd if s => {
            fs.push(format!("(=> (<= 0 {a}) (and (<= 0 {r}) (<= {r} {a})))"));
            fs.push(format!("(=> (<= 0 {b}) (and (<= 0 {r}) (<= {r} {b})))"));
            fs.push(format!(
                "(=> (and (< {a} 0) (< {b} 0)) (and (< {r} 0) (<= {r} {a}) (<= {r} {b})))"
            ));
        }
        BinOp::BitAnd => {
            fs.push(format!("(<= {r} {a})"));
            fs.push(format!("(<= {r} {b})"));
        }
        BinOp::BitOr if s => {
            fs.push(format!(
                "(=> (and (<= 0 {a}) (<= 0 {b})) (and (<= {a} {r}) (<= {b} {r})))"
            ));
            fs.push(format!("(=> (< {a} 0) (and (<= {a} {r}) (< {r} 0)))"));
            fs.push(format!("(=> (< {b} 0) (and (<= {b} {r}) (< {r} 0)))"));
        }
        BinOp::BitOr => {
            fs.push(format!("(<= {a} {r})"));
            fs.push(format!("(<= {b} {r})"));
        }
        BinOp::BitXor if s => {
            fs.push(format!("(= (< {r} 0) (not (= (< {a} 0) (< {b} 0))))"));
        }
        BinOp::Shr if s => {
            fs.push(format!("(=> (<= 0 {a}) (and (<= 0 {r}) (<= {r} {a})))"));
            fs.push(format!("(=> (< {a} 0) (and (<= {a} {r}) (< {r} 0)))"));
        }
        BinOp::Shr => fs.push(format!("(<= {r} {a})")),
        _ => {}
    }
    fs
}

/// The operations of `f64` (ADR-0061 §5) as the encoder names them: in an
/// ordinary query each is an uninterpreted function or predicate over
/// `Float64` (`f64_decls`); a `by (bit_vector)` sub-proof, and the search
/// for a counterexample (`f64_decls` interpreted), give them their IEEE 754
/// meaning with rounding to nearest even. `>` and `>=` are `f64.lt` and
/// `f64.le` with the operands swapped (IEEE: `a > b` iff `b < a`, NaN
/// included). `f64.from` takes an integer (`f64::from`, ADR-0061 §4).
pub const F64_FUNS: [(&str, &str, &str, &str); 14] = [
    (
        "f64.add",
        "(Float64 Float64)",
        "Float64",
        "(fp.add RNE a b)",
    ),
    (
        "f64.sub",
        "(Float64 Float64)",
        "Float64",
        "(fp.sub RNE a b)",
    ),
    (
        "f64.mul",
        "(Float64 Float64)",
        "Float64",
        "(fp.mul RNE a b)",
    ),
    (
        "f64.div",
        "(Float64 Float64)",
        "Float64",
        "(fp.div RNE a b)",
    ),
    ("f64.neg", "(Float64)", "Float64", "(fp.neg a)"),
    ("f64.sqrt", "(Float64)", "Float64", "(fp.sqrt RNE a)"),
    ("f64.abs", "(Float64)", "Float64", "(fp.abs a)"),
    ("f64.min", "(Float64 Float64)", "Float64", "(fp.min a b)"),
    ("f64.max", "(Float64 Float64)", "Float64", "(fp.max a b)"),
    (
        "f64.from",
        "(Int)",
        "Float64",
        "((_ to_fp 11 53) RNE (to_real a))",
    ),
    ("f64.lt", "(Float64 Float64)", "Bool", "(fp.lt a b)"),
    ("f64.le", "(Float64 Float64)", "Bool", "(fp.leq a b)"),
    ("f64.eq", "(Float64 Float64)", "Bool", "(fp.eq a b)"),
    ("f64.is_nan", "(Float64)", "Bool", "(fp.isNaN a)"),
];

/// `f64.is_finite` is defined from `f64.is_nan`'s kin in both readings: a
/// separate symbol keeps the opaque query free of `fp.*` terms.
const F64_IS_FINITE: (&str, &str) = (
    "(declare-fun f64.is_finite (Float64) Bool)",
    "(define-fun f64.is_finite ((a Float64)) Bool (not (or (fp.isInfinite a) (fp.isNaN a))))",
);

/// The declarations of the `f64` operations `text` applies, in a fixed
/// order: uninterpreted (an ordinary query, ADR-0061 §5), or with their
/// IEEE definitions (`interpreted`, the counterexample search). Empty for
/// a query without `f64` operations, so its text is unchanged.
pub fn f64_decls(text: &str, interpreted: bool) -> String {
    if !text.contains("(f64.") {
        return String::new();
    }
    let mut out = String::new();
    for (name, args, ret, def) in F64_FUNS {
        if !text.contains(&format!("({name} ")) {
            continue;
        }
        if interpreted {
            let params: Vec<String> = args
                .trim_matches(|c| c == '(' || c == ')')
                .split(' ')
                .zip(["a", "b"])
                .map(|(s, p)| format!("({p} {s})"))
                .collect();
            out.push_str(&format!(
                "(define-fun {name} ({}) {ret} {def})\n",
                params.join(" ")
            ));
        } else {
            out.push_str(&format!("(declare-fun {name} {args} {ret})\n"));
        }
    }
    if text.contains("(f64.is_finite ") {
        out.push_str(if interpreted {
            F64_IS_FINITE.1
        } else {
            F64_IS_FINITE.0
        });
        out.push('\n');
    }
    out
}

/// An `f64` value as an SMT term (ADR-0061 §5): its exact bit pattern
/// (sign, exponent, significand); every NaN is SMT-LIB's one NaN.
pub fn f64_lit(bits: u64) -> String {
    if f64::from_bits(bits).is_nan() {
        return "(_ NaN 11 53)".into();
    }
    format!(
        "(fp #b{} #b{:011b} #x{:013x})",
        bits >> 63,
        (bits >> 52) & 0x7ff,
        bits & ((1u64 << 52) - 1)
    )
}

/// `a op b` on `f64` (ADR-0061 §3, §5): arithmetic and comparisons as the
/// encoder's `f64.*` symbols. `None` for an operation `f64` does not have.
fn f64_binary(op: BinOp, a: &str, b: &str) -> Option<String> {
    Some(match op {
        BinOp::Add => format!("(f64.add {a} {b})"),
        BinOp::Sub => format!("(f64.sub {a} {b})"),
        BinOp::Mul => format!("(f64.mul {a} {b})"),
        BinOp::Div => format!("(f64.div {a} {b})"),
        BinOp::Lt => format!("(f64.lt {a} {b})"),
        BinOp::Le => format!("(f64.le {a} {b})"),
        BinOp::Gt => format!("(f64.lt {b} {a})"),
        BinOp::Ge => format!("(f64.le {b} {a})"),
        BinOp::Eq => format!("(f64.eq {a} {b})"),
        BinOp::Ne => format!("(not (f64.eq {a} {b}))"),
        _ => return None,
    })
}

/// `-x` on `f64`: a literal's negation is the literal of the negated value
/// (`-0.0` is negative zero, as in Rust).
fn f64_neg(x: &Expr, v: &str) -> String {
    match x.kind {
        ExprKind::Float(b) => f64_lit(b ^ (1u64 << 63)),
        _ => format!("(f64.neg {v})"),
    }
}

/// A method of `f64` (ADR-0061 §3) as its symbol applied to `a` (and `b`).
fn f64_method(m: &str, a: &str, b: Option<&str>) -> String {
    match b {
        Some(b) => format!("(f64.{m} {a} {b})"),
        None => format!("(f64.{m} {a})"),
    }
}

/// `2^53`: the integers up to it in absolute value are exactly `f64`s
/// (ADR-0061 §4).
const F64_EXACT: &str = "9007199254740992";

/// `&`, `|`, `^` on `bool`: both sides evaluated (Rust).
fn bool_bit(op: BinOp, a: &str, b: &str) -> String {
    match op {
        BinOp::BitAnd => format!("(and {a} {b})"),
        BinOp::BitOr => format!("(or {a} {b})"),
        BinOp::BitXor => format!("(xor {a} {b})"),
        _ => unreachable!("only `&`, `|` and `^` take `bool`"),
    }
}

/// A bit method `a.m(n)` of integer type `i` (ADR-0048 §2, backlog 2N-03)
/// in a specification: its term and the facts about it when `a` is in
/// range (code assumes them, as `bit_op`'s). Rust's semantics: a rotation or
/// wrapping shift moves by `n mod w` (`n` is a `u32`); the counts are `u32`
/// values in `0..=w`. A constant amount is linear (ADR-0048 §4): a wrapping
/// shift has `<<`'s or `>>`'s form, a rotation is the sum of its two parts.
/// Anything else is one bit-vector term behind `bv2nat`, through a named
/// function (`subproof::helper_def`), so the abstracted query sees one
/// constant with its facts and code and specification terms stay
/// comparable (`bit_congruences`). Checked by `tests/lemmas.rs`.
pub fn bit_method(m: &str, i: IntTy, a: &str, n: Option<&str>) -> (String, Vec<String>) {
    let xn = n.map(|n| to_bv(i, n));
    let t = method_term(m, i, a, &to_bv(i, a), n, xn.as_deref());
    let facts = method_facts(m, i, a, &t);
    (t, facts)
}

/// `bit_method`'s term over `xa`, the bit-vector of `a`, and `xn`, the
/// amount modulo `2^w` as a bit-vector of `a`'s width `w` (which `w`
/// divides, so the amount modulo `w` is its low bits).
fn method_term(m: &str, i: IntTy, a: &str, xa: &str, n: Option<&str>, xn: Option<&str>) -> String {
    let w = i.bits();
    let wi = w as i128;
    if let Some(k) = n.and_then(numeral) {
        let k = k.rem_euclid(wi);
        let left = if m == "rotate_right" {
            (wi - k) % wi
        } else {
            k
        };
        match m {
            "wrapping_shl" => {
                return linear_bit(BinOp::Shl, i, a, &k.to_string()).expect("a constant shift");
            }
            "wrapping_shr" => {
                return linear_bit(BinOp::Shr, i, a, &k.to_string()).expect("a constant shift");
            }
            "rotate_left" | "rotate_right" if left == 0 => return a.to_string(),
            "rotate_left" | "rotate_right" => {
                // The low `w - left` bits move up, the high `left` bits down,
                // of the unsigned image of `a`.
                let lo = num(1i128 << (wi - left));
                let u = if i.signed() {
                    format!("(mod {a} {})", num(1i128 << w))
                } else {
                    a.to_string()
                };
                let r = format!(
                    "(+ (* (mod {a} {lo}) {}) (div {u} {lo}))",
                    num(1i128 << left)
                );
                return if i.signed() { wrap_int(i, &r) } else { r };
            }
            _ => {}
        }
    }
    let h = |op: &str| format!("bmb_{op}{w}");
    match m {
        "wrapping_shl" | "wrapping_shr" => {
            let s = format!("(bvand {} (_ bv{} {w}))", xn.expect("an amount"), w - 1);
            let op = if m == "wrapping_shl" {
                BinOp::Shl
            } else {
                BinOp::Shr
            };
            bv_term(op, i, xa, &s)
        }
        "rotate_left" | "rotate_right" => {
            let f = if m == "rotate_left" { "rotl" } else { "rotr" };
            from_bv(i, &format!("({} {xa} {})", h(f), xn.expect("an amount")))
        }
        "swap_bytes" if w == 8 => a.to_string(),
        "swap_bytes" => from_bv(i, &format!("({} {xa})", h("bswap"))),
        "count_ones" => format!("(bv2nat ({} {xa}))", h("ctpop")),
        "count_zeros" => format!("(- {w} (bv2nat ({} {xa})))", h("ctpop")),
        "leading_zeros" => format!("(bv2nat ({} {xa}))", h("clz")),
        "trailing_zeros" => format!("(bv2nat ({} {xa}))", h("ctz")),
        _ => unreachable!("bit method {m}"),
    }
}

/// What a bit method's bit-vector term `r` satisfies when `a` is in `i`'s
/// range: a value of `i`, or a count in `0..=w` that is extreme exactly at
/// zero. A linear term needs none.
fn method_facts(m: &str, i: IntTy, a: &str, r: &str) -> Vec<String> {
    if !r.contains("(bv2nat ") {
        return Vec::new();
    }
    let w = i.bits();
    let fs = match m {
        "count_ones" => vec![
            format!("(and (<= 0 {r}) (<= {r} {w}))"),
            format!("(= (= {r} 0) (= {a} 0))"),
        ],
        "count_zeros" | "leading_zeros" | "trailing_zeros" => vec![
            format!("(and (<= 0 {r}) (<= {r} {w}))"),
            format!("(= (= {r} {w}) (= {a} 0))"),
        ],
        _ => vec![range_int(i, r)],
    };
    let ra = range_int(i, a);
    fs.into_iter().map(|f| format!("(=> {ra} {f})")).collect()
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
    let signed = t.is_signed();
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
    let i = t.int().expect("integer type");
    let (min, max) = (num(i.min_value()), num(i.max_value()));
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
                    format!("(and (not (= {b} 0)) (not (and (= {a} {min}) (= {b} (- 1)))))")
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

/// `e` with each atom in `m` replaced by its term.
fn subst_atoms(e: &smt::Sexp, m: &BTreeMap<String, smt::Sexp>) -> smt::Sexp {
    match e {
        smt::Sexp::Atom(a) => m.get(a).cloned().unwrap_or_else(|| e.clone()),
        smt::Sexp::List(xs) => smt::Sexp::List(xs.iter().map(|x| subst_atoms(x, m)).collect()),
    }
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
            dead: false,
            loops: Vec::new(),
            ghost: false,
            valid_decl: BTreeSet::new(),
            valid_depth: 0,
            valid_defs: Vec::new(),
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
        (v, variants(value_ty(&t), self.prog.t).is_some())
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
                loop {
                    // An element of a `Vec` payload (backlog 2O-03c): an
                    // index below that `Vec`'s length.
                    let (head, arg) = if let Some((arr, i)) = select_app(cur)
                        && let Some((sel, x)) = sel_app(arr)
                        && let Some(len) = sel.strip_suffix(".e|").map(|p| format!("{p}.len|"))
                        && self.prog.selectors.contains_key(&len)
                    {
                        testers.push(format!("(and (<= 0 {i}) (< {i} ({len} {x})))"));
                        (sel, x)
                    } else if let Some(p) = sel_app(cur) {
                        p
                    } else {
                        break;
                    };
                    let Some(c) = self.prog.selectors.get(head) else {
                        break;
                    };
                    // A recursive struct's value is its one constructor
                    // (the placeholder is no program value).
                    if !self.prog.struct_ctors.contains(c) {
                        testers.push(format!("((_ is {c}) {arg})"));
                    }
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

    /// A value of the recursive struct `t` with `fields` (backlog 2O-03d):
    /// a fresh constant of its sort whose selectors are the field leaves,
    /// as ground facts (no constructor function: the queries stay free of
    /// quantifiers).
    fn mk_struct(&mut self, t: &TypeKind, fields: Vec<Val>) -> Val {
        let t = value_ty(t).clone();
        let x = self.fresh(&format!("{}!new", dt_ctor_name(&t)), &sort_of(&t));
        let name = dt_ctor_name(&t);
        let fs = struct_fields(&t, self.prog.t).expect("a struct");
        for ((k, (_, ft)), v) in fs.iter().enumerate().zip(&fields) {
            let mut terms = Vec::new();
            flatten(v, &mut terms);
            for ((sel, _), term) in payload_leaves(&t, &name, k, ft, self.prog.t)
                .iter()
                .zip(terms)
            {
                self.decls
                    .push(format!("(assert (= ({} {x}) {term}))", sym(sel)));
            }
        }
        Val::S(x)
    }

    /// `v` with the place at `steps` replaced by `new`.
    fn set_at(&mut self, v: Val, steps: &[Step], new: Val) -> Val {
        let tb = self.prog.t;
        set_at(v, steps, new, tb, &mut |t, fs| self.mk_struct(t, fs))
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
        let wanted = e.int().is_some_and(|i| !i.signed() || signed);
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
        let v = elem_template(e, self.prog.t, &mut terms.into_iter());
        // Every inner `Vec`'s length is a length (backlog 2O-03b).
        let mut facts = Vec::new();
        inner_len_facts(&v, e, self.prog.t, &mut facts);
        self.pc.extend(facts);
        v
    }

    fn fresh_val(&mut self, base: &str, t: &TypeKind) -> Val {
        if let Some(fs) = flat_fields(t, self.prog.t) {
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
            } else if let Some(vs) = enum_variants(t, self.prog.t) {
                // A value of code: each variant's payload leaves are in
                // their ranges when it is that variant (backlog 1B-50; a
                // `match` states the same facts on its arms, ADR-0027 §5).
                // A spec fn taking a payload slice unfolds only for an
                // argument whose length is in range.
                for (v, ps) in &vs {
                    if ps.is_empty() {
                        continue;
                    }
                    let (_, facts) = self.payload(t, v, &c);
                    if !facts.is_empty() {
                        self.pc
                            .push(format!("(=> ((_ is {}) {c}) {})", ctor(t, v), and(&facts)));
                    }
                }
            }
            Val::S(c)
        }
    }

    /// What a `proof fn` assumes of a parameter value `v` of type `t`, and
    /// what a call proves of the argument (E0819, ADR-0062 §2): the
    /// facts `fresh_val` states of a fresh value (the range of every
    /// integer leaf, the length bounds, every element's range in an integer
    /// array) except an enum value's payload ranges. Those are one level
    /// deep, so a lemma recursing into a payload (`&**t`) could not prove
    /// them of the payload's own payload; it states what it needs in
    /// `requires`.
    fn type_facts(&mut self, v: &Val, t: &TypeKind) -> Vec<String> {
        let mut out = Vec::new();
        if let (Some(fs), Val::Struct(vs)) = (flat_fields(t, self.prog.t), v) {
            for ((_, ft), fv) in fs.iter().zip(vs) {
                out.extend(self.type_facts(fv, ft));
            }
            return out;
        }
        if let TypeKind::Ref { inner, .. } = t
            && !is_array(t)
        {
            return self.type_facts(v, &inner.kind);
        }
        let len_fact = |len: &str, e: &TypeKind| {
            (!fixed_len(len)).then(|| format!("(and (<= 0 {len}) (<= {len} {}))", len_max(e)))
        };
        match v {
            Val::A { elems, len } if is_array(t) => {
                let e = elem_ty(t);
                if let Some(r) = range(&e, &format!("(select {elems} k)")) {
                    out.push(format!(
                        "(forall ((k Int)) (! {r} :pattern ((select {elems} k))))"
                    ));
                }
                out.extend(len_fact(len, &e));
            }
            Val::Arr { len, .. } if is_array(t) => {
                let e = elem_ty(t);
                inner_len_facts(v, &e, self.prog.t, &mut out);
                out.extend(len_fact(len, &e));
            }
            Val::S(c) => out.extend(range(t, c)),
            _ => {}
        }
        out
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

    /// A bitvector of `i`'s width standing for operand `x` of a bit
    /// operation in code: a literal, or a fresh constant whose value is
    /// stated (`bv_value`; `x` is in range there, as every value of code is,
    /// and a shift amount is below the width by its obligation).
    fn operand_bv(&mut self, i: IntTy, x: &str) -> String {
        let w = i.bits();
        if let Some(v) = numeral(x) {
            return format!("(_ bv{} {w})", v.rem_euclid(1i128 << w));
        }
        let c = self.fresh("'bv", &format!("(_ BitVec {w})"));
        self.pc.push(format!("(= (bv2nat {c}) {})", bv_value(i, x)));
        c
    }

    /// A bit method in code (`bit_method`): the operand is a named
    /// bit-vector, and the term's facts are assumed. The amount is
    /// converted as in a specification (`int2bv` of `a`'s width), so the two
    /// read alike for the congruence facts. A constant amount is linear and
    /// names nothing.
    fn bit_method_code(&mut self, m: &str, i: IntTy, a: &str, n: Option<&str>) -> String {
        if n.and_then(numeral).is_some() || (m == "swap_bytes" && i.bits() == 8) {
            return method_term(m, i, a, "", n, None);
        }
        let xa = self.operand_bv(i, a);
        let xn = n.map(|n| to_bv(i, n));
        let t = method_term(m, i, a, &xa, n, xn.as_deref());
        let facts = method_facts(m, i, a, &t);
        self.pc.extend(facts);
        t
    }

    /// A shift amount `s` (of any integer type) must be in `0..w`, `w` the
    /// width of the shifted value's type `i` (ADR-0048 §2: Rust's debug
    /// build panics there, so it is an overflow obligation, E0803).
    fn shift_obligation(&mut self, e: &Expr, op: BinOp, i: IntTy, amount: &Expr, s: &str) {
        let w = i.bits();
        let goal = match numeral(s) {
            Some(k) => (0 <= k && k < w as i128).to_string(),
            None if !self.ty(amount).is_signed() => format!("(< {s} {w})"),
            None => format!("(and (<= 0 {s}) (< {s} {w}))"),
        };
        self.oblige(
            "E0803",
            e.span,
            format!(
                "`{}` may overflow `{}`: the shift amount may be outside 0..{w}",
                op.symbol(),
                i.name()
            ),
            goal,
        );
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
                sub: None,
                shown: None,
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
            ExprKind::Return(_)
            | ExprKind::Try(_)
            | ExprKind::Loop { .. }
            | ExprKind::Break { .. }
            | ExprKind::Continue { .. } => unreachable!("rejected by the checker (E0503)"),
            ExprKind::Int(v, _) => Val::S(num(*v as i128)),
            ExprKind::Float(b) => Val::S(f64_lit(*b)),
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
            ExprKind::Unary(UnOp::Neg, x) if self.ty(x) == TypeKind::F64 => {
                let v = self.spec(x, env, old).s();
                Val::S(f64_neg(x, &v))
            }
            ExprKind::Unary(UnOp::Neg, x) => {
                if let ExprKind::Int(v, _) = x.kind {
                    return Val::S(num(-(v as i128)));
                }
                Val::S(format!("(- {})", self.spec(x, env, old).s()))
            }
            ExprKind::Unary(UnOp::Not, x) => {
                let v = self.spec(x, env, old).s();
                match self.ty(x).int() {
                    Some(i) => Val::S(not_term(i, &v)),
                    None => Val::S(format!("(not {v})")),
                }
            }
            // Structural `==` (ADR-0026 §2): field by field; a value that
            // holds an array compares its elements below the length only,
            // also inside enum payloads (backlog 2V-05).
            ExprKind::Binary(op @ (BinOp::Eq | BinOp::Ne), l, r)
                if flat_fields(&self.ty(l), self.prog.t).is_some()
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
                let signed = self.ty(l).is_signed();
                if self.ty(l) == TypeKind::F64
                    && let Some(t) = f64_binary(*op, &lt, &rt)
                {
                    return Val::S(t);
                }
                if op.is_bit() || op.is_shift() {
                    // In a specification a bit operation is its term alone:
                    // the facts and the shift obligation belong to code. Its
                    // width is its type's (operands may mix there).
                    return Val::S(match self.ty(e).int() {
                        Some(i) => bit_op(*op, i, &lt, &rt).0,
                        None => bool_bit(*op, &lt, &rt),
                    });
                }
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
                    _ => unreachable!("bit operations are above"),
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
                    m if value_ty(&self.ty(recv)) == &TypeKind::F64 => {
                        let b = args.first().map(|x| self.spec(x, env, old).s());
                        Val::S(f64_method(m, &rv.s(), b.as_deref()))
                    }
                    m if crate::check::bit_method_arity(m).is_some() => {
                        let i = self.ty(recv).int().expect("an integer method");
                        let n = args.first().map(|x| self.spec(x, env, old).s());
                        Val::S(bit_method(m, i, &rv.s(), n.as_deref()).0)
                    }
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
            // `f64::from(x)` (ADR-0061 §4): an integer's `f64`, or `x` itself.
            ExprKind::Call { path, args, .. } if crate::check::is_f64_from(path) => {
                let v = self.spec(&args[0], env, old).s();
                if self.ty(&args[0]) == TypeKind::F64 {
                    Val::S(v)
                } else {
                    Val::S(format!("(f64.from {v})"))
                }
            }
            ExprKind::Call { path, args, .. } if self.is_variant(path) => {
                let vals: Vec<Val> = args.iter().map(|a| self.spec(a, env, old)).collect();
                self.variant_value(&self.ty(e), &path.last().unwrap().name, &vals)
            }
            ExprKind::Path(segs) if crate::check::int_const(segs).is_some() => {
                Val::S(num(crate::check::int_const(segs).expect("checked").1))
            }
            ExprKind::Path(segs) if crate::check::f64_const(segs).is_some() => {
                Val::S(f64_lit(crate::check::f64_const(segs).expect("checked")))
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
                let vals: Vec<Val> = vals.into_iter().map(|(_, v)| v).collect();
                // The checker rejects a recursive struct built here (E0432).
                assert!(
                    !dt_struct(&t, self.prog.t),
                    "recursive struct literal in a specification"
                );
                Val::Struct(vals)
            }
            ExprKind::Field { base, name } => {
                let bt = self.ty(base);
                let k = field_index(&bt, self.prog.t, &name.name);
                match self.spec(base, env, old) {
                    Val::Struct(mut fs) => fs.swap_remove(k),
                    Val::S(x) if dt_struct(&bt, self.prog.t) => {
                        dt_fields(value_ty(&bt), &x, self.prog.t).swap_remove(k)
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
                sub: None,
                shown: None,
            });
            return;
        }
        self.oblige("E0813", span, msg, goal);
    }

    // ---------- executable code ----------

    fn eval(&mut self, e: &Expr) -> Val {
        if self.ghost {
            self.ghost_expr(e);
            return Val::Unit;
        }
        match &e.kind {
            ExprKind::Loop {
                label,
                invariants,
                decreases,
                body,
            } => self.loop_core(
                label.as_ref(),
                None,
                invariants,
                decreases.as_deref(),
                body,
                None,
                e.span,
            ),
            // The state goes to the loop's exit; the path ends.
            ExprKind::Break { label, value } => {
                let v = match value {
                    Some(x) => self.eval(x),
                    None => Val::Unit,
                };
                if !self.dead {
                    let k = self.jump_target(label);
                    self.record_break(k, v);
                }
                Val::Unit
            }
            // Back to the loop's head: its invariants and measure (a `for`
            // steps its variable first).
            ExprKind::Continue { label } => {
                if !self.dead {
                    let k = self.jump_target(label);
                    self.end_iteration(k);
                    self.dead = true;
                }
                Val::Unit
            }
            // `match x { None => return None, Some(v) => v }` (`Ok(v) => v,
            // Err(e) => return Err(e)`: the arms in declaration order),
            // encoded as that expansion (ADR-0049 §3).
            ExprKind::Try(x) => {
                let sv = self.eval(x).s();
                let st = self.ty(x);
                let (ok, bad) = crate::check::try_variants(&st).expect("checked `?`");
                let t = self.ty(e);
                // The checked result type: its data types by full path (the
                // written one names a unit's type by its short name).
                let rt = crate::check::subst(&self.prog.t.fns[&self.path].ret, &self.tsub);
                let go_on = |s: &mut Self| {
                    let (vals, facts) = s.payload(&st, ok, &sv);
                    s.pc.extend(facts);
                    vals.into_iter().next().expect("one payload")
                };
                let leave = |s: &mut Self| {
                    let (vals, facts) = s.payload(&st, bad, &sv);
                    s.pc.extend(facts);
                    let r = s.variant_value(&rt, bad, &vals);
                    s.post_check(r, e.span, "at this `?`");
                    s.dead = true;
                    Val::Unit
                };
                if ok == "Some" {
                    let c = format!("((_ is {}) {sv})", ctor(&st, bad));
                    self.branch(&c, &t, leave, go_on)
                } else {
                    let c = format!("((_ is {}) {sv})", ctor(&st, ok));
                    self.branch(&c, &t, go_on, leave)
                }
            }
            // The postconditions hold here, and the path ends (ADR-0049 §3).
            ExprKind::Return(x) => {
                let v = match x {
                    Some(x) => self.eval(x),
                    None => Val::Unit,
                };
                if !self.dead {
                    self.post_check(v, e.span, "at this `return`");
                    self.dead = true;
                }
                Val::Unit
            }
            ExprKind::Int(v, _) => Val::S(num(*v as i128)),
            ExprKind::Float(b) => Val::S(f64_lit(*b)),
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
            // `f64` arithmetic is total in Rust: no obligation (ADR-0061 §3).
            ExprKind::Unary(UnOp::Neg, x) if self.ty(x) == TypeKind::F64 => {
                let v = self.eval(x).s();
                Val::S(f64_neg(x, &v))
            }
            ExprKind::Unary(UnOp::Neg, x) => {
                if let ExprKind::Int(v, _) = x.kind {
                    return Val::S(num(-(v as i128)));
                }
                let t = format!("(- {})", self.eval(x).s());
                let ty = self.ty(e);
                if let Some(r) = range(&ty, &t) {
                    self.oblige("E0803", e.span, "negation may overflow", r);
                }
                Val::S(t)
            }
            ExprKind::Unary(UnOp::Not, x) => {
                let v = self.eval(x).s();
                match self.ty(x).int() {
                    Some(i) => Val::S(not_term(i, &v)),
                    None => Val::S(format!("(not {v})")),
                }
            }
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
                // A right side that returned: the path goes on where it did
                // not run, and the value is the left one's.
                if std::mem::replace(&mut self.dead, false) {
                    self.pc.truncate(n);
                    self.pc.push(format!("(not {guard})"));
                    return Val::S(lt);
                }
                // Facts learned on the right (a callee's `ensures`, division
                // lemmas, element ranges) hold whenever it ran (1B-29).
                let facts: Vec<String> = self.pc[n + 1..].to_vec();
                self.pc.truncate(n);
                if !facts.is_empty() {
                    self.pc.push(format!("(=> {guard} {})", and(&facts)));
                }
                Val::S(format!("({name} {lt} {rt})"))
            }
            ExprKind::Binary(op, l, r) if self.ty(l) == TypeKind::F64 => {
                let lt = self.eval(l).s();
                let rt = self.eval(r).s();
                Val::S(f64_binary(*op, &lt, &rt).expect("an `f64` operation"))
            }
            ExprKind::Binary(op, l, r) => {
                let lt = self.eval(l).s();
                let rt = self.eval(r).s();
                let ty = self.ty(l);
                let signed = ty.is_signed();
                if op.is_bit() || op.is_shift() {
                    let Some(i) = ty.int() else {
                        return Val::S(bool_bit(*op, &lt, &rt));
                    };
                    if op.is_shift() {
                        self.shift_obligation(e, *op, i, r, &rt);
                    }
                    if let Some(t) = linear_bit(*op, i, &lt, &rt) {
                        return Val::S(t);
                    }
                    let xa = self.operand_bv(i, &lt);
                    let xb = self.operand_bv(i, &rt);
                    let t = bv_term(*op, i, &xa, &xb);
                    let facts = bit_facts(*op, i, &lt, &rt, &t);
                    self.pc.extend(facts);
                    return Val::S(t);
                }
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
                            let i = ty.int().expect("integer type");
                            self.oblige(
                                "E0803",
                                e.span,
                                format!("`{}::MIN % -1` may occur", i.name()),
                                format!(
                                    "(not (and (= {lt} {}) (= {rt} (- 1))))",
                                    num(i.min_value())
                                ),
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
                    _ => unreachable!("handled above"),
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
                    m if value_ty(&self.ty(recv)) == &TypeKind::F64 => {
                        let b = args.first().map(|x| self.eval(x).s());
                        Val::S(f64_method(m, &rv.s(), b.as_deref()))
                    }
                    m if crate::check::bit_method_arity(m).is_some() => {
                        let i = self.ty(recv).int().expect("an integer method");
                        let n = args.first().map(|x| self.eval(x).s());
                        Val::S(self.bit_method_code(m, i, &rv.s(), n.as_deref()))
                    }
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
            // `f64::from(x)` (ADR-0061 §4): exact for a source of 32 bits or
            // fewer; a 64-bit source must be within `2^53` (E0815).
            ExprKind::Call { path, args, .. } if crate::check::is_f64_from(path) => {
                let v = self.eval(&args[0]).s();
                let st = self.ty(&args[0]);
                match st.int() {
                    None => Val::S(v),
                    Some(i) => {
                        if i.bits() == 64 {
                            self.oblige(
                                "E0815",
                                e.span,
                                format!(
                                    "`f64::from` of `{}` may not be exact (|x| <= 2^53)",
                                    i.name()
                                ),
                                format!("(and (<= (- {F64_EXACT}) {v}) (<= {v} {F64_EXACT}))"),
                            );
                        }
                        Val::S(format!("(f64.from {v})"))
                    }
                }
            }
            ExprKind::Call { path, args, .. } if crate::check::mem_op(path).is_some() => {
                self.mem_call(crate::check::mem_op(path).expect("checked"), args)
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
                let vals: Vec<Val> = vals.into_iter().map(|(_, v)| v).collect();
                if dt_struct(&t, self.prog.t) {
                    return self.mk_struct(&t, vals);
                }
                Val::Struct(vals)
            }
            ExprKind::Field { base, name } => {
                // A place under `&`/`&mut` has no recorded type.
                let bt = match self.ty(base) {
                    TypeKind::Error => self.place_ty(base),
                    t => t,
                };
                let k = field_index(&bt, self.prog.t, &name.name);
                match self.eval(base) {
                    Val::Struct(mut fs) => fs.swap_remove(k),
                    // A recursive struct (backlog 2O-03d): its selectors
                    // (their facts are added to each query, `struct_facts`).
                    Val::S(x) if dt_struct(&bt, self.prog.t) => {
                        dt_fields(value_ty(&bt), &x, self.prog.t).swap_remove(k)
                    }
                    v => panic!("field of non-struct {v:?}"),
                }
            }
            // A reference denotes its value (references are second-class).
            ExprKind::Deref(x) => self.eval(x),
            ExprKind::Path(segs) if crate::check::int_const(segs).is_some() => {
                Val::S(num(crate::check::int_const(segs).expect("checked").1))
            }
            ExprKind::Path(segs) if crate::check::f64_const(segs).is_some() => {
                Val::S(f64_lit(crate::check::f64_const(segs).expect("checked")))
            }
            ExprKind::Path(segs) => {
                self.variant_value(&self.ty(e), &segs.last().unwrap().name, &[])
            }
            ExprKind::Match { scrut, arms } => {
                let sv = self.eval(scrut).s();
                let st = self.ty(scrut);
                let t = self.ty(e);
                // A `&mut` match (ADR-0055): the arm's bindings are written
                // back to the scrutinee's place at its end.
                let back = match (&st, &scrut.kind) {
                    (TypeKind::Ref { mutable: true, .. }, ExprKind::Ref { expr, .. }) => {
                        Some((**expr).clone())
                    }
                    (TypeKind::Ref { mutable: true, .. }, _) => Some((**scrut).clone()),
                    _ => None,
                };
                self.match_arms(&sv, &st, arms, 0, &t, back.as_ref())
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
        let then_dead = std::mem::replace(&mut self.dead, false);
        self.pc.truncate(n);
        self.pc.push(format!("(not {c})"));
        let ev = else_f(self);
        let else_vars = std::mem::take(&mut self.vars);
        let else_facts: Vec<String> = self.pc[n + 1..].to_vec();
        let else_dead = std::mem::replace(&mut self.dead, false);
        self.pc.truncate(n);
        // A side that returned does not reach the join (ADR-0049 §3): the
        // path goes on under the other side's condition and facts.
        match (then_dead, else_dead) {
            (true, true) => {
                self.dead = true;
                self.vars = base_vars;
                return tv;
            }
            (true, false) => {
                self.vars = else_vars;
                self.pc.push(format!("(not {c})"));
                self.pc.extend(else_facts);
                return ev;
            }
            (false, true) => {
                self.vars = then_vars;
                self.pc.push(c.to_string());
                self.pc.extend(then_facts);
                return tv;
            }
            (false, false) => {}
        }
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
                let tys: Vec<TypeKind> = match flat_fields(t, self.prog.t) {
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

    /// The postconditions of the function at a point where it returns
    /// `result` (the end of the body, or a `return`, ADR-0049 §3).
    fn post_check(&mut self, result: Val, at: Span, how: &str) {
        let f = self.prog.defs[&self.path].f;
        let (mut env, old) = self.cur_env();
        if let Some(n) = &f.ret.name {
            env.insert(n.name.clone(), result.clone());
        }
        for en in &f.ensures {
            let g = self.spec(en, &env, &old).s();
            self.oblige(
                "E0802",
                en.span,
                format!("postcondition `{}` may not hold", pred_name(en)),
                g,
            );
        }
        // What the function lets go is valid (ADR-0060 §2).
        self.exit_valid(&result, at, how);
    }

    fn block(&mut self, b: &Block) -> Val {
        // What follows a `return` on this path does not run.
        for s in &b.stmts {
            if self.dead {
                return Val::Unit;
            }
            self.stmt(s);
        }
        match &b.tail {
            Some(t) if !self.dead => self.eval(t),
            _ => Val::Unit,
        }
    }

    fn cur_env(&self) -> (BTreeMap<String, Val>, BTreeMap<String, Val>) {
        (self.vars.clone(), self.old.clone())
    }

    /// Ghost code (ADR-0062 §2): `if` and `match` branch on specifications
    /// (the facts proved on a side hold under its condition), a `proof fn`
    /// call proves its preconditions and assumes its `ensures`, and any
    /// other expression is a specification with no effect.
    fn ghost_expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::If { cond, then, els } => {
                let (env, old) = self.cur_env();
                let c = self.spec(cond, &env, &old).s();
                self.branch(
                    &c,
                    &TypeKind::Unit,
                    |s| s.block(then),
                    |s| match els {
                        Some(x) => s.eval(x),
                        None => Val::Unit,
                    },
                );
            }
            ExprKind::Match { scrut, arms } => {
                let (env, old) = self.cur_env();
                let sv = self.spec(scrut, &env, &old).s();
                let st = value_ty(&self.ty(scrut)).clone();
                self.match_arms(&sv, &st, arms, 0, &TypeKind::Unit, None);
            }
            ExprKind::Block(b) => {
                self.block(b);
            }
            ExprKind::Call { path, args, .. } => {
                let target = self.resolve(path, &self.mod_path.clone());
                if self
                    .prog
                    .t
                    .fns
                    .get(&target)
                    .is_some_and(|s| s.kind == FnKind::Proof)
                {
                    self.call(path, args, e.span);
                }
            }
            _ => {}
        }
    }

    /// A `proof { .. }` block in executable code: its facts hold from here
    /// on; its `let`s end with it.
    fn proof_block(&mut self, body: &Block) {
        let before: BTreeSet<String> = self.vars.keys().cloned().collect();
        let ghost = std::mem::replace(&mut self.ghost, true);
        self.block(body);
        self.ghost = ghost;
        self.vars.retain(|k, _| before.contains(k));
    }

    /// `assert(e) by (bit_vector)` (ADR-0048 §4, backlog 2N-03): `e` is
    /// proved in a query of its own (`subproof::bit_vector`) that sees its
    /// free variables only, each over its whole type, and none of the path's
    /// facts (Verus's rule; a needed fact is written as `p ==> q`). The
    /// query translates the very term the enclosing query then assumes,
    /// under the free variables' ranges and the guards of unfolded spec fns
    /// (true of every value of code: a variable of code is in its type).
    /// Code, a `proof` block and a `proof fn` body treat it alike.
    fn sub_proof(&mut self, _prover: Prover, cond: &Expr, span: Span) {
        let mut free = BTreeMap::new();
        self.free_scalars(cond, &mut Vec::new(), &mut free);
        let (env, old) = self.cur_env();
        let fresh: BTreeMap<String, Val> =
            free.keys().map(|n| (n.clone(), Val::S(sym(n)))).collect();
        let term = self.spec(cond, &fresh, &BTreeMap::new()).s();
        let fact = self.spec(cond, &env, &old).s();
        let vars: Vec<(String, crate::subproof::Sort)> =
            free.iter().map(|(n, s)| (sym(n), *s)).collect();
        let revealed: BTreeSet<String> = self
            .prog
            .revealed(self.prog.defs[&self.path].f, &self.mod_path)
            .into_iter()
            .collect();
        let mut hidden = Vec::new();
        let query = match smt::parse(&term) {
            Some(t) => crate::subproof::bit_vector(&t, &vars, &mut |f, ps| {
                self.unfold_spec(f, ps, &revealed, &mut hidden)
            }),
            None => Err("the assertion has no SMT term".to_string()),
        }
        .map_err(|why| (why, hidden));
        // The variables' values here, for the assumed fact.
        let real: BTreeMap<String, smt::Sexp> = free
            .keys()
            .filter_map(|n| Some((sym(n), smt::parse(&env.get(n)?.s())?)))
            .collect();
        let mut ante: Vec<String> = free
            .iter()
            .filter_map(|(n, s)| match s {
                crate::subproof::Sort::Int(i) => Some(range_int(*i, &env.get(n)?.s())),
                crate::subproof::Sort::Bool | crate::subproof::Sort::F64 => None,
            })
            .collect();
        if let Ok(q) = &query {
            for g in &q.guards {
                if let Some(g) = smt::parse(g) {
                    ante.push(smt::print(&subst_atoms(&g, &real)));
                }
            }
        }
        let goal = query
            .as_ref()
            .map_or("false".to_string(), |q| q.goal.clone());
        self.obls.push(Obl {
            code: "E0818",
            span,
            msg: "bit-vector assertion may not hold".into(),
            assumptions: Vec::new(),
            goal,
            sub: Some(query),
            shown: None,
        });
        self.pc.push(if ante.is_empty() {
            fact
        } else {
            format!("(=> {} {fact})", and(&ante))
        });
    }

    /// The free variables of a `by (bit_vector)` assertion (integers and
    /// `bool`s, E0515) with their sorts; `bound` are the `let`s around.
    fn free_scalars(
        &self,
        e: &Expr,
        bound: &mut Vec<String>,
        out: &mut BTreeMap<String, crate::subproof::Sort>,
    ) {
        match &e.kind {
            ExprKind::Var(id) if !bound.contains(&id.name) => {
                let sort = match self.ty(e) {
                    TypeKind::Int(i) => crate::subproof::Sort::Int(i),
                    TypeKind::F64 => crate::subproof::Sort::F64,
                    _ => crate::subproof::Sort::Bool,
                };
                out.insert(id.name.clone(), sort);
            }
            ExprKind::Unary(_, x) => self.free_scalars(x, bound, out),
            ExprKind::Binary(_, a, b) => {
                self.free_scalars(a, bound, out);
                self.free_scalars(b, bound, out);
            }
            ExprKind::If { cond, then, els } => {
                self.free_scalars(cond, bound, out);
                self.free_block(then, bound, out);
                if let Some(x) = els {
                    self.free_scalars(x, bound, out);
                }
            }
            ExprKind::Block(b) => self.free_block(b, bound, out),
            ExprKind::Call { args, .. } => {
                args.iter().for_each(|a| self.free_scalars(a, bound, out));
            }
            ExprKind::Method { recv, args, .. } => {
                self.free_scalars(recv, bound, out);
                args.iter().for_each(|a| self.free_scalars(a, bound, out));
            }
            _ => {}
        }
    }

    fn free_block(
        &self,
        b: &Block,
        bound: &mut Vec<String>,
        out: &mut BTreeMap<String, crate::subproof::Sort>,
    ) {
        let n = bound.len();
        for s in &b.stmts {
            if let Stmt::Let { name, init, .. } = s {
                self.free_scalars(init, bound, out);
                bound.push(name.name.clone());
            }
        }
        if let Some(t) = &b.tail {
            self.free_scalars(t, bound, out);
        }
        bound.truncate(n);
    }

    /// The body of the spec fn whose SMT symbol is `f`, over the
    /// placeholders `ps` for its parameters, for a sub-proof. Only a
    /// non-recursive, non-generic spec fn of integers and `bool`s that the
    /// definition reveals (`#[uses]`) unfolds; its body is translated as the
    /// encoder translates it everywhere (`spec_block`).
    fn unfold_spec(
        &mut self,
        f: &str,
        ps: &[String],
        revealed: &BTreeSet<String>,
        hidden: &mut Vec<String>,
    ) -> Result<Option<crate::subproof::Unfolded>, String> {
        use crate::subproof::Sort;
        if f == "bmb_tdiv" || f == "bmb_trem" {
            return Err(
                "signed `/` and `%` have no bit-vector translation (divide by a constant of an unsigned type)"
                    .into(),
            );
        }
        let prog = self.prog;
        let Some((path, d)) = prog
            .defs
            .iter()
            .find(|(p, d)| d.f.kind == FnKind::Spec && sym(p) == f)
        else {
            return Ok(None);
        };
        let sig = &prog.t.fns[path];
        if d.inst || !sig.generics.is_empty() {
            return Err(format!(
                "`{path}` is generic: a bit-vector sub-proof unfolds spec fns of integers, `f64`s and `bool`s"
            ));
        }
        if !revealed.contains(path) {
            // As `hidden_predicates` names it: local in its own unit.
            let local = path
                .strip_prefix(&format!("{}::", self.mod_path))
                .filter(|l| !l.contains("::"));
            hidden.push(local.unwrap_or(path).to_string());
            return Err(format!(
                "`{path}` is not revealed (the sub-proof unfolds the spec fns the assertion calls)"
            ));
        }
        if prog.t.recursive.contains_key(path) {
            return Err(format!(
                "`{path}` is recursive: a bit-vector sub-proof unfolds only non-recursive spec fns"
            ));
        }
        let sort = |t: &TypeKind| match t {
            TypeKind::Int(i) => Some(Sort::Int(*i)),
            TypeKind::Bool => Some(Sort::Bool),
            TypeKind::F64 => Some(Sort::F64),
            _ => None,
        };
        let params: Option<Vec<Sort>> = sig.params.iter().map(sort).collect();
        let (Some(params), Some(_)) = (params, sort(&sig.ret)) else {
            return Err(format!(
                "`{path}` takes or returns a value that is not an integer, `f64` or `bool`"
            ));
        };
        let env: BTreeMap<String, Val> =
            d.f.params
                .iter()
                .zip(ps)
                .map(|(p, s)| (p.name.name.clone(), Val::S(s.clone())))
                .collect();
        let body = self
            .in_def(&d.mod_path, BTreeMap::new(), |s| {
                s.spec_block(&d.f.body, &env, &BTreeMap::new())
            })
            .s();
        let body = smt::parse(&body).ok_or_else(|| format!("`{path}` has no SMT term"))?;
        Ok(Some(crate::subproof::Unfolded { body, params }))
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::LetFields { .. } => unreachable!("rejected by the checker (E0230)"),
            // A ghost `let` binds a specification value.
            Stmt::Let { name, init, .. } if self.ghost => {
                let (env, old) = self.cur_env();
                let v = self.spec(init, &env, &old);
                self.vars.insert(name.name.clone(), v);
            }
            Stmt::Let { name, init, .. } => {
                let v = self.eval(init);
                self.vars.insert(name.name.clone(), v);
            }
            Stmt::Proof { body, .. } => self.proof_block(body),
            Stmt::Assign { target, value, .. } => {
                // Rust evaluates the value, then the place (and its bounds).
                let v = self.eval(value);
                let mut steps = Vec::new();
                let root = self.place(target, &mut steps, true);
                let cur = self.vars[&root].clone();
                let nv = self.set_at(cur, &steps, v);
                self.vars.insert(root, nv);
            }
            Stmt::Assert {
                cond,
                by: Some(p),
                span,
            } => self.sub_proof(*p, cond, *span),
            Stmt::Assert { cond, span, .. } => {
                let (env, old) = self.cur_env();
                let g = self.spec(cond, &env, &old).s();
                self.oblige("E0810", *span, "assertion may not hold", g);
            }
            Stmt::Expr(e) => {
                self.eval(e);
            }
            Stmt::While {
                label,
                cond,
                invariants,
                decreases,
                body,
                span,
            } => {
                self.loop_core(
                    label.as_ref(),
                    Some(cond),
                    invariants,
                    decreases.as_ref(),
                    body,
                    None,
                    *span,
                );
            }
            // `let mut i = lo; while i < hi invariant lo <= i && (i <= hi
            // || i == lo), .. decreases hi - i { body; i = i + 1; }` with
            // the bounds evaluated once (ADR-0049 §2, §3): the encoding is
            // that expansion's, query for query (backlog 2K-01d).
            Stmt::For {
                label,
                var,
                lo,
                hi,
                inclusive,
                invariants,
                body,
                span,
            } => {
                let lo = self.eval(lo).s();
                let hi = self.eval(hi).s();
                let ty = self.local_ty(&var.name);
                self.vars.insert(var.name.clone(), Val::S(lo.clone()));
                let step = ForStep {
                    var: var.name.clone(),
                    ty,
                    lo,
                    hi,
                    inclusive: *inclusive,
                    span: *span,
                };
                self.loop_core(
                    label.as_ref(),
                    None,
                    invariants,
                    None,
                    body,
                    Some(step),
                    *span,
                );
                self.vars.remove(&var.name);
            }
        }
    }

    /// A `while` (with `cond`), a range `for` (with `step`) or a `loop`
    /// (with neither): its value is that of the `break e` that leaves a
    /// `loop` (ADR-0049 §3).
    #[allow(clippy::too_many_arguments)]
    fn loop_core(
        &mut self,
        label: Option<&Ident>,
        cond: Option<&Expr>,
        invariants: &[Expr],
        decreases: Option<&Expr>,
        body: &Block,
        step: Option<ForStep>,
        span: Span,
    ) -> Val {
        // A `while` and a `for` also leave when their condition is false.
        let exits = cond.is_some() || step.is_some();
        if let Some(st) = &step {
            let g = for_bounds(st, &st.lo);
            self.oblige("E0806", st.span, "loop invariant may not hold on entry", g);
        }
        for inv in invariants {
            let (env, old) = self.cur_env();
            let g = self.spec(inv, &env, &old).s();
            self.oblige("E0806", inv.span, "loop invariant may not hold on entry", g);
        }
        // Havoc everything the loop may change, then assume the invariants.
        let mut modified = BTreeSet::new();
        assigned_in_block(body, &mut modified);
        if let Some(cond) = cond {
            assigned_in_expr(cond, &mut modified);
        }
        if let Some(st) = &step {
            modified.insert(st.var.clone());
        }
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
        // The values the loop havocs are valid on entry, and assumed valid
        // at the head (ADR-0060 §2).
        let valid_vars = self.loop_valid_vars(&modified);
        for (v, ty) in &valid_vars {
            let cur = self.vars[v].clone();
            self.oblige_valid(span, Boundary::LoopEntry(v), &cur, ty);
        }
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
        for (v, ty) in &valid_vars {
            let cur = self.vars[v].clone();
            self.assume_valid(&cur, ty);
        }
        // Earlier facts mention only earlier constants, so they stay valid.
        if let Some(st) = &step {
            let i = self.vars[&st.var].s();
            self.pc.push(for_bounds(st, &i));
        }
        for inv in invariants {
            let (env, old) = self.cur_env();
            let g = self.spec(inv, &env, &old).s();
            self.pc.push(g);
        }
        let c = match (cond, &step) {
            (Some(cond), _) => self.eval(cond).s(),
            (None, Some(st)) => {
                let op = if st.inclusive { BinOp::Le } else { BinOp::Lt };
                ord_term(op, &st.ty, &self.vars[&st.var].s(), &st.hi)
            }
            (None, None) => "true".to_string(),
        };
        let after_cond_vars = self.vars.clone();
        let n = self.pc.len();
        // One arbitrary iteration.
        self.pc.push(c.clone());
        let measure = match &step {
            Some(st) => Some(format!("(- {} {})", st.hi, self.vars[&st.var].s())),
            None => decreases.map(|d| {
                let (env, old) = self.cur_env();
                self.spec(d, &env, &old).s()
            }),
        };
        self.loops.push(VLoop {
            label: label.map(|l| l.name.clone()),
            start: n,
            invariants: invariants.to_vec(),
            decreases: decreases.cloned(),
            measure,
            breaks: Vec::new(),
            step,
            span,
            valid_vars,
        });
        self.block(body);
        // A body that left (`return`, `break`, `continue`) does not come
        // back to the head from its end.
        if !self.dead {
            self.end_iteration(self.loops.len() - 1);
        }
        self.dead = false;
        let lp = self.loops.pop().expect("pushed");
        // Exit: the head with the condition false, or the state at a
        // `break` (the conditions are exclusive: a `break` runs in an
        // iteration, under the condition).
        self.pc.truncate(n);
        let conds: Vec<String> = lp.breaks.iter().map(|(pc, _, _)| and(pc)).collect();
        let mut alts: Vec<String> = conds.clone();
        if exits {
            alts.insert(0, format!("(not {c})"));
        }
        // A `loop` without a `break` does not finish.
        if alts.is_empty() {
            self.vars = after_cond_vars;
            self.dead = true;
            return Val::Unit;
        }
        self.pc.push(if alts.len() == 1 {
            alts[0].clone()
        } else {
            format!("(or {})", alts.join(" "))
        });
        let mut vars = BTreeMap::new();
        for (k, v0) in &after_cond_vars {
            let mut v = if exits {
                v0.clone()
            } else {
                lp.breaks.last().map_or(v0, |b| &b.1[k]).clone()
            };
            let n_breaks = lp.breaks.len() - usize::from(!exits);
            for i in (0..n_breaks).rev() {
                let b = &lp.breaks[i].1[k];
                if *b != v {
                    let t = self.local_ty(k);
                    v = self.merge(&conds[i], b, &v, &t);
                }
            }
            vars.insert(k.clone(), v);
        }
        self.vars = vars;
        if let Some(cond) = cond {
            self.mark_reach(cond.span, "exit of the loop");
        }
        if let Some(st) = &lp.step {
            self.mark_reach(st.span, "exit of the loop");
        }
        // The value of a `loop`: its `break`s'.
        let Some((_, _, last)) = lp.breaks.last() else {
            return Val::Unit;
        };
        let mut v = last.clone();
        for i in (0..lp.breaks.len() - 1).rev() {
            let b = lp.breaks[i].2.clone();
            v = ite_val(&conds[i], &b, &v);
        }
        v
    }

    /// The state goes to loop `k`'s exit with value `v`; the path ends.
    fn record_break(&mut self, k: usize, v: Val) {
        let pc = self.pc[self.loops[k].start..].to_vec();
        let vars = self.vars.clone();
        self.loops[k].breaks.push((pc, vars, v));
        self.dead = true;
    }

    /// The end of an iteration of loop `k` (its body's end, or a
    /// `continue`): a `for` steps its variable as `if i == hi { break; } i
    /// = i + 1;` (the test only for `lo..=hi`) would, then the path goes
    /// back to the head.
    fn end_iteration(&mut self, k: usize) {
        if let Some(st) = self.loops[k].step.clone() {
            let i = self.vars[&st.var].s();
            if st.inclusive {
                let at_end = format!("(= {i} {})", st.hi);
                self.branch(
                    &at_end,
                    &TypeKind::Unit,
                    |s| {
                        s.record_break(k, Val::Unit);
                        Val::Unit
                    },
                    |_| Val::Unit,
                );
                if self.dead {
                    return;
                }
            }
            let next = format!("(+ {i} 1)");
            if let Some(rg) = range(&st.ty, &next) {
                let msg = format!("`+` may overflow `{}`", crate::check::ty_str(&st.ty));
                self.oblige("E0803", st.span, msg, rg);
            }
            self.vars.insert(st.var.clone(), Val::S(next));
        }
        self.back_to_head(k);
    }

    /// The obligations of going back to the head of loop `k` from here (the
    /// end of its body, or a `continue`): its invariants hold, its measure
    /// decreased.
    fn back_to_head(&mut self, k: usize) {
        let span = self.loops[k].span;
        for (v, ty) in self.loops[k].valid_vars.clone() {
            if let Some(cur) = self.vars.get(&v).cloned() {
                self.oblige_valid(span, Boundary::LoopBack(&v), &cur, &ty);
            }
        }
        if let Some(st) = self.loops[k].step.clone() {
            let g = for_bounds(&st, &self.vars[&st.var].s());
            self.oblige("E0807", st.span, "loop invariant may not be preserved", g);
        }
        let invariants = self.loops[k].invariants.clone();
        for inv in &invariants {
            let (env, old) = self.cur_env();
            let g = self.spec(inv, &env, &old).s();
            self.oblige("E0807", inv.span, "loop invariant may not be preserved", g);
        }
        // The measure here: a `for`'s is `hi - i` (ADR-0049 §2).
        let (step, decreases) = (self.loops[k].step.clone(), self.loops[k].decreases.clone());
        let d1 = match (&step, &decreases) {
            (Some(st), _) => Some((st.span, format!("(- {} {})", st.hi, self.vars[&st.var].s()))),
            (None, Some(d)) => {
                let (env, old) = self.cur_env();
                Some((d.span, self.spec(d, &env, &old).s()))
            }
            (None, None) => None,
        };
        if let (Some(d0), Some((span, d1))) = (self.loops[k].measure.clone(), d1) {
            self.oblige(
                "E0808",
                span,
                "`decreases` may not decrease or may be negative",
                format!("(and (<= 0 {d0}) (< {d1} {d0}))"),
            );
        }
    }

    /// The loop a `break`/`continue` leaves: by label, or the innermost.
    fn jump_target(&self, label: &Option<Ident>) -> usize {
        match label {
            Some(l) => self
                .loops
                .iter()
                .rposition(|c| c.label.as_deref() == Some(l.name.as_str()))
                .expect("checked label"),
            None => self.loops.len() - 1,
        }
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
        let nv = self.set_at(cur, &steps, new);
        self.vars.insert(root, nv);
        result
    }

    /// `std::mem::replace(&mut p, v)`: the old value of `p`, which now
    /// holds `v`; `std::mem::swap(&mut a, &mut b)` exchanges the values of
    /// two places of different variables (ADR-0047 §2).
    fn mem_call(&mut self, op: &str, args: &[Expr]) -> Val {
        let inner = |a: &Expr| match &a.kind {
            ExprKind::Ref { expr, .. } => (**expr).clone(),
            _ => a.clone(),
        };
        let (p0, p1) = (inner(&args[0]), inner(&args[1]));
        let old = self.eval(&p0);
        let new = self.eval(if op == "replace" { &args[1] } else { &p1 });
        let mut steps = Vec::new();
        let root = self.place(&p0, &mut steps, true);
        let cur = self.vars[&root].clone();
        let nv = self.set_at(cur, &steps, new);
        self.vars.insert(root, nv);
        if op == "replace" {
            return old;
        }
        let mut steps = Vec::new();
        let root = self.place(&p1, &mut steps, true);
        let cur = self.vars[&root].clone();
        let nv = self.set_at(cur, &steps, old);
        self.vars.insert(root, nv);
        Val::Unit
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
        if agg_elem(&el, self.prog.t) {
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
        let vs = variants(t, self.prog.t).expect("match on an enum");
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
        if let Some(fs) = flat_fields(t, tb) {
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
        if let Some(vs) = variants(t, tb)
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
    fn match_arms(
        &mut self,
        sv: &str,
        st: &TypeKind,
        arms: &[Arm],
        i: usize,
        t: &TypeKind,
        back: Option<&Expr>,
    ) -> Val {
        let arm = &arms[i];
        let v = arm.pat.path.last().unwrap().name.clone();
        let run = |s: &mut Self| {
            let (vals, facts) = s.payload(st, &v, sv);
            s.pc.extend(facts);
            for (b, val) in arm.pat.binds.iter().zip(&vals) {
                if let Some(id) = b {
                    s.vars.insert(id.name.clone(), val.clone());
                }
            }
            let r = s.eval(&arm.body);
            if let Some(place) = back {
                let now: Vec<Val> = arm
                    .pat
                    .binds
                    .iter()
                    .zip(vals)
                    .map(|(b, val)| match b {
                        Some(id) => s.vars[&id.name].clone(),
                        None => val,
                    })
                    .collect();
                let new = s.variant_value(st, &v, &now);
                let mut steps = Vec::new();
                let root = s.place(place, &mut steps, false);
                let cur = s.vars[&root].clone();
                let nv = s.set_at(cur, &steps, new);
                s.vars.insert(root, nv);
            }
            r
        };
        if i + 1 == arms.len() {
            return run(self);
        }
        let c = format!("((_ is {}) {sv})", ctor(st, &v));
        self.branch(&c, t, run, |s| s.match_arms(sv, st, arms, i + 1, t, back))
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
                let k = field_index(&bt, self.prog.t, &name.name);
                steps.push(if dt_struct(&bt, self.prog.t) {
                    Step::DtField(k, value_ty(&bt).clone())
                } else {
                    Step::Field(k)
                });
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
    /// is assumed afterwards. `at` says where the obligations stand
    /// (`RefineAt`).
    fn refines(&mut self, f: &FnDef, decl: &str, tsub: BTreeMap<String, TypeKind>, at: RefineAt) {
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
            // The domain the `ensures` are compared on: `decl`'s
            // `requires`, or `f`'s own (the backward direction of a version
            // comparison, `RefineJob::own_domain`).
            let mut assumed = match at {
                RefineAt::Version { own_domain: true } => f
                    .requires
                    .iter()
                    .map(|r| self.spec(r, &env, &old).s())
                    .collect(),
                _ => treq.clone(),
            };
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
                // A version comparison tells the clauses apart by their
                // spans and shows the result in a counterexample.
                let span = match at {
                    RefineAt::Trait => f.name.span,
                    RefineAt::Version { .. } => en.span,
                };
                let n = self.obls.len();
                self.oblige(
                    "E0817",
                    span,
                    format!("postcondition `{}` of `{decl}` may not hold", pred_name(en)),
                    format!("(=> {hyp} {g})"),
                );
                // The result, or for a `()` function the new value behind
                // its first `&mut` parameter.
                if let RefineAt::Version { .. } = at
                    && self.obls.len() > n
                {
                    let after = f.params.iter().find(|p| {
                        ret == TypeKind::Unit
                            && matches!(p.ty.kind, TypeKind::Ref { mutable: true, .. })
                    });
                    self.obls[n].shown = match after {
                        Some(p) => Some((
                            format!("{} after", p.name.name),
                            post[&p.name.name].clone(),
                            self.local_ty(&p.name.name),
                        )),
                        None => {
                            let name = f.ret.name.as_ref().map_or("result", |r| r.name.as_str());
                            Some((name.to_string(), result.clone(), ret.clone()))
                        }
                    };
                }
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
            // A lemma's arguments are specifications (ADR-0062 §2).
            let v = if self.ghost {
                let (env, old) = self.cur_env();
                self.spec(a, &env, &old)
            } else {
                self.eval(a)
            };
            argv.push(v);
        }
        let Some(def) = self.prog.defs.get(&target) else {
            // The built-in functions have no contract but `print_f64`'s
            // precision bound (Rust panics above it); anything else
            // unresolved would drop a callee's effects.
            assert!(
                matches!(
                    target.as_str(),
                    "print_i64" | "print_u64" | "print_bytes" | "print_f64"
                ),
                "unresolved call in {}",
                self.path
            );
            if target == "print_f64" && !self.ghost {
                let d = argv[1].s();
                self.oblige(
                    "E0801",
                    span,
                    "`print_f64` may get more than 65535 decimals (Rust panics)",
                    format!("(<= {d} 65535)"),
                );
            }
            return Val::Unit;
        };
        let f = def.f;
        let pre_env = bind_params(f, &argv);
        let unit = def.mod_path.clone();
        let param_tys: Vec<TypeKind> = self.prog.t.fns[&target]
            .params
            .iter()
            .map(|t| crate::check::subst(t, &csub))
            .collect();
        // Every argument leaves the caller's hands valid (ADR-0060 §2).
        for ((v, t), a) in argv.iter().zip(&param_tys).zip(args) {
            self.oblige_valid(a.span, Boundary::Arg(a, &target), v, t);
        }
        // The lemma is proved for values of its parameter types; ghost
        // arithmetic is mathematical, so an argument may lie outside
        // (E0819, ADR-0062 §2).
        if f.kind == FnKind::Proof {
            let tys: Vec<TypeKind> = self.prog.t.fns[&target]
                .params
                .iter()
                .map(|t| crate::check::subst(t, &csub))
                .collect();
            for ((v, t), a) in argv.iter().zip(&tys).zip(args) {
                for g in self.type_facts(v, t) {
                    if self.pc.contains(&g) {
                        continue;
                    }
                    self.oblige(
                        "E0819",
                        a.span,
                        format!(
                            "argument `{}` of `{target}` may be outside `{}`",
                            crate::fmt::expr_text(a),
                            crate::check::ty_str(t)
                        ),
                        g,
                    );
                }
            }
        }
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
                // The callee left it valid.
                self.assume_valid(&nv, &param_tys[i]);
                let cur = self.vars[&root].clone();
                let whole = self.set_at(cur, &steps, nv.clone());
                self.vars.insert(root, whole);
                post_args[i] = nv;
            }
        }
        let mut post_env = bind_params(f, &post_args);
        let ret_ty = crate::check::subst(&self.prog.t.fns[&target].ret, &csub);
        let result = self.fresh_val(&format!("{target}.result"), &ret_ty);
        self.assume_valid(&result, &ret_ty);
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
    match t {
        TypeKind::Bool => "false",
        TypeKind::F64 => "(_ +zero 11 53)",
        _ => "0",
    }
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
            // The variable's type is in the checker's table of locals.
            Stmt::For {
                lo,
                hi,
                invariants,
                body,
                ..
            } => {
                find_let_expr(lo, name, out);
                find_let_expr(hi, name, out);
                invariants.iter().for_each(|e| find_let_expr(e, name, out));
                find_let(body, name, out);
            }
            Stmt::Assert { cond, .. } => find_let_expr(cond, name, out),
            Stmt::LetFields { init, .. } => find_let_expr(init, name, out),
            Stmt::Expr(e) => find_let_expr(e, name, out),
            Stmt::Proof { body, .. } => find_let(body, name, out),
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
        ExprKind::Field { base, .. } | ExprKind::Deref(base) | ExprKind::Try(base) => {
            find_let_expr(base, name, out)
        }
        ExprKind::StructLit { fields, .. } => {
            fields.iter().for_each(|(_, x)| find_let_expr(x, name, out))
        }
        ExprKind::Match { scrut, arms } => {
            find_let_expr(scrut, name, out);
            arms.iter().for_each(|a| find_let_expr(&a.body, name, out));
        }
        ExprKind::Return(x) | ExprKind::Break { value: x, .. } => {
            x.iter().for_each(|x| find_let_expr(x, name, out))
        }
        ExprKind::Loop { body, .. } => find_let(body, name, out),
        ExprKind::Continue { .. } => {}
        ExprKind::Int(..)
        | ExprKind::ByteLit(_)
        | ExprKind::Float(_)
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
            // The loop variable is local to the loop (and stepped by it).
            Stmt::For {
                var, lo, hi, body, ..
            } => {
                assigned_in_expr(lo, out);
                assigned_in_expr(hi, out);
                assigned_in_block(body, out);
                out.insert(var.name.clone());
            }
            Stmt::Expr(e) => assigned_in_expr(e, out),
            // Ghost code assigns nothing (E0257).
            Stmt::Assert { .. } | Stmt::Proof { .. } => {}
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
        // A `&mut` match writes its scrutinee's place through the bindings
        // (ADR-0055); the arms may assign anything.
        ExprKind::Match { scrut, arms } => {
            if let ExprKind::Ref {
                mutable: true,
                expr,
            } = &scrut.kind
                && let Some(r) = root_var(expr)
            {
                out.insert(r);
            }
            // `match e` on a `&mut` parameter (`?e`: kept when it is one).
            if let ExprKind::Var(id) = &scrut.kind {
                out.insert(format!("?{}", id.name));
            }
            assigned_in_expr(scrut, out);
            arms.iter().for_each(|a| assigned_in_expr(&a.body, out));
        }
        ExprKind::Loop { body, .. } => assigned_in_block(body, out),
        ExprKind::Return(Some(x))
        | ExprKind::Break { value: Some(x), .. }
        | ExprKind::Field { base: x, .. }
        | ExprKind::Deref(x)
        | ExprKind::Try(x)
        | ExprKind::Repeat { value: x, .. } => assigned_in_expr(x, out),
        ExprKind::Array(items) => items.iter().for_each(|a| assigned_in_expr(a, out)),
        ExprKind::StructLit { fields, .. } => {
            fields.iter().for_each(|(_, a)| assigned_in_expr(a, out))
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
    /// The facts of the recursive struct fields `text` reads outside a
    /// quantifier (backlog 2O-03d): each selector application's range or
    /// length bounds, which every program value meets. Ground, so the
    /// queries stay quantifier-free. `text` is an obligation's path
    /// condition and goal, never the definitions of the values `mk_struct`
    /// made (`decls`): those include values made later or on another path,
    /// and the facts of their fields would assume what the obligations on
    /// those fields prove (a soundness fault the owned-data fuzzer found).
    /// A term of the path condition denotes a value that exists on the path,
    /// whose fields the obligations before it checked.
    fn struct_facts(&self, text: &str) -> String {
        fn walk(e: &smt::Sexp, sels: &BTreeMap<String, Leaf>, out: &mut BTreeSet<String>) {
            if let smt::Sexp::List(xs) = e {
                if let [smt::Sexp::Atom(h), _] = xs.as_slice()
                    && let Some(leaf) = sels.get(h)
                    && let Some(f) = leaf.fact(&smt::print(e))
                {
                    out.insert(f);
                }
                if !matches!(xs.first(), Some(smt::Sexp::Atom(q)) if q == "forall" || q == "exists")
                {
                    xs.iter().for_each(|x| walk(x, sels, out));
                }
            }
        }
        if self.struct_sels.is_empty() {
            return String::new();
        }
        let mut facts = BTreeSet::new();
        for cmd in smt::split_commands(text) {
            if let Some(e) = smt::parse(&cmd) {
                walk(&e, &self.struct_sels, &mut facts);
            }
        }
        facts.iter().map(|f| format!("(assert {f})\n")).collect()
    }

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

    /// Definition of a revealed spec fn (ADR-0017 §4). A recursive one is a
    /// quantified axiom with its application as the pattern. Any other is a
    /// macro, `(define-fun f (..) (ite guard body (f!ext ..)))`: the same
    /// meaning (the body where the parameters are in their types, an
    /// unconstrained value elsewhere) with no quantifier, so a satisfiable
    /// query needs no model search over it (backlog T2-04: a quantified
    /// definition over an `(Array Int Float64)` parameter kept Z3 from
    /// ending a satisfiable query, `incomplete (theory array)`).
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
        // An instance of a generic spec fn is recursive when its generic is.
        let base = self.t.insts.get(path).map_or(path, |(b, _)| b.as_str());
        let recursive = self.t.recursive.contains_key(base);
        // Only a leaf (a body that names no other spec fn) becomes a macro:
        // Z3 expands every macro use in place, so nested macros multiply the
        // term size (SHA-256's round functions over its sigma functions).
        let leaf = !bodies.iter().any(|b| {
            self.t.fns.iter().any(|(q, sig)| {
                q != path && sig.kind == FnKind::Spec && b.contains(&spec_key(q, self.t))
            })
        });
        // One definition per SMT function: a struct result has one per leaf.
        let mut quant = Vec::new();
        let mut mac = Vec::new();
        for ((f, sort), body) in spec_funs(path, self.t).iter().zip(&bodies) {
            let app = if app_args.is_empty() {
                f.clone()
            } else {
                format!("({f} {})", app_args.join(" "))
            };
            if binders.is_empty() {
                quant.push(format!("(assert (= {app} {body}))"));
                continue;
            }
            quant.push(format!(
                "(assert (forall ({}) (! (=> {guard} (= {app} {body})) :pattern ({app}))))",
                binders.join(" "),
            ));
            let value = if guard == "true" {
                body.clone()
            } else {
                let ext = macro_ext(f);
                let sorts: Vec<&str> = binders
                    .iter()
                    .map(|b| b[1..b.len() - 1].split_once(' ').map_or("", |(_, s)| s))
                    .collect();
                mac.push(format!("(declare-fun {ext} ({}) {sort})", sorts.join(" ")));
                format!("(ite {guard} {body} ({ext} {}))", app_args.join(" "))
            };
            mac.push(format!(
                "(define-fun {f} ({}) {sort} {value})",
                binders.join(" ")
            ));
        }
        // A macro keeps its quantified form after the marker: a query whose
        // patterns name the spec fn takes that instead (`relevant_axioms`).
        // Only a definition that binds an array (a slice or `Vec` parameter)
        // becomes a macro: Z3's model search fails on quantifiers over array
        // sorts (`incomplete (theory array)`), and over scalars a quantified
        // definition is fine and leaves nonlinear proofs the search they had.
        let binds_array = binders.iter().any(|b| b.contains("(Array "));
        let text = if !recursive && leaf && binds_array {
            format!("{}{QUANTIFIED}{}", mac.join("\n"), quant.join("\n"))
        } else {
            quant.join("\n")
        };
        // Ground instances (ADR-0023) cover single-result spec fns.
        let parts = AxiomParts {
            sym: spec_key(path, self.t),
            params: app_args,
            guard,
            body: bodies.first().cloned().unwrap_or_default(),
        };
        (text, parts)
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
/// `rlimit`); the datatypes the query names, the prelude and the
/// declarations of the spec fns it mentions follow (`needed_decls`).
pub fn query_header() -> String {
    format!("{}\n{PRELUDE}", header(0).join("\n"))
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
        // A revealed non-recursive spec fn is defined by its macro instead.
        if text.contains(s.as_str()) && !text.contains(&format!("(define-fun {s}")) {
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
    // The prelude after the datatypes: Z3 4.15.4 does not end a satisfiable
    // query (and ignores `rlimit`) when a datatype nested in an array is
    // declared after the prelude's `div` macros (backlog 2O-03c,
    // `dev-docs/measurements/z3-nested-datatypes`).
    // The bit-vector functions of the bit methods the query applies (none
    // in a query without them: its text is unchanged).
    // The `f64` operations the query applies, uninterpreted (ADR-0061 §5;
    // none in a query without them).
    format!(
        "{head}{PRELUDE}\n{}{}{out}",
        crate::subproof::helper_defs(text),
        f64_decls(text, false)
    )
}

/// Whether the `requires` clauses of every executable function that has
/// any can hold together (backlog 2D-08). The query is the prover's own
/// encoding of the function's entry state: parameter range facts and type
/// invariants (ADR-0060), the definitions its `#[uses]` reveals, and the
/// clauses, under the pinned seed
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
            // The entry state the prover assumes: valid parameters too.
            enc.assume_valid(&v, &ty);
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
        let decls = enc.decls.join("\n");
        let q = enc
            .pc
            .iter()
            .map(|a| format!("(assert {a})"))
            .collect::<Vec<_>>()
            .join("\n");
        let q = valid_instances(&enc.valid_defs, q);
        let q = format!("{q}\n{}", prog.struct_facts(&q));
        let relevant = relevant_axioms(&axioms, &q);
        let mut base = header(0).join("\n");
        base.push('\n');
        base.push_str(&sort_decls(&t.fns[path]));
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
            let inst = format!("{inst}{}", prog.struct_facts(&inst));
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
        let hit = incr.as_ref().and_then(|s| s.proof_lookup(path, d.f));
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

        let job = REFINE.with(|r| r.borrow().as_ref().map(|x| x.jobs.get(path).cloned()));
        match (job, d.f.kind) {
            // Under `contract_refinements`: only the listed definitions,
            // each its comparison on the entry state of its parameters (as
            // an executable function's), no body.
            (Some(None), _) => continue,
            (Some(Some(job)), _) => {
                for u in prog.revealed(d.f, &d.mod_path) {
                    let (ax, parts) = prog.axiom(&u);
                    axioms.push((spec_key(&u, t), ax));
                    axiom_parts.push(parts);
                }
                for prm in &d.f.params {
                    let ty = enc.local_ty(&prm.name.name);
                    let v = enc.fresh_val(&prm.name.name, &ty);
                    enc.assume_valid(&v, &ty);
                    if let TypeKind::Ref { mutable: true, .. } = prm.ty.kind {
                        enc.old.insert(prm.name.name.clone(), v.clone());
                    }
                    enc.inputs.push((prm.name.name.clone(), v.clone(), ty));
                    enc.vars.insert(prm.name.name.clone(), v);
                }
                let at = RefineAt::Version {
                    own_domain: job.own_domain,
                };
                enc.refines(d.f, &job.decl, job.tsub, at);
                let made = enc.obls.iter().map(|o| (o.code, o.span)).collect();
                REFINE.with(|r| {
                    if let Some(x) = r.borrow_mut().as_mut() {
                        x.made.push((path.clone(), made));
                    }
                });
            }
            (None, FnKind::Spec) => {
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
            // A lemma (ADR-0062 §2): its body is ghost code, proved like an
            // executable function's against its contract.
            (None, FnKind::Proof) => {
                for u in prog.revealed(d.f, &d.mod_path) {
                    let (ax, parts) = prog.axiom(&u);
                    axioms.push((spec_key(&u, t), ax));
                    axiom_parts.push(parts);
                }
                // Each parameter satisfies what every call proves of its
                // argument (`type_facts`), and nothing more.
                for prm in &d.f.params {
                    let ty = enc.local_ty(&prm.name.name);
                    let n = enc.pc.len();
                    let v = enc.fresh_val(&prm.name.name, &ty);
                    enc.pc.truncate(n);
                    let facts = enc.type_facts(&v, &ty);
                    enc.pc.extend(facts);
                    enc.inputs.push((prm.name.name.clone(), v.clone(), ty));
                    enc.vars.insert(prm.name.name.clone(), v);
                }
                let (env, old) = enc.cur_env();
                for r in &d.f.requires {
                    let g = enc.spec(r, &env, &old).s();
                    enc.pc.push(g);
                }
                if let Some(dec) = &d.f.decreases {
                    enc.entry_measure = Some(enc.measure(dec, &env, &old));
                }
                enc.ghost = true;
                enc.block(&d.f.body);
                enc.ghost = false;
                enc.mark_reach(d.f.name.span, "end of the body");
                enc.post_check(Val::Unit, d.f.name.span, "at the end of the body");
            }
            (None, FnKind::Exec) => {
                for u in prog.revealed(d.f, &d.mod_path) {
                    let (ax, parts) = prog.axiom(&u);
                    axioms.push((spec_key(&u, t), ax));
                    axiom_parts.push(parts);
                }
                for prm in &d.f.params {
                    let ty = enc.local_ty(&prm.name.name);
                    let v = enc.fresh_val(&prm.name.name, &ty);
                    // A parameter is valid (ADR-0060 §2): its caller proved it.
                    enc.assume_valid(&v, &ty);
                    if let TypeKind::Ref { mutable: true, .. } = prm.ty.kind {
                        enc.old.insert(prm.name.name.clone(), v.clone());
                    }
                    enc.inputs.push((prm.name.name.clone(), v.clone(), ty));
                    enc.vars.insert(prm.name.name.clone(), v);
                }
                if let Some((decl, rsub)) = refinement_of(t, path) {
                    enc.refines(d.f, &decl, rsub, RefineAt::Trait);
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
                if !enc.dead {
                    enc.mark_reach(d.f.name.span, "end of the body");
                    let at = d.f.body.tail.as_ref().map_or(d.f.name.span, |t| t.span);
                    enc.post_check(result, at, "at the end of the body");
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
                let q = valid_instances(&enc.valid_defs, q);
                let q = format!("{q}\n{}", prog.struct_facts(&q));
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
            if let Some(sub) = &o.sub {
                let key = (o.code, o.span.file, o.span.lo, o.span.hi);
                let q = match sub {
                    Ok(q) => q,
                    Err((why, hidden)) => {
                        failed += 1;
                        failed_keys.insert(key);
                        let d = Diag::new(
                            "E0818",
                            o.span,
                            format!(
                                "the assertion has no bit-vector translation in `{path}`: {why}"
                            ),
                        );
                        // An unrevealed spec fn: the hint `bmb uses` reads.
                        diags.push(if hidden.is_empty() {
                            d.fix(
                                "a bit-vector sub-proof takes integers, `f64`s and `bool`s, bit operations, `+`, `-`, `*`, division by positive constants (integers), the `f64` operations, comparisons, and calls of non-recursive spec fns named in `#[uses(..)]`",
                                None,
                            )
                        } else {
                            with_hint(d, hidden, false)
                        });
                        continue;
                    }
                };
                let base = format!("{}\n{}", header.join("\n"), q.base());
                let neg = q.negated_goal();
                script.push_str(&format!(
                    "; obligation {k}: {} {} (sub-proof)\n(reset)\n{base}{neg}\n(check-sat)\n",
                    o.code, o.msg
                ));
                let cache_key = cache
                    .as_ref()
                    .map(|_| ProofCache::key(&format!("{base}\n{neg}")));
                if let (Some(c), Some(ck)) = (cache.as_mut(), cache_key.as_ref())
                    && let Some((total, max, _)) = c.get(ck)
                {
                    rlimit_total += total;
                    rlimit_max = rlimit_max.max(max);
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
                let answer = sol
                    .run("(reset)")
                    .and_then(|_| sol.run(&base))
                    .and_then(|_| sol.run(&neg))
                    .and_then(|_| sol.check_sat());
                let mut used = 0;
                if answer.is_ok()
                    && let Ok(u) = sol.rlimit_used()
                {
                    used = u;
                    rlimit_total += u;
                    rlimit_max = rlimit_max.max(u);
                }
                match answer {
                    Ok(Answer::Unsat) => {
                        proved_keys.insert(key);
                        def_proved.push(key);
                        if let (Some(c), Some(ck)) = (cache.as_mut(), cache_key) {
                            c.insert(ck, used, used, false);
                        }
                        continue;
                    }
                    Ok(Answer::Sat) => {
                        // `f64` operations are defined here, but `min` and
                        // `max` of zeros of both signs are not (G1).
                        let checked = f64_model_check(sol, &format!("{base}{neg}"));
                        let cex = match checked {
                            Ok(()) => sub_counterexample(sol, q),
                            Err(_) => String::new(),
                        };
                        let mut msg = format!("{} in `{path}`", o.msg);
                        if !cex.is_empty() {
                            msg.push_str(&format!("; counterexample: {cex}"));
                        }
                        diags.push(Diag::new("E0818", o.span, msg).fix(
                            "the sub-proof sees only the assertion's variables, each over its whole type, and no fact of the path: write what it needs into the assertion as `p ==> e`",
                            None,
                        ));
                    }
                    Ok(Answer::Unknown(why)) => {
                        let why = if why.contains("canceled") || why.contains("resource") {
                            format!("resource limit reached, rlimit {}", smt::rlimit())
                        } else {
                            why
                        };
                        diags.push(Diag::new(
                            "E0811",
                            o.span,
                            format!("{} in `{path}`: solver could not decide ({why})", o.msg),
                        ));
                    }
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
                failed += 1;
                failed_keys.insert(key);
                continue;
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
            let q = valid_instances(&enc.valid_defs, q);
            let q = format!("{q}\n{}", prog.struct_facts(&q));
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
            let key = (o.code, o.span.file, o.span.lo, o.span.hi);
            let cache_key = cache
                .as_ref()
                .map(|_| ProofCache::key(&format!("{base}\n{q}")));
            // The ground-instance retry of an obligation: the same text
            // whether it runs or comes from the proof cache (2V-03).
            let retry_script = |q: &str| {
                let (inst, complete) = ground_instances(&axiom_parts, q, 3);
                let inst = format!("{inst}{}", prog.struct_facts(&inst));
                if inst.is_empty() {
                    return None;
                }
                let used = needed_decls(&spec_decls, &dts, &format!("{decls}\n{inst}{q}"));
                Some((format!("{prefix}{used}{decls}"), inst, complete))
            };
            // Bit operations abstracted (ADR-0048 §4): their range facts
            // decide most obligations, and the solver does not bit-blast a
            // definition it does not need. It is tried first; proved there is
            // proved, anything else goes on to the exact query. With
            // quantified definitions their ground instances stand in for them
            // (as in the retry, sound for a proof): Z3's model-based
            // instantiation did not finish there.
            let abstracted = if q.contains("(bv2nat ") {
                let (head, body) = match retry_script(&q) {
                    Some((head, inst, _)) if !axiom_parts.is_empty() => {
                        (head, format!("{inst}{q}"))
                    }
                    _ => (base.clone(), q.clone()),
                };
                abstract_bits(&head, &body).map(|aq| (head, aq))
            } else {
                None
            };
            match &abstracted {
                Some((head, aq)) => script.push_str(&format!(
                    "; obligation {k}: {} {} (bit operations abstracted)\n(reset)\n{head}\n{aq}\n(check-sat)\n; exact query of obligation {k}\n(reset)\n{base}\n{q}\n(check-sat)\n",
                    o.code, o.msg
                )),
                None => script.push_str(&format!(
                    "; obligation {k}: {} {}\n(reset)\n{base}\n{q}\n(check-sat)\n",
                    o.code, o.msg
                )),
            }
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
            let mut proved_abstract = None;
            if let Some((head, aq)) = &abstracted {
                let a = sol
                    .run("(reset)")
                    .and_then(|_| sol.run(head))
                    .and_then(|_| sol.run(aq))
                    .and_then(|_| sol.check_sat());
                if a.is_ok()
                    && let Ok(used) = sol.rlimit_used()
                {
                    rlimit_total += used;
                    rlimit_max = rlimit_max.max(used);
                    first_used = used;
                }
                if matches!(a, Ok(Answer::Unsat)) {
                    proved_abstract = Some(a);
                }
            }
            let mut answer = match proved_abstract {
                Some(a) => a,
                None => {
                    let answer = sol
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
                    answer
                }
            };
            // What the solver holds when it answers (the model's query).
            let mut answered = format!("{base}\n{q}");
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
                    Ok(Answer::Sat) if complete => {
                        answer = retry;
                        answered = format!("{head}\n{inst}{q}");
                    }
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
                    // An `f64` model is shown only when IEEE arithmetic
                    // agrees with it (G1), else one is searched for with
                    // IEEE arithmetic (ADR-0061 §5, backlog T2-03).
                    let confirmed = match f64_model_check(sol, &answered) {
                        Ok(()) => Ok(()),
                        Err(why) => {
                            let exact = f64_interpreted(&answered);
                            let a = sol
                                .run("(reset)")
                                .and_then(|_| sol.run(&exact))
                                .and_then(|_| sol.check_sat());
                            match a {
                                Ok(Answer::Sat) => f64_model_check(sol, &exact),
                                Ok(Answer::Unsat) => Err(
                                    "it holds with IEEE 754 arithmetic, which only `by (bit_vector)` uses"
                                        .to_string(),
                                ),
                                _ => Err(why),
                            }
                        }
                    };
                    match confirmed {
                        Ok(()) => {
                            let cex = counterexample(sol, &shown_inputs(&enc.inputs, o), t);
                            let mut msg = format!("{} in `{path}`", o.msg);
                            if !cex.is_empty() {
                                msg.push_str(&format!("; counterexample: {cex}"));
                            }
                            diags.push(with_hint(Diag::new(o.code, o.span, msg), &hint, false));
                        }
                        Err(why) => diags.push(f64_undecided(o, path, &why)),
                    }
                }
                Ok(Answer::Unknown(why)) => {
                    let why = if why.contains("canceled") || why.contains("resource") {
                        format!("resource limit reached, rlimit {}", smt::rlimit())
                    } else {
                        why
                    };
                    let mut msg = format!("{} in `{path}`: solver could not decide ({why})", o.msg);
                    // Z3 often keeps a candidate model; show it, marked as
                    // unconfirmed (and only when its `f64` values are IEEE's).
                    let cex = match f64_model_check(sol, &answered) {
                        Ok(()) => counterexample(sol, &shown_inputs(&enc.inputs, o), t),
                        Err(_) => String::new(),
                    };
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
            s.proof_record(&status, d.f, &def_proved, &script);
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

/// Query `q` (assertions) with each ground bit operation term
/// `(bv2nat ..)` replaced by an integer constant `|bv#n|` (the same term by
/// the same constant), declared first; `None` when `q` has none. Facts about
/// a term stay as facts about its constant, and only its bitvector
/// definition is dropped, so the abstracted query is weaker: unsatisfiable
/// means the obligation holds (ADR-0048 §4). A term under a quantifier that
/// mentions a bound variable is kept as it is (a constant cannot stand for
/// it). Terms that compute the same thing in different notation are related
/// by congruence facts (`bit_congruences`; `head` declares the constants).
fn abstract_bits(head: &str, q: &str) -> Option<String> {
    use smt::Sexp::{Atom, List};
    fn mentions(e: &smt::Sexp, bound: &[String]) -> bool {
        match e {
            Atom(a) => bound.contains(a),
            List(xs) => xs.iter().any(|x| mentions(x, bound)),
        }
    }
    fn walk(e: &smt::Sexp, bound: &mut Vec<String>, names: &mut Vec<String>) -> smt::Sexp {
        let List(xs) = e else { return e.clone() };
        let head = match xs.first() {
            Some(Atom(h)) => h.as_str(),
            _ => "",
        };
        match head {
            "bv2nat" if !mentions(e, bound) => {
                let text = smt::print(e);
                let k = match names.iter().position(|n| *n == text) {
                    Some(k) => k,
                    None => {
                        names.push(text);
                        names.len() - 1
                    }
                };
                Atom(format!("|bv#{k}|"))
            }
            "forall" | "exists" | "let" if xs.len() == 3 => {
                let List(bs) = &xs[1] else { return e.clone() };
                let vars: Vec<String> = bs
                    .iter()
                    .filter_map(|b| match b {
                        List(p) => match p.first() {
                            Some(Atom(v)) => Some(v.clone()),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                // `let` binds over its body only.
                let binders = if head == "let" {
                    List(
                        bs.iter()
                            .map(|b| match b {
                                List(p) if p.len() == 2 => {
                                    List(vec![p[0].clone(), walk(&p[1], bound, names)])
                                }
                                other => other.clone(),
                            })
                            .collect(),
                    )
                } else {
                    xs[1].clone()
                };
                let n = bound.len();
                bound.extend(vars);
                let body = walk(&xs[2], bound, names);
                bound.truncate(n);
                List(vec![xs[0].clone(), binders, body])
            }
            _ => List(xs.iter().map(|x| walk(x, bound, names)).collect()),
        }
    }
    if !q.contains("(bv2nat ") {
        return None;
    }
    let mut names = Vec::new();
    let mut out = String::new();
    let mut cmds = Vec::new();
    for c in smt::split_commands(q) {
        let e = smt::parse(&c)?;
        out.push_str(&smt::print(&walk(&e, &mut Vec::new(), &mut names)));
        out.push('\n');
        cmds.push(e);
    }
    if names.is_empty() {
        return None;
    }
    // The congruences go through the same abstraction: their terms are
    // among `names`, and their leaf values may add more.
    let terms = names.clone();
    for fact in bit_congruences(head, &cmds, &terms) {
        let e = smt::parse(&fact)?;
        out.push_str(&smt::print(&walk(&e, &mut Vec::new(), &mut names)));
        out.push('\n');
    }
    let decls: String = (0..names.len())
        .map(|k| format!("(declare-const |bv#{k}| Int)\n"))
        .collect();
    Some(format!("{decls}{out}"))
}

/// Congruence facts (assertions) between the abstracted ground terms
/// `terms` (each a `(bv2nat ..)`) of the query `cmds` that apply the same
/// operations to the same values written differently (backlog 1B-49). Code
/// names each operand of a bit operation by a bitvector constant with a
/// stated value (`Enc::operand_bv`: `(= (bv2nat |'bv@k|) v)` on its path), a
/// specification converts its operand (`((_ int2bv w) v)`), and a literal is
/// a bitvector numeral. Without these facts the abstraction gives the
/// code's `x ^ k` and a specification's `x ^ k` two unrelated constants,
/// and only the exact query relates them, through `int2bv`, which Z3 did not
/// finish for one 32-bit `^` (rlimit 5000000).
///
/// Two terms whose trees agree once each leaf is read as its width and its
/// value (a constant's stated value; in a value, terms are read the same
/// way) get `(=> (and (= a1 b1) ..) (= t1 t2))` over the leaves that
/// differ. A leaf's side is `(bv2nat c)` for a constant `c`, `v` for
/// `((_ int2bv w) v)` (`(mod v 2^w)` against a constant whose stated value
/// is the two's complement form, `bv_value`) and `n` for a numeral. Each
/// fact holds in every model: `bv2nat` is one-to-one on a width, a value
/// equal to a `bv2nat` is in range, where `int2bv` inverts it, and
/// `(mod v 2^w)` is the `bv2nat` of `((_ int2bv w) v)`. So the abstracted
/// query with them stays weaker than the exact query (ADR-0048 §4).
fn bit_congruences(head: &str, cmds: &[smt::Sexp], terms: &[String]) -> Vec<String> {
    use smt::Sexp::{Atom, List};
    let mut width: BTreeMap<String, u32> = BTreeMap::new();
    for c in smt::split_commands(head) {
        if let Some(List(xs)) = smt::parse(&c)
            && let [Atom(d), Atom(name), List(sort)] = xs.as_slice()
            && d == "declare-const"
            && name.starts_with("|'bv@")
            && let [Atom(u), Atom(bv), Atom(w)] = sort.as_slice()
            && u == "_"
            && bv == "BitVec"
            && let Ok(w) = w.parse()
        {
            width.insert(name.clone(), w);
        }
    }
    // The stated value of each constant (one per constant, `operand_bv`).
    fn defs(e: &smt::Sexp, out: &mut BTreeMap<String, smt::Sexp>) {
        let List(xs) = e else { return };
        if let [Atom(eq), List(l), v] = xs.as_slice()
            && eq == "="
            && let [Atom(b), Atom(c)] = l.as_slice()
            && b == "bv2nat"
            && c.starts_with("|'bv@")
        {
            out.entry(c.clone()).or_insert_with(|| v.clone());
        }
        for x in xs {
            defs(x, out);
        }
    }
    let mut value: BTreeMap<String, smt::Sexp> = BTreeMap::new();
    for c in cmds {
        defs(c, &mut value);
    }
    let cx = Congr { width, value };
    // The terms and the terms nested in them (a specification's `(a ^ b) |
    // c` converts the inner term's value), inner ones first.
    fn nested(e: &smt::Sexp, seen: &mut BTreeSet<String>, out: &mut Vec<smt::Sexp>) {
        let List(xs) = e else { return };
        for x in xs {
            nested(x, seen, out);
        }
        if matches!(xs.first(), Some(Atom(h)) if h == "bv2nat") && seen.insert(smt::print(e)) {
            out.push(e.clone());
        }
    }
    let mut all = Vec::new();
    let mut seen = BTreeSet::new();
    for t in terms {
        if let Some(e) = smt::parse(t) {
            nested(&e, &mut seen, &mut all);
        }
    }
    // Terms by their reading; the first of each reading is the one the
    // others are equated with. The firsts of different readings of one
    // shape (the tree with each leaf read as its width only) are related
    // pairwise, under the equality of the leaves that differ (backlog
    // 2N-03: a table entry `t[i]` and the spec fn value it equals,
    // `byte(i, 0)`, are different leaves of one `^`).
    let mut first: BTreeMap<String, smt::Sexp> = BTreeMap::new();
    let mut shapes: BTreeMap<String, Vec<smt::Sexp>> = BTreeMap::new();
    let mut facts = Vec::new();
    let fact = |a: &smt::Sexp, b: &smt::Sexp, facts: &mut Vec<String>| {
        let mut eqs = Vec::new();
        if !cx.leaf_pairs(a, b, &mut eqs) {
            return;
        }
        let ante = match eqs.len() {
            0 => "true".to_string(),
            1 => eqs[0].clone(),
            _ => format!("(and {})", eqs.join(" ")),
        };
        facts.push(format!(
            "(assert (=> {ante} (= {} {})))",
            smt::print(a),
            smt::print(b)
        ));
    };
    for e in all {
        let List(xs) = &e else { continue };
        let Some(bv) = xs.get(1) else { continue };
        // A constant's own `bv2nat` is a leaf's value, not an operation.
        if cx.leaf(bv).is_some() {
            continue;
        }
        let Some(key) = cx.read(&e, true) else {
            continue;
        };
        match first.get(&key) {
            None => {
                if let Some(shape) = cx.read(&e, false) {
                    let reps = shapes.entry(shape).or_default();
                    if reps.len() < MAX_SHAPE_PAIRS {
                        for r in reps.iter() {
                            fact(r, &e, &mut facts);
                        }
                        reps.push(e.clone());
                    }
                }
                first.insert(key, e);
            }
            Some(f) => fact(f, &e, &mut facts),
        }
    }
    facts
}

/// The most terms of one shape `bit_congruences` relates pairwise.
const MAX_SHAPE_PAIRS: usize = 64;

/// The reading of bitvector terms behind `bit_congruences`.
struct Congr {
    /// Width of each operand constant.
    width: BTreeMap<String, u32>,
    /// Stated value of each operand constant.
    value: BTreeMap<String, smt::Sexp>,
}

/// The kind of a bitvector leaf.
enum BvLeaf {
    /// A code operand constant; `signed` when its stated value is the two's
    /// complement form `(ite (< v 0) (+ v 2^w) v)` (the leaf's value is `v`).
    Const { name: String, signed: bool },
    /// `((_ int2bv w) v)`.
    Conv,
    /// `(_ bvN w)`.
    Num,
}

impl Congr {
    /// `e` as a leaf: its kind, width and value.
    fn leaf(&self, e: &smt::Sexp) -> Option<(BvLeaf, u32, smt::Sexp)> {
        use smt::Sexp::{Atom, List};
        let is = |e: &smt::Sexp, s: &str| matches!(e, Atom(a) if a == s);
        match e {
            Atom(a) if a.starts_with("|'bv@") => {
                let w = *self.width.get(a)?;
                let v = self.value.get(a)?;
                // `bv_value` of a signed operand.
                if let List(xs) = v
                    && let [ite, List(c), List(p), x] = xs.as_slice()
                    && is(ite, "ite")
                    && c.len() == 3
                    && is(&c[0], "<")
                    && c[1] == *x
                    && is(&c[2], "0")
                    && p.len() == 3
                    && is(&p[0], "+")
                    && p[1] == *x
                    && is(&p[2], &num(1i128 << w))
                {
                    let name = a.clone();
                    return Some((BvLeaf::Const { name, signed: true }, w, x.clone()));
                }
                let name = a.clone();
                Some((
                    BvLeaf::Const {
                        name,
                        signed: false,
                    },
                    w,
                    v.clone(),
                ))
            }
            List(xs) => match xs.as_slice() {
                [List(f), v] if f.len() == 3 && is(&f[0], "_") && is(&f[1], "int2bv") => {
                    let Atom(w) = &f[2] else { return None };
                    Some((BvLeaf::Conv, w.parse().ok()?, v.clone()))
                }
                [u, Atom(n), Atom(w)] if is(u, "_") && n.starts_with("bv") => {
                    let n: i128 = n[2..].parse().ok()?;
                    Some((BvLeaf::Num, w.parse().ok()?, Atom(num(n))))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// The reading of a bitvector term: leaves as width and read value (as
    /// width alone without `values`: the term's shape), operations as they
    /// are. `None` for a form this does not know.
    fn read_bv(&self, e: &smt::Sexp, values: bool) -> Option<String> {
        use smt::Sexp::{Atom, List};
        if let Some((_, w, v)) = self.leaf(e) {
            if !values {
                return Some(format!("[{w}]"));
            }
            return Some(format!("[{w} {}]", self.read(&v, true)?));
        }
        let List(xs) = e else { return None };
        let Some(Atom(op)) = xs.first() else {
            return None;
        };
        // A bit-vector operation, or a bit method's named function
        // (`subproof::helper_def`).
        if !(op.starts_with("bv") || op.starts_with("bmb_")) || xs.len() < 2 {
            return None;
        }
        let args: Option<Vec<String>> = xs[1..].iter().map(|x| self.read_bv(x, values)).collect();
        Some(format!("({op} {})", args?.join(" ")))
    }

    /// The reading of an integer term: each `(bv2nat t)` in it by the
    /// reading of `t`, everything else as it is.
    fn read(&self, e: &smt::Sexp, values: bool) -> Option<String> {
        use smt::Sexp::{Atom, List};
        match e {
            Atom(a) => Some(a.clone()),
            List(xs) => {
                if let [Atom(b), t] = xs.as_slice()
                    && b == "bv2nat"
                {
                    return Some(format!("(bv2nat {})", self.read_bv(t, values)?));
                }
                let parts: Option<Vec<String>> = xs.iter().map(|x| self.read(x, values)).collect();
                Some(format!("({})", parts?.join(" ")))
            }
        }
    }

    /// The integer side of a leaf against the leaf `other` it is paired with.
    fn side(mine: &(BvLeaf, u32, smt::Sexp), other: &BvLeaf) -> String {
        match (mine, other) {
            ((BvLeaf::Const { name, .. }, _, _), _) => format!("(bv2nat {name})"),
            ((_, w, v), BvLeaf::Const { signed: true, .. }) => {
                format!("(mod {} {})", smt::print(v), num(1i128 << w))
            }
            ((_, _, v), _) => smt::print(v),
        }
    }

    /// The equalities of the leaves where two terms of the same reading
    /// differ; false when their shapes do not match.
    fn leaf_pairs(&self, a: &smt::Sexp, b: &smt::Sexp, eqs: &mut Vec<String>) -> bool {
        use smt::Sexp::List;
        if a == b {
            return true;
        }
        match (self.leaf(a), self.leaf(b)) {
            (Some(la), Some(lb)) => {
                let x = Self::side(&la, &lb.0);
                let y = Self::side(&lb, &la.0);
                if x != y {
                    eqs.push(format!("(= {x} {y})"));
                }
                true
            }
            (None, None) => match (a, b) {
                (List(xs), List(ys))
                    if xs.len() == ys.len() && !xs.is_empty() && xs[0] == ys[0] =>
                {
                    xs[1..]
                        .iter()
                        .zip(&ys[1..])
                        .all(|(x, y)| self.leaf_pairs(x, y, eqs))
                }
                _ => false,
            },
            _ => false,
        }
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
    // A spec fn named in a pattern must stay uninterpreted: Z3 expands a
    // macro inside a pattern, which leaves no usable trigger (ADR-0064). The
    // patterns are those of the query and of the chosen definitions' bodies.
    let mut seen = text.to_string();
    for (k, (_, ax)) in axioms.iter().enumerate() {
        if chosen[k] {
            seen.push_str(ax.split(QUANTIFIED).next().unwrap_or(ax));
        }
    }
    let patterns = pattern_terms(&seen);
    let forms: Vec<&str> = axioms
        .iter()
        .map(|(symbol, ax)| match ax.split_once(QUANTIFIED) {
            Some((_, quant)) if patterns.iter().any(|p| p.contains(symbol.as_str())) => quant,
            Some((mac, _)) => mac,
            None => ax.as_str(),
        })
        .collect();
    // A macro (`define-fun`) must come before every definition that names it.
    let is_macro: Vec<bool> = axioms
        .iter()
        .zip(&forms)
        .map(|((symbol, _), form)| form.contains(&format!("(define-fun {symbol}")))
        .collect();
    let mut done = vec![false; axioms.len()];
    let mut out = String::new();
    loop {
        let mut progressed = false;
        for k in 0..axioms.len() {
            if !chosen[k] || done[k] {
                continue;
            }
            let waits = (0..axioms.len()).any(|j| {
                j != k
                    && chosen[j]
                    && !done[j]
                    && is_macro[j]
                    && forms[k].contains(axioms[j].0.as_str())
            });
            if !waits {
                done[k] = true;
                out.push_str(forms[k]);
                out.push('\n');
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    out
}

/// Separates a macro definition from its quantified form in an axiom text
/// (`Prog::axiom`).
const QUANTIFIED: &str = "\n; quantified form\n";

/// The terms of every `:pattern (..)` in `text`.
fn pattern_terms(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find(":pattern (") {
        let start = i + ":pattern ".len();
        let mut depth = 0usize;
        let mut end = start;
        for (j, c) in rest[start..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + j + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.push(&rest[start..end]);
        rest = &rest[end.max(start)..];
    }
    out
}

/// The uninterpreted part of spec fn symbol `f`'s macro: its value outside
/// the parameters' types (`|m::f|` → `|m::f!ext|`).
fn macro_ext(f: &str) -> String {
    match f.strip_suffix('|') {
        Some(head) => format!("{head}!ext|"),
        None => format!("{f}!ext"),
    }
}

/// Query `q` with the ground instances of the validity functions of
/// recursive types (ADR-0060 §2) for the applications it holds, two levels
/// deep; `q` itself when there are none.
fn valid_instances(defs: &[AxiomParts], q: String) -> String {
    if defs.is_empty() {
        return q;
    }
    let (inst, _) = ground_instances(defs, &q, 2);
    format!("{q}\n{inst}")
}

/// The values a counterexample of `o` shows: the inputs, and the value an
/// invariant obligation is about.
fn shown_inputs(inputs: &[(String, Val, TypeKind)], o: &Obl) -> Vec<(String, Val, TypeKind)> {
    let mut out = inputs.to_vec();
    out.extend(o.shown.iter().cloned());
    out
}

/// The free variables of a failed sub-proof in its model, as values of
/// their types (`x = 4294967295, b = -1`).
fn sub_counterexample(sol: &mut Solver, q: &crate::subproof::Query) -> String {
    let terms: Vec<String> = q.vars.iter().map(|(s, _)| s.clone()).collect();
    let Ok(vals) = sol.get_values(&terms) else {
        return String::new();
    };
    q.vars
        .iter()
        .zip(vals)
        .map(|((s, sort), v)| {
            let name = s.trim_matches('|');
            let shown = match sort {
                crate::subproof::Sort::Int(i) => {
                    crate::subproof::model_int(&v, *i).map_or(v.clone(), |x| x.to_string())
                }
                crate::subproof::Sort::Bool | crate::subproof::Sort::F64 => v,
            };
            format!("{name} = {shown}")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Whether the current model gives every `f64` operation of the query
/// (`script`: what was asserted) the value IEEE 754 gives its operands'
/// values (ADR-0061 §5, backlog T2-03). In an ordinary query the operations
/// are uninterpreted, so a model may make `x < 1.0` false at `x = 0.0`: a
/// counterexample read from it would be false (G1). Each application
/// outside a quantifier is recomputed with Rust's `f64` arithmetic on the
/// model's values of its operands and compared with the model's value of
/// the application (NaN with NaN, anything else bit for bit). `Err` names
/// why the model is not confirmed: a mismatch, an application under a
/// quantifier or in a definition's body (its operands are not values), or
/// `min`/`max` of zeros of both signs (Rust leaves the sign unspecified).
fn f64_model_check(sol: &mut Solver, script: &str) -> Result<(), String> {
    use smt::Sexp::{Atom, List};
    if !script.contains("(f64.") {
        return Ok(());
    }
    fn mentions_f64(e: &smt::Sexp) -> bool {
        match e {
            Atom(a) => a.starts_with("f64."),
            List(xs) => xs.iter().any(mentions_f64),
        }
    }
    fn apps(e: &smt::Sexp, out: &mut Vec<smt::Sexp>) -> Result<(), String> {
        let List(xs) = e else { return Ok(()) };
        match xs.first() {
            Some(Atom(q)) if (q == "forall" || q == "exists") && mentions_f64(e) => {
                Err("an `f64` operation under a quantifier".into())
            }
            Some(Atom(h)) if h.starts_with("f64.") => {
                for x in &xs[1..] {
                    apps(x, out)?;
                }
                if !out.contains(e) {
                    out.push(e.clone());
                }
                Ok(())
            }
            _ => xs.iter().try_for_each(|x| apps(x, out)),
        }
    }
    let mut found = Vec::new();
    for cmd in smt::split_commands(script) {
        let Some(c) = smt::parse(&cmd) else { continue };
        let List(xs) = &c else { continue };
        match xs.first() {
            Some(Atom(a)) if a == "assert" => {
                if let Some(x) = xs.get(1) {
                    apps(x, &mut found)?;
                }
            }
            // A defined symbol of the query (not an `f64` operation's own
            // definition) whose body applies one.
            Some(Atom(a)) if a == "define-fun" => {
                let named_f64 = matches!(xs.get(1), Some(Atom(n)) if n.starts_with("f64."));
                if !named_f64 && xs.get(4).is_some_and(mentions_f64) {
                    return Err("an `f64` operation in a definition".into());
                }
            }
            _ => {}
        }
    }
    if found.is_empty() {
        return Ok(());
    }
    let mut terms = Vec::new();
    for a in &found {
        let List(xs) = a else { unreachable!() };
        terms.push(smt::print(a));
        terms.extend(xs[1..].iter().map(smt::print));
    }
    let vals = sol.get_value_terms(&terms)?;
    let mut it = vals.into_iter();
    for a in &found {
        let List(xs) = a else { unreachable!() };
        let Atom(h) = &xs[0] else { unreachable!() };
        let got = it.next().ok_or("missing model value")?;
        let ops: Vec<smt::Sexp> = (1..xs.len()).filter_map(|_| it.next()).collect();
        if !f64_agrees(h, &ops, &got)? {
            return Err(format!(
                "the model gives `{}` a value IEEE 754 does not",
                smt::print(a)
            ));
        }
    }
    Ok(())
}

/// Whether `got` is the IEEE 754 value (rounding to nearest even) of the
/// `f64` operation `h` on the model values `ops` (`f64_model_check`).
fn f64_agrees(h: &str, ops: &[smt::Sexp], got: &smt::Sexp) -> Result<bool, String> {
    let fp = |k: usize| -> Result<f64, String> {
        ops.get(k)
            .and_then(smt::fp_value)
            .ok_or_else(|| "a model value that is not an `f64`".to_string())
    };
    let same = |x: f64, y: f64| (x.is_nan() && y.is_nan()) || x.to_bits() == y.to_bits();
    let boolean = |b: bool| -> Result<bool, String> {
        match got {
            smt::Sexp::Atom(a) if a == "true" || a == "false" => Ok((a == "true") == b),
            _ => Err("a model value that is not a `bool`".into()),
        }
    };
    let value = |x: f64| -> Result<bool, String> {
        let y = smt::fp_value(got).ok_or("a model value that is not an `f64`")?;
        Ok(same(x, y))
    };
    match h {
        "f64.add" => value(fp(0)? + fp(1)?),
        "f64.sub" => value(fp(0)? - fp(1)?),
        "f64.mul" => value(fp(0)? * fp(1)?),
        "f64.div" => value(fp(0)? / fp(1)?),
        "f64.neg" => value(-fp(0)?),
        "f64.sqrt" => value(fp(0)?.sqrt()),
        "f64.abs" => value(fp(0)?.abs()),
        "f64.min" | "f64.max" => {
            let (a, b) = (fp(0)?, fp(1)?);
            if a == 0.0 && b == 0.0 && a.to_bits() != b.to_bits() {
                return Err("`min`/`max` of zeros of both signs".into());
            }
            value(if h == "f64.min" { a.min(b) } else { a.max(b) })
        }
        "f64.from" => {
            let n: i128 = smt::render_value(&ops[0])
                .parse()
                .map_err(|_| "an integer model value out of range")?;
            value(n as f64)
        }
        "f64.lt" => boolean(fp(0)? < fp(1)?),
        "f64.le" => boolean(fp(0)? <= fp(1)?),
        "f64.eq" => boolean(fp(0)? == fp(1)?),
        "f64.is_nan" => boolean(fp(0)?.is_nan()),
        "f64.is_finite" => boolean(fp(0)?.is_finite()),
        _ => Err(format!("unknown `f64` operation `{h}`")),
    }
}

/// `script` with each `f64` operation declared uninterpreted replaced by
/// its IEEE 754 definition (`f64_decls`): a model of it is a model of the
/// program's arithmetic. It runs under the same pinned `rlimit` (bit-precise
/// queries spend resource units fast but cheaply: `x + y == y + x` takes
/// 2.8M units in half a second).
fn f64_interpreted(script: &str) -> String {
    let opaque = f64_decls(script, false);
    let exact = f64_decls(script, true);
    let mut s = script.to_string();
    for (o, x) in opaque.lines().zip(exact.lines()) {
        s = s.replace(o, x);
    }
    s
}

/// E0811 for an obligation whose `f64` counterexample IEEE 754 arithmetic
/// does not confirm (`f64_model_check`): undecided, no counterexample.
fn f64_undecided(o: &Obl, path: &str, why: &str) -> Diag {
    Diag::new(
        "E0811",
        o.span,
        format!(
            "{} in `{path}`: solver could not decide (`f64` operations are uninterpreted here; no IEEE 754 counterexample: {why})",
            o.msg
        ),
    )
    .fix(
        "outside `by (bit_vector)` the `f64` operations are uninterpreted (ADR-0061 §5): prove the `f64` fact the obligation needs with `assert(e) by (bit_vector);` before it",
        None,
    )
}

/// `x = 5, a = [1, 2] (len 2), p = Point { x: 1, y: 2 }` from the current
/// model.
fn counterexample(sol: &mut Solver, inputs: &[(String, Val, TypeKind)], tb: &Tables) -> String {
    // A recursive struct is shown by its fields, one level deep (backlog
    // 2O-03d): its own model value is an element of an uninterpreted sort.
    let inputs: Vec<(String, Val, TypeKind)> = inputs
        .iter()
        .map(|(n, v, t)| match v {
            Val::S(x) if dt_struct(t, tb) => (
                n.clone(),
                Val::Struct(dt_fields(value_ty(t), x, tb)),
                t.clone(),
            ),
            _ => (n.clone(), v.clone(), t.clone()),
        })
        .collect();
    let inputs = &inputs;
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
    let x = &unlet(x, &BTreeMap::new());
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
    let vs = variants(t, tb).unwrap_or_default();
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

/// `x` with Z3's `let` bindings substituted (a model value may share
/// subterms, as with a datatype nested in an array).
fn unlet(x: &smt::Sexp, env: &BTreeMap<String, smt::Sexp>) -> smt::Sexp {
    use smt::Sexp::{Atom, List};
    match x {
        Atom(a) => env.get(a).cloned().unwrap_or_else(|| x.clone()),
        List(xs) => {
            if let [Atom(l), List(binds), body] = xs.as_slice()
                && l == "let"
            {
                let mut inner = env.clone();
                for b in binds {
                    if let List(p) = b
                        && let [Atom(n), v] = p.as_slice()
                    {
                        let v = unlet(v, env);
                        inner.insert(n.clone(), v);
                    }
                }
                return unlet(body, &inner);
            }
            List(xs.iter().map(|y| unlet(y, env)).collect())
        }
    }
}

/// A value of type `t` from its model leaves (constructor arguments).
fn show_leaves<'a>(
    t: &TypeKind,
    tb: &Tables,
    it: &mut impl Iterator<Item = &'a smt::Sexp>,
) -> String {
    if let Some(fs) = flat_fields(t, tb) {
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
            && (is_scalar_ty(&e) || variants(&e, tb).is_some())
        {
            let shown: Option<Vec<String>> = (0..n.min(SHOW as i64))
                .map(|k| {
                    model_select(a, k).map(|x| match variants(&e, tb) {
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
        Some(_) if dt_struct(t, tb) => format!("{} {{ .. }}", dt_ctor_name(t)),
        Some(x) if variants(t, tb).is_some() => show_variant(x, t, tb),
        Some(x) => smt::render_value(x),
        None => String::new(),
    }
}

fn is_scalar_ty(t: &TypeKind) -> bool {
    matches!(t, TypeKind::Int(_) | TypeKind::Bool)
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
        Val::S(_) if dt_struct(t, tb) => {
            it.next();
            format!("{} {{ .. }}", dt_ctor_name(t))
        }
        Val::S(_) if variants(t, tb).is_some() => {
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
                    _ if dt_struct(&e, tb) => format!("{} {{ .. }}", dt_ctor_name(&e)),
                    Some(v) if variants(&e, tb).is_some() => show_variant(&v, &e, tb),
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

    /// Congruence facts of the abstracted query (backlog 1B-49) relate a
    /// code term and a specification term of the same operation, unsigned
    /// and in two's complement, through nested terms, and every fact holds
    /// without the path condition: Z3 refutes its negation at edge and
    /// interior values of the operands (at symbolic values Z3 does not
    /// decide `int2bv` at 16 bits in a minute).
    #[test]
    fn bit_congruences_relate_notations_and_hold() {
        let head = "(declare-const |'bv@0| (_ BitVec 32))\n(declare-const |'bv@1| (_ BitVec 16))\n(declare-const |'bv@2| (_ BitVec 32))\n";
        // Code: `x ^ 5`, `(x ^ 5) | 1` and `y & 7` (y: i16), on a path `p`.
        let q = "(assert (=> p (= (bv2nat |'bv@0|) x)))
(assert (=> p (= (bv2nat |'bv@1|) (ite (< y 0) (+ y 65536) y))))
(assert (=> p (= (bv2nat |'bv@2|) (bv2nat (bvxor |'bv@0| (_ bv5 32))))))
(assert (= a (bv2nat (bvor |'bv@2| (_ bv1 32)))))
(assert (= b (bv2nat (bvor ((_ int2bv 32) (bv2nat (bvxor ((_ int2bv 32) x) ((_ int2bv 32) 5)))) ((_ int2bv 32) 1)))))
(assert (= c (bv2nat (bvand |'bv@1| (_ bv7 16)))))
(assert (= d (bv2nat (bvand ((_ int2bv 16) y) ((_ int2bv 16) 7)))))
";
        let cmds: Vec<smt::Sexp> = smt::split_commands(q)
            .iter()
            .map(|c| smt::parse(c).unwrap())
            .collect();
        let terms = [
            "(bv2nat (bvor |'bv@2| (_ bv1 32)))",
            "(bv2nat (bvor ((_ int2bv 32) (bv2nat (bvxor ((_ int2bv 32) x) ((_ int2bv 32) 5)))) ((_ int2bv 32) 1)))",
            "(bv2nat (bvand |'bv@1| (_ bv7 16)))",
            "(bv2nat (bvand ((_ int2bv 16) y) ((_ int2bv 16) 7)))",
            "(bv2nat |'bv@2|)",
            "(bv2nat (bvxor |'bv@0| (_ bv5 32)))",
        ]
        .map(String::from);
        let facts = bit_congruences(head, &cmds, &terms);
        assert_eq!(
            facts,
            [
                "(assert (=> (= (bv2nat |'bv@2|) (bv2nat (bvxor ((_ int2bv 32) x) ((_ int2bv 32) 5)))) (= (bv2nat (bvor |'bv@2| (_ bv1 32))) (bv2nat (bvor ((_ int2bv 32) (bv2nat (bvxor ((_ int2bv 32) x) ((_ int2bv 32) 5)))) ((_ int2bv 32) 1))))))",
                "(assert (=> (= (bv2nat |'bv@1|) (mod y 65536)) (= (bv2nat (bvand |'bv@1| (_ bv7 16))) (bv2nat (bvand ((_ int2bv 16) y) ((_ int2bv 16) 7))))))",
                "(assert (=> (= x (bv2nat |'bv@0|)) (= (bv2nat (bvxor ((_ int2bv 32) x) ((_ int2bv 32) 5))) (bv2nat (bvxor |'bv@0| (_ bv5 32))))))",
            ]
        );
        // With the facts, the abstracted query is unsatisfiable under `p`
        // for `a = b` and `c = d`: check that each fact holds everywhere.
        let xs = ["0", "5", "4294967295", "2147483648", "123456789"];
        let ys = ["0", "7", "(- 1)", "(- 32768)", "32767", "(- 1000)"];
        for fact in &facts {
            for x in xs {
                for y in ys {
                    let body = fact.strip_prefix("(assert ").unwrap();
                    let body = &body[..body.len() - 1];
                    let script = format!(
                        "{head}(declare-const x Int)\n(declare-const y Int)\n(assert (= x {x}))\n(assert (= y {y}))\n(assert (not {body}))\n"
                    );
                    let mut s = smt::Solver::start().expect("z3 must be installed");
                    s.run(&script).unwrap();
                    let answer = s.check_sat().unwrap();
                    assert!(
                        matches!(answer, smt::Answer::Unsat),
                        "{fact} at x = {x}, y = {y}"
                    );
                }
            }
        }
    }

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
