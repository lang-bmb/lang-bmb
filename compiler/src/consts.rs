//! `const` items (ADR-0052, backlog 2V-19): collection, evaluation and
//! the rewrite of array lengths and repeat counts written as a name.
//!
//! The initializer is a constant expression: literals, other `const`s,
//! the integer types' `MIN`/`MAX`, unary and binary operators and
//! comparisons, `if`, the wrapping and saturating integer methods and the
//! checked conversion `x as T`. No call and no loop, so evaluation always
//! ends; it follows Rust's rules and a failure (overflow, division by zero,
//! a shift of the width or more, a conversion out of range) is an error,
//! E0260, with the operation and its operands. A `const` that reaches
//! itself is E0318, another expression E0259.
//!
//! This first part takes integer, `bool` and `char` constants; an array or
//! struct `const` is E0437 until backlog 2V-19b. The checker gives a
//! reference its type and the resolved program the value as a literal at
//! the reference's span (`Checker::const_uses`), so later stages see a
//! literal of the `const`'s type, and a changed value changes the identity
//! of every definition that reads it.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::diag::{Diag, Diags};
use crate::source::Span;

/// A constant's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CVal {
    Int(i128, IntTy),
    Bool(bool),
    Char(u32),
}

impl CVal {
    pub fn ty(self) -> TypeKind {
        match self {
            CVal::Int(_, t) => TypeKind::Int(t),
            CVal::Bool(_) => TypeKind::Bool,
            CVal::Char(_) => TypeKind::Char,
        }
    }

    fn show(self) -> String {
        match self {
            CVal::Int(v, t) => format!("{v}_{}", t.name()),
            CVal::Bool(b) => b.to_string(),
            CVal::Char(c) => format!("{:?}", char::from_u32(c).unwrap_or('?')),
        }
    }
}

/// A `const` item: its unit, declared type and value (`None` when it did
/// not evaluate; the error is reported).
#[derive(Clone, Debug)]
pub struct Const {
    pub unit: String,
    pub ty: TypeKind,
    pub value: Option<CVal>,
}

/// Every `const` of `p` by path (`unit::NAME`), evaluated; `p`'s array
/// lengths and repeat counts written as a name hold the value afterwards.
pub fn resolve(p: &mut Program, diags: &mut Diags) -> BTreeMap<String, Const> {
    let mut defs: BTreeMap<String, (String, ConstDef)> = BTreeMap::new();
    let mut lens: Vec<(Span, Vec<Ident>, String)> = Vec::new();
    fn collect(
        m: &Mod,
        prefix: &str,
        defs: &mut BTreeMap<String, (String, ConstDef)>,
        lens: &mut Vec<(Span, Vec<Ident>, String)>,
        diags: &mut Diags,
    ) {
        let unit = format!("{prefix}{}", m.name.name);
        let mut names: BTreeSet<String> = m
            .fns
            .iter()
            .map(|f| f.name.name.clone())
            .chain(m.types.iter().map(|t| t.name.name.clone()))
            .collect();
        for c in &m.consts {
            if !names.insert(c.name.name.clone()) {
                diags.push(Diag::new(
                    "E0302",
                    c.name.span,
                    format!("`{}` is defined more than once in `{unit}`", c.name.name),
                ));
                continue;
            }
            defs.insert(
                format!("{unit}::{}", c.name.name),
                (unit.clone(), c.clone()),
            );
        }
        for (sp, path) in &m.lens {
            lens.push((*sp, path.clone(), unit.clone()));
        }
        for s in &m.mods {
            collect(s, &format!("{unit}::"), defs, lens, diags);
        }
    }
    for m in &p.mods {
        collect(m, "", &mut defs, &mut lens, diags);
    }
    let mut ev = Eval {
        defs: &defs,
        done: BTreeMap::new(),
        stack: Vec::new(),
        diags,
    };
    let paths: Vec<String> = defs.keys().cloned().collect();
    for path in &paths {
        ev.konst(path);
    }
    let Eval { done, diags, .. } = ev;
    let consts: BTreeMap<String, Const> = defs
        .iter()
        .map(|(path, (unit, c))| {
            (
                path.clone(),
                Const {
                    unit: unit.clone(),
                    ty: c.ty.kind.clone(),
                    value: done.get(path).copied().flatten(),
                },
            )
        })
        .collect();

    // `[T; N]` and `[e; N]`: the length, a `usize` constant.
    let length = |path: &[String], unit: &str, span: Span, diags: &mut Diags| -> Option<u64> {
        let Some(full) = lookup(&consts, path, unit) else {
            diags.push(
                Diag::new(
                    "E0301",
                    span,
                    format!("unknown constant `{}`", path.join("::")),
                )
                .fix("an array length is a literal or a `usize` `const`", None),
            );
            return None;
        };
        match consts[&full].value {
            Some(CVal::Int(v, IntTy::Usize)) if v <= u32::MAX as i128 => Some(v as u64),
            Some(CVal::Int(_, IntTy::Usize)) => {
                diags.push(Diag::new("E0406", span, "array length is too large"));
                None
            }
            Some(v) => {
                diags.push(Diag::new(
                    "E0401",
                    span,
                    format!(
                        "an array length is a `usize`, `{}` is a `{}`",
                        path.join("::"),
                        crate::check::ty_str(&v.ty())
                    ),
                ));
                None
            }
            None => None,
        }
    };
    // The unit a type is written in: the innermost unit whose span holds it.
    let mut mod_spans: Vec<(Span, String)> = Vec::new();
    fn mspans(m: &Mod, prefix: &str, out: &mut Vec<(Span, String)>) {
        let unit = format!("{prefix}{}", m.name.name);
        out.push((m.span, unit.clone()));
        for s in &m.mods {
            mspans(s, &format!("{unit}::"), out);
        }
    }
    for m in &p.mods {
        mspans(m, "", &mut mod_spans);
    }
    let unit_of = |sp: Span| -> String {
        mod_spans
            .iter()
            .filter(|(m, _)| m.file == sp.file && m.lo <= sp.lo && sp.hi <= m.hi)
            .max_by_key(|(m, _)| m.lo)
            .map(|(_, u)| u.clone())
            .unwrap_or_default()
    };
    crate::ast::types_mut(p, &mut |t| {
        fn walk(t: &mut Type, f: &mut dyn FnMut(&mut Type)) {
            match &mut t.kind {
                TypeKind::Array(e, _)
                | TypeKind::ArrayN(e, _)
                | TypeKind::Slice(e)
                | TypeKind::Vec(e)
                | TypeKind::Box(e) => walk(e, f),
                TypeKind::Ref { inner, .. } => walk(inner, f),
                TypeKind::Named { args, .. } => args.iter_mut().for_each(|a| walk(a, f)),
                _ => {}
            }
            f(t);
        }
        walk(t, &mut |t| {
            if let TypeKind::ArrayN(e, path) = &t.kind {
                let unit = unit_of(t.span);
                t.kind = match length(path, &unit, t.span, diags) {
                    Some(n) => TypeKind::Array(e.clone(), n),
                    None => TypeKind::Error,
                };
            }
        });
    });
    let counts: BTreeMap<(u32, u32, u32), u128> = lens
        .iter()
        .filter_map(|(sp, path, unit)| {
            let names: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
            length(&names, unit, *sp, diags).map(|n| ((sp.file, sp.lo, sp.hi), n as u128))
        })
        .collect();
    crate::ast::exprs_mut(p, &mut |e| {
        if let ExprKind::Repeat {
            count, count_span, ..
        } = &mut e.kind
            && let Some(n) = counts.get(&(count_span.file, count_span.lo, count_span.hi))
        {
            *count = *n;
        }
    });
    consts
}

/// `v` as an expression: a literal (a negative integer is `-` of one, at
/// the same span, so both have the `const`'s type).
pub fn literal(v: CVal, span: Span) -> ExprKind {
    match v {
        CVal::Int(n, _) if n < 0 => ExprKind::Unary(
            UnOp::Neg,
            Box::new(Expr {
                kind: ExprKind::Int(n.unsigned_abs(), Radix::Dec),
                span,
            }),
        ),
        CVal::Int(n, _) => ExprKind::Int(n as u128, Radix::Dec),
        CVal::Bool(b) => ExprKind::Bool(b),
        CVal::Char(c) => ExprKind::CharLit(c),
    }
}

/// The `const` that `path` names from `unit`: `N` in its own unit, or a
/// full path.
pub fn lookup(consts: &BTreeMap<String, Const>, path: &[String], unit: &str) -> Option<String> {
    let local = format!("{unit}::{}", path.join("::"));
    if path.len() == 1 && consts.contains_key(&local) {
        return Some(local);
    }
    let full = path.join("::");
    consts.contains_key(&full).then_some(full)
}

struct Eval<'a> {
    defs: &'a BTreeMap<String, (String, ConstDef)>,
    /// Evaluated (`None`: failed, reported).
    done: BTreeMap<String, Option<CVal>>,
    /// The `const`s being evaluated, outermost first.
    stack: Vec<String>,
    diags: &'a mut Diags,
}

/// The declared type's kind, if a `const` may have it.
fn scalar(t: &TypeKind) -> bool {
    matches!(t, TypeKind::Int(_) | TypeKind::Bool | TypeKind::Char)
}

impl Eval<'_> {
    fn konst(&mut self, path: &str) -> Option<CVal> {
        if let Some(v) = self.done.get(path) {
            return *v;
        }
        let (unit, c) = self.defs[path].clone();
        if self.stack.iter().any(|p| p == path) {
            let cycle: Vec<&str> = self
                .stack
                .iter()
                .skip_while(|p| *p != path)
                .map(String::as_str)
                .chain(std::iter::once(path))
                .collect();
            self.diags.push(
                Diag::new(
                    "E0318",
                    c.name.span,
                    format!("constants refer to each other: {}", cycle.join(" -> ")),
                )
                .fix(
                    "give one of them a value that names none of the others",
                    None,
                ),
            );
            self.done.insert(path.to_string(), None);
            return None;
        }
        if !scalar(&c.ty.kind) {
            let what = match &c.ty.kind {
                TypeKind::Array(..) | TypeKind::ArrayN(..) | TypeKind::Named { .. } => {
                    "an array or data `const` is not available yet (backlog 2V-19b)"
                }
                _ => "a `const` holds a value type: an integer, `bool` or `char` (ADR-0052 §2)",
            };
            self.diags.push(
                Diag::new(
                    "E0437",
                    c.ty.span,
                    format!(
                        "`{}` cannot be a `const`: {what}",
                        crate::check::ty_str(&c.ty.kind)
                    ),
                )
                .fix(
                    "write the value where it is used, or name each scalar part as its own `const`",
                    None,
                ),
            );
            self.done.insert(path.to_string(), None);
            return None;
        }
        self.stack.push(path.to_string());
        let v = self.expr(&c.value, Some(&c.ty.kind), &unit);
        self.stack.pop();
        let v = v.and_then(|v| {
            if v.ty() == c.ty.kind {
                Some(v)
            } else {
                self.diags.push(Diag::new(
                    "E0401",
                    c.value.span,
                    format!(
                        "expected `{}`, found `{}`",
                        crate::check::ty_str(&c.ty.kind),
                        crate::check::ty_str(&v.ty())
                    ),
                ));
                None
            }
        });
        self.done.insert(path.to_string(), v);
        v
    }

    fn not_const(&mut self, e: &Expr, what: &str) -> Option<CVal> {
        self.diags.push(
            Diag::new(
                "E0259",
                e.span,
                format!(
                    "`{}` is not a constant expression: {what}",
                    crate::fmt::expr_text(e)
                ),
            )
            .fix(
                "a `const` is built from literals, other `const`s, operators, `if`, the wrapping and saturating methods and `as` (ADR-0052 §3)",
                None,
            ),
        );
        None
    }

    fn fail(&mut self, span: Span, what: String) -> Option<CVal> {
        self.diags.push(
            Diag::new("E0260", span, format!("constant evaluation failed: {what}")).fix(
                "a `const` is evaluated as Rust evaluates it: change the value, or give the `const` a type that holds it",
                None,
            ),
        );
        None
    }

    fn int(&mut self, v: i128, t: IntTy, span: Span, op: &str) -> Option<CVal> {
        if v < t.min_value() || v > t.max_value() {
            return self.fail(span, format!("{op} overflows `{}`", t.name()));
        }
        Some(CVal::Int(v, t))
    }

    /// A literal is untyped until its context gives the type.
    fn untyped(e: &Expr) -> bool {
        matches!(e.kind, ExprKind::Int(..))
            || matches!(&e.kind, ExprKind::Unary(UnOp::Neg, x) if matches!(x.kind, ExprKind::Int(..)))
    }

    fn expr(&mut self, e: &Expr, want: Option<&TypeKind>, unit: &str) -> Option<CVal> {
        match &e.kind {
            ExprKind::Int(v, _) => {
                let t = match want {
                    Some(TypeKind::Int(t)) => *t,
                    // An integer literal without a context is an `i64`,
                    // as the checker reads it.
                    _ => IntTy::I64,
                };
                if *v > i128::MAX as u128 {
                    return self.fail(e.span, format!("`{v}` overflows `{}`", t.name()));
                }
                self.int(*v as i128, t, e.span, &format!("`{v}`"))
            }
            ExprKind::Bool(b) => Some(CVal::Bool(*b)),
            ExprKind::CharLit(c) => Some(CVal::Char(*c)),
            ExprKind::ByteLit(b) => Some(CVal::Int(*b as i128, IntTy::U8)),
            ExprKind::Var(id) => {
                let path = vec![id.name.clone()];
                match lookup_defs(self.defs, &path, unit) {
                    Some(p) => self.konst(&p),
                    None => self.not_const(e, "only other `const`s are named"),
                }
            }
            ExprKind::Path(path) => {
                if let Some((TypeKind::Int(t), v)) = crate::check::int_const(path) {
                    return Some(CVal::Int(v, t));
                }
                let names: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
                match lookup_defs(self.defs, &names, unit) {
                    Some(p) => self.konst(&p),
                    None => self.not_const(e, "only other `const`s are named"),
                }
            }
            ExprKind::Unary(UnOp::Neg, x) => match self.expr(x, want, unit)? {
                CVal::Int(v, t) if t.signed() => self.int(-v, t, e.span, "`-`"),
                v => self.fail(e.span, format!("`-` of `{}`", v.show())),
            },
            ExprKind::Unary(UnOp::Not, x) => match self.expr(x, want, unit)? {
                CVal::Bool(b) => Some(CVal::Bool(!b)),
                CVal::Int(v, t) if t.signed() => Some(CVal::Int(!v, t)),
                CVal::Int(v, t) => Some(CVal::Int(t.max_value() ^ v, t)),
                v => self.fail(e.span, format!("`!` of `{}`", v.show())),
            },
            ExprKind::Binary(op, a, b) => self.binary(e, *op, a, b, want, unit),
            ExprKind::If { cond, then, els } => {
                let Some(els) = els else {
                    return self.not_const(e, "an `if` without `else`");
                };
                if !then.stmts.is_empty() {
                    return self.not_const(e, "a block with statements");
                }
                let Some(t) = &then.tail else {
                    return self.not_const(e, "an empty block");
                };
                match self.expr(cond, Some(&TypeKind::Bool), unit)? {
                    CVal::Bool(true) => self.expr(t, want, unit),
                    CVal::Bool(false) => self.expr(els, want, unit),
                    v => self.fail(cond.span, format!("the condition is `{}`", v.show())),
                }
            }
            ExprKind::Block(b) if b.stmts.is_empty() && b.tail.is_some() => {
                self.expr(b.tail.as_ref().expect("a tail"), want, unit)
            }
            ExprKind::Method { recv, name, args } => self.method(e, recv, name, args, want, unit),
            _ => self.not_const(e, "not one of the forms of ADR-0052 §3"),
        }
    }

    fn binary(
        &mut self,
        e: &Expr,
        op: BinOp,
        a: &Expr,
        b: &Expr,
        want: Option<&TypeKind>,
        unit: &str,
    ) -> Option<CVal> {
        use BinOp::*;
        let cmp = matches!(op, Eq | Ne | Lt | Le | Gt | Ge);
        if matches!(op, And | Or) {
            let x = self.expr(a, Some(&TypeKind::Bool), unit)?;
            return match (op, x) {
                (And, CVal::Bool(false)) => Some(CVal::Bool(false)),
                (Or, CVal::Bool(true)) => Some(CVal::Bool(true)),
                (_, CVal::Bool(_)) => match self.expr(b, Some(&TypeKind::Bool), unit)? {
                    v @ CVal::Bool(_) => Some(v),
                    v => self.fail(b.span, format!("`{}` is not a `bool`", v.show())),
                },
                (_, v) => self.fail(a.span, format!("`{}` is not a `bool`", v.show())),
            };
        }
        if op == Implies {
            return self.not_const(e, "`==>` belongs to specifications");
        }
        if matches!(op, Shl | Shr) {
            let x = self.expr(a, want, unit)?;
            // The amount is any integer type; a literal amount is a `u32`.
            let k = self.expr(b, Some(&TypeKind::Int(IntTy::U32)), unit)?;
            let (CVal::Int(v, t), CVal::Int(k, _)) = (x, k) else {
                return self.fail(e.span, "a shift of integers".into());
            };
            if k < 0 || k >= t.bits() as i128 {
                return self.fail(
                    e.span,
                    format!(
                        "`{}` by {k}: the amount is not below {}",
                        op.symbol(),
                        t.bits()
                    ),
                );
            }
            let r = if op == Shl { wrap(v << k, t) } else { v >> k };
            return Some(CVal::Int(r, t));
        }
        // The operands have one type; an untyped literal takes the other's.
        let hint = if cmp { None } else { want };
        let (x, y) = if Self::untyped(a) && !Self::untyped(b) {
            let y = self.expr(b, hint, unit)?;
            let x = self.expr(a, Some(&y.ty()), unit)?;
            (x, y)
        } else {
            let x = self.expr(a, hint, unit)?;
            let y = self.expr(b, Some(&x.ty()), unit)?;
            (x, y)
        };
        if x.ty() != y.ty() {
            self.diags.push(Diag::new(
                "E0401",
                b.span,
                format!(
                    "expected `{}`, found `{}`",
                    crate::check::ty_str(&x.ty()),
                    crate::check::ty_str(&y.ty())
                ),
            ));
            return None;
        }
        if cmp {
            let o = match (x, y) {
                (CVal::Int(p, _), CVal::Int(q, _)) => p.cmp(&q),
                (CVal::Bool(p), CVal::Bool(q)) => p.cmp(&q),
                (CVal::Char(p), CVal::Char(q)) => p.cmp(&q),
                _ => unreachable!("same types"),
            };
            use std::cmp::Ordering as O;
            let r = match op {
                Eq => o == O::Equal,
                Ne => o != O::Equal,
                Lt => o == O::Less,
                Le => o != O::Greater,
                Gt => o == O::Greater,
                _ => o != O::Less,
            };
            return Some(CVal::Bool(r));
        }
        match (x, y) {
            (CVal::Bool(p), CVal::Bool(q)) => match op {
                BitAnd => Some(CVal::Bool(p & q)),
                BitOr => Some(CVal::Bool(p | q)),
                BitXor => Some(CVal::Bool(p ^ q)),
                _ => self.fail(e.span, format!("`{}` of `bool`s", op.symbol())),
            },
            (CVal::Int(p, t), CVal::Int(q, _)) => {
                let shown = format!("`{} {} {}`", x.show(), op.symbol(), y.show());
                match op {
                    Add => self.int(p + q, t, e.span, &shown),
                    Sub => self.int(p - q, t, e.span, &shown),
                    Mul => match p.checked_mul(q) {
                        Some(r) => self.int(r, t, e.span, &shown),
                        None => self.fail(e.span, format!("{shown} overflows `{}`", t.name())),
                    },
                    Div | Rem if q == 0 => self.fail(e.span, format!("{shown} divides by zero")),
                    Div => self.int(p / q, t, e.span, &shown),
                    Rem => self.int(p % q, t, e.span, &shown),
                    BitAnd => Some(CVal::Int(p & q, t)),
                    BitOr => Some(CVal::Int(p | q, t)),
                    BitXor => Some(CVal::Int(p ^ q, t)),
                    _ => self.fail(e.span, format!("`{}` of integers", op.symbol())),
                }
            }
            _ => self.fail(e.span, format!("`{}` of `{}`", op.symbol(), x.show())),
        }
    }

    fn method(
        &mut self,
        e: &Expr,
        recv: &Expr,
        name: &Ident,
        args: &[Expr],
        want: Option<&TypeKind>,
        unit: &str,
    ) -> Option<CVal> {
        // `x as T`, as parsed: `T::try_from(x).unwrap()` (ADR-0095 A4).
        if name.name == "unwrap"
            && args.is_empty()
            && let ExprKind::Call {
                path, args: cargs, ..
            } = &recv.kind
            && let [ty, f] = path.as_slice()
            && f.name == "try_from"
            && let Some(t) = IntTy::from_name(&ty.name)
            && let [x] = cargs.as_slice()
        {
            return match self.expr(x, None, unit)? {
                CVal::Int(v, _) => self.int(v, t, e.span, &format!("`{v} as {}`", t.name())),
                v => self.fail(e.span, format!("`{} as {}`", v.show(), t.name())),
            };
        }
        let ops = [
            "wrapping_add",
            "wrapping_sub",
            "wrapping_mul",
            "saturating_add",
            "saturating_sub",
            "saturating_mul",
        ];
        if !ops.contains(&name.name.as_str()) || args.len() != 1 {
            return self.not_const(e, "a method other than the wrapping and saturating ones");
        }
        let x = self.expr(recv, want, unit)?;
        let y = self.expr(&args[0], Some(&x.ty()), unit)?;
        let (CVal::Int(p, t), CVal::Int(q, u)) = (x, y) else {
            return self.fail(e.span, format!("`{}` of non-integers", name.name));
        };
        if t != u {
            self.diags.push(Diag::new(
                "E0401",
                args[0].span,
                format!("expected `{}`, found `{}`", t.name(), u.name()),
            ));
            return None;
        }
        let exact = match &name.name[name.name.len() - 3..] {
            "add" => p + q,
            "sub" => p - q,
            _ => p.saturating_mul(q),
        };
        let r = if name.name.starts_with("wrapping") {
            match &name.name[name.name.len() - 3..] {
                "mul" => wrap(p.wrapping_mul(q), t),
                _ => wrap(exact, t),
            }
        } else {
            exact.clamp(t.min_value(), t.max_value())
        };
        Some(CVal::Int(r, t))
    }
}

fn lookup_defs(
    defs: &BTreeMap<String, (String, ConstDef)>,
    path: &[String],
    unit: &str,
) -> Option<String> {
    let local = format!("{unit}::{}", path.join("::"));
    if path.len() == 1 && defs.contains_key(&local) {
        return Some(local);
    }
    let full = path.join("::");
    defs.contains_key(&full).then_some(full)
}

/// `v` reduced into `t`'s range (two's complement).
fn wrap(v: i128, t: IntTy) -> i128 {
    let bits = t.bits();
    let m = 1i128 << bits;
    let r = v.rem_euclid(m);
    if t.signed() && r >= m / 2 { r - m } else { r }
}
