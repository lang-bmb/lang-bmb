//! Proof gate (backlog 1B-02 .. 1B-11, semantics in ADR-0017).
//!
//! Each executable function is symbolically executed over its checked syntax
//! tree. Local variables become single-assignment SMT constants; loops are cut
//! by their invariants; calls assume only the callee's `ensures`. Every implicit
//! and explicit proof obligation is checked separately by Z3. Only `unsat`
//! proves an obligation.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
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
    A { elems: String, len: String },
    Unit,
}

impl Val {
    fn s(&self) -> String {
        match self {
            Val::S(t) => t.clone(),
            Val::Unit => "true".into(),
            Val::A { .. } => panic!("array used as scalar"),
        }
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
    /// Entry constants of the parameters, for counterexamples: (name, terms).
    inputs: Vec<(String, Val)>,
    /// Spec translation: enclosing conditions, and the recursion check context.
    spec_pc: Vec<String>,
    term_check: Option<(usize, String)>,
    forall_depth: u32,
    triggers: Vec<Vec<String>>,
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

fn sort_of(t: &TypeKind) -> &'static str {
    match t {
        TypeKind::Bool => "Bool",
        _ => "Int",
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
    if *elem == TypeKind::Bool {
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

    fn fresh_val(&mut self, base: &str, t: &TypeKind) -> Val {
        if is_array(t) {
            let e = elem_ty(t);
            let elems = self.fresh(base, &format!("(Array Int {})", sort_of(&e)));
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
            let c = self.fresh(base, sort_of(t));
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

    fn resolve(&self, path: &[Ident], mod_path: &str) -> String {
        if path.len() == 1 {
            let local = format!("{mod_path}::{}", path[0].name);
            if self.prog.t.fns.contains_key(&local) {
                return local;
            }
            return path[0].name.clone();
        }
        path.iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::")
    }

    // ---------- specification terms ----------

    fn spec(&mut self, e: &Expr, env: &BTreeMap<String, Val>, old: &BTreeMap<String, Val>) -> Val {
        match &e.kind {
            ExprKind::Int(v) => Val::S(num(*v as i128)),
            ExprKind::Bool(b) => Val::S(b.to_string()),
            ExprKind::Unit => Val::Unit,
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
                let Val::A { elems, .. } = self.spec(base, env, old) else {
                    panic!("index of non-array")
                };
                let i = self.spec(index, env, old).s();
                Val::S(format!("(select {elems} {i})"))
            }
            ExprKind::Method { recv, name, args } => {
                let rv = self.spec(recv, env, old);
                match name.name.as_str() {
                    "len" => match rv {
                        Val::A { len, .. } => Val::S(len),
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
            ExprKind::Call { path, args } => {
                let target = self.resolve(path, &self.spec_mod.clone());
                let mut flat = Vec::new();
                let mut argv = Vec::new();
                for a in args {
                    let v = self.spec(a, env, old);
                    match &v {
                        Val::A { elems, len } => {
                            flat.push(elems.clone());
                            flat.push(len.clone());
                        }
                        Val::S(t) => flat.push(t.clone()),
                        Val::Unit => {}
                    }
                    argv.push(v);
                }
                self.check_spec_recursion(&target, &argv, e.span);
                Val::S(if flat.is_empty() {
                    sym(&target)
                } else {
                    format!("({} {})", sym(&target), flat.join(" "))
                })
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
                Val::S(format!(
                    "(forall ({}) (! {inner} :pattern ({})))",
                    decl.join(" "),
                    pats.join(" ")
                ))
            }
            ExprKind::Trigger(x) => {
                let v = self.spec(x, env, old);
                if let Some(top) = self.triggers.last_mut() {
                    top.push(v.s());
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
                match (a, b) {
                    (Val::S(a), Val::S(b)) => Val::S(ite(&c, &a, &b)),
                    _ => Val::Unit,
                }
            }
            ExprKind::Block(b) => self.spec_block(b, env, old),
            ExprKind::Array(_)
            | ExprKind::Repeat { .. }
            | ExprKind::Ref { .. }
            | ExprKind::Error => {
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
                self.pc.push(guard);
                let rt = self.eval(r).s();
                self.pc.truncate(n);
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
                let Val::A { elems, len } = self.eval(base) else {
                    panic!("index of non-array")
                };
                let i = self.eval(index).s();
                self.oblige(
                    "E0805",
                    e.span,
                    "index may be out of bounds",
                    format!("(and (<= 0 {i}) (< {i} {len}))"),
                );
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
                        Val::A { len, .. } => Val::S(len),
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
            ExprKind::Call { path, args } => self.call(path, args, e.span),
            ExprKind::If { cond, then, els } => {
                let c = self.eval(cond).s();
                self.branch(
                    &c,
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
        }
    }

    /// Runs two branches from the same state and merges them under `c`.
    fn branch(
        &mut self,
        c: &str,
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
            let v = match (a, b) {
                (Val::S(x), Val::S(y)) => Val::S(ite(c, x, y)),
                (Val::A { elems: e1, len: l1 }, Val::A { elems: e2, len: l2 }) => Val::A {
                    elems: ite(c, e1, e2),
                    len: ite(c, l1, l2),
                },
                _ => a.clone(),
            };
            merged.insert(k.clone(), v);
        }
        self.vars = merged;
        if !then_facts.is_empty() {
            self.pc.push(format!("(=> {c} {})", and(&then_facts)));
        }
        if !else_facts.is_empty() {
            self.pc.push(format!("(=> (not {c}) {})", and(&else_facts)));
        }
        match (tv, ev) {
            (Val::S(a), Val::S(b)) => Val::S(ite(c, &a, &b)),
            _ => Val::Unit,
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
            Stmt::Let { name, init, .. } => {
                let v = self.eval(init);
                self.vars.insert(name.name.clone(), v);
            }
            Stmt::Assign { target, value, .. } => match &target.kind {
                ExprKind::Var(id) => {
                    let v = self.eval(value);
                    self.vars.insert(id.name.clone(), v);
                }
                ExprKind::Index { base, index } => {
                    let ExprKind::Var(id) = &base.kind else {
                        unreachable!()
                    };
                    let i = self.eval(index).s();
                    let v = self.eval(value).s();
                    let Val::A { elems, len } = self.vars[&id.name].clone() else {
                        unreachable!()
                    };
                    self.oblige(
                        "E0805",
                        target.span,
                        "index may be out of bounds",
                        format!("(and (<= 0 {i}) (< {i} {len}))"),
                    );
                    self.vars.insert(
                        id.name.clone(),
                        Val::A {
                            elems: format!("(store {elems} {i} {v})"),
                            len,
                        },
                    );
                }
                _ => unreachable!(),
            },
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
                        elems: self.fresh(v, &format!("(Array Int {})", sort_of(&e))),
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

    fn local_ty(&self, name: &str) -> TypeKind {
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
            // Built-in (`print_i64`): no contract.
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
        // `&mut` arguments: the callee may change the elements, never the length.
        let mut post_args = argv.clone();
        let mut old_env = BTreeMap::new();
        for (i, p) in f.params.iter().enumerate() {
            if let TypeKind::Ref { mutable: true, .. } = p.ty.kind
                && let Some(root) = root_var(&args[i])
                && let Val::A { len, .. } = &argv[i]
            {
                let e = elem_ty(&p.ty.kind);
                let elems = self.fresh(&root, &format!("(Array Int {})", sort_of(&e)));
                let nv = Val::A {
                    elems,
                    len: len.clone(),
                };
                old_env.insert(p.name.name.clone(), argv[i].clone());
                self.vars.insert(root, nv.clone());
                post_args[i] = nv;
            }
        }
        let mut post_env = bind_params(f, &post_args);
        let result = self.fresh_val(&format!("{target}.result"), &f.ret.ty.kind);
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
        ExprKind::Ref { expr, .. } => root_var(expr),
        _ => None,
    }
}

fn find_let(b: &Block, name: &str, out: &mut Option<TypeKind>) {
    for s in &b.stmts {
        match s {
            Stmt::Let {
                name: n, ty, init, ..
            } => {
                if n.name == name {
                    *out = Some(ty.kind.clone());
                }
                find_let_expr(init, name, out);
            }
            Stmt::While { body, .. } => find_let(body, name, out),
            Stmt::Expr(e) => find_let_expr(e, name, out),
            _ => {}
        }
    }
    if let Some(t) = &b.tail {
        find_let_expr(t, name, out);
    }
}

fn find_let_expr(e: &Expr, name: &str, out: &mut Option<TypeKind>) {
    match &e.kind {
        ExprKind::If { then, els, .. } => {
            find_let(then, name, out);
            if let Some(x) = els {
                find_let_expr(x, name, out);
            }
        }
        ExprKind::Block(b) => find_let(b, name, out),
        _ => {}
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
            Stmt::Let { init, .. } => assigned_in_expr(init, out),
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
}

fn param_sorts(f: &FnDef) -> Vec<String> {
    let mut v = Vec::new();
    for p in &f.params {
        if is_array(&p.ty.kind) {
            v.push(format!("(Array Int {})", sort_of(&elem_ty(&p.ty.kind))));
            v.push("Int".into());
        } else {
            v.push(sort_of(&p.ty.kind).into());
        }
    }
    v
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
    fn spec_decls(&self) -> Vec<String> {
        self.defs
            .iter()
            .filter(|(_, d)| d.f.kind == FnKind::Spec)
            .map(|(path, d)| {
                format!(
                    "(declare-fun {} ({}) {})",
                    sym(path),
                    param_sorts(d.f).join(" "),
                    sort_of(&d.f.ret.ty.kind)
                )
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
        for p in &d.f.params {
            let base = format!("{}!p", p.name.name);
            if is_array(&p.ty.kind) {
                let e = sym(&format!("{base}.e"));
                let l = sym(&format!("{base}.len"));
                binders.push(format!(
                    "({e} (Array Int {}))",
                    sort_of(&elem_ty(&p.ty.kind))
                ));
                binders.push(format!("({l} Int)"));
                facts.push(format!(
                    "(and (<= 0 {l}) (<= {l} {}))",
                    len_max(&elem_ty(&p.ty.kind))
                ));
                app_args.push(e.clone());
                app_args.push(l.clone());
                env.insert(p.name.name.clone(), Val::A { elems: e, len: l });
            } else {
                let s = sym(&base);
                binders.push(format!("({s} {})", sort_of(&p.ty.kind)));
                if let Some(r) = range(&p.ty.kind, &s) {
                    facts.push(r);
                }
                app_args.push(s.clone());
                env.insert(p.name.name.clone(), Val::S(s));
            }
        }
        let body = enc.spec_block(&d.f.body, &env, &BTreeMap::new()).s();
        let app = if app_args.is_empty() {
            sym(path)
        } else {
            format!("({} {})", sym(path), app_args.join(" "))
        };
        let guard = and(&facts);
        let ax = if binders.is_empty() {
            format!("(assert (= {app} {body}))")
        } else {
            format!(
                "(assert (forall ({}) (! (=> {guard} (= {app} {body})) :pattern ({app}))))",
                binders.join(" "),
            )
        };
        let parts = AxiomParts {
            sym: sym(path),
            params: app_args,
            guard,
            body,
        };
        (ax, enc.unsupported, parts)
    }
}

fn uses_of(f: &FnDef, mod_path: &str, fns: &BTreeMap<String, FnSig>) -> Vec<String> {
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
    for (path, d) in &prog.defs {
        let mut enc = Enc::new(&prog, path, &d.mod_path);
        let mut header = vec![
            "(set-option :produce-models true)".to_string(),
            "(set-option :smt.random_seed 0)".to_string(),
            format!("(set-option :rlimit {})", smt::rlimit()),
            PRELUDE.to_string(),
        ];
        header.extend(spec_decls.iter().cloned());
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
                    let v = enc.fresh_val(&prm.name.name, &prm.ty.kind);
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
                    axioms.push((sym(&u), ax));
                    axiom_parts.push(parts);
                    enc.unsupported.extend(unsup);
                }
                for prm in &d.f.params {
                    let v = enc.fresh_val(&prm.name.name, &prm.ty.kind);
                    if let TypeKind::Ref { mutable: true, .. } = prm.ty.kind {
                        enc.old.insert(prm.name.name.clone(), v.clone());
                    }
                    enc.inputs.push((prm.name.name.clone(), v.clone()));
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

        // Assemble and solve. Every obligation runs on a reset solver holding only
        // the definition's base (options, prelude, axioms, declarations), so one
        // obligation's result never depends on another's (ADR-0017 §5).
        let mut base = header.join("\n");
        base.push('\n');
        let decls = enc.decls.join("\n");
        let mut script = String::new();
        let mut failed = 0usize;
        let (mut rlimit_total, mut rlimit_max) = (0u64, 0u64);
        let mut solver = None;
        if !enc.obls.is_empty() {
            match Solver::start() {
                Ok(s) => solver = Some(s),
                Err(e) => {
                    diags.push(Diag::new(
                        "E0812",
                        d.f.name.span,
                        format!("solver error: {e}"),
                    ));
                    failed = enc.obls.len();
                }
            }
        }
        for (k, o) in enc.obls.iter().enumerate() {
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
            let head = format!("{base}{decls}");
            let base = format!("{base}{relevant}{decls}");
            script.push_str(&format!(
                "; obligation {k}: {} {}\n(reset)\n{base}\n{q}\n(check-sat)\n",
                o.code, o.msg
            ));
            let Some(sol) = solver.as_mut() else { continue };
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
            }
            // Undecided with quantified definitions: retry quantifier-free with
            // ground instances (sound for both answers, see `ground_instances`).
            if matches!(answer, Ok(Answer::Unknown(_))) && !axiom_parts.is_empty() {
                let (inst, complete) = ground_instances(&axiom_parts, &q, 3);
                if !inst.is_empty() {
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
            let key = (o.code, o.span.file, o.span.lo, o.span.hi);
            if matches!(answer, Ok(Answer::Unsat)) {
                proved_keys.insert(key);
            } else {
                failed_keys.insert(key);
            }
            match answer {
                Ok(Answer::Unsat) => failed -= 1,
                Ok(Answer::Sat) => {
                    let cex = counterexample(sol, &enc.inputs);
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
                    let cex = counterexample(sol, &enc.inputs);
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
                }
            }
        }
        statuses.push(FnStatus {
            path: path.clone(),
            obligations: enc.obls.len(),
            failed: failed + enc.unsupported.len(),
            rlimit_total,
            rlimit_max,
        });
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

/// `x = 5, a = [1, 2] (len 2)` from the current model.
fn counterexample(sol: &mut Solver, inputs: &[(String, Val)]) -> String {
    const SHOW: usize = 4;
    let mut terms = Vec::new();
    for (_, v) in inputs {
        match v {
            Val::S(t) => terms.push(t.clone()),
            Val::A { elems, len } => {
                terms.push(len.clone());
                for k in 0..SHOW {
                    terms.push(format!("(select {elems} {k})"));
                }
            }
            Val::Unit => {}
        }
    }
    let Ok(vals) = sol.get_values(&terms) else {
        return String::new();
    };
    let mut it = vals.into_iter();
    let mut parts = Vec::new();
    for (name, v) in inputs {
        match v {
            Val::S(_) => parts.push(format!("{name} = {}", it.next().unwrap_or_default())),
            Val::A { .. } => {
                let len = it.next().unwrap_or_default();
                let n: usize = len.parse().unwrap_or(SHOW).min(SHOW);
                let elems: Vec<String> = (0..SHOW).filter_map(|_| it.next()).take(n).collect();
                let more = if len.parse::<usize>().map(|l| l > SHOW).unwrap_or(false) {
                    ", .."
                } else {
                    ""
                };
                parts.push(format!("{name} = [{}{more}] (len {len})", elems.join(", ")));
            }
            Val::Unit => {}
        }
    }
    parts.join(", ")
}
