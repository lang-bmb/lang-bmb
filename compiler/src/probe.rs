//! Candidate loop invariants, tried without a commit (ADR-0109, backlog
//! 2D-35, AI-user feedback P2).
//!
//! An agent proposes several candidate invariants for one loop of one
//! definition; the deterministic part filters them. Each candidate is parsed
//! as the tail of a function of its own file (so every diagnostic names the
//! candidate it is about by its file), added to the loop's invariants in a
//! copy of the program, and the definition alone is proved. A candidate that
//! fails on entry (E0806), is not kept by the body (E0807), is undecided
//! there (E0811 of those), or does not check, leaves; the rest are tried
//! again until none leaves (Houdini: the largest subset that is inductive
//! together). Each candidate's answer is reported: a kept one holds, one
//! that left fails (or is undecided) where it did.
//! The program is not changed.

use std::cell::Cell;
use std::collections::BTreeSet;

use crate::ast::{Block, Expr, ExprKind, FnDef, Program, Stmt};
use crate::diag::{Diags, json_str};
use crate::source::SourceMap;

/// Rounds at most (each one proves the definition once): one per candidate
/// that can leave, and the last.
pub const MAX_CANDIDATES: usize = 16;

/// Calls `f` with the start and the invariants of every loop of `body`:
/// statement `while` and `for` (also inside `loop` bodies, `proof` blocks
/// and block expressions) and `loop` expressions, at any depth. One walk
/// for counting and for adding, so the two agree.
fn each_loop(body: &mut Block, f: &mut dyn FnMut(u32, &mut Vec<Expr>)) {
    fn shallow(b: &mut Block, f: &mut dyn FnMut(u32, &mut Vec<Expr>)) {
        for s in &mut b.stmts {
            match s {
                Stmt::While {
                    span,
                    invariants,
                    body,
                    ..
                }
                | Stmt::For {
                    span,
                    invariants,
                    body,
                    ..
                } => {
                    f(span.lo, invariants);
                    shallow(body, f);
                }
                Stmt::Proof { body, .. } => shallow(body, f),
                _ => {}
            }
        }
    }
    shallow(body, f);
    // Every expression, at any depth (also inside statement loops' bodies):
    // its own loop, and the statements of the blocks it holds.
    crate::ast::block_mut(body, &mut |e: &mut Expr| {
        let at = e.span.lo;
        match &mut e.kind {
            ExprKind::Loop {
                invariants, body, ..
            } => {
                f(at, invariants);
                shallow(body, f);
            }
            ExprKind::Block(b) => shallow(b, f),
            ExprKind::If { then, .. } => shallow(then, f),
            ExprKind::Closure(c) => shallow(&mut c.body, f),
            _ => {}
        }
    });
}

/// Where each loop of `f` starts, in source order. A loop inside a closure
/// is not counted: the closure's body is a definition of its own after
/// lowering (ADR-0053), which a proof of `f` does not prove.
fn loop_starts(f: &FnDef) -> Vec<u32> {
    let mut f = f.clone();
    let mut closures: Vec<(u32, u32)> = Vec::new();
    crate::ast::block_mut(&mut f.body, &mut |e: &mut Expr| {
        if let ExprKind::Closure(c) = &e.kind {
            closures.push((c.span.lo, c.span.hi));
        }
    });
    let mut out = BTreeSet::new();
    each_loop(&mut f.body, &mut |lo, _| {
        if !closures.iter().any(|(a, b)| *a <= lo && lo < *b) {
            out.insert(lo);
        }
    });
    out.into_iter().collect()
}

/// Adds `cands` to the invariants of the loop of `body` that starts at
/// `lo`; whether it was there.
fn add_to_loop(body: &mut Block, lo: u32, cands: &[Expr]) -> bool {
    let done = Cell::new(false);
    each_loop(body, &mut |at, inv| {
        if at == lo && !done.get() {
            inv.extend(cands.iter().cloned());
            done.set(true);
        }
    });
    done.get()
}

fn find_fn<'p>(p: &'p mut Program, path: &str) -> Option<&'p mut FnDef> {
    let (unit, name) = path.rsplit_once("::")?;
    let segs: Vec<&str> = unit.split("::").collect();
    let mut mods = &mut p.mods;
    let mut found = None;
    for (i, seg) in segs.iter().enumerate() {
        let m = mods.iter_mut().find(|m| m.name.name == *seg)?;
        if i + 1 == segs.len() {
            found = Some(m);
            break;
        }
        mods = &mut m.mods;
    }
    found?.fns.iter_mut().find(|f| f.name.name == name)
}

/// What one round said of one candidate.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Said {
    Holds,
    Fails,
    Undecided,
}

impl Said {
    fn text(self) -> &'static str {
        match self {
            Said::Holds => "holds",
            Said::Fails => "fails",
            Said::Undecided => "undecided",
        }
    }
}

/// The answer to a probe, as JSONL: per candidate
/// `{"candidate":K,"text","round"?,"entry","kept_by_body","counterexample"?,"error"?,"kept"}`:
/// a kept one `holds` on both; one that left, in `round`, `fails` or
/// `undecided` where it did (`-` for the other); one that does not check,
/// its `error`. Then
/// `{"probe":true,"def","loop","kept":[K..],"verified","rounds"}` where
/// `verified` says whether the definition proves with the kept ones.
/// `Err` when the program does not check, the definition or the loop is
/// not there, or there are more than `MAX_CANDIDATES` candidates.
pub fn probe(
    files: &[(String, String)],
    def: &str,
    loop_index: usize,
    candidates: &[String],
) -> Result<String, String> {
    if candidates.is_empty() || candidates.len() > MAX_CANDIDATES {
        return Err(format!("give 1 to {MAX_CANDIDATES} candidates"));
    }
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut program = Program { mods: Vec::new() };
    for (name, text) in files {
        let id = sm.add(name.clone(), text.clone());
        let toks = crate::lexer::lex(id, text, &mut diags);
        program.mods.extend(
            crate::parser::Parser::new(toks, &mut diags, text)
                .parse_program()
                .mods,
        );
    }
    crate::stdlib::add(&mut sm, &mut program, &mut diags);
    {
        let mut d = diags.clone();
        crate::check::check_resolved(program.clone(), &mut d);
        if d.has_errors() {
            return Err(d.to_jsonl(&sm));
        }
    }
    let starts = {
        let f = find_fn(&mut program, def).ok_or_else(|| format!("no function `{def}`"))?;
        loop_starts(f)
    };
    let Some(&lo) = starts.get(loop_index) else {
        return Err(format!(
            "`{def}` has {} loop(s); `loop` counts them from 0 in source order",
            starts.len()
        ));
    };
    // Each candidate in a file of its own: `fn __probe() -> bool { C }`.
    let mut parsed: Vec<Option<(u32, Expr)>> = Vec::new();
    let mut errors: Vec<Option<String>> = vec![None; candidates.len()];
    for (k, c) in candidates.iter().enumerate() {
        let text = format!(
            "#[summary(\"probe\")]\nmod __probe {{\n    #[effects(pure)]\n    fn __candidate() -> bool {{\n        {c}\n    }}\n}}\n"
        );
        let id = sm.add(format!("<candidate {k}>"), text.clone());
        let mut d = Diags::default();
        let toks = crate::lexer::lex(id, &text, &mut d);
        let mut p = crate::parser::Parser::new(toks, &mut d, &text).parse_program();
        let tail = p
            .mods
            .first_mut()
            .and_then(|m| m.fns.first_mut())
            .and_then(|f| f.body.tail.take());
        match tail {
            Some(e) if !d.has_errors() => parsed.push(Some((id, *e))),
            _ => {
                errors[k] = Some(
                    d.to_jsonl(&sm)
                        .lines()
                        .next()
                        .unwrap_or("does not parse")
                        .to_string(),
                );
                parsed.push(None);
            }
        }
    }
    let only: BTreeSet<String> = [def.to_string()].into();
    let mut kept: Vec<usize> = (0..candidates.len())
        .filter(|k| parsed[*k].is_some())
        .collect();
    // For a candidate that left, the round it left in and what that round
    // said of it. A failure is definite (it has a model); a `holds` beside
    // another candidate's failure is not, since the proof assumes a failed
    // obligation afterwards: so only a kept candidate, in the last round
    // (no candidate fails), is reported as holding.
    type Left = (usize, Said, Said, Option<String>);
    let mut left: Vec<Option<Left>> = vec![None; candidates.len()];
    let mut rounds = 0;
    let mut verified = false;
    loop {
        rounds += 1;
        let mut p = program.clone();
        let cands: Vec<Expr> = kept
            .iter()
            .map(|k| parsed[*k].as_ref().unwrap().1.clone())
            .collect();
        let added = find_fn(&mut p, def).is_some_and(|f| add_to_loop(&mut f.body, lo, &cands));
        if !added {
            return Err(format!(
                "loop {loop_index} of `{def}` could not take the candidates"
            ));
        }
        let mut d = diags.clone();
        let (resolved, tables) = crate::check::check_resolved(p, &mut d);
        let mut leave: BTreeSet<usize> = BTreeSet::new();
        let file_of = |k: usize| parsed[k].as_ref().map(|(id, _)| *id);
        if d.has_errors() {
            for &k in &kept {
                if let Some(x) = d.items.iter().find(|x| Some(x.span.file) == file_of(k)) {
                    errors[k] = Some(x.message.clone());
                    leave.insert(k);
                }
            }
            if leave.is_empty() {
                return Err(d.to_jsonl(&sm));
            }
        } else {
            let out = crate::verify::verify_only(&resolved, &tables, &mut d, &only, None, None);
            for &k in &kept {
                let mine: Vec<_> = d
                    .items
                    .iter()
                    .filter(|x| Some(x.span.file) == file_of(k))
                    .collect();
                let said = |code: &str| {
                    if mine.iter().any(|x| x.code == code) {
                        Said::Fails
                    } else if mine
                        .iter()
                        .any(|x| x.code == "E0811" && x.obligation == Some(code))
                    {
                        Said::Undecided
                    } else {
                        Said::Holds
                    }
                };
                let (entry, body) = (said("E0806"), said("E0807"));
                if entry != Said::Holds || body != Said::Holds {
                    let cex = mine.iter().find_map(|x| {
                        x.message
                            .split_once("; counterexample: ")
                            .map(|(_, c)| c.to_string())
                    });
                    left[k] = Some((rounds, entry, body, cex));
                    leave.insert(k);
                }
            }
            if leave.is_empty() {
                verified = out.statuses.iter().any(|s| s.path == def && s.failed == 0);
            }
        }
        if leave.is_empty() {
            break;
        }
        kept.retain(|k| !leave.contains(k));
        if kept.is_empty() {
            // The definition without any candidate.
            let mut d = diags.clone();
            let (resolved, tables) = crate::check::check_resolved(program.clone(), &mut d);
            let out = crate::verify::verify_only(&resolved, &tables, &mut d, &only, None, None);
            rounds += 1;
            verified = out.statuses.iter().any(|s| s.path == def && s.failed == 0);
            break;
        }
    }
    let mut text = String::new();
    for (k, c) in candidates.iter().enumerate() {
        let mut row = format!("{{\"candidate\":{k},\"text\":{}", json_str(c));
        if let Some(e) = &errors[k] {
            row.push_str(&format!(",\"error\":{}", json_str(e)));
        } else if let Some((round, entry, body, cex)) = &left[k] {
            // Only what failed or was undecided: a `holds` there may be
            // vacuous (see `left`).
            let shown = |s: &Said| if *s == Said::Holds { "-" } else { s.text() };
            row.push_str(&format!(
                ",\"round\":{round},\"entry\":\"{}\",\"kept_by_body\":\"{}\"",
                shown(entry),
                shown(body)
            ));
            if let Some(c) = cex {
                row.push_str(&format!(",\"counterexample\":{}", json_str(c)));
            }
        } else if kept.contains(&k) {
            row.push_str(",\"entry\":\"holds\",\"kept_by_body\":\"holds\"");
        }
        row.push_str(&format!(",\"kept\":{}}}\n", kept.contains(&k)));
        text.push_str(&row);
    }
    let kept_list: Vec<String> = kept.iter().map(|k| k.to_string()).collect();
    text.push_str(&format!(
        "{{\"probe\":true,\"def\":{},\"loop\":{loop_index},\"kept\":[{}],\"verified\":{verified},\"rounds\":{rounds}}}\n",
        json_str(def),
        kept_list.join(",")
    ));
    Ok(text)
}
