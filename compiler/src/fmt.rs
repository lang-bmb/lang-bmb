//! Normal form printer (ADR-0019 §1), the qualified text the incremental
//! keys read (ADR-0032), and the nameless identity printer (ADR-0059 §1;
//! `identity.rs` computes the hashes).

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::check::FnSig;
use crate::identity::Key;
use crate::source::Span;
use crate::verify::Tables;

/// How names are written: as in the source, fully qualified for the
/// incremental keys, or without names for the identity hash.
pub(crate) struct Printer<'a> {
    /// `(known definitions, unit of the definition being printed)`.
    qualify: Option<(&'a BTreeMap<String, FnSig>, String)>,
    /// The identity printer (ADR-0059 §1).
    id: Option<IdMode<'a>>,
    pub(crate) out: String,
}

/// How the identity printer writes a reference to another definition
/// (ADR-0059 §1): `#<id>`, `#self.k` for a member of the recursive component
/// being hashed, and a trait method or an enum variant as `#<id>#k`.
pub(crate) trait Resolve {
    fn entity(&self, key: &Key) -> String;
    /// Method `name` of trait `trait_path`: the trait's reference and the
    /// method's position in the trait (`identity.rs`).
    fn method(&self, trait_path: &str, name: &str) -> String;
}

/// The identity printer's state: the checker's tables to resolve names,
/// and the binders seen so far, each written as its position (`_0`, `_1`,
/// ..; labels `'_0`, ..). Binder names are unique in a function (E0303), so
/// one map serves the whole definition.
pub(crate) struct IdMode<'a> {
    refs: &'a dyn Resolve,
    t: &'a Tables,
    traits: &'a BTreeSet<String>,
    unit: String,
    binders: BTreeMap<String, String>,
    labels: BTreeMap<String, String>,
    next: usize,
    next_label: usize,
}

/// What a call path names, for the identity printer.
enum Callee {
    /// A definition (function, `spec fn`, trait method) by full path.
    Fn(String),
    /// Variant `k` of the enum at this full path.
    Variant(String, usize),
    /// A built-in (`print_i64`, `Some`, `Vec::new`, `std::mem::replace`).
    Plain,
}

/// The type without references and boxes around it (where a field or
/// variant is read through them).
fn peel(t: &TypeKind) -> &TypeKind {
    match t {
        TypeKind::Ref { inner, .. } => peel(&inner.kind),
        TypeKind::Box(e) => peel(&e.kind),
        t => t,
    }
}

impl IdMode<'_> {
    fn ty(&self, t: &TypeKind) -> String {
        match t {
            TypeKind::Named { path, args } => {
                let head = self.type_ref(path);
                if args.is_empty() {
                    head
                } else {
                    let a: Vec<String> = args.iter().map(|a| self.ty(&a.kind)).collect();
                    format!("{head}<{}>", a.join(", "))
                }
            }
            TypeKind::Array(e, n) => format!("[{}; {n}]", self.ty(&e.kind)),
            TypeKind::Slice(e) => format!("[{}]", self.ty(&e.kind)),
            TypeKind::Ref { mutable, inner } => format!(
                "&{}{}",
                if *mutable { "mut " } else { "" },
                self.ty(&inner.kind)
            ),
            TypeKind::Vec(e) => format!("Vec<{}>", self.ty(&e.kind)),
            TypeKind::Box(e) => format!("Box<{}>", self.ty(&e.kind)),
            TypeKind::Param(n) => self.binders.get(n).cloned().unwrap_or_else(|| n.clone()),
            t => crate::check::ty_str(t),
        }
    }

    /// A type path as written in the unit: a type parameter, a declared
    /// type (by reference), or a built-in by name.
    fn type_ref(&self, path: &[String]) -> String {
        if path.len() == 1
            && let Some(b) = self.binders.get(&path[0])
        {
            return b.clone();
        }
        let full = if path.len() == 1 {
            format!("{}::{}", self.unit, path[0])
        } else {
            path.join("::")
        };
        if self.t.adts.contains_key(&full) {
            self.refs.entity(&Key::Type(full))
        } else {
            path.join("::")
        }
    }

    /// A trait bound: a declared trait by reference, `Copy`, `Eq` and `Ord`
    /// by name.
    fn bound(&self, b: &[Ident]) -> String {
        let plain = path_str(b);
        let local = format!("{}::{plain}", self.unit);
        if b.len() == 1 && self.traits.contains(&local) {
            self.refs.entity(&Key::Trait(local))
        } else if self.traits.contains(&plain) {
            self.refs.entity(&Key::Trait(plain))
        } else {
            plain
        }
    }

    /// A reference to the function, `spec fn` or trait method at `path`.
    fn def_ref(&self, path: &str) -> String {
        match self.t.fns.get(path).and_then(|s| s.trait_of.as_ref()) {
            Some(tr) => self
                .refs
                .method(tr, path.rsplit("::").next().unwrap_or_default()),
            None => self.refs.entity(&Key::Def(path.to_string())),
        }
    }

    /// What a call path names, as the checker resolves it (`map::resolve`):
    /// `f`, `T::f` and `Trait::f` of the unit, a full path, or a variant of
    /// the call's enum type.
    fn callee(&self, path: &[Ident], span: Span) -> Callee {
        let plain = path_str(path);
        if path.len() <= 2 {
            let local = format!("{}::{plain}", self.unit);
            if let Some(sig) = self.t.fns.get(&local)
                && !sig.builtin
                && (path.len() == 1 || sig.in_impl || sig.trait_of.is_some())
            {
                return Callee::Fn(local);
            }
        }
        if self.t.fns.get(&plain).is_some_and(|s| !s.builtin) {
            return Callee::Fn(plain);
        }
        match path.last().and_then(|v| self.variant_at(span, &v.name)) {
            Some((tp, k)) => Callee::Variant(tp, k),
            None => Callee::Plain,
        }
    }

    /// The declared struct or enum of the expression at `span`.
    fn adt_at(&self, span: Span) -> Option<(String, &crate::check::Adt)> {
        let t = self.t.types.get(&(span.file, span.lo, span.hi))?;
        match peel(t) {
            TypeKind::Named { path, .. } => {
                let full = path.join("::");
                self.t.adts.get(&full).map(|a| (full, a))
            }
            _ => None,
        }
    }

    /// Variant `name` of the enum the expression at `span` has.
    fn variant_at(&self, span: Span, name: &str) -> Option<(String, usize)> {
        let (full, adt) = self.adt_at(span)?;
        match &adt.kind {
            crate::check::AdtKind::Enum(vs) => {
                vs.iter().position(|(n, _)| n == name).map(|k| (full, k))
            }
            _ => None,
        }
    }

    /// Position of field `name` of the struct the expression at `span` has.
    fn field_at(&self, span: Span, name: &str) -> Option<(String, usize)> {
        let (full, adt) = self.adt_at(span)?;
        match &adt.kind {
            crate::check::AdtKind::Struct(fs) => {
                fs.iter().position(|(n, _)| n == name).map(|k| (full, k))
            }
            _ => None,
        }
    }

    fn variant_ref(&self, tp: &str, k: usize) -> String {
        format!("{}#{k}", self.refs.entity(&Key::Type(tp.to_string())))
    }
}

const INDENT: &str = "    ";

/// Precedence of prefix operators (`-x`, `!x`, `&x`, `*p`).
const PREFIX: u8 = 11;
/// Precedence of postfix forms and atoms.
const POSTFIX: u8 = 12;

/// Rust's operator precedence (ADR-0019 §1, ADR-0048 §2).
fn prec(e: &Expr) -> u8 {
    match &e.kind {
        ExprKind::Binary(op, ..) => match op {
            BinOp::Implies => 1,
            BinOp::Or => 2,
            BinOp::And => 3,
            o if o.is_cmp() => 4,
            BinOp::BitOr => 5,
            BinOp::BitXor => 6,
            BinOp::BitAnd => 7,
            BinOp::Shl | BinOp::Shr => 8,
            BinOp::Add | BinOp::Sub => 9,
            _ => 10,
        },
        ExprKind::Unary(..) | ExprKind::Ref { .. } | ExprKind::Trigger(_) | ExprKind::Deref(_) => {
            PREFIX
        }
        // Always parenthesized when nested (ADR-0019 §1).
        ExprKind::If { .. }
        | ExprKind::Block(_)
        | ExprKind::Forall { .. }
        | ExprKind::Match { .. }
        | ExprKind::Loop { .. }
        | ExprKind::Return(_)
        | ExprKind::Break { .. } => 0,
        _ => POSTFIX,
    }
}

pub fn type_str(t: &Type) -> String {
    crate::check::ty_str(&t.kind)
}

fn path_str(p: &[Ident]) -> String {
    p.iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

/// `<T: Copy, U: Copy + geo::Shape>`, or nothing (ADR-0046).
pub fn generics_str(gs: &[TypeParam]) -> String {
    if gs.is_empty() {
        return String::new();
    }
    let ps: Vec<String> = gs
        .iter()
        .map(|g| {
            if g.bounds.is_empty() {
                return g.name.name.clone();
            }
            let bs: Vec<String> = g
                .bounds
                .iter()
                .map(|b| {
                    b.iter()
                        .map(|i| i.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::")
                })
                .collect();
            format!("{}: {}", g.name.name, bs.join(" + "))
        })
        .collect();
    format!("<{}>", ps.join(", "))
}

/// `T<A, B>`, or `T`.
fn applied(name: &str, args: &[Type]) -> String {
    if args.is_empty() {
        return name.to_string();
    }
    let a: Vec<String> = args.iter().map(type_str).collect();
    format!("{name}<{}>", a.join(", "))
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl<'a> Printer<'a> {
    fn path(&self, path: &[Ident]) -> String {
        let plain = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        match &self.qualify {
            // `f`, `T::f` (an `impl` function of the unit's type) and
            // `Trait::f` (a method of the unit's trait) are local, as the
            // checker resolves them.
            Some((fns, unit)) if path.len() <= 2 => {
                let local = format!("{unit}::{plain}");
                match fns.get(&local) {
                    Some(sig) if path.len() == 1 || sig.in_impl || sig.trait_of.is_some() => local,
                    _ => plain,
                }
            }
            _ => plain,
        }
    }

    fn uses_name(&self, name: &str) -> String {
        if let Some(m) = &self.id {
            return match crate::check::uses_target(name, &m.unit, &m.t.fns) {
                Some(p) => m.def_ref(&p),
                None => name.to_string(),
            };
        }
        match &self.qualify {
            Some((fns, unit)) => {
                crate::check::uses_target(name, unit, fns).unwrap_or_else(|| name.to_string())
            }
            _ => name.to_string(),
        }
    }

    // ---------- names (identity printer, ADR-0059 §1) ----------

    /// The identity printer over the definitions of `unit`.
    pub(crate) fn identity(
        refs: &'a dyn Resolve,
        t: &'a Tables,
        traits: &'a BTreeSet<String>,
        unit: &str,
    ) -> Printer<'a> {
        Printer {
            qualify: None,
            id: Some(IdMode {
                refs,
                t,
                traits,
                unit: unit.to_string(),
                binders: BTreeMap::new(),
                labels: BTreeMap::new(),
                next: 0,
                next_label: 0,
            }),
            out: String::new(),
        }
    }

    /// Forgets the binders (a new definition, or the next trait method).
    pub(crate) fn reset_binders(&mut self) {
        if let Some(m) = &mut self.id {
            m.binders.clear();
            m.labels.clear();
            m.next = 0;
            m.next_label = 0;
        }
    }

    /// A type as written: in the identity printer, without names.
    pub(crate) fn ty(&self, t: &Type) -> String {
        match &self.id {
            Some(m) => m.ty(&t.kind),
            None => type_str(t),
        }
    }

    /// Binds `name` (a parameter, local, type parameter, ..): in the
    /// identity printer, its position.
    fn bind(&mut self, name: &str) -> String {
        match &mut self.id {
            Some(m) => {
                let b = format!("_{}", m.next);
                m.next += 1;
                m.binders.insert(name.to_string(), b.clone());
                b
            }
            None => name.to_string(),
        }
    }

    /// A use of a bound name; any other name (`None`) as written.
    fn local(&self, name: &str) -> String {
        match &self.id {
            Some(m) => m
                .binders
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.to_string()),
            None => name.to_string(),
        }
    }

    fn bind_label(&mut self, name: &str) -> String {
        match &mut self.id {
            Some(m) => {
                let b = format!("_{}", m.next_label);
                m.next_label += 1;
                m.labels.insert(name.to_string(), b.clone());
                b
            }
            None => name.to_string(),
        }
    }

    fn label(&self, name: &str) -> String {
        match &self.id {
            Some(m) => m
                .labels
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.to_string()),
            None => name.to_string(),
        }
    }

    /// `<T: Copy, U: Copy + geo::Shape>` (binding the parameters), or
    /// nothing.
    pub(crate) fn generics(&mut self, gs: &[TypeParam]) -> String {
        if self.id.is_none() {
            return generics_str(gs);
        }
        if gs.is_empty() {
            return String::new();
        }
        let ps: Vec<String> = gs
            .iter()
            .map(|g| {
                let n = self.bind(&g.name.name);
                let m = self.id.as_ref().unwrap();
                if g.bounds.is_empty() {
                    return n;
                }
                let bs: Vec<String> = g.bounds.iter().map(|b| m.bound(b)).collect();
                format!("{n}: {}", bs.join(" + "))
            })
            .collect();
        format!("<{}>", ps.join(", "))
    }

    /// A reference to a declared type by its name in the unit (an `impl`
    /// head) with its arguments.
    pub(crate) fn applied_ty(&self, name: &str, args: &[Type]) -> String {
        match &self.id {
            Some(m) => {
                let head = m.type_ref(&[name.to_string()]);
                if args.is_empty() {
                    head
                } else {
                    let a: Vec<String> = args.iter().map(|a| m.ty(&a.kind)).collect();
                    format!("{head}<{}>", a.join(", "))
                }
            }
            None => applied(name, args),
        }
    }

    /// A trait method by its trait's path (identity printer).
    pub(crate) fn method_ref(&self, trait_path: &str, name: &str) -> String {
        match &self.id {
            Some(m) => m.refs.method(trait_path, name),
            None => format!("{trait_path}::{name}"),
        }
    }

    /// A pattern; in the identity printer the variant by its position in
    /// the scrutinee's enum and the binders by theirs.
    fn pat(&mut self, p: &Pat, scrut: Span) -> String {
        if self.id.is_none() {
            return pat_str(p);
        }
        let m = self.id.as_ref().unwrap();
        let path = match p.path.last().and_then(|v| m.variant_at(scrut, &v.name)) {
            Some((tp, k)) => m.variant_ref(&tp, k),
            None => path_str(&p.path),
        };
        if !p.parens {
            return path;
        }
        let bs: Vec<String> = p
            .binds
            .iter()
            .map(|b| b.as_ref().map_or("_".to_string(), |i| self.bind(&i.name)))
            .collect();
        format!("{path}({})", bs.join(", "))
    }

    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str(INDENT);
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    // ---------- items ----------

    fn module(&mut self, m: &Mod, depth: usize) {
        let summary = m.attrs.iter().find_map(|a| match a.args.first() {
            Some(AttrArg::Str(s, _)) if a.name.name == "summary" => Some(s.clone()),
            _ => None,
        });
        if let Some(s) = summary {
            self.line(depth, &format!("#[summary(\"{}\")]", escape(&s)));
        }
        self.line(depth, &format!("mod {} {{", m.name.name));
        let mut subs: Vec<&Mod> = m.mods.iter().collect();
        subs.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let mut fns: Vec<&FnDef> = m.fns.iter().collect();
        fns.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        // Types by name, each followed by its `impl` block (ADR-0026 §6).
        let mut types: Vec<&TypeDef> = m.types.iter().collect();
        types.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let mut impls: Vec<&Impl> = m.impls.iter().collect();
        impls.sort_by(|a, b| a.ty.name.cmp(&b.ty.name));
        // Traits by name, then the trait `impl`s for types of other units;
        // an `impl` for a type of this unit follows that type (ADR-0046).
        let mut traits: Vec<&TraitDef> = m.traits.iter().collect();
        traits.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let local_type = |i: &TraitImpl| match &i.self_ty.kind {
            TypeKind::Named { path, .. } if path.len() == 1 => {
                m.types.iter().any(|t| t.name.name == path[0])
            }
            _ => false,
        };
        let mut timpls: Vec<&TraitImpl> = m.trait_impls.iter().collect();
        timpls.sort_by_key(|i| (path_str(&i.trait_path), type_str(&i.self_ty)));
        let mut first = true;
        for s in subs {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.module(s, depth + 1);
        }
        for t in &traits {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.trait_def(t, depth + 1);
        }
        for i in timpls.iter().filter(|i| !local_type(i)) {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.trait_impl(i, depth + 1);
        }
        let mut printed = BTreeSet::new();
        for t in &types {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.type_def(t, depth + 1);
            for i in impls.iter().filter(|i| i.ty.name == t.name.name) {
                self.out.push('\n');
                self.impl_block(i, depth + 1);
                printed.insert(i.ty.name.clone());
            }
            for i in timpls.iter().filter(|i| {
                matches!(&i.self_ty.kind, TypeKind::Named { path, .. } if path.len() == 1 && path[0] == t.name.name)
            }) {
                self.out.push('\n');
                self.trait_impl(i, depth + 1);
            }
        }
        // An `impl` without its type in the unit (the checker reports it).
        for i in impls.iter().filter(|i| !printed.contains(&i.ty.name)) {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.impl_block(i, depth + 1);
        }
        for f in fns {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.function(f, depth + 1, None, true);
        }
        self.line(depth, "}");
    }

    fn attr_line(&mut self, a: &Attr, depth: usize) {
        // A type's invariant names its predicate as `#[uses]` does: by
        // full path in the qualified text, by `id` in the identity text
        // (ADR-0060 §3, ADR-0059 §1).
        let inv = a.name.name == "invariant";
        let args: Vec<String> = a
            .args
            .iter()
            .map(|x| match x {
                AttrArg::Ident(i) if inv => self.uses_name(&i.name),
                AttrArg::Ident(i) => i.name.clone(),
                AttrArg::Str(s, _) => format!("\"{}\"", escape(s)),
            })
            .collect();
        self.line(depth, &format!("#[{}({})]", a.name.name, args.join(", ")));
    }

    /// `struct T { f: U, .. }` / `enum T { A, B(U), .. }` on one line; fields and
    /// variants keep their declaration order (ADR-0026 §6).
    pub(crate) fn type_def(&mut self, t: &TypeDef, depth: usize) {
        self.type_def_as(t, depth, &t.name.name);
    }

    pub(crate) fn type_def_as(&mut self, t: &TypeDef, depth: usize, name: &str) {
        for a in &t.attrs {
            self.attr_line(a, depth);
        }
        let gens = self.generics(&t.generics);
        if self.id.is_some() {
            // Fields and variants by position: their types (ADR-0059 §1).
            let (kw, items): (&str, Vec<String>) = match &t.kind {
                TypeDefKind::Struct(fs) => ("struct", fs.iter().map(|f| self.ty(&f.ty)).collect()),
                TypeDefKind::Enum(vs) => (
                    "enum",
                    vs.iter()
                        .map(|v| {
                            let ps: Vec<String> = v.payload.iter().map(|p| self.ty(p)).collect();
                            format!("({})", ps.join(", "))
                        })
                        .collect(),
                ),
            };
            self.line(
                depth,
                &format!("{kw} {name}{gens} {{ {} }}", items.join(", ")),
            );
            return;
        }
        let (kw, items): (&str, Vec<String>) = match &t.kind {
            TypeDefKind::Struct(fs) => (
                "struct",
                fs.iter()
                    .map(|f| format!("{}: {}", f.name.name, type_str(&f.ty)))
                    .collect(),
            ),
            TypeDefKind::Enum(vs) => (
                "enum",
                vs.iter()
                    .map(|v| {
                        if v.payload.is_empty() {
                            v.name.name.clone()
                        } else {
                            let ps: Vec<String> = v.payload.iter().map(type_str).collect();
                            format!("{}({})", v.name.name, ps.join(", "))
                        }
                    })
                    .collect(),
            ),
        };
        let name = format!("{name}{gens}");
        if items.is_empty() {
            self.line(depth, &format!("{kw} {name} {{}}"));
        } else {
            self.line(depth, &format!("{kw} {name} {{ {} }}", items.join(", ")));
        }
    }

    fn impl_block(&mut self, i: &Impl, depth: usize) {
        let mut fns: Vec<&FnDef> = i.fns.iter().collect();
        fns.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let head = format!(
            "impl{} {}",
            generics_str(&i.generics),
            applied(&i.ty.name, &i.ty_args)
        );
        if fns.is_empty() {
            self.line(depth, &format!("{head} {{}}"));
            return;
        }
        self.line(depth, &format!("{head} {{"));
        for (k, f) in fns.iter().enumerate() {
            if k > 0 {
                self.out.push('\n');
            }
            self.function(f, depth + 1, None, true);
        }
        self.line(depth, "}");
    }

    /// `trait T { fn m(..) -> R; .. }`, methods by name (ADR-0046 §1).
    fn trait_def(&mut self, t: &TraitDef, depth: usize) {
        for a in &t.attrs {
            self.attr_line(a, depth);
        }
        let mut fns: Vec<&TraitFn> = t.fns.iter().collect();
        fns.sort_by(|a, b| a.f.name.name.cmp(&b.f.name.name));
        if fns.is_empty() {
            self.line(depth, &format!("trait {} {{}}", t.name.name));
            return;
        }
        self.line(depth, &format!("trait {} {{", t.name.name));
        for (k, tf) in fns.iter().enumerate() {
            if k > 0 {
                self.out.push('\n');
            }
            self.trait_fn(tf, depth + 1, None);
        }
        self.line(depth, "}");
    }

    /// A trait method: its default body, or its signature ending with `;`.
    pub(crate) fn trait_fn(&mut self, tf: &TraitFn, depth: usize, rename: Option<&str>) {
        if tf.has_body {
            self.function(&tf.f, depth, rename, true);
        } else {
            self.function(&tf.f, depth, rename, false);
            // The signature ends with `;` (after its last clause).
            if self.out.ends_with(",\n") {
                self.out.truncate(self.out.len() - 2);
            } else {
                self.out.pop();
            }
            self.out.push_str(";\n");
        }
    }

    /// The trait's identity text (ADR-0059 §1): its attributes and its
    /// methods in `order` (positions in `t.fns`), each without names.
    pub(crate) fn trait_identity(&mut self, t: &TraitDef, order: &[usize]) {
        for a in &t.attrs {
            self.attr_line(a, 0);
        }
        self.line(0, "trait _ {");
        for (k, &i) in order.iter().enumerate() {
            if k > 0 {
                self.out.push('\n');
            }
            self.reset_binders();
            self.trait_fn(&t.fns[i], 1, Some("_"));
        }
        self.line(0, "}");
    }

    /// `impl<..> Trait for T { .. }`, functions by name (ADR-0046 §2).
    fn trait_impl(&mut self, i: &TraitImpl, depth: usize) {
        let mut fns: Vec<&FnDef> = i.fns.iter().collect();
        fns.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let head = format!(
            "impl{} {} for {}",
            generics_str(&i.generics),
            path_str(&i.trait_path),
            type_str(&i.self_ty)
        );
        if fns.is_empty() {
            self.line(depth, &format!("{head} {{}}"));
            return;
        }
        self.line(depth, &format!("{head} {{"));
        for (k, f) in fns.iter().enumerate() {
            if k > 0 {
                self.out.push('\n');
            }
            self.function(f, depth + 1, None, true);
        }
        self.line(depth, "}");
    }

    /// `rename`: print the definition's own name as this (hashing uses `_`).
    pub(crate) fn function(
        &mut self,
        f: &FnDef,
        depth: usize,
        rename: Option<&str>,
        with_body: bool,
    ) {
        let effects = f.attrs.iter().find(|a| a.name.name == "effects");
        if let Some(a) = effects {
            let args: Vec<String> = a
                .args
                .iter()
                .map(|x| match x {
                    AttrArg::Ident(i) => i.name.clone(),
                    AttrArg::Str(s, _) => format!("\"{}\"", escape(s)),
                })
                .collect();
            self.line(depth, &format!("#[effects({})]", args.join(", ")));
        }
        let uses: BTreeSet<String> = f
            .attrs
            .iter()
            .filter(|a| a.name.name == "uses")
            .flat_map(|a| a.args.iter())
            .filter_map(|x| match x {
                AttrArg::Ident(i) => Some(self.uses_name(&i.name)),
                AttrArg::Str(..) => None,
            })
            .collect();
        if !uses.is_empty() {
            self.line(
                depth,
                &format!(
                    "#[uses({})]",
                    uses.into_iter().collect::<Vec<_>>().join(", ")
                ),
            );
        }
        // Type parameters first: the parameter types name them.
        let gens = self.generics(&f.generics);
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| {
                let n = self.bind(&p.name.name);
                format!("{n}: {}", self.ty(&p.ty))
            })
            .collect();
        let ret = match &f.ret.name {
            Some(n) => {
                let n = self.bind(&n.name);
                format!("({n}: {})", self.ty(&f.ret.ty))
            }
            None => self.ty(&f.ret.ty),
        };
        let name = rename.unwrap_or(&f.name.name);
        let sig = format!(
            "{} {name}{gens}({}){}",
            fn_keyword(f),
            params.join(", "),
            ret_text(f, &ret)
        );
        let has_clauses = !f.requires.is_empty() || !f.ensures.is_empty() || f.decreases.is_some();
        if !with_body {
            self.line(depth, &sig);
            self.clauses(f, depth);
            return;
        }
        if has_clauses {
            self.line(depth, &sig);
            self.clauses(f, depth);
            self.block_lines(&f.body, depth, "");
        } else {
            self.block_open(&f.body, depth, &format!("{sig} "), "");
        }
    }

    fn clauses(&mut self, f: &FnDef, depth: usize) {
        for (kw, items) in [("requires", &f.requires), ("ensures", &f.ensures)] {
            if !items.is_empty() {
                let xs: Vec<String> = items.iter().map(|e| self.expr(e, 0)).collect();
                self.line(depth + 1, &format!("{kw} {},", xs.join(", ")));
            }
        }
        if let Some(d) = &f.decreases {
            let d = self.expr(d, 0);
            self.line(depth + 1, &format!("decreases {d},"));
        }
    }

    // ---------- blocks and statements ----------

    /// Prints `{` on its own line at `depth`, then the block.
    fn block_lines(&mut self, b: &Block, depth: usize, suffix: &str) {
        self.block_open(b, depth, "", suffix);
    }

    /// Prints `prefix{` ... `}suffix` with the block's contents indented.
    fn block_open(&mut self, b: &Block, depth: usize, prefix: &str, suffix: &str) {
        if b.stmts.is_empty() && b.tail.is_none() {
            self.line(depth, &format!("{prefix}{{}}{suffix}"));
            return;
        }
        self.line(depth, &format!("{prefix}{{"));
        for s in &b.stmts {
            self.stmt(s, depth + 1);
        }
        if let Some(t) = &b.tail {
            self.expr_stmt(t, depth + 1, "");
        }
        self.line(depth, &format!("}}{suffix}"));
    }

    /// Prints an expression that may span lines (`if`, blocks) as a statement.
    fn expr_stmt(&mut self, e: &Expr, depth: usize, suffix: &str) {
        match &e.kind {
            ExprKind::If { .. } => self.if_lines(e, depth, "", suffix),
            ExprKind::Block(b) => self.block_open(b, depth, "", suffix),
            ExprKind::Match { .. } => self.match_lines(e, depth, "", suffix),
            ExprKind::Loop { .. } => self.loop_lines(e, depth, "", suffix),
            _ => {
                let s = self.expr(e, 0);
                self.line(depth, &format!("{s}{suffix}"));
            }
        }
    }

    /// `head['l: ]loop`, its clauses on their own lines as `while`'s, and
    /// the body.
    fn loop_lines(&mut self, e: &Expr, depth: usize, head: &str, suffix: &str) {
        let ExprKind::Loop {
            label,
            invariants,
            decreases,
            body,
        } = &e.kind
        else {
            unreachable!()
        };
        let l = label.as_ref().map_or(String::new(), |l| {
            format!("'{}: ", self.bind_label(&l.name))
        });
        if invariants.is_empty() && decreases.is_none() {
            self.block_open(body, depth, &format!("{head}{l}loop "), suffix);
            return;
        }
        self.line(depth, &format!("{head}{l}loop"));
        if !invariants.is_empty() {
            let xs: Vec<String> = invariants.iter().map(|e| self.expr(e, 0)).collect();
            self.line(depth + 1, &format!("invariant {},", xs.join(", ")));
        }
        if let Some(d) = decreases {
            let d = self.expr(d, 0);
            self.line(depth + 1, &format!("decreases {d},"));
        }
        self.block_open(body, depth, "", suffix);
    }

    /// `match` with one arm per line (each arm's body on one line).
    fn match_lines(&mut self, e: &Expr, depth: usize, prefix: &str, suffix: &str) {
        let ExprKind::Match { scrut, arms } = &e.kind else {
            unreachable!()
        };
        let s = self.cond(scrut);
        self.line(depth, &format!("{prefix}match {s} {{"));
        for a in arms {
            // The pattern binds names the body uses.
            let pat = self.pat(&a.pat, scrut.span);
            let body = self.expr(&a.body, 0);
            self.line(depth + 1, &format!("{pat} => {body},"));
        }
        self.line(depth, &format!("}}{suffix}"));
    }

    /// A condition or scrutinee: parenthesized when it holds a struct literal,
    /// which Rust does not accept there unparenthesized.
    fn cond(&mut self, e: &Expr) -> String {
        let s = self.expr(e, 0);
        if has_struct_lit(e) {
            format!("({s})")
        } else {
            s
        }
    }

    fn if_lines(&mut self, e: &Expr, depth: usize, prefix: &str, suffix: &str) {
        let ExprKind::If { cond, then, els } = &e.kind else {
            unreachable!()
        };
        let c = self.cond(cond);
        match els {
            None => self.block_open(then, depth, &format!("{prefix}if {c} "), suffix),
            Some(x) => {
                self.block_open(then, depth, &format!("{prefix}if {c} "), "");
                // Join `}` and `else` on one line.
                self.out.pop();
                match &x.kind {
                    ExprKind::If { .. } => {
                        let rest = self.capture(|p| p.if_lines(x, depth, "", suffix));
                        self.out.push_str(" else ");
                        self.out.push_str(rest.trim_start());
                    }
                    ExprKind::Block(b) => {
                        let rest = self.capture(|p| p.block_open(b, depth, "", suffix));
                        self.out.push_str(" else ");
                        self.out.push_str(rest.trim_start());
                    }
                    _ => unreachable!("else branch is a block or if"),
                }
            }
        }
    }

    fn capture(&mut self, f: impl FnOnce(&mut Self)) -> String {
        let saved = std::mem::take(&mut self.out);
        f(self);
        std::mem::replace(&mut self.out, saved)
    }

    fn stmt(&mut self, s: &Stmt, depth: usize) {
        match s {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
                ..
            } => {
                let m = if *mutable { "mut " } else { "" };
                let n = self.bind(&name.name);
                let head = format!("let {m}{n}: {} = ", self.ty(ty));
                self.multiline_value(init, depth, &head, ";");
            }
            Stmt::Assign { target, value, .. } => {
                let head = format!("{} = ", self.expr(target, 0));
                self.multiline_value(value, depth, &head, ";");
            }
            Stmt::While {
                label,
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                let l = label.as_ref().map_or(String::new(), |l| {
                    format!("'{}: ", self.bind_label(&l.name))
                });
                let c = self.cond(cond);
                self.line(depth, &format!("{l}while {c}"));
                if !invariants.is_empty() {
                    let xs: Vec<String> = invariants.iter().map(|e| self.expr(e, 0)).collect();
                    self.line(depth + 1, &format!("invariant {},", xs.join(", ")));
                }
                if let Some(d) = decreases {
                    let d = self.expr(d, 0);
                    self.line(depth + 1, &format!("decreases {d},"));
                }
                self.block_lines(body, depth, "");
            }
            Stmt::For {
                label,
                var,
                lo,
                hi,
                inclusive,
                invariants,
                body,
                ..
            } => {
                let l = label.as_ref().map_or(String::new(), |l| {
                    format!("'{}: ", self.bind_label(&l.name))
                });
                let (a, b) = (self.cond(lo), self.cond(hi));
                let op = if *inclusive { "..=" } else { ".." };
                let v = self.bind(&var.name);
                let head = format!("{l}for {v} in {a}{op}{b}");
                if invariants.is_empty() {
                    self.block_open(body, depth, &format!("{head} "), "");
                } else {
                    self.line(depth, &head);
                    let xs: Vec<String> = invariants.iter().map(|e| self.expr(e, 0)).collect();
                    self.line(depth + 1, &format!("invariant {},", xs.join(", ")));
                    self.block_lines(body, depth, "");
                }
            }
            Stmt::Assert { cond, by, .. } => {
                let c = self.expr(cond, 0);
                let by = by.map_or_else(String::new, |p| format!(" by ({})", p.name()));
                self.line(depth, &format!("assert({c}){by};"));
            }
            // A statement where it is written: what it proves holds from
            // there on (ADR-0062, open question 3).
            Stmt::Proof { body, .. } => self.block_open(body, depth, "proof ", ""),
            Stmt::LetFields {
                path, fields, init, ..
            } => {
                let ty = match &self.id {
                    Some(m) => m.type_ref(&path.iter().map(|i| i.name.clone()).collect::<Vec<_>>()),
                    None => self.path(path),
                };
                let fs: Vec<String> = fields.iter().map(|f| self.bind(&f.name)).collect();
                let head = format!("let {ty} {{ {} }} = ", fs.join(", "));
                self.multiline_value(init, depth, &head, ";");
            }
            Stmt::Expr(e) => match &e.kind {
                ExprKind::If { .. }
                | ExprKind::Block(_)
                | ExprKind::Match { .. }
                | ExprKind::Loop { .. } => self.expr_stmt(e, depth, ""),
                _ => {
                    let x = self.expr(e, 0);
                    self.line(depth, &format!("{x};"));
                }
            },
        }
    }

    /// `head value;` where the value may be a multi-line `if` or block.
    fn multiline_value(&mut self, v: &Expr, depth: usize, head: &str, suffix: &str) {
        match &v.kind {
            ExprKind::If { .. } => self.if_lines(v, depth, head, suffix),
            ExprKind::Block(b) => self.block_open(b, depth, head, suffix),
            ExprKind::Match { .. } => self.match_lines(v, depth, head, suffix),
            ExprKind::Loop { .. } => self.loop_lines(v, depth, head, suffix),
            _ => {
                let x = self.expr(v, 0);
                self.line(depth, &format!("{head}{x}{suffix}"));
            }
        }
    }

    // ---------- expressions (single line) ----------

    /// Prints `e`, parenthesized if its precedence is below `min`.
    fn expr(&mut self, e: &Expr, min: u8) -> String {
        let s = self.expr_raw(e);
        if prec(e) < min || (prec(e) == 0 && min > 0) {
            format!("({s})")
        } else {
            s
        }
    }

    fn expr_raw(&mut self, e: &Expr) -> String {
        match &e.kind {
            ExprKind::Int(v, r) => r.text(*v),
            ExprKind::ByteLit(v) => crate::lexer::byte_text(&[*v], '\''),
            ExprKind::ByteStr(v) => crate::lexer::byte_text(v, '"'),
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Unit => "()".into(),
            ExprKind::Return(None) => "return".into(),
            ExprKind::Return(Some(v)) => format!("return {}", self.expr(v, 0)),
            ExprKind::Break { label, value } => {
                let mut t = "break".to_string();
                if let Some(l) = label {
                    t.push_str(&format!(" '{}", self.label(&l.name)));
                }
                if let Some(v) = value {
                    t.push_str(&format!(" {}", self.expr(v, 0)));
                }
                t
            }
            ExprKind::Continue { label: None } => "continue".into(),
            ExprKind::Continue { label: Some(l) } => format!("continue '{}", self.label(&l.name)),
            ExprKind::Var(id) => self.local(&id.name),
            ExprKind::Call { path, targs, args } if self.id.is_some() => {
                let a: Vec<String> = args.iter().map(|x| self.expr(x, 0)).collect();
                let m = self.id.as_ref().unwrap();
                // Type arguments by their segment counted from the end
                // (`f::<A>`: 0, `T::<A>::f`: 1), whatever the path's length.
                let ts: String = targs
                    .iter()
                    .map(|t| {
                        let xs: Vec<String> = t.args.iter().map(|a| m.ty(&a.kind)).collect();
                        format!("::[{}]<{}>", path.len() - 1 - t.seg, xs.join(", "))
                    })
                    .collect();
                let head = match m.callee(path, e.span) {
                    Callee::Fn(p) => m.def_ref(&p),
                    Callee::Variant(tp, k) => m.variant_ref(&tp, k),
                    Callee::Plain => path_str(path),
                };
                format!("{head}{ts}({})", a.join(", "))
            }
            ExprKind::Call { path, targs, args } => {
                let a: Vec<String> = args.iter().map(|x| self.expr(x, 0)).collect();
                let mut segs: Vec<String> = self.path(path).split("::").map(String::from).collect();
                // A qualified path gains its unit's segments in front.
                let off = segs.len() - path.len();
                for t in targs {
                    let ts: Vec<String> = t.args.iter().map(type_str).collect();
                    segs[t.seg + off] = format!("{}::<{}>", segs[t.seg + off], ts.join(", "));
                }
                format!("{}({})", segs.join("::"), a.join(", "))
            }
            ExprKind::Method { recv, name, args } => {
                let r = self.expr(recv, POSTFIX);
                let a: Vec<String> = args.iter().map(|x| self.expr(x, 0)).collect();
                format!("{r}.{}({})", name.name, a.join(", "))
            }
            ExprKind::Index { base, index } => {
                format!("{}[{}]", self.expr(base, POSTFIX), self.expr(index, 0))
            }
            ExprKind::Unary(op, x) => {
                let o = if *op == UnOp::Neg { "-" } else { "!" };
                format!("{o}{}", self.expr(x, PREFIX))
            }
            ExprKind::Binary(op, l, r) => {
                let p = prec(e);
                let (lm, rm) = match op {
                    BinOp::Implies => (p + 1, p),
                    o if o.is_cmp() => (p + 1, p + 1),
                    _ => (p, p + 1),
                };
                format!("{} {} {}", self.expr(l, lm), op.symbol(), self.expr(r, rm))
            }
            ExprKind::Ref { mutable, expr } => {
                format!(
                    "&{}{}",
                    if *mutable { "mut " } else { "" },
                    self.expr(expr, PREFIX)
                )
            }
            ExprKind::Array(items) => {
                let a: Vec<String> = items.iter().map(|x| self.expr(x, 0)).collect();
                format!("[{}]", a.join(", "))
            }
            ExprKind::Repeat { value, count, .. } => format!("[{}; {count}]", self.expr(value, 0)),
            ExprKind::Old(x) => format!("old({})", self.expr(x, 0)),
            ExprKind::Forall { binders, body } => {
                let bs: Vec<String> = binders
                    .iter()
                    .map(|b| {
                        let n = self.bind(&b.name.name);
                        format!("{n}: {}", self.ty(&b.ty))
                    })
                    .collect();
                format!("forall|{}| {}", bs.join(", "), self.expr(body, 0))
            }
            ExprKind::Trigger(x) => format!("#[trigger] {}", self.expr(x, PREFIX)),
            ExprKind::StructLit { path, fields, rest } => {
                let mut fs: Vec<String> = fields
                    .iter()
                    .map(|(n, x)| {
                        // A field by its position (identity printer).
                        let f = match self.id.as_ref().and_then(|m| m.field_at(e.span, &n.name)) {
                            Some((_, k)) => k.to_string(),
                            None => n.name.clone(),
                        };
                        format!("{f}: {}", self.expr(x, 0))
                    })
                    .collect();
                if let Some(r) = rest {
                    fs.push(format!("..{}", self.expr(r, 0)));
                }
                let head = match self.id.as_ref().and_then(|m| m.adt_at(e.span)) {
                    Some((tp, _)) => self.id.as_ref().unwrap().refs.entity(&Key::Type(tp)),
                    None => self.path(path),
                };
                format!("{head} {{ {} }}", fs.join(", "))
            }
            ExprKind::Path(path) => {
                match self.id.as_ref().and_then(|m| {
                    let v = path.last()?;
                    let (tp, k) = m.variant_at(e.span, &v.name)?;
                    Some(m.variant_ref(&tp, k))
                }) {
                    Some(v) => v,
                    None => self.path(path),
                }
            }
            ExprKind::Deref(x) => format!("*{}", self.expr(x, PREFIX)),
            ExprKind::Try(x) => format!("{}?", self.expr(x, POSTFIX)),
            ExprKind::Field { base, name } => {
                let f = match self
                    .id
                    .as_ref()
                    .and_then(|m| m.field_at(base.span, &name.name))
                {
                    Some((_, k)) => k.to_string(),
                    None => name.name.clone(),
                };
                format!("{}.{f}", self.expr(base, POSTFIX))
            }
            // Nested multi-line forms are printed on one line inside expressions.
            ExprKind::If { .. }
            | ExprKind::Block(_)
            | ExprKind::Match { .. }
            | ExprKind::Loop { .. } => {
                let text = self.capture(|p| p.expr_stmt(e, 0, ""));
                text.lines().map(str::trim).collect::<Vec<_>>().join(" ")
            }
            ExprKind::Error => "<error>".into(),
        }
    }
}

/// Normal form of a whole program, one string per top-level unit, keyed by
/// unit name (ADR-0019 §3).
pub fn format_units(p: &Program) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for m in &p.mods {
        let mut pr = plain();
        pr.module(m, 0);
        out.insert(m.name.name.clone(), pr.out);
    }
    out
}

/// The identity and the body and interface hashes of one definition or
/// type declaration (ADR-0059 §1, §4).
pub struct DefHash {
    pub path: String,
    pub id: String,
    pub body: String,
    pub iface: String,
}

/// Qualified normal form of one definition: other definitions by full path
/// (ADR-0019 §2). The incremental keys read it (ADR-0032); the identity
/// hashes do not (ADR-0059).
pub fn qualified_text(
    f: &FnDef,
    unit: &str,
    fns: &BTreeMap<String, FnSig>,
    with_body: bool,
) -> String {
    let mut pr = Printer {
        qualify: Some((fns, unit.to_string())),
        id: None,
        out: String::new(),
    };
    // The `impl` a function is checked and proved in (backlog 2T-03): its
    // type parameters with their bounds, and for a trait's method or a
    // trait impl's function the trait and the type, which its contract and
    // refinement obligations depend on.
    if let Some(sig) = fns
        .values()
        .find(|s| s.span == f.span && !s.builtin && !s.path.ends_with('>'))
        && (sig.impl_generics > 0 || sig.path.starts_with('<'))
        && let Some((head, _)) = sig.path.rsplit_once("::")
    {
        let gs: Vec<String> = sig.generics[..sig.impl_generics]
            .iter()
            .map(|g| format!("{}: {}", g.name, g.bounds.join(" + ")))
            .collect();
        pr.out = format!("impl<{}> {head}\n", gs.join(", "));
    }
    pr.function(f, 0, Some("_"), with_body);
    pr.out
}

/// Identity hashes of every definition and type declaration, by path
/// (ADR-0059 §1: `bmb hash`).
pub fn hashes(p: &Program, t: &Tables) -> Vec<DefHash> {
    let ids = crate::identity::Ids::new(p, t);
    let mut out: Vec<DefHash> = crate::check::defs_of(p, &t.fns)
        .into_iter()
        .map(|(path, _, _)| crate::identity::Key::Def(path))
        .chain(type_paths(p).into_iter().map(crate::identity::Key::Type))
        .filter_map(|k| {
            let h = ids.get(&k)?;
            Some(DefHash {
                path: k.path().to_string(),
                id: h.id,
                body: h.body,
                iface: h.iface,
            })
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The full paths of the type declarations.
fn type_paths(p: &Program) -> Vec<String> {
    fn walk(m: &Mod, prefix: &str, out: &mut Vec<String>) {
        let mp = format!("{prefix}{}", m.name.name);
        for s in &m.mods {
            walk(s, &format!("{mp}::"), out);
        }
        out.extend(m.types.iter().map(|t| format!("{mp}::{}", t.name.name)));
    }
    let mut out = Vec::new();
    for m in &p.mods {
        walk(m, "", &mut out);
    }
    out
}

/// The qualified body hash of every definition and type declaration, by
/// path: what changed in a definition's own text, other definitions named
/// by path (ADR-0019 §2). A commit reports these (`workspace::Commit`).
pub fn qualified_hashes(p: &Program, fns: &BTreeMap<String, FnSig>) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = crate::check::defs_of(p, fns)
        .into_iter()
        .map(|(path, unit, f)| {
            let body = crate::sha256::hex(qualified_text(f, &unit, fns, true).as_bytes());
            (path, body)
        })
        .collect();
    out.extend(type_hashes(p));
    out
}

/// A trait declaration in normal form on one line (map L1 `signature`).
pub fn trait_text(t: &TraitDef) -> String {
    let mut pr = plain();
    pr.trait_def(t, 0);
    pr.out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A type declaration in normal form on one line (map L1 `signature`).
pub fn type_text(t: &TypeDef) -> String {
    let mut pr = plain();
    pr.type_def(t, 0);
    // An attribute (`#[invariant(p)]`, ADR-0060 §3) goes before the
    // declaration on the same line.
    pr.out.trim_end().replace('\n', " ")
}

/// Qualified hashes of the type declarations, by path. A type declaration
/// is its own surface, its name written `_` (ADR-0026 §6, as for
/// definitions, ADR-0019 §2).
fn type_hashes(p: &Program) -> Vec<(String, String)> {
    let mut out = Vec::new();
    fn types(m: &Mod, prefix: &str, out: &mut Vec<(String, String)>) {
        let mp = format!("{prefix}{}", m.name.name);
        for s in &m.mods {
            types(s, &format!("{mp}::"), out);
        }
        let local: BTreeSet<&str> = m.types.iter().map(|t| t.name.name.as_str()).collect();
        // Unit-local type names written in full (the qualified normal form).
        fn qualify(t: &Type, mp: &str, local: &BTreeSet<&str>) -> Type {
            let q = |x: &Type| Box::new(qualify(x, mp, local));
            let kind = match &t.kind {
                TypeKind::Named { path, args }
                    if path.len() == 1 && local.contains(path[0].as_str()) =>
                {
                    TypeKind::Named {
                        path: vec![mp.to_string(), path[0].clone()],
                        args: args.iter().map(|a| qualify(a, mp, local)).collect(),
                    }
                }
                TypeKind::Named { path, args } => TypeKind::Named {
                    path: path.clone(),
                    args: args.iter().map(|a| qualify(a, mp, local)).collect(),
                },
                TypeKind::Array(e, n) => TypeKind::Array(q(e), *n),
                TypeKind::Slice(e) => TypeKind::Slice(q(e)),
                TypeKind::Ref { mutable, inner } => TypeKind::Ref {
                    mutable: *mutable,
                    inner: q(inner),
                },
                k => k.clone(),
            };
            Type { kind, span: t.span }
        }
        for t in &m.types {
            let mut q = t.clone();
            match &mut q.kind {
                TypeDefKind::Struct(fs) => fs
                    .iter_mut()
                    .for_each(|f| f.ty = qualify(&f.ty, &mp, &local)),
                TypeDefKind::Enum(vs) => vs
                    .iter_mut()
                    .flat_map(|v| v.payload.iter_mut())
                    .for_each(|ty| *ty = qualify(ty, &mp, &local)),
            }
            let mut pr = plain();
            pr.type_def_as(&q, 0, "_");
            let h = crate::sha256::hex(pr.out.as_bytes());
            out.push((format!("{mp}::{}", t.name.name), h));
        }
    }
    for m in &p.mods {
        types(m, "", &mut out);
    }
    out
}

/// The keyword of a definition: `fn`, `spec fn` or `proof fn`.
fn fn_keyword(f: &FnDef) -> &'static str {
    match f.kind {
        FnKind::Exec => "fn",
        FnKind::Spec => "spec fn",
        FnKind::Proof => "proof fn",
    }
}

/// ` -> T` after the parameters; a `proof fn` writes none (ADR-0062 §1).
fn ret_text(f: &FnDef, ret: &str) -> String {
    if f.kind == FnKind::Proof {
        String::new()
    } else {
        format!(" -> {ret}")
    }
}

/// What a caller's query reads of an executable function (backlog 2D-22):
/// its qualified surface without `#[uses]`. A definition's `#[uses]` only
/// reveals spec fn bodies in its own queries (`verify::uses_of`), so the
/// incremental keys of its callers leave it out.
pub fn caller_text(f: &FnDef, unit: &str, fns: &BTreeMap<String, FnSig>) -> String {
    let mut g = f.clone();
    g.attrs.retain(|a| a.name.name != "uses");
    qualified_text(&g, unit, fns, false)
}

fn plain() -> Printer<'static> {
    Printer {
        qualify: None,
        id: None,
        out: String::new(),
    }
}

/// Normal form of one definition.
pub fn def_text(f: &FnDef) -> String {
    let mut pr = plain();
    pr.function(f, 0, None, true);
    pr.out
}

/// An expression in normal form on one line (the text of a fix).
pub fn expr_text(e: &Expr) -> String {
    plain().expr(e, 0)
}

/// A pattern in normal form.
pub fn pat_text(p: &Pat) -> String {
    pat_str(p)
}

/// Normal form of a definition without its body (its surface).
pub fn iface_text(f: &FnDef) -> String {
    let mut pr = plain();
    pr.function(f, 0, None, false);
    pr.out
}

/// One-line signature and contract clauses in normal form (map L1).
pub fn surface_parts(f: &FnDef) -> (String, Vec<String>, Vec<String>, Option<String>) {
    let mut pr = plain();
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.name, type_str(&p.ty)))
        .collect();
    let ret = match &f.ret.name {
        Some(n) => format!("({}: {})", n.name, type_str(&f.ret.ty)),
        None => type_str(&f.ret.ty),
    };
    let sig = format!(
        "{} {}{}({}){}",
        fn_keyword(f),
        f.name.name,
        generics_str(&f.generics),
        params.join(", "),
        ret_text(f, &ret)
    );
    let req = f.requires.iter().map(|e| pr.expr(e, 0)).collect();
    let ens = f.ensures.iter().map(|e| pr.expr(e, 0)).collect();
    let dec = f.decreases.as_ref().map(|e| pr.expr(e, 0));
    (sig, req, ens, dec)
}

fn pat_str(p: &Pat) -> String {
    let path = p
        .path
        .iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::");
    if !p.parens {
        return path;
    }
    let bs: Vec<&str> = p
        .binds
        .iter()
        .map(|b| b.as_ref().map_or("_", |i| i.name.as_str()))
        .collect();
    format!("{path}({})", bs.join(", "))
}

/// Whether a struct literal occurs in `e` outside a delimited argument list.
fn has_struct_lit(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::StructLit { .. } => true,
        ExprKind::Binary(_, l, r) => has_struct_lit(l) || has_struct_lit(r),
        ExprKind::Unary(_, x) | ExprKind::Ref { expr: x, .. } | ExprKind::Deref(x) => {
            has_struct_lit(x)
        }
        ExprKind::Field { base, .. } | ExprKind::Try(base) => has_struct_lit(base),
        ExprKind::Method { recv, .. } => has_struct_lit(recv),
        ExprKind::Index { base, .. } => has_struct_lit(base),
        _ => false,
    }
}
