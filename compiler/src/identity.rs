//! Definition identity: hash references, names as metadata (ADR-0059 §1).
//!
//! A definition's identity (`id`) is the SHA-256 of its nameless normal
//! form (`fmt::Printer::identity`):
//!
//! - another definition (function, `spec fn`, type, trait) is written as its
//!   `id` (`#<hex>`), a trait method or an enum variant as `#<id>#k`;
//! - binders (parameters, the result name, locals, labels, type
//!   parameters, `forall` variables, pattern names) by the position they are
//!   bound at (`_0`, `_1`, ..), fields and variants by declaration position;
//! - attributes, effects, contracts, `decreases` and triggers as written,
//!   `#[uses]` targets by `id`; unit summaries are not part of any
//!   definition; built-ins (`Vec`, `Option`, `std::mem::replace`, integer
//!   types) by name.
//!
//! **Nominal types and traits.** Two declarations with the same structure
//! (`struct Sq { side: i64 }`, `struct Big { n: i64 }`) are different
//! types in Rust: a trait call dispatches on which one it is. So a type's
//! text holds the trait `impl`s for it, and a trait's text holds all its
//! `impl`s (each as `impl #trait for <type> { #trait#k = #fn, .. }`). An
//! `impl`'s function names both in its head. A trait, its `impl`s and the
//! types they are for are therefore one recursive component (below), where
//! every member has its own position.
//!
//! **Recursive components.** Definitions that refer to one another form a
//! strongly connected component of the reference graph (functions, types
//! and traits alike). Inside it a member is written `#self.k`. The order
//! (Unison's rule, ADR-0059 §1): each member's text with the component's
//! members erased (`#self`), hashed; then, until the partition stops
//! splitting, each member's text with the others written by their previous
//! hash (colour refinement, which separates members that are alike only
//! once erased); remaining ties by path. A tie that survives refinement is a
//! symmetry of the component in practice (alpha-equivalent members referred
//! to alike), where the order does not change any text. The component's
//! hash is the hash of the members' hashes in that order; a member's `id` is
//! `<component>.k`, a component of one is its own hash.
//!
//! A trait's methods are ordered by the hash of each method's text (the
//! trait's own methods erased) together with the hashes of the `impl`
//! functions that define it; remaining ties by name. Default bodies are
//! part of the trait's text; each is also a definition of its own
//! (`unit::Trait::m`), written `default #<trait>#k` and its function.
//!
//! The work is lazy: asking for one definition hashes what it reaches.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::fmt::{Printer, Resolve};
use crate::verify::Tables;

/// A hashed item: a definition with a body (functions, `spec fn`s, `impl`
/// functions, trait default bodies), a type declaration or a trait.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Def(String),
    Type(String),
    Trait(String),
}

impl Key {
    pub fn path(&self) -> &str {
        match self {
            Key::Def(p) | Key::Type(p) | Key::Trait(p) => p,
        }
    }
}

/// The hashes of one item. For a type or a trait all of them hash its
/// text; for a `spec fn` `iface` is `body` (its body is its surface).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hashes {
    /// `sha256:<hex>`, or `sha256:<hex>.k` for member `k` of a recursive
    /// component.
    pub id: String,
    /// The definition's own nameless text (members of its component as
    /// `#self.k`).
    pub body: String,
    /// The same without the body.
    pub iface: String,
    /// `iface` without `#[uses]` (what a caller's query reads, ADR-0043).
    pub caller: String,
}

/// Where a definition with a body sits.
enum Ctx<'p> {
    Free,
    Impl(&'p Impl),
    TraitImpl(&'p TraitImpl),
    /// A trait method's default body: the trait's path.
    Default(String),
}

enum Ent<'p> {
    Def {
        unit: String,
        f: &'p FnDef,
        ctx: Ctx<'p>,
    },
    Type {
        unit: String,
        t: &'p TypeDef,
    },
    Trait {
        unit: String,
        t: &'p TraitDef,
    },
}

/// A trait `impl` block: its unit, its syntax, its trait's full path and
/// its functions (method name, definition path).
struct ImplInfo<'p> {
    unit: String,
    i: &'p TraitImpl,
    trait_path: String,
    fns: Vec<(String, String)>,
}

/// What printing a text produces: the text with or without its body, or
/// the surface a caller reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Body,
    Iface,
    Caller,
}

struct Done {
    hashes: Hashes,
    /// A trait's method names in identity order.
    methods: Vec<String>,
}

/// The identities of a checked program's definitions, computed on demand.
pub struct Ids<'p> {
    t: &'p Tables,
    ents: BTreeMap<Key, Ent<'p>>,
    traits: BTreeSet<String>,
    impls: Vec<ImplInfo<'p>>,
    /// Indices into `impls`: by the type they are for, by trait.
    impls_of_type: BTreeMap<String, Vec<usize>>,
    impls_of_trait: BTreeMap<String, Vec<usize>>,
    edges: RefCell<BTreeMap<Key, Vec<Key>>>,
    done: RefCell<BTreeMap<Key, Done>>,
}

/// Records what a text refers to (the edges of the reference graph).
struct Collect(RefCell<BTreeSet<Key>>);

impl Resolve for Collect {
    fn entity(&self, key: &Key) -> String {
        self.0.borrow_mut().insert(key.clone());
        "#?".into()
    }
    fn method(&self, trait_path: &str, _: &str) -> String {
        self.0
            .borrow_mut()
            .insert(Key::Trait(trait_path.to_string()));
        "#?#?".into()
    }
}

/// How a member of the component being hashed is written.
#[derive(Clone, Copy)]
enum Member<'a> {
    /// `#self` (the first ordering pass).
    Erased,
    /// `#self<hash>` (refinement passes).
    Colored(&'a BTreeMap<Key, String>),
    /// `#self.k` (the final texts).
    At(&'a BTreeMap<Key, usize>),
}

/// Writes references while a component is hashed: its members as
/// `member` says, everything else by `id`.
struct Final<'a, 'p> {
    ids: &'a Ids<'p>,
    comp: &'a BTreeSet<Key>,
    member: Member<'a>,
    /// The method order of the component's traits (empty while it is
    /// being computed).
    methods: &'a BTreeMap<String, Vec<String>>,
}

impl Resolve for Final<'_, '_> {
    fn entity(&self, key: &Key) -> String {
        if self.comp.contains(key) {
            return match self.member {
                Member::Erased => "#self".into(),
                Member::Colored(c) => format!("#self<{}>", hex(&c[key])),
                Member::At(at) => format!("#self.{}", at[key]),
            };
        }
        match self.ids.done.borrow().get(key) {
            Some(d) => format!("#{}", hex(&d.hashes.id)),
            // Not an item of the program (cannot happen in a checked one).
            None => format!("#?{}", key.path()),
        }
    }

    fn method(&self, trait_path: &str, name: &str) -> String {
        let base = self.entity(&Key::Trait(trait_path.to_string()));
        let done = self.ids.done.borrow();
        let order = match self.methods.get(trait_path) {
            Some(o) => Some(o),
            None => done
                .get(&Key::Trait(trait_path.to_string()))
                .map(|d| &d.methods),
        };
        match order.and_then(|o| o.iter().position(|m| m == name)) {
            Some(k) => format!("{base}#{k}"),
            None => format!("{base}#?"),
        }
    }
}

fn sha(s: &str) -> String {
    crate::sha256::hex(s.as_bytes())
}

/// A hash without its `sha256:` prefix, as references write it.
fn hex(h: &str) -> &str {
    h.trim_start_matches("sha256:")
}

impl<'p> Ids<'p> {
    pub fn new(p: &'p Program, t: &'p Tables) -> Ids<'p> {
        let mut ents = BTreeMap::new();
        let mut traits = BTreeSet::new();
        let mut blocks: Vec<(String, &'p TraitImpl)> = Vec::new();
        // Where each function with a body sits, by its span.
        let mut ctx: BTreeMap<(u32, u32, u32), Ctx<'p>> = BTreeMap::new();
        fn walk<'p>(
            m: &'p Mod,
            prefix: &str,
            ents: &mut BTreeMap<Key, Ent<'p>>,
            traits: &mut BTreeSet<String>,
            blocks: &mut Vec<(String, &'p TraitImpl)>,
            ctx: &mut BTreeMap<(u32, u32, u32), Ctx<'p>>,
        ) {
            let mp = format!("{prefix}{}", m.name.name);
            for s in &m.mods {
                walk(s, &format!("{mp}::"), ents, traits, blocks, ctx);
            }
            let at = |f: &FnDef| (f.span.file, f.span.lo, f.span.hi);
            for i in &m.impls {
                for f in &i.fns {
                    ctx.insert(at(f), Ctx::Impl(i));
                }
            }
            for i in &m.trait_impls {
                blocks.push((mp.clone(), i));
                for f in &i.fns {
                    ctx.insert(at(f), Ctx::TraitImpl(i));
                }
            }
            for td in &m.types {
                ents.insert(
                    Key::Type(format!("{mp}::{}", td.name.name)),
                    Ent::Type {
                        unit: mp.clone(),
                        t: td,
                    },
                );
            }
            for tr in &m.traits {
                let path = format!("{mp}::{}", tr.name.name);
                for tf in tr.fns.iter().filter(|tf| tf.has_body) {
                    ctx.insert(at(&tf.f), Ctx::Default(path.clone()));
                }
                traits.insert(path.clone());
                ents.insert(
                    Key::Trait(path),
                    Ent::Trait {
                        unit: mp.clone(),
                        t: tr,
                    },
                );
            }
        }
        for m in &p.mods {
            walk(m, "", &mut ents, &mut traits, &mut blocks, &mut ctx);
        }
        // The definition path of each function, by span.
        let mut path_at: BTreeMap<(u32, u32, u32), String> = BTreeMap::new();
        for (path, unit, f) in crate::check::defs_of(p, &t.fns) {
            let at = (f.span.file, f.span.lo, f.span.hi);
            path_at.insert(at, path.clone());
            let c = ctx.remove(&at).unwrap_or(Ctx::Free);
            ents.insert(Key::Def(path), Ent::Def { unit, f, ctx: c });
        }
        let mut ids = Ids {
            t,
            ents,
            traits,
            impls: Vec::new(),
            impls_of_type: BTreeMap::new(),
            impls_of_trait: BTreeMap::new(),
            edges: RefCell::default(),
            done: RefCell::default(),
        };
        for (unit, i) in blocks {
            let trait_path = ids.trait_path(&unit, &i.trait_path);
            if !ids.traits.contains(&trait_path) {
                continue;
            }
            let fns: Vec<(String, String)> = i
                .fns
                .iter()
                .filter_map(|f| {
                    let p = path_at.get(&(f.span.file, f.span.lo, f.span.hi))?;
                    Some((f.name.name.clone(), p.clone()))
                })
                .collect();
            let k = ids.impls.len();
            if let TypeKind::Named { path, .. } = &i.self_ty.kind {
                let full = if path.len() == 1 {
                    format!("{unit}::{}", path[0])
                } else {
                    path.join("::")
                };
                if t.adts.contains_key(&full) {
                    ids.impls_of_type.entry(full).or_default().push(k);
                }
            }
            ids.impls_of_trait
                .entry(trait_path.clone())
                .or_default()
                .push(k);
            ids.impls.push(ImplInfo {
                unit,
                i,
                trait_path,
                fns,
            });
        }
        ids
    }

    /// The hashes of `key`, or `None` when the program has no such item.
    pub fn get(&self, key: &Key) -> Option<Hashes> {
        if !self.ents.contains_key(key) {
            return None;
        }
        self.ensure(key);
        self.done.borrow().get(key).map(|d| d.hashes.clone())
    }

    /// The hashes of the definition with a body at `path`.
    pub fn def(&self, path: &str) -> Option<Hashes> {
        self.get(&Key::Def(path.to_string()))
    }

    /// Every item of the program, by key.
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.ents.keys()
    }

    /// The items `key`'s text refers to (the edges of the reference
    /// graph).
    pub fn refs(&self, key: &Key) -> Vec<Key> {
        if !self.ents.contains_key(key) {
            return Vec::new();
        }
        self.succ(key)
    }

    /// The items `key`'s surface refers to: a function's text without its
    /// body (signature, `#[uses]`, contract), a `spec fn`'s with its body
    /// (its body is its surface), a type's or a trait's whole text.
    pub fn surface_refs(&self, key: &Key) -> Vec<Key> {
        let part = match self.ents.get(key) {
            None => return Vec::new(),
            Some(Ent::Def { f, .. }) if f.kind != FnKind::Spec => Part::Iface,
            Some(_) => Part::Body,
        };
        let c = Collect(RefCell::default());
        let order: Vec<String> = match &self.ents[key] {
            Ent::Trait { t, .. } => t.fns.iter().map(|tf| tf.f.name.name.clone()).collect(),
            _ => Vec::new(),
        };
        self.print(key, &c, part, &order);
        c.0.into_inner()
            .into_iter()
            .filter(|k| self.ents.contains_key(k))
            .collect()
    }

    /// The items `key`'s text refers to.
    fn succ(&self, key: &Key) -> Vec<Key> {
        if let Some(e) = self.edges.borrow().get(key) {
            return e.clone();
        }
        let c = Collect(RefCell::default());
        let order: Vec<String> = match &self.ents[key] {
            Ent::Trait { t, .. } => t.fns.iter().map(|tf| tf.f.name.name.clone()).collect(),
            _ => Vec::new(),
        };
        self.print(key, &c, Part::Body, &order);
        let out: Vec<Key> =
            c.0.into_inner()
                .into_iter()
                .filter(|k| self.ents.contains_key(k))
                .collect();
        self.edges.borrow_mut().insert(key.clone(), out.clone());
        out
    }

    /// One trait `impl` as a line: `impl<..> #trait for <type> { #trait#k =
    /// #fn, .. }`.
    fn impl_line(&self, info: &ImplInfo, res: &dyn Resolve) -> String {
        let mut pr = Printer::identity(res, self.t, &self.traits, &info.unit);
        let g = pr.generics(&info.i.generics);
        let mut fns: Vec<String> = info
            .fns
            .iter()
            .map(|(name, path)| {
                format!(
                    "{} = {}",
                    pr.method_ref(&info.trait_path, name),
                    res.entity(&Key::Def(path.clone()))
                )
            })
            .collect();
        fns.sort();
        format!(
            "impl{g} {} for {} {{ {} }}\n",
            res.entity(&Key::Trait(info.trait_path.clone())),
            pr.ty(&info.i.self_ty),
            fns.join(", ")
        )
    }

    /// The `impl` lines of `list`, sorted (their order in the source is no
    /// part of the meaning).
    fn impl_lines(&self, list: Option<&Vec<usize>>, res: &dyn Resolve) -> String {
        let mut lines: Vec<String> = list
            .into_iter()
            .flatten()
            .map(|&k| self.impl_line(&self.impls[k], res))
            .collect();
        lines.sort();
        lines.concat()
    }

    /// The text of `key`; a trait's methods in `order` (names).
    fn print(&self, key: &Key, res: &dyn Resolve, part: Part, order: &[String]) -> String {
        match &self.ents[key] {
            Ent::Def { unit, f, ctx } => {
                let mut pr = Printer::identity(res, self.t, &self.traits, unit);
                // The `impl` a function is checked in: its type parameters
                // and, for a trait's, the trait method it defines (ADR-0046).
                let head = match ctx {
                    Ctx::Impl(i) if !i.generics.is_empty() => {
                        let g = pr.generics(&i.generics);
                        Some(format!("impl{g} {}", pr.applied_ty(&i.ty.name, &i.ty_args)))
                    }
                    Ctx::TraitImpl(i) => {
                        let g = pr.generics(&i.generics);
                        let tr = self.trait_path(unit, &i.trait_path);
                        Some(format!(
                            "impl{g} {} for {}",
                            pr.method_ref(&tr, &f.name.name),
                            pr.ty(&i.self_ty)
                        ))
                    }
                    Ctx::Default(tr) => {
                        Some(format!("default {}", pr.method_ref(tr, &f.name.name)))
                    }
                    _ => None,
                };
                if let Some(h) = head {
                    pr.out = format!("{h}\n");
                }
                if part == Part::Caller {
                    let mut g = (*f).clone();
                    g.attrs.retain(|a| a.name.name != "uses");
                    pr.function(&g, 0, Some("_"), false);
                } else {
                    pr.function(f, 0, Some("_"), part == Part::Body);
                }
                pr.out
            }
            Ent::Type { unit, t } => {
                let mut pr = Printer::identity(res, self.t, &self.traits, unit);
                pr.type_def_as(t, 0, "_");
                let lines = self.impl_lines(self.impls_of_type.get(key.path()), res);
                format!("{}{lines}", pr.out)
            }
            Ent::Trait { unit, t } => {
                let idx: Vec<usize> = order
                    .iter()
                    .filter_map(|m| t.fns.iter().position(|tf| &tf.f.name.name == m))
                    .collect();
                let mut pr = Printer::identity(res, self.t, &self.traits, unit);
                pr.trait_identity(t, &idx);
                let lines = self.impl_lines(self.impls_of_trait.get(key.path()), res);
                format!("{}{lines}", pr.out)
            }
        }
    }

    /// The full path of a trait written in `unit`.
    fn trait_path(&self, unit: &str, path: &[Ident]) -> String {
        let plain = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let local = format!("{unit}::{plain}");
        if path.len() == 1 && self.traits.contains(&local) {
            local
        } else {
            plain
        }
    }

    /// Hashes `start` and everything it reaches (Tarjan's algorithm, which
    /// finishes a component after the components it refers to).
    fn ensure(&self, start: &Key) {
        if self.done.borrow().contains_key(start) {
            return;
        }
        let mut index: BTreeMap<Key, usize> = BTreeMap::new();
        let mut low: BTreeMap<Key, usize> = BTreeMap::new();
        let mut on_stack: BTreeSet<Key> = BTreeSet::new();
        let mut stack: Vec<Key> = Vec::new();
        let mut frames: Vec<(Key, Vec<Key>, usize)> = Vec::new();
        index.insert(start.clone(), 0);
        low.insert(start.clone(), 0);
        stack.push(start.clone());
        on_stack.insert(start.clone());
        frames.push((start.clone(), self.succ(start), 0));
        while let Some((v, succ, next)) = frames.last_mut() {
            if *next < succ.len() {
                let w = succ[*next].clone();
                *next += 1;
                let v = v.clone();
                if self.done.borrow().contains_key(&w) {
                    continue;
                }
                if !index.contains_key(&w) {
                    let n = index.len();
                    index.insert(w.clone(), n);
                    low.insert(w.clone(), n);
                    stack.push(w.clone());
                    on_stack.insert(w.clone());
                    let s = self.succ(&w);
                    frames.push((w, s, 0));
                } else if on_stack.contains(&w) {
                    let m = low[&v].min(index[&w]);
                    low.insert(v, m);
                }
                continue;
            }
            let (v, _, _) = frames.pop().unwrap();
            if let Some((u, _, _)) = frames.last() {
                let m = low[u].min(low[&v]);
                low.insert(u.clone(), m);
            }
            if low[&v] == index[&v] {
                let mut comp = Vec::new();
                loop {
                    let w = stack.pop().unwrap();
                    on_stack.remove(&w);
                    let last = w == v;
                    comp.push(w);
                    if last {
                        break;
                    }
                }
                comp.sort();
                self.finish(comp);
            }
        }
    }

    /// The method order of trait `path` in component `comp`: each method's
    /// text with the component erased, with the erased texts of the `impl`
    /// functions that define it; ties by name.
    fn method_order(&self, path: &str, comp: &BTreeSet<Key>) -> Vec<String> {
        let Some(Ent::Trait { unit, t }) = self.ents.get(&Key::Trait(path.to_string())) else {
            return Vec::new();
        };
        let none = BTreeMap::new();
        let res = Final {
            ids: self,
            comp,
            member: Member::Erased,
            methods: &none,
        };
        let mut keyed: Vec<(String, String)> = t
            .fns
            .iter()
            .map(|tf| {
                let mut pr = Printer::identity(&res, self.t, &self.traits, unit);
                pr.trait_fn(tf, 0, Some("_"));
                let mut defs: Vec<String> = self
                    .impls_of_trait
                    .get(path)
                    .into_iter()
                    .flatten()
                    .flat_map(|&k| &self.impls[k].fns)
                    .filter(|(n, _)| n == &tf.f.name.name)
                    .map(|(_, p)| sha(&self.print(&Key::Def(p.clone()), &res, Part::Body, &[])))
                    .collect();
                defs.sort();
                (
                    sha(&format!("{}\n{}", pr.out, defs.join("\n"))),
                    tf.f.name.name.clone(),
                )
            })
            .collect();
        keyed.sort();
        keyed.into_iter().map(|(_, n)| n).collect()
    }

    /// Hashes one component whose references outside it are hashed.
    fn finish(&self, comp: Vec<Key>) {
        let cyclic = comp.len() > 1 || self.succ(&comp[0]).contains(&comp[0]);
        let set: BTreeSet<Key> = if cyclic {
            comp.iter().cloned().collect()
        } else {
            BTreeSet::new()
        };
        let methods: BTreeMap<String, Vec<String>> = comp
            .iter()
            .filter_map(|k| match k {
                Key::Trait(p) => Some((p.clone(), self.method_order(p, &set))),
                _ => None,
            })
            .collect();
        let order_of =
            |k: &Key| -> Vec<String> { methods.get(k.path()).cloned().unwrap_or_default() };
        let text = |k: &Key, member: Member, part: Part| {
            let res = Final {
                ids: self,
                comp: &set,
                member,
                methods: &methods,
            };
            self.print(k, &res, part, &order_of(k))
        };
        // The member order: erased hashes, refined until the partition
        // stops splitting; ties by path.
        let mut members = comp;
        if cyclic {
            let mut colors: BTreeMap<Key, String> = members
                .iter()
                .map(|k| (k.clone(), sha(&text(k, Member::Erased, Part::Body))))
                .collect();
            let classes = |c: &BTreeMap<Key, String>| c.values().collect::<BTreeSet<_>>().len();
            for _ in 0..members.len() {
                let next: BTreeMap<Key, String> = members
                    .iter()
                    .map(|k| {
                        let t = text(k, Member::Colored(&colors), Part::Body);
                        (k.clone(), sha(&format!("{}\n{t}", colors[k])))
                    })
                    .collect();
                let split = classes(&next) > classes(&colors);
                colors = next;
                if !split {
                    break;
                }
            }
            members.sort_by(|a, b| colors[a].cmp(&colors[b]).then(a.cmp(b)));
        }
        let at: BTreeMap<Key, usize> = members
            .iter()
            .enumerate()
            .map(|(i, k)| (k.clone(), i))
            .collect();
        let mut rows = Vec::new();
        for k in &members {
            let body_text = text(k, Member::At(&at), Part::Body);
            let body = sha(&body_text);
            let (iface, caller) = match &self.ents[k] {
                Ent::Def { f, .. } if f.kind == FnKind::Exec => (
                    sha(&text(k, Member::At(&at), Part::Iface)),
                    sha(&text(k, Member::At(&at), Part::Caller)),
                ),
                _ => (body.clone(), body.clone()),
            };
            rows.push((k.clone(), body, iface, caller, order_of(k)));
        }
        let comp_hash = if rows.len() == 1 {
            rows[0].1.clone()
        } else {
            sha(&rows
                .iter()
                .map(|r| format!("{}\n", r.1))
                .collect::<String>())
        };
        let n = rows.len();
        let mut done = self.done.borrow_mut();
        for (i, (k, body, iface, caller, methods)) in rows.into_iter().enumerate() {
            let id = if n == 1 {
                comp_hash.clone()
            } else {
                format!("{comp_hash}.{i}")
            };
            done.insert(
                k,
                Done {
                    hashes: Hashes {
                        id,
                        body,
                        iface,
                        caller,
                    },
                    methods,
                },
            );
        }
    }
}
