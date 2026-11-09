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
//! A `const` holds a value type (ADR-0052 §2): an integer, `bool`, `char`,
//! or an array, struct or enum of them (backlog 2V-19b). The initializer
//! also builds arrays (`[a, b]`, `[e; N]`), structs and variants, and reads
//! fields, elements (`T[i]`, out of range is E0260) and `match`es a
//! variant. The evaluator types its literals from the declared type; the
//! checker checks an array or data `const`'s initializer against it
//! (`Checker::check_const`), so a wrong field or variant is the checker's
//! error. The checker gives a reference its type and the resolved program
//! the value as a literal tree at the reference (`Checker::const_uses`,
//! `literal_tree`), so later stages see literals of the `const`'s type,
//! and a changed value changes the identity of every definition that reads
//! it.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::diag::{Diag, Diags};
use crate::source::Span;

/// A constant's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CVal {
    Int(i128, IntTy),
    Bool(bool),
    Char(u32),
    /// `[a, b, ..]`
    Array(Vec<CVal>),
    /// A struct (`variant: None`, the fields in declaration order) or an
    /// enum variant (`variant: Some(name)`, the payload as fields `0`, `1`,
    /// ..). `ty` is the type's full path (`unit::T`), or `Option`/`Result`.
    Data {
        ty: String,
        variant: Option<String>,
        fields: Vec<(String, CVal)>,
    },
}

impl CVal {
    /// The type of a scalar (`Error` for an array or data value).
    pub fn ty(&self) -> TypeKind {
        match self {
            CVal::Int(_, t) => TypeKind::Int(*t),
            CVal::Bool(_) => TypeKind::Bool,
            CVal::Char(_) => TypeKind::Char,
            CVal::Array(_) | CVal::Data { .. } => TypeKind::Error,
        }
    }

    fn scalar(&self) -> bool {
        matches!(self, CVal::Int(..) | CVal::Bool(_) | CVal::Char(_))
    }

    fn show(&self) -> String {
        match self {
            CVal::Int(v, t) => format!("{v}_{}", t.name()),
            CVal::Bool(b) => b.to_string(),
            CVal::Char(c) => format!("{:?}", char::from_u32(*c).unwrap_or('?')),
            CVal::Array(vs) => format!("an array of {}", vs.len()),
            CVal::Data {
                ty,
                variant: Some(v),
                ..
            } => format!("`{ty}::{v}`"),
            CVal::Data { ty, .. } => format!("a `{ty}`"),
        }
    }
}

/// A struct or enum declaration as written, for typing the literals of an
/// initializer before the checker runs: its unit (where its field types
/// are written), type parameters and fields or variants.
#[derive(Clone, Debug)]
struct TyDef {
    unit: String,
    generics: Vec<String>,
    kind: TyDefKind,
}

#[derive(Clone, Debug)]
enum TyDefKind {
    Struct(Vec<(String, TypeKind)>),
    Enum(Vec<(String, Vec<TypeKind>)>),
}

/// The largest array a `const` builds (elements): evaluation stays small.
const MAX_ELEMS: u128 = 1 << 16;

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
    let mut tydefs: BTreeMap<String, TyDef> = BTreeMap::new();
    fn collect(
        m: &Mod,
        prefix: &str,
        defs: &mut BTreeMap<String, (String, ConstDef)>,
        lens: &mut Vec<(Span, Vec<Ident>, String)>,
        tydefs: &mut BTreeMap<String, TyDef>,
        diags: &mut Diags,
    ) {
        let unit = format!("{prefix}{}", m.name.name);
        for t in &m.types {
            let kind = match &t.kind {
                TypeDefKind::Struct(fs) => TyDefKind::Struct(
                    fs.iter()
                        .map(|f| (f.name.name.clone(), f.ty.kind.clone()))
                        .collect(),
                ),
                TypeDefKind::Enum(vs) => TyDefKind::Enum(
                    vs.iter()
                        .map(|v| {
                            (
                                v.name.name.clone(),
                                v.payload.iter().map(|t| t.kind.clone()).collect(),
                            )
                        })
                        .collect(),
                ),
            };
            tydefs.insert(
                format!("{unit}::{}", t.name.name),
                TyDef {
                    unit: unit.clone(),
                    generics: t.generics.iter().map(|g| g.name.name.clone()).collect(),
                    kind,
                },
            );
        }
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
            collect(s, &format!("{unit}::"), defs, lens, tydefs, diags);
        }
    }
    for m in &p.mods {
        collect(m, "", &mut defs, &mut lens, &mut tydefs, diags);
    }
    // A repeat count written as a name, by the count's span.
    let named: BTreeMap<(u32, u32, u32), Vec<String>> = lens
        .iter()
        .map(|(sp, path, _)| {
            (
                (sp.file, sp.lo, sp.hi),
                path.iter().map(|i| i.name.clone()).collect(),
            )
        })
        .collect();
    let mut ev = Eval {
        defs: &defs,
        tydefs: &tydefs,
        named: &named,
        done: BTreeMap::new(),
        stack: Vec::new(),
        env: Vec::new(),
        diags,
    };
    let paths: Vec<String> = defs.keys().cloned().collect();
    for path in &paths {
        ev.konst(path);
    }
    let Eval { done, diags, .. } = ev;
    let mut consts: BTreeMap<String, Const> = defs
        .iter()
        .map(|(path, (unit, c))| {
            (
                path.clone(),
                Const {
                    unit: unit.clone(),
                    ty: c.ty.kind.clone(),
                    value: done.get(path).cloned().flatten(),
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
        match &consts[&full].value {
            Some(CVal::Int(v, IntTy::Usize)) if *v <= u32::MAX as i128 => Some(*v as u64),
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
    crate::ast::unit_types_mut(p, &mut |unit, t| {
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
                t.kind = match length(path, unit, t.span, diags) {
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
    let mut set = |e: &mut Expr| {
        if let ExprKind::Repeat {
            count, count_span, ..
        } = &mut e.kind
            && let Some(n) = counts.get(&(count_span.file, count_span.lo, count_span.hi))
        {
            *count = *n;
        }
    };
    crate::ast::exprs_mut(p, &mut set);
    crate::ast::const_exprs_mut(p, &mut set);
    // The declared types as rewritten (a `[T; N]` holds its length).
    fn declared(m: &Mod, prefix: &str, consts: &mut BTreeMap<String, Const>) {
        let unit = format!("{prefix}{}", m.name.name);
        for c in &m.consts {
            if let Some(k) = consts.get_mut(&format!("{unit}::{}", c.name.name)) {
                k.ty = c.ty.kind.clone();
            }
        }
        for s in &m.mods {
            declared(s, &format!("{unit}::"), consts);
        }
    }
    for m in &p.mods {
        declared(m, "", &mut consts);
    }
    consts
}

/// A scalar `v` as an expression: a literal (a negative integer is `-` of
/// one, at the same span, so both have the `const`'s type).
pub fn literal(v: &CVal, span: Span) -> ExprKind {
    match *v {
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
        CVal::Array(_) | CVal::Data { .. } => unreachable!("a scalar"),
    }
}

/// `v` of the checked type `ty` as an expression at `at`: literals,
/// arrays, struct literals and variants, each node at a span of its own
/// (file `source::CONST_VALUES`, from `next` on) with its type in `types`,
/// as the checker would have typed it.
pub fn literal_tree(
    v: &CVal,
    ty: &TypeKind,
    at: Span,
    adts: &BTreeMap<String, crate::check::Adt>,
    types: &mut BTreeMap<(u32, u32, u32), TypeKind>,
    next: &mut u32,
) -> Expr {
    let fresh = |next: &mut u32| {
        *next += 1;
        Span {
            file: crate::source::CONST_VALUES,
            lo: *next,
            hi: *next,
        }
    };
    let id = |name: &str, span: Span| Ident {
        name: name.to_string(),
        span,
    };
    let args_of = |ty: &TypeKind| -> Vec<TypeKind> {
        match ty {
            TypeKind::Named { args, .. } => args.iter().map(|a| a.kind.clone()).collect(),
            _ => Vec::new(),
        }
    };
    let kind = match v {
        CVal::Int(..) | CVal::Bool(_) | CVal::Char(_) => literal(v, at),
        CVal::Array(vs) => {
            let TypeKind::Array(el, _) = ty else {
                unreachable!("an array value of an array type")
            };
            ExprKind::Array(
                vs.iter()
                    .map(|x| {
                        let sp = fresh(next);
                        literal_tree(x, &el.kind, sp, adts, types, next)
                    })
                    .collect(),
            )
        }
        CVal::Data {
            ty: path,
            variant,
            fields,
        } => {
            let targs = args_of(ty);
            // The field or payload types of the declaration, in `ty`'s
            // arguments.
            let decl: Vec<TypeKind> = match (path.as_str(), variant.as_deref()) {
                ("Option", Some("Some")) | ("Result", Some("Ok")) => {
                    targs.first().cloned().into_iter().collect()
                }
                ("Result", Some("Err")) => targs.get(1).cloned().into_iter().collect(),
                ("Option", _) => Vec::new(),
                _ => {
                    let a = &adts[path.as_str()];
                    let sub: BTreeMap<String, TypeKind> = a
                        .generics
                        .iter()
                        .map(|g| g.name.clone())
                        .zip(targs.iter().cloned())
                        .collect();
                    let raw: Vec<TypeKind> = match (&a.kind, variant) {
                        (crate::check::AdtKind::Struct(fs), None) => {
                            fs.iter().map(|(_, t)| t.clone()).collect()
                        }
                        (crate::check::AdtKind::Enum(vs), Some(var)) => vs
                            .iter()
                            .find(|(n, _)| n == var)
                            .map(|(_, ps)| ps.clone())
                            .unwrap_or_default(),
                        _ => unreachable!("a checked value"),
                    };
                    raw.iter().map(|t| crate::check::subst(t, &sub)).collect()
                }
            };
            let parts: Vec<(String, Expr)> = fields
                .iter()
                .zip(&decl)
                .map(|((n, x), t)| {
                    let sp = fresh(next);
                    (n.clone(), literal_tree(x, t, sp, adts, types, next))
                })
                .collect();
            let segs = |extra: Option<&str>, next: &mut u32| -> Vec<Ident> {
                let mut p: Vec<Ident> = path.split("::").map(|s| id(s, fresh(next))).collect();
                if let Some(x) = extra {
                    p.push(id(x, fresh(next)));
                }
                p
            };
            match variant.as_deref() {
                None => ExprKind::StructLit {
                    path: segs(None, next),
                    fields: parts
                        .into_iter()
                        .map(|(n, x)| (id(&n, fresh(next)), x))
                        .collect(),
                    rest: None,
                },
                Some("None") if path == "Option" => ExprKind::Var(id("None", fresh(next))),
                Some(var) if path == "Option" || path == "Result" => ExprKind::Call {
                    path: vec![id(var, fresh(next))],
                    targs: Vec::new(),
                    args: parts.into_iter().map(|(_, x)| x).collect(),
                    method: None,
                },
                Some(var) if parts.is_empty() => ExprKind::Path(segs(Some(var), next)),
                Some(var) => ExprKind::Call {
                    path: segs(Some(var), next),
                    targs: Vec::new(),
                    args: parts.into_iter().map(|(_, x)| x).collect(),
                    method: None,
                },
            }
        }
    };
    types.insert((at.file, at.lo, at.hi), ty.clone());
    Expr { kind, span: at }
}

/// A value as source text (an array of more than 32 elements shows its
/// first 32 and its length; the map's hash is of the whole value).
pub fn value_text(v: &CVal) -> String {
    fn go(v: &CVal, out: &mut String) {
        match v {
            CVal::Int(n, _) => out.push_str(&n.to_string()),
            CVal::Bool(b) => out.push_str(&b.to_string()),
            CVal::Char(c) => out.push_str(&crate::lexer::str_text(
                &char::from_u32(*c).unwrap_or('\0').to_string(),
                '\'',
            )),
            CVal::Array(vs) => {
                out.push('[');
                for (k, x) in vs.iter().take(32).enumerate() {
                    if k > 0 {
                        out.push_str(", ");
                    }
                    go(x, out);
                }
                if vs.len() > 32 {
                    out.push_str(&format!(", .. ({} elements)", vs.len()));
                }
                out.push(']');
            }
            CVal::Data {
                ty,
                variant,
                fields,
            } => {
                let name = ty.rsplit("::").next().unwrap_or(ty);
                match variant {
                    None => {
                        out.push_str(name);
                        out.push_str(" { ");
                        for (k, (n, x)) in fields.iter().enumerate() {
                            if k > 0 {
                                out.push_str(", ");
                            }
                            out.push_str(n);
                            out.push_str(": ");
                            go(x, out);
                        }
                        out.push_str(" }");
                    }
                    Some(var) => {
                        if ty != "Option" && ty != "Result" {
                            out.push_str(name);
                            out.push_str("::");
                        }
                        out.push_str(var);
                        if !fields.is_empty() {
                            out.push('(');
                            for (k, (_, x)) in fields.iter().enumerate() {
                                if k > 0 {
                                    out.push_str(", ");
                                }
                                go(x, out);
                            }
                            out.push(')');
                        }
                    }
                }
            }
        }
    }
    let mut out = String::new();
    go(v, &mut out);
    out
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
    tydefs: &'a BTreeMap<String, TyDef>,
    /// Repeat counts written as a name, by the count's span.
    named: &'a BTreeMap<(u32, u32, u32), Vec<String>>,
    /// Evaluated (`None`: failed, reported).
    done: BTreeMap<String, Option<CVal>>,
    /// The `const`s being evaluated, outermost first.
    stack: Vec<String>,
    /// The names a `match` arm binds, innermost last.
    env: Vec<BTreeMap<String, CVal>>,
    diags: &'a mut Diags,
}

/// Whether a `const` of this declared type is a scalar.
pub fn scalar(t: &TypeKind) -> bool {
    matches!(t, TypeKind::Int(_) | TypeKind::Bool | TypeKind::Char)
}

/// Whether a `const` may be declared with this type before the checker
/// sees it: a scalar, an array or a named type (the checker tells which
/// named types hold only values, `Checker::check_const`).
fn declarable(t: &TypeKind) -> bool {
    scalar(t)
        || matches!(
            t,
            TypeKind::Array(..) | TypeKind::ArrayN(..) | TypeKind::Named { .. }
        )
}

impl Eval<'_> {
    fn konst(&mut self, path: &str) -> Option<CVal> {
        if let Some(v) = self.done.get(path) {
            return v.clone();
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
        if !declarable(&c.ty.kind) {
            self.diags.push(not_value(&c.ty.kind, c.ty.span));
            self.done.insert(path.to_string(), None);
            return None;
        }
        self.stack.push(path.to_string());
        let want = self.norm(&c.ty.kind, &unit, &BTreeMap::new());
        let saved = std::mem::take(&mut self.env);
        let v = self.expr(&c.value, Some(&want), &unit);
        self.env = saved;
        self.stack.pop();
        let v = v.and_then(|v| {
            // An array or data value's shape is the checker's to judge.
            if !scalar(&c.ty.kind) || v.ty() == c.ty.kind {
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
        self.done.insert(path.to_string(), v.clone());
        v
    }

    /// `t` with its named types as full paths (`unit::T`; `Option` and
    /// `Result` as they are) and type parameters replaced by `sub`.
    fn norm(&self, t: &TypeKind, unit: &str, sub: &BTreeMap<String, TypeKind>) -> TypeKind {
        let boxed = |x: &Type| {
            Box::new(Type {
                kind: self.norm(&x.kind, unit, sub),
                span: x.span,
            })
        };
        match t {
            TypeKind::Named { path, args } if path.len() == 1 && sub.contains_key(&path[0]) => {
                sub[&path[0]].clone()
            }
            TypeKind::Named { path, args } => {
                let path = match self.type_path(path, unit) {
                    Some(full) => full.split("::").map(str::to_string).collect(),
                    None => path.clone(),
                };
                TypeKind::Named {
                    path,
                    args: args
                        .iter()
                        .map(|a| Type {
                            kind: self.norm(&a.kind, unit, sub),
                            span: a.span,
                        })
                        .collect(),
                }
            }
            TypeKind::Array(e, n) => TypeKind::Array(boxed(e), *n),
            TypeKind::ArrayN(e, n) => TypeKind::ArrayN(boxed(e), n.clone()),
            _ => t.clone(),
        }
    }

    /// The full path of the struct or enum `path` names from `unit`.
    fn type_path(&self, path: &[String], unit: &str) -> Option<String> {
        let local = format!("{unit}::{}", path.join("::"));
        if self.tydefs.contains_key(&local) {
            return Some(local);
        }
        let full = path.join("::");
        self.tydefs.contains_key(&full).then_some(full)
    }

    /// The declaration `want` names and its parameters' arguments.
    fn decl_of(
        &self,
        path: &str,
        want: Option<&TypeKind>,
    ) -> Option<(TyDef, BTreeMap<String, TypeKind>)> {
        let d = self.tydefs.get(path)?.clone();
        let mut sub = BTreeMap::new();
        if let Some(TypeKind::Named { path: wp, args }) = want
            && wp.join("::") == path
        {
            for (g, a) in d.generics.iter().zip(args) {
                sub.insert(g.clone(), a.kind.clone());
            }
        }
        Some((d, sub))
    }

    /// The element type an array expected as `want` holds.
    fn elem(want: Option<&TypeKind>) -> Option<TypeKind> {
        match want {
            Some(TypeKind::Array(e, _) | TypeKind::ArrayN(e, _)) => Some(e.kind.clone()),
            _ => None,
        }
    }

    /// The argument `k` of an expected `Option`/`Result`.
    fn arg(want: Option<&TypeKind>, ty: &str, k: usize) -> Option<TypeKind> {
        match want {
            Some(TypeKind::Named { path, args }) if path.len() == 1 && path[0] == ty => {
                args.get(k).map(|a| a.kind.clone())
            }
            _ => None,
        }
    }

    /// A variant of `Option`/`Result` or of a declared enum, built from
    /// `args` (`None` if `path` names no variant).
    fn variant(
        &mut self,
        path: &[Ident],
        args: &[Expr],
        want: Option<&TypeKind>,
        unit: &str,
    ) -> Option<Option<CVal>> {
        let names: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
        let builtin = match names.as_slice() {
            [v] if v == "Some" => Some(("Option", vec![Self::arg(want, "Option", 0)])),
            [v] if v == "None" => Some(("Option", vec![])),
            [v] if v == "Ok" => Some(("Result", vec![Self::arg(want, "Result", 0)])),
            [v] if v == "Err" => Some(("Result", vec![Self::arg(want, "Result", 1)])),
            _ => None,
        };
        let (ty, payload) = if let Some((ty, payload)) = builtin {
            (ty.to_string(), payload)
        } else {
            let (v, tpath) = names.split_last()?;
            let full = self.type_path(tpath, unit)?;
            let (d, sub) = self.decl_of(&full, want)?;
            let TyDefKind::Enum(vs) = &d.kind else {
                return None;
            };
            let (_, ps) = vs.iter().find(|(n, _)| n == v)?;
            let ps = ps
                .iter()
                .map(|t| Some(self.norm(t, &d.unit, &sub)))
                .collect();
            (full, ps)
        };
        if payload.len() != args.len() {
            return Some(None);
        }
        let mut fields = Vec::new();
        for (k, (a, w)) in args.iter().zip(&payload).enumerate() {
            let Some(x) = self.expr(a, w.as_ref(), unit) else {
                return Some(None);
            };
            fields.push((k.to_string(), x));
        }
        Some(Some(CVal::Data {
            ty,
            variant: Some(names.last().expect("a path").clone()),
            fields,
        }))
    }

    /// The `const` or `match` binding a one-segment name reads.
    fn name(&mut self, e: &Expr, path: &[String], unit: &str) -> Option<CVal> {
        if let [n] = path
            && let Some(v) = self.env.iter().rev().find_map(|m| m.get(n))
        {
            return Some(v.clone());
        }
        match lookup_defs(self.defs, path, unit) {
            Some(p) => self.konst(&p),
            None => self.not_const(e, "only other `const`s are named"),
        }
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
                "a `const` is built from literals, other `const`s, operators, `if`, `match`, the wrapping and saturating methods, `as`, arrays, structs, variants, fields and elements (ADR-0052 §3)",
                None,
            ),
        );
        None
    }

    /// Whether a one-segment name is a `match` binding or a `const`.
    fn bound(&self, n: &str, unit: &str) -> bool {
        self.env.iter().any(|m| m.contains_key(n))
            || lookup_defs(self.defs, &[n.to_string()], unit).is_some()
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
            ExprKind::Var(id) if id.name == "None" && !self.bound("None", unit) => self
                .variant(std::slice::from_ref(id), &[], want, unit)
                .flatten(),
            ExprKind::Var(id) => self.name(e, std::slice::from_ref(&id.name), unit),
            ExprKind::Path(path) => {
                if let Some((TypeKind::Int(t), v)) = crate::check::int_const(path) {
                    return Some(CVal::Int(v, t));
                }
                let names: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
                if lookup_defs(self.defs, &names, unit).is_none()
                    && let Some(v) = self.variant(path, &[], want, unit)
                {
                    return v;
                }
                self.name(e, &names, unit)
            }
            ExprKind::Call { path, args, .. } => match self.variant(path, args, want, unit) {
                Some(v) => v,
                None => self.not_const(e, "a call (a `const` names no function, ADR-0052 §3)"),
            },
            ExprKind::Array(items) => {
                let mut w = Self::elem(want);
                let mut vals = Vec::new();
                for it in items {
                    let v = self.expr(it, w.as_ref(), unit)?;
                    if w.is_none() && v.scalar() {
                        w = Some(v.ty());
                    }
                    vals.push(v);
                }
                Some(CVal::Array(vals))
            }
            ExprKind::Repeat {
                value,
                count,
                count_span,
            } => {
                let n = match self
                    .named
                    .get(&(count_span.file, count_span.lo, count_span.hi))
                {
                    Some(path) => {
                        let p = lookup_defs(self.defs, path, unit)?;
                        match self.konst(&p)? {
                            CVal::Int(n, IntTy::Usize) => n as u128,
                            v => {
                                return self
                                    .fail(*count_span, format!("the count is {}", v.show()));
                            }
                        }
                    }
                    None => *count,
                };
                if n > MAX_ELEMS {
                    return self.fail(
                        e.span,
                        format!("an array of {n} elements (a `const` builds at most {MAX_ELEMS})"),
                    );
                }
                let v = self.expr(value, Self::elem(want).as_ref(), unit)?;
                Some(CVal::Array(vec![v; n as usize]))
            }
            ExprKind::StructLit { path, fields, rest } => {
                if rest.is_some() {
                    return self.not_const(e, "`..base`");
                }
                let names: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
                let full = self.type_path(&names, unit)?;
                let (d, sub) = self.decl_of(&full, want)?;
                let TyDefKind::Struct(fs) = &d.kind else {
                    return None;
                };
                let mut out = Vec::new();
                for (n, ft) in fs {
                    let (_, x) = fields.iter().find(|(i, _)| i.name == *n)?;
                    let w = self.norm(ft, &d.unit, &sub);
                    out.push((n.clone(), self.expr(x, Some(&w), unit)?));
                }
                Some(CVal::Data {
                    ty: full,
                    variant: None,
                    fields: out,
                })
            }
            ExprKind::Field { base, name } => match self.expr(base, None, unit)? {
                CVal::Data {
                    variant: None,
                    fields,
                    ..
                } => fields
                    .into_iter()
                    .find(|(n, _)| *n == name.name)
                    .map(|(_, v)| v),
                v => self.fail(e.span, format!("`.{}` of {}", name.name, v.show())),
            },
            ExprKind::Index { base, index } => {
                let b = self.expr(base, None, unit)?;
                let i = self.expr(index, Some(&TypeKind::Int(IntTy::Usize)), unit)?;
                match (b, i) {
                    (CVal::Array(vs), CVal::Int(k, _)) => {
                        match usize::try_from(k).ok().and_then(|k| vs.get(k)) {
                            Some(v) => Some(v.clone()),
                            None => self.fail(
                                e.span,
                                format!("index {k} is out of bounds of an array of {}", vs.len()),
                            ),
                        }
                    }
                    (b, i) => self.fail(e.span, format!("{} indexed by {}", b.show(), i.show())),
                }
            }
            ExprKind::Match { scrut, arms } => {
                let v = self.expr(scrut, None, unit)?;
                let CVal::Data {
                    variant: Some(var),
                    fields,
                    ..
                } = &v
                else {
                    return self.fail(scrut.span, format!("a `match` of {}", v.show()));
                };
                for a in arms {
                    let last = &a.pat.path.last().expect("a pattern path").name;
                    if last != "_" && last != var {
                        continue;
                    }
                    let mut binds = BTreeMap::new();
                    for (b, (_, x)) in a.pat.binds.iter().zip(fields) {
                        if let Some(b) = b {
                            binds.insert(b.name.clone(), x.clone());
                        }
                    }
                    self.env.push(binds);
                    let r = self.expr(&a.body, want, unit);
                    self.env.pop();
                    return r;
                }
                None
            }
            // `-128` is one literal of `i8`, as Rust reads it.
            ExprKind::Unary(UnOp::Neg, x) if let ExprKind::Int(v, _) = x.kind => {
                let t = match want {
                    Some(TypeKind::Int(t)) => *t,
                    _ => IntTy::I64,
                };
                if !t.signed() {
                    return self.fail(e.span, format!("`-{v}` of `{}`", t.name()));
                }
                match i128::try_from(v) {
                    Ok(v) => self.int(-v, t, e.span, &format!("`-{v}`")),
                    Err(_) => self.fail(e.span, format!("`-{v}` overflows `{}`", t.name())),
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
            let o = match (&x, &y) {
                (CVal::Int(p, _), CVal::Int(q, _)) => p.cmp(q),
                (CVal::Bool(p), CVal::Bool(q)) => p.cmp(q),
                (CVal::Char(p), CVal::Char(q)) => p.cmp(q),
                (x, _) => return self.fail(e.span, format!("`{}` of {}", op.symbol(), x.show())),
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
        match (x.clone(), y.clone()) {
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
                    // `MIN % -1` overflows as `MIN / -1` does (Rust).
                    Rem if t.signed() && p == t.min_value() && q == -1 => {
                        self.fail(e.span, format!("{shown} overflows `{}`", t.name()))
                    }
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

/// E0437: a `const` of a type that holds more than values.
pub fn not_value(t: &TypeKind, span: Span) -> Diag {
    Diag::new(
        "E0437",
        span,
        format!(
            "`{}` cannot be a `const`: a `const` holds a value type, an integer, `bool` or `char`, or an array, struct or enum of them (ADR-0052 §2)",
            crate::check::ty_str(t)
        ),
    )
    .fix(
        "build the value where it is used (a `const` allocates nothing: no `Vec`, `String`, `Box` or reference)",
        None,
    )
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
