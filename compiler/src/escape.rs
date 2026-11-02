//! Escapes and their rationales in a checked program (ADR-0029 §1, §3,
//! backlog 2E-03, 2E-04): where they are, the hash a rationale is bound to,
//! and whether the hash it carries is still that one.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::check::FnSig;
use crate::source::Span;

/// One `#[rationale(..)]`: where it is, its kind, and the hash it carries.
pub struct Rationale {
    pub span: Span,
    pub kind: Option<String>,
    pub hash: Option<String>,
    /// The attribute's arguments, to rewrite it with another hash.
    pub attr: Attr,
}

/// The escapes of one definition.
#[derive(Default)]
pub struct DefEscapes {
    /// `assume` statements, with their spans (rationale included).
    pub assumes: Vec<Span>,
    pub external: bool,
    pub rationales: Vec<Rationale>,
}

impl DefEscapes {
    /// `assume` statements plus one for `#[external_body]`.
    pub fn count(&self) -> usize {
        self.assumes.len() + usize::from(self.external)
    }
}

/// The hash every rationale of a definition is bound to (ADR-0029 §3):
/// the definition's qualified normal form, which leaves rationales out, and
/// the whole text of each `spec fn` it names, so the rationale goes
/// unconfirmed when the code it speaks for or a predicate it assumes
/// changes, not when its own wording does. 16 hex digits of a SHA-256.
pub struct Hashes<'p> {
    fns: &'p BTreeMap<String, FnSig>,
    defs: BTreeMap<String, (String, &'p FnDef)>,
}

impl<'p> Hashes<'p> {
    pub fn new(p: &'p Program, fns: &'p BTreeMap<String, FnSig>) -> Hashes<'p> {
        let defs = crate::check::defs_of(p, fns)
            .into_iter()
            .map(|(path, unit, f)| (path, (unit, f)))
            .collect();
        Hashes { fns, defs }
    }

    /// The hash of the definition at `path`.
    pub fn of(&self, path: &str) -> Option<String> {
        let (unit, f) = self.defs.get(path)?;
        let own = crate::fmt::qualified_text(f, unit, self.fns, true);
        let named: BTreeSet<&str> = own
            .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
            .filter(|w| *w != path)
            .filter(|w| {
                self.fns
                    .get(*w)
                    .is_some_and(|s| s.kind == FnKind::Spec && !s.builtin)
            })
            .collect();
        let mut text = own.clone();
        for n in named {
            if let Some((u, g)) = self.defs.get(n) {
                text.push_str(&format!("\n{n}\n"));
                text.push_str(&crate::fmt::qualified_text(g, u, self.fns, true));
            }
        }
        let h = crate::sha256::hex(text.as_bytes());
        Some(h.trim_start_matches("sha256:")[..16].to_string())
    }
}

fn rationale(a: &Attr) -> Rationale {
    let mut kind = None;
    let mut hash = None;
    for arg in &a.args {
        if let AttrArg::Pair(k, v) = arg {
            match (k.name.as_str(), &**v) {
                ("kind", AttrArg::Ident(i)) => kind = Some(i.name.clone()),
                ("hash", AttrArg::Str(s, _)) => hash = Some(s.clone()),
                _ => {}
            }
        }
    }
    Rationale {
        span: a.span,
        kind,
        hash,
        attr: a.clone(),
    }
}

/// The escapes of `f`: its `assume` statements (with their rationales),
/// `#[external_body]`, and its own `#[rationale(..)]`.
pub fn of(f: &FnDef) -> DefEscapes {
    let mut out = DefEscapes {
        external: f.attrs.iter().any(|a| a.name.name == "external_body"),
        ..DefEscapes::default()
    };
    if let Some(a) = f.attrs.iter().find(|a| a.name.name == "rationale") {
        out.rationales.push(rationale(a));
    }
    fn block(b: &Block, out: &mut DefEscapes) {
        for s in &b.stmts {
            stmt(s, out);
        }
        if let Some(e) = &b.tail {
            expr(e, out);
        }
    }
    fn stmt(s: &Stmt, out: &mut DefEscapes) {
        match s {
            Stmt::Assume {
                rationale: r, span, ..
            } => {
                out.assumes.push(*span);
                if let Some(a) = r {
                    out.rationales.push(rationale(a));
                }
            }
            Stmt::Let { init, .. } => expr(init, out),
            Stmt::Assign { value, .. } => expr(value, out),
            Stmt::While { body, .. } | Stmt::For { body, .. } => block(body, out),
            Stmt::Proof { body, .. } => block(body, out),
            Stmt::Expr(e) => expr(e, out),
            _ => {}
        }
    }
    fn expr(e: &Expr, out: &mut DefEscapes) {
        match &e.kind {
            ExprKind::Block(b) | ExprKind::Loop { body: b, .. } => block(b, out),
            ExprKind::If { then, els, .. } => {
                block(then, out);
                if let Some(x) = els {
                    expr(x, out);
                }
            }
            ExprKind::Match { arms, .. } => {
                for a in arms {
                    expr(&a.body, out);
                }
            }
            _ => {}
        }
    }
    block(&f.body, &mut out);
    out
}

/// Whether a definition's rationales are all confirmed: `None` without any.
pub fn confirmed(e: &DefEscapes, hash: &str) -> Option<bool> {
    if e.rationales.is_empty() {
        return None;
    }
    Some(e.rationales.iter().all(|r| r.hash.as_deref() == Some(hash)))
}

/// The rationale attribute `a` carrying `hash`, as the normal form writes it
/// (`#[rationale(kind = K, hash = "..", "why")]`).
pub fn with_hash(a: &Attr, hash: &str) -> String {
    let mut kind = None;
    let mut rest = Vec::new();
    for arg in &a.args {
        match arg {
            AttrArg::Pair(k, _) if k.name == "hash" => {}
            AttrArg::Pair(k, v) if k.name == "kind" => {
                if let AttrArg::Ident(i) = &**v {
                    kind = Some(i.name.clone());
                }
            }
            AttrArg::Str(s, _) => rest.push(format!("\"{}\"", crate::fmt::escape(s))),
            _ => {}
        }
    }
    let mut parts = Vec::new();
    if let Some(k) = kind {
        parts.push(format!("kind = {k}"));
    }
    parts.push(format!("hash = \"{hash}\""));
    parts.extend(rest);
    format!("#[rationale({})]", parts.join(", "))
}
