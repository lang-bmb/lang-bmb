//! Proof gate (backlog 1B-02 .. 1B-11, semantics in ADR-0017).
//!
//! Each executable function is symbolically executed over its checked syntax
//! tree. Local variables become single-assignment SMT constants; loops are cut
//! by their invariants; calls assume only the callee's `ensures`. Every implicit
//! and explicit proof obligation is checked separately by Z3. Only `unsat`
//! proves an obligation.

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
            flatten(elem, out);
            out.push(len.clone());
        }
        Val::Unit => {}
    }
}

/// One SMT leaf of a value: a scalar, or an array's elements or length
/// (`Some(n)` for a fixed-size array).
enum Leaf {
    Scalar(TypeKind),
    Elems(TypeKind),
    Len(TypeKind, Option<u64>),
}

impl Leaf {
    fn sort(&self) -> String {
        match self {
            Leaf::Scalar(t) => sort_of(t),
            Leaf::Elems(e) => format!("(Array Int {})", sort_of(e)),
            Leaf::Len(..) => "Int".into(),
        }
    }

    /// What every value of the leaf satisfies (ADR-0017 §3).
    fn fact(&self, x: &str) -> Option<String> {
        match self {
            Leaf::Scalar(t) => range(t, x),
            Leaf::Elems(_) => None,
            Leaf::Len(_, Some(n)) => Some(format!("(= {x} {n})")),
            Leaf::Len(e, None) => Some(format!("(and (<= 0 {x}) (<= {x} {}))", len_max(e))),
        }
    }
}

/// The fields of the struct `t` denotes (through a reference), in
/// declaration order.
pub(crate) fn struct_fields<'t>(t: &TypeKind, tb: &'t Tables) -> Option<&'t [(String, TypeKind)]> {
    let t = match t {
        TypeKind::Ref { inner, .. } => &inner.kind,
        t => t,
    };
    let TypeKind::Named { path, .. } = t else {
        return None;
    };
    match tb.adts.get(&path.join("::")) {
        Some(crate::check::Adt {
            kind: crate::check::AdtKind::Struct(fs),
            ..
        }) => Some(fs),
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
            Some(crate::check::Adt {
                kind: crate::check::AdtKind::Enum(vs),
                ..
            }) => Some(vs.clone()),
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
    if is_array(t) {
        return true;
    }
    if let Some(fs) = struct_fields(t, tb) {
        return fs.iter().any(|(_, ft)| has_array(ft, tb));
    }
    enum_variants(t, tb).is_some_and(|vs| {
        vs.iter()
            .any(|(_, ps)| ps.iter().any(|pt| has_array(pt, tb)))
    })
}

/// Whether a struct holds an enum whose payload holds an array (SMT
/// equality of that enum would compare elements past the length).
fn enum_with_array(t: &TypeKind, tb: &Tables) -> bool {
    match struct_fields(t, tb) {
        Some(fs) => fs.iter().any(|(_, ft)| enum_with_array(ft, tb)),
        None => enum_variants(t, tb).is_some() && has_array(t, tb),
    }
}

/// Every enum instance a program uses, as (constructor prefix, sort,
/// `declare-datatypes`), dependencies first (types are not recursive,
/// E0421).
fn datatypes(tb: &Tables) -> Vec<(String, String, String)> {
    fn visit(
        t: &TypeKind,
        tb: &Tables,
        seen: &mut BTreeSet<String>,
        out: &mut Vec<(String, String, String)>,
    ) {
        match t {
            TypeKind::Array(e, _) | TypeKind::Slice(e) => return visit(&e.kind, tb, seen, out),
            TypeKind::Ref { inner, .. } => return visit(&inner.kind, tb, seen, out),
            TypeKind::Named { .. } => {}
            _ => return,
        }
        if let Some(fs) = struct_fields(t, tb) {
            for (_, ft) in fs {
                visit(ft, tb, seen, out);
            }
            return;
        }
        let Some(vs) = enum_variants(t, tb) else {
            return;
        };
        let ts = crate::check::ty_str(t);
        if !seen.insert(ts.clone()) {
            return;
        }
        let mut ctors = Vec::new();
        for (v, ps) in &vs {
            let mut fields = Vec::new();
            for (k, pt) in ps.iter().enumerate() {
                visit(pt, tb, seen, out);
                for (name, leaf) in payload_leaves(t, v, k, pt, tb) {
                    fields.push(format!("({} {})", sym(&name), leaf.sort()));
                }
            }
            ctors.push(if fields.is_empty() {
                format!("({})", ctor(t, v))
            } else {
                format!("({} {})", ctor(t, v), fields.join(" "))
            });
        }
        let sort = sym(&ts);
        let decl = format!("(declare-datatypes (({sort} 0)) (({})))", ctors.join(" "));
        out.push((format!("|{ts}::"), sort, decl));
    }
    let mut all: Vec<&TypeKind> = tb.types.values().collect();
    all.extend(tb.locals.values().flat_map(|m| m.values()));
    for sig in tb.fns.values() {
        all.extend(sig.params.iter());
        all.push(&sig.ret);
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for t in all {
        visit(t, tb, &mut seen, &mut out);
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
        for (n, ft) in fs {
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
            // A struct element: one array per scalar leaf (ADR-0027 §2).
            let mut el = Vec::new();
            leaves(&e, tb, "", &mut el);
            if struct_fields(&e, tb).is_some() {
                for (suf, leaf) in el {
                    let Leaf::Scalar(lt) = leaf else {
                        panic!("array inside an array element")
                    };
                    out.push((format!("{base}.e{suf}"), Leaf::Elems(lt)));
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
            if struct_fields(&e, tb).is_some() {
                let elem = Box::new(build(&e, tb, terms));
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

/// `a == b` for values of one type: fieldwise for structs, SMT equality for
/// scalars. `None` when a field is an array (its elements past the length
/// are not part of the value).
fn eq_val(a: &Val, b: &Val) -> Option<String> {
    match (a, b) {
        (Val::S(x), Val::S(y)) => Some(format!("(= {x} {y})")),
        (Val::Struct(xs), Val::Struct(ys)) => {
            let parts: Option<Vec<String>> = xs.iter().zip(ys).map(|(x, y)| eq_val(x, y)).collect();
            parts.map(|p| and(&p))
        }
        (Val::Unit, Val::Unit) => Some("true".into()),
        // Arrays are equal on their elements below the length (a struct
        // field's array has a fixed one).
        (Val::A { elems: e1, len: l1 }, Val::A { elems: e2, len: l2 }) => Some(format!(
            "(and (= {l1} {l2}) (forall ((|eq!k| Int)) (! (=> (and (<= 0 |eq!k|) (< |eq!k| {l1})) (= (select {e1} |eq!k|) (select {e2} |eq!k|))) :pattern ((select {e1} |eq!k|)) :pattern ((select {e2} |eq!k|)))))"
        )),
        _ => None,
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
}

struct Prog<'p> {
    defs: BTreeMap<String, Def<'p>>,
    t: &'p Tables,
}

/// Symbolic state of one definition.
struct Enc<'p, 'q> {
    prog: &'q Prog<'p>,
    path: String,
    mod_path: String,
    /// Unit whose names the specification being translated refers to.
    spec_mod: String,
    decls: Vec<String>,
    counters: BTreeMap<String, u32>,
    pc: Vec<String>,
    vars: BTreeMap<String, Val>,
    old: BTreeMap<String, Val>,
    entry_measure: Option<String>,
    obls: Vec<Obl>,
    /// Entry constants of the parameters, for counterexamples: (name, terms,
    /// type).
    inputs: Vec<(String, Val, TypeKind)>,
    /// Spec translation: enclosing conditions, and the recursion check context.
    spec_pc: Vec<String>,
    term_check: Option<(usize, String)>,
    forall_depth: u32,
    /// Per enclosing `forall`, its `#[trigger]` terms: each one a list of
    /// alternatives (a struct-valued trigger has one per leaf).
    triggers: Vec<Vec<Vec<String>>>,
    bound: u32,
    unsupported: Vec<(Span, String)>,
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
        _ => None,
    }
}

fn len_max(elem: &TypeKind) -> &'static str {
    // A data type may be one byte (a fieldless enum, a struct of a bool):
    // only `isize::MAX` is known without its layout.
    if matches!(elem, TypeKind::Bool | TypeKind::Named { .. }) {
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
        _ => format!("(mod {x} {TWO_64})"),
    }
}

fn sym(name: &str) -> String {
    format!("|{name}|")
}

impl<'p, 'q> Enc<'p, 'q> {
    fn new(prog: &'q Prog<'p>, path: &str, mod_path: &str) -> Self {
        Enc {
            prog,
            path: path.to_string(),
            mod_path: mod_path.to_string(),
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
            forall_depth: 0,
            triggers: Vec::new(),
            bound: 0,
            unsupported: Vec::new(),
        }
    }

    fn ty(&self, e: &Expr) -> TypeKind {
        self.prog
            .t
            .types
            .get(&(e.span.file, e.span.lo, e.span.hi))
            .cloned()
            .unwrap_or(TypeKind::Error)
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
        let wanted =
            matches!(e, TypeKind::U64 | TypeKind::Usize) || (signed && matches!(e, TypeKind::I64));
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
        leaves(e, self.prog.t, &format!("{base}.e"), &mut ls);
        let terms: Vec<String> = ls
            .iter()
            .map(|(name, leaf)| self.fresh(name, &format!("(Array Int {})", leaf.sort())))
            .collect();
        build(e, self.prog.t, &mut terms.into_iter())
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
        if is_array(t) && struct_fields(&elem_ty(t), self.prog.t).is_some() {
            let e = elem_ty(t);
            let elem = Box::new(self.fresh_struct_elems(base, &e));
            let l = self.fresh(&format!("{base}.len"), "Int");
            self.pc
                .push(format!("(and (<= 0 {l}) (<= {l} {}))", len_max(&e)));
            return Val::Arr { elem, len: l };
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
    /// checker resolves it: `f` and `T::f` (a type of the unit) are local,
    /// anything else is a full path.
    fn resolve(&self, path: &[Ident], mod_path: &str) -> String {
        let name = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let local = format!("{mod_path}::{name}");
        let local_type = path.len() == 2
            && self
                .prog
                .t
                .adts
                .contains_key(&format!("{mod_path}::{}", path[0].name));
        if (path.len() == 1 || local_type) && self.prog.t.fns.contains_key(&local) {
            return local;
        }
        name
    }

    // ---------- specification terms ----------

    fn spec(&mut self, e: &Expr, env: &BTreeMap<String, Val>, old: &BTreeMap<String, Val>) -> Val {
        match &e.kind {
            ExprKind::Int(v) => Val::S(num(*v as i128)),
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
            // `==` on an enum is SMT equality, unless a payload holds an
            // array (its elements past the length would count).
            ExprKind::Binary(BinOp::Eq | BinOp::Ne, l, _)
                if enum_variants(&self.ty(l), self.prog.t).is_some()
                    && has_array(&self.ty(l), self.prog.t) =>
            {
                self.unsupported.push((
                    e.span,
                    "`==` on an enum with an array in a payload is not supported yet".into(),
                ));
                Val::S("true".into())
            }
            // Structural `==` on structs (ADR-0026 §2): field by field.
            ExprKind::Binary(op @ (BinOp::Eq | BinOp::Ne), l, r)
                if struct_fields(&self.ty(l), self.prog.t).is_some() =>
            {
                let a = self.spec(l, env, old);
                let b = self.spec(r, env, old);
                let q = if enum_with_array(&self.ty(l), self.prog.t) {
                    None
                } else {
                    eq_val(&a, &b)
                };
                match q {
                    Some(q) if *op == BinOp::Eq => Val::S(q),
                    Some(q) => Val::S(format!("(not {q})")),
                    None => {
                        self.unsupported.push((
                            e.span,
                            "`==` on a struct holding an enum with an array payload is not supported yet".into(),
                        ));
                        Val::S("true".into())
                    }
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
                    BinOp::Lt => format!("(< {lt} {rt})"),
                    BinOp::Le => format!("(<= {lt} {rt})"),
                    BinOp::Gt => format!("(> {lt} {rt})"),
                    BinOp::Ge => format!("(>= {lt} {rt})"),
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
                    op => {
                        let a = self.spec(&args[0], env, old).s();
                        let sym = match op {
                            "wrapping_add" => "+",
                            "wrapping_sub" => "-",
                            _ => "*",
                        };
                        Val::S(wrap(&self.ty(recv), &format!("({sym} {} {a})", rv.s())))
                    }
                }
            }
            ExprKind::Call { path, args } if self.is_variant(path) => {
                let vals: Vec<Val> = args.iter().map(|a| self.spec(a, env, old)).collect();
                self.variant_value(&self.ty(e), &path.last().unwrap().name, &vals)
            }
            ExprKind::Path(segs) => {
                self.variant_value(&self.ty(e), &segs.last().unwrap().name, &[])
            }
            ExprKind::Match { scrut, arms } => self.spec_match(scrut, arms, env, old),
            ExprKind::Call { path, args } => {
                let target = self.resolve(path, &self.spec_mod.clone());
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
                    decl.push(format!("({s} Int)"));
                    if let Some(r) = range(&b.ty.kind, &s) {
                        facts.push(r);
                    }
                    env2.insert(b.name.name.clone(), Val::S(s));
                }
                self.forall_depth += 1;
                self.triggers.push(Vec::new());
                let bt = self.spec(body, &env2, old).s();
                let pats = self.triggers.pop().unwrap();
                self.forall_depth -= 1;
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
            ExprKind::Array(_) | ExprKind::Repeat { .. } | ExprKind::Error => {
                self.unsupported.push((
                    e.span,
                    "this expression is not supported in specifications".into(),
                ));
                Val::S("true".into())
            }
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
                other => {
                    let sp = match other {
                        Stmt::Assign { span, .. }
                        | Stmt::While { span, .. }
                        | Stmt::Assert { span, .. } => *span,
                        _ => b.span,
                    };
                    self.unsupported
                        .push((sp, "statement not allowed in a specification".into()));
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
        let Some((comp, caller_measure)) = self.term_check.clone() else {
            return;
        };
        if self.prog.t.recursive.get(target) != Some(&comp) {
            return;
        }
        if self.forall_depth > 0 {
            self.unsupported.push((
                span,
                "recursive call inside `forall` is not supported".into(),
            ));
            return;
        }
        let def = &self.prog.defs[target];
        let Some(d) = &def.f.decreases else { return };
        let env = bind_params(def.f, args);
        let saved = self.term_check.take();
        let callee_measure = self.in_unit(&def.mod_path.clone(), |s| {
            s.spec(d, &env, &BTreeMap::new()).s()
        });
        self.term_check = saved;
        self.oblige(
            "E0813",
            span,
            format!("recursive call to `{target}` may not decrease its `decreases` measure"),
            format!("(and (<= 0 {caller_measure}) (< {callee_measure} {caller_measure}))"),
        );
    }

    // ---------- executable code ----------

    fn eval(&mut self, e: &Expr) -> Val {
        match &e.kind {
            ExprKind::Int(v) => Val::S(num(*v as i128)),
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
                    BinOp::Mul => format!("(* {lt} {rt})"),
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
                    BinOp::Lt => return Val::S(format!("(< {lt} {rt})")),
                    BinOp::Le => return Val::S(format!("(<= {lt} {rt})")),
                    BinOp::Gt => return Val::S(format!("(> {lt} {rt})")),
                    BinOp::Ge => return Val::S(format!("(>= {lt} {rt})")),
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
            ExprKind::Method { recv, name, args } => {
                let rv = self.eval(recv);
                match name.name.as_str() {
                    "len" => match rv {
                        Val::A { len, .. } | Val::Arr { len, .. } => Val::S(len),
                        _ => panic!("len of scalar"),
                    },
                    op => {
                        let a = self.eval(&args[0]).s();
                        let s = match op {
                            "wrapping_add" => "+",
                            "wrapping_sub" => "-",
                            _ => "*",
                        };
                        Val::S(wrap(&self.ty(recv), &format!("({s} {} {a})", rv.s())))
                    }
                }
            }
            ExprKind::Call { path, args } if self.is_variant(path) => {
                let vals: Vec<Val> = args.iter().map(|a| self.eval(a)).collect();
                self.variant_value(&self.ty(e), &path.last().unwrap().name, &vals)
            }
            ExprKind::Call { path, args } => self.call(path, args, e.span),
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
                        len,
                    }
                }
                Val::Struct(_) => {
                    let ty = self.local_ty(v);
                    self.fresh_val(v, &ty)
                }
                // The elements may change, never the length.
                Val::Arr { len, .. } => {
                    let e = elem_ty(&self.local_ty(v));
                    Val::Arr {
                        elem: Box::new(self.fresh_struct_elems(v, &e)),
                        len,
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
        if let Some(t) = self.prog.t.locals.get(&self.path).and_then(|m| m.get(name)) {
            return t.clone();
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

    fn call(&mut self, path: &[Ident], args: &[Expr], span: Span) -> Val {
        let target = self.resolve(path, &self.mod_path.clone());
        let mut argv = Vec::new();
        for a in args {
            argv.push(self.eval(a));
        }
        let Some(def) = self.prog.defs.get(&target) else {
            // The one built-in function has no contract; anything else
            // unresolved would drop a callee's effects.
            assert_eq!(target, "print_i64", "unresolved call in {}", self.path);
            return Val::Unit;
        };
        let f = def.f;
        let pre_env = bind_params(f, &argv);
        let unit = def.mod_path.clone();
        for r in &f.requires {
            let g = self.in_unit(&unit, |s| s.spec(r, &pre_env, &BTreeMap::new()).s());
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
            let m1 = self.in_unit(&unit, |s| s.spec(d, &pre_env, &BTreeMap::new()).s());
            self.oblige(
                "E0809",
                span,
                format!("recursive call to `{target}` may not decrease its measure"),
                format!("(and (<= 0 {m0}) (< {m1} {m0}))"),
            );
        }
        // `&mut` arguments: the callee may change the place, the elements
        // of a slice but never its length.
        let mut post_args = argv.clone();
        let mut old_env = BTreeMap::new();
        let param_tys = self.prog.t.fns[&target].params.clone();
        for (i, p) in f.params.iter().enumerate() {
            if let TypeKind::Ref { mutable: true, .. } = p.ty.kind
                && root_var(&args[i]).is_some()
            {
                let mut steps = Vec::new();
                let root = self.place(&args[i], &mut steps, false);
                let nv = match &argv[i] {
                    Val::A { len, .. } => {
                        let e = elem_ty(&param_tys[i]);
                        Val::A {
                            elems: self.fresh_elems(&root, &e, false),
                            len: len.clone(),
                        }
                    }
                    Val::Arr { len, .. } => {
                        let e = elem_ty(&param_tys[i]);
                        Val::Arr {
                            elem: Box::new(self.fresh_struct_elems(&root, &e)),
                            len: len.clone(),
                        }
                    }
                    _ => self.fresh_val(&root, &param_tys[i]),
                };
                old_env.insert(p.name.name.clone(), argv[i].clone());
                let cur = self.vars[&root].clone();
                self.vars.insert(root, set_at(cur, &steps, nv.clone()));
                post_args[i] = nv;
            }
        }
        let mut post_env = bind_params(f, &post_args);
        let ret_ty = self.prog.t.fns[&target].ret.clone();
        let result = self.fresh_val(&format!("{target}.result"), &ret_ty);
        if let Some(n) = &f.ret.name {
            post_env.insert(n.name.clone(), result.clone());
        }
        for en in &f.ensures {
            let g = self.in_unit(&unit, |s| s.spec(en, &post_env, &old_env).s());
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
        ExprKind::Method { recv, args, .. } => {
            assigned_in_expr(recv, out);
            args.iter().for_each(|a| assigned_in_expr(a, out));
        }
        _ => {}
    }
}

// ---------- program-level driver ----------

fn collect<'p>(m: &'p Mod, prefix: &str, out: &mut BTreeMap<String, Def<'p>>) {
    let mp = format!("{prefix}{}", m.name.name);
    for s in &m.mods {
        collect(s, &format!("{mp}::"), out);
    }
    for f in &m.fns {
        out.entry(format!("{mp}::{}", f.name.name)).or_insert(Def {
            f,
            mod_path: mp.clone(),
        });
    }
    for i in &m.impls {
        for f in &i.fns {
            out.entry(format!("{mp}::{}::{}", i.ty.name, f.name.name))
                .or_insert(Def {
                    f,
                    mod_path: mp.clone(),
                });
        }
    }
}

/// The first place a program's definitions touch a data type (backlog 2B-04:
/// checked, but neither encoded nor compiled yet).
pub(crate) fn program_data_type_use(p: &Program, t: &Tables) -> Option<Span> {
    let mut defs = BTreeMap::new();
    for m in &p.mods {
        collect(m, "", &mut defs);
    }
    let tainted = data_spec_fns(&defs, t, Gate::Codegen);
    defs.iter()
        .find_map(|(path, d)| data_type_use(d.f, path, t, &tainted, Gate::Codegen))
}

/// Which data types a back end cannot handle yet.
#[derive(Clone, Copy, PartialEq)]
enum Gate {
    /// The proof gate encodes structs, enums and slices of them (backlog
    /// 2B-05a..c), not yet array elements holding an array.
    Prover,
    /// The code generator, the same shapes (backlog 2B-06).
    Codegen,
}

impl Gate {
    fn rejects(self, t: &TypeKind, tb: &Tables) -> bool {
        match self {
            // The code generator lowers what the prover encodes (2B-06).
            Gate::Codegen => Gate::Prover.rejects(t, tb),
            Gate::Prover => match t {
                TypeKind::Named { .. } => match (struct_fields(t, tb), enum_variants(t, tb)) {
                    (Some(fs), _) => fs.iter().any(|(_, ft)| self.rejects(ft, tb)),
                    (_, Some(vs)) => vs
                        .iter()
                        .any(|(_, ps)| ps.iter().any(|pt| self.rejects(pt, tb))),
                    _ => true,
                },
                // Elements with an array inside are not encoded yet.
                TypeKind::Array(e, _) | TypeKind::Slice(e) => {
                    self.rejects(&e.kind, tb) || (has_named(&e.kind) && has_array(&e.kind, tb))
                }
                TypeKind::Ref { inner, .. } => self.rejects(&inner.kind, tb),
                _ => false,
            },
        }
    }
}

/// The spec fns whose definition touches a data type the gate rejects,
/// closed over calls: their axioms would carry it into any obligation that
/// reveals them.
fn data_spec_fns(defs: &BTreeMap<String, Def>, t: &Tables, gate: Gate) -> BTreeSet<String> {
    let mut tainted = BTreeSet::new();
    loop {
        let next: BTreeSet<String> = defs
            .iter()
            .filter(|(path, d)| {
                d.f.kind == FnKind::Spec && data_type_use(d.f, path, t, &tainted, gate).is_some()
            })
            .map(|(p, _)| p.clone())
            .collect();
        if next == tainted {
            return tainted;
        }
        tainted = next;
    }
}

fn has_named(t: &TypeKind) -> bool {
    match t {
        TypeKind::Named { .. } => true,
        TypeKind::Array(e, _) | TypeKind::Slice(e) => has_named(&e.kind),
        TypeKind::Ref { inner, .. } => has_named(&inner.kind),
        _ => false,
    }
}

/// Where a definition touches a data type `gate` rejects: a parameter, the
/// result, a local, any expression's type, or a call to a spec fn that does.
/// Such definitions are E0814 until their back end handles them.
fn data_type_use(
    f: &FnDef,
    path: &str,
    t: &Tables,
    tainted: &BTreeSet<String>,
    gate: Gate,
) -> Option<Span> {
    struct W<'a> {
        t: &'a Tables,
        tainted: &'a BTreeSet<String>,
        gate: Gate,
        locals: Option<&'a BTreeMap<String, TypeKind>>,
    }
    impl W<'_> {
        fn local_rejected(&self, name: &str) -> bool {
            self.locals
                .and_then(|m| m.get(name))
                .is_some_and(|ty| self.gate.rejects(ty, self.t))
        }
        fn block(&self, b: &Block) -> Option<Span> {
            for s in &b.stmts {
                let found = match s {
                    Stmt::LetFields { span, .. } => Some(*span),
                    Stmt::Let { name, ty, init, .. } => {
                        if self.local_rejected(&name.name) {
                            return Some(ty.span);
                        }
                        self.expr(init)
                    }
                    Stmt::Assign { target, value, .. } => {
                        self.expr(target).or_else(|| self.expr(value))
                    }
                    Stmt::While {
                        cond,
                        invariants,
                        decreases,
                        body,
                        ..
                    } => self
                        .expr(cond)
                        .or_else(|| invariants.iter().find_map(|x| self.expr(x)))
                        .or_else(|| decreases.as_ref().and_then(|x| self.expr(x)))
                        .or_else(|| self.block(body)),
                    Stmt::Assert { cond, .. } => self.expr(cond),
                    Stmt::Expr(e) => self.expr(e),
                };
                if found.is_some() {
                    return found;
                }
            }
            b.tail.as_deref().and_then(|e| self.expr(e))
        }
        fn expr(&self, e: &Expr) -> Option<Span> {
            let ty = self.t.types.get(&(e.span.file, e.span.lo, e.span.hi));
            if ty.is_some_and(|ty| self.gate.rejects(ty, self.t)) {
                return Some(e.span);
            }
            let sub: Vec<&Expr> = match &e.kind {
                ExprKind::Call { path, args } => {
                    let name = path
                        .iter()
                        .map(|i| i.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::");
                    let spec = self
                        .tainted
                        .iter()
                        .any(|p| p == &name || p.ends_with(&format!("::{name}")));
                    // A checked call that names no function builds a variant.
                    let variant = matches!(name.as_str(), "Some" | "Ok" | "Err")
                        || (path.len() > 1
                            && !self
                                .t
                                .fns
                                .iter()
                                .any(|(p, _)| p == &name || p.ends_with(&format!("::{name}"))));
                    let _ = variant;
                    if spec {
                        return Some(e.span);
                    }
                    args.iter().collect()
                }
                ExprKind::Method { recv, args, .. } => {
                    std::iter::once(&**recv).chain(args.iter()).collect()
                }
                ExprKind::Index { base, index } => vec![base, index],
                ExprKind::Unary(_, x)
                | ExprKind::Ref { expr: x, .. }
                | ExprKind::Old(x)
                | ExprKind::Trigger(x)
                | ExprKind::Deref(x) => vec![x],
                ExprKind::Field { base, .. } => vec![base],
                ExprKind::Binary(_, l, r) => vec![l, r],
                ExprKind::If { cond, then, els } => {
                    if let Some(sp) = self.block(then) {
                        return Some(sp);
                    }
                    std::iter::once(&**cond).chain(els.as_deref()).collect()
                }
                ExprKind::Block(b) => return self.block(b),
                ExprKind::Array(xs) => xs.iter().collect(),
                ExprKind::Repeat { value, .. } => vec![value],
                ExprKind::Forall { body, .. } => vec![body],
                ExprKind::StructLit { fields, rest, .. } => fields
                    .iter()
                    .map(|(_, x)| x)
                    .chain(rest.as_deref())
                    .collect(),
                ExprKind::Match { scrut, arms } => std::iter::once(&**scrut)
                    .chain(arms.iter().map(|a| &a.body))
                    .collect(),
                ExprKind::Path(_)
                | ExprKind::Int(_)
                | ExprKind::Bool(_)
                | ExprKind::Unit
                | ExprKind::Var(_)
                | ExprKind::Error => Vec::new(),
            };
            sub.into_iter().find_map(|x| self.expr(x))
        }
    }
    let w = W {
        t,
        tainted,
        gate,
        locals: t.locals.get(path),
    };
    for p in &f.params {
        if w.local_rejected(&p.name.name) {
            return Some(p.ty.span);
        }
    }
    if let Some(sig) = t.fns.get(path)
        && gate.rejects(&sig.ret, t)
    {
        return Some(f.ret.ty.span);
    }
    let clauses = f
        .requires
        .iter()
        .chain(&f.ensures)
        .chain(f.decreases.iter());
    for c in clauses {
        if let Some(sp) = w.expr(c) {
            return Some(sp);
        }
    }
    w.block(&f.body)
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
];

/// The same for unsigned operands, where `/` and `%` are SMT `div` and `mod`.
pub const UNSIGNED_DIV_LEMMAS: &[&str] = &[
    "(=> (and (>= {a} 0) (> {b} 0)) (and (<= 0 {q}) (<= {q} {a})))",
    "(=> (> {b} 0) (and (<= 0 {r}) (< {r} {b})))",
];

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
            .filter(|(_, d)| d.f.kind == FnKind::Spec)
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

    /// Definitional axiom of a spec fn (ADR-0017 §4).
    fn axiom(&self, path: &str) -> (String, Vec<(Span, String)>, AxiomParts) {
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
        (ax.join("\n"), enc.unsupported, parts)
    }
}

pub(crate) fn uses_of(f: &FnDef, mod_path: &str, fns: &BTreeMap<String, FnSig>) -> Vec<String> {
    let mut out = BTreeSet::new();
    for a in &f.attrs {
        if a.name.name == "uses" {
            for arg in &a.args {
                if let AttrArg::Ident(i) = arg {
                    let full = if i.name.contains("::") {
                        i.name.clone()
                    } else {
                        format!("{mod_path}::{}", i.name)
                    };
                    if fns.contains_key(&full) {
                        out.insert(full);
                    }
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
    let mut defs = BTreeMap::new();
    for m in &p.mods {
        collect(m, "", &mut defs);
    }
    let prog = Prog { defs, t };
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
    let mut defs = BTreeMap::new();
    for m in &p.mods {
        collect(m, "", &mut defs);
    }
    let prog = Prog { defs, t };
    let spec_decls = prog.spec_decls();
    let dts = datatypes(t);
    let tainted = data_spec_fns(&prog.defs, t, Gate::Prover);
    let mut solver: Option<Solver> = None;
    let mut solver_down = crate::toolchain::check_z3().is_err();
    let mut out = Vec::new();
    for (path, d) in &prog.defs {
        if d.f.kind != FnKind::Exec || d.f.requires.is_empty() {
            continue;
        }
        let span = d.f.requires[0]
            .span
            .to(d.f.requires[d.f.requires.len() - 1].span);
        let mut enc = Enc::new(&prog, path, &d.mod_path);
        let mut axioms: Vec<(String, String)> = Vec::new();
        let mut axiom_parts: Vec<AxiomParts> = Vec::new();
        for u in uses_of(d.f, &d.mod_path, &t.fns) {
            let (ax, unsup, parts) = prog.axiom(&u);
            axioms.push((spec_key(&u, t), ax));
            axiom_parts.push(parts);
            enc.unsupported.extend(unsup);
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
        if !enc.unsupported.is_empty()
            || solver_down
            || data_type_use(d.f, path, t, &tainted, Gate::Prover).is_some()
        {
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
    let mut defs = BTreeMap::new();
    for m in &p.mods {
        collect(m, "", &mut defs);
    }
    let prog = Prog { defs, t };
    let mut statuses = Vec::new();
    let mut scripts = BTreeMap::new();
    let mut proved_keys: BTreeSet<ObKey> = BTreeSet::new();
    let mut failed_keys: BTreeSet<ObKey> = BTreeSet::new();
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
        .filter(|(_, d)| d.f.kind == FnKind::Spec)
        .map(|(p, _)| (p.clone(), sym(p)))
        .collect();
    // One solver process serves every definition (backlog 1D-18): each
    // obligation starts with `(reset)` and re-sends its whole base, options
    // included (ADR-0018 §1), so nothing carries over between obligations.
    // Started on the first obligation the cache does not hold, replaced when
    // too old to take on another definition (`Solver::reusable`).
    let tainted = data_spec_fns(&prog.defs, t, Gate::Prover);
    let mut solver: Option<Solver> = None;
    let mut stopped = false;
    for (path, d) in &prog.defs {
        if stopped {
            break;
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
        let data = data_type_use(d.f, path, t, &tainted, Gate::Prover);
        if let Some(sp) = data {
            enc.unsupported.push((
                sp,
                "this data type is not supported by the proof gate yet (backlog 2B-05d)".into(),
            ));
        }
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
                let m = enc.spec(dec, &env, &BTreeMap::new()).s();
                enc.term_check = Some((comp, m));
                enc.spec_block(&d.f.body, &env, &BTreeMap::new());
                enc.term_check = None;
            }
            FnKind::Exec => {
                for u in uses_of(d.f, &d.mod_path, &t.fns) {
                    let (ax, unsup, parts) = prog.axiom(&u);
                    axioms.push((spec_key(&u, t), ax));
                    axiom_parts.push(parts);
                    enc.unsupported.extend(unsup);
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
                let (env, old) = enc.cur_env();
                for r in &d.f.requires {
                    let g = enc.spec(r, &env, &old).s();
                    enc.pc.push(g);
                }
                if let Some(dec) = &d.f.decreases {
                    enc.entry_measure = Some(enc.spec(dec, &env, &old).s());
                }
                let result = enc.block(&d.f.body);
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

        for (sp, msg) in &enc.unsupported {
            diags.push(Diag::new("E0814", *sp, msg.clone()));
        }
        // Data type terms have no encoding yet (2B-05): nothing to solve.
        if data.is_some() {
            enc.obls.clear();
        }

        // Assemble and solve. Every obligation runs on a reset solver holding only
        // the definition's base (options, prelude, axioms, declarations), so one
        // obligation's result never depends on another's (ADR-0017 §5).
        let mut prefix = header.join("\n");
        prefix.push('\n');
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
            if stop_at_failure && (failed > 0 || !enc.unsupported.is_empty()) {
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
            if let (Some(c), Some(ck)) = (cache.as_mut(), cache_key.as_ref())
                && let Some((total, max)) = c.get(ck)
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
            let total_before = rlimit_total;
            let mut first_used = 0;
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
            if matches!(answer, Ok(Answer::Unknown(_))) && !axiom_parts.is_empty() {
                let (inst, complete) = ground_instances(&axiom_parts, &q, 3);
                if !inst.is_empty() {
                    let used = needed_decls(&spec_decls, &dts, &format!("{decls}\n{inst}{q}"));
                    let head = format!("{prefix}{used}{decls}");
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
                        Ok(Answer::Unsat) => answer = retry,
                        Ok(Answer::Sat) if complete => answer = retry,
                        Ok(Answer::Sat) => {
                            answer = Ok(Answer::Unknown(
                                "a model exists for the instantiated definitions only".into(),
                            ))
                        }
                        _ => {}
                    }
                }
            }
            failed += 1;
            if matches!(answer, Ok(Answer::Unsat)) {
                proved_keys.insert(key);
                def_proved.push(key);
                if let (Some(c), Some(ck)) = (cache.as_mut(), cache_key) {
                    c.insert(ck, rlimit_total - total_before, first_used);
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
        stopped = stop_at_failure && (failed > 0 || !enc.unsupported.is_empty());
        let status = FnStatus {
            path: path.clone(),
            obligations: enc.obls.len(),
            failed: failed + enc.unsupported.len(),
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
        it.next();
        it.next();
        return "[..]".into();
    }
    match it.next() {
        Some(x) if enum_variants(t, tb).is_some() => show_variant(x, t, tb),
        Some(x) => smt::render_value(x),
        None => String::new(),
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
