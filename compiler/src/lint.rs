//! Contract lint (DESIGN.md §8.2 "계약 조정", backlog 2D-08).
//!
//! Three deterministic reports, as warning diagnostics (`W09xx`):
//!
//! - `W0901` the `requires` clauses of a function cannot hold together
//!   (with the definitions its `#[uses]` reveals, in the prover's own
//!   encoding, `verify::preconditions_satisfiable`). Such a function can
//!   never be called and every proof about it holds vacuously. An undecided
//!   query reports nothing.
//! - `W0902` a `requires` clause the function's proof does not need: with
//!   that one clause removed from the program, the function still verifies
//!   completely. Removing a precondition never breaks a caller (callers only
//!   have less to prove), so the clause can be dropped or weakened.
//! - `W0903` an `ensures` clause no caller's proof uses: with that one clause
//!   removed, every caller still verifies completely. Only functions that
//!   have callers, all of them verified, are examined.
//!
//! Clauses are removed from the checked program itself (the syntax tree;
//! nothing else changes, so the type tables still apply) and only the
//! affected definitions are proved again (`verify::verify_defs`), through an
//! in-memory proof cache. Each clause is judged on its own: two clauses
//! reported separately may not be removable together. Re-verifications are
//! capped (`MAX_PRE_TRIALS`, `MAX_POST_TRIALS`); clauses beyond the cap are
//! counted as `skipped` in the summary, in a fixed (path, clause) order. It
//! runs only on request (`bmb lint`); `check` and `verify` are unchanged.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{FnDef, FnKind, Mod, Program};
use crate::cache::ProofCache;
use crate::diag::{Diag, Diags};
use crate::source::Span;

/// Upper bound on re-verifications for unused-precondition checks (W0902).
pub const MAX_PRE_TRIALS: usize = 32;
/// Upper bound on re-verifications for unused-postcondition checks (W0903).
pub const MAX_POST_TRIALS: usize = 32;

pub struct LintOutput {
    /// Lint diagnostics as deterministic JSONL (plus any `E0812` when the
    /// proof gate cannot run).
    pub diagnostics: String,
    /// Executable functions examined.
    pub defs: usize,
    /// Lint diagnostics reported.
    pub reports: usize,
    /// Restricted re-verifications run (not counting the first full one).
    pub verifications: usize,
    /// Clauses not examined because a cap was reached.
    pub skipped: usize,
    /// False when the proof gate could not run.
    pub ok: bool,
}

impl LintOutput {
    /// JSONL: the diagnostics, then
    /// `{"summary":true,"defs","reports","verifications","skipped"}`.
    pub fn report(&self) -> String {
        format!(
            "{}{{\"summary\":true,\"defs\":{},\"reports\":{},\"verifications\":{},\"skipped\":{}}}\n",
            self.diagnostics, self.defs, self.reports, self.verifications, self.skipped
        )
    }
}

fn collect<'p>(m: &'p Mod, prefix: &str, out: &mut BTreeMap<String, &'p FnDef>) {
    let mp = format!("{prefix}{}", m.name.name);
    for s in &m.mods {
        collect(s, &format!("{mp}::"), out);
    }
    for f in &m.fns {
        out.entry(format!("{mp}::{}", f.name.name)).or_insert(f);
    }
}

/// The function at `path` in `p`, mutably (first definition, as `collect`).
fn find_mut<'p>(p: &'p mut Program, path: &str) -> Option<&'p mut FnDef> {
    fn walk<'p>(m: &'p mut Mod, prefix: &str, path: &str) -> Option<&'p mut FnDef> {
        let mp = format!("{prefix}{}", m.name.name);
        let rest = path.strip_prefix(&format!("{mp}::"))?;
        if let Some(i) = m.fns.iter().position(|f| f.name.name == rest) {
            return Some(&mut m.fns[i]);
        }
        let sub = format!("{mp}::");
        m.mods.iter_mut().find_map(|s| walk(s, &sub, path))
    }
    p.mods.iter_mut().find_map(|m| walk(m, "", path))
}

/// Definitions in `paths` that verify completely: status with no failed
/// obligation and no diagnostic inside their span.
fn clean(
    outcome: &crate::verify::Outcome,
    diags: &Diags,
    spans: &BTreeMap<String, Span>,
    paths: &BTreeSet<String>,
) -> bool {
    paths.iter().all(|p| {
        let sp = spans[p];
        outcome
            .statuses
            .iter()
            .any(|s| &s.path == p && s.failed == 0)
            && !diags
                .items
                .iter()
                .any(|d| d.span.file == sp.file && d.span.lo >= sp.lo && d.span.hi <= sp.hi)
    })
}

/// Runs the contract lint (see the module comment). `Err` carries the
/// diagnostics when the program does not check. Deterministic: the result
/// depends only on the input.
pub fn lint_sources(files: &[(String, String)]) -> Result<LintOutput, String> {
    let mut fe = crate::frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let mut cache = ProofCache::default();
    let base = crate::verify::verify_program_with(
        &fe.program,
        &fe.tables,
        &mut fe.diags,
        0,
        Some(&mut cache),
    );

    let mut out = Diags::default();
    let gate_down: Vec<Diag> = fe
        .diags
        .items
        .iter()
        .filter(|d| d.code == "E0812")
        .cloned()
        .collect();
    let ok = gate_down.is_empty();
    for d in gate_down {
        out.push(d);
    }

    let mut defs: BTreeMap<String, &FnDef> = BTreeMap::new();
    for m in &fe.program.mods {
        collect(m, "", &mut defs);
    }
    let spans: BTreeMap<String, Span> = defs.iter().map(|(p, f)| (p.clone(), f.span)).collect();
    let exec: Vec<&String> = defs
        .iter()
        .filter(|(_, f)| f.kind == FnKind::Exec)
        .map(|(p, _)| p)
        .collect();
    let clause_text =
        |sp: Span| files[sp.file as usize].1[sp.lo as usize..sp.hi as usize].to_string();

    // W0901: unsatisfiable preconditions.
    let mut unsat = BTreeSet::new();
    for (path, span, verdict) in crate::verify::preconditions_satisfiable(&fe.program, &fe.tables) {
        if verdict == Some(false) {
            out.push(
                Diag::new(
                    "W0901",
                    span,
                    format!(
                        "the preconditions of `{path}` cannot hold together (with the definitions its #[uses] reveals): no call can satisfy them, and every proof about `{path}` holds vacuously"
                    ),
                )
                .fix("weaken or correct the preconditions", None),
            );
            unsat.insert(path);
        }
    }

    let verified = |p: &String| {
        let one: BTreeSet<String> = [p.clone()].into();
        clean(&base, &fe.diags, &spans, &one)
    };
    let mut verifications = 0;
    let mut skipped = 0;

    // W0902: preconditions the function's own proof does not need.
    let mut trials = 0;
    for path in &exec {
        let f = defs[*path];
        if f.requires.is_empty() || unsat.contains(*path) || !verified(path) {
            continue;
        }
        for i in 0..f.requires.len() {
            if trials == MAX_PRE_TRIALS {
                skipped += 1;
                continue;
            }
            trials += 1;
            verifications += 1;
            let mut prog = fe.program.clone();
            find_mut(&mut prog, path)
                .expect("definition exists")
                .requires
                .remove(i);
            let only: BTreeSet<String> = [(*path).clone()].into();
            let mut diags = Diags::default();
            let o =
                crate::verify::verify_defs(&prog, &fe.tables, &mut diags, &only, Some(&mut cache));
            if clean(&o, &diags, &spans, &only) {
                let sp = f.requires[i].span;
                out.push(
                    Diag::new(
                        "W0902",
                        sp,
                        format!(
                            "precondition `{}` is not needed by this function's proof: `{path}` verifies without it",
                            clause_text(sp)
                        ),
                    )
                    .fix(
                        "remove or weaken it if it does not state intent; callers then have less to prove",
                        None,
                    ),
                );
            }
        }
    }

    // W0903: postconditions no caller's proof uses.
    let model = crate::map::Model::build(
        &fe.program,
        &fe.tables,
        &fe.sm,
        &crate::verify::Outcome::default(),
    );
    let mut callers: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for e in model.edges.iter().filter(|e| e.kind == "calls") {
        if defs.contains_key(&e.from) {
            callers
                .entry(e.to.as_str())
                .or_default()
                .insert(e.from.clone());
        }
    }
    let mut trials = 0;
    for path in &exec {
        let f = defs[*path];
        let Some(cs) = callers.get(path.as_str()) else {
            continue;
        };
        if f.ensures.is_empty() || !clean(&base, &fe.diags, &spans, cs) {
            continue;
        }
        for i in 0..f.ensures.len() {
            if trials == MAX_POST_TRIALS {
                skipped += 1;
                continue;
            }
            trials += 1;
            verifications += 1;
            let mut prog = fe.program.clone();
            find_mut(&mut prog, path)
                .expect("definition exists")
                .ensures
                .remove(i);
            let mut diags = Diags::default();
            let o = crate::verify::verify_defs(&prog, &fe.tables, &mut diags, cs, Some(&mut cache));
            if clean(&o, &diags, &spans, cs) {
                let sp = f.ensures[i].span;
                let n = cs.len();
                out.push(
                    Diag::new(
                        "W0903",
                        sp,
                        format!(
                            "postcondition `{}` is not used by any caller's proof: {n} caller{} of `{path}` verif{} without it",
                            clause_text(sp),
                            if n == 1 { "" } else { "s" },
                            if n == 1 { "ies" } else { "y" },
                        ),
                    )
                    .fix(
                        "keep it if it states intent for future callers; otherwise it can be removed or weakened",
                        None,
                    ),
                );
            }
        }
    }

    let reports = out.items.iter().filter(|d| d.code.starts_with('W')).count();
    Ok(LintOutput {
        diagnostics: out.to_jsonl(&fe.sm),
        defs: exec.len(),
        reports,
        verifications,
        skipped,
        ok,
    })
}
