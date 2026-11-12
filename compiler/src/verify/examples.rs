//! Examples of a contract (backlog T1-44c, ADR-0078 H-INTENT): from a
//! function's `requires` and `ensures` alone, never its body, inputs with a
//! result the specification allows, results it rejects for the same
//! inputs, and inputs the `requires` exclude. For a reader of the
//! specification.
//!
//! Every value shown is read from a model of a query over the entry state
//! the proof gate assumes (`Enc::entry_params`, `Enc::entry_requires`: the
//! one encoding W0901 and W0918 use), decided by `sat_of` as theirs are,
//! under the pinned seed and `rlimit`. A query that is not decided shows
//! nothing and is counted.
//!
//! - allowed: `requires ∧ ensures`, each next input different from all
//!   earlier ones (a blocking clause over the input's values).
//! - allowed again, for the same input: `requires ∧ ensures` with the
//!   input fixed and the result different from the one shown. A model
//!   means the contract leaves the result open there (W0918's question,
//!   asked of one input).
//! - rejected: `requires ∧ ¬ensures` with the input fixed. Then, while
//!   fewer than the count are found, with a free input different from all
//!   earlier ones.
//! - excluded: the parameters' type facts and `¬requires`.
//!
//! Small values first: each query is asked within the bounds of the
//! bounded search (`bound_terms`, ADR-0108 §1: |integers| <= 8), then
//! without them. Within those bounds the n-th input of a kind (from 0) is
//! first asked with every input length at least n, so that the inputs are
//! not all of the shortest length; and a `bool` result of an allowed row is
//! first asked to be `true` for an even n and `false` for an odd one, with
//! and then without the integer bounds. Lengths stay within what the counterexample printer shows
//! in full (`SHOW`): a value printed with `..` is not an example.

use super::*;

/// The most inputs asked for per definition (`bmb examples --count`).
pub const MAX_EXAMPLES: usize = 64;

/// One example: its kind (`allowed`, `rejected`, `excluded`), the
/// parameters' values on entry, the result (none for `()` and for
/// `excluded`), and the values behind the `&mut` parameters afterwards.
/// Values are in the notation of a counterexample (`counterexample`).
pub struct Example {
    pub kind: &'static str,
    pub input: Vec<(String, String)>,
    pub result: Option<String>,
    pub after: Vec<(String, String)>,
}

/// The examples of one definition's contract, in order.
pub struct Examples {
    pub def: String,
    pub rows: Vec<Example>,
    /// Queries that were not decided: each would have been one row, or the
    /// end of a kind's rows.
    pub undecided: usize,
    pub rlimit: u64,
    /// Why there are no examples: a type the values of which cannot be
    /// shown in full, or a contract through a spec fn seen by name only.
    pub reason: Option<String>,
}

impl Examples {
    fn count(&self, kind: &str) -> usize {
        self.rows.iter().filter(|r| r.kind == kind).count()
    }

    /// The rows as JSONL, then the summary row.
    pub fn jsonl(&self) -> String {
        use crate::diag::json_str;
        let obj = |xs: &[(String, String)]| {
            let fields: Vec<String> = xs
                .iter()
                .map(|(n, v)| format!("{}:{}", json_str(n), json_str(v)))
                .collect();
            format!("{{{}}}", fields.join(","))
        };
        let mut out = String::new();
        for r in &self.rows {
            out.push_str(&format!(
                "{{\"def\":{},\"kind\":\"{}\",\"input\":{}",
                json_str(&self.def),
                r.kind,
                obj(&r.input)
            ));
            if let Some(x) = &r.result {
                out.push_str(&format!(",\"result\":{}", json_str(x)));
            }
            if !r.after.is_empty() {
                out.push_str(&format!(",\"after\":{}", obj(&r.after)));
            }
            out.push_str("}\n");
        }
        out.push_str(&format!(
            "{{\"def\":{},\"summary\":true,\"allowed\":{},\"rejected\":{},\"excluded\":{},\"undecided\":{},\"rlimit\":{},\"max_len\":{SHOW}",
            json_str(&self.def),
            self.count("allowed"),
            self.count("rejected"),
            self.count("excluded"),
            self.undecided,
            self.rlimit
        ));
        if let Some(r) = &self.reason {
            out.push_str(&format!(",\"reason\":{}", json_str(r)));
        }
        out.push_str("}\n");
        out
    }
}

/// Whether every value of type `t` is shown in full by `counterexample`
/// and fixed by `pin`: integers and `bool`, structs of them, and arrays and
/// slices (of at most `SHOW` elements when the length is fixed) of
/// integers, `bool` or structs of those.
fn shown_whole(t: &TypeKind, tb: &Tables, element: bool) -> bool {
    if is_array(t) {
        let short = !matches!(t, TypeKind::Array(_, n) if *n > SHOW as u64);
        return !element && short && shown_whole(&elem_ty(t), tb, true);
    }
    match t {
        TypeKind::Int(_) | TypeKind::Bool => true,
        TypeKind::Ref { inner, .. } => !element && shown_whole(&inner.kind, tb, false),
        _ => flat_fields(t, tb)
            .is_some_and(|fs| fs.iter().all(|(_, ft)| shown_whole(ft, tb, element))),
    }
}

fn is_bool(t: &TypeKind) -> bool {
    match t {
        TypeKind::Bool => true,
        TypeKind::Ref { inner, .. } => is_bool(&inner.kind),
        _ => false,
    }
}

/// The terms whose model values fix `v` (of type `t`), in the order `pin`
/// reads them, each with whether it is a `bool`.
fn leaves(v: &Val, t: &TypeKind, tb: &Tables, out: &mut Vec<(String, bool)>) {
    match v {
        Val::S(x) => out.push((x.clone(), is_bool(t))),
        Val::A { elems, len } => {
            out.push((len.clone(), false));
            let b = is_bool(&elem_ty(t));
            for k in 0..SHOW {
                out.push((format!("(select {elems} {k})"), b));
            }
        }
        Val::Struct(fs) => {
            let tys = flat_fields(t, tb).unwrap_or_default();
            for (f, (_, ft)) in fs.iter().zip(&tys) {
                leaves(f, ft, tb, out);
            }
        }
        Val::Arr { elem, len } => {
            out.push((len.clone(), false));
            let e = elem_ty(t);
            for k in 0..SHOW {
                leaves(&select_val(elem, &k.to_string()), &e, tb, out);
            }
        }
        Val::Unit => {}
    }
}

/// Why a model does not give a value.
enum Unread {
    /// This `bool` term has no literal value in the model (the solver
    /// gives a formula, as for a result equal to a quantified predicate):
    /// asked again with the term asserted, then denied.
    Open(String),
    /// A value that is not a literal, or a length that is not shown whole.
    Not,
}

/// An integer or `bool` literal as the solver writes it.
fn literal(x: &smt::Sexp) -> bool {
    use smt::Sexp::{Atom, List};
    let digits = |a: &String| !a.is_empty() && a.chars().all(|c| c.is_ascii_digit());
    match x {
        Atom(a) => a == "true" || a == "false" || digits(a),
        List(xs) => matches!(xs.as_slice(), [Atom(m), Atom(n)] if m == "-" && digits(n)),
    }
}

/// `v` equal to its model value (`vals`: the `leaves` with their values),
/// one equation per leaf: the length and the elements below it.
fn pin(
    v: &Val,
    vals: &mut impl Iterator<Item = (String, bool, smt::Sexp)>,
    out: &mut Vec<String>,
) -> Result<(), Unread> {
    let mut leaf = |keep: bool, out: &mut Vec<String>| -> Result<smt::Sexp, Unread> {
        let (term, is_bool, x) = vals.next().ok_or(Unread::Not)?;
        if keep {
            if !literal(&x) {
                return Err(if is_bool {
                    Unread::Open(term)
                } else {
                    Unread::Not
                });
            }
            out.push(format!("(= {term} {})", smt::print(&x)));
        }
        Ok(x)
    };
    let len_of = |x: smt::Sexp| {
        smt::render_value(&x)
            .parse::<usize>()
            .ok()
            .filter(|n| *n <= SHOW)
            .ok_or(Unread::Not)
    };
    match v {
        Val::S(_) => {
            leaf(true, out)?;
        }
        Val::A { len, .. } => {
            let n = len_of(leaf(!fixed_len(len), out)?)?;
            for k in 0..SHOW {
                leaf(k < n, out)?;
            }
        }
        Val::Struct(fs) => {
            for f in fs {
                pin(f, vals, out)?;
            }
        }
        Val::Arr { elem, len } => {
            let n = len_of(leaf(!fixed_len(len), out)?)?;
            for k in 0..SHOW {
                let mut one = Vec::new();
                let r = pin(&select_val(elem, &k.to_string()), vals, &mut one);
                if k < n {
                    r?;
                    out.extend(one);
                }
            }
        }
        Val::Unit => {}
    }
    Ok(())
}

/// The lengths of `v` that are not fixed.
fn lens(v: &Val, out: &mut Vec<String>) {
    match v {
        Val::A { len, .. } | Val::Arr { len, .. } => {
            if !fixed_len(len) {
                out.push(len.clone());
            }
        }
        Val::Struct(fs) => fs.iter().for_each(|f| lens(f, out)),
        Val::S(_) | Val::Unit => {}
    }
}

/// The values with each struct replaced by its fields, for `bound_terms`
/// (which bounds an integer and an array, not a struct of them).
fn flat(v: &Val, t: &TypeKind, tb: &Tables, out: &mut Vec<(String, Val, TypeKind)>) {
    match (v, flat_fields(t, tb)) {
        (Val::Struct(vs), Some(fs)) => {
            for (fv, (_, ft)) in vs.iter().zip(&fs) {
                flat(fv, ft, tb, out);
            }
        }
        _ => out.push((String::new(), v.clone(), t.clone())),
    }
}

/// `(and ..)` of terms; `true` of none.
fn all(terms: &[String]) -> String {
    match terms {
        [] => "true".to_string(),
        [one] => one.clone(),
        _ => format!("(and {})", terms.join(" ")),
    }
}

/// A model, as shown and as terms.
struct Model {
    input: Vec<(String, String)>,
    result: Option<String>,
    after: Vec<(String, String)>,
    /// The inputs equal to their values.
    same_input: String,
    /// The result and the values behind `&mut` parameters equal to theirs.
    same_output: String,
}

enum Found {
    Model(Box<Model>),
    /// No model within the lengths shown.
    None,
    Undecided,
}

/// One definition's queries.
struct Gen<'a, 'p, 'q> {
    run_prog: &'a Prog<'p>,
    t: &'a Tables,
    path: &'a str,
    enc: Enc<'p, 'q>,
    axioms: Vec<(String, String)>,
    axiom_parts: Vec<AxiomParts>,
    spec_decls: &'a [(String, String, String)],
    dts: &'a [(String, String, String)],
    solver: &'a mut Option<Solver>,
    solver_down: &'a mut bool,
    spent: u64,
    inputs: Vec<(String, Val, TypeKind)>,
    /// The result (named as the contract names it), when it is not `()`.
    result: Option<(String, Val, TypeKind)>,
    /// The values behind the `&mut` parameters after the function.
    after: Vec<(String, Val, TypeKind)>,
}

impl Gen<'_, '_, '_> {
    /// A model of `facts`, small values first. With `outputs`, the result
    /// and the values after are part of the query and of the model. `nth`:
    /// which input of its kind this is, from 0. Asked first with every
    /// input length at least `nth` (so that the inputs of a kind are not
    /// all of the shortest length), then without that. With `vary`, a
    /// `bool` result is first asked to be `true` for an even `nth` and
    /// `false` for an odd one, before small values (so that the allowed
    /// rows show both where the contract allows both).
    fn ask(&mut self, facts: &[String], outputs: bool, nth: usize, vary: bool) -> Found {
        let mut vals: Vec<&(String, Val, TypeKind)> = self.inputs.iter().collect();
        if outputs {
            vals.extend(self.result.iter());
            vals.extend(self.after.iter());
        }
        let mut caps = Vec::new();
        let mut leaves_of = Vec::new();
        for (_, v, ty) in &vals {
            let mut ls = Vec::new();
            lens(v, &mut ls);
            caps.extend(ls.iter().map(|l| format!("(<= {l} {SHOW})")));
            flat(v, ty, self.t, &mut leaves_of);
        }
        let mut longer = Vec::new();
        for (_, v, _) in &self.inputs {
            let mut ls = Vec::new();
            lens(v, &mut ls);
            longer.extend(ls.iter().map(|l| format!("(<= {} {l})", nth.min(SHOW))));
        }
        let small = bound_terms(&leaves_of);
        // The lengths are bounded by `caps` in every attempt.
        let small: Vec<String> = small.into_iter().filter(|b| !caps.contains(b)).collect();
        if nth == 0 {
            longer.clear();
        }
        let turn = match &self.result {
            Some((_, Val::S(r), ty)) if vary && is_bool(ty) => {
                vec![if nth.is_multiple_of(2) {
                    r.clone()
                } else {
                    format!("(not {r})")
                }]
            }
            _ => Vec::new(),
        };
        // The preferences, dropped one after another.
        let mut tries = Vec::new();
        if !turn.is_empty() {
            if !small.is_empty() {
                tries.push([&caps[..], &small[..], &longer[..], &turn[..]].concat());
            }
            tries.push([&caps[..], &longer[..], &turn[..]].concat());
        }
        if !small.is_empty() {
            if !longer.is_empty() {
                tries.push([&caps[..], &small[..], &longer[..]].concat());
            }
            tries.push([&caps[..], &small[..]].concat());
        }
        tries.push(caps);
        let last = tries.len() - 1;
        for (k, bounds) in tries.into_iter().enumerate() {
            let q = [facts, &bounds[..]].concat();
            match self.decide(&q) {
                Some(true) => return self.settle(q, outputs),
                Some(false) if k == last => return Found::None,
                None if k == last => return Found::Undecided,
                _ => {}
            }
        }
        Found::Undecided
    }

    /// Whether `q` can hold, as W0901 and W0918 ask (`sat_of`); a model is
    /// then the solver's current one.
    fn decide(&mut self, q: &[String]) -> Option<bool> {
        if *self.solver_down {
            return None;
        }
        self.enc.pc = q.to_vec();
        sat_of(&mut SatRun {
            prog: self.run_prog,
            t: self.t,
            path: self.path,
            enc: &self.enc,
            axioms: &self.axioms,
            axiom_parts: &self.axiom_parts,
            spec_decls: self.spec_decls,
            dts: self.dts,
            solver: self.solver,
            solver_down: self.solver_down,
            spent: &mut self.spent,
        })
    }

    /// The model of `q` the solver is at, with each `bool` it leaves open
    /// (`Unread::Open`) fixed by asking again: `q` with it, and where that
    /// cannot hold, `q` without it.
    fn settle(&mut self, mut q: Vec<String>, outputs: bool) -> Found {
        for _ in 0..MAX_OPEN {
            let open = match self.read(outputs) {
                Ok(m) => return Found::Model(Box::new(m)),
                Err(Unread::Not) => return Found::Undecided,
                Err(Unread::Open(b)) => b,
            };
            q.push(open.clone());
            match self.decide(&q) {
                Some(true) => {}
                Some(false) => {
                    q.pop();
                    q.push(format!("(not {open})"));
                    if self.decide(&q) != Some(true) {
                        return Found::Undecided;
                    }
                }
                None => return Found::Undecided,
            }
        }
        Found::Undecided
    }

    /// The current model.
    fn read(&mut self, outputs: bool) -> Result<Model, Unread> {
        let t = self.t;
        let sol = self.solver.as_mut().ok_or(Unread::Not)?;
        let shown = |sol: &mut Solver, xs: &[(String, Val, TypeKind)]| {
            let named = model_values(sol, xs, t).ok_or(Unread::Not)?;
            // One value per parameter (a closure's `self` would be several).
            if named.len() == xs.iter().filter(|(_, v, _)| *v != Val::Unit).count() {
                Ok(named)
            } else {
                Err(Unread::Not)
            }
        };
        let pinned = |sol: &mut Solver, xs: &[(String, Val, TypeKind)]| {
            let mut terms = Vec::new();
            for (_, v, ty) in xs {
                leaves(v, ty, t, &mut terms);
            }
            let asked: Vec<String> = terms.iter().map(|(x, _)| x.clone()).collect();
            let vals = sol.get_value_terms(&asked).map_err(|_| Unread::Not)?;
            let mut vals = terms.into_iter().zip(vals).map(|((x, b), v)| (x, b, v));
            let mut eqs = Vec::new();
            for (_, v, _) in xs {
                pin(v, &mut vals, &mut eqs)?;
            }
            Ok(all(&eqs))
        };
        // The values as terms first: every value shown is a literal.
        let same_input = pinned(sol, &self.inputs)?;
        let input = shown(sol, &self.inputs)?;
        if !outputs {
            return Ok(Model {
                input,
                result: None,
                after: Vec::new(),
                same_input,
                same_output: "true".into(),
            });
        }
        let result: Vec<(String, Val, TypeKind)> = self.result.iter().cloned().collect();
        let outs = [&result[..], &self.after[..]].concat();
        let same_output = pinned(sol, &outs)?;
        Ok(Model {
            input,
            result: shown(sol, &result)?.pop().map(|(_, v)| v),
            after: shown(sol, &self.after)?,
            same_input,
            same_output,
        })
    }
}

/// The most `bool` values of one model fixed by asking again (`settle`).
const MAX_OPEN: usize = 64;

fn row(kind: &'static str, m: &Model) -> Example {
    Example {
        kind,
        input: m.input.clone(),
        result: m.result.clone(),
        after: m.after.clone(),
    }
}

/// The examples of the contracts of `defs` (paths of executable
/// functions), `count` inputs each, in the order given. `Err` names a path
/// that is not an executable function of the program.
pub fn examples(
    p: &Program,
    t: &Tables,
    defs: &[String],
    count: usize,
) -> Result<Vec<Examples>, String> {
    let view = prepare(p, t);
    let t = &view;
    let prog = prog_of(p, t);
    let spec_decls = prog.spec_decls();
    let dts = datatypes(t);
    let mut solver: Option<Solver> = None;
    let mut solver_down = crate::toolchain::check_z3().is_err();
    let spec_syms: Vec<(String, String)> = prog
        .defs
        .iter()
        .filter(|(_, d)| d.f.kind == FnKind::Spec && !d.inst)
        .map(|(p, _)| (p.clone(), sym(p)))
        .collect();
    let mut out = Vec::new();
    for path in defs {
        let d = match prog.defs.get(path) {
            Some(d) if d.f.kind == FnKind::Exec && !d.inst => d,
            _ => return Err(format!("`{path}` is not an executable function")),
        };
        let f = d.f;
        let mut ex = Examples {
            def: path.clone(),
            rows: Vec::new(),
            undecided: 0,
            rlimit: 0,
            reason: None,
        };
        let mut enc = Enc::new(&prog, path, &d.mod_path);
        // A value of a type that is not shown in full is not an example.
        let ret = crate::check::subst(&t.fns[path].ret, &enc.tsub);
        let mut odd = None;
        for (i, prm) in f.params.iter().enumerate() {
            let ty = enc.local_ty(&prm.name.name);
            if enc.text_at(path, Some(i)) || !shown_whole(&ty, t, false) {
                odd = Some(format!(
                    "parameter `{}` of type `{}`: its values are not shown in full",
                    prm.name.name,
                    crate::fmt::type_str(&prm.ty)
                ));
                break;
            }
        }
        if odd.is_none()
            && ret != TypeKind::Unit
            && (enc.text_at(path, None) || !shown_whole(&ret, t, false))
        {
            odd = Some(format!(
                "result of type `{}`: its values are not shown in full",
                crate::fmt::type_str(&f.ret.ty)
            ));
        }
        if odd.is_some() {
            ex.reason = odd;
            out.push(ex);
            continue;
        }
        let mut axioms: Vec<(String, String)> = Vec::new();
        let mut axiom_parts: Vec<AxiomParts> = Vec::new();
        for u in prog.revealed(path, f, &d.mod_path) {
            let (ax, parts) = prog.axiom(&u);
            axioms.push((spec_key(&u, t), ax));
            axiom_parts.extend(parts);
        }
        enc.entry_params(f);
        let inputs = enc.inputs.clone();
        let requires = enc.entry_requires(f);
        // The facts of the entry state: the parameters' types.
        let entry = enc.pc.clone();
        // The state after: a result, and new values behind the `&mut`
        // parameters (their entry values are `old`).
        let (mut env, old) = enc.cur_env();
        let mut after = Vec::new();
        for (prm, (name, before, ty)) in f.params.iter().zip(&inputs) {
            if let TypeKind::Ref {
                mutable: true,
                inner,
            } = &prm.ty.kind
            {
                let v = enc.behind_mut_after(
                    &format!("{name}.after"),
                    before,
                    ty,
                    growable(&inner.kind),
                );
                enc.assume_valid(&v, ty);
                env.insert(name.clone(), v.clone());
                after.push((name.clone(), v, ty.clone()));
            }
        }
        let result = (ret != TypeKind::Unit).then(|| {
            let v = enc.fresh_val("result", &ret);
            enc.assume_valid(&v, &ret);
            let name = f.ret.name.as_ref().map_or("result", |n| n.name.as_str());
            (name.to_string(), v, ret.clone())
        });
        if let (Some(n), Some((_, v, _))) = (&f.ret.name, &result) {
            env.insert(n.name.clone(), v.clone());
        }
        let ensures: Vec<String> = f
            .ensures
            .iter()
            .map(|en| enc.spec(en, &env, &old).s())
            .collect();
        // With the facts of the state after (the result's type).
        let post = enc.pc.clone();
        // A clause through a spec fn seen by name only says nothing a model
        // can show (the value would be the uninterpreted symbol's): no
        // examples, as W0918 does not judge it.
        let mut text = [&post[..], &requires[..], &ensures[..]].concat().join("\n");
        for (_, ax) in &axioms {
            text.push_str(ax);
        }
        let hidden = hidden_predicates(&spec_syms, &axioms, &text, &d.mod_path);
        if !hidden.is_empty() {
            ex.reason = Some(format!(
                "the contract names {} by name only (not in `#[uses(..)]`)",
                hidden
                    .iter()
                    .map(|h| format!("`{h}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            out.push(ex);
            continue;
        }
        // Operations a model does not interpret (`f64`, ADR-0061 §5).
        if text.contains("(f64.") {
            ex.reason = Some("the contract uses `f64` operations".into());
            out.push(ex);
            continue;
        }
        let has_output = result.is_some() || !after.is_empty();
        let mut g = Gen {
            run_prog: &prog,
            t,
            path,
            enc,
            axioms,
            axiom_parts,
            spec_decls: &spec_decls,
            dts: &dts,
            solver: &mut solver,
            solver_down: &mut solver_down,
            spent: 0,
            inputs,
            result,
            after,
        };
        let holds = [&post[..], &requires[..], &ensures[..]].concat();
        let fails = {
            let mut q = [&post[..], &requires[..]].concat();
            q.push(format!("(not {})", all(&ensures)));
            q
        };
        // Allowed, one input after another; for each, a second result the
        // contract allows and a result it rejects.
        let mut seen: Vec<String> = Vec::new();
        let mut rejected = 0;
        for nth in 0..count {
            let m = match g.ask(&[&holds[..], &seen[..]].concat(), true, nth, true) {
                Found::Model(m) => m,
                Found::None => break,
                Found::Undecided => {
                    ex.undecided += 1;
                    break;
                }
            };
            ex.rows.push(row("allowed", &m));
            seen.push(format!("(not {})", m.same_input));
            if !has_output {
                continue;
            }
            let other = [
                &holds[..],
                &[m.same_input.clone(), format!("(not {})", m.same_output)],
            ]
            .concat();
            match g.ask(&other, true, 0, false) {
                Found::Model(m2) => ex.rows.push(row("allowed", &m2)),
                Found::None => {}
                Found::Undecided => ex.undecided += 1,
            }
            if ensures.is_empty() {
                continue;
            }
            match g.ask(
                &[&fails[..], std::slice::from_ref(&m.same_input)].concat(),
                true,
                0,
                false,
            ) {
                Found::Model(m3) => {
                    ex.rows.push(row("rejected", &m3));
                    rejected += 1;
                }
                Found::None => {}
                Found::Undecided => ex.undecided += 1,
            }
        }
        // Rejected at other inputs, while fewer than `count` are shown.
        while has_output && !ensures.is_empty() && rejected < count {
            match g.ask(&[&fails[..], &seen[..]].concat(), true, rejected, false) {
                Found::Model(m) => {
                    ex.rows.push(row("rejected", &m));
                    seen.push(format!("(not {})", m.same_input));
                    rejected += 1;
                }
                Found::None => break,
                Found::Undecided => {
                    ex.undecided += 1;
                    break;
                }
            }
        }
        // Excluded: inputs of the parameters' types the `requires` reject.
        if !requires.is_empty() {
            let mut q = entry.clone();
            q.push(format!("(not {})", all(&requires)));
            for nth in 0..count {
                match g.ask(&q, false, nth, false) {
                    Found::Model(m) => {
                        ex.rows.push(row("excluded", &m));
                        q.push(format!("(not {})", m.same_input));
                    }
                    Found::None => break,
                    Found::Undecided => {
                        ex.undecided += 1;
                        break;
                    }
                }
            }
        }
        ex.rlimit = g.spent;
        out.push(ex);
    }
    Ok(out)
}
