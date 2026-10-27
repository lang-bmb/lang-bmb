//! Name resolution, type checking, second-class reference rules, specification
//! placement and effect checking for the milestone-1 subset.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::diag::{Diag, Diags};
use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Pure,
    Io,
}

/// A type parameter of a generic definition (ADR-0046): its name and its
/// bounds (`Copy` is built in).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TParam {
    pub name: String,
    pub bounds: Vec<String>,
}

/// `t` with each type parameter in `m` replaced by its type (simultaneous:
/// a replacement is not substituted again).
pub fn subst(t: &TypeKind, m: &BTreeMap<String, TypeKind>) -> TypeKind {
    if m.is_empty() {
        return t.clone();
    }
    let sub = |x: &Type| Type {
        kind: subst(&x.kind, m),
        span: x.span,
    };
    match t {
        TypeKind::Param(n) => m.get(n).cloned().unwrap_or_else(|| t.clone()),
        TypeKind::Array(e, n) => TypeKind::Array(Box::new(sub(e)), *n),
        TypeKind::Slice(e) => TypeKind::Slice(Box::new(sub(e))),
        TypeKind::Vec(e) => TypeKind::Vec(Box::new(sub(e))),
        TypeKind::Box(e) => TypeKind::Box(Box::new(sub(e))),
        TypeKind::Ref { mutable, inner } => TypeKind::Ref {
            mutable: *mutable,
            inner: Box::new(sub(inner)),
        },
        TypeKind::Named { path, args } => TypeKind::Named {
            path: path.clone(),
            args: args.iter().map(sub).collect(),
        },
        other => other.clone(),
    }
}

/// Whether `t` mentions a type parameter.
pub fn has_param(t: &TypeKind) -> bool {
    match t {
        TypeKind::Param(_) => true,
        TypeKind::Array(e, _) | TypeKind::Slice(e) | TypeKind::Vec(e) | TypeKind::Box(e) => {
            has_param(&e.kind)
        }
        TypeKind::Ref { inner, .. } => has_param(&inner.kind),
        TypeKind::Named { args, .. } => args.iter().any(|a| has_param(&a.kind)),
        _ => false,
    }
}

/// Binds the parameters `ps` occurring in `pat` by matching it against `t`
/// (first binding wins; a mismatch binds nothing there).
fn unify(pat: &TypeKind, t: &TypeKind, ps: &[TParam], out: &mut BTreeMap<String, TypeKind>) {
    match (pat, t) {
        (TypeKind::Param(n), _) if ps.iter().any(|p| p.name == *n) => {
            if *t != TypeKind::Error {
                out.entry(n.clone()).or_insert_with(|| t.clone());
            }
        }
        (TypeKind::Array(a, _), TypeKind::Array(b, _))
        | (TypeKind::Slice(a), TypeKind::Slice(b))
        | (TypeKind::Vec(a), TypeKind::Vec(b))
        | (TypeKind::Box(a), TypeKind::Box(b)) => unify(&a.kind, &b.kind, ps, out),
        (TypeKind::Ref { inner: a, .. }, TypeKind::Ref { inner: b, .. }) => {
            unify(&a.kind, &b.kind, ps, out)
        }
        // `&[T]` from `&[i64; 4]` or a `Vec<i64>` viewed as a slice.
        (TypeKind::Slice(a), TypeKind::Array(b, _) | TypeKind::Vec(b)) => {
            unify(&a.kind, &b.kind, ps, out)
        }
        (TypeKind::Named { path: p, args: a }, TypeKind::Named { path: q, args: b })
            if p == q && a.len() == b.len() =>
        {
            for (x, y) in a.iter().zip(b) {
                unify(&x.kind, &y.kind, ps, out);
            }
        }
        _ => {}
    }
}

/// The substitution of a generic data type's parameters by the arguments
/// of its instance `t` (empty for a non-generic type).
pub fn adt_subst(adt: &Adt, t: &TypeKind) -> BTreeMap<String, TypeKind> {
    match t {
        TypeKind::Named { args, .. } => adt
            .generics
            .iter()
            .zip(args)
            .map(|(p, a)| (p.name.clone(), a.kind.clone()))
            .collect(),
        _ => BTreeMap::new(),
    }
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub path: String,
    pub kind: FnKind,
    /// Type parameters (ADR-0046): an `impl`'s first, then the function's.
    pub generics: Vec<TParam>,
    /// How many of `generics` are the `impl`'s.
    pub impl_generics: usize,
    pub params: Vec<TypeKind>,
    pub ret: TypeKind,
    pub effect: Effect,
    /// A spec fn whose body is the literal `true`.
    pub vacuous: bool,
    pub builtin: bool,
    /// Defined in an `impl T` block: named `T::f` in its unit (ADR-0026),
    /// or in an `impl Trait for T` block: named `<T as Trait>::f`.
    pub in_impl: bool,
    /// A method of a trait declaration (`unit::Trait::f`): its first type
    /// parameter is `Self` (ADR-0046 §2).
    pub trait_of: Option<String>,
    /// A trait method with a default body (ADR-0046 §2, backlog 2T-07c):
    /// an impl that does not define it runs this one.
    pub default: bool,
    /// The definition's span (`defs` finds trait `impl` functions by it).
    pub span: Span,
}

/// A trait declaration (ADR-0046 §1): its methods by name, in declaration
/// order (each is the function `path::name`).
#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub path: String,
    pub methods: Vec<String>,
    pub span: Span,
}

/// An `impl Trait for T` block (ADR-0046 §2): the trait, the impl's type
/// parameters, the implementing type `head` (naming them), and its unit.
#[derive(Debug, Clone)]
pub struct TraitImplInfo {
    pub trait_path: String,
    pub generics: Vec<TParam>,
    pub head: TypeKind,
    pub unit: String,
    /// The `impl` block's span.
    pub span: Span,
}

impl TraitImplInfo {
    /// The path of the impl's function `name`: `<geo::Sq as geo::Shape>::area`.
    pub fn fn_path(&self, name: &str) -> String {
        format!("<{} as {}>::{name}", ty_str(&self.head), self.trait_path)
    }

    /// The substitution of the impl's parameters that makes `head` the type
    /// `t`, if it matches.
    pub fn matches(&self, t: &TypeKind) -> Option<BTreeMap<String, TypeKind>> {
        let mut m = BTreeMap::new();
        unify(&self.head, t, &self.generics, &mut m);
        (subst(&self.head, &m) == *t).then_some(m)
    }
}

/// The function a call of trait method `target` with type arguments `args`
/// (`Self` first) runs: the impl's function and its type arguments (the
/// impl's, then the method's own). `None` when `target` is no trait method
/// or `Self` is a type parameter (the trait's signature stands for it).
pub fn dispatch(
    fns: &BTreeMap<String, FnSig>,
    impls: &[TraitImplInfo],
    target: &str,
    args: &[TypeKind],
) -> Option<(String, Vec<TypeKind>)> {
    let tr = fns.get(target)?.trait_of.as_ref()?;
    let self_ty = args.first()?;
    let name = target.rsplit("::").next()?;
    let (i, m) = impls
        .iter()
        .filter(|i| &i.trait_path == tr)
        .find_map(|i| i.matches(self_ty).map(|m| (i, m)))?;
    let path = i.fn_path(name);
    if !fns.contains_key(&path) {
        // The impl leaves it out: the trait's default body runs, for the
        // impl's type (backlog 2T-07c).
        return fns[target]
            .default
            .then(|| (target.to_string(), args.to_vec()));
    }
    let mut out: Vec<TypeKind> = i
        .generics
        .iter()
        .map(|g| m.get(&g.name).cloned().unwrap_or(TypeKind::Error))
        .collect();
    out.extend(args[1..].iter().cloned());
    Some((path, out))
}

/// Every definition with a body, in walk order (in each unit, its
/// sub-units first, then its functions, `impl T` functions, trait `impl`
/// functions and default method bodies): (path, unit, syntax). A trait `impl` function has the path
/// the checker gave it (`<T as Trait>::f`); one it rejected is left out.
pub fn defs_of<'p>(
    p: &'p Program,
    fns: &BTreeMap<String, FnSig>,
) -> Vec<(String, String, &'p FnDef)> {
    let by_span: BTreeMap<(u32, u32, u32), &str> = fns
        .values()
        // Not an instance (`<..>::f<i64>`, the encoder's tables).
        .filter(|s| s.in_impl && s.path.starts_with('<') && !s.path.ends_with('>'))
        .map(|s| ((s.span.file, s.span.lo, s.span.hi), s.path.as_str()))
        .collect();
    fn walk<'p>(
        m: &'p Mod,
        prefix: &str,
        by_span: &BTreeMap<(u32, u32, u32), &str>,
        out: &mut Vec<(String, String, &'p FnDef)>,
    ) {
        let mp = format!("{prefix}{}", m.name.name);
        for s in &m.mods {
            walk(s, &format!("{mp}::"), by_span, out);
        }
        for f in &m.fns {
            out.push((format!("{mp}::{}", f.name.name), mp.clone(), f));
        }
        for i in &m.impls {
            for f in &i.fns {
                out.push((
                    format!("{mp}::{}::{}", i.ty.name, f.name.name),
                    mp.clone(),
                    f,
                ));
            }
        }
        for i in &m.trait_impls {
            for f in &i.fns {
                if let Some(path) = by_span.get(&(f.span.file, f.span.lo, f.span.hi)) {
                    out.push((path.to_string(), mp.clone(), f));
                }
            }
        }
        // Default method bodies (backlog 2T-07c), generic in `Self`.
        for t in &m.traits {
            for tf in t.fns.iter().filter(|tf| tf.has_body) {
                out.push((
                    format!("{mp}::{}::{}", t.name.name, tf.f.name.name),
                    mp.clone(),
                    &tf.f,
                ));
            }
        }
    }
    let mut out = Vec::new();
    for m in &p.mods {
        walk(m, "", &by_span, &mut out);
    }
    out
}

/// The definition `#[uses(name)]` names from unit `mp`: `f` or `Trait::f`
/// of the unit, else a full path (ADR-0046 §4: a trait's `spec fn` method
/// stands for its impls' definitions).
pub fn uses_target(name: &str, mp: &str, fns: &BTreeMap<String, FnSig>) -> Option<String> {
    let local = format!("{mp}::{name}");
    if fns.get(&local).is_some_and(|s| !s.builtin) {
        return Some(local);
    }
    fns.get(name)
        .filter(|s| !s.builtin)
        .map(|_| name.to_string())
}

/// The method declarations of every trait: (path `unit::Trait::f`, unit,
/// syntax).
pub fn trait_decls(p: &Program) -> Vec<(String, String, &FnDef)> {
    fn walk<'p>(m: &'p Mod, prefix: &str, out: &mut Vec<(String, String, &'p FnDef)>) {
        let mp = format!("{prefix}{}", m.name.name);
        for s in &m.mods {
            walk(s, &format!("{mp}::"), out);
        }
        for t in &m.traits {
            for tf in &t.fns {
                out.push((
                    format!("{mp}::{}::{}", t.name.name, tf.f.name.name),
                    mp.clone(),
                    &tf.f,
                ));
            }
        }
    }
    let mut out = Vec::new();
    for m in &p.mods {
        walk(m, "", &mut out);
    }
    out
}

/// A declared data type (ADR-0026), its field and payload types resolved
/// (`TypeKind::Named` paths are full paths).
#[derive(Debug, Clone)]
pub enum AdtKind {
    Struct(Vec<(String, TypeKind)>),
    Enum(Vec<VariantSig>),
}

/// An enum variant: its name and payload types.
pub type VariantSig = (String, Vec<TypeKind>);

#[derive(Debug, Clone)]
pub struct Adt {
    pub path: String,
    /// Type parameters (ADR-0046); field and payload types name them as
    /// `TypeKind::Param`.
    pub generics: Vec<TParam>,
    pub kind: AdtKind,
    pub span: Span,
}

/// Names the language defines (ADR-0026): user definitions cannot take them.
const BUILTIN_NAMES: &[&str] = &[
    "Option", "Result", "Some", "None", "Ok", "Err", "Vec", "Box",
];

pub struct Checker<'a> {
    diags: &'a mut Diags,
    pub fns: BTreeMap<String, FnSig>,
    /// Declared structs and enums by full path (backlog 2B-04).
    pub adts: BTreeMap<String, Adt>,
    /// Per unit, the names its `use` items import and their full paths
    /// (backlog 2V-15: rewritten by E0214).
    imports: BTreeMap<String, BTreeMap<String, String>>,
    /// Exec-to-exec and spec-to-spec call edges, by path.
    calls: BTreeMap<String, BTreeSet<String>>,
    /// Type of every checked expression, keyed by its span `(file, lo, hi)`.
    pub types: BTreeMap<(u32, u32, u32), TypeKind>,
    /// Recursive definitions: path -> id of its strongly connected component.
    pub recursive: BTreeMap<String, usize>,
    /// Canonical type of every parameter and local, by definition path.
    pub locals: BTreeMap<String, BTreeMap<String, TypeKind>>,
    /// Where each executable function frees its `Vec` values (ADR-0031 §2).
    pub moves: BTreeMap<String, crate::moves::MoveInfo>,
    /// The type arguments of every call of a generic function, by the
    /// call's span: the `impl`'s, then the function's (ADR-0046).
    pub targs: BTreeMap<(u32, u32, u32), Vec<TypeKind>>,
    /// Trait declarations by full path (ADR-0046 §1).
    pub traits: BTreeMap<String, TraitInfo>,
    /// `impl Trait for T` blocks the checker accepted (ADR-0046 §2).
    pub trait_impls: Vec<TraitImplInfo>,
    /// The type `Self` stands for in the trait `impl` being read, and its
    /// text (E0305 on `Self` writes it, and checking goes on with it).
    self_head: Option<(TypeKind, String)>,
    /// The type parameters in scope where types are resolved.
    tparams: Vec<TParam>,
    /// Calls of generic functions: caller, callee, span, type arguments
    /// (E0430 needs the recursive ones).
    gcalls: Vec<(String, String, Span, Vec<TypeKind>)>,
    /// Every field type is resolved, so which types move is known
    /// (`is_move`); fields themselves are checked by `check_owned_fields`.
    owned_ready: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Exec,
    /// Specification context. `old` is allowed only in `ensures`.
    Spec {
        allow_old: bool,
    },
}

#[derive(Clone)]
struct Local {
    ty: TypeKind,
    mutable: bool,
    is_param: bool,
}

struct FnCtx {
    path: String,
    mod_path: String,
    effect: Effect,
    scopes: Vec<BTreeMap<String, Local>>,
    all_names: BTreeSet<String>,
    forall_depth: u32,
}

pub fn ty_str(t: &TypeKind) -> String {
    match t {
        TypeKind::I64 => "i64".into(),
        TypeKind::U64 => "u64".into(),
        TypeKind::U8 => "u8".into(),
        TypeKind::Usize => "usize".into(),
        TypeKind::Bool => "bool".into(),
        TypeKind::Unit => "()".into(),
        TypeKind::Array(e, n) => format!("[{}; {n}]", ty_str(&e.kind)),
        TypeKind::Slice(e) => format!("[{}]", ty_str(&e.kind)),
        TypeKind::Ref { mutable, inner } => format!(
            "&{}{}",
            if *mutable { "mut " } else { "" },
            ty_str(&inner.kind)
        ),
        TypeKind::Named { path, args } if args.is_empty() => path.join("::"),
        TypeKind::Named { path, args } => format!(
            "{}<{}>",
            path.join("::"),
            args.iter()
                .map(|a| ty_str(&a.kind))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeKind::Vec(e) => format!("Vec<{}>", ty_str(&e.kind)),
        TypeKind::Box(e) => format!("Box<{}>", ty_str(&e.kind)),
        TypeKind::Param(n) => n.clone(),
        TypeKind::Error => "{error}".into(),
    }
}

/// The type `ty_str` printed (`ty_str(&parse_ty(&ty_str(t))?) ==
/// ty_str(t)`), for stored results (the incremental IR cache, backlog
/// 2T-04). A one-segment name other than `Option` and `Result` is a type
/// parameter.
pub fn parse_ty(s: &str) -> Option<TypeKind> {
    fn ty(s: &str) -> Option<TypeKind> {
        let s = s.trim();
        let t = |k: TypeKind| Type {
            kind: k,
            span: Span::default(),
        };
        Some(match s {
            "i64" => TypeKind::I64,
            "u64" => TypeKind::U64,
            "u8" => TypeKind::U8,
            "usize" => TypeKind::Usize,
            "bool" => TypeKind::Bool,
            "()" => TypeKind::Unit,
            _ if s.starts_with("&mut ") => TypeKind::Ref {
                mutable: true,
                inner: Box::new(t(ty(&s[5..])?)),
            },
            _ if s.starts_with('&') => TypeKind::Ref {
                mutable: false,
                inner: Box::new(t(ty(&s[1..])?)),
            },
            _ if s.starts_with('[') && s.ends_with(']') => {
                let inner = &s[1..s.len() - 1];
                let parts = split_top(inner, ';');
                match parts.as_slice() {
                    [e] => TypeKind::Slice(Box::new(t(ty(e)?))),
                    [e, n] => TypeKind::Array(Box::new(t(ty(e)?)), n.trim().parse().ok()?),
                    _ => return None,
                }
            }
            _ => {
                let (head, args) = match s.find('<') {
                    Some(i) if s.ends_with('>') => (&s[..i], &s[i + 1..s.len() - 1]),
                    Some(_) => return None,
                    None => (s, ""),
                };
                let args: Vec<Type> = if args.is_empty() {
                    Vec::new()
                } else {
                    split_top(args, ',')
                        .iter()
                        .map(|a| ty(a).map(t))
                        .collect::<Option<_>>()?
                };
                let path: Vec<String> = head.split("::").map(String::from).collect();
                if path
                    .iter()
                    .any(|p| !crate::parser::is_ident(p) && p != "Self")
                {
                    return None;
                }
                match (path.as_slice(), args.len()) {
                    ([v], 1) if v == "Vec" => {
                        TypeKind::Vec(Box::new(args.into_iter().next().expect("one")))
                    }
                    ([v], 1) if v == "Box" => {
                        TypeKind::Box(Box::new(args.into_iter().next().expect("one")))
                    }
                    ([n], 0) if n != "Option" && n != "Result" => TypeKind::Param(n.clone()),
                    _ => TypeKind::Named { path, args },
                }
            }
        })
    }
    /// `s` split at `sep` outside brackets.
    fn split_top(s: &str, sep: char) -> Vec<&str> {
        let (mut depth, mut start, mut out) = (0i32, 0usize, Vec::new());
        for (i, c) in s.char_indices() {
            match c {
                '<' | '[' | '(' => depth += 1,
                '>' | ']' | ')' => depth -= 1,
                c if c == sep && depth == 0 => {
                    out.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        out.push(&s[start..]);
        out
    }
    ty(s)
}

fn is_int(t: &TypeKind) -> bool {
    matches!(
        t,
        TypeKind::I64 | TypeKind::U64 | TypeKind::Usize | TypeKind::U8
    )
}
fn is_scalar(t: &TypeKind) -> bool {
    is_int(t) || matches!(t, TypeKind::Bool)
}

/// Whether values of `t` move (ADR-0047 §1): a `Vec`, a `Box`, and the
/// structs, enums and arrays that hold one through fields, payloads,
/// elements or type arguments. The classification is structural (no
/// `Copy` derive); type parameters carry `Copy` until backlog 2O-04.
pub fn is_move(t: &TypeKind, adts: &BTreeMap<String, Adt>) -> bool {
    fn go(t: &TypeKind, adts: &BTreeMap<String, Adt>, seen: &mut BTreeSet<String>) -> bool {
        match t {
            TypeKind::Vec(_) | TypeKind::Box(_) => true,
            TypeKind::Array(e, _) => go(&e.kind, adts, seen),
            TypeKind::Named { path, args } => {
                if args.iter().any(|a| go(&a.kind, adts, seen)) {
                    return true;
                }
                let p = path.join("::");
                if !seen.insert(p.clone()) {
                    return false;
                }
                match adts.get(&p).map(|a| &a.kind) {
                    Some(AdtKind::Struct(fs)) => fs.iter().any(|(_, ft)| go(ft, adts, seen)),
                    Some(AdtKind::Enum(vs)) => vs
                        .iter()
                        .any(|(_, ps)| ps.iter().any(|pt| go(pt, adts, seen))),
                    None => false,
                }
            }
            _ => false,
        }
    }
    go(t, adts, &mut BTreeSet::new())
}

/// The text of a type parameter with its bounds as written.
fn self_text_of(g: &TypeParam) -> String {
    let bounds: Vec<String> = g
        .bounds
        .iter()
        .map(|b| {
            b.iter()
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>()
                .join("::")
        })
        .collect();
    format!("{}: {}", g.name.name, bounds.join(" + "))
}
fn elem_of(t: &TypeKind) -> Option<&TypeKind> {
    match t {
        TypeKind::Array(e, _) | TypeKind::Vec(e) => Some(&e.kind),
        TypeKind::Ref { inner, .. } => match &inner.kind {
            TypeKind::Slice(e) | TypeKind::Vec(e) => Some(&e.kind),
            _ => None,
        },
        _ => None,
    }
}

/// The element type of a `Vec<T>`, `&Vec<T>` or `&mut Vec<T>`.
fn vec_elem(t: &TypeKind) -> Option<&TypeKind> {
    match t {
        TypeKind::Vec(e) => Some(&e.kind),
        TypeKind::Ref { inner, .. } => match &inner.kind {
            TypeKind::Vec(e) => Some(&e.kind),
            _ => None,
        },
        _ => None,
    }
}
fn mk(kind: TypeKind) -> Box<Type> {
    Box::new(Type {
        kind,
        span: Span::default(),
    })
}

/// The help of E0402 for an array inside an array element (backlog 2B-14).
const ARRAY_ELEMENT_HELP: &str = "an array element is a scalar, a struct or enum, or a fixed-size array; a growable element (`Vec`) is ADR-0047 (proposed)";

/// Whether `e` may be the element of a fixed-size array (backlog 2V-04b):
/// a scalar, a struct or enum, or such an array again; not a `Vec`.
fn array_elem_ok(e: &TypeKind) -> bool {
    match e {
        TypeKind::Named { path, .. } => !(path.len() == 1 && path[0] == "Vec"),
        TypeKind::Array(inner, _) => array_elem_ok(&inner.kind),
        TypeKind::Error => true,
        t => is_scalar(t),
    }
}

impl<'a> Checker<'a> {
    pub fn new(diags: &'a mut Diags) -> Self {
        let mut fns = BTreeMap::new();
        fns.insert(
            "print_i64".to_string(),
            FnSig {
                path: "print_i64".into(),
                kind: FnKind::Exec,
                params: vec![TypeKind::I64],
                ret: TypeKind::Unit,
                effect: Effect::Io,
                vacuous: false,
                builtin: true,
                in_impl: false,
                trait_of: None,
                default: false,
                span: Span::default(),
                generics: Vec::new(),
                impl_generics: 0,
            },
        );
        fns.insert(
            "print_bytes".to_string(),
            FnSig {
                path: "print_bytes".into(),
                kind: FnKind::Exec,
                params: vec![TypeKind::Ref {
                    mutable: false,
                    inner: mk(TypeKind::Slice(mk(TypeKind::U8))),
                }],
                ret: TypeKind::Unit,
                effect: Effect::Io,
                vacuous: false,
                builtin: true,
                in_impl: false,
                trait_of: None,
                default: false,
                span: Span::default(),
                generics: Vec::new(),
                impl_generics: 0,
            },
        );
        fns.insert(
            "print_u64".to_string(),
            FnSig {
                path: "print_u64".into(),
                kind: FnKind::Exec,
                params: vec![TypeKind::U64],
                ret: TypeKind::Unit,
                effect: Effect::Io,
                vacuous: false,
                builtin: true,
                in_impl: false,
                trait_of: None,
                default: false,
                span: Span::default(),
                generics: Vec::new(),
                impl_generics: 0,
            },
        );
        Checker {
            diags,
            fns,
            adts: BTreeMap::new(),
            imports: BTreeMap::new(),
            calls: BTreeMap::new(),
            types: BTreeMap::new(),
            recursive: BTreeMap::new(),
            locals: BTreeMap::new(),
            moves: BTreeMap::new(),
            targs: BTreeMap::new(),
            traits: BTreeMap::new(),
            trait_impls: Vec::new(),
            self_head: None,
            tparams: Vec::new(),
            gcalls: Vec::new(),
            owned_ready: false,
        }
    }

    /// Hands the tables the proof gate needs to the verifier.
    pub fn into_tables(self) -> crate::verify::Tables {
        crate::verify::Tables {
            fns: self.fns,
            types: self.types,
            recursive: self.recursive,
            adts: self.adts,
            locals: self.locals,
            moves: self.moves,
            targs: self.targs,
            trait_impls: self.trait_impls,
            insts: BTreeMap::new(),
            inst_types: Vec::new(),
        }
    }

    fn err(&mut self, code: &'static str, span: Span, msg: impl Into<String>) {
        self.diags.push(Diag::new(code, span, msg));
    }

    pub fn check_program(&mut self, p: &Program) {
        // Pass 1: collect signatures.
        let mut seen = BTreeMap::new();
        for m in &p.mods {
            if let Some(prev) = seen.insert(m.name.name.clone(), m.name.span) {
                let _ = prev;
                self.err(
                    "E0302",
                    m.name.span,
                    format!("unit `{}` is defined more than once", m.name.name),
                );
            }
        }
        // Trait names, then types: bounds, signatures and fields may name
        // any unit's traits and types.
        for m in &p.mods {
            self.collect_trait_names(m, "");
        }
        for m in &p.mods {
            self.collect_types(m, "");
        }
        // Which types implement which traits, before any bound is checked.
        for m in &p.mods {
            self.collect_impl_heads(m, "");
        }
        for m in &p.mods {
            self.resolve_types(m, "");
        }
        self.owned_ready = true;
        for m in &p.mods {
            self.check_owned_fields(m, "");
        }
        self.check_type_cycles();
        for m in &p.mods {
            self.collect_trait_sigs(m, "");
        }
        for m in &p.mods {
            self.collect_mod(m, "");
        }
        // Pass 2: check bodies.
        for m in &p.mods {
            self.check_mod(m, "");
        }
        self.check_recursion(p);
        self.check_main(p);
    }

    /// Registers the unit's trait names (ADR-0046 §1; their methods come
    /// after the types).
    fn collect_trait_names(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        for sub in &m.mods {
            self.collect_trait_names(sub, &format!("{mp}::"));
        }
        for t in &m.traits {
            if matches!(t.name.name.as_str(), "Copy" | "Eq" | "Ord") {
                self.diags.push(
                    Diag::new(
                        "E0309",
                        t.name.span,
                        format!("`{}` is a built-in trait (ADR-0046 §1)", t.name.name),
                    )
                    .fix("choose another name", None),
                );
            }
            for a in &t.attrs {
                self.diags.push(
                    Diag::new(
                        "E0605",
                        a.span,
                        format!("unknown attribute `{}` on a trait", a.name.name),
                    )
                    .fix("traits take no attributes", Some(String::new())),
                );
            }
            let path = format!("{mp}::{}", t.name.name);
            // A second declaration is reported by `collect_mod`.
            self.traits.entry(path.clone()).or_insert(TraitInfo {
                path,
                methods: Vec::new(),
                span: t.name.span,
            });
        }
    }

    /// The implementing types of the unit's `impl Trait for T` blocks
    /// (ADR-0046 §2): no blanket impl (E0235), every impl parameter in the
    /// type, the impl in the unit of the trait or of the type (E0312), no
    /// two impls of a trait for types that can be the same (E0311,
    /// conservatively: a type parameter matches anything).
    fn collect_impl_heads(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        for sub in &m.mods {
            self.collect_impl_heads(sub, &format!("{mp}::"));
        }
        for i in &m.trait_impls {
            let name = Self::path_text(&i.trait_path);
            let tspan = i.trait_path[0]
                .span
                .to(i.trait_path[i.trait_path.len() - 1].span);
            let Some(tr) = self.trait_path(&i.trait_path, &mp) else {
                let help = if self.type_path(&i.trait_path, &mp).is_some() {
                    format!("`{name}` is a type; its own functions go in `impl {name} {{ .. }}`")
                } else {
                    "a trait of another unit is named by full path, e.g. `unit::Shape`".into()
                };
                let d = self.unknown_trait(&name, tspan, &mp, help);
                self.diags.push(d);
                continue;
            };
            let gens = self.tparams_of(&i.generics, true, &mp);
            self.tparams = gens.clone();
            let head = self.canon(&i.self_ty, &mp, true);
            self.tparams.clear();
            let span = i.self_ty.span;
            if let TypeKind::Param(n) = &head {
                self.diags.push(
                    Diag::new(
                        "E0235",
                        span,
                        format!(
                            "`impl<{n}> {name} for {n}` implements `{name}` for every type (a blanket impl)"
                        ),
                    )
                    .fix(
                        format!("implement it for each type: `impl {name} for Point {{ .. }}`, or bound a type parameter `T: {name}` where it is used"),
                        None,
                    ),
                );
                continue;
            }
            if head == TypeKind::Error {
                continue;
            }
            let copy = TParam {
                name: "Self".into(),
                bounds: vec!["Copy".into()],
            };
            if !self.targ_ok(&head, &copy, &tr, span, true) {
                continue;
            }
            if let Some((g, p)) = i
                .generics
                .iter()
                .zip(&gens)
                .find(|(_, p)| !mentions_param(&head, &p.name))
            {
                self.diags.push(
                    Diag::new(
                        "E0420",
                        g.span,
                        format!(
                            "type parameter `{}` of the impl does not appear in `{}`",
                            p.name,
                            ty_str_in(&head, &mp)
                        ),
                    )
                    .fix("remove it, or use it in the implemented type", None),
                );
                continue;
            }
            let trait_unit = tr.rsplit_once("::").map_or("", |x| x.0).to_string();
            let type_unit = match &head {
                TypeKind::Named { path, .. } if self.adts.contains_key(&path.join("::")) => {
                    Some(path[..path.len() - 1].join("::"))
                }
                _ => None,
            };
            let ty = ty_str_in(&head, &mp);
            if mp != trait_unit && type_unit.as_deref() != Some(mp.as_str()) {
                let help = match &type_unit {
                    Some(u) if *u != trait_unit => {
                        format!("move it to unit `{trait_unit}` or unit `{u}`")
                    }
                    _ => format!("move it to unit `{trait_unit}`"),
                };
                self.diags.push(
                    Diag::new(
                        "E0312",
                        span,
                        format!(
                            "`impl {name} for {ty}` is in neither the unit of the trait nor the unit of the type"
                        ),
                    )
                    .fix(help, None),
                );
                continue;
            }
            if let Some(prev) = self
                .trait_impls
                .iter()
                .find(|p| p.trait_path == tr && overlaps(&p.head, &head))
            {
                let prev_ty = ty_str_in(&prev.head, &mp);
                self.diags.push(
                    Diag::new(
                        "E0311",
                        span,
                        format!("`impl {name} for {ty}` overlaps `impl {name} for {prev_ty}`"),
                    )
                    .fix(
                        "a trait has one impl for a type: remove one of them, or make their types differ",
                        None,
                    ),
                );
                continue;
            }
            self.trait_impls.push(TraitImplInfo {
                trait_path: tr,
                generics: gens,
                head,
                unit: mp.clone(),
                span: i.span,
            });
        }
    }

    /// The method signatures of the unit's traits (ADR-0046 §2): generic in
    /// `Self` (bounded by the trait), then their own parameters. Their
    /// contracts and default bodies are checked with the other bodies
    /// (`check_fn`); a `decreases` is E0207.
    fn collect_trait_sigs(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        for sub in &m.mods {
            self.collect_trait_sigs(sub, &format!("{mp}::"));
        }
        for t in &m.traits {
            let path = format!("{mp}::{}", t.name.name);
            if self.traits.get(&path).is_none_or(|x| x.span != t.name.span) {
                continue;
            }
            let selfp = TParam {
                name: "Self".into(),
                bounds: vec!["Copy".into(), path.clone()],
            };
            let mut names: Vec<String> = Vec::new();
            for tf in &t.fns {
                let f = &tf.f;
                if names.contains(&f.name.name) {
                    self.err(
                        "E0302",
                        f.name.span,
                        format!(
                            "`{}` is defined more than once in trait `{}`",
                            f.name.name, t.name.name
                        ),
                    );
                    continue;
                }
                names.push(f.name.name.clone());
                // A call through a bound carries no measure (ADR-0046 §4):
                // recursion through it is rejected (`trait_cycles`).
                if let Some(d) = &f.decreases {
                    self.diags.push(
                        Diag::new(
                            "E0207",
                            d.span,
                            "a trait method takes no `decreases`: recursion through a bound is not supported",
                        )
                        .fix("remove it; an impl's function may recurse on its own type with its own `decreases`", None),
                    );
                }
                let own = self.tparams_of(&f.generics, false, &mp);
                self.tparams = vec![selfp.clone()];
                self.tparams.extend(own);
                self.receiver_rule(f, Some("Self"), &mp);
                self.tparams.clear();
                self.collect_fn(
                    f,
                    format!("{path}::{}", f.name.name),
                    &mp,
                    false,
                    std::slice::from_ref(&selfp),
                );
                if let Some(sig) = self.fns.get_mut(&format!("{path}::{}", f.name.name)) {
                    sig.trait_of = Some(path.clone());
                    sig.default = tf.has_body;
                }
            }
            if let Some(x) = self.traits.get_mut(&path) {
                x.methods = names;
            }
        }
    }

    /// The functions of an `impl Trait for T` block against the trait's
    /// methods (ADR-0046 §2): each one defined (E0315), with the trait's
    /// signature for `Self = T` (E0316). That its contract refines the
    /// trait's (E0817) is a proof obligation (`verify`).
    fn conform(&mut self, info: &TraitImplInfo, ti: &TraitImpl, mp: &str) {
        let tr = &info.trait_path;
        let name = Self::path_text(&ti.trait_path);
        let head = ty_str_in(&info.head, mp);
        let methods = self.traits[tr].methods.clone();
        let missing: Vec<&str> = methods
            .iter()
            .filter(|m| !ti.fns.iter().any(|f| f.name.name == **m))
            .filter(|m| {
                !self
                    .fns
                    .get(&format!("{tr}::{m}"))
                    .is_some_and(|s| s.default)
            })
            .map(|m| m.as_str())
            .collect();
        if !missing.is_empty() {
            self.diags.push(
                Diag::new(
                    "E0315",
                    ti.self_ty.span,
                    format!(
                        "`impl {name} for {head}` does not define `{}`",
                        missing.join("`, `")
                    ),
                )
                .fix(
                    "define every method of the trait, with the trait's signature for `Self`",
                    None,
                ),
            );
        }
        for f in &ti.fns {
            if !methods.contains(&f.name.name) {
                self.diags.push(
                    Diag::new(
                        "E0315",
                        f.name.span,
                        format!("`{}` is not a method of `{name}`", f.name.name),
                    )
                    .fix(
                        format!("remove it, or declare it in `{name}`; a function of the type alone goes in `impl {head} {{ .. }}`"),
                        None,
                    ),
                );
                continue;
            }
            let Some(tsig) = self.fns.get(&format!("{tr}::{}", f.name.name)).cloned() else {
                continue;
            };
            let Some(isig) = self.fns.get(&info.fn_path(&f.name.name)).cloned() else {
                continue;
            };
            let town = &tsig.generics[tsig.impl_generics..];
            let iown = &isig.generics[isig.impl_generics..];
            let mut m = BTreeMap::from([("Self".to_string(), info.head.clone())]);
            for (a, b) in town.iter().zip(iown) {
                m.insert(a.name.clone(), TypeKind::Param(b.name.clone()));
            }
            let want: Vec<TypeKind> = tsig.params.iter().map(|t| subst(t, &m)).collect();
            let want_ret = subst(&tsig.ret, &m);
            let same_bounds = town.len() == iown.len()
                && town.iter().zip(iown).all(|(a, b)| {
                    let x: BTreeSet<&String> = a.bounds.iter().collect();
                    let y: BTreeSet<&String> = b.bounds.iter().collect();
                    x == y
                });
            let error = |t: &TypeKind| *t == TypeKind::Error;
            let known = !want.iter().chain(&isig.params).any(error)
                && !error(&want_ret)
                && !error(&isig.ret);
            if known
                && (tsig.kind != isig.kind
                    || !same_bounds
                    || want != isig.params
                    || want_ret != isig.ret)
            {
                let gs = if town.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        town.iter()
                            .map(|g| format!("{}: {}", g.name, g.bounds.join(" + ")))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                let kw = if tsig.kind == FnKind::Spec {
                    "spec fn"
                } else {
                    "fn"
                };
                let ps: Vec<String> = want
                    .iter()
                    .enumerate()
                    .map(|(k, t)| {
                        let n = match f.params.get(k) {
                            Some(p) if f.params.len() == want.len() => p.name.name.as_str(),
                            _ => "_",
                        };
                        format!("{n}: {}", ty_str_in(t, mp))
                    })
                    .collect();
                self.diags.push(
                    Diag::new(
                        "E0316",
                        f.name.span,
                        format!(
                            "`{}` does not match its declaration in `{name}`",
                            f.name.name
                        ),
                    )
                    .fix(
                        format!(
                            "for `Self = {head}` the signature is `{kw} {}{gs}({}) -> {}`",
                            f.name.name,
                            ps.join(", "),
                            ty_str_in(&want_ret, mp)
                        ),
                        None,
                    ),
                );
            }
            if tsig.effect == Effect::Pure && isig.effect == Effect::Io {
                self.diags.push(
                    Diag::new(
                        "E0316",
                        f.name.span,
                        format!(
                            "`{}` has effect `io`, but `{name}` declares it `pure`",
                            f.name.name
                        ),
                    )
                    .fix(
                        "make the function pure, or declare the trait method `#[effects(io)]`",
                        None,
                    ),
                );
            }
        }
    }

    /// Registers the unit's struct and enum names (their fields come next).
    fn collect_types(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        for sub in &m.mods {
            self.collect_types(sub, &format!("{mp}::"));
        }
        for t in &m.types {
            let path = format!("{mp}::{}", t.name.name);
            if self.adts.contains_key(&path) {
                continue; // reported as a duplicate name by `collect_mod`
            }
            let kind = match t.kind {
                TypeDefKind::Struct(_) => AdtKind::Struct(Vec::new()),
                TypeDefKind::Enum(_) => AdtKind::Enum(Vec::new()),
            };
            let generics = self.tparams_of(&t.generics, false, &mp);
            self.adts.insert(
                path.clone(),
                Adt {
                    path,
                    generics,
                    kind,
                    span: t.name.span,
                },
            );
            for a in &t.attrs {
                let d = if a.name.name == "derive" {
                    // Rust habit: copying and specification `==` are built in.
                    Diag::new("E0220", a.span, "`#[derive(..)]` is not used").fix(
                        "remove it: data types copy by value and `==` is built in for specifications",
                        Some(String::new()),
                    )
                } else {
                    Diag::new(
                        "E0605",
                        a.span,
                        format!("unknown attribute `{}` on a type", a.name.name),
                    )
                    .fix("types take no attributes", Some(String::new()))
                };
                self.diags.push(d);
            }
        }
    }

    /// Resolves field and payload types and checks what a field may hold.
    fn resolve_types(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        for sub in &m.mods {
            self.resolve_types(sub, &format!("{mp}::"));
        }
        let mut done = BTreeSet::new();
        for t in &m.types {
            let path = format!("{mp}::{}", t.name.name);
            if !done.insert(path.clone()) {
                continue;
            }
            self.tparams = self.tparams_of(&t.generics, true, &mp);
            let kind = match &t.kind {
                TypeDefKind::Struct(fields) => {
                    if fields.is_empty() {
                        self.err("E0420", t.name.span, "a struct needs at least one field");
                    }
                    let mut seen = BTreeSet::new();
                    let mut out = Vec::new();
                    for f in fields {
                        if !seen.insert(f.name.name.clone()) {
                            self.err(
                                "E0302",
                                f.name.span,
                                format!(
                                    "field `{}` is defined more than once in `{path}`",
                                    f.name.name
                                ),
                            );
                        }
                        let ty = self.field_type(&f.ty, &mp);
                        out.push((f.name.name.clone(), ty));
                    }
                    AdtKind::Struct(out)
                }
                TypeDefKind::Enum(variants) => {
                    if variants.is_empty() {
                        self.err("E0420", t.name.span, "an enum needs at least one variant");
                    }
                    let mut seen = BTreeSet::new();
                    let mut out = Vec::new();
                    for v in variants {
                        if !seen.insert(v.name.name.clone()) {
                            self.err(
                                "E0302",
                                v.name.span,
                                format!(
                                    "variant `{}` is defined more than once in `{path}`",
                                    v.name.name
                                ),
                            );
                        }
                        let tys = v
                            .payload
                            .iter()
                            .map(|ty| self.field_type(ty, &mp))
                            .collect();
                        out.push((v.name.name.clone(), tys));
                    }
                    AdtKind::Enum(out)
                }
            };
            self.tparams.clear();
            if let Some(a) = self.adts.get_mut(&path) {
                a.kind = kind;
            }
        }
    }

    /// The type parameters `gs` (ADR-0046). With `report`, their problems
    /// are diagnosed (once per definition): an unknown bound, a duplicate
    /// name, and a parameter without `Copy` (a parameter that may be an
    /// owned type comes with ADR-0047, phase 2O).
    fn tparams_of(&mut self, gs: &[TypeParam], report: bool, mp: &str) -> Vec<TParam> {
        let mut out: Vec<TParam> = Vec::new();
        for g in gs {
            let mut bounds = Vec::new();
            for b in &g.bounds {
                let name = b
                    .iter()
                    .map(|i| i.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                if matches!(name.as_str(), "Copy" | "Eq" | "Ord") {
                    bounds.push(name);
                } else if let Some(tr) = self.trait_path(b, mp) {
                    bounds.push(tr);
                } else if report {
                    let span = b[0].span.to(b[b.len() - 1].span);
                    let d = self.unknown_trait(&name, span, mp, "the built-in bounds are `Copy`, `Eq` and `Ord`; a trait of another unit is named by full path, e.g. `unit::Shape`".into());
                    self.diags.push(d);
                }
            }
            if report {
                if out.iter().any(|p| p.name == g.name.name) {
                    self.diags.push(
                        Diag::new(
                            "E0302",
                            g.name.span,
                            format!("type parameter `{}` is declared twice", g.name.name),
                        )
                        .fix("give each type parameter its own name", None),
                    );
                }
                if !bounds.iter().any(|b| b == "Copy") {
                    let r = if g.bounds.is_empty() {
                        format!("{}: Copy", g.name.name)
                    } else {
                        format!("{} + Copy", self_text_of(g))
                    };
                    self.diags.push(
                        Diag::new(
                            "E0402",
                            g.span,
                            format!(
                                "type parameter `{}` needs `Copy`: a parameter that may be an owned type comes with ADR-0047 (phase 2O)",
                                g.name.name
                            ),
                        )
                        .fix(format!("write `{r}`"), Some(r)),
                    );
                }
            }
            if !bounds.iter().any(|b| b == "Copy") {
                bounds.push("Copy".into());
            }
            out.push(TParam {
                name: g.name.name.clone(),
                bounds,
            });
        }
        out
    }

    /// E0305 for trait `name`: the nearest trait of the unit, else `help`.
    fn unknown_trait(&self, name: &str, span: Span, mp: &str, help: String) -> Diag {
        let local: Vec<String> = self
            .traits
            .keys()
            .filter_map(|k| k.strip_prefix(&format!("{mp}::")).map(String::from))
            .filter(|k| !k.contains("::"))
            .collect();
        let d = Diag::new("E0305", span, format!("unknown trait `{name}`"));
        match closest(name, local.iter()) {
            Some(near) => d.fix(format!("did you mean `{near}`?"), Some(near)),
            None => d.fix(help, None),
        }
    }

    /// The full path of the trait named by `segs` from unit `mp`.
    fn trait_path(&self, segs: &[Ident], mp: &str) -> Option<String> {
        let name = Self::path_text(segs);
        let local = format!("{mp}::{name}");
        if segs.len() == 1 && self.traits.contains_key(&local) {
            return Some(local);
        }
        self.traits.contains_key(&name).then_some(name)
    }

    /// Whether `t` implements the user trait `tr` (ADR-0046 §2): a type
    /// parameter bounded by it, or a type an `impl` of it matches (with the
    /// impl's own bounds met).
    fn has_trait(&self, t: &TypeKind, tr: &str) -> bool {
        if let TypeKind::Param(n) = t {
            return self
                .tparams
                .iter()
                .any(|p| p.name == *n && p.bounds.iter().any(|b| b == tr));
        }
        self.trait_impls
            .iter()
            .filter(|i| i.trait_path == tr)
            .any(|i| {
                i.matches(t).is_some_and(|m| {
                    i.generics
                        .iter()
                        .all(|g| m.get(&g.name).is_some_and(|a| self.meets(a, &g.bounds)))
                })
            })
    }

    /// Whether `t` meets every bound in `bounds` (no diagnostics).
    fn meets(&self, t: &TypeKind, bounds: &[String]) -> bool {
        bounds.iter().all(|b| match b.as_str() {
            "Copy" => self.is_copy(t),
            "Eq" | "Ord" => self.has_builtin(t, b),
            tr => self.has_trait(t, tr),
        })
    }

    /// Whether values of `t` copy (Rust `Copy`): everything but a `&mut`
    /// and a type that moves (`is_move`).
    fn is_copy(&self, t: &TypeKind) -> bool {
        !matches!(t, TypeKind::Ref { mutable: true, .. }) && !is_move(t, &self.adts)
    }

    /// Whether `t` has the built-in trait `b` (`Eq` or `Ord`): the integer
    /// types, `bool`, and a type parameter in scope bounded by it (`Ord`
    /// includes `Eq`).
    fn has_builtin(&self, t: &TypeKind, b: &str) -> bool {
        match t {
            TypeKind::Param(n) => self.tparams.iter().any(|p| {
                p.name == *n && p.bounds.iter().any(|x| x == b || (b == "Eq" && x == "Ord"))
            }),
            t => is_scalar(t),
        }
    }

    /// E0428/E0403 for a type argument `t` (at `span`) of parameter `p`:
    /// a value type that satisfies the bounds. False when reported.
    fn targ_ok(&mut self, t: &TypeKind, p: &TParam, of: &str, span: Span, report: bool) -> bool {
        match t {
            TypeKind::Error => false,
            TypeKind::Ref { .. } | TypeKind::Slice(_) | TypeKind::Unit => {
                if report {
                    self.diags.push(
                        Diag::new(
                            "E0403",
                            span,
                            format!(
                                "type argument `{}` for `{}` of `{of}` is not a value type",
                                ty_str(t),
                                p.name
                            ),
                        )
                        .fix(
                            "type arguments are value types: references are second-class (ADR-0015)",
                            None,
                        ),
                    );
                }
                false
            }
            _ if p
                .bounds
                .iter()
                .find(|b| matches!(b.as_str(), "Eq" | "Ord") && !self.has_builtin(t, b))
                .is_some() =>
            {
                let b = p
                    .bounds
                    .iter()
                    .find(|b| matches!(b.as_str(), "Eq" | "Ord") && !self.has_builtin(t, b))
                    .unwrap()
                    .clone();
                if report {
                    self.diags.push(
                        Diag::new(
                            "E0428",
                            span,
                            format!(
                                "`{}` is not `{b}`, a bound of `{}` in `{of}`",
                                ty_str(t),
                                p.name
                            ),
                        )
                        .fix(
                            format!("`{b}` holds for the integer types, `bool` and type parameters bounded by it; for data types it is ADR-0046 open question 4"),
                            None,
                        ),
                    );
                }
                false
            }
            _ if p
                .bounds
                .iter()
                .any(|b| b.contains("::") && !self.has_trait(t, b)) =>
            {
                let b = p
                    .bounds
                    .iter()
                    .find(|b| b.contains("::") && !self.has_trait(t, b))
                    .unwrap()
                    .clone();
                if report {
                    let what = if p.name == "Self" {
                        format!("`{}` does not implement `{b}`", ty_str(t))
                    } else {
                        format!(
                            "`{}` does not implement `{b}`, a bound of `{}` in `{of}`",
                            ty_str(t),
                            p.name
                        )
                    };
                    let help = if matches!(t, TypeKind::Param(_)) {
                        format!("bound the type parameter: `{}: {b}`", ty_str(t))
                    } else {
                        format!("write `impl {b} for {} {{ .. }}`", ty_str(t))
                    };
                    self.diags
                        .push(Diag::new("E0428", span, what).fix(help, None));
                }
                false
            }
            _ if p.bounds.iter().any(|b| b == "Copy") && !self.is_copy(t) => {
                if report {
                    self.diags.push(
                        Diag::new(
                            "E0428",
                            span,
                            format!(
                                "`{}` is not `Copy`, a bound of `{}` in `{of}`",
                                ty_str(t),
                                p.name
                            ),
                        )
                        .fix(
                            "pass a `Copy` type (a scalar, a struct or enum, an array of them); a `Vec`, a `Box` and what holds them move (ADR-0047)",
                            None,
                        ),
                    );
                }
                false
            }
            _ => true,
        }
    }

    /// A field or payload type: a value type without references (ADR-0015).
    fn field_type(&mut self, t: &Type, mp: &str) -> TypeKind {
        match &t.kind {
            TypeKind::Ref { .. } | TypeKind::Slice(_) => {
                self.diags.push(
                    Diag::new(
                        "E0403",
                        t.span,
                        "a field cannot hold a reference or a slice",
                    )
                    .fix(
                        "references are second-class: pass them as arguments only (ADR-0015)",
                        None,
                    ),
                );
                TypeKind::Error
            }
            TypeKind::Array(e, _) if !array_elem_ok(&e.kind) => {
                self.diags.push(
                    Diag::new(
                        "E0402",
                        t.span,
                        "array elements must be scalars, structs, enums or arrays",
                    )
                    .fix(ARRAY_ELEMENT_HELP, None),
                );
                TypeKind::Error
            }
            TypeKind::Array(..) => self.canon(t, mp, true),
            TypeKind::Unit => {
                self.err("E0402", t.span, "a field cannot have type `()`");
                TypeKind::Error
            }
            TypeKind::Named { path, .. } if path.len() == 1 && path[0] == "Vec" => {
                self.err(
                    "E0402",
                    t.span,
                    "a struct or enum cannot hold a `Vec` (ADR-0031 §1)",
                );
                TypeKind::Error
            }
            _ => self.canon(t, mp, true),
        }
    }

    /// Recursive types (ADR-0047 §1, backlog 2O-02). A type that contains
    /// itself by value has no finite size (E0421: put the recursion in a
    /// `Box`); one whose every value would be infinite has no value at all
    /// (E0421: end the recursion in a variant). A generic type may contain
    /// itself only with its own parameters, and not through another type
    /// (instances would never end). A recursive type holds no array yet
    /// (E0402: its specification equality would recurse without end).
    fn check_type_cycles(&mut self) {
        /// The types `t` holds by value (`Box` and `Vec` break it).
        fn contained(t: &TypeKind, out: &mut Vec<String>) {
            match t {
                TypeKind::Named { path, args } => {
                    if !(path.len() == 1 && matches!(path[0].as_str(), "Option" | "Result")) {
                        out.push(path.join("::"));
                    }
                    for a in args {
                        contained(&a.kind, out);
                    }
                }
                TypeKind::Array(e, _) => contained(&e.kind, out),
                _ => {}
            }
        }
        /// Every user type `t` names, through anything.
        fn named(t: &TypeKind, out: &mut Vec<(String, Vec<TypeKind>)>) {
            match t {
                TypeKind::Named { path, args } => {
                    out.push((
                        path.join("::"),
                        args.iter().map(|a| a.kind.clone()).collect(),
                    ));
                    for a in args {
                        named(&a.kind, out);
                    }
                }
                TypeKind::Array(e, _) | TypeKind::Box(e) | TypeKind::Vec(e) => named(&e.kind, out),
                _ => {}
            }
        }
        let parts = |a: &Adt| -> Vec<TypeKind> {
            match &a.kind {
                AdtKind::Struct(fs) => fs.iter().map(|(_, t)| t.clone()).collect(),
                AdtKind::Enum(vs) => vs.iter().flat_map(|(_, ts)| ts.iter().cloned()).collect(),
            }
        };
        let adts = self.adts.clone();
        let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut uses: BTreeMap<String, Vec<(String, Vec<TypeKind>)>> = BTreeMap::new();
        for (p, a) in &adts {
            let mut out = Vec::new();
            let mut all = Vec::new();
            for t in parts(a) {
                contained(&t, &mut out);
                named(&t, &mut all);
            }
            edges.insert(p.clone(), out);
            uses.insert(p.clone(), all);
        }
        let reach = |edges: &BTreeMap<String, Vec<String>>, p: &str| -> BTreeSet<String> {
            let mut stack: Vec<String> = edges.get(p).cloned().unwrap_or_default();
            let mut seen = BTreeSet::new();
            while let Some(q) = stack.pop() {
                if seen.insert(q.clone())
                    && let Some(next) = edges.get(&q)
                {
                    stack.extend(next.iter().cloned());
                }
            }
            seen
        };
        let mut reported = BTreeSet::new();
        for (p, a) in &adts {
            if reach(&edges, p).contains(p) {
                reported.insert(p.clone());
                self.diags.push(
                    Diag::new("E0421", a.span, format!("`{p}` contains itself"))
                        .fix("hold the recursive part in a `Box<..>` (ADR-0047 §1)", None),
                );
            }
        }
        // Types with a finite value: the least fixpoint.
        fn finite(t: &TypeKind, ok: &BTreeSet<String>) -> bool {
            match t {
                TypeKind::Named { path, .. } if path.len() == 1 && path[0] == "Option" => true,
                TypeKind::Named { path, args } if path.len() == 1 && path[0] == "Result" => {
                    args.iter().any(|a| finite(&a.kind, ok))
                }
                TypeKind::Named { path, .. } => ok.contains(&path.join("::")),
                TypeKind::Array(_, 0) | TypeKind::Vec(_) => true,
                TypeKind::Array(e, _) | TypeKind::Box(e) => finite(&e.kind, ok),
                _ => true,
            }
        }
        let mut ok: BTreeSet<String> = BTreeSet::new();
        loop {
            let before = ok.len();
            for (p, a) in &adts {
                let fin = match &a.kind {
                    AdtKind::Struct(fs) => fs.iter().all(|(_, t)| finite(t, &ok)),
                    AdtKind::Enum(vs) => vs.iter().any(|(_, ts)| ts.iter().all(|t| finite(t, &ok))),
                };
                if fin {
                    ok.insert(p.clone());
                }
            }
            if ok.len() == before {
                break;
            }
        }
        for (p, a) in &adts {
            // An enum without variants is reported where it is declared.
            let empty = matches!(&a.kind, AdtKind::Enum(vs) if vs.is_empty());
            if !ok.contains(p) && !reported.contains(p) && !empty {
                reported.insert(p.clone());
                self.diags.push(
                    Diag::new("E0421", a.span, format!("`{p}` has no finite value")).fix(
                        "end the recursion: an enum variant without it, or `Option<Box<..>>` (ADR-0047 §1)",
                        None,
                    ),
                );
            }
        }
        // Recursion through anything.
        let all_edges: BTreeMap<String, Vec<String>> = uses
            .iter()
            .map(|(p, us)| (p.clone(), us.iter().map(|(q, _)| q.clone()).collect()))
            .collect();
        for (p, a) in &adts {
            if reported.contains(p) {
                continue;
            }
            let r = reach(&all_edges, p);
            if !r.contains(p) {
                continue;
            }
            if !a.generics.is_empty() {
                let mutual = r
                    .iter()
                    .any(|q| q != p && adts.contains_key(q) && reach(&all_edges, q).contains(p));
                let own: Vec<TypeKind> = a
                    .generics
                    .iter()
                    .map(|g| TypeKind::Param(g.name.clone()))
                    .collect();
                let changed = uses[p].iter().any(|(q, args)| q == p && *args != own);
                if mutual || changed {
                    self.diags.push(
                        Diag::new(
                            "E0421",
                            a.span,
                            format!("`{p}` contains itself with other type arguments or through another type"),
                        )
                        .fix(
                            "a generic type may contain itself only as itself (`List<T>` in `List<T>`): its instances must be finitely many",
                            None,
                        ),
                    );
                    continue;
                }
            }
            if self.holds_array(
                &TypeKind::Named {
                    path: p.split("::").map(String::from).collect(),
                    args: Vec::new(),
                },
                &mut BTreeSet::new(),
            ) {
                self.diags.push(
                    Diag::new(
                        "E0402",
                        a.span,
                        format!("the recursive type `{p}` holds an array"),
                    )
                    .fix(
                        "hold the elements in a `Vec` or in separate fields: a recursive type with arrays is not supported yet (ADR-0047)",
                        None,
                    ),
                );
            }
        }
    }

    /// E0401 unless `t` is a measure: an integer, or the value of a
    /// recursive enum (its structural size, ADR-0047 §3).
    fn measure_type(&mut self, t: &TypeKind, span: Span) {
        let v = match t {
            TypeKind::Ref { inner, .. } => &inner.kind,
            t => t,
        };
        let v = match v {
            TypeKind::Box(inner) => &inner.kind,
            v => v,
        };
        if is_int(t) || *t == TypeKind::Error {
            return;
        }
        let is_enum = match v {
            TypeKind::Named { path, .. } => {
                (path.len() == 1 && matches!(path[0].as_str(), "Option" | "Result"))
                    || matches!(
                        self.adts.get(&path.join("::")).map(|a| &a.kind),
                        Some(AdtKind::Enum(_))
                    )
            }
            _ => false,
        };
        if is_enum && self.reaches_itself(v) {
            return;
        }
        let fix = if matches!(v, TypeKind::Named { .. }) && self.reaches_itself(v) {
            "decrease the enum-valued part that holds the recursion (a field such as `n.next`)"
        } else {
            "an integer, or a value of a recursive enum (its structural size)"
        };
        self.diags.push(
            Diag::new(
                "E0401",
                span,
                format!(
                    "`decreases` must be an integer or a recursive enum, found `{}`",
                    ty_str(t)
                ),
            )
            .fix(fix, None),
        );
    }

    /// Whether values of `t` can hold a value of type `t` again (a
    /// recursive type, ADR-0047 §1): through fields, payloads, type
    /// arguments of `Option` and `Result`, `Box`es and arrays.
    fn reaches_itself(&self, t: &TypeKind) -> bool {
        let target = ty_str(t);
        let mut seen = BTreeSet::new();
        let mut todo = self.components(t);
        while let Some(c) = todo.pop() {
            let k = ty_str(&c);
            if k == target {
                return true;
            }
            if seen.insert(k) {
                todo.extend(self.components(&c));
            }
        }
        false
    }

    /// The types a value of type `t` holds directly.
    fn components(&self, t: &TypeKind) -> Vec<TypeKind> {
        match t {
            TypeKind::Array(e, _) | TypeKind::Box(e) | TypeKind::Vec(e) => vec![e.kind.clone()],
            TypeKind::Named { path, args }
                if path.len() == 1 && matches!(path[0].as_str(), "Option" | "Result") =>
            {
                args.iter().map(|a| a.kind.clone()).collect()
            }
            TypeKind::Named { path, .. } => match self.adts.get(&path.join("::")) {
                Some(a) => {
                    let m = adt_subst(a, t);
                    match &a.kind {
                        AdtKind::Struct(fs) => fs.iter().map(|(_, ft)| subst(ft, &m)).collect(),
                        AdtKind::Enum(vs) => vs
                            .iter()
                            .flat_map(|(_, ps)| ps.iter().map(|pt| subst(pt, &m)))
                            .collect(),
                    }
                }
                None => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    /// The type with every named type resolved to its full path (ADR-0026).
    /// `report`: emit the diagnostics (once, where the type is checked).
    fn canon(&mut self, t: &Type, mp: &str, report: bool) -> TypeKind {
        let sub = |c: &mut Self, x: &Type| Type {
            kind: c.canon(x, mp, report),
            span: x.span,
        };
        match &t.kind {
            TypeKind::Named { path, args }
                if path.len() == 1 && self.tparams.iter().any(|p| p.name == path[0]) =>
            {
                if !args.is_empty() && report {
                    self.diags.push(
                        Diag::new(
                            "E0420",
                            t.span,
                            format!("type parameter `{}` takes no type arguments", path[0]),
                        )
                        .fix(format!("write `{}`", path[0]), Some(path[0].clone())),
                    );
                }
                TypeKind::Param(path[0].clone())
            }
            TypeKind::Named { path, args } => {
                let name = path.join("::");
                if name == "Self"
                    && let Some((head, _)) = self.self_head.clone()
                {
                    if report {
                        let d = self.unknown_type(&name, t.span, mp);
                        self.diags.push(d);
                    }
                    return head;
                }
                if path.len() == 1 && name == "Vec" {
                    return self.vec_type(t, args, mp, report);
                }
                if path.len() == 1 && name == "Box" {
                    return self.box_type(t, args, mp, report);
                }
                let last = path.last().map_or("", String::as_str);
                if (path.len() == 1 || path[0] == "std")
                    && matches!(last, "Rc" | "RefCell" | "Cell" | "Arc")
                    && !self.adts.contains_key(&format!("{mp}::{name}"))
                {
                    if report {
                        self.diags.push(
                            Diag::new(
                                "E0238",
                                t.span,
                                format!("`{last}` is not used: values have one owner"),
                            )
                            .fix(
                                "hold the value in an owned tree (`Box`, `Vec`) or refer to it by an index into a `Vec` (ADR-0047 §5)",
                                None,
                            ),
                        );
                    }
                    return TypeKind::Error;
                }
                if path.len() == 1
                    && name == "String"
                    && !self.adts.contains_key(&format!("{mp}::String"))
                {
                    if report {
                        self.diags.push(
                            Diag::new(
                                "E0402",
                                t.span,
                                "there is no `String`: text is bytes (ADR-0044)",
                            )
                            .fix("use `Vec<u8>`", Some("Vec<u8>".into())),
                        );
                    }
                    return TypeKind::Error;
                }
                if path.len() == 1 && (name == "Option" || name == "Result") {
                    let want = if name == "Option" { 1 } else { 2 };
                    if args.len() != want {
                        if report {
                            self.err(
                                "E0420",
                                t.span,
                                format!(
                                    "`{name}` takes {want} type argument(s), found {}",
                                    args.len()
                                ),
                            );
                        }
                        return TypeKind::Error;
                    }
                    let args: Vec<Type> = args.iter().map(|a| sub(self, a)).collect();
                    if let Some(v) = args.iter().find(|a| matches!(a.kind, TypeKind::Vec(_))) {
                        if report {
                            self.err(
                                "E0402",
                                v.span,
                                format!("`{name}` cannot hold a `Vec` (ADR-0031 §1)"),
                            );
                        }
                        return TypeKind::Error;
                    }
                    return TypeKind::Named {
                        path: path.clone(),
                        args,
                    };
                }
                let full = if path.len() == 1 {
                    format!("{mp}::{name}")
                } else {
                    name.clone()
                };
                if let Some(adt) = self.adts.get(&full) {
                    let ps = adt.generics.clone();
                    if args.len() != ps.len() {
                        if report {
                            let want = if ps.is_empty() {
                                format!("`{name}` has no type parameters: write `{name}`")
                            } else {
                                format!(
                                    "write one type for each parameter: `{name}<{}>`",
                                    ps.iter()
                                        .map(|p| p.name.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            };
                            self.diags.push(
                                Diag::new(
                                    "E0420",
                                    t.span,
                                    format!(
                                        "`{name}` takes {} type argument(s), found {}",
                                        ps.len(),
                                        args.len()
                                    ),
                                )
                                .fix(want, None),
                            );
                        }
                        return TypeKind::Error;
                    }
                    let args: Vec<Type> = args.iter().map(|a| sub(self, a)).collect();
                    let mut ok = true;
                    for (a, p) in args.iter().zip(&ps) {
                        ok &= self.targ_ok(&a.kind, p, &name, a.span, report);
                    }
                    if !ok {
                        return TypeKind::Error;
                    }
                    TypeKind::Named {
                        path: full.split("::").map(String::from).collect(),
                        args,
                    }
                } else {
                    if report {
                        let d = self.unknown_type(&name, t.span, mp);
                        self.diags.push(d);
                    }
                    TypeKind::Error
                }
            }
            TypeKind::Array(e, n) => {
                let el = sub(self, e);
                // A `Vec` element is reported where the type is written
                // (`array_elem_ok`).
                if self.owned_ready
                    && is_move(&el.kind, &self.adts)
                    && !matches!(el.kind, TypeKind::Vec(_))
                {
                    if report {
                        self.owned_elem("array", &el);
                    }
                    return TypeKind::Error;
                }
                TypeKind::Array(Box::new(el), *n)
            }
            TypeKind::Slice(e) => TypeKind::Slice(Box::new(sub(self, e))),
            TypeKind::Ref { mutable, inner } => TypeKind::Ref {
                mutable: *mutable,
                inner: Box::new(sub(self, inner)),
            },
            other => other.clone(),
        }
    }

    /// E0402 for an array or `Vec` element `el` that moves (backlog 2O-03).
    fn owned_elem(&mut self, what: &str, el: &Type) {
        self.diags.push(
            Diag::new(
                "E0402",
                el.span,
                format!(
                    "{what} elements that own heap values (`{}`) come with backlog 2O-03 (ADR-0047 §1)",
                    ty_str(&el.kind)
                ),
            )
            .fix(
                "keep the owned values in separate locals or fields for now",
                None,
            ),
        );
    }

    /// `Box<T>` (ADR-0047 §1): `T` a value type; a `Box` holding a `Vec`
    /// comes with nested containers (backlog 2O-03).
    fn box_type(&mut self, t: &Type, args: &[Type], mp: &str, report: bool) -> TypeKind {
        if args.len() != 1 {
            if report {
                self.err(
                    "E0420",
                    t.span,
                    format!("`Box` takes 1 type argument, found {}", args.len()),
                );
            }
            return TypeKind::Error;
        }
        let e = self.canon(&args[0], mp, report);
        match &e {
            TypeKind::Error => return TypeKind::Error,
            TypeKind::Ref { .. } | TypeKind::Slice(_) | TypeKind::Unit => {
                if report {
                    self.diags.push(
                        Diag::new(
                            "E0402",
                            args[0].span,
                            format!("a `Box` holds a value, found `{}`", ty_str(&e)),
                        )
                        .fix(
                            "box a scalar, a struct or enum, an array or another `Box`",
                            None,
                        ),
                    );
                }
                return TypeKind::Error;
            }
            TypeKind::Vec(_) => {
                if report {
                    self.err(
                        "E0402",
                        args[0].span,
                        "a `Box` holding a `Vec` comes with backlog 2O-03 (ADR-0047 §1)",
                    );
                }
                return TypeKind::Error;
            }
            _ => {}
        }
        TypeKind::Box(Box::new(Type {
            kind: e,
            span: args[0].span,
        }))
    }

    /// Fields that hold owned values (ADR-0047 §1), once every field type
    /// is known: no array or `Vec` of them yet (backlog 2O-03), and none
    /// as the argument of a `Copy` type parameter.
    fn check_owned_fields(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        for sub in &m.mods {
            self.check_owned_fields(sub, &format!("{mp}::"));
        }
        for t in &m.types {
            let path = format!("{mp}::{}", t.name.name);
            let Some(adt) = self.adts.get(&path) else {
                continue;
            };
            let written: Vec<&Type> = match &t.kind {
                TypeDefKind::Struct(fs) => fs.iter().map(|f| &f.ty).collect(),
                TypeDefKind::Enum(vs) => vs.iter().flat_map(|v| v.payload.iter()).collect(),
            };
            let resolved: Vec<TypeKind> = match &adt.kind {
                AdtKind::Struct(fs) => fs.iter().map(|(_, ft)| ft.clone()).collect(),
                AdtKind::Enum(vs) => vs.iter().flat_map(|(_, ps)| ps.iter().cloned()).collect(),
            };
            for (w, r) in written.into_iter().zip(resolved) {
                self.owned_field(&r, w.span);
            }
        }
    }

    fn owned_field(&mut self, t: &TypeKind, span: Span) {
        match t {
            TypeKind::Array(e, _) if is_move(&e.kind, &self.adts) => {
                let el = Type {
                    kind: e.kind.clone(),
                    span,
                };
                self.owned_elem("array", &el);
            }
            TypeKind::Array(e, _) | TypeKind::Box(e) => self.owned_field(&e.kind, span),
            TypeKind::Named { path, args } => {
                let ps = self
                    .adts
                    .get(&path.join("::"))
                    .map(|a| a.generics.clone())
                    .unwrap_or_default();
                for (a, p) in args.iter().zip(&ps) {
                    if p.bounds.iter().any(|b| b == "Copy") && !self.is_copy(&a.kind) {
                        self.targ_ok(&a.kind, p, &path.join("::"), span, true);
                    }
                }
                for a in args {
                    self.owned_field(&a.kind, span);
                }
            }
            _ => {}
        }
    }

    /// `Vec<T>` (ADR-0031 §1): `T` a scalar, or a struct or enum without
    /// arrays (the boundary of data type slices in the proof gate).
    fn vec_type(&mut self, t: &Type, args: &[Type], mp: &str, report: bool) -> TypeKind {
        if args.len() != 1 {
            if report {
                self.err(
                    "E0420",
                    t.span,
                    format!("`Vec` takes 1 type argument, found {}", args.len()),
                );
            }
            return TypeKind::Error;
        }
        let e = self.canon(&args[0], mp, report);
        if self.owned_ready && is_move(&e, &self.adts) {
            if report {
                self.owned_elem(
                    "`Vec`",
                    &Type {
                        kind: e,
                        span: args[0].span,
                    },
                );
            }
            return TypeKind::Error;
        }
        let ok = match &e {
            TypeKind::Named { .. } => !self.holds_array(&e, &mut BTreeSet::new()),
            TypeKind::Error => return TypeKind::Error,
            other => is_scalar(other),
        };
        if !ok {
            if report {
                self.err(
                    "E0402",
                    args[0].span,
                    format!(
                        "`Vec` elements are scalars, or structs and enums without arrays (ADR-0031), found `{}`",
                        ty_str(&e)
                    ),
                );
            }
            return TypeKind::Error;
        }
        TypeKind::Vec(Box::new(Type {
            kind: e,
            span: args[0].span,
        }))
    }

    /// Whether a value of type `t` holds an array (through fields and payloads).
    fn holds_array(&self, t: &TypeKind, seen: &mut BTreeSet<String>) -> bool {
        match t {
            TypeKind::Array(..) => true,
            TypeKind::Named { path, args } => {
                if args.iter().any(|a| self.holds_array(&a.kind, seen)) {
                    return true;
                }
                let p = path.join("::");
                if !seen.insert(p.clone()) {
                    return false;
                }
                match self.adts.get(&p).map(|a| &a.kind) {
                    Some(AdtKind::Struct(fs)) => {
                        fs.iter().any(|(_, ft)| self.holds_array(ft, seen))
                    }
                    Some(AdtKind::Enum(vs)) => vs
                        .iter()
                        .any(|(_, ps)| ps.iter().any(|pt| self.holds_array(pt, seen))),
                    None => false,
                }
            }
            _ => false,
        }
    }

    fn collect_mod(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        // `use` is not part of the language: the item goes, and every name
        // it imports is written in full where it is used (backlog 2V-15).
        for u in &m.uses {
            let full = u
                .path
                .iter()
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>()
                .join("::");
            let last = u.path.last().expect("a path").name.clone();
            self.imports
                .entry(mp.clone())
                .or_default()
                .insert(last, full.clone());
            self.diags.push(
                Diag::new(
                    "E0214",
                    u.del,
                    format!("`use {full};` is not used: write `{full}` where it is used"),
                )
                .fix(
                    "delete the `use`; its names are rewritten to full paths",
                    Some(String::new()),
                ),
            );
        }
        let mut names = BTreeSet::new();
        for t in &m.types {
            if BUILTIN_NAMES.contains(&t.name.name.as_str()) {
                self.err(
                    "E0309",
                    t.name.span,
                    format!("`{}` is a built-in name (ADR-0026)", t.name.name),
                );
            }
            if !names.insert(t.name.name.clone()) {
                self.err(
                    "E0302",
                    t.name.span,
                    format!("`{}` is defined more than once in `{mp}`", t.name.name),
                );
            }
        }
        for t in &m.traits {
            if !names.insert(t.name.name.clone()) {
                self.err(
                    "E0302",
                    t.name.span,
                    format!("`{}` is defined more than once in `{mp}`", t.name.name),
                );
            }
        }
        for sub in &m.mods {
            if !names.insert(sub.name.name.clone()) {
                self.err(
                    "E0302",
                    sub.name.span,
                    format!("`{}` is defined more than once in `{mp}`", sub.name.name),
                );
            }
            self.collect_mod(sub, &format!("{mp}::"));
        }
        for f in &m.fns {
            if BUILTIN_NAMES.contains(&f.name.name.as_str()) {
                self.err(
                    "E0309",
                    f.name.span,
                    format!("`{}` is a built-in name (ADR-0026)", f.name.name),
                );
            }
            if !names.insert(f.name.name.clone()) {
                self.err(
                    "E0302",
                    f.name.span,
                    format!("`{}` is defined more than once in `{mp}`", f.name.name),
                );
                continue;
            }
            let path = format!("{mp}::{}", f.name.name);
            self.collect_fn(f, path, &mp, false, &[]);
        }
        let mut with_impl = BTreeSet::new();
        for i in &m.impls {
            let ty = format!("{mp}::{}", i.ty.name);
            if !m.types.iter().any(|t| t.name.name == i.ty.name) {
                self.err(
                    "E0308",
                    i.ty.span,
                    format!(
                        "`impl {}` needs the type declared in the same unit `{mp}`",
                        i.ty.name
                    ),
                );
                continue;
            }
            if !with_impl.insert(ty.clone()) {
                self.err(
                    "E0308",
                    i.ty.span,
                    format!("`{}` already has an `impl` block; one per type", i.ty.name),
                );
                continue;
            }
            let ig = self.tparams_of(&i.generics, true, &mp);
            let arity = self.adts.get(&ty).map_or(0, |a| a.generics.len());
            let written: Vec<Option<&str>> = i
                .ty_args
                .iter()
                .map(|a| match &a.kind {
                    TypeKind::Named { path, args } if path.len() == 1 && args.is_empty() => {
                        Some(path[0].as_str())
                    }
                    _ => None,
                })
                .collect();
            let own: Vec<Option<&str>> = ig.iter().map(|p| Some(p.name.as_str())).collect();
            if ig.len() != arity || written != own {
                let ps: Vec<String> = self.adts[&ty]
                    .generics
                    .iter()
                    .map(|p| p.name.clone())
                    .collect();
                let r = if ps.is_empty() {
                    format!("impl {}", i.ty.name)
                } else {
                    format!(
                        "impl<{}> {}<{}>",
                        ps.iter()
                            .map(|p| format!("{p}: Copy"))
                            .collect::<Vec<_>>()
                            .join(", "),
                        i.ty.name,
                        ps.join(", ")
                    )
                };
                self.diags.push(
                    Diag::new(
                        "E0420",
                        i.ty.span,
                        format!(
                            "the `impl` of `{}` takes its type parameters in order",
                            i.ty.name
                        ),
                    )
                    .fix(format!("write `{r}`"), None),
                );
                continue;
            }
            let mut fnames = BTreeSet::new();
            for f in &i.fns {
                if !fnames.insert(f.name.name.clone()) {
                    self.err(
                        "E0302",
                        f.name.span,
                        format!(
                            "`{}` is defined more than once in `impl {}`",
                            f.name.name, i.ty.name
                        ),
                    );
                    continue;
                }
                let fg = self.tparams_of(&f.generics, false, &mp);
                self.tparams = ig.clone();
                self.tparams.extend(fg);
                self.receiver_rule(f, Some(&ty), &mp);
                self.tparams.clear();
                self.collect_fn(f, format!("{ty}::{}", f.name.name), &mp, true, &ig);
            }
        }
        for f in &m.fns {
            self.tparams = self.tparams_of(&f.generics, false, &mp);
            self.receiver_rule(f, None, &mp);
            self.tparams.clear();
        }
        for ti in &m.trait_impls {
            let Some(info) = self.trait_impls.iter().find(|x| x.span == ti.span).cloned() else {
                continue;
            };
            let recv = match &info.head {
                TypeKind::Named { path, .. } => path.join("::"),
                t => ty_str(t),
            };
            self.self_head = Some((info.head.clone(), ty_str_in(&info.head, &mp)));
            let mut fnames = BTreeSet::new();
            for f in &ti.fns {
                if !fnames.insert(f.name.name.clone()) {
                    self.err(
                        "E0302",
                        f.name.span,
                        format!("`{}` is defined more than once in this `impl`", f.name.name),
                    );
                    continue;
                }
                let fg = self.tparams_of(&f.generics, false, &mp);
                self.tparams = info.generics.clone();
                self.tparams.extend(fg);
                self.receiver_rule(f, Some(&recv), &mp);
                self.tparams.clear();
                self.collect_fn(f, info.fn_path(&f.name.name), &mp, true, &info.generics);
            }
            self.conform(&info, ti, &mp);
            self.self_head = None;
        }
    }

    /// `self` names the receiver: the first parameter of an `impl T` function
    /// whose type is `T`, `&T` or `&mut T`; that parameter is named `self`
    /// (ADR-0026 §3).
    fn receiver_rule(&mut self, f: &FnDef, impl_ty: Option<&str>, mp: &str) {
        for (k, p) in f.params.iter().enumerate() {
            let ty = self.canon(&p.ty, mp, false);
            let own = |t: &TypeKind| match t {
                TypeKind::Named { path, .. } => Some(path.join("::")) == impl_ty.map(String::from),
                TypeKind::Error => false,
                t => impl_ty.is_some_and(|i| ty_str(t) == i),
            };
            let receiver_type = match &ty {
                TypeKind::Ref { inner, .. } => own(&inner.kind),
                t => own(t),
            };
            let is_self = p.name.name == "self";
            if k == 0 && receiver_type && !is_self {
                self.diags.push(
                    Diag::new(
                        "E0310",
                        p.name.span,
                        format!("the receiver of `{}` is named `self`", f.name.name),
                    )
                    .fix(
                        format!(
                            "rename `{}` to `self` in the signature and the body",
                            p.name.name
                        ),
                        None,
                    ),
                );
            } else if is_self && (k != 0 || !receiver_type) && ty != TypeKind::Error {
                self.diags.push(
                    Diag::new(
                        "E0310",
                        p.name.span,
                        "`self` names only the receiver: the first parameter of an `impl T` function, typed `T`, `&T` or `&mut T`",
                    )
                    .fix("choose another name", None),
                );
            }
        }
    }

    fn collect_fn(&mut self, f: &FnDef, path: String, mp: &str, in_impl: bool, ig: &[TParam]) {
        let own = self.tparams_of(&f.generics, true, mp);
        for (g, p) in f.generics.iter().zip(&own) {
            if ig.iter().any(|q| q.name == p.name) {
                self.err(
                    "E0302",
                    g.name.span,
                    format!("type parameter `{}` is already the `impl`'s", p.name),
                );
            }
        }
        let mut generics = ig.to_vec();
        generics.extend(own);
        self.tparams = generics.clone();
        let params = f
            .params
            .iter()
            .map(|p| self.canon(&p.ty, mp, false))
            .collect();
        let ret = self.canon(&f.ret.ty, mp, false);
        self.tparams.clear();
        {
            let effect = self.fn_effect(f);
            let vacuous = f.kind == FnKind::Spec
                && f.body.stmts.is_empty()
                && matches!(
                    f.body.tail.as_deref(),
                    Some(Expr {
                        kind: ExprKind::Bool(true),
                        ..
                    })
                );
            self.fns.insert(
                path.clone(),
                FnSig {
                    path,
                    kind: f.kind,
                    generics,
                    impl_generics: ig.len(),
                    params,
                    ret,
                    effect,
                    vacuous,
                    builtin: false,
                    in_impl,
                    trait_of: None,
                    default: false,
                    span: f.span,
                },
            );
        }
    }

    /// Reads `#[effects(..)]` and reports attribute problems once, during collection.
    fn fn_effect(&mut self, f: &FnDef) -> Effect {
        let mut effect = None;
        for a in &f.attrs {
            match a.name.name.as_str() {
                "effects" => {
                    if f.kind == FnKind::Spec {
                        self.diags.push(
                            Diag::new("E0604", a.span, "spec functions are pure by definition")
                                .fix("remove the attribute", Some(String::new())),
                        );
                        continue;
                    }
                    if effect.is_some() {
                        self.diags.push(
                            Diag::new("E0605", a.span, "duplicate `#[effects(..)]` attribute")
                                .fix("remove the attribute", Some(String::new())),
                        );
                        continue;
                    }
                    let mut e = Effect::Pure;
                    if a.args.len() != 1 {
                        self.err(
                            "E0602",
                            a.span,
                            "`#[effects(..)]` takes exactly one effect: `pure` or `io`",
                        );
                    }
                    for arg in &a.args {
                        match arg {
                            AttrArg::Ident(i) if i.name == "pure" => e = Effect::Pure,
                            AttrArg::Ident(i) if i.name == "io" => e = Effect::Io,
                            AttrArg::Ident(i) => self.diags.push(
                                Diag::new(
                                    "E0602",
                                    i.span,
                                    format!(
                                        "unknown effect `{}`; milestone 1 has `pure` and `io`",
                                        i.name
                                    ),
                                )
                                .fix("write `pure`, or `io` if the function prints", None),
                            ),
                            AttrArg::Str(_, sp) => {
                                self.err("E0602", *sp, "effect must be an identifier")
                            }
                        }
                    }
                    effect = Some(e);
                }
                "uses" => {}
                other => self.diags.push(
                    Diag::new(
                        "E0605",
                        a.span,
                        format!("unknown attribute `{other}` on a function"),
                    )
                    .fix(
                        "remove it: functions take `#[effects(..)]` and `#[uses(..)]`",
                        Some(String::new()),
                    ),
                ),
            }
        }
        if f.kind == FnKind::Exec && effect.is_none() {
            self.diags.push(
                Diag::new(
                    "E0601",
                    // The `fn` keyword, so the replacement can prepend the attribute.
                    Span {
                        hi: f.span.lo + 2,
                        ..f.span
                    },
                    format!("`{}` has no `#[effects(..)]` attribute", f.name.name),
                )
                .fix(
                    "add `#[effects(pure)]` above the function",
                    Some("#[effects(pure)] fn".into()),
                ),
            );
        }
        effect.unwrap_or(Effect::Pure)
    }

    fn check_mod(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        let mut has_summary = false;
        for a in &m.attrs {
            if a.name.name == "summary"
                && a.args.len() == 1
                && matches!(a.args[0], AttrArg::Str(..))
            {
                has_summary = true;
            } else if a.name.name == "summary" {
                self.err("E0704", a.span, "`#[summary(..)]` takes exactly one string");
                has_summary = true;
            } else {
                self.diags.push(
                    Diag::new(
                        "E0605",
                        a.span,
                        format!("unknown attribute `{}` on a unit", a.name.name),
                    )
                    .fix(
                        "remove it: units take `#[summary(\"..\")]`",
                        Some(String::new()),
                    ),
                );
            }
        }
        if !has_summary {
            self.diags.push(
                Diag::new(
                    "E0704",
                    m.name.span,
                    format!("unit `{}` has no summary", m.name.name),
                )
                .fix("add `#[summary(\"one line\")]` above `mod`", None),
            );
        }
        for sub in &m.mods {
            self.check_mod(sub, &format!("{mp}::"));
        }
        for f in &m.fns {
            self.check_fn(f, &mp, &format!("{mp}::{}", f.name.name));
        }
        for i in &m.impls {
            for f in &i.fns {
                self.check_fn(f, &mp, &format!("{mp}::{}::{}", i.ty.name, f.name.name));
            }
        }
        // Trait method declarations: their signatures and contracts.
        for t in &m.traits {
            for tf in &t.fns {
                let path = format!("{mp}::{}::{}", t.name.name, tf.f.name.name);
                if self.fns.get(&path).is_some_and(|s| s.span == tf.f.span) {
                    self.check_fn(&tf.f, &mp, &path);
                }
            }
        }
        for ti in &m.trait_impls {
            let Some(info) = self.trait_impls.iter().find(|x| x.span == ti.span).cloned() else {
                continue;
            };
            self.self_head = Some((info.head.clone(), ty_str_in(&info.head, &mp)));
            for f in &ti.fns {
                let path = info.fn_path(&f.name.name);
                if self.fns.get(&path).is_some_and(|s| s.span == f.span) {
                    self.check_fn(f, &mp, &path);
                }
            }
            self.self_head = None;
        }
    }

    fn resolve(&self, path: &[Ident], mod_path: &str) -> Option<String> {
        // `T::f(..)` on a type of this unit (ADR-0026 §3), `Trait::f(..)`
        // on a trait of this unit (ADR-0046 §2).
        if path.len() == 2
            && (self
                .adts
                .contains_key(&format!("{mod_path}::{}", path[0].name))
                || self
                    .traits
                    .contains_key(&format!("{mod_path}::{}", path[0].name)))
        {
            let full = format!("{mod_path}::{}::{}", path[0].name, path[1].name);
            if self.fns.contains_key(&full) {
                return Some(full);
            }
        }
        if path.len() == 1 {
            let local = format!("{mod_path}::{}", path[0].name);
            if self.fns.contains_key(&local) {
                return Some(local);
            }
            if self.fns.get(&path[0].name).is_some_and(|s| s.builtin) {
                return Some(path[0].name.clone());
            }
            return None;
        }
        let p = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        self.fns.contains_key(&p).then_some(p)
    }

    // ---------- functions ----------

    fn check_fn(&mut self, f: &FnDef, mod_path: &str, path: &str) {
        let path = path.to_string();
        let Some(sig) = self.fns.get(&path).cloned() else {
            return;
        };
        self.tparams = sig.generics.clone();
        let mut cx = FnCtx {
            path: path.clone(),
            mod_path: mod_path.to_string(),
            effect: sig.effect,
            scopes: vec![BTreeMap::new()],
            all_names: BTreeSet::new(),
            forall_depth: 0,
        };
        self.calls.entry(path.clone()).or_default();

        for a in &f.attrs {
            if a.name.name == "uses" {
                for arg in &a.args {
                    match arg {
                        AttrArg::Ident(i) => {
                            let target = uses_target(&i.name, mod_path, &self.fns);
                            let kind = target.and_then(|p| self.fns.get(&p)).map(|s| s.kind);
                            if kind != Some(FnKind::Spec) {
                                let fix = if kind.is_some() {
                                    "remove it from `#[uses(..)]`: only spec function bodies are \
                                     revealed; executable functions are seen through their contracts"
                                } else {
                                    "remove it from `#[uses(..)]`, or define the spec function \
                                     (another unit's as `unit::name`)"
                                };
                                self.diags.push(
                                    Diag::new(
                                        "E0509",
                                        i.span,
                                        format!("`{}` is not a spec function", i.name),
                                    )
                                    .fix(fix, None),
                                );
                            }
                        }
                        AttrArg::Str(_, sp) => {
                            self.err("E0509", *sp, "`#[uses(..)]` takes spec function names")
                        }
                    }
                }
            }
        }

        for p in &f.params {
            self.check_param_type(&p.ty, f.kind);
            let ty = self.canon(&p.ty, mod_path, true);
            self.declare(&mut cx, &p.name, ty, false, true);
        }
        self.check_value_type(&f.ret.ty, "return type");
        if f.kind == FnKind::Spec
            && matches!(&f.ret.ty.kind, TypeKind::Named { path, .. } if path.len() == 1 && path[0] == "Vec")
        {
            self.spec_vec(f.ret.ty.span);
        }
        let ret_ty = self.canon(&f.ret.ty, mod_path, true);

        if f.kind == FnKind::Spec && (!f.requires.is_empty() || !f.ensures.is_empty()) {
            let sp = f.requires.first().or(f.ensures.first()).unwrap().span;
            self.err(
                "E0207",
                sp,
                "spec functions take no `requires` / `ensures` in milestone 1",
            );
        }

        for r in &f.requires {
            self.check_contract(&mut cx, r, false);
        }
        if !f.ensures.is_empty() {
            cx.scopes.push(BTreeMap::new());
            if let Some(n) = &f.ret.name {
                self.declare(&mut cx, n, ret_ty.clone(), false, false);
            }
            for e in &f.ensures {
                self.check_contract(&mut cx, e, true);
            }
            cx.scopes.pop();
        } else if let Some(n) = &f.ret.name {
            // The result name is still reserved so the body cannot rebind it.
            cx.all_names.insert(n.name.clone());
        }
        if let Some(d) = &f.decreases {
            let t = self.expr(&mut cx, d, None, Mode::Spec { allow_old: false });
            self.measure_type(&t, d.span);
        }

        // A trait method declaration without a default body (ADR-0046 §1).
        if sig.trait_of.is_some() && !sig.default {
            self.tparams.clear();
            return;
        }
        let mode = if f.kind == FnKind::Spec {
            Mode::Spec { allow_old: false }
        } else {
            Mode::Exec
        };
        let t = self.block(&mut cx, &f.body, Some(&ret_ty), mode);
        self.expect_ty(
            &t,
            &ret_ty,
            f.body.tail.as_ref().map_or(f.body.span, |e| e.span),
        );
        if f.kind == FnKind::Exec {
            let none = BTreeMap::new();
            let locals = self.locals.get(&path).unwrap_or(&none);
            let (diags, info) = crate::moves::check(f, locals, &self.types, &self.adts);
            for d in diags {
                self.diags.push(d);
            }
            self.moves.insert(path, info);
        }
        self.tparams.clear();
    }

    fn check_param_type(&mut self, t: &Type, kind: FnKind) {
        match &t.kind {
            TypeKind::Ref { mutable, inner } => match &inner.kind {
                // `&[T]`, `&Point`, `&[Point]` (ADR-0026 §2).
                TypeKind::Slice(e) if array_elem_ok(&e.kind) => {
                    if *mutable && kind == FnKind::Spec {
                        self.err("E0402", t.span, "spec function parameters cannot be `&mut`");
                    }
                }
                TypeKind::Named { path, .. } if path.len() == 1 && path[0] == "Vec" => {
                    if kind == FnKind::Spec {
                        self.spec_vec(t.span);
                    }
                }
                TypeKind::Named { .. } => {
                    if *mutable && kind == FnKind::Spec {
                        self.err("E0402", t.span, "spec function parameters cannot be `&mut`");
                    }
                }
                // `&i64`, `&mut bool` (backlog 2T-08): a receiver of a
                // trait impl for a scalar, an out-parameter.
                k if is_scalar(k) => {
                    if *mutable && kind == FnKind::Spec {
                        self.err("E0402", t.span, "spec function parameters cannot be `&mut`");
                    }
                }
                TypeKind::Error => {}
                _ => self.diags.push(
                    Diag::new(
                        "E0402",
                        t.span,
                        "references are `&[T]`, `&mut [T]`, `&T` or `&mut T` with `T` a scalar, struct or enum",
                    )
                    .fix(
                        "pass the value itself, or a reference to the scalar, struct, enum or slice that holds it",
                        None,
                    ),
                ),
            },
            TypeKind::Named { path, .. }
                if kind == FnKind::Spec && path.len() == 1 && path[0] == "Vec" =>
            {
                self.spec_vec(t.span)
            }
            // Checked after canonicalization (`data_array`).
            TypeKind::Array(e, _) if matches!(e.kind, TypeKind::Named { .. }) => {}
            _ => self.check_value_type(t, "parameter type"),
        }
    }

    /// E0402: specifications see a `Vec` as a slice (ADR-0031 §3).
    fn spec_vec(&mut self, span: Span) {
        self.diags.push(
            Diag::new(
                "E0402",
                span,
                "a spec function cannot take or return a `Vec`",
            )
            .fix("take `&[T]` and pass `v.as_slice()`", None),
        );
    }

    /// Types for locals and results: no references, no bare slices.
    fn check_value_type(&mut self, t: &Type, what: &str) {
        match &t.kind {
            TypeKind::Ref { .. } => self.diags.push(
                Diag::new("E0403", t.span, format!("a reference cannot be a {what}")).fix(
                    "references are second-class: pass them as arguments only (ADR-0015)",
                    None,
                ),
            ),
            TypeKind::Slice(_) => self.err(
                "E0402",
                t.span,
                "a bare slice is unsized; use `&[T]` as a parameter or `[T; N]`",
            ),
            TypeKind::Array(e, _) if !array_elem_ok(&e.kind) => {
                self.diags.push(
                    Diag::new(
                        "E0402",
                        t.span,
                        "array elements must be scalars, structs, enums or arrays",
                    )
                    .fix(ARRAY_ELEMENT_HELP, None),
                );
            }
            _ => {}
        }
    }

    /// An element of an array literal or repeat: not a `Vec` (backlog
    /// 1B-35, 2B-14, 2V-04b).
    fn literal_elem(&mut self, t: &TypeKind, span: Span) {
        if is_move(t, &self.adts) && !matches!(t, TypeKind::Vec(_)) {
            self.owned_elem(
                "array",
                &Type {
                    kind: t.clone(),
                    span,
                },
            );
            return;
        }
        let bad = matches!(t, TypeKind::Vec(_));
        if bad {
            self.diags.push(
                Diag::new(
                    "E0402",
                    span,
                    format!(
                        "array elements are scalars, structs, enums or arrays, found `{}`",
                        ty_str(t)
                    ),
                )
                .fix(ARRAY_ELEMENT_HELP, None),
            );
        }
    }

    fn check_contract(&mut self, cx: &mut FnCtx, e: &Expr, ensures: bool) {
        let mode = Mode::Spec { allow_old: ensures };
        let t = self.expr(cx, e, Some(&TypeKind::Bool), mode);
        let named = match &e.kind {
            ExprKind::Call { path, .. } => self
                .resolve(path, &cx.mod_path)
                .and_then(|p| self.fns.get(&p).cloned()),
            _ => None,
        };
        match named {
            Some(sig) if sig.kind == FnKind::Spec && sig.ret == TypeKind::Bool => {
                if sig.vacuous {
                    self.err(
                        "E0502",
                        e.span,
                        format!("`{}` is the constant `true`; it states nothing", sig.path),
                    );
                }
            }
            _ => {
                if t != TypeKind::Error {
                    self.diags.push(
                        Diag::new("E0501", e.span, "contract clause is not a named predicate")
                            .fix("move the condition into `spec fn name(..) -> bool { .. }` and call it here", None),
                    );
                }
            }
        }
    }

    fn declare(
        &mut self,
        cx: &mut FnCtx,
        name: &Ident,
        ty: TypeKind,
        mutable: bool,
        is_param: bool,
    ) {
        if !cx.all_names.insert(name.name.clone()) {
            self.diags.push(
                Diag::new(
                    "E0303",
                    name.span,
                    format!("`{}` is already bound in this function", name.name),
                )
                .fix("choose a new name", None),
            );
        }
        self.locals
            .entry(cx.path.clone())
            .or_default()
            .insert(name.name.clone(), ty.clone());
        cx.scopes.last_mut().unwrap().insert(
            name.name.clone(),
            Local {
                ty,
                mutable,
                is_param,
            },
        );
    }

    fn lookup(cx: &FnCtx, name: &str) -> Option<Local> {
        cx.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }

    /// E0301 for a local that is not in scope, with the one visible local
    /// closest to it (edit distance at most 2) as the fix (backlog 2D-18).
    fn unknown_local(&mut self, cx: &FnCtx, name: &str, span: Span) {
        let mut d = Diag::new("E0301", span, format!("unknown name `{name}`"));
        if let Some(near) = closest(name, cx.scopes.iter().flat_map(|s| s.keys())) {
            d = d.fix(format!("did you mean `{near}`?"), Some(near));
        }
        self.diags.push(d);
    }

    /// E0401 with the fix `*e` when `e` is a `&T` / `&mut T` given where a
    /// struct or enum `T` is expected (backlog 2B-16: `moved(old(self), ..)`).
    /// Whether it reported.
    fn deref_fix(&mut self, found: &TypeKind, want: &TypeKind, e: &Expr) -> bool {
        let TypeKind::Ref { inner, .. } = found else {
            return false;
        };
        if inner.kind != *want || !matches!(want, TypeKind::Named { .. }) {
            return false;
        }
        let r = format!("*{}", crate::fmt::expr_text(e));
        self.diags.push(
            Diag::new(
                "E0401",
                e.span,
                format!("expected `{}`, found `{}`", ty_str(want), ty_str(found)),
            )
            .fix(format!("pass the value: `{r}`"), Some(r)),
        );
        true
    }

    fn expect_ty(&mut self, found: &TypeKind, want: &TypeKind, span: Span) {
        if found != want && *found != TypeKind::Error && *want != TypeKind::Error {
            let d = Diag::new(
                "E0401",
                span,
                format!("expected `{}`, found `{}`", ty_str(want), ty_str(found)),
            );
            // A binding of a `match` on a reference refers to its payload.
            let d = match found {
                TypeKind::Ref { inner, .. } if inner.kind == *want => {
                    d.fix("read the value it refers to: `*x`", None)
                }
                _ => d,
            };
            self.diags.push(d);
        }
    }

    // ---------- blocks and statements ----------

    fn block(
        &mut self,
        cx: &mut FnCtx,
        b: &Block,
        expected: Option<&TypeKind>,
        mode: Mode,
    ) -> TypeKind {
        cx.scopes.push(BTreeMap::new());
        for s in &b.stmts {
            self.stmt(cx, s, mode);
        }
        let t = match &b.tail {
            Some(e) => self.expr(cx, e, expected, mode),
            None => TypeKind::Unit,
        };
        cx.scopes.pop();
        t
    }

    fn stmt(&mut self, cx: &mut FnCtx, s: &Stmt, mode: Mode) {
        match s {
            // Rust habit: destructuring. The field types make the fix.
            Stmt::LetFields {
                path: _,
                fields,
                init,
                span,
            } => {
                let it = self.expr(cx, init, None, mode);
                let decl = self.struct_fields(&it);
                let mut lets = Vec::new();
                for f in fields {
                    let t = decl
                        .as_ref()
                        .and_then(|(_, fs)| fs.iter().find(|(n, _)| *n == f.name))
                        .map(|(_, t)| t.clone());
                    if let (Some(t), ExprKind::Var(v)) = (&t, &init.kind) {
                        lets.push(format!(
                            "let {}: {} = {}.{};",
                            f.name,
                            ty_str(t),
                            v.name,
                            f.name
                        ));
                    }
                    self.declare(cx, f, t.unwrap_or(TypeKind::Error), false, false);
                }
                let mut d = Diag::new("E0230", *span, "destructuring `let` is not allowed");
                d = if lets.len() == fields.len() {
                    d.fix("read each field into its own `let`", Some(lets.join(" ")))
                } else {
                    d.fix(
                        "read each field into its own `let`: `let x: T = p.x;`",
                        None,
                    )
                };
                self.diags.push(d);
            }
            Stmt::Let {
                mutable,
                name,
                ty_missing: true,
                init,
                ..
            } => {
                // Rust habit: infer the type only to offer it as the fix.
                let t = self.expr(cx, init, None, mode);
                let mut d = Diag::new(
                    "E0201",
                    name.span,
                    format!("`let {}` requires an explicit type", name.name),
                );
                d = if t == TypeKind::Error {
                    d.fix("add `: T` after the name", None)
                } else {
                    let r = format!("{}: {}", name.name, ty_str(&t));
                    d.fix(format!("write `{r}`"), Some(r))
                };
                self.diags.push(d);
                self.declare(cx, name, t, *mutable, false);
            }
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
                ..
            } => {
                self.check_value_type(ty, "local variable type");
                if *mutable && mode != Mode::Exec {
                    self.err(
                        "E0503",
                        name.span,
                        "`let mut` is not allowed in a specification",
                    );
                }
                let lty = self.canon(ty, &cx.mod_path.clone(), true);
                let t = self.expr(cx, init, Some(&lty), mode);
                if !self.deref_fix(&t, &lty, init) {
                    self.expect_ty(&t, &lty, init.span);
                }
                self.declare(cx, name, lty, *mutable, false);
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                if mode != Mode::Exec {
                    self.err(
                        "E0503",
                        *span,
                        "assignment is not allowed in a specification",
                    );
                }
                let tt = self.place(cx, target);
                // The target's type, for the move check (`moves.rs`).
                self.types.insert(
                    (target.span.file, target.span.lo, target.span.hi),
                    tt.clone(),
                );
                let vt = self.expr(cx, value, Some(&tt), mode);
                self.expect_ty(&vt, &tt, value.span);
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                span,
            } => {
                if mode != Mode::Exec {
                    self.err("E0503", *span, "loops are not allowed in a specification");
                }
                let ct = self.expr(cx, cond, Some(&TypeKind::Bool), mode);
                self.expect_ty(&ct, &TypeKind::Bool, cond.span);
                // `old(a)` in an invariant or assertion is `a` on function entry.
                for inv in invariants {
                    let t = self.expr(
                        cx,
                        inv,
                        Some(&TypeKind::Bool),
                        Mode::Spec { allow_old: true },
                    );
                    self.expect_ty(&t, &TypeKind::Bool, inv.span);
                }
                match decreases {
                    Some(d) => {
                        let t = self.expr(cx, d, None, Mode::Spec { allow_old: false });
                        if !is_int(&t) && t != TypeKind::Error {
                            self.err(
                                "E0401",
                                d.span,
                                format!("`decreases` must be an integer, found `{}`", ty_str(&t)),
                            );
                        }
                    }
                    None => {
                        let sp = Span {
                            hi: cond.span.hi,
                            ..*span
                        };
                        self.diags.push(
                            Diag::new("E0506", sp, "`while` has no `decreases` clause")
                                .fix("add `decreases <expr>,` before the loop body", None),
                        );
                    }
                }
                let bt = self.block(cx, body, Some(&TypeKind::Unit), mode);
                self.expect_ty(&bt, &TypeKind::Unit, body.span);
            }
            Stmt::Assert { cond, span } => {
                if mode != Mode::Exec {
                    self.err("E0503", *span, "`assert` is not allowed in a specification");
                }
                let t = self.expr(
                    cx,
                    cond,
                    Some(&TypeKind::Bool),
                    Mode::Spec { allow_old: true },
                );
                self.expect_ty(&t, &TypeKind::Bool, cond.span);
            }
            Stmt::Expr(e) => {
                self.expr(cx, e, None, mode);
            }
        }
    }

    /// Checks an assignment target and returns its type.
    fn place(&mut self, cx: &mut FnCtx, target: &Expr) -> TypeKind {
        match &target.kind {
            ExprKind::Var(id) => match Self::lookup(cx, &id.name) {
                Some(l) => {
                    if !l.mutable {
                        let fix = if l.is_param {
                            "copy the parameter into a `let mut` local"
                        } else {
                            "declare it with `let mut`"
                        };
                        self.diags.push(
                            Diag::new(
                                "E0408",
                                target.span,
                                format!("`{}` is not mutable", id.name),
                            )
                            .fix(fix, None),
                        );
                    }
                    l.ty
                }
                None => {
                    self.unknown_local(cx, &id.name, id.span);
                    TypeKind::Error
                }
            },
            // An element of an array field or of an inner array: `s.w[i] = v`,
            // `p.q.w[i] = v` (backlog 2B-15), `g[i][j] = v`, `b.r[i].c[j] = v`
            // (2V-04b), or of a boxed array: `(*b)[i] = v` (2O-01). The base's type is recorded for the back ends
            // (assignment targets have none otherwise).
            ExprKind::Index { base, index }
                if matches!(
                    base.kind,
                    ExprKind::Field { .. } | ExprKind::Index { .. } | ExprKind::Deref(_)
                ) =>
            {
                let (bt, writable) = self.place_of(cx, base);
                self.types
                    .insert((base.span.file, base.span.lo, base.span.hi), bt.clone());
                let it = self.expr(cx, index, Some(&TypeKind::Usize), Mode::Exec);
                self.expect_ty(&it, &TypeKind::Usize, index.span);
                let Some(elem) = elem_of(&bt).cloned() else {
                    if bt != TypeKind::Error {
                        self.err(
                            "E0407",
                            base.span,
                            format!("type `{}` cannot be indexed", ty_str(&bt)),
                        );
                    }
                    return TypeKind::Error;
                };
                if !writable && bt != TypeKind::Error {
                    self.diags.push(
                        Diag::new("E0408", target.span, "this place cannot be assigned").fix(
                            "assign through a `let mut` local or a `&mut` parameter",
                            None,
                        ),
                    );
                }
                elem
            }
            ExprKind::Index { base, index } => {
                let ExprKind::Var(id) = &base.kind else {
                    self.err(
                        "E0408",
                        target.span,
                        "only elements of a variable, a field or an element can be assigned",
                    );
                    return TypeKind::Error;
                };
                let Some(l) = Self::lookup(cx, &id.name) else {
                    self.unknown_local(cx, &id.name, id.span);
                    return TypeKind::Error;
                };
                let writable = match &l.ty {
                    TypeKind::Array(..) | TypeKind::Vec(_) => l.mutable,
                    TypeKind::Ref { mutable, .. } => *mutable,
                    _ => false,
                };
                let it = self.expr(cx, index, Some(&TypeKind::Usize), Mode::Exec);
                self.expect_ty(&it, &TypeKind::Usize, index.span);
                let Some(elem) = elem_of(&l.ty).cloned() else {
                    self.err(
                        "E0407",
                        base.span,
                        format!(
                            "`{}` has type `{}` and cannot be indexed",
                            id.name,
                            ty_str(&l.ty)
                        ),
                    );
                    return TypeKind::Error;
                };
                if !writable {
                    self.err(
                        "E0408",
                        target.span,
                        format!(
                            "elements of `{}` cannot be assigned (`{}`)",
                            id.name,
                            ty_str(&l.ty)
                        ),
                    );
                }
                elem
            }
            ExprKind::Field { .. } | ExprKind::Deref(_) => {
                let (t, writable) = self.place_of(cx, target);
                if !writable && t != TypeKind::Error {
                    self.diags.push(
                        Diag::new("E0408", target.span, "this place cannot be assigned").fix(
                            "assign through a `let mut` local or a `&mut` parameter",
                            None,
                        ),
                    );
                }
                t
            }
            _ => {
                self.err("E0408", target.span, "invalid assignment target");
                TypeKind::Error
            }
        }
    }

    /// The type of a place under a field or `*` target, and whether it can be
    /// written: a `let mut` local, a `&mut` parameter, a writable element.
    fn place_of(&mut self, cx: &mut FnCtx, e: &Expr) -> (TypeKind, bool) {
        match &e.kind {
            ExprKind::Var(id) => match Self::lookup(cx, &id.name) {
                Some(l) => {
                    let w = match &l.ty {
                        TypeKind::Ref { mutable, .. } => *mutable,
                        _ => l.mutable,
                    };
                    (l.ty, w)
                }
                None => {
                    self.unknown_local(cx, &id.name, id.span);
                    (TypeKind::Error, false)
                }
            },
            ExprKind::Field { base, name } => {
                self.deref_field(e, base, name);
                let (bt, w) = self.place_of(cx, base);
                (self.field_type_of(&bt, name), w)
            }
            ExprKind::Deref(x) => {
                let (t, w) = self.place_of(cx, x);
                match t {
                    TypeKind::Ref { inner, .. } if !matches!(inner.kind, TypeKind::Slice(_)) => {
                        (inner.kind, w)
                    }
                    TypeKind::Box(inner) => (inner.kind, w),
                    TypeKind::Error => (TypeKind::Error, false),
                    other => {
                        self.err(
                            "E0401",
                            e.span,
                            format!(
                                "`*` needs a reference `&T` or `&mut T` or a `Box<T>`, found `{}`",
                                ty_str(&other)
                            ),
                        );
                        (TypeKind::Error, false)
                    }
                }
            }
            ExprKind::Index { base, index } => {
                let (bt, w) = self.place_of(cx, base);
                let it = self.expr(cx, index, Some(&TypeKind::Usize), Mode::Exec);
                self.expect_ty(&it, &TypeKind::Usize, index.span);
                match elem_of(&bt) {
                    Some(t) => (t.clone(), w),
                    None => {
                        if bt != TypeKind::Error {
                            self.err(
                                "E0407",
                                base.span,
                                format!("type `{}` cannot be indexed", ty_str(&bt)),
                            );
                        }
                        (TypeKind::Error, false)
                    }
                }
            }
            _ => {
                self.err("E0408", e.span, "invalid assignment target");
                (TypeKind::Error, false)
            }
        }
    }

    /// E0231: `(*p).x` is written `p.x` (fields project through references).
    fn deref_field(&mut self, e: &Expr, base: &Expr, name: &Ident) {
        if let ExprKind::Deref(x) = &base.kind {
            let r = format!("{}.{}", crate::fmt::expr_text(x), name.name);
            self.diags.push(
                Diag::new("E0231", e.span, format!("write `{r}`"))
                    .fix("fields are read through a reference directly", Some(r)),
            );
        }
    }

    /// The struct a type denotes (through one reference and any `Box`es:
    /// `p.x` on `p: &Point` or `p: &Box<Point>`).
    fn struct_fields(&self, t: &TypeKind) -> Option<(String, Vec<(String, TypeKind)>)> {
        let mut t = match t {
            TypeKind::Ref { inner, .. } => &inner.kind,
            t => t,
        };
        while let TypeKind::Box(inner) = t {
            t = &inner.kind;
        }
        let TypeKind::Named { path, .. } = t else {
            return None;
        };
        match self.adts.get(&path.join("::")) {
            Some(
                a @ Adt {
                    kind: AdtKind::Struct(fs),
                    path,
                    ..
                },
            ) => {
                let m = adt_subst(a, t);
                Some((
                    path.clone(),
                    fs.iter()
                        .map(|(n, ft)| (n.clone(), subst(ft, &m)))
                        .collect(),
                ))
            }
            _ => None,
        }
    }

    /// `base.name` for a struct (or a reference to one).
    fn field_type_of(&mut self, bt: &TypeKind, name: &Ident) -> TypeKind {
        if *bt == TypeKind::Error {
            return TypeKind::Error;
        }
        match self.struct_fields(bt) {
            Some((path, fs)) => match fs.iter().find(|(n, _)| *n == name.name) {
                Some((_, t)) => t.clone(),
                None => {
                    let mut d = Diag::new(
                        "E0306",
                        name.span,
                        format!("`{path}` has no field `{}`", name.name),
                    );
                    let names: Vec<String> = fs.iter().map(|(n, _)| n.clone()).collect();
                    if let Some(near) = closest(&name.name, names.iter()) {
                        d = d.fix(format!("did you mean `{near}`?"), Some(near));
                    } else {
                        d = d.fix(format!("fields of `{path}`: {}", names.join(", ")), None);
                    }
                    self.diags.push(d);
                    TypeKind::Error
                }
            },
            None => {
                let fix = if matches!(bt, TypeKind::Named { .. }) {
                    "enum values have no fields: `match` the variants"
                } else {
                    "only structs have fields"
                };
                self.diags.push(
                    Diag::new(
                        "E0306",
                        name.span,
                        format!("`{}` has no field `{}`", ty_str(bt), name.name),
                    )
                    .fix(fix, None),
                );
                TypeKind::Error
            }
        }
    }

    /// E0214 for a path whose first segment a `use` of unit `mp` imports:
    /// the fix writes the imported full path in its place.
    fn imported(&mut self, path: &[&str], span: Span, mp: &str) -> bool {
        let Some(text) = self.import_text(path, mp) else {
            return false;
        };
        self.diags.push(
            Diag::new(
                "E0214",
                span,
                format!("`{}` comes from a `use`", path.join("::")),
            )
            .fix(format!("write `{text}`"), Some(text)),
        );
        true
    }

    /// The full path for `path` when its first segment comes from a `use`
    /// of unit `mp`.
    fn import_text(&self, path: &[&str], mp: &str) -> Option<String> {
        let mut text = self.imports.get(mp)?.get(path[0])?.clone();
        for seg in &path[1..] {
            text.push_str("::");
            text.push_str(seg);
        }
        Some(text)
    }

    /// E0305 for a type name, with the nearest type of unit `mp` as the fix.
    fn unknown_type(&self, name: &str, span: Span, mp: &str) -> Diag {
        if name == "Self" {
            let d = Diag::new(
                "E0305",
                span,
                "`Self` names the implementing type only in a trait declaration",
            );
            return match &self.self_head {
                Some((_, t)) => d.fix(format!("write `{t}`"), Some(t.clone())),
                None => d.fix("write the type", None),
            };
        }
        let segs: Vec<&str> = name.split("::").collect();
        if let Some(text) = self.import_text(&segs, mp) {
            return Diag::new("E0214", span, format!("`{name}` comes from a `use`"))
                .fix(format!("write `{text}`"), Some(text));
        }
        let mut d = Diag::new("E0305", span, format!("unknown type `{name}`"));
        let local: Vec<String> = self
            .adts
            .keys()
            .filter_map(|k| k.strip_prefix(&format!("{mp}::")).map(String::from))
            .filter(|k| !k.contains("::"))
            .collect();
        if let Some(near) = closest(name, local.iter()) {
            d = d.fix(format!("did you mean `{near}`?"), Some(near));
        } else if !name.contains("::") {
            d = d.fix(
                "types of other units are named by full path, e.g. `unit::Point`",
                None,
            );
        }
        d
    }

    /// The full path of a type named by `segs` from unit `mp`.
    fn type_path(&self, segs: &[Ident], mp: &str) -> Option<String> {
        let name = segs
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let local = format!("{mp}::{name}");
        if segs.len() == 1 && self.adts.contains_key(&local) {
            return Some(local);
        }
        self.adts.contains_key(&name).then_some(name)
    }

    /// `T::V` / `unit::T::V`: (enum path, variant index, payload types).
    fn variant(&self, segs: &[Ident], mp: &str) -> Option<(String, usize, Vec<TypeKind>)> {
        let (v, ty) = segs.split_last()?;
        if ty.is_empty() {
            return None;
        }
        let tp = self.type_path(ty, mp)?;
        let AdtKind::Enum(vs) = &self.adts[&tp].kind else {
            return None;
        };
        let k = vs.iter().position(|(n, _)| *n == v.name)?;
        Some((tp, k, vs[k].1.clone()))
    }

    /// `Option::V` / `Result::V` for a built-in variant `V` of that type.
    fn builtin_path(segs: &[Ident]) -> bool {
        segs.len() == 2
            && matches!(
                (segs[0].name.as_str(), segs[1].name.as_str()),
                ("Option", "Some" | "None") | ("Result", "Ok" | "Err")
            )
    }

    /// E0221: the built-in variants are written bare (ADR-0026 §3).
    fn builtin_by_path(&mut self, segs: &[Ident]) {
        let v = &segs[1].name;
        self.diags.push(
            Diag::new(
                "E0221",
                segs[0].span.to(segs[1].span),
                format!("write `{v}`, not `{}::{v}`", segs[0].name),
            )
            .fix(format!("`{v}` is a built-in name"), Some(v.clone())),
        );
    }

    /// Whether `segs` without its last segment names an enum.
    fn enum_prefix(&self, segs: &[Ident], mp: &str) -> bool {
        segs.len() > 1
            && self
                .type_path(&segs[..segs.len() - 1], mp)
                .is_some_and(|tp| matches!(self.adts[&tp].kind, AdtKind::Enum(_)))
    }

    /// A variant value: `T::V` (no payload) or `T::V(a, ..)`.
    fn variant_value(
        &mut self,
        cx: &mut FnCtx,
        segs: &[Ident],
        args: &[Expr],
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let Some((tp, _, payload)) = self.variant(segs, &cx.mod_path) else {
            // Not a variant: an enum with no such variant, or an unknown type.
            let (v, ty) = segs.split_last().unwrap();
            match self.type_path(ty, &cx.mod_path) {
                Some(tp) => {
                    let names: Vec<String> = match &self.adts[&tp].kind {
                        AdtKind::Enum(vs) => vs.iter().map(|(n, _)| n.clone()).collect(),
                        AdtKind::Struct(_) => Vec::new(),
                    };
                    let mut d = Diag::new(
                        "E0307",
                        v.span,
                        format!("`{tp}` has no variant `{}`", v.name),
                    );
                    if let Some(near) = closest(&v.name, names.iter()) {
                        d = d.fix(format!("did you mean `{near}`?"), Some(near));
                    } else if names.is_empty() {
                        d = d.fix(
                            format!(
                                "`{tp}` is a struct: build it with `{} {{ .. }}`",
                                ty.last().unwrap().name
                            ),
                            None,
                        );
                    } else {
                        d = d.fix(format!("variants of `{tp}`: {}", names.join(", ")), None);
                    }
                    self.diags.push(d);
                }
                None => {
                    let name = ty
                        .iter()
                        .map(|i| i.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::");
                    let d = self.unknown_type(
                        &name,
                        ty[0].span.to(ty[ty.len() - 1].span),
                        &cx.mod_path,
                    );
                    self.diags.push(d);
                }
            }
            for a in args {
                self.expr(cx, a, None, mode);
            }
            return TypeKind::Error;
        };
        if args.len() != payload.len() {
            let tys: Vec<String> = payload.iter().map(ty_str).collect();
            self.diags.push(
                Diag::new(
                    "E0405",
                    span,
                    format!(
                        "`{}` takes {} value(s), found {}",
                        segs.last().unwrap().name,
                        payload.len(),
                        args.len()
                    ),
                )
                .fix(format!("the payload is `({})`", tys.join(", ")), None),
            );
        }
        let ps = self.adts[&tp].generics.clone();
        let mut tm = self.expected_args(&tp, expected);
        let slots: Vec<(&Expr, TypeKind)> = args.iter().zip(payload).collect();
        self.check_slots(cx, &ps, &mut tm, &slots, mode);
        if ps.is_empty() {
            return TypeKind::Named {
                path: tp.split("::").map(String::from).collect(),
                args: Vec::new(),
            };
        }
        let v = Self::path_text(segs);
        self.instance(&tp, &tm, &format!("`{v}`"), span)
    }

    /// `Some(x)`, `None`, `Ok(x)`, `Err(e)`: the type comes from the expected
    /// one (`Some(x)` alone is `Option<type of x>`).
    fn builtin_variant(
        &mut self,
        cx: &mut FnCtx,
        name: &str,
        args: &[Expr],
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let arity = usize::from(name != "None");
        if args.len() != arity {
            self.err(
                "E0405",
                span,
                format!("`{name}` takes {arity} value(s), found {}", args.len()),
            );
        }
        let want = match (name, expected) {
            ("Some" | "None", Some(TypeKind::Named { path, args: ta })) if path == &["Option"] => {
                Some((
                    expected.unwrap().clone(),
                    ta.first().map(|t| t.kind.clone()),
                ))
            }
            ("Ok" | "Err", Some(TypeKind::Named { path, args: ta })) if path == &["Result"] => {
                let k = usize::from(name == "Err");
                Some((expected.unwrap().clone(), ta.get(k).map(|t| t.kind.clone())))
            }
            _ => None,
        };
        match want {
            Some((whole, Some(inner))) => {
                for a in args {
                    let at = self.expr(cx, a, Some(&inner), mode);
                    self.expect_ty(&at, &inner, a.span);
                }
                whole
            }
            _ if name == "Some" && args.len() == 1 => {
                let at = self.expr(cx, &args[0], None, mode);
                if at == TypeKind::Error {
                    return TypeKind::Error;
                }
                TypeKind::Named {
                    path: vec!["Option".into()],
                    args: vec![Type {
                        kind: at,
                        span: Span::default(),
                    }],
                }
            }
            _ => {
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                self.diags.push(
                    Diag::new("E0424", span, format!("the type of `{name}` is not known here"))
                        .fix("give it a type: `let o: Option<i64> = None;`, or pass it where the type is written", None),
                );
                TypeKind::Error
            }
        }
    }

    /// The variants a `match` on a `t` value covers, in declaration order:
    /// (type path, variants). `Option` is `None, Some`, `Result` is `Ok, Err`.
    fn match_variants(&self, t: &TypeKind) -> Option<(String, Vec<VariantSig>)> {
        let TypeKind::Named { path, args } = t else {
            return None;
        };
        match (path.as_slice(), args.as_slice()) {
            ([o], [a]) if o == "Option" => Some((
                "Option".into(),
                vec![
                    ("None".into(), Vec::new()),
                    ("Some".into(), vec![a.kind.clone()]),
                ],
            )),
            ([r], [a, b]) if r == "Result" => Some((
                "Result".into(),
                vec![
                    ("Ok".into(), vec![a.kind.clone()]),
                    ("Err".into(), vec![b.kind.clone()]),
                ],
            )),
            _ => match self.adts.get(&path.join("::")) {
                Some(
                    a @ Adt {
                        kind: AdtKind::Enum(vs),
                        path,
                        ..
                    },
                ) => {
                    let m = adt_subst(a, t);
                    Some((
                        path.clone(),
                        vs.iter()
                            .map(|(n, ps)| (n.clone(), ps.iter().map(|p| subst(p, &m)).collect()))
                            .collect(),
                    ))
                }
                _ => None,
            },
        }
    }

    /// How a variant is written from unit `mp`: `Some`, `Shape::Dot`,
    /// `geo::Shape::Dot`.
    fn variant_text(tp: &str, v: &str, mp: &str) -> String {
        if tp == "Option" || tp == "Result" {
            return v.to_string();
        }
        match tp.strip_prefix(mp).and_then(|r| r.strip_prefix("::")) {
            Some(short) if !short.contains("::") => format!("{short}::{v}"),
            _ => format!("{tp}::{v}"),
        }
    }

    /// The variant index an arm pattern names, for a value of enum `tp`.
    fn arm_variant(
        &mut self,
        cx: &FnCtx,
        p: &Pat,
        tp: &str,
        vs: &[VariantSig],
        st: &TypeKind,
    ) -> Option<usize> {
        let names: Vec<String> = vs.iter().map(|(n, _)| n.clone()).collect();
        let (v, mut ty) = p.path.split_last().unwrap();
        if Self::builtin_path(&p.path) {
            self.builtin_by_path(&p.path);
            ty = &[];
        }
        let pt = if ty.is_empty() {
            match v.name.as_str() {
                "Some" | "None" => "Option".to_string(),
                "Ok" | "Err" => "Result".to_string(),
                _ => {
                    // `Dot` alone would bind everything in Rust.
                    let mut d = Diag::new(
                        "E0307",
                        v.span,
                        format!("`{}` is not a variant path", v.name),
                    );
                    if names.contains(&v.name) {
                        d = d.fix(
                            "a user enum variant is written with its type",
                            Some(Self::variant_text(tp, &v.name, &cx.mod_path)),
                        );
                    } else if let Some(near) = closest(&v.name, names.iter()) {
                        d = d.fix(
                            format!(
                                "did you mean `{}`?",
                                Self::variant_text(tp, &near, &cx.mod_path)
                            ),
                            Some(Self::variant_text(tp, &near, &cx.mod_path)),
                        );
                    } else {
                        d = d.fix("a variant pattern is `Type::Variant`", None);
                    }
                    self.diags.push(d);
                    return None;
                }
            }
        } else {
            match self.type_path(ty, &cx.mod_path) {
                Some(pt) => pt,
                None => {
                    let name = ty
                        .iter()
                        .map(|i| i.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::");
                    let d = self.unknown_type(
                        &name,
                        ty[0].span.to(ty[ty.len() - 1].span),
                        &cx.mod_path,
                    );
                    self.diags.push(d);
                    return None;
                }
            }
        };
        if pt != tp {
            let arms: Vec<String> = vs
                .iter()
                .map(|(n, _)| Self::variant_text(tp, n, &cx.mod_path))
                .collect();
            self.diags.push(
                Diag::new(
                    "E0401",
                    p.span,
                    format!(
                        "this pattern is a `{pt}` variant; the value is `{}`",
                        ty_str(st)
                    ),
                )
                .fix(
                    format!("the variants of `{}`: {}", ty_str(st), arms.join(", ")),
                    None,
                ),
            );
            return None;
        }
        match names.iter().position(|n| *n == v.name) {
            Some(k) => Some(k),
            None => {
                let mut d = Diag::new(
                    "E0307",
                    v.span,
                    format!("`{tp}` has no variant `{}`", v.name),
                );
                if let Some(near) = closest(&v.name, names.iter()) {
                    d = d.fix(format!("did you mean `{near}`?"), Some(near));
                } else {
                    d = d.fix(format!("variants of `{tp}`: {}", names.join(", ")), None);
                }
                self.diags.push(d);
                None
            }
        }
    }

    /// E0224 for a `_ =>` arm and E0225 for `if let`: the fix is the whole
    /// `match`, one arm per variant in declaration order, the wildcard's
    /// body on each variant it stood for.
    #[allow(clippy::too_many_arguments)]
    fn wildcard_arms(
        &mut self,
        cx: &FnCtx,
        scrut: &Expr,
        arms: &[Arm],
        kinds: &[Option<usize>],
        tp: &str,
        vs: &[VariantSig],
        span: Span,
    ) {
        let wild = arms.iter().position(|a| {
            a.pat.origin == PatOrigin::IfLetElse
                || (a.pat.origin == PatOrigin::Written
                    && a.pat.path.len() == 1
                    && a.pat.path[0].name == "_")
        });
        let Some(w) = wild else {
            return;
        };
        let if_let = arms[w].pat.origin == PatOrigin::IfLetElse;
        // Mechanical only when every other arm names its variant.
        let exact = arms
            .iter()
            .zip(kinds)
            .enumerate()
            .all(|(i, (a, k))| i == w || (k.is_some() && a.pat.origin == PatOrigin::Written));
        let text = exact.then(|| {
            let wild_body = crate::fmt::expr_text(&arms[w].body);
            let lines: Vec<String> = (0..vs.len())
                .map(|k| match kinds.iter().position(|x| *x == Some(k)) {
                    Some(i) => format!(
                        "{} => {}",
                        crate::fmt::pat_text(&arms[i].pat),
                        crate::fmt::expr_text(&arms[i].body)
                    ),
                    None => {
                        let vt = Self::variant_text(tp, &vs[k].0, &cx.mod_path);
                        let pat = if vs[k].1.is_empty() {
                            vt
                        } else {
                            format!("{vt}({})", vec!["_"; vs[k].1.len()].join(", "))
                        };
                        format!("{pat} => {wild_body}")
                    }
                })
                .collect();
            format!(
                "match {} {{ {} }}",
                crate::fmt::expr_text(scrut),
                lines.join(", ")
            )
        });
        let d = if if_let {
            Diag::new("E0225", span, "`if let` is not allowed")
        } else {
            Diag::new("E0224", span, "a wildcard arm is not allowed")
        };
        self.diags.push(d.fix(
            if if_let {
                "`match` with one arm per variant"
            } else {
                "one arm per variant, the wildcard's body on each it stood for"
            },
            text,
        ));
    }

    /// `match s { E::A => e0, E::B(x, _) => e1 }`: one arm per variant in
    /// declaration order, each payload position a new name or `_`, arms of
    /// one type (ADR-0026 §3).
    fn match_expr(
        &mut self,
        cx: &mut FnCtx,
        scrut: &Expr,
        arms: &[Arm],
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        // A reference matched (Rust's default binding mode, ADR-0055):
        // `match r` for `r: &E`, `match &place`; each binding is a
        // reference to its payload.
        let st = match &scrut.kind {
            ExprKind::Ref { mutable, expr: x } => {
                let t = self.expr(cx, x, None, mode);
                let rt = if t == TypeKind::Error {
                    t
                } else {
                    TypeKind::Ref {
                        mutable: *mutable,
                        inner: mk(t),
                    }
                };
                self.types
                    .insert((scrut.span.file, scrut.span.lo, scrut.span.hi), rt.clone());
                rt
            }
            _ => self.expr(cx, scrut, None, mode),
        };
        let (st, by_ref) = match &st {
            TypeKind::Ref { mutable, inner } if self.match_variants(&inner.kind).is_some() => {
                if *mutable {
                    self.diags.push(
                        Diag::new(
                            "E0207",
                            scrut.span,
                            "a `match` through `&mut` is not supported yet (backlog 2O-04)",
                        )
                        .fix(
                            "match a shared reference (`match &*r`) and write through the `&mut` parameter",
                            None,
                        ),
                    );
                }
                (inner.kind.clone(), true)
            }
            _ => (st, false),
        };
        let vs = self.match_variants(&st);
        if vs.is_none() && st != TypeKind::Error {
            let mut d = Diag::new(
                "E0423",
                scrut.span,
                format!("`match` needs an enum value, found `{}`", ty_str(&st)),
            );
            d = match (&st, &scrut.kind) {
                (TypeKind::Ref { inner, .. }, ExprKind::Var(v))
                    if self.match_variants(&inner.kind).is_some() =>
                {
                    d.fix(
                        "match the value the reference points to",
                        Some(format!("*{}", v.name)),
                    )
                }
                _ => d.fix("branch with `if` on a condition", None),
            };
            self.diags.push(d);
        }
        // The variant of each arm (None: wildcard or unresolved) and the
        // types its bindings get.
        let mut kinds: Vec<Option<usize>> = Vec::new();
        let mut bind_tys: Vec<Vec<TypeKind>> = Vec::new();
        let mut wildcard = false;
        let mut unresolved = false;
        for a in arms {
            let p = &a.pat;
            // A wildcard (E0224), the `else` of an `if let` (E0225) or a
            // pattern the parser rejected (E0232): covers the rest.
            if p.origin != PatOrigin::Written || (p.path.len() == 1 && p.path[0].name == "_") {
                wildcard = true;
                kinds.push(None);
                bind_tys.push(Vec::new());
                continue;
            }
            let Some((tp, vs)) = &vs else {
                kinds.push(None);
                bind_tys.push(Vec::new());
                continue;
            };
            let k = self.arm_variant(cx, p, tp, vs, &st);
            let mut tys = Vec::new();
            match k {
                Some(k) => {
                    let payload = &vs[k].1;
                    let vt = Self::variant_text(tp, &vs[k].0, &cx.mod_path);
                    if payload.is_empty() && p.parens {
                        self.diags.push(
                            Diag::new("E0427", p.span, format!("`{vt}` has no payload"))
                                .fix(format!("write `{vt}`"), Some(vt.clone())),
                        );
                    } else if p.binds.len() != payload.len() {
                        let holes = vec!["_"; payload.len()].join(", ");
                        self.diags.push(
                            Diag::new(
                                "E0427",
                                p.span,
                                format!(
                                    "`{vt}` holds {} value(s), the pattern names {}",
                                    payload.len(),
                                    p.binds.len()
                                ),
                            )
                            .fix(
                                "one new name or `_` per payload position",
                                Some(format!("{vt}({holes})")),
                            ),
                        );
                    }
                    tys = if by_ref {
                        payload
                            .iter()
                            .map(|t| TypeKind::Ref {
                                mutable: false,
                                inner: mk(t.clone()),
                            })
                            .collect()
                    } else {
                        payload.clone()
                    };
                }
                None => unresolved = true,
            }
            kinds.push(k);
            bind_tys.push(tys);
        }
        if let Some((tp, vs)) = &vs {
            let mut seen = BTreeSet::new();
            let mut dup = false;
            for (a, k) in arms.iter().zip(&kinds) {
                if let Some(k) = k
                    && !seen.insert(*k)
                {
                    dup = true;
                    let vt = Self::variant_text(tp, &vs[*k].0, &cx.mod_path);
                    self.diags.push(
                        Diag::new("E0426", a.pat.span, format!("a second arm for `{vt}`"))
                            .fix("one arm per variant: merge the two", None),
                    );
                }
            }
            let missing: Vec<String> = (0..vs.len())
                .filter(|k| !seen.contains(k))
                .map(|k| {
                    let vt = Self::variant_text(tp, &vs[k].0, &cx.mod_path);
                    if vs[k].1.is_empty() {
                        vt
                    } else {
                        format!("{vt}({})", vec!["_"; vs[k].1.len()].join(", "))
                    }
                })
                .collect();
            self.wildcard_arms(cx, scrut, arms, &kinds, tp, vs, span);
            if !wildcard && !unresolved && !missing.is_empty() {
                self.diags.push(
                    Diag::new(
                        "E0426",
                        span,
                        format!("`match` misses `{}`", missing.join("`, `")),
                    )
                    .fix(
                        format!("add an arm for each: `{} => ..`", missing.join(" => .., ")),
                        None,
                    ),
                );
            }
            let ks: Vec<usize> = kinds.iter().flatten().copied().collect();
            if !dup && ks.windows(2).any(|w| w[0] > w[1]) {
                let at = kinds
                    .iter()
                    .zip(arms)
                    .scan(None, |prev: &mut Option<usize>, (k, a)| {
                        let out = matches!((*prev, k), (Some(p), Some(k)) if p > *k)
                            .then_some(a.pat.span);
                        if k.is_some() {
                            *prev = *k;
                        }
                        Some(out)
                    })
                    .flatten()
                    .next()
                    .unwrap_or(span);
                let order: Vec<String> = vs
                    .iter()
                    .map(|(n, _)| Self::variant_text(tp, n, &cx.mod_path))
                    .collect();
                self.diags.push(
                    Diag::new("E0426", at, "`match` arms are not in declaration order").fix(
                        format!("order the arms as declared: {}", order.join(", ")),
                        None,
                    ),
                );
            }
        }
        // A literal arm takes its type from the others, as in `if`.
        let first = if expected.is_none() {
            arms.iter()
                .position(|a| !untyped_literal(&a.body))
                .unwrap_or(0)
        } else {
            0
        };
        let order = std::iter::once(first).chain((0..arms.len()).filter(|&i| i != first));
        // A known expected type is the type of every arm.
        let mut ty: Option<TypeKind> = expected.filter(|t| **t != TypeKind::Error).cloned();
        for i in order.take(arms.len()) {
            let a = &arms[i];
            cx.scopes.push(BTreeMap::new());
            for (pos, b) in a.pat.binds.iter().enumerate() {
                if let Some(id) = b {
                    let t = bind_tys[i].get(pos).cloned().unwrap_or(TypeKind::Error);
                    self.declare(cx, id, t, false, false);
                }
            }
            let hint = ty.as_ref().filter(|t| **t != TypeKind::Error).or(expected);
            let bt = self.expr(cx, &a.body, hint, mode);
            match &ty {
                Some(t) => self.expect_ty(&bt, t, a.body.span),
                None => ty = Some(bt),
            }
            cx.scopes.pop();
        }
        ty.unwrap_or(TypeKind::Error)
    }

    /// `T { f: e, .. }`: every field once.
    /// The bindings of a generic data type's parameters from the expected
    /// type, when it is an instance of that type.
    fn expected_args(&self, tp: &str, expected: Option<&TypeKind>) -> BTreeMap<String, TypeKind> {
        match expected {
            Some(t @ TypeKind::Named { path, .. }) if path.join("::") == tp => {
                adt_subst(&self.adts[tp], t)
            }
            _ => BTreeMap::new(),
        }
    }

    /// Checks the field or payload values `xs` against their declared types,
    /// which may name the parameters `ps` of a generic data type: a
    /// parameter `tm` does not bind yet is bound from the value (integer
    /// literals last, so another value fixes their type).
    fn check_slots(
        &mut self,
        cx: &mut FnCtx,
        ps: &[TParam],
        tm: &mut BTreeMap<String, TypeKind>,
        xs: &[(&Expr, TypeKind)],
        mode: Mode,
    ) {
        let mut deferred = Vec::new();
        for (x, t) in xs {
            if has_param_of(t, ps, tm) {
                if untyped_literal(x) {
                    deferred.push((*x, t));
                    continue;
                }
                let xt = self.expr(cx, x, None, mode);
                unify(t, &xt, ps, tm);
                let want = subst(t, tm);
                if !has_param_of(t, ps, tm) {
                    self.expect_ty(&xt, &want, x.span);
                }
            } else {
                let want = subst(t, tm);
                let xt = self.expr(cx, x, Some(&want), mode);
                self.expect_ty(&xt, &want, x.span);
            }
        }
        for (x, t) in deferred {
            if has_param_of(t, ps, tm) {
                self.expr(cx, x, None, mode);
                continue;
            }
            let want = subst(t, tm);
            let xt = self.expr(cx, x, Some(&want), mode);
            self.expect_ty(&xt, &want, x.span);
        }
    }

    /// The instance of generic data type `tp` the bindings `tm` give, or
    /// E0424 when a parameter is still unknown.
    fn instance(
        &mut self,
        tp: &str,
        tm: &BTreeMap<String, TypeKind>,
        what: &str,
        span: Span,
    ) -> TypeKind {
        let ps = self.adts[tp].generics.clone();
        let path: Vec<String> = tp.split("::").map(String::from).collect();
        if ps.iter().any(|p| !tm.contains_key(&p.name)) {
            self.diags.push(
                Diag::new("E0424", span, format!("the type of {what} is not known here")).fix(
                    format!(
                        "give it a type: `let x: {}<..> = ..;`, or pass it where the type is written",
                        path.last().unwrap()
                    ),
                    None,
                ),
            );
            return TypeKind::Error;
        }
        TypeKind::Named {
            path,
            args: ps
                .iter()
                .map(|p| Type {
                    kind: tm[&p.name].clone(),
                    span: Span::default(),
                })
                .collect(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn struct_lit(
        &mut self,
        cx: &mut FnCtx,
        path: &[Ident],
        fields: &[(Ident, Expr)],
        rest: Option<&Expr>,
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let name = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let decl = match self.type_path(path, &cx.mod_path) {
            Some(tp) => match &self.adts[&tp].kind {
                AdtKind::Struct(fs) => Some((tp.clone(), fs.clone())),
                AdtKind::Enum(_) => {
                    self.diags.push(
                        Diag::new("E0422", span, format!("`{name}` is an enum, not a struct"))
                            .fix(format!("build a variant: `{name}::V(..)`"), None),
                    );
                    None
                }
            },
            None => {
                let d = self.unknown_type(
                    &name,
                    path[0].span.to(path[path.len() - 1].span),
                    &cx.mod_path,
                );
                self.diags.push(d);
                None
            }
        };
        let Some((tp, fs)) = decl else {
            for (_, x) in fields {
                self.expr(cx, x, None, mode);
            }
            return TypeKind::Error;
        };
        let ps = self.adts[&tp].generics.clone();
        let mut tm = self.expected_args(&tp, expected);
        let mut slots = Vec::new();
        let mut given = BTreeSet::new();
        for (f, x) in fields {
            if !given.insert(f.name.clone()) {
                self.diags.push(
                    Diag::new(
                        "E0422",
                        f.span,
                        format!("field `{}` is given more than once", f.name),
                    )
                    .fix("give each field once", None),
                );
            }
            match fs.iter().find(|(n, _)| *n == f.name) {
                Some((_, t)) => slots.push((x, t.clone())),
                None => {
                    let names: Vec<String> = fs.iter().map(|(n, _)| n.clone()).collect();
                    let mut d =
                        Diag::new("E0306", f.span, format!("`{tp}` has no field `{}`", f.name));
                    if let Some(near) = closest(&f.name, names.iter()) {
                        d = d.fix(format!("did you mean `{near}`?"), Some(near));
                    } else {
                        d = d.fix(format!("fields of `{tp}`: {}", names.join(", ")), None);
                    }
                    self.diags.push(d);
                    self.expr(cx, x, None, mode);
                }
            }
        }
        self.check_slots(cx, &ps, &mut tm, &slots, mode);
        let whole = if ps.is_empty() {
            TypeKind::Named {
                path: tp.split("::").map(String::from).collect(),
                args: Vec::new(),
            }
        } else {
            let name = path.last().unwrap().name.clone();
            self.instance(&tp, &tm, &format!("`{name} {{ .. }}`"), span)
        };
        let missing: Vec<&str> = fs
            .iter()
            .map(|(n, _)| n.as_str())
            .filter(|n| !given.contains(*n))
            .collect();
        if let Some(r) = rest {
            // Rust habit: `..base`. Every field is written out instead.
            let want = whole.clone();
            let rt = self.expr(cx, r, Some(&want), mode);
            self.expect_ty(&rt, &want, r.span);
            // The replacement is the whole literal, so the span is too.
            let mut d = Diag::new("E0223", span, "struct update syntax `..` is not allowed");
            d = if matches!(r.kind, ExprKind::Var(_) | ExprKind::Field { .. }) {
                let base = crate::fmt::expr_text(r);
                let all: Vec<String> = fs
                    .iter()
                    .map(|(n, _)| match fields.iter().find(|(f, _)| f.name == *n) {
                        Some((_, x)) => format!("{n}: {}", crate::fmt::expr_text(x)),
                        None => format!("{n}: {base}.{n}"),
                    })
                    .collect();
                d.fix(
                    "write every field",
                    Some(format!("{name} {{ {} }}", all.join(", "))),
                )
            } else {
                d.fix("write every field: `f: base.f`", None)
            };
            self.diags.push(d);
        } else if !missing.is_empty() {
            self.diags.push(
                Diag::new(
                    "E0422",
                    span,
                    format!("missing field(s) `{}` of `{tp}`", missing.join("`, `")),
                )
                .fix(
                    "give every field a value (ADR-0026: no `..` update syntax)",
                    None,
                ),
            );
        }
        whole
    }

    // ---------- expressions ----------

    fn expr(
        &mut self,
        cx: &mut FnCtx,
        e: &Expr,
        expected: Option<&TypeKind>,
        mode: Mode,
    ) -> TypeKind {
        let t = self.expr_inner(cx, e, expected, mode);
        self.types
            .insert((e.span.file, e.span.lo, e.span.hi), t.clone());
        t
    }

    fn expr_inner(
        &mut self,
        cx: &mut FnCtx,
        e: &Expr,
        expected: Option<&TypeKind>,
        mode: Mode,
    ) -> TypeKind {
        match &e.kind {
            ExprKind::Error => TypeKind::Error,
            ExprKind::Path(path) if int_const(path).is_some() => {
                int_const(path).expect("checked").0
            }
            ExprKind::Path(path) if path.len() == 2 && int_type(&path[0].name).is_some() => {
                self.diags.push(
                    Diag::new(
                        "E0305",
                        e.span,
                        format!("`{}` has no constant `{}`", path[0].name, path[1].name),
                    )
                    .fix(
                        format!(
                            "the integer constants are `{0}::MIN` and `{0}::MAX`",
                            path[0].name
                        ),
                        None,
                    ),
                );
                TypeKind::Error
            }
            // A function named without arguments, not a variant (variants are
            // capitalised, ADR-0026).
            ExprKind::Path(path)
                if self.resolve(path, &cx.mod_path).is_some()
                    || path
                        .last()
                        .is_some_and(|i| i.name.starts_with(|c: char| c.is_ascii_lowercase())) =>
            {
                self.err("E0301", e.span, "a path must be followed by call arguments");
                TypeKind::Error
            }
            ExprKind::StructLit { path, fields, rest } => {
                self.struct_lit(cx, path, fields, rest.as_deref(), expected, e.span, mode)
            }
            ExprKind::Path(path) if Self::builtin_path(path) => {
                self.builtin_by_path(path);
                let v = &path[1].name;
                self.builtin_variant(cx, v, &[], expected, e.span, mode)
            }
            ExprKind::Path(path) => self.variant_value(cx, path, &[], expected, e.span, mode),
            ExprKind::Field { base, name } => {
                self.deref_field(e, base, name);
                let bt = self.expr(cx, base, None, mode);
                self.field_type_of(&bt, name)
            }
            ExprKind::Deref(x) => {
                let t = self.expr(cx, x, None, mode);
                match t {
                    TypeKind::Ref { inner, .. } if !matches!(inner.kind, TypeKind::Slice(_)) => {
                        inner.kind
                    }
                    TypeKind::Box(inner) => inner.kind,
                    TypeKind::Error => TypeKind::Error,
                    other => {
                        self.err(
                            "E0401",
                            e.span,
                            format!(
                                "`*` needs a reference `&T` or `&mut T` or a `Box<T>`, found `{}`",
                                ty_str(&other)
                            ),
                        );
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Match { scrut, arms } => {
                self.match_expr(cx, scrut, arms, expected, e.span, mode)
            }
            ExprKind::Unit => TypeKind::Unit,
            ExprKind::Bool(_) => TypeKind::Bool,
            ExprKind::Int(v) => self.int_lit(*v, false, expected, e.span, mode),
            ExprKind::ByteLit(_) => TypeKind::U8,
            // A `&[u8]` is a reference: second-class (ADR-0015, ADR-0044 §2).
            ExprKind::ByteStr(_) => {
                self.diags.push(
                    Diag::new(
                        "E0409",
                        e.span,
                        "a byte string may only appear directly as a call argument",
                    )
                    .fix(
                        "pass it to a `&[u8]` parameter, e.g. `bytes::extend(&mut v, b\"..\")`",
                        None,
                    ),
                );
                TypeKind::Error
            }
            ExprKind::Var(id) if id.name == "None" && Self::lookup(cx, "None").is_none() => {
                self.builtin_variant(cx, "None", &[], expected, e.span, mode)
            }
            ExprKind::Var(id) => match Self::lookup(cx, &id.name) {
                Some(l) => l.ty,
                None => {
                    if id.name == "result" {
                        self.diags.push(
                            Diag::new("E0301", id.span, "unknown name `result`")
                                .fix("name the result: `-> (r: T)` and use `r`", None),
                        );
                    } else {
                        self.unknown_local(cx, &id.name, id.span);
                    }
                    TypeKind::Error
                }
            },
            ExprKind::Unary(UnOp::Neg, inner) => {
                if let ExprKind::Int(v) = inner.kind {
                    return self.int_lit(v, true, expected, e.span, mode);
                }
                let t = self.expr(cx, inner, expected, mode);
                match t {
                    TypeKind::I64 | TypeKind::Error => t,
                    // Specification integers are mathematical: `-x` of an
                    // unsigned value is just an integer (ADR-0024).
                    TypeKind::U64 | TypeKind::Usize | TypeKind::U8 if mode != Mode::Exec => {
                        TypeKind::I64
                    }
                    TypeKind::U64 | TypeKind::Usize | TypeKind::U8 => {
                        self.err(
                            "E0411",
                            e.span,
                            format!("cannot negate a value of type `{}`", ty_str(&t)),
                        );
                        TypeKind::Error
                    }
                    _ => {
                        self.err("E0401", e.span, format!("cannot negate `{}`", ty_str(&t)));
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Unary(UnOp::Not, inner) => {
                let t = self.expr(cx, inner, Some(&TypeKind::Bool), mode);
                self.expect_ty(&t, &TypeKind::Bool, inner.span);
                TypeKind::Bool
            }
            ExprKind::Binary(op, l, r) => self.binary(cx, *op, l, r, expected, mode, e.span),
            ExprKind::Index { base, index } => {
                let bt = self.expr(cx, base, None, mode);
                let it = self.expr(cx, index, Some(&TypeKind::Usize), mode);
                self.expect_ty(&it, &TypeKind::Usize, index.span);
                match elem_of(&bt) {
                    Some(t) => t.clone(),
                    None => {
                        if bt != TypeKind::Error {
                            self.err(
                                "E0407",
                                base.span,
                                format!("type `{}` cannot be indexed", ty_str(&bt)),
                            );
                        }
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Method { recv, name, args } if conv_call(recv).is_some() => {
                self.conversion(cx, recv, name, args, e.span, mode)
            }
            ExprKind::Method { recv, name, args } => {
                self.method(cx, recv, name, args, e.span, mode)
            }
            // A conversion without `.unwrap()` / `.ok()` (ADR-0028 §1), and
            // `T::from(x)` (E0212).
            ExprKind::Call { path, args, .. }
                if conv_call(e).is_some() || from_call(path).is_some() =>
            {
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                let x = args.first().map(crate::fmt::expr_text).unwrap_or_default();
                if let Some(t) = from_call(path) {
                    let r = format!("{}::try_from({x}).unwrap()", ty_str(&t));
                    self.diags
                        .push(Diag::new("E0212", e.span, format!("write `{r}`")).fix(
                            "an integer conversion has a range obligation (ADR-0028)",
                            Some(r),
                        ));
                    return t;
                }
                let (t, _) = conv_call(e).unwrap();
                let r = format!("{}::try_from({x}).unwrap()", ty_str(&t));
                self.diags.push(
                    Diag::new(
                        "E0412",
                        e.span,
                        "`T::try_from(x)` is used as `.unwrap()` or `.ok()`",
                    )
                    .fix(
                        "`.unwrap()`: the value, with a range obligation; `.ok()`: an `Option`",
                        Some(r),
                    ),
                );
                TypeKind::Error
            }
            // Rust habit: `Option::Some(x)`, `Result::Err(e)`, `Option::None`.
            ExprKind::Call { path, args, .. } if Self::builtin_path(path) => {
                self.builtin_by_path(path);
                let v = &path[1].name;
                self.builtin_variant(cx, v, args, expected, e.span, mode)
            }
            // The built-in variants of `Option` and `Result` (ADR-0026).
            ExprKind::Call { path, args, .. }
                if path.len() == 1 && matches!(path[0].name.as_str(), "Some" | "Ok" | "Err") =>
            {
                self.builtin_variant(cx, &path[0].name, args, expected, e.span, mode)
            }
            // `E::V(..)`: a variant constructor (no function of that name
            // and `E` an enum; a missing one is an unknown variant).
            ExprKind::Call { path, args, .. }
                if self.resolve(path, &cx.mod_path).is_none()
                    && self.enum_prefix(path, &cx.mod_path) =>
            {
                self.variant_value(cx, path, args, expected, e.span, mode)
            }
            ExprKind::Call { path, args, .. }
                if self.resolve(path, &cx.mod_path).is_none() && is_box_new(path) =>
            {
                self.box_new(cx, args, expected, e.span, mode)
            }
            ExprKind::Call { path, args, .. }
                if self.resolve(path, &cx.mod_path).is_none() && vec_builtin(path) =>
            {
                self.vec_call(cx, path, args, expected, e.span, mode)
            }
            ExprKind::Call { path, targs, args } => {
                self.call(cx, path, targs, args, expected, e.span, mode)
            }
            ExprKind::Ref { .. } => {
                self.diags.push(Diag::new(
                    "E0409",
                    e.span,
                    "`&` / `&mut` may only appear directly as a call argument",
                ));
                TypeKind::Error
            }
            ExprKind::If { cond, then, els } => {
                let ct = self.expr(cx, cond, Some(&TypeKind::Bool), mode);
                self.expect_ty(&ct, &TypeKind::Bool, cond.span);
                // `if c { 5 } else { y }`: the literal branch takes its type
                // from the other one, as in Rust.
                let then_lit =
                    then.stmts.is_empty() && then.tail.as_ref().is_some_and(|t| untyped_literal(t));
                if expected.is_none()
                    && then_lit
                    && let Some(el) = els
                    && !untyped_literal(el)
                {
                    let et = self.expr(cx, el, None, mode);
                    let want = Some(&et).filter(|t| is_int(t));
                    let tt = self.block(cx, then, want, mode);
                    self.expect_ty(&et, &tt, el.span);
                    return tt;
                }
                let tt = self.block(cx, then, expected, mode);
                match els {
                    Some(el) => {
                        let et = self.expr(
                            cx,
                            el,
                            Some(&tt).filter(|t| **t != TypeKind::Error).or(expected),
                            mode,
                        );
                        self.expect_ty(&et, &tt, el.span);
                        tt
                    }
                    None => {
                        self.expect_ty(&tt, &TypeKind::Unit, then.span);
                        TypeKind::Unit
                    }
                }
            }
            ExprKind::Block(b) => self.block(cx, b, expected, mode),
            ExprKind::Array(items) => {
                let want_elem = match expected {
                    Some(TypeKind::Array(el, _)) => Some(el.kind.clone()),
                    _ => None,
                };
                let mut elem = want_elem.clone();
                for it in items {
                    let t = self.expr(cx, it, elem.as_ref(), mode);
                    self.literal_elem(&t, it.span);
                    match &elem {
                        None => elem = Some(t),
                        Some(want) => self.expect_ty(&t, want, it.span),
                    }
                }
                match elem {
                    Some(el) => TypeKind::Array(mk(el), items.len() as u64),
                    None => {
                        self.err(
                            "E0401",
                            e.span,
                            "cannot determine the element type of an empty array",
                        );
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Repeat {
                value,
                count,
                count_span,
            } => {
                let want_elem = match expected {
                    Some(TypeKind::Array(el, _)) => Some(el.kind.clone()),
                    _ => None,
                };
                let t = self.expr(cx, value, want_elem.as_ref(), mode);
                self.literal_elem(&t, value.span);
                if *count > u32::MAX as u128 {
                    self.err("E0406", *count_span, "array length is too large");
                }
                TypeKind::Array(mk(t), *count as u64)
            }
            ExprKind::Old(inner) => {
                let ok_mode = matches!(mode, Mode::Spec { allow_old: true });
                let var_ty = match &inner.kind {
                    ExprKind::Var(id) => Self::lookup(cx, &id.name),
                    _ => None,
                };
                match var_ty {
                    Some(Local {
                        ty: ty @ TypeKind::Ref { mutable: true, .. },
                        is_param: true,
                        ..
                    }) if ok_mode => ty,
                    _ => {
                        let code = if mode == Mode::Exec { "E0503" } else { "E0508" };
                        self.err(
                            code,
                            e.span,
                            "`old(x)` is only allowed in `ensures`, loop invariants and `assert`, on a `&mut` parameter `x`",
                        );
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Forall { binders, body } => {
                if mode == Mode::Exec {
                    self.err(
                        "E0503",
                        e.span,
                        "`forall` is only allowed in specifications",
                    );
                    return TypeKind::Error;
                }
                cx.scopes.push(BTreeMap::new());
                for b in binders {
                    if !is_int(&b.ty.kind) && b.ty.kind != TypeKind::Error {
                        self.err("E0402", b.ty.span, "`forall` binders must be integers");
                    }
                    self.declare(cx, &b.name, b.ty.kind.clone(), false, false);
                }
                let mut triggers = Vec::new();
                collect_triggers(body, &mut triggers);
                if triggers.is_empty() {
                    self.diags.push(
                        Diag::new("E0511", e.span, "`forall` has no `#[trigger]`").fix(
                            "mark a term that mentions every bound variable, e.g. `#[trigger] a[i]`",
                            None,
                        ),
                    );
                }
                // All triggers of one `forall` form a single multi-pattern.
                if !triggers.is_empty() {
                    for b in binders {
                        if !triggers.iter().any(|t| mentions_strict(t, &b.name.name)) {
                            self.err(
                                "E0512",
                                b.name.span,
                                format!("no trigger mentions bound variable `{}`", b.name.name),
                            );
                        }
                    }
                }
                cx.forall_depth += 1;
                let t = self.expr(cx, body, Some(&TypeKind::Bool), mode);
                cx.forall_depth -= 1;
                cx.scopes.pop();
                self.expect_ty(&t, &TypeKind::Bool, body.span);
                TypeKind::Bool
            }
            ExprKind::Trigger(inner) => {
                if mode == Mode::Exec {
                    self.err(
                        "E0503",
                        e.span,
                        "`#[trigger]` is only allowed in specifications",
                    );
                } else if cx.forall_depth == 0 {
                    self.diags.push(
                        Diag::new("E0510", e.span, "`#[trigger]` outside `forall`").fix(
                            "remove `#[trigger]`, or move the term into a `forall` body",
                            None,
                        ),
                    );
                }
                self.expr(cx, inner, expected, mode)
            }
        }
    }

    fn int_lit(
        &mut self,
        v: u128,
        negative: bool,
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let t = match expected {
            // A negative literal in a specification is just an integer (ADR-0022).
            Some(TypeKind::U64 | TypeKind::Usize | TypeKind::U8)
                if negative && mode != Mode::Exec =>
            {
                TypeKind::I64
            }
            Some(t) if is_int(t) => t.clone(),
            _ => TypeKind::I64,
        };
        let fits = match t {
            TypeKind::I64 => {
                if negative {
                    v <= 1u128 << 63
                } else {
                    v <= i64::MAX as u128
                }
            }
            _ => {
                if negative && v != 0 {
                    self.err(
                        "E0411",
                        span,
                        format!("cannot negate a value of type `{}`", ty_str(&t)),
                    );
                    return TypeKind::Error;
                }
                let max = if t == TypeKind::U8 {
                    255
                } else {
                    u64::MAX as u128
                };
                v <= max
            }
        };
        // Specification integers are unbounded (ADR-0017, ADR-0024).
        if !fits && mode == Mode::Exec {
            self.err(
                "E0406",
                span,
                format!("literal does not fit in `{}`", ty_str(&t)),
            );
        }
        t
    }

    #[allow(clippy::too_many_arguments)]
    fn binary(
        &mut self,
        cx: &mut FnCtx,
        op: BinOp,
        l: &Expr,
        r: &Expr,
        expected: Option<&TypeKind>,
        mode: Mode,
        span: Span,
    ) -> TypeKind {
        match op {
            BinOp::And | BinOp::Or | BinOp::Implies => {
                if op == BinOp::Implies && mode == Mode::Exec {
                    self.err("E0503", span, "`==>` is only allowed in specifications");
                }
                let lt = self.expr(cx, l, Some(&TypeKind::Bool), mode);
                let rt = self.expr(cx, r, Some(&TypeKind::Bool), mode);
                self.expect_ty(&lt, &TypeKind::Bool, l.span);
                self.expect_ty(&rt, &TypeKind::Bool, r.span);
                TypeKind::Bool
            }
            _ => {
                // Literals take their type from the other operand.
                let lit = untyped_literal;
                let hint = if op.is_arith() {
                    expected.filter(|t| is_int(t))
                } else {
                    None
                };
                let (lt, rt) = if lit(l) && !lit(r) {
                    let rt = self.expr(cx, r, hint, mode);
                    let lt = self.expr(cx, l, Some(&rt), mode);
                    (lt, rt)
                } else {
                    let lt = self.expr(cx, l, hint, mode);
                    let rt = self.expr(cx, r, Some(&lt), mode);
                    (lt, rt)
                };
                if lt == TypeKind::Error || rt == TypeKind::Error {
                    return if op.is_arith() {
                        TypeKind::Error
                    } else {
                        TypeKind::Bool
                    };
                }
                // Specifications compute over unbounded integers (ADR-0017 §2),
                // so integer types may mix there (ADR-0022).
                if lt != rt && mode != Mode::Exec && is_int(&lt) && is_int(&rt) {
                    return if op.is_arith() {
                        TypeKind::I64
                    } else {
                        TypeKind::Bool
                    };
                }
                if lt != rt {
                    self.err(
                        "E0401",
                        span,
                        format!(
                            "`{}` needs operands of one type, found `{}` and `{}`",
                            op.symbol(),
                            ty_str(&lt),
                            ty_str(&rt)
                        ),
                    );
                    return if op.is_arith() {
                        TypeKind::Error
                    } else {
                        TypeKind::Bool
                    };
                }
                // A `Vec`'s array goes on past its length, so array equality
                // is not `Vec` equality (ADR-0031 §3).
                if matches!(op, BinOp::Eq | BinOp::Ne) && matches!(lt, TypeKind::Vec(_)) {
                    self.diags.push(
                        Diag::new(
                            "E0514",
                            span,
                            format!("`{}` is not defined for `{}`", op.symbol(), ty_str(&lt)),
                        )
                        .fix(
                            "compare the lengths and the elements with a spec predicate over `v.as_slice()`",
                            None,
                        ),
                    );
                    return TypeKind::Bool;
                }
                // Structural `==` on data types, arrays and slices is a
                // specification term only (ADR-0026 §2; elements below the
                // length, backlog 2V-05).
                let slice = matches!(&lt, TypeKind::Ref { inner, .. } if matches!(inner.kind, TypeKind::Slice(_)));
                // A type parameter: equality in specifications (ADR-0046 §4);
                // in code it needs the bound `Eq` (traits, backlog 2T).
                if let TypeKind::Param(n) = &lt
                    && op.is_cmp()
                    && !matches!(op, BinOp::Eq | BinOp::Ne)
                {
                    if !self.has_builtin(&lt, "Ord") {
                        self.diags.push(
                            Diag::new(
                                "E0428",
                                span,
                                format!("`{}` on `{n}` needs the bound `{n}: Ord`", op.symbol()),
                            )
                            .fix(format!("add `Ord` to the bounds of `{n}`"), None),
                        );
                    }
                    return TypeKind::Bool;
                }
                if matches!(op, BinOp::Eq | BinOp::Ne) && matches!(lt, TypeKind::Param(_)) {
                    if mode == Mode::Exec && !self.has_builtin(&lt, "Eq") {
                        self.diags.push(
                            Diag::new(
                                "E0428",
                                span,
                                format!(
                                    "`{}` on `{}` in code needs the bound `{}: Eq`",
                                    op.symbol(),
                                    ty_str(&lt),
                                    ty_str(&lt)
                                ),
                            )
                            .fix(
                                format!("add `Eq` (or `Ord`) to the bounds of `{}`", ty_str(&lt)),
                                None,
                            ),
                        );
                    }
                    return TypeKind::Bool;
                }
                if matches!(op, BinOp::Eq | BinOp::Ne)
                    && (matches!(
                        lt,
                        TypeKind::Named { .. } | TypeKind::Array(..) | TypeKind::Box(_)
                    ) || slice)
                {
                    if mode == Mode::Exec {
                        let fix = if matches!(lt, TypeKind::Box(_)) {
                            format!(
                                "compare the values: `*{} {} *{}`",
                                crate::fmt::expr_text(l),
                                op.symbol(),
                                crate::fmt::expr_text(r)
                            )
                        } else {
                            "compare the fields (or `match` the variants) in code".into()
                        };
                        self.diags.push(
                            Diag::new(
                                "E0425",
                                span,
                                format!(
                                    "`{}` on `{}` is only defined in specifications",
                                    op.symbol(),
                                    ty_str(&lt)
                                ),
                            )
                            .fix(fix, None),
                        );
                    }
                    return TypeKind::Bool;
                }
                let ok = match op {
                    BinOp::Eq | BinOp::Ne => is_scalar(&lt),
                    _ => is_int(&lt),
                };
                if !ok {
                    self.err(
                        "E0401",
                        span,
                        format!("`{}` is not defined for `{}`", op.symbol(), ty_str(&lt)),
                    );
                    return if op.is_arith() {
                        TypeKind::Error
                    } else {
                        TypeKind::Bool
                    };
                }
                if op.is_arith() { lt } else { TypeKind::Bool }
            }
        }
    }

    fn method(
        &mut self,
        cx: &mut FnCtx,
        recv: &Expr,
        name: &Ident,
        args: &[Expr],
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let rt = self.expr(cx, recv, None, mode);
        if let Some(et) = vec_elem(&rt).cloned() {
            return self.vec_method(cx, recv, &rt, et, name, args, span, mode);
        }
        let value = match &rt {
            TypeKind::Ref { inner, .. } => &inner.kind,
            t => t,
        };
        if let TypeKind::Named { path, .. } = value {
            let tp = path.join("::");
            if (tp == "Option" || tp == "Result")
                && matches!(name.name.as_str(), "unwrap" | "expect")
            {
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                let arms = if tp == "Option" {
                    "None => .., Some(v) => v"
                } else {
                    "Ok(v) => v, Err(e) => .."
                };
                self.diags.push(
                    Diag::new("E0229", span, format!("`.{}()` is not allowed", name.name))
                        .fix(format!("`match` both cases: `match x {{ {arms} }}`"), None),
                );
                return TypeKind::Error;
            }
            if let Some(sig) = self.fns.get(&format!("{tp}::{}", name.name)).cloned() {
                // Rust habit: a method call. The call is by path (ADR-0026 §3).
                let amp = match sig.params.first() {
                    Some(TypeKind::Ref { mutable: true, .. }) => "&mut ",
                    Some(TypeKind::Ref { .. }) => "&",
                    _ => "",
                };
                let r = crate::fmt::expr_text(recv);
                let first = if matches!(rt, TypeKind::Ref { .. }) || amp.is_empty() {
                    r
                } else {
                    format!("{amp}{r}")
                };
                let all: Vec<String> = std::iter::once(first)
                    .chain(args.iter().map(crate::fmt::expr_text))
                    .collect();
                let callee = Self::variant_text(&tp, &name.name, &cx.mod_path);
                self.diags.push(
                    Diag::new("E0227", span, "method calls are not allowed").fix(
                        "call by path",
                        Some(format!("{callee}({})", all.join(", "))),
                    ),
                );
                for (a, t) in args.iter().zip(sig.params.iter().skip(1)) {
                    let at = self.expr(cx, a, Some(t), mode);
                    self.expect_ty(&at, t, a.span);
                }
                return sig.ret.clone();
            }
        }
        match name.name.as_str() {
            "len" => {
                if !args.is_empty() {
                    self.err("E0405", name.span, "`len` takes no arguments");
                }
                if elem_of(&rt).is_none() && rt != TypeKind::Error {
                    self.err(
                        "E0404",
                        name.span,
                        format!("`len` is not defined for `{}`", ty_str(&rt)),
                    );
                    return TypeKind::Error;
                }
                TypeKind::Usize
            }
            "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add"
            | "saturating_sub" | "saturating_mul" | "checked_add" | "checked_sub"
            | "checked_mul" | "checked_div" | "checked_rem" => {
                if args.len() != 1 {
                    self.err(
                        "E0405",
                        name.span,
                        format!("`{}` takes one argument", name.name),
                    );
                    return TypeKind::Error;
                }
                let at = self.expr(cx, &args[0], Some(&rt), mode);
                if !is_int(&rt) && rt != TypeKind::Error {
                    self.err(
                        "E0404",
                        name.span,
                        format!("`{}` is not defined for `{}`", name.name, ty_str(&rt)),
                    );
                    return TypeKind::Error;
                }
                self.expect_ty(&at, &rt, args[0].span);
                // `checked_*` is `Some` exactly when the result is in range
                // (ADR-0028 §3).
                if name.name.starts_with("checked_") && rt != TypeKind::Error {
                    return option_of(rt);
                }
                rt
            }
            "into" | "try_into" if is_int(&rt) => {
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                self.diags.push(
                    Diag::new(
                        "E0212",
                        name.span,
                        format!("`.{}()` is not used", name.name),
                    )
                    .fix(
                        format!(
                            "write `T::try_from({}).unwrap()` with the target type `T`",
                            crate::fmt::expr_text(recv)
                        ),
                        None,
                    ),
                );
                TypeKind::Error
            }
            other => {
                let fix = match other {
                    "unwrap" | "expect" => {
                        Some("handle each case explicitly; panics are not allowed")
                    }
                    "iter" | "iter_mut" | "into_iter" | "map" | "for_each" => {
                        Some("use `while` with an index")
                    }
                    "clone" => Some("arrays and scalars are copied by value; remove `.clone()`"),
                    "pow" | "abs" | "overflowing_add" | "overflowing_sub" | "overflowing_mul" => {
                        Some(
                            "not available; use `checked_*`, `saturating_*` or `wrapping_*` (ADR-0028 §3)",
                        )
                    }
                    _ => None,
                };
                let mut d = Diag::new("E0404", name.span, format!("unknown method `{other}`"));
                if let Some(f) = fix {
                    d = d.fix(f, None);
                }
                self.diags.push(d);
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                TypeKind::Error
            }
        }
    }

    /// A built-in method of `Vec<T>` (ADR-0031 §1): `push`, `pop`, `len`,
    /// `truncate`, `clone`; `as_slice` / `as_mut_slice` only as arguments.
    #[allow(clippy::too_many_arguments)]
    fn vec_method(
        &mut self,
        cx: &mut FnCtx,
        recv: &Expr,
        rt: &TypeKind,
        et: TypeKind,
        name: &Ident,
        args: &[Expr],
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let m = name.name.as_str();
        let (arity, mutates, ret) = match m {
            "push" => (1, true, TypeKind::Unit),
            "pop" => (0, true, option_of(et.clone())),
            "truncate" => (1, true, TypeKind::Unit),
            "len" => (0, false, TypeKind::Usize),
            "clone" => (0, false, TypeKind::Vec(mk(et.clone()))),
            "as_slice" | "as_mut_slice" => {
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                self.diags.push(
                    Diag::new(
                        "E0409",
                        span,
                        format!("`.{m}()` may only appear directly as a call argument"),
                    )
                    .fix("pass it to a `&[T]` / `&mut [T]` parameter", None),
                );
                return TypeKind::Error;
            }
            _ => {
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                let fix = match m {
                    "get" | "first" | "last" => Some("index with `v[i]`: its bound is proved"),
                    "is_empty" => Some("compare `v.len() == 0`"),
                    "iter" | "iter_mut" | "into_iter" => Some("use `while` with an index"),
                    "extend_from_slice" | "resize" | "insert" | "remove" | "swap_remove"
                    | "sort" | "reverse" | "contains" => {
                        Some("not available; write the loop (ADR-0031 §1)")
                    }
                    _ => None,
                };
                let mut d = Diag::new("E0404", name.span, format!("unknown method `{m}`"));
                if let Some(f) = fix {
                    d = d.fix(f, None);
                }
                self.diags.push(d);
                return TypeKind::Error;
            }
        };
        if mode != Mode::Exec && m != "len" {
            self.err(
                "E0503",
                span,
                format!("`.{m}()` is not allowed in a specification"),
            );
        }
        if args.len() != arity {
            self.err(
                "E0405",
                name.span,
                format!("`{m}` takes {arity} argument(s), found {}", args.len()),
            );
        }
        let want = if m == "push" { et } else { TypeKind::Usize };
        for a in args {
            let at = self.expr(cx, a, Some(&want), mode);
            self.expect_ty(&at, &want, a.span);
        }
        if mutates && mode == Mode::Exec {
            let writable = match &recv.kind {
                ExprKind::Var(id) => Self::lookup(cx, &id.name).is_some_and(|l| match &l.ty {
                    TypeKind::Ref { mutable, .. } => *mutable,
                    _ => l.mutable,
                }),
                _ => false,
            };
            if !writable && *rt != TypeKind::Error {
                let fix = if matches!(rt, TypeKind::Ref { .. }) {
                    "take the parameter as `&mut Vec<T>`"
                } else {
                    "declare it with `let mut`"
                };
                self.diags.push(
                    Diag::new(
                        "E0408",
                        recv.span,
                        format!("`.{m}()` needs a mutable `Vec`"),
                    )
                    .fix(fix, None),
                );
            }
        }
        ret
    }

    /// `Vec::new()`, `Vec::with_capacity(n)` (the element type is the
    /// expected one), and the Rust habits `drop(v)`, `mem::take` (E0219).
    fn vec_call(
        &mut self,
        cx: &mut FnCtx,
        path: &[Ident],
        args: &[Expr],
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let last = path.last().unwrap().name.as_str();
        if path[0].name != "Vec" {
            for a in args {
                self.expr(cx, a, None, mode);
            }
            self.diags.push(
                Diag::new(
                    "E0219",
                    span,
                    format!("`{}` is not used", Self::path_text(path)),
                )
                .fix(
                    "a `Vec` is freed at the end of its block; assign a new value to replace it",
                    None,
                ),
            );
            return TypeKind::Error;
        }
        if mode != Mode::Exec {
            self.err(
                "E0503",
                span,
                format!("`Vec::{last}` is not allowed in a specification"),
            );
        }
        let arity = usize::from(last == "with_capacity");
        if args.len() != arity {
            self.err(
                "E0405",
                span,
                format!(
                    "`Vec::{last}` takes {arity} argument(s), found {}",
                    args.len()
                ),
            );
        }
        for a in args {
            let t = self.expr(cx, a, Some(&TypeKind::Usize), mode);
            self.expect_ty(&t, &TypeKind::Usize, a.span);
        }
        match expected {
            Some(t @ TypeKind::Vec(_)) => t.clone(),
            // The written type is wrong and reported.
            Some(TypeKind::Error) => TypeKind::Error,
            _ => {
                self.diags.push(
                    Diag::new(
                        "E0401",
                        span,
                        format!("the element type of `Vec::{last}()` comes from the expected type"),
                    )
                    .fix("write it: `let v: Vec<T> = Vec::new();`", None),
                );
                TypeKind::Error
            }
        }
    }

    /// `Box::new(v)` (ADR-0047 §1): the boxed type is the expected
    /// type's, else `v`'s.
    fn box_new(
        &mut self,
        cx: &mut FnCtx,
        args: &[Expr],
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        if args.len() != 1 {
            self.err(
                "E0405",
                span,
                format!("`Box::new` takes 1 argument, found {}", args.len()),
            );
            for a in args {
                self.expr(cx, a, None, mode);
            }
            return TypeKind::Error;
        }
        let want = match expected {
            Some(TypeKind::Box(e)) => Some(e.kind.clone()),
            // The written type is wrong and reported.
            Some(TypeKind::Error) => {
                self.expr(cx, &args[0], Some(&TypeKind::Error), mode);
                return TypeKind::Error;
            }
            _ => None,
        };
        let t = self.expr(cx, &args[0], want.as_ref(), mode);
        if let Some(w) = want {
            self.expect_ty(&t, &w, args[0].span);
            return TypeKind::Box(mk(w));
        }
        match t {
            TypeKind::Error => TypeKind::Error,
            TypeKind::Vec(_) | TypeKind::Unit => {
                self.err(
                    "E0402",
                    args[0].span,
                    format!("a `Box<{}>` is not supported (ADR-0047 §1)", ty_str(&t)),
                );
                TypeKind::Error
            }
            t => TypeKind::Box(mk(t)),
        }
    }

    fn path_text(path: &[Ident]) -> String {
        path.iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::")
    }

    /// The type of argument `a` from its syntax, before it is checked: a
    /// local, a reference to one, or a `Vec` viewed as a slice. Used to
    /// infer type arguments (E0237).
    fn arg_peek(&self, cx: &FnCtx, a: &Expr) -> Option<TypeKind> {
        match &a.kind {
            ExprKind::Var(id) => Self::lookup(cx, &id.name).map(|l| l.ty),
            ExprKind::Ref { mutable, expr } => Some(TypeKind::Ref {
                mutable: *mutable,
                inner: mk(self.place_peek(cx, expr)?),
            }),
            // `old(p)` in a contract: the parameter's type.
            ExprKind::Old(x) => self.arg_peek(cx, x),
            ExprKind::Method { recv, name, args }
                if args.is_empty() && matches!(name.name.as_str(), "as_slice" | "as_mut_slice") =>
            {
                let ExprKind::Var(id) = &recv.kind else {
                    return None;
                };
                let e = vec_elem(&Self::lookup(cx, &id.name)?.ty)?.clone();
                Some(TypeKind::Ref {
                    mutable: name.name == "as_mut_slice",
                    inner: mk(TypeKind::Slice(mk(e))),
                })
            }
            _ => None,
        }
    }

    /// The type of a place `x`, `x.f.g`, `*p`, from the locals' types.
    fn place_peek(&self, cx: &FnCtx, e: &Expr) -> Option<TypeKind> {
        match &e.kind {
            ExprKind::Var(id) => Self::lookup(cx, &id.name).map(|l| l.ty),
            ExprKind::Field { base, name } => {
                let (_, fs) = self.struct_fields(&self.place_peek(cx, base)?)?;
                fs.into_iter()
                    .find(|(n, _)| *n == name.name)
                    .map(|(_, t)| t)
            }
            ExprKind::Deref(x) => match self.place_peek(cx, x)? {
                TypeKind::Ref { inner, .. } | TypeKind::Box(inner) => Some(inner.kind),
                _ => None,
            },
            ExprKind::Index { base, .. } => elem_of(&self.place_peek(cx, base)?).cloned(),
            ExprKind::Trigger(x) => self.place_peek(cx, x),
            // `&Point { .. }`: a struct without type parameters names itself.
            ExprKind::StructLit { path, .. } => {
                let tp = self.type_path(path, &cx.mod_path)?;
                self.adts[&tp].generics.is_empty().then(|| TypeKind::Named {
                    path: tp.split("::").map(String::from).collect(),
                    args: Vec::new(),
                })
            }
            _ => None,
        }
    }

    /// The type arguments written in a call of `sig` (ADR-0046 §3): the
    /// `impl`'s after the type segment, the function's after its own name.
    /// Returns the bindings and whether a level was left out.
    fn written_targs(
        &mut self,
        cx: &FnCtx,
        path: &[Ident],
        targs: &[TArgs],
        sig: &FnSig,
    ) -> (BTreeMap<String, TypeKind>, bool) {
        let mut m = BTreeMap::new();
        let ig = sig.impl_generics;
        // A trait method's `Self` is never written: it comes from the
        // arguments (ADR-0046 §2).
        let own_ty = if sig.trait_of.is_some() {
            &sig.generics[..0]
        } else {
            &sig.generics[..ig]
        };
        let levels = [
            (path.len().wrapping_sub(2), own_ty, "the type"),
            (path.len() - 1, &sig.generics[ig..], "the function"),
        ];
        let mut missing = false;
        for (seg, ps, what) in levels {
            let written: Vec<&TArgs> = targs.iter().filter(|t| t.seg == seg).collect();
            match written.as_slice() {
                [] => missing |= !ps.is_empty(),
                [t] if sig.trait_of.is_some() && what == "the type" => {
                    self.diags.push(
                        Diag::new(
                            "E0429",
                            t.span,
                            format!(
                                "the trait of `{}` takes no type arguments: `Self` comes from the arguments",
                                Self::path_text(path)
                            ),
                        )
                        .fix("remove them", Some(String::new())),
                    );
                }
                [t] if t.args.len() == ps.len() => {
                    for (a, p) in t.args.iter().zip(ps) {
                        let k = self.canon(a, &cx.mod_path, true);
                        let name = Self::path_text(path);
                        if self.targ_ok(&k, p, &name, a.span, true) {
                            m.insert(p.name.clone(), k);
                        } else {
                            m.insert(p.name.clone(), TypeKind::Error);
                        }
                    }
                }
                [t, ..] => {
                    self.diags.push(
                        Diag::new(
                            "E0429",
                            t.span,
                            format!(
                                "{what} `{}` takes {} type argument(s), found {}",
                                Self::path_text(path),
                                ps.len(),
                                t.args.len()
                            ),
                        )
                        .fix(
                            format!(
                                "write one type for each of `{}`",
                                ps.iter()
                                    .map(|p| p.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join("`, `")
                            ),
                            None,
                        ),
                    );
                    for p in ps {
                        m.insert(p.name.clone(), TypeKind::Error);
                    }
                }
            }
        }
        for t in targs {
            if t.seg != path.len() - 1 && t.seg != path.len().wrapping_sub(2) {
                self.err(
                    "E0429",
                    t.span,
                    "type arguments follow the type or the function name",
                );
            }
        }
        (m, missing)
    }

    /// E0237: the call with its type arguments written out, for the
    /// inferred `m` (`None` when one could not be inferred).
    fn targs_fix(
        &mut self,
        path: &[Ident],
        targs: &[TArgs],
        sig: &FnSig,
        m: &BTreeMap<String, TypeKind>,
        mp: &str,
    ) -> bool {
        let lo = path[0].span;
        let hi = targs
            .iter()
            .map(|t| t.span)
            .chain([path[path.len() - 1].span])
            .max_by_key(|s| s.hi)
            .unwrap();
        let span = lo.to(hi);
        let unbound: Vec<&str> = sig
            .generics
            .iter()
            .filter(|p| !m.contains_key(&p.name))
            .map(|p| p.name.as_str())
            .collect();
        let name = Self::path_text(path);
        if !unbound.is_empty() {
            self.diags.push(
                Diag::new(
                    "E0237",
                    span,
                    format!(
                        "the type arguments of `{name}` are not written, and `{}` cannot be inferred from the arguments",
                        unbound.join("`, `")
                    ),
                )
                .fix(format!("write them: `{name}::<..>(..)`"), None),
            );
            return false;
        }
        let ig = sig.impl_generics;
        let text = |ps: &[TParam]| {
            ps.iter()
                .map(|p| ty_str_in(&m[&p.name], mp))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut segs: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
        let n = segs.len();
        if ig > 0 && sig.trait_of.is_none() {
            segs[n - 2] = format!("{}::<{}>", segs[n - 2], text(&sig.generics[..ig]));
        }
        if sig.generics.len() > ig {
            segs[n - 1] = format!("{}::<{}>", segs[n - 1], text(&sig.generics[ig..]));
        }
        let r = segs.join("::");
        self.diags.push(
            Diag::new(
                "E0237",
                span,
                format!("the type arguments of `{name}` are written out: `{r}`"),
            )
            .fix("write the inferred type arguments (ADR-0046 §3)", Some(r)),
        );
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn call(
        &mut self,
        cx: &mut FnCtx,
        path: &[Ident],
        targs: &[TArgs],
        args: &[Expr],
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let name = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let Some(target) = self.resolve(path, &cx.mod_path) else {
            let segs: Vec<&str> = path.iter().map(|i| i.name.as_str()).collect();
            let pspan = path[0].span.to(path[path.len() - 1].span);
            if self.imported(&segs, pspan, &cx.mod_path.clone()) {
                for a in args {
                    if !matches!(a.kind, ExprKind::Ref { .. }) {
                        self.expr(cx, a, None, mode);
                    }
                }
                return TypeKind::Error;
            }
            let hint = if path.len() == 1 {
                format!(
                    "no `{name}` in unit `{}`; other units are reached by full path `unit::{name}`",
                    cx.mod_path
                )
            } else {
                format!("no function `{name}`")
            };
            self.diags.push(
                Diag::new("E0304", span, format!("unknown function `{name}`")).fix(hint, None),
            );
            for a in args {
                if !matches!(a.kind, ExprKind::Ref { .. }) {
                    self.expr(cx, a, None, mode);
                }
            }
            return TypeKind::Error;
        };
        let sig = self.fns[&target].clone();
        match (mode, sig.kind) {
            (Mode::Exec, FnKind::Spec) => {
                self.err(
                    "E0505",
                    span,
                    format!(
                        "`{name}` is a spec function and cannot be called from executable code"
                    ),
                );
            }
            (Mode::Spec { .. }, FnKind::Exec) => {
                self.err(
                    "E0504",
                    span,
                    format!("`{name}` is executable and cannot be called from a specification"),
                );
            }
            (Mode::Exec, FnKind::Exec) => {
                if cx.effect == Effect::Pure && sig.effect == Effect::Io {
                    self.diags.push(
                        Diag::new(
                            "E0603",
                            span,
                            format!("`{name}` has effect `io`, but `{}` is `pure`", cx.path),
                        )
                        .fix(
                            format!(
                                "declare `#[effects(io)]` on `{}`, or move the call out of it",
                                cx.path
                            ),
                            None,
                        ),
                    );
                }
                self.calls
                    .entry(cx.path.clone())
                    .or_default()
                    .insert(target.clone());
            }
            (Mode::Spec { .. }, FnKind::Spec) => {
                self.calls
                    .entry(cx.path.clone())
                    .or_default()
                    .insert(target.clone());
            }
        }
        if args.len() != sig.params.len() {
            self.err(
                "E0405",
                span,
                format!(
                    "`{name}` takes {} argument(s), found {}",
                    sig.params.len(),
                    args.len()
                ),
            );
        }
        // Type arguments (ADR-0046 §3): written, or inferred from the
        // arguments with E0237 writing them out.
        if sig.generics.is_empty() && !targs.is_empty() {
            self.diags.push(
                Diag::new(
                    "E0429",
                    targs[0].span,
                    format!("`{name}` takes no type arguments"),
                )
                .fix("remove them", Some(String::new())),
            );
        }
        let (mut tm, missing) = if sig.generics.is_empty() {
            (BTreeMap::new(), false)
        } else {
            self.written_targs(cx, path, targs, &sig)
        };
        // A trait method's `Self` is always inferred (ADR-0046 §2).
        let is_trait = sig.trait_of.is_some();
        let infer = missing || is_trait;
        // Rust infers from the expected result type too.
        if infer && let Some(want) = expected {
            unify(&sig.ret, want, &sig.generics, &mut tm);
        }
        let unbound = |tm: &BTreeMap<String, TypeKind>, t: &TypeKind| {
            has_param(&subst(t, tm))
                && sig
                    .generics
                    .iter()
                    .any(|p| mentions_param(t, &p.name) && !tm.contains_key(&p.name))
        };
        let mut deferred = Vec::new();
        let mut mut_roots: Vec<(String, usize)> = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let Some(pt0) = sig.params.get(i) else {
                self.expr(cx, a, None, mode);
                continue;
            };
            if infer && unbound(&tm, pt0) {
                if untyped_literal(a) {
                    deferred.push(i);
                    continue;
                }
                let at = if matches!(pt0, TypeKind::Ref { .. }) {
                    self.arg_peek(cx, a)
                } else {
                    Some(self.expr(cx, a, None, mode))
                };
                if let Some(at) = &at {
                    unify(pt0, at, &sig.generics, &mut tm);
                }
                if !matches!(pt0, TypeKind::Ref { .. }) {
                    let pt = subst(pt0, &tm);
                    if let Some(t) = at
                        && !has_param_of(&pt, &sig.generics, &tm)
                        && !(mode != Mode::Exec && is_int(&t) && is_int(&pt))
                    {
                        self.expect_ty(&t, &pt, a.span);
                    }
                    continue;
                }
            }
            let pt = subst(pt0, &tm);
            // A reference whose type parameters stay unknown: reported below.
            if is_trait && matches!(pt, TypeKind::Ref { .. }) && unbound(&tm, pt0) {
                continue;
            }
            if let TypeKind::Ref { mutable: pm, inner } = &pt {
                if let Some(root) = self.ref_arg(cx, a, *pm, inner, mode)
                    && *pm
                {
                    mut_roots.push((root, i));
                }
            } else {
                let t = self.expr(cx, a, Some(&pt), mode);
                if !(mode != Mode::Exec && is_int(&t) && is_int(&pt)) && !self.deref_fix(&t, &pt, a)
                {
                    self.expect_ty(&t, &pt, a.span);
                }
            }
        }
        for i in deferred {
            let pt = subst(&sig.params[i], &tm);
            let a = &args[i];
            if has_param_of(&pt, &sig.generics, &tm) {
                self.expr(cx, a, None, mode);
                continue;
            }
            let t = self.expr(cx, a, Some(&pt), mode);
            if !(mode != Mode::Exec && is_int(&t) && is_int(&pt)) {
                self.expect_ty(&t, &pt, a.span);
            }
        }
        if is_trait {
            let Some(t) = tm.get("Self").filter(|t| **t != TypeKind::Error).cloned() else {
                let tr = sig.trait_of.clone().unwrap_or_default();
                self.diags.push(
                    Diag::new(
                        "E0237",
                        span,
                        format!(
                            "the implementing type of `{name}` cannot be inferred: no argument or expected result gives `Self`"
                        ),
                    )
                    .fix(
                        format!("give an argument or the result a type that implements `{tr}`"),
                        None,
                    ),
                );
                return TypeKind::Error;
            };
            if !self.targ_ok(&t, &sig.generics[0], &name, span, true) {
                return TypeKind::Error;
            }
        }
        if missing {
            let inferred = sig
                .generics
                .iter()
                .all(|p| tm.get(&p.name).is_some_and(|t| *t != TypeKind::Error));
            if inferred {
                for p in &sig.generics[usize::from(is_trait)..] {
                    let t = tm[&p.name].clone();
                    if !self.targ_ok(&t, p, &name, span, true) {
                        tm.insert(p.name.clone(), TypeKind::Error);
                    }
                }
            }
            let known: BTreeMap<String, TypeKind> = tm
                .iter()
                .filter(|(_, t)| **t != TypeKind::Error)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            if !self.targs_fix(path, targs, &sig, &known, &cx.mod_path.clone()) {
                return TypeKind::Error;
            }
        }
        if !sig.generics.is_empty() {
            if sig
                .generics
                .iter()
                .any(|p| tm.get(&p.name).is_none_or(|t| *t == TypeKind::Error))
            {
                return TypeKind::Error;
            }
            let args: Vec<TypeKind> = sig.generics.iter().map(|p| tm[&p.name].clone()).collect();
            // A trait call with a known `Self` runs its impl's function.
            if let Some((imp, _)) = dispatch(&self.fns, &self.trait_impls, &target, &args) {
                self.calls.entry(cx.path.clone()).or_default().insert(imp);
            }
            self.gcalls
                .push((cx.path.clone(), target.clone(), span, args.clone()));
            self.targs.insert((span.file, span.lo, span.hi), args);
        }
        for (root, i) in mut_roots {
            for (j, a) in args.iter().enumerate() {
                // A scalar argument that only reads the array is a value
                // computed before the call, so it cannot alias (ADR-0024):
                // `fill(&mut a, a.len())`, `put(&mut a, a[0])`.
                let by_value = !matches!(
                    sig.params.get(j),
                    Some(TypeKind::Ref { .. } | TypeKind::Vec(_))
                );
                if j != i && by_value && reads_only(a) {
                    continue;
                }
                if j != i && mentions_strict(a, &root) {
                    self.err(
                        "E0410",
                        a.span,
                        format!("`{root}` is passed as `&mut` and also used by another argument"),
                    );
                }
            }
        }
        subst(&sig.ret, &tm)
    }

    /// `T::try_from(x).unwrap()` / `.ok()` (ADR-0028 §1): the value of `x`
    /// in integer type `T` (a range obligation, E0815, for the prover), or
    /// an `Option<T>`.
    fn conversion(
        &mut self,
        cx: &mut FnCtx,
        recv: &Expr,
        name: &Ident,
        margs: &[Expr],
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let (target, args) = conv_call(recv).unwrap();
        if args.len() != 1 {
            self.err("E0405", recv.span, "`try_from` takes one value");
            return TypeKind::Error;
        }
        let st = self.expr(cx, &args[0], None, mode);
        for a in margs {
            self.expr(cx, a, None, mode);
        }
        let x = crate::fmt::expr_text(&args[0]);
        let unwrapped = format!("{}::try_from({x}).unwrap()", ty_str(&target));
        if mode != Mode::Exec {
            self.diags.push(
                Diag::new("E0513", span, "conversions are not used in specifications").fix(
                    "integer types mix in specifications (ADR-0022): write the value",
                    Some(x),
                ),
            );
            return st;
        }
        if !is_int(&st) {
            if st != TypeKind::Error {
                let msg = format!(
                    "`try_from` converts between integer types, found `{}`",
                    ty_str(&st)
                );
                // A `bool` becomes an integer by choosing the values.
                self.diags
                    .push(if st == TypeKind::Bool && name.name == "unwrap" {
                        Diag::new("E0412", span, msg).fix(
                            "choose the values",
                            Some(format!("if {x} {{ 1 }} else {{ 0 }}")),
                        )
                    } else {
                        Diag::new("E0412", args[0].span, msg)
                    });
            }
            return TypeKind::Error;
        }
        match name.name.as_str() {
            "unwrap" if margs.is_empty() => target,
            "ok" if margs.is_empty() => option_of(target),
            "expect" => {
                self.diags
                    .push(Diag::new("E0213", span, "write `.unwrap()`").fix(
                        "a conversion has one form (its range is proved)",
                        Some(unwrapped),
                    ));
                target
            }
            _ => {
                self.diags.push(
                    Diag::new(
                        "E0412",
                        span,
                        "`T::try_from(x)` is used as `.unwrap()` or `.ok()`",
                    )
                    .fix(
                        "`.unwrap()`: the value, with a range obligation",
                        Some(unwrapped),
                    ),
                );
                TypeKind::Error
            }
        }
    }

    /// Checks an argument for a reference parameter; returns the root variable name.
    fn ref_arg(
        &mut self,
        cx: &mut FnCtx,
        a: &Expr,
        pm: bool,
        inner: &Type,
        mode: Mode,
    ) -> Option<String> {
        let want = format!("&{}{}", if pm { "mut " } else { "" }, ty_str(&inner.kind));
        let TypeKind::Slice(pe) = &inner.kind else {
            return self.data_ref_arg(cx, a, pm, &inner.kind, &want, mode);
        };
        if let Some(r) = self.slice_of_vec(cx, a, pm, &pe.kind, &want, mode) {
            return r;
        }
        if let ExprKind::ByteStr(_) = &a.kind {
            let st = TypeKind::Ref {
                mutable: false,
                inner: mk(TypeKind::Slice(mk(TypeKind::U8))),
            };
            self.types.insert((a.span.file, a.span.lo, a.span.hi), st);
            if pm || pe.kind != TypeKind::U8 {
                self.err(
                    "E0401",
                    a.span,
                    format!("expected `{want}`, found a byte string (`&[u8]`, read-only)"),
                );
            }
            return None;
        }
        let (var, explicit_ref, am) = match &a.kind {
            ExprKind::Ref { mutable, expr } => match &expr.kind {
                ExprKind::Var(id) => (id, true, *mutable),
                _ => {
                    self.err("E0409", a.span, format!("expected `{want}` of a variable"));
                    return None;
                }
            },
            ExprKind::Var(id) => (id, false, false),
            ExprKind::Old(inner_e) => {
                let t = self.expr(cx, a, None, mode);
                if let (ExprKind::Var(id), TypeKind::Ref { inner: ri, .. }) = (&inner_e.kind, &t)
                    && !pm
                    && ri.kind == inner.kind
                {
                    return Some(id.name.clone());
                }
                if t != TypeKind::Error {
                    self.err(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{}`", ty_str(&t)),
                    );
                }
                return None;
            }
            _ => {
                let t = self.expr(cx, a, None, mode);
                if t != TypeKind::Error {
                    self.err(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{}`", ty_str(&t)),
                    );
                }
                return None;
            }
        };
        let Some(l) = Self::lookup(cx, &var.name) else {
            self.unknown_local(cx, &var.name, var.span);
            return None;
        };
        let ok = match (&l.ty, explicit_ref) {
            // `&arr` / `&mut arr` for a local array.
            (TypeKind::Array(e, _), true) => {
                if e.kind != pe.kind {
                    false
                } else if am && !l.mutable {
                    self.diags.push(
                        Diag::new("E0408", a.span, format!("`{}` is not mutable", var.name))
                            .fix("declare it with `let mut`", None),
                    );
                    return None;
                } else {
                    am == pm
                }
            }
            // Passing a reference parameter on: `f(a)`.
            (
                TypeKind::Ref {
                    mutable: lm,
                    inner: li,
                },
                false,
            ) => li.kind == inner.kind && (*lm || !pm),
            _ => false,
        };
        if !ok {
            let found = if explicit_ref {
                format!("&{}{}", if am { "mut " } else { "" }, ty_str(&l.ty))
            } else {
                ty_str(&l.ty)
            };
            let mut d = Diag::new(
                "E0401",
                a.span,
                format!("expected `{want}`, found `{found}`"),
            );
            if !explicit_ref && matches!(l.ty, TypeKind::Array(..)) {
                let r = format!("&{}{}", if pm { "mut " } else { "" }, var.name);
                d = d.fix(format!("pass `{r}`"), Some(r));
            }
            self.diags.push(d);
            return None;
        }
        Some(var.name.clone())
    }

    /// A `&[T]` / `&mut [T]` argument made from a `Vec` (ADR-0031 §1):
    /// `v.as_slice()`, `v.as_mut_slice()`, and the Rust habits `&v`, `&mut v`
    /// and a `&Vec<T>` parameter passed on (E0218). `None` when the
    /// argument is not about a `Vec`; else the result for `ref_arg`.
    fn slice_of_vec(
        &mut self,
        cx: &mut FnCtx,
        a: &Expr,
        pm: bool,
        pe: &TypeKind,
        want: &str,
        mode: Mode,
    ) -> Option<Option<String>> {
        let want_m = if pm { "as_mut_slice" } else { "as_slice" };
        let vec_var = |cx: &FnCtx, x: &Expr| match &x.kind {
            ExprKind::Var(id) => Self::lookup(cx, &id.name).filter(|l| vec_elem(&l.ty).is_some()),
            _ => None,
        };
        match &a.kind {
            ExprKind::Method { recv, name, args }
                if matches!(name.name.as_str(), "as_slice" | "as_mut_slice") =>
            {
                let rt = self.expr(cx, recv, None, mode);
                let Some(et) = vec_elem(&rt).cloned() else {
                    if rt != TypeKind::Error {
                        self.err(
                            "E0404",
                            name.span,
                            format!("`{}` is not defined for `{}`", name.name, ty_str(&rt)),
                        );
                    }
                    return Some(None);
                };
                if !args.is_empty() {
                    self.err(
                        "E0405",
                        name.span,
                        format!("`{}` takes no arguments", name.name),
                    );
                }
                let st = TypeKind::Ref {
                    mutable: name.name == "as_mut_slice",
                    inner: mk(TypeKind::Slice(mk(et.clone()))),
                };
                let found = ty_str(&st);
                self.types.insert((a.span.file, a.span.lo, a.span.hi), st);
                if et != *pe || (name.name == "as_mut_slice") != pm {
                    let r = format!("{}.{want_m}()", crate::fmt::expr_text(recv));
                    let mut d = Diag::new(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{found}`"),
                    );
                    if et == *pe {
                        d = d.fix(format!("pass `{r}`"), Some(r));
                    }
                    self.diags.push(d);
                    return Some(None);
                }
                if !pm || mode != Mode::Exec {
                    return Some(None);
                }
                let writable = vec_var(cx, recv).is_some_and(|l| match &l.ty {
                    TypeKind::Ref { mutable, .. } => *mutable,
                    _ => l.mutable,
                });
                if !writable {
                    self.diags.push(
                        Diag::new(
                            "E0408",
                            recv.span,
                            "`.as_mut_slice()` needs a mutable `Vec`",
                        )
                        .fix("declare it with `let mut` or take `&mut Vec<T>`", None),
                    );
                    return Some(None);
                }
                Some(root_name(recv))
            }
            ExprKind::Ref { expr, .. } | ExprKind::Old(expr) if vec_var(cx, expr).is_some() => {
                self.e0218(cx, a, expr, want_m, mode)
            }
            ExprKind::Var(_) if vec_var(cx, a).is_some() => self.e0218(cx, a, a, want_m, mode),
            _ => None,
        }
    }

    /// E0218: a `Vec` given where a slice is expected (Rust's Deref coercion).
    fn e0218(
        &mut self,
        cx: &mut FnCtx,
        a: &Expr,
        v: &Expr,
        want_m: &str,
        mode: Mode,
    ) -> Option<Option<String>> {
        self.expr(cx, v, None, mode);
        let r = format!("{}.{want_m}()", crate::fmt::expr_text(v));
        self.diags.push(
            Diag::new("E0218", a.span, "a `Vec` is not a slice")
                .fix(format!("pass `{r}`"), Some(r)),
        );
        Some(None)
    }

    /// Checks an argument for a `&T` / `&mut T` parameter of a struct or
    /// enum `T` (ADR-0026 §2): `&e` of a `T` value, `&mut p` of a mutable
    /// place, or a reference parameter passed on. Returns the root variable
    /// of a `&mut` argument (for the aliasing check, E0410).
    fn data_ref_arg(
        &mut self,
        cx: &mut FnCtx,
        a: &Expr,
        pm: bool,
        want_ty: &TypeKind,
        want: &str,
        mode: Mode,
    ) -> Option<String> {
        match &a.kind {
            ExprKind::Ref { mutable: am, expr } => {
                if *am != pm {
                    let r = format!(
                        "&{}{}",
                        if pm { "mut " } else { "" },
                        crate::fmt::expr_text(expr)
                    );
                    self.diags.push(
                        Diag::new("E0401", a.span, format!("expected `{want}`"))
                            .fix(format!("pass `{r}`"), Some(r)),
                    );
                }
                if !pm {
                    let t = self.expr(cx, expr, Some(want_ty), mode);
                    self.expect_ty(&t, want_ty, expr.span);
                    return None;
                }
                let (t, writable) = self.place_of(cx, expr);
                self.expect_ty(&t, want_ty, expr.span);
                let root = root_name(expr);
                if !writable && t != TypeKind::Error {
                    self.diags.push(
                        Diag::new(
                            "E0408",
                            expr.span,
                            format!("`{}` is not mutable", crate::fmt::expr_text(expr)),
                        )
                        .fix("declare the variable with `let mut`", None),
                    );
                    return None;
                }
                root
            }
            // Passing a reference parameter on: `f(p)`.
            ExprKind::Var(id) => {
                let t = self.expr(cx, a, None, mode);
                let ok = match &t {
                    TypeKind::Ref { mutable: lm, inner } => inner.kind == *want_ty && (*lm || !pm),
                    TypeKind::Error => true,
                    _ => false,
                };
                if !ok {
                    let mut d = Diag::new(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{}`", ty_str(&t)),
                    );
                    if t == *want_ty {
                        let r = format!("&{}{}", if pm { "mut " } else { "" }, id.name);
                        d = d.fix(format!("pass `{r}`"), Some(r));
                    }
                    self.diags.push(d);
                    return None;
                }
                pm.then(|| id.name.clone())
            }
            // `old(p)` of a reference parameter, passed on in a contract.
            ExprKind::Old(_) if !pm => {
                let t = self.expr(cx, a, None, mode);
                let ok = match &t {
                    TypeKind::Ref { inner, .. } => inner.kind == *want_ty,
                    TypeKind::Error => true,
                    _ => false,
                };
                if !ok {
                    self.diags.push(Diag::new(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{}`", ty_str(&t)),
                    ));
                }
                None
            }
            _ => {
                let t = self.expr(cx, a, None, mode);
                if t != TypeKind::Error {
                    let mut d = Diag::new(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{}`", ty_str(&t)),
                    );
                    if t == *want_ty && !pm {
                        let r = format!("&{}", crate::fmt::expr_text(a));
                        d = d.fix(format!("pass `{r}`"), Some(r));
                    }
                    self.diags.push(d);
                }
                None
            }
        }
    }

    // ---------- whole-program checks ----------

    fn check_recursion(&mut self, p: &Program) {
        let mut defs: BTreeMap<String, (Span, bool)> = BTreeMap::new();
        for (path, _, f) in defs_of(p, &self.fns) {
            defs.entry(path)
                .or_insert((f.name.span, f.decreases.is_some()));
        }
        let comps = sccs_with_cycles(&self.calls);
        for (id, comp) in comps.iter().enumerate() {
            for path in comp {
                self.recursive.insert(path.clone(), id);
            }
        }
        let through_trait = self.trait_cycles(&defs);
        for path in comps.into_iter().flatten() {
            if let Some((span, has_dec)) = defs.get(&path)
                && !has_dec
                && !through_trait.contains(&path)
            {
                self.diags.push(
                    Diag::new(
                        "E0507",
                        *span,
                        format!("`{path}` is recursive and has no `decreases` clause"),
                    )
                    .fix("add `decreases <expr>,` after `ensures`", None),
                );
            }
        }
        // A recursive call that wraps a type parameter in a larger type
        // (`f::<Option<T>>` inside `f<T>`) has no last instance.
        for (caller, callee, span, args) in std::mem::take(&mut self.gcalls) {
            let same = self.recursive.get(&caller);
            if same.is_none() || same != self.recursive.get(&callee) {
                continue;
            }
            if let Some(a) = args
                .iter()
                .find(|a| has_param(a) && !matches!(a, TypeKind::Param(_)))
            {
                self.diags.push(
                    Diag::new(
                        "E0430",
                        span,
                        format!(
                            "the recursive call of `{callee}` wraps a type parameter in `{}`: its instances would never end",
                            ty_str(a)
                        ),
                    )
                    .fix(
                        "pass a type parameter itself or a type without parameters",
                        None,
                    ),
                );
            }
        }
    }

    /// Recursion through a call on a bounded `Self` (ADR-0046 §2): a cycle,
    /// on the graph of the instances the program reaches from its
    /// definitions without type parameters, that takes such a call. A
    /// generic definition is proved once, where that call names only the
    /// trait's contract, which carries no measure (a trait method takes no
    /// `decreases`), so none is checked along it. Reports E0207 on the
    /// definitions of such
    /// a cycle and returns them. Structural delegation (`impl<T: Shape>
    /// Shape for Wrap<T>` calling `T`'s) makes no cycle.
    fn trait_cycles(&mut self, defs: &BTreeMap<String, (Span, bool)>) -> BTreeSet<String> {
        let mut edges: BTreeMap<&str, Vec<(&str, Vec<TypeKind>)>> = BTreeMap::new();
        for (caller, callee, _, args) in &self.gcalls {
            edges
                .entry(caller.as_str())
                .or_default()
                .push((callee.as_str(), args.clone()));
        }
        for (caller, cs) in &self.calls {
            for c in cs {
                if self.fns.get(c).is_some_and(|s| s.generics.is_empty()) {
                    edges
                        .entry(caller.as_str())
                        .or_default()
                        .push((c.as_str(), Vec::new()));
                }
            }
        }
        let key = |p: &str, a: &[TypeKind]| {
            if a.is_empty() {
                p.to_string()
            } else {
                crate::verify::inst_key(p, a)
            }
        };
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut bound_edges: BTreeSet<(String, String)> = BTreeSet::new();
        let mut base_of: BTreeMap<String, String> = BTreeMap::new();
        let mut work: Vec<(String, Vec<TypeKind>)> = defs
            .keys()
            .filter(|p| self.fns.get(*p).is_some_and(|s| s.generics.is_empty()))
            .map(|p| (p.clone(), Vec::new()))
            .collect();
        while let Some((p, args)) = work.pop() {
            let k = key(&p, &args);
            if graph.contains_key(&k) {
                continue;
            }
            graph.insert(k.clone(), BTreeSet::new());
            base_of.insert(k.clone(), p.clone());
            let sub: BTreeMap<String, TypeKind> = self.fns[&p]
                .generics
                .iter()
                .map(|g| g.name.clone())
                .zip(args.iter().cloned())
                .collect();
            for (c, cargs) in edges.get(p.as_str()).cloned().unwrap_or_default() {
                let a: Vec<TypeKind> = cargs.iter().map(|t| subst(t, &sub)).collect();
                if a.iter()
                    .any(|t| crate::verify::type_depth(t) > crate::verify::MAX_INST_DEPTH)
                {
                    continue;
                }
                let (tp, ta, bound) = if self.fns[c].trait_of.is_some() {
                    match dispatch(&self.fns, &self.trait_impls, c, &a) {
                        Some((ip, ia)) => {
                            (ip, ia, matches!(cargs.first(), Some(TypeKind::Param(_))))
                        }
                        None => continue,
                    }
                } else {
                    (c.to_string(), a, false)
                };
                let ck = key(&tp, &ta);
                if bound {
                    bound_edges.insert((k.clone(), ck.clone()));
                }
                graph.get_mut(&k).expect("inserted").insert(ck);
                work.push((tp, ta));
            }
        }
        let mut out = BTreeSet::new();
        for comp in sccs_with_cycles(&graph) {
            let set: BTreeSet<&String> = comp.iter().collect();
            let Some((_, to)) = bound_edges
                .iter()
                .find(|(a, b)| set.contains(a) && set.contains(b))
            else {
                continue;
            };
            let to = to.clone();
            for k in &comp {
                let b = base_of[k].clone();
                if out.insert(b.clone())
                    && let Some((span, _)) = defs.get(&b)
                {
                    self.diags.push(
                        Diag::new(
                            "E0207",
                            *span,
                            format!(
                                "`{b}` is recursive through a call on a bounded `Self` (reaching `{to}`), which carries no measure (ADR-0046 §4)"
                            ),
                        )
                        .fix(
                            "call with a known `Self` type, or break the cycle",
                            None,
                        ),
                    );
                }
            }
        }
        out
    }

    fn check_main(&mut self, p: &Program) {
        let mut mains = Vec::new();
        fn walk<'b>(m: &'b Mod, out: &mut Vec<&'b FnDef>) {
            for s in &m.mods {
                walk(s, out);
            }
            out.extend(
                m.fns
                    .iter()
                    .filter(|f| f.kind == FnKind::Exec && f.name.name == "main"),
            );
        }
        for m in &p.mods {
            walk(m, &mut mains);
        }
        for (i, f) in mains.iter().enumerate() {
            if i > 0 {
                self.err("E0702", f.name.span, "the program already has a `main`");
            }
            let io = f.attrs.iter().any(|a| {
                a.name.name == "effects"
                    && a.args
                        .iter()
                        .any(|x| matches!(x, AttrArg::Ident(i) if i.name == "io"))
            });
            if !f.params.is_empty() || f.ret.ty.kind != TypeKind::Unit || !io {
                self.diags.push(
                    Diag::new(
                        "E0701",
                        f.name.span,
                        "`main` must be `#[effects(io)] fn main() -> ()`",
                    )
                    .fix(
                        "write `#[effects(io)] fn main() -> ()`: no parameters, no result",
                        None,
                    ),
                );
            }
        }
    }
}

/// True if `e` mentions variable `name` anywhere.
fn mentions(e: &Expr, name: &str) -> bool {
    match &e.kind {
        ExprKind::Var(id) => id.name == name,
        ExprKind::Call { args, .. } | ExprKind::Array(args) => {
            args.iter().any(|a| mentions(a, name))
        }
        ExprKind::Method { recv, args, .. } => {
            mentions(recv, name) || args.iter().any(|a| mentions(a, name))
        }
        ExprKind::Index { base, index } => mentions(base, name) || mentions(index, name),
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Deref(x)
        | ExprKind::Trigger(x) => mentions(x, name),
        ExprKind::Repeat { value, .. } => mentions(value, name),
        ExprKind::Binary(_, l, r) => mentions(l, name) || mentions(r, name),
        ExprKind::Field { base, .. } => mentions(base, name),
        ExprKind::StructLit { fields, .. } => fields.iter().any(|(_, x)| mentions(x, name)),
        // Blocks and conditionals as call arguments are rare; be conservative.
        ExprKind::If { .. }
        | ExprKind::Block(_)
        | ExprKind::Forall { .. }
        | ExprKind::Match { .. } => true,
        ExprKind::Int(_)
        | ExprKind::ByteLit(_)
        | ExprKind::ByteStr(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Path(_)
        | ExprKind::Error => false,
    }
}

/// `#[trigger]` terms of a `forall` body, not descending into nested `forall`s.
fn collect_triggers<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
    match &e.kind {
        ExprKind::Trigger(inner) => {
            out.push(inner);
            collect_triggers(inner, out);
        }
        ExprKind::Forall { .. } => {}
        ExprKind::Call { args, .. } | ExprKind::Array(args) => {
            args.iter().for_each(|a| collect_triggers(a, out))
        }
        ExprKind::Method { recv, args, .. } => {
            collect_triggers(recv, out);
            args.iter().for_each(|a| collect_triggers(a, out));
        }
        ExprKind::Index { base, index } => {
            collect_triggers(base, out);
            collect_triggers(index, out);
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Deref(x) => collect_triggers(x, out),
        ExprKind::Repeat { value, .. } => collect_triggers(value, out),
        ExprKind::Binary(_, l, r) => {
            collect_triggers(l, out);
            collect_triggers(r, out);
        }
        ExprKind::If { cond, then, els } => {
            collect_triggers(cond, out);
            if let Some(t) = &then.tail {
                collect_triggers(t, out);
            }
            if let Some(x) = els {
                collect_triggers(x, out);
            }
        }
        ExprKind::Block(b) => {
            if let Some(t) = &b.tail {
                collect_triggers(t, out);
            }
        }
        ExprKind::Field { base, .. } => collect_triggers(base, out),
        ExprKind::StructLit { fields, .. } => {
            fields.iter().for_each(|(_, x)| collect_triggers(x, out))
        }
        ExprKind::Match { scrut, arms } => {
            collect_triggers(scrut, out);
            arms.iter().for_each(|a| collect_triggers(&a.body, out));
        }
        ExprKind::Var(_)
        | ExprKind::Path(_)
        | ExprKind::Int(_)
        | ExprKind::ByteLit(_)
        | ExprKind::ByteStr(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Error => {}
    }
}

/// Like `mentions`, but exact for every expression kind (used for triggers).
/// True if any statement or the tail of `b` mentions `name`.
fn block_mentions(b: &Block, name: &str) -> bool {
    b.stmts.iter().any(|s| match s {
        Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => mentions_strict(init, name),
        Stmt::Assign { target, value, .. } => {
            mentions_strict(target, name) || mentions_strict(value, name)
        }
        Stmt::While {
            cond,
            invariants,
            decreases,
            body,
            ..
        } => {
            mentions_strict(cond, name)
                || invariants.iter().any(|i| mentions_strict(i, name))
                || decreases.as_ref().is_some_and(|d| mentions_strict(d, name))
                || block_mentions(body, name)
        }
        Stmt::Assert { cond, .. } => mentions_strict(cond, name),
        Stmt::Expr(e) => mentions_strict(e, name),
    }) || b.tail.as_ref().is_some_and(|t| mentions_strict(t, name))
}

fn mentions_strict(e: &Expr, name: &str) -> bool {
    match &e.kind {
        ExprKind::If { cond, then, els } => {
            mentions_strict(cond, name)
                || block_mentions(then, name)
                || els.as_ref().is_some_and(|x| mentions_strict(x, name))
        }
        ExprKind::Block(b) => block_mentions(b, name),
        ExprKind::Forall { body, .. } => mentions_strict(body, name),
        ExprKind::Call { args, .. } | ExprKind::Array(args) => {
            args.iter().any(|a| mentions_strict(a, name))
        }
        ExprKind::Method { recv, args, .. } => {
            mentions_strict(recv, name) || args.iter().any(|a| mentions_strict(a, name))
        }
        ExprKind::Index { base, index } => {
            mentions_strict(base, name) || mentions_strict(index, name)
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Trigger(x) => mentions_strict(x, name),
        ExprKind::Repeat { value, .. } => mentions_strict(value, name),
        ExprKind::Binary(_, l, r) => mentions_strict(l, name) || mentions_strict(r, name),
        _ => mentions(e, name),
    }
}

/// Strongly connected components that contain a cycle (Tarjan), each sorted,
/// in a deterministic order.
fn sccs_with_cycles(g: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
    struct T<'g> {
        g: &'g BTreeMap<String, BTreeSet<String>>,
        index: BTreeMap<&'g str, usize>,
        low: BTreeMap<&'g str, usize>,
        on: BTreeSet<&'g str>,
        stack: Vec<&'g str>,
        next: usize,
        out: Vec<Vec<String>>,
    }
    impl<'g> T<'g> {
        fn visit(&mut self, v: &'g str) {
            self.index.insert(v, self.next);
            self.low.insert(v, self.next);
            self.next += 1;
            self.stack.push(v);
            self.on.insert(v);
            if let Some(ws) = self.g.get(v) {
                for w in ws {
                    let w = w.as_str();
                    if !self.index.contains_key(w) {
                        self.visit(w);
                        let lw = self.low[w];
                        let lv = self.low.get_mut(v).unwrap();
                        *lv = (*lv).min(lw);
                    } else if self.on.contains(w) {
                        let iw = self.index[w];
                        let lv = self.low.get_mut(v).unwrap();
                        *lv = (*lv).min(iw);
                    }
                }
            }
            if self.low[v] == self.index[v] {
                let mut comp = Vec::new();
                loop {
                    let w = self.stack.pop().unwrap();
                    self.on.remove(w);
                    comp.push(w);
                    if w == v {
                        break;
                    }
                }
                let self_loop = self.g.get(v).is_some_and(|s| s.contains(v));
                if comp.len() > 1 || self_loop {
                    let mut c: Vec<String> = comp.iter().map(|s| s.to_string()).collect();
                    c.sort();
                    self.out.push(c);
                }
            }
        }
    }
    let mut t = T {
        g,
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        on: BTreeSet::new(),
        stack: Vec::new(),
        next: 0,
        out: Vec::new(),
    };
    for v in g.keys() {
        if !t.index.contains_key(v.as_str()) {
            t.visit(v);
        }
    }
    t.out.sort();
    t.out
}

/// An integer expression made of literals only (`5`, `-1`, `(2 * 3)`,
/// `{ 7 }`, `if c { 1 } else { 2 }`): it has no type of its own and takes one
/// from its context, as unsuffixed literals do in Rust.
/// Whether `t` names type parameter `n`.
/// Whether two implementing types can be the same type (a type parameter
/// matches anything; conservative for a parameter written twice).
fn overlaps(a: &TypeKind, b: &TypeKind) -> bool {
    match (a, b) {
        (TypeKind::Param(_), _) | (_, TypeKind::Param(_)) => true,
        (TypeKind::Array(x, n), TypeKind::Array(y, k)) => n == k && overlaps(&x.kind, &y.kind),
        (TypeKind::Slice(x), TypeKind::Slice(y))
        | (TypeKind::Vec(x), TypeKind::Vec(y))
        | (TypeKind::Box(x), TypeKind::Box(y)) => overlaps(&x.kind, &y.kind),
        (
            TypeKind::Ref {
                mutable: p,
                inner: x,
            },
            TypeKind::Ref {
                mutable: q,
                inner: y,
            },
        ) => p == q && overlaps(&x.kind, &y.kind),
        (TypeKind::Named { path: p, args: x }, TypeKind::Named { path: q, args: y }) => {
            p == q && x.len() == y.len() && x.iter().zip(y).all(|(a, b)| overlaps(&a.kind, &b.kind))
        }
        _ => a == b,
    }
}

fn mentions_param(t: &TypeKind, n: &str) -> bool {
    match t {
        TypeKind::Param(x) => x == n,
        TypeKind::Array(e, _) | TypeKind::Slice(e) | TypeKind::Vec(e) | TypeKind::Box(e) => {
            mentions_param(&e.kind, n)
        }
        TypeKind::Ref { inner, .. } => mentions_param(&inner.kind, n),
        TypeKind::Named { args, .. } => args.iter().any(|a| mentions_param(&a.kind, n)),
        _ => false,
    }
}

/// Whether `t` still names a parameter of `ps` that `tm` does not bind.
fn has_param_of(t: &TypeKind, ps: &[TParam], tm: &BTreeMap<String, TypeKind>) -> bool {
    ps.iter()
        .any(|p| !tm.contains_key(&p.name) && mentions_param(t, &p.name))
}

/// `ty_str` with the types of unit `mp` written by their own name.
pub fn ty_str_in(t: &TypeKind, mp: &str) -> String {
    let local = |path: &[String]| {
        let full = path.join("::");
        full.strip_prefix(&format!("{mp}::"))
            .filter(|r| !r.contains("::"))
            .map_or(full.clone(), String::from)
    };
    match t {
        TypeKind::Array(e, n) => format!("[{}; {n}]", ty_str_in(&e.kind, mp)),
        TypeKind::Slice(e) => format!("[{}]", ty_str_in(&e.kind, mp)),
        TypeKind::Ref { mutable, inner } => format!(
            "&{}{}",
            if *mutable { "mut " } else { "" },
            ty_str_in(&inner.kind, mp)
        ),
        TypeKind::Named { path, args } if args.is_empty() => local(path),
        TypeKind::Named { path, args } => format!(
            "{}<{}>",
            local(path),
            args.iter()
                .map(|a| ty_str_in(&a.kind, mp))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeKind::Vec(e) => format!("Vec<{}>", ty_str_in(&e.kind, mp)),
        TypeKind::Box(e) => format!("Box<{}>", ty_str_in(&e.kind, mp)),
        other => ty_str(other),
    }
}

fn untyped_literal(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_) => true,
        ExprKind::Unary(UnOp::Neg, i) => untyped_literal(i),
        ExprKind::Binary(op, l, r) => op.is_arith() && untyped_literal(l) && untyped_literal(r),
        ExprKind::Block(b) => {
            b.stmts.is_empty() && b.tail.as_ref().is_some_and(|t| untyped_literal(t))
        }
        ExprKind::If { then, els, .. } => {
            then.stmts.is_empty()
                && then.tail.as_ref().is_some_and(|t| untyped_literal(t))
                && els.as_ref().is_some_and(|e| untyped_literal(e))
        }
        _ => false,
    }
}

/// An expression that cannot write anything: no calls (a call may take
/// `&mut`) and no `&mut` borrows. Built-in methods (`len`, `wrapping_*`) only
/// read.
fn reads_only(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Call { .. }
        | ExprKind::Ref { mutable: true, .. }
        | ExprKind::Block(_)
        | ExprKind::If { .. } => false,
        ExprKind::Method { recv, args, .. } => reads_only(recv) && args.iter().all(reads_only),
        ExprKind::Index { base, index } => reads_only(base) && reads_only(index),
        ExprKind::Unary(_, x) | ExprKind::Ref { expr: x, .. } => reads_only(x),
        ExprKind::Binary(_, l, r) => reads_only(l) && reads_only(r),
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Unit | ExprKind::Var(_) => true,
        _ => false,
    }
}

/// Levenshtein distance between two names (characters).
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != *cb)).min(row[j] + 1).min(cur + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

/// The one candidate nearest to `name` within edit distance 2 (and shorter
/// than the name), or None when there is none or two are equally near.
fn closest<'a>(name: &str, candidates: impl Iterator<Item = &'a String>) -> Option<String> {
    let mut best: Option<(usize, &str)> = None;
    let mut tie = false;
    for c in candidates {
        let d = edit_distance(name, c);
        if d == 0 || d > 2 || d >= name.chars().count() {
            continue;
        }
        match best {
            Some((bd, bc)) if d == bd && c != bc => tie = true,
            Some((bd, _)) if d >= bd => {}
            _ => {
                best = Some((d, c));
                tie = false;
            }
        }
    }
    if tie {
        None
    } else {
        best.map(|(_, c)| c.to_string())
    }
}

/// The variable a place expression starts from: `p` of `p.a.x`, `*p`, `a[i]`.
fn root_name(e: &Expr) -> Option<String> {
    match &e.kind {
        ExprKind::Var(id) => Some(id.name.clone()),
        ExprKind::Deref(x) | ExprKind::Field { base: x, .. } | ExprKind::Index { base: x, .. } => {
            root_name(x)
        }
        _ => None,
    }
}

/// `T::try_from(x)` with `T` an integer type: `T` and the arguments.
pub(crate) fn conv_call(e: &Expr) -> Option<(TypeKind, &[Expr])> {
    let ExprKind::Call { path, args, .. } = &e.kind else {
        return None;
    };
    if path.len() != 2 || path[1].name != "try_from" {
        return None;
    }
    Some((int_type(&path[0].name)?, args))
}

/// `T::from` with `T` an integer type (a Rust habit, E0212).
fn from_call(path: &[Ident]) -> Option<TypeKind> {
    if path.len() == 2 && path[1].name == "from" {
        int_type(&path[0].name)
    } else {
        None
    }
}

/// `T::MIN` / `T::MAX` of an integer type (Rust's associated constants,
/// backlog 2N-06): its type and value.
pub fn int_const(path: &[Ident]) -> Option<(TypeKind, i128)> {
    let [ty, c] = path else { return None };
    let t = int_type(&ty.name)?;
    let (min, max): (i128, i128) = match t {
        TypeKind::I64 => (i64::MIN as i128, i64::MAX as i128),
        TypeKind::U8 => (0, 255),
        _ => (0, u64::MAX as i128),
    };
    match c.name.as_str() {
        "MIN" => Some((t, min)),
        "MAX" => Some((t, max)),
        _ => None,
    }
}

fn int_type(name: &str) -> Option<TypeKind> {
    match name {
        "i64" => Some(TypeKind::I64),
        "u64" => Some(TypeKind::U64),
        "usize" => Some(TypeKind::Usize),
        "u8" => Some(TypeKind::U8),
        _ => None,
    }
}

/// Whether converting a `source` value to `target` has a range obligation
/// (ADR-0028 §1, ADR-0044 §1): every pair but `u64` <-> `usize`, a type to
/// itself, and a `u8` to any other type (every type holds 0..=255).
pub(crate) fn conv_obligation(source: &TypeKind, target: &TypeKind) -> bool {
    let wide = |t: &TypeKind| matches!(t, TypeKind::U64 | TypeKind::Usize);
    source != target && !(wide(source) && wide(target)) && *source != TypeKind::U8
}

/// `Option<t>`.
fn option_of(t: TypeKind) -> TypeKind {
    TypeKind::Named {
        path: vec!["Option".into()],
        args: vec![Type {
            kind: t,
            span: Span::default(),
        }],
    }
}

/// `Box::new` (ADR-0047 §1).
pub fn is_box_new(path: &[Ident]) -> bool {
    path.len() == 2 && path[0].name == "Box" && path[1].name == "new"
}

/// `Vec::new` / `Vec::with_capacity`, and `drop`, `mem::take` and its
/// kin (a Rust habit, E0219).
fn vec_builtin(path: &[Ident]) -> bool {
    let p: Vec<&str> = path.iter().map(|i| i.name.as_str()).collect();
    matches!(
        p.as_slice(),
        ["Vec", "new" | "with_capacity"]
            | ["drop"]
            | ["mem" | "std::mem", "take" | "replace" | "swap"]
            | ["std", "mem", "take" | "replace" | "swap"]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ty_reads_what_ty_str_prints() {
        for s in [
            "i64",
            "u8",
            "bool",
            "()",
            "[i64; 4]",
            "[[u8; 2]; 3]",
            "&[geo::Pt]",
            "&mut u64",
            "Vec<geo::Cell<i64>>",
            "Box<Option<Box<u8>>>",
            "Option<Result<i64, bool>>",
            "geo::Pair<geo::Cell<[bool; 2]>, usize>",
            "T",
        ] {
            let t = parse_ty(s).unwrap_or_else(|| panic!("{s}"));
            assert_eq!(ty_str(&t), s);
        }
        assert!(matches!(parse_ty("T"), Some(TypeKind::Param(_))));
        assert!(parse_ty("geo::<").is_none());
    }
}
