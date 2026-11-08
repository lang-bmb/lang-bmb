//! The standard library (ADR-0092 §1, backlog 4B-02): BMB source under
//! `lib/std/`, embedded in the compiler (`build.rs`), proved by the same
//! gate as any program, and added to every program that names it.
//!
//! Each `lib/std/NAME.bmb` holds one unit `mod NAME { .. }`, the library
//! unit `std::NAME`. A program gets the units it names by a path
//! `std::NAME::..` (a call, a type, a pattern, a bound, a `#[uses]`
//! target: `refs::walk`), and the units those name, under one top-level
//! unit `std` after its own units. Their files are named `std/NAME.bmb` and
//! come after the program's files in the source map, so a span of the
//! program's files still indexes the caller's file list. A unit the program
//! does not name is not added, so it is neither checked nor proved there
//! (ADR-0092 open question 1: the shipped proof cache comes with 5C).
//!
//! The unit name `std` is the library's (E0309). The tools that write
//! source leave it alone: the normal form and the workspace's files hold
//! the program's units only, a transaction or a fix does not touch it.

use crate::ast::*;
use crate::diag::{Diag, Diags};
use crate::refs::{Rewrite, Site};
use crate::source::{SourceMap, Span};
use std::collections::BTreeSet;

include!(concat!(env!("OUT_DIR"), "/std_units.rs"));

/// The name of the library's top-level unit.
pub const UNIT: &str = "std";

/// Whether `m` is the library's top-level unit.
pub fn is_std(m: &Mod) -> bool {
    m.name.name == UNIT
}

/// Whether `path` (a definition's or a type's) is in the library.
pub fn in_std(path: &str) -> bool {
    path == UNIT || path.starts_with("std::")
}

/// The library units a path names.
struct Named(BTreeSet<&'static str>);

impl Rewrite for Named {
    fn path(&mut self, segs: &mut Vec<String>, _: Site, _: &[String], _: &[String]) {
        if segs.len() >= 2
            && segs[0] == UNIT
            && let Some((u, _)) = STD_UNITS.iter().find(|(u, _)| *u == segs[1])
        {
            self.0.insert(u);
        }
    }
}

/// The units whose functions are methods of a built-in type: `std::str`'s
/// of text (`s.starts_with(p)`, ADR-0092 §2).
const METHOD_UNITS: &[&str] = &["str"];

/// The function names of library unit `u` (parsed once).
fn fn_names(u: &str) -> &'static BTreeSet<String> {
    static NAMES: std::sync::OnceLock<Vec<(&'static str, BTreeSet<String>)>> =
        std::sync::OnceLock::new();
    let all = NAMES.get_or_init(|| {
        METHOD_UNITS
            .iter()
            .filter_map(|u| STD_UNITS.iter().find(|(n, _)| n == u))
            .map(|(u, text)| {
                let mut d = Diags::default();
                let toks = crate::lexer::lex(0, text, &mut d);
                let p = crate::parser::Parser::new(toks, &mut d, text).parse_program();
                let names = p
                    .mods
                    .iter()
                    .flat_map(|m| &m.fns)
                    .map(|f| f.name.name.clone())
                    .collect();
                (*u, names)
            })
            .collect()
    });
    static EMPTY: BTreeSet<String> = BTreeSet::new();
    all.iter().find(|(n, _)| *n == u).map_or(&EMPTY, |(_, s)| s)
}

/// The library units `p` names: by path, and by a method call named as a
/// function of a unit of `METHOD_UNITS` (whatever the receiver: a unit
/// added for nothing is checked and proved, no more).
fn named(p: &Program) -> BTreeSet<&'static str> {
    let mut n = Named(BTreeSet::new());
    let mut q = p.clone();
    crate::refs::walk(&mut q, &mut n);
    let mut methods = BTreeSet::new();
    crate::ast::exprs_mut(&mut q, &mut |e| {
        if let ExprKind::Method { name, .. } = &e.kind {
            methods.insert(name.name.clone());
        }
    });
    for u in METHOD_UNITS {
        if fn_names(u).iter().any(|f| methods.contains(f)) {
            n.0.insert(u);
        }
    }
    n.0
}

/// Adds to `program` the library units it names, and those they name, as
/// the unit `std` (parsed into `sm` and `diags`). A unit of the program
/// named `std` is E0309 and nothing is added.
pub fn add(sm: &mut SourceMap, program: &mut Program, diags: &mut Diags) {
    if let Some(m) = program.mods.iter().find(|m| is_std(m)) {
        diags.push(
            Diag::new(
                "E0309",
                m.name.span,
                "`std` is the standard library's unit (ADR-0092)",
            )
            .fix(
                "name the unit otherwise; the library's units are `std::str`, `std::num`, ..",
                None,
            ),
        );
        return;
    }
    let mut want = named(program);
    let mut done: BTreeSet<&str> = BTreeSet::new();
    let mut mods = Vec::new();
    let mut first: Option<u32> = None;
    while let Some(u) = want.iter().find(|u| !done.contains(*u)).copied() {
        done.insert(u);
        let Some((_, text)) = STD_UNITS.iter().find(|(n, _)| *n == u) else {
            continue;
        };
        let id = sm.add(format!("std/{u}.bmb"), text.to_string());
        first.get_or_insert(id);
        let toks = crate::lexer::lex(id, text, diags);
        let mut parser = crate::parser::Parser::new(toks, diags, text);
        let parsed = parser.parse_program();
        want.extend(named(&parsed));
        mods.extend(parsed.mods);
    }
    let Some(file) = first else {
        return;
    };
    let span = Span { file, lo: 0, hi: 0 };
    let ident = |name: &str| Ident {
        name: name.into(),
        span,
    };
    program.mods.push(Mod {
        name: ident(UNIT),
        attrs: vec![Attr {
            name: ident("summary"),
            args: vec![AttrArg::Str("표준 라이브러리 (ADR-0092)".into(), span)],
            span,
        }],
        fns: Vec::new(),
        mods,
        types: Vec::new(),
        impls: Vec::new(),
        traits: Vec::new(),
        trait_impls: Vec::new(),
        uses: Vec::new(),
        span,
    });
}

/// `p` without the library's unit: what the normal form and the
/// workspace's files hold.
pub fn without_std(p: &Program) -> Program {
    Program {
        mods: p.mods.iter().filter(|m| !is_std(m)).cloned().collect(),
    }
}

/// `p` with the library units it names, its diagnostics, for a check
/// outside a source map of its own (workspace transactions): the library's
/// files get ids after every file `p`'s units come from.
pub fn with_std(p: &Program) -> (Program, Diags) {
    let mut sm = SourceMap::default();
    let next = p.mods.iter().map(|m| m.span.file + 1).max().unwrap_or(0);
    for k in 0..next {
        sm.add(format!("<file {k}>"), String::new());
    }
    let mut q = p.clone();
    let mut d = Diags::default();
    add(&mut sm, &mut q, &mut d);
    (q, d)
}
