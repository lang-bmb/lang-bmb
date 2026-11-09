//! Function values (ADR-0053, backlog 2V-20): lowered before checking to
//! the traits, structs and `impl`s Rust itself has underneath, so the
//! checker, prover and code generator see ADR-0046 generics only.
//!
//! - A bound `F: Fn(A, B) -> R` of function `g` is a trait of `g`'s unit,
//!   `__fn_g_F`, with the spec methods `pre(self, a, b)` and
//!   `post(self, a, b, r)` and the method `call(self, a, b) -> (r: R)
//!   requires self.pre(a, b) ensures self.post(a, b, r)`, whose effects are
//!   `g`'s (ADR-0053 §5: calling `f` is an effect of `g`). In `g`, `f(a, b)`
//!   is `f.call(a, b)`, `f.requires(a, b)` is `f.pre(a, b)` and
//!   `f.ensures(a, b, r)` is `f.post(a, b, r)`.
//! - A function item `h` passed where `g` takes an `F` (`g::<_>(h, ..)`,
//!   the `_` written for the nameless type, ADR-0053 §2) is a struct with no
//!   fields in `h`'s unit, `__fnitem_h_g_F`, whose `impl` of the trait is
//!   `h`: `pre` is `h`'s `requires`, `post` its `ensures`, `call` calls `h`
//!   (and has `h`'s effects, which the trait's must hold: E0608 through
//!   the `impl`'s conformance). The argument is that struct's value and the
//!   `_` its type.
//!
//! The generated items are text parsed under file ids of their own
//! (`source::GENERATED` down), so every node has its own span. The source
//! program the formatter prints is not lowered: `check::check_resolved`
//! lowers its copy.

use std::collections::BTreeMap;

use crate::ast::*;
use crate::diag::{Diag, Diags};

/// An `Fn`-bounded type parameter: its position among the type
/// parameters, its name and bound, and the value parameters of type `F` or
/// `&F` (position, by reference).
type Bounded = (usize, String, FnBound, Vec<(usize, bool)>);

/// An argument to rewrite: the call's span (file, lo, hi), the argument's
/// position, the struct that is its value, the type argument's position,
/// the generic function, and whether the parameter takes it by reference.
type Rewrite = (u32, u32, u32, usize, String, usize, String, bool);

/// A function with `Fn`-bounded type parameters.
struct Generic {
    unit: String,
    /// For each such parameter: its position among the type parameters,
    /// its name and bound, and the value parameters of type `F` or `&F`
    /// (position, by reference).
    bounded: Vec<Bounded>,
    effects: String,
}

/// A function's text needed to make it a value.
struct Item {
    unit: String,
    params: Vec<(String, String)>,
    ret_name: String,
    ret: String,
    requires: Vec<String>,
    ensures: Vec<String>,
    effects: String,
}

fn effects_of(f: &FnDef) -> String {
    f.attrs
        .iter()
        .find(|a| a.name.name == "effects")
        .map(|a| {
            a.args
                .iter()
                .map(|x| match x {
                    AttrArg::Ident(i) => i.name.clone(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "pure".into())
}

fn trait_name(g: &str, f: &str) -> String {
    format!("__fn_{g}_{f}")
}

/// The value parameters of `f` whose type is type parameter `t` or a
/// reference to it.
fn takes(f: &FnDef, t: &str) -> Vec<(usize, bool)> {
    let is_t = |k: &TypeKind| matches!(k, TypeKind::Named { path, args } if args.is_empty() && path.len() == 1 && path[0] == t);
    f.params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match &p.ty.kind {
            k if is_t(k) => Some((i, false)),
            TypeKind::Ref { inner, .. } if is_t(&inner.kind) => Some((i, true)),
            _ => None,
        })
        .collect()
}

/// Returns the generated files whose diagnostics repeat one already
/// reported (the caller drops them after checking).
pub fn lower(p: &mut Program, diags: &mut Diags) -> Vec<u32> {
    // The generic functions with `Fn` bounds, and every free function.
    let mut generics: BTreeMap<String, Generic> = BTreeMap::new();
    let mut items: BTreeMap<String, Item> = BTreeMap::new();
    fn scan(
        m: &Mod,
        prefix: &str,
        generics: &mut BTreeMap<String, Generic>,
        items: &mut BTreeMap<String, Item>,
    ) {
        let unit = format!("{prefix}{}", m.name.name);
        for f in &m.fns {
            let path = format!("{unit}::{}", f.name.name);
            let bounded: Vec<_> = f
                .generics
                .iter()
                .enumerate()
                .filter_map(|(i, g)| {
                    let b = g.fn_bound.clone()?;
                    Some((i, g.name.name.clone(), b, takes(f, &g.name.name)))
                })
                .collect();
            if !bounded.is_empty() {
                generics.insert(
                    path.clone(),
                    Generic {
                        unit: unit.clone(),
                        bounded,
                        effects: effects_of(f),
                    },
                );
            }
            if f.kind == FnKind::Exec && f.generics.is_empty() {
                let ret_name = f
                    .ret
                    .name
                    .as_ref()
                    .map_or("result".to_string(), |n| n.name.clone());
                items.insert(
                    path,
                    Item {
                        unit: unit.clone(),
                        params: f
                            .params
                            .iter()
                            .map(|p| (p.name.name.clone(), crate::fmt::type_str(&p.ty)))
                            .collect(),
                        ret_name,
                        ret: crate::fmt::type_str(&f.ret.ty),
                        requires: f.requires.iter().map(crate::fmt::expr_text).collect(),
                        ensures: f.ensures.iter().map(crate::fmt::expr_text).collect(),
                        effects: effects_of(f),
                    },
                );
            }
        }
        for s in &m.mods {
            scan(s, &format!("{unit}::"), generics, items);
        }
    }
    for m in &p.mods {
        scan(m, "", &mut generics, &mut items);
    }
    if generics.is_empty() {
        return Vec::new();
    }
    let mut pending: Vec<Diag> = Vec::new();

    // Generated text, by unit.
    // Generated text by unit; a second file per unit holds the items made
    // for an argument already reported (E0608), whose own errors would
    // repeat it.
    let mut text: BTreeMap<(String, bool), String> = BTreeMap::new();
    for (path, g) in &generics {
        let gname = path.rsplit("::").next().unwrap_or_default();
        for (_, f, b, _) in &g.bounded {
            let args: Vec<String> = (0..b.params.len()).map(|i| format!("a{i}")).collect();
            let typed: Vec<String> = b
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("a{i}: {}", crate::fmt::type_str(t)))
                .collect();
            let ret = crate::fmt::type_str(&b.ret);
            let comma = |xs: &[String]| xs.iter().map(|x| format!(", {x}")).collect::<String>();
            let t = text.entry((g.unit.clone(), false)).or_default();
            t.push_str(&format!(
                "trait {tn} {{\n    spec fn pre(self: &Self{p}) -> bool;\n\n    spec fn post(self: &Self{p}, r: {ret}) -> bool;\n\n    #[effects({e})]\n    fn call(self: &Self{p}) -> (r: {ret})\n        requires self.pre({a}),\n        ensures self.post({ar}r);\n}}\n\n",
                tn = trait_name(gname, f),
                p = comma(&typed),
                e = g.effects,
                a = args.join(", "),
                ar = args.iter().map(|x| format!("{x}, ")).collect::<String>(),
            ));
        }
    }

    // Each function item passed as a function value: its struct and impl,
    // once per (item, trait).
    let mut made: BTreeMap<(String, String), String> = BTreeMap::new();
    let resolve = |path: &[Ident], unit: &str| -> Option<String> {
        let full = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        if path.len() == 1 {
            let local = format!("{unit}::{full}");
            if generics.contains_key(&local) || items.contains_key(&local) {
                return Some(local);
            }
        }
        (generics.contains_key(&full) || items.contains_key(&full)).then_some(full)
    };
    // (span of the call) -> rewrites of its arguments and type arguments.
    let mut rewrites: Vec<Rewrite> = Vec::new();
    {
        fn units<'m>(m: &'m Mod, prefix: &str, out: &mut Vec<(String, &'m Mod)>) {
            let unit = format!("{prefix}{}", m.name.name);
            out.push((unit.clone(), m));
            for s in &m.mods {
                units(s, &format!("{unit}::"), out);
            }
        }
        let mut all = Vec::new();
        for m in &p.mods {
            units(m, "", &mut all);
        }
        for (unit, m) in &all {
            for f in m
                .fns
                .iter()
                .chain(m.impls.iter().flat_map(|i| &i.fns))
                .chain(m.trait_impls.iter().flat_map(|i| &i.fns))
            {
                let mut f = f.clone();
                crate::ast::fn_exprs_mut(&mut f, &mut |e| {
                    let ExprKind::Call { path, args, .. } = &e.kind else {
                        return;
                    };
                    let Some(gp) = resolve(path, unit) else {
                        return;
                    };
                    let Some(g) = generics.get(&gp) else {
                        return;
                    };
                    let gname = gp.rsplit("::").next().unwrap_or_default();
                    for (k, fname, _, at) in &g.bounded {
                        for (j, by_ref) in at {
                            let Some(a) = args.get(*j) else { continue };
                            let target = match &a.kind {
                                ExprKind::Var(id) => resolve(std::slice::from_ref(id), unit),
                                ExprKind::Path(ps) => resolve(ps, unit),
                                _ => None,
                            };
                            let Some(h) = target.filter(|h| items.contains_key(h)) else {
                                continue;
                            };
                            let tn = trait_name(gname, fname);
                            let key = (h.clone(), format!("{}::{tn}", g.unit));
                            // Calling `f` is an effect of `g` (ADR-0053 §5).
                            let set = |e: &str| -> Vec<String> {
                                e.split(", ")
                                    .filter(|x| !x.is_empty() && *x != "pure")
                                    .map(String::from)
                                    .collect()
                            };
                            let (have, allowed) = (set(&items[&h].effects), set(&g.effects));
                            let extra: Vec<&String> =
                                have.iter().filter(|x| !allowed.contains(x)).collect();
                            let reported = !extra.is_empty();
                            if reported {
                                let extra: Vec<&str> = extra.iter().map(|x| x.as_str()).collect();
                                pending.push(
                                    Diag::new(
                                        "E0608",
                                        a.span,
                                        format!(
                                            "`{}` has effect `{}`, which `{gp}` does not declare for its `{fname}`",
                                            h.rsplit("::").next().unwrap_or_default(),
                                            extra.join(", ")
                                        ),
                                    )
                                    .fix(
                                        format!("declare `{}` on `{gp}`, or pass a function without it", extra.join(", ")),
                                        None,
                                    ),
                                );
                            }
                            let sname = made.entry(key.clone()).or_insert_with(|| {
                                let hn = h.rsplit("::").next().unwrap_or_default();
                                let sname = format!("__fnitem_{hn}_{gname}_{fname}");
                                let it = &items[&h];
                                let typed: String = it
                                    .params
                                    .iter()
                                    .map(|(n, t)| format!(", {n}: {t}"))
                                    .collect();
                                let names: Vec<&str> =
                                    it.params.iter().map(|(n, _)| n.as_str()).collect();
                                let conj = |xs: &[String]| {
                                    if xs.is_empty() {
                                        "true".to_string()
                                    } else {
                                        xs.iter().map(|x| format!("({x})")).collect::<Vec<_>>().join(" && ")
                                    }
                                };
                                let t = text.entry((it.unit.clone(), reported)).or_default();
                                t.push_str(&format!(
                                    "struct {sname} {{ item: bool }}\n\nimpl {tr} for {sname} {{\n    spec fn pre(self: &{sname}{typed}) -> bool {{ {pre} }}\n\n    spec fn post(self: &{sname}{typed}, {rn}: {ret}) -> bool {{ {post} }}\n\n    #[effects({e})]\n    fn call(self: &{sname}{typed}) -> ({rn}: {ret})\n        requires self.pre({a}),\n        ensures self.post({a}{sep}{rn}),\n    {{\n        {hn}({a})\n    }}\n}}\n\n",
                                    tr = key.1,
                                    pre = conj(&it.requires),
                                    post = conj(&it.ensures),
                                    rn = it.ret_name,
                                    ret = it.ret,
                                    e = it.effects,
                                    a = names.join(", "),
                                    sep = if names.is_empty() { "" } else { ", " },
                                ));
                                format!("{}::{sname}", it.unit)
                            });
                            rewrites.push((
                                e.span.file,
                                e.span.lo,
                                e.span.hi,
                                *j,
                                sname.clone(),
                                *k,
                                gp.clone(),
                                *by_ref,
                            ));
                        }
                    }
                });
            }
        }
    }

    // The generic functions themselves: the bound is the trait, the calls
    // are its methods.
    fn rewrite_generic(m: &mut Mod, prefix: &str, generics: &BTreeMap<String, Generic>) {
        let unit = format!("{prefix}{}", m.name.name);
        for f in &mut m.fns {
            let path = format!("{unit}::{}", f.name.name);
            let Some(g) = generics.get(&path) else {
                continue;
            };
            let mut values: Vec<String> = Vec::new();
            for (k, fname, _, at) in &g.bounded {
                let tp = &mut f.generics[*k];
                tp.fn_bound = None;
                tp.bounds.push(vec![Ident {
                    name: trait_name(&f.name.name, fname),
                    span: tp.name.span,
                }]);
                values.extend(at.iter().map(|(i, _)| f.params[*i].name.name.clone()));
            }
            crate::ast::fn_exprs_mut(f, &mut |e| match &mut e.kind {
                ExprKind::Call { path, args, .. }
                    if path.len() == 1 && values.contains(&path[0].name) =>
                {
                    let recv = Expr {
                        kind: ExprKind::Var(path[0].clone()),
                        span: path[0].span,
                    };
                    e.kind = ExprKind::Method {
                        recv: Box::new(recv),
                        name: Ident {
                            name: "call".into(),
                            span: path[0].span,
                        },
                        args: std::mem::take(args),
                    };
                }
                ExprKind::Method { recv, name, .. }
                    if matches!(&recv.kind, ExprKind::Var(v) if values.contains(&v.name))
                        && (name.name == "requires" || name.name == "ensures") =>
                {
                    name.name = if name.name == "requires" {
                        "pre"
                    } else {
                        "post"
                    }
                    .into();
                }
                _ => {}
            });
        }
        for s in &mut m.mods {
            rewrite_generic(s, &format!("{unit}::"), generics);
        }
    }
    for m in &mut p.mods {
        rewrite_generic(m, "", &generics);
    }

    // The calls: the argument is the struct's value, the `_` its type.
    crate::ast::exprs_mut(p, &mut |e| {
        let key = (e.span.file, e.span.lo, e.span.hi);
        let mine: Vec<_> = rewrites
            .iter()
            .filter(|r| (r.0, r.1, r.2) == key)
            .cloned()
            .collect();
        if mine.is_empty() {
            return;
        }
        let ExprKind::Call {
            path, targs, args, ..
        } = &mut e.kind
        else {
            return;
        };
        for (_, _, _, j, sname, k, _, by_ref) in mine {
            let sp = args[j].span;
            let segs: Vec<Ident> = sname
                .split("::")
                .map(|s| Ident {
                    name: s.to_string(),
                    span: sp,
                })
                .collect();
            let lit = Expr {
                // A struct has a field (E0420): one that carries nothing.
                kind: ExprKind::StructLit {
                    path: segs,
                    fields: vec![(
                        Ident {
                            name: "item".into(),
                            span: sp,
                        },
                        Expr {
                            kind: ExprKind::Bool(false),
                            span: sp,
                        },
                    )],
                    rest: None,
                },
                span: sp,
            };
            args[j] = if by_ref {
                Expr {
                    kind: ExprKind::Ref {
                        mutable: false,
                        expr: Box::new(lit),
                    },
                    span: sp,
                }
            } else {
                lit
            };
            let last = path.len() - 1;
            if let Some(t) = targs.iter_mut().find(|t| t.seg == last)
                && let Some(a) = t.args.get_mut(k)
            {
                a.kind = TypeKind::Named {
                    path: sname.split("::").map(String::from).collect(),
                    args: Vec::new(),
                };
            }
        }
    });

    for d in pending {
        diags.push(d);
    }
    // The generated items, parsed, into their units.
    let mut quiet = Vec::new();
    for (made_files, ((unit, reported), body)) in text.into_iter().enumerate() {
        let last = unit.rsplit("::").next().unwrap_or_default().to_string();
        let src = format!("#[summary(\"\")]\nmod {last} {{\n{body}}}\n");
        // A file id of its own for each text, so every node has its own span.
        let id = crate::source::GENERATED - made_files as u32;
        if reported {
            quiet.push(id);
        }
        let toks = crate::lexer::lex(id, &src, diags);
        let parsed = crate::parser::Parser::new(toks, diags, &src).parse_program();
        let Some(made_mod) = parsed.mods.into_iter().next() else {
            continue;
        };
        let segs: Vec<&str> = unit.split("::").collect();
        if let Some(m) = find(&mut p.mods, &segs) {
            m.traits.extend(made_mod.traits);
            m.types.extend(made_mod.types);
            m.trait_impls.extend(made_mod.trait_impls);
        }
    }
    quiet
}

fn find<'m>(mods: &'m mut [Mod], segs: &[&str]) -> Option<&'m mut Mod> {
    let (first, rest) = segs.split_first()?;
    let m = mods.iter_mut().find(|m| m.name.name == *first)?;
    if rest.is_empty() {
        Some(m)
    } else {
        find(&mut m.mods, rest)
    }
}
