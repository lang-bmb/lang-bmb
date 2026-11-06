//! Contract lint (DESIGN.md §8.2 "계약 조정", backlog 2D-08).
//!
//! Six deterministic reports, as warning diagnostics (`W09xx`):
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
//! - `W0904` a single change to a function's body (backlog 2D-09) that its
//!   specification does not distinguish: the mutated function still
//!   verifies completely. Changes: `+`/`-`, `<`/`<=`, `>`/`>=`, `==`/`!=`,
//!   `/`/`%`, `&&`/`||` swapped, `*` replaced by `+` (backlog 2D-16), an
//!   integer literal `c` replaced by `c + 1`, a `bool` literal flipped, an
//!   `if` condition negated, an index `a[i]` replaced by `a[i + 1]` and by
//!   `a[i - 1]` (two mutants; often killed by the bounds proof, which is a
//!   kill like any other), each in executable code only (not in loop
//!   invariants, measures or `assert`). Only verified functions
//!   with an `ensures` clause and satisfiable preconditions are mutated.
//!   Equivalence of a mutant is not decided, so a report may be an
//!   equivalent change (informational). At most `MAX_MUTANTS` mutants per
//!   run, taken round-robin over functions in path order (site order within
//!   a function); the rest are counted as `mutants_skipped`.
//! - `W0905` a loop exit or the end of the body of a verified function that
//!   its own proof cannot reach (`verify::vacuous_ends`, backlog 1B-44):
//!   what follows holds vacuously. No honest program yields one.
//!
//! W0902, W0904 and W0905 skip `#[external_body]` functions (ADR-0029 §1):
//! their body is not proved, so every mutant and every dropped clause would
//! "still verify" (backlog 2E-05).
//! - `W0909` an executable function declares an effect its body does not
//!   need (ADR-0030 §3): its own operations and its callees' declared
//!   effects need none of it. Syntactic, no proof involved.
//! - `W0910` a loop of an `io` function that prints and carries state (a
//!   local declared before the loop and assigned in it) whose every update
//!   is pure (backlog 2D-30, from T0-34): the update can be a pure function
//!   with a contract, proved, and only the printing shell needs a test.
//!   Syntactic.
//! - `W0906` a `forall` whose one trigger term and another term of its body
//!   read the same array (or call the same function) at different arguments
//!   that mention the bound variables, as `#[trigger] out[k + 1] == out[k] +
//!   a[k + 1]` (backlog 1B-45): each instance yields a term the trigger
//!   matches again, a matching loop that can run the solver to its limit.
//!   Syntactic, no proof involved. A `forall` with several trigger terms
//!   (one multi-pattern) is not reported.
//!
//! Clauses are removed from the checked program itself (the syntax tree;
//! nothing else changes, so the type tables still apply) and only the
//! affected definitions are proved again (`verify::verify_defs`), through a
//! proof cache (in memory, or `DIR/proofs` with `--cache DIR`; with it, the
//! first whole-program verification also goes through the incremental cache,
//! backlog 2D-15). Each clause is judged on its own: two clauses
//! reported separately may not be removable together. Re-verifications are
//! capped (`MAX_PRE_TRIALS`, `MAX_POST_TRIALS`); clauses beyond the cap are
//! counted as `skipped` in the summary, in a fixed (path, clause) order. It
//! runs only on request (`bmb lint`); `check` and `verify` are unchanged.
//!
//! `bmb lint --select CODES` runs only the listed checks (backlog T2-12):
//! each reports what it reports in a full run, and the first verification
//! and the precondition check run only when a selected check needs them.
//! The summary carries the solver cost of the two satisfiability checks
//! (`verify::SatCost`): W0901's and W0905's `rlimit` and undecided count,
//! `null` when the check did not run.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::ast::{
    BinOp, Block, Expr, ExprKind, FnDef, FnKind, Ident, IntTy, Mod, Program, Radix, Stmt, TypeKind,
    UnOp,
};
use crate::cache::ProofCache;
use crate::diag::{Diag, Diags};
use crate::incremental::Session;
use crate::source::Span;
use crate::verify::SatCost;

/// Upper bound on re-verifications for unused-precondition checks (W0902).
pub const MAX_PRE_TRIALS: usize = 32;
/// Upper bound on re-verifications for unused-postcondition checks (W0903).
pub const MAX_POST_TRIALS: usize = 32;
/// Upper bound on mutants verified per run (W0904).
pub const MAX_MUTANTS: usize = 64;

/// The checks of the lint, by the code each reports (`bmb lint --select`).
pub const CODES: [&str; 11] = [
    "W0901", "W0902", "W0903", "W0904", "W0905", "W0906", "W0907", "W0909", "W0910", "W1001",
    "W1002",
];

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
    /// Mutants verified (W0904), not counted in `verifications`.
    pub mutants: usize,
    /// Mutants that still verified (each one a W0904).
    pub surviving: usize,
    /// Mutants not verified because `MAX_MUTANTS` was reached.
    pub mutants_skipped: usize,
    /// What the precondition check (W0901) cost, if it ran (for W0901, or
    /// for the checks that skip the functions it refutes).
    pub pre_cost: Option<SatCost>,
    /// What the reach-point check (W0905) cost, if it ran.
    pub reach_cost: Option<SatCost>,
    /// False when the proof gate could not run.
    pub ok: bool,
}

impl LintOutput {
    /// JSONL: the diagnostics, then
    /// `{"summary":true,"defs","reports","verifications","skipped","mutants","surviving","mutants_skipped","pre_rlimit","pre_undecided","reach_rlimit","reach_undecided"}`.
    pub fn report(&self) -> String {
        let cost = |c: Option<SatCost>| {
            c.map_or(("null".to_string(), "null".to_string()), |c| {
                (c.rlimit.to_string(), c.undecided.to_string())
            })
        };
        let (pre_rlimit, pre_undecided) = cost(self.pre_cost);
        let (reach_rlimit, reach_undecided) = cost(self.reach_cost);
        format!(
            "{}{{\"summary\":true,\"defs\":{},\"reports\":{},\"verifications\":{},\"skipped\":{},\"mutants\":{},\"surviving\":{},\"mutants_skipped\":{},\"pre_rlimit\":{pre_rlimit},\"pre_undecided\":{pre_undecided},\"reach_rlimit\":{reach_rlimit},\"reach_undecided\":{reach_undecided}}}\n",
            self.diagnostics,
            self.defs,
            self.reports,
            self.verifications,
            self.skipped,
            self.mutants,
            self.surviving,
            self.mutants_skipped
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
    lint_sources_cached(files, None)
}

/// As `lint_sources`, with the proof cache and the incremental cache kept in
/// `cache_dir` (as `bmb verify --cache`, backlog 2D-15): the first, whole
/// program verification reuses every unchanged definition, and the
/// restricted re-verifications reuse proved obligations from earlier runs.
/// The result is the same.
pub fn lint_sources_cached(
    files: &[(String, String)],
    cache_dir: Option<&Path>,
) -> Result<LintOutput, String> {
    lint_sources_selected(files, cache_dir, &CODES)
}

/// As `lint_sources_cached`, running only the checks whose codes `select`
/// lists (`CODES`; backlog T2-12). Each selected check reports exactly what
/// it reports in a full run. The first verification runs only for the
/// checks that judge verified functions (W0902 to W0905), the precondition
/// check also for the ones that skip its unsatisfiable functions (W0902,
/// W0904, W0905); a summary count of a check that did not run is 0, the
/// cost of a satisfiability check that did not run `null`.
pub fn lint_sources_selected(
    files: &[(String, String)],
    cache_dir: Option<&Path>,
    select: &[&str],
) -> Result<LintOutput, String> {
    let mut fe = crate::frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let runs = |code: &str| select.contains(&code);
    let needs_base = ["W0902", "W0903", "W0904", "W0905", "W1002"]
        .iter()
        .any(|c| runs(c));
    let needs_pre = ["W0901", "W0902", "W0904", "W0905"].iter().any(|c| runs(c));
    let mut cache = cache_dir.map(ProofCache::load).unwrap_or_default();
    // Only the first verification covers the whole program; the restricted
    // ones below each change the definitions they prove, so the incremental
    // cache serves this one alone (and records it, with `cache_dir`).
    let base = match cache_dir {
        _ if !needs_base => {
            // No first verification: a solver that cannot run is still
            // reported (E0812), as the verification would, when W0901 runs.
            if needs_pre && let Err(e) = crate::toolchain::check_z3() {
                let at = fe
                    .program
                    .mods
                    .first()
                    .map_or(Span::default(), |m| m.name.span);
                fe.diags.push(Diag::new(
                    "E0812",
                    at,
                    format!("proof gate cannot run: {e}"),
                ));
            }
            crate::verify::Outcome::default()
        }
        Some(d) => {
            let mut session = Session::new(d, &fe.program, &fe.tables, &fe.sm);
            let o = crate::verify::verify_program_incremental(
                &fe.program,
                &fe.tables,
                &mut fe.diags,
                Some(&mut cache),
                &mut session,
            );
            session.save();
            o
        }
        None => crate::verify::verify_program_with(
            &fe.program,
            &fe.tables,
            &mut fe.diags,
            0,
            Some(&mut cache),
        ),
    };

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
    let (pre, pre_cost) = if needs_pre {
        let (v, c) = crate::verify::preconditions_satisfiable(&fe.program, &fe.tables);
        (v, Some(c))
    } else {
        (Vec::new(), None)
    };
    for (path, span, verdict) in pre {
        if verdict != Some(false) {
            continue;
        }
        if runs("W0901") {
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
        }
        unsat.insert(path);
    }

    // An `#[external_body]` body is not proved (ADR-0029 §1): what its proof
    // needs, reaches or tells apart says nothing about it.
    let external = |p: &String| fe.tables.external.contains(p);
    let verified = |p: &String| {
        let one: BTreeSet<String> = [p.clone()].into();
        clean(&base, &fe.diags, &spans, &one)
    };

    // W0905: points of verified functions their own proof cannot reach.
    let mut reach_cost = None;
    if runs("W0905") {
        let (points, cost) = crate::verify::vacuous_ends(&fe.program, &fe.tables);
        reach_cost = Some(cost);
        for (path, span, what) in points {
            if unsat.contains(&path) || external(&path) || !verified(&path) {
                continue;
            }
            out.push(w0905(&path, span, what));
        }
    }
    let mut verifications = 0;
    let mut skipped = 0;

    // W0902: preconditions the function's own proof does not need.
    let mut trials = 0;
    for path in &exec {
        let f = defs[*path];
        if !runs("W0902")
            || f.requires.is_empty()
            || unsat.contains(*path)
            || external(path)
            || !verified(path)
        {
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
        let Some(cs) = callers.get(path.as_str()).filter(|_| runs("W0903")) else {
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

    // W0904: body mutations the specification does not distinguish.
    let mut plan: Vec<(&String, Vec<Site>)> = Vec::new();
    for path in &exec {
        let f = defs[*path];
        if !runs("W0904")
            || f.ensures.is_empty()
            || unsat.contains(*path)
            || external(path)
            || !verified(path)
        {
            continue;
        }
        let text = &files[f.span.file as usize].1;
        plan.push((path, sites(f, text)));
    }
    // Round-robin over definitions so the cap spreads across them.
    let mut order: Vec<(usize, usize)> = Vec::new();
    let rounds = plan.iter().map(|(_, s)| s.len()).max().unwrap_or(0);
    for k in 0..rounds {
        for (i, (_, s)) in plan.iter().enumerate() {
            if k < s.len() {
                order.push((i, k));
            }
        }
    }
    let mutants = order.len().min(MAX_MUTANTS);
    let mutants_skipped = order.len() - mutants;
    let mut surviving = 0;
    for &(i, k) in &order[..mutants] {
        let (path, ss) = &plan[i];
        let f = defs[*path];
        let text = &files[f.span.file as usize].1;
        let mut prog = fe.program.clone();
        apply_site(
            find_mut(&mut prog, path).expect("definition exists"),
            text,
            k,
        );
        let only: BTreeSet<String> = [(*path).clone()].into();
        let mut diags = Diags::default();
        let o = crate::verify::verify_defs(&prog, &fe.tables, &mut diags, &only, Some(&mut cache));
        if clean(&o, &diags, &spans, &only) {
            surviving += 1;
            let s = &ss[k];
            out.push(
                Diag::new(
                    "W0904",
                    s.span,
                    format!(
                        "the specification does not distinguish this change: `{}` -> `{}` (`{path}` still verifies with it)",
                        s.before, s.after
                    ),
                )
                .fix(
                    "strengthen the postcondition so that it rules this change out, unless the change is equivalent on every input the precondition allows",
                    None,
                ),
            );
        }
    }

    // W0906: self-matching triggers (syntactic).
    for f in defs.values().filter(|_| runs("W0906")) {
        let text = &files[f.span.file as usize].1;
        for d in self_matching(f, text) {
            out.push(d);
        }
    }

    // W0907: counting `while` loops with a canonical `for` (syntactic).
    if runs("W0907") {
        for d in canonical_loops(&fe.program, files) {
            out.push(d);
        }
    }

    // W0909: declared effects the body does not need (syntactic).
    if runs("W0909") {
        for d in unneeded_effects(&fe.tables, &defs) {
            out.push(d);
        }
    }

    // W0910: pure state updates in printing loops (syntactic).
    if runs("W0910") {
        for d in io_loop_cores(&fe.tables, &defs) {
            out.push(d);
        }
    }

    // W1001, W1002: rationales and escapes (ADR-0029 §3, §2).
    if runs("W1001") {
        for d in unconfirmed_rationales(&fe.program, &fe.tables, true) {
            out.push(d);
        }
    }
    if runs("W1002") {
        for d in needless_assumes(&base, &defs) {
            out.push(d);
        }
    }

    if let Some(d) = cache_dir
        && let Err(e) = cache.save(d)
    {
        eprintln!("bmb: cannot write proof cache in {}: {e}", d.display());
    }

    let reports = out.items.iter().filter(|d| d.code.starts_with('W')).count();
    Ok(LintOutput {
        diagnostics: out.to_jsonl(&fe.sm),
        defs: exec.len(),
        reports,
        verifications,
        skipped,
        mutants,
        surviving,
        mutants_skipped,
        pre_cost,
        reach_cost,
        ok,
    })
}

// ---------- mutation sites (W0904) ----------

/// One single-node change to an executable body.
struct Site {
    /// The changed node (the diagnostic span).
    span: Span,
    /// The text shown before and after the change.
    before: String,
    after: String,
}

/// The mutated operator of a binary node, if it has one. `*` becomes `+`
/// (not the other way: `+` already becomes `-`, and a second mutant for the
/// most common operator would crowd the `MAX_MUTANTS` budget).
fn swap_op(op: BinOp) -> Option<BinOp> {
    Some(match op {
        BinOp::Add => BinOp::Sub,
        BinOp::Sub => BinOp::Add,
        BinOp::Mul => BinOp::Add,
        BinOp::Div => BinOp::Rem,
        BinOp::Rem => BinOp::Div,
        BinOp::And => BinOp::Or,
        BinOp::Or => BinOp::And,
        BinOp::Lt => BinOp::Le,
        BinOp::Le => BinOp::Lt,
        BinOp::Gt => BinOp::Ge,
        BinOp::Ge => BinOp::Gt,
        BinOp::Eq => BinOp::Ne,
        BinOp::Ne => BinOp::Eq,
        BinOp::BitAnd => BinOp::BitOr,
        BinOp::BitOr => BinOp::BitAnd,
        BinOp::BitXor => BinOp::BitOr,
        BinOp::Shl => BinOp::Shr,
        BinOp::Shr => BinOp::Shl,
        _ => return None,
    })
}

/// Largest literal that is replaced by `c + 1` (so `c + 1` stays in range
/// for every integer type).
const MAX_LIT: u128 = i64::MAX as u128 - 1;

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Index offsets tried at an index node `a[i]`: `a[i + 1]`, then `a[i - 1]`.
const INDEX_SHIFTS: [BinOp; 2] = [BinOp::Add, BinOp::Sub];

/// The sites at node `e`, described against its file's `text`, in the order
/// `mutate` numbers them. `parent` is the span of the enclosing operator
/// node, shown as context for a changed literal.
fn sites_of(e: &Expr, parent: Option<Span>, text: &str) -> Vec<Site> {
    if let ExprKind::Index { index, .. } = &e.kind {
        if index.span.lo < e.span.lo || e.span.hi < index.span.hi {
            return Vec::new();
        }
        let src = &text[e.span.lo as usize..e.span.hi as usize];
        let (a, b) = (
            (index.span.lo - e.span.lo) as usize,
            (index.span.hi - e.span.lo) as usize,
        );
        let i = &src[a..b];
        return INDEX_SHIFTS
            .iter()
            .map(|op| Site {
                span: e.span,
                before: squash(src),
                after: squash(&format!("{}{i} {} 1{}", &src[..a], op.symbol(), &src[b..])),
            })
            .collect();
    }
    site_of(e, parent, text).into_iter().collect()
}

/// The single site at a non-index node `e`, if `e` is one.
fn site_of(e: &Expr, parent: Option<Span>, text: &str) -> Option<Site> {
    let src = |sp: Span| &text[sp.lo as usize..sp.hi as usize];
    // The text of `ctx` with bytes `lo..hi` replaced by `new`.
    let splice = |ctx: Span, lo: u32, hi: u32, new: &str| {
        let c = src(ctx);
        let (a, b) = ((lo - ctx.lo) as usize, (hi - ctx.lo) as usize);
        (squash(c), squash(&format!("{}{new}{}", &c[..a], &c[b..])))
    };
    let (before, after) = match &e.kind {
        ExprKind::Binary(op, l, r) => {
            let new = swap_op(*op)?;
            let at = (l.span.hi <= r.span.lo)
                .then(|| text[l.span.hi as usize..r.span.lo as usize].find(op.symbol()))
                .flatten();
            match at {
                Some(k) if e.span.lo <= l.span.hi && r.span.hi <= e.span.hi => {
                    let lo = l.span.hi + k as u32;
                    splice(e.span, lo, lo + op.symbol().len() as u32, new.symbol())
                }
                _ => (op.symbol().to_string(), new.symbol().to_string()),
            }
        }
        ExprKind::Int(c, radix) if *c <= MAX_LIT => {
            let ctx = parent
                .filter(|p| p.lo <= e.span.lo && e.span.hi <= p.hi)
                .unwrap_or(e.span);
            splice(ctx, e.span.lo, e.span.hi, &radix.text(c + 1))
        }
        ExprKind::Bool(b) => (b.to_string(), (!b).to_string()),
        ExprKind::If { cond, .. } => {
            let c = squash(src(cond.span));
            (c.clone(), format!("!({c})"))
        }
        _ => return None,
    };
    Some(Site {
        span: e.span,
        before,
        after,
    })
}

/// Applies the `variant`-th change `sites_of` describes at node `e`.
fn mutate(e: &mut Expr, variant: usize) {
    match &mut e.kind {
        // `i` becomes `i + 1` or `i - 1`. The new nodes take the span of `i`,
        // so the type tables (keyed by span) give them the index type.
        ExprKind::Index { index, .. } => {
            let span = index.span;
            let inner = std::mem::replace(
                index.as_mut(),
                Expr {
                    kind: ExprKind::Unit,
                    span,
                },
            );
            **index = Expr {
                kind: ExprKind::Binary(
                    INDEX_SHIFTS[variant],
                    Box::new(inner),
                    Box::new(Expr {
                        kind: ExprKind::Int(1, Radix::Dec),
                        span,
                    }),
                ),
                span,
            };
        }
        ExprKind::Binary(op, _, _) => *op = swap_op(*op).expect("a mutation site"),
        ExprKind::Int(c, _) => *c += 1,
        ExprKind::Bool(b) => *b = !*b,
        ExprKind::If { cond, .. } => {
            let span = cond.span;
            let inner = std::mem::replace(
                cond.as_mut(),
                Expr {
                    kind: ExprKind::Unit,
                    span,
                },
            );
            **cond = Expr {
                kind: ExprKind::Unary(UnOp::Not, Box::new(inner)),
                span,
            };
        }
        _ => unreachable!("not a mutation site"),
    }
}

/// Visits the executable expressions of a body in a fixed pre-order.
/// Specification parts (loop invariants and measures, `assert`, `old`,
/// `forall`) are left out. `f` gets each node and, when the node is an
/// operand, the span of its operator node.
fn walk_block(b: &mut Block, f: &mut dyn FnMut(&mut Expr, Option<Span>)) {
    for s in &mut b.stmts {
        match s {
            Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => walk_expr(init, None, f),
            Stmt::Assign { target, value, .. } => {
                walk_expr(target, None, f);
                walk_expr(value, None, f);
            }
            Stmt::While { cond, body, .. } => {
                walk_expr(cond, None, f);
                walk_block(body, f);
            }
            Stmt::For { lo, hi, body, .. } => {
                walk_expr(lo, None, f);
                walk_expr(hi, None, f);
                walk_block(body, f);
            }
            Stmt::Assert { .. } | Stmt::Assume { .. } | Stmt::Proof { .. } => {}
            Stmt::Expr(e) => walk_expr(e, None, f),
        }
    }
    if let Some(t) = &mut b.tail {
        walk_expr(t, None, f);
    }
}

fn walk_expr(e: &mut Expr, parent: Option<Span>, f: &mut dyn FnMut(&mut Expr, Option<Span>)) {
    f(e, parent);
    let here = e.span;
    match &mut e.kind {
        ExprKind::Call { args, .. } => args.iter_mut().for_each(|a| walk_expr(a, None, f)),
        ExprKind::Method { recv, args, .. } => {
            walk_expr(recv, None, f);
            args.iter_mut().for_each(|a| walk_expr(a, None, f));
        }
        ExprKind::Index { base, index } => {
            walk_expr(base, None, f);
            walk_expr(index, None, f);
        }
        ExprKind::Unary(_, x) => walk_expr(x, Some(here), f),
        ExprKind::Ref { expr, .. } => walk_expr(expr, None, f),
        ExprKind::Range { lo: l, hi: r } => {
            walk_expr(l, Some(here), f);
            walk_expr(r, Some(here), f);
        }
        ExprKind::Binary(_, l, r) => {
            walk_expr(l, Some(here), f);
            walk_expr(r, Some(here), f);
        }
        ExprKind::If { cond, then, els } => {
            walk_expr(cond, None, f);
            walk_block(then, f);
            if let Some(x) = els {
                walk_expr(x, None, f);
            }
        }
        ExprKind::Block(b) => walk_block(b, f),
        ExprKind::Array(xs) => xs.iter_mut().for_each(|x| walk_expr(x, None, f)),
        ExprKind::Repeat { value, .. } => walk_expr(value, None, f),
        ExprKind::Field { base, .. } | ExprKind::Deref(base) | ExprKind::Try(base) => {
            walk_expr(base, None, f)
        }
        ExprKind::StructLit { fields, .. } => {
            fields.iter_mut().for_each(|(_, x)| walk_expr(x, None, f))
        }
        ExprKind::Match { scrut, arms } => {
            walk_expr(scrut, None, f);
            arms.iter_mut()
                .for_each(|a| walk_expr(&mut a.body, None, f));
        }
        ExprKind::Return(x) | ExprKind::Break { value: x, .. } => {
            if let Some(x) = x {
                walk_expr(x, None, f);
            }
        }
        ExprKind::Loop { body, .. } => walk_block(body, f),
        ExprKind::Continue { .. } => {}
        ExprKind::Path(_)
        | ExprKind::Old(_)
        | ExprKind::Quant { .. }
        | ExprKind::Trigger(_)
        | ExprKind::Int(..)
        | ExprKind::ByteLit(_)
        | ExprKind::Float(_)
        | ExprKind::ByteStr(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Var(_)
        | ExprKind::Error => {}
    }
}

/// The mutation sites of `f`'s body, in visiting order.
fn sites(f: &FnDef, text: &str) -> Vec<Site> {
    let mut body = f.body.clone();
    let mut out = Vec::new();
    walk_block(&mut body, &mut |e, parent| {
        out.extend(sites_of(e, parent, text));
    });
    out
}

/// Applies the `k`-th site of `f`'s body, numbered as `sites` numbers them.
fn apply_site(f: &mut FnDef, text: &str, k: usize) {
    let mut i = 0;
    walk_block(&mut f.body, &mut |e, parent| {
        let n = sites_of(e, parent, text).len();
        if i <= k && k < i + n {
            mutate(e, k - i);
        }
        i += n;
    });
}

// ---------- mutants as programs (`bmb mutate`, backlog 2A-10) ----------

/// One W0904 change applied to one function, as program text (the hidden
/// mutant sets of the spec-strength benchmark BSB are made from these).
pub struct Mutant {
    /// The changed function.
    pub def: String,
    /// The site number within `def`, in the order W0904 visits them.
    pub site: usize,
    /// Position of the changed node in its input file (1-based).
    pub line: u32,
    pub col: u32,
    /// The text shown before and after the change, as in W0904.
    pub before: String,
    pub after: String,
    /// The top-level unit holding `def`, and that unit's normal form
    /// (ADR-0019) with the change applied. Every other unit is unchanged.
    pub unit: String,
    pub text: String,
}

impl Mutant {
    /// One JSONL row: `{"def","site","line","col","before","after","unit","text"}`.
    pub fn row(&self) -> String {
        use crate::diag::json_str;
        format!(
            "{{\"def\":{},\"site\":{},\"line\":{},\"col\":{},\"before\":{},\"after\":{},\"unit\":{},\"text\":{}}}\n",
            json_str(&self.def),
            self.site,
            self.line,
            self.col,
            json_str(&self.before),
            json_str(&self.after),
            json_str(&self.unit),
            json_str(&self.text)
        )
    }
}

/// Every W0904 mutant of the executable functions `defs` (in the given
/// order, sites in visiting order), uncapped and unproved: equivalence and
/// verdicts are left to the caller. `Err` carries the diagnostics when the
/// program does not check, or names a path that is not an executable
/// function. Deterministic: the result depends only on the input.
pub fn mutate_sources(files: &[(String, String)], defs: &[String]) -> Result<Vec<Mutant>, String> {
    let fe = crate::frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let mut table: BTreeMap<String, &FnDef> = BTreeMap::new();
    for m in &fe.program.mods {
        collect(m, "", &mut table);
    }
    let mut out = Vec::new();
    for path in defs {
        let f = match table.get(path) {
            Some(f) if f.kind == FnKind::Exec => *f,
            _ => return Err(format!("`{path}` is not an executable function\n")),
        };
        let text = &files[f.span.file as usize].1;
        let unit = path.split("::").next().unwrap_or_default().to_string();
        for (k, s) in sites(f, text).into_iter().enumerate() {
            let mut prog = fe.program.clone();
            apply_site(
                find_mut(&mut prog, path).expect("definition exists"),
                text,
                k,
            );
            let mut units = crate::fmt::format_units(&prog);
            let (line, col) = fe.sm.line_col(s.span.file, s.span.lo);
            out.push(Mutant {
                def: path.clone(),
                site: k,
                line,
                col,
                before: s.before,
                after: s.after,
                text: units.remove(&unit).unwrap_or_default(),
                unit: unit.clone(),
            });
        }
    }
    Ok(out)
}

/// W0905 for reach point `what` (backlog 1B-44). No honest program yields
/// one: in a verified function every point is reached, so a report means
/// contradictory contracts or a fault in the encoding.
fn w0905(path: &str, span: Span, what: &str) -> Diag {
    Diag::new(
        "W0905",
        span,
        format!(
            "the {what} of `{path}` is unreachable in its proof: every obligation after it holds vacuously"
        ),
    )
    .fix(
        "look for a contradiction in the loop invariants, the callees' postconditions and the revealed definitions",
        None,
    )
}

// ---------- escapes (W1001, W1002, ADR-0029, backlog 2E-04) ----------

/// W1001 for each `#[rationale(..)]` whose hash is not the one it is bound
/// to (ADR-0029 §3): what it speaks for (the definition, the predicates it
/// names) changed since it was confirmed, or it never was. The fix writes
/// the current hash; it is a judgment, so `bmb fix` applies it only with
/// `--confirm`.
///
/// With `all` false, only rationales that carry no hash: `bmb fix` writes
/// those (the tools fill the hash in, §1), and `--confirm` the others too.
pub fn unconfirmed_rationales(p: &Program, t: &crate::verify::Tables, all: bool) -> Vec<Diag> {
    let hashes = crate::escape::Hashes::new(p, &t.fns);
    let mut out = Vec::new();
    for (path, _, f) in crate::check::defs_of(p, &t.fns) {
        let path = &path;
        let e = crate::escape::of(f);
        if e.rationales.is_empty() {
            continue;
        }
        let Some(h) = hashes.of(path) else { continue };
        for r in &e.rationales {
            if r.hash.as_deref() == Some(h.as_str()) || (!all && r.hash.is_some()) {
                continue;
            }
            let why = match &r.hash {
                None => "it carries no hash yet".to_string(),
                Some(_) => {
                    format!("`{path}` or a predicate it names changed since it was confirmed")
                }
            };
            out.push(
                Diag::new("W1001", r.span, format!("unconfirmed rationale: {why}"))
                    .fix(
                        "if the reason still holds, confirm it: `bmb fix --confirm` writes the current hash",
                        Some(crate::escape::with_hash(&r.attr, &h)),
                    ),
            );
        }
    }
    out
}

/// W1002 for each `assume` of a definition that is `proved` (ADR-0029 §2):
/// its proof does not need the trusted fact, so the escape (and its
/// rationale) can go. The fix removes it.
fn needless_assumes(base: &crate::verify::Outcome, defs: &BTreeMap<String, &FnDef>) -> Vec<Diag> {
    let mut out = Vec::new();
    for s in &base.statuses {
        if s.failed > 0 || s.trust != crate::verify::Trust::Proved {
            continue;
        }
        let Some(f) = defs.get(&s.path) else { continue };
        for sp in crate::escape::of(f).assumes {
            out.push(
                Diag::new(
                    "W1002",
                    sp,
                    format!(
                        "`{}` is proved without this `assume`: the escape is not needed",
                        s.path
                    ),
                )
                .fix("remove it (and its rationale)", Some(String::new())),
            );
        }
    }
    out
}

// ---------- unneeded effects (W0909, ADR-0030 §3, backlog 2F-02) ----------

/// W0909 for each executable function that declares an effect its body
/// does not need (`Tables::effect_needs`: its own operations and its
/// callees' declared effects): the declaration is an interface and may be
/// wider, but a wider one makes every caller declare it too. Not reported
/// for a trait's method or an `impl`'s function of a trait (the trait's
/// declaration is the interface) or for `main`'s `io` (required, E0701).
fn unneeded_effects(t: &crate::verify::Tables, defs: &BTreeMap<String, &FnDef>) -> Vec<Diag> {
    use crate::check::Effects;
    let mut out = Vec::new();
    for (path, (_, needed)) in &t.effect_needs {
        let Some(sig) = t.fns.get(path) else {
            continue;
        };
        if sig.trait_of.is_some() || sig.default || path.starts_with('<') {
            continue;
        }
        let Some(f) = defs.get(path) else {
            continue;
        };
        let keep = if f.name.name == "main" && !path.contains("::main::") {
            needed.union(Effects::IO)
        } else {
            *needed
        };
        let extra = sig.effect.minus(keep);
        if extra.is_pure() {
            continue;
        }
        let Some(attr) = f.attrs.iter().find(|a| a.name.name == "effects") else {
            continue;
        };
        out.push(
            Diag::new(
                "W0909",
                attr.span,
                format!(
                    "`{path}` declares effects its body does not need: `{}`",
                    extra.text()
                ),
            )
            .fix(
                format!(
                    "declare `#[effects({})]`, unless the wider interface is meant (callers must declare it too)",
                    keep.text()
                ),
                Some(format!("#[effects({})]", keep.text())),
            ),
        );
    }
    out
}

// ---------- pure state in printing loops (W0910, backlog 2D-30) ----------

/// W0910 for each outermost `while`/`for` of an `io` function whose body
/// calls a function that needs `io` and assigns locals declared before the
/// loop, every such assignment's value calling nothing that needs `io`.
/// That loop is a state machine with its output: the update is a pure
/// function of the state and the loop's inputs, which a contract can
/// specify and the gate prove, and only the printing remains to test (the
/// agents of T0-30 kept both in one `io` function, H-EFFECT). The suggested
/// signature takes the variables the updates read and returns the state.
fn io_loop_cores(t: &crate::verify::Tables, defs: &BTreeMap<String, &FnDef>) -> Vec<Diag> {
    use crate::check::Effects;
    let mut out = Vec::new();
    for (path, f) in defs {
        let Some(sig) = t.fns.get(path) else {
            continue;
        };
        if !sig.effect.contains(Effects::IO) || path.starts_with('<') {
            continue;
        }
        let mod_path = path.rsplit_once("::").map_or("", |p| p.0);
        let needs_io = |e: &Expr| {
            let mut found = false;
            each_expr(e, &mut |x| {
                if let ExprKind::Call { path: p, .. } = &x.kind {
                    let callee = crate::verify::resolve_in(t, p, mod_path);
                    if t.fns
                        .get(&callee)
                        .is_some_and(|s| s.effect.contains(Effects::IO))
                    {
                        found = true;
                    }
                }
            });
            found
        };
        let mut loops = Vec::new();
        outer_loops(&f.body, &mut loops);
        for (span, body, var, wcond) in loops {
            let mut lp = LoopParts::default();
            loop_parts(body, &mut lp, &needs_io);
            let LoopParts {
                declared,
                lets,
                assigns,
                conds,
                prints,
            } = lp;
            // The loop's counter (in the `while` condition, only stepped)
            // is the shell's, not state.
            let in_cond = |n: &str| {
                let mut found = false;
                if let Some(c) = wcond {
                    each_expr(c, &mut |x| {
                        if matches!(&x.kind, ExprKind::Var(v) if v.name == n) {
                            found = true;
                        }
                    });
                }
                found
            };
            let counter = |n: &str| {
                in_cond(n)
                    && assigns
                        .iter()
                        .filter(|(m, _)| *m == n)
                        .all(|(_, e)| simple_update(e) && !matches!(e.kind, ExprKind::Call { .. }))
            };
            let state: Vec<&str> = {
                let mut s: Vec<&str> = Vec::new();
                for (n, _) in &assigns {
                    if !declared.contains(*n) && !s.contains(n) && !counter(n) {
                        s.push(n);
                    }
                }
                s
            };
            if !prints || state.is_empty() {
                continue;
            }
            let updates: Vec<&Expr> = assigns
                .iter()
                .filter(|(n, _)| state.contains(n))
                .map(|(_, e)| *e)
                .collect();
            if updates.iter().any(|e| needs_io(e)) {
                continue;
            }
            // Only a decision or computation written in the loop itself: a
            // loop that steps counters and asks pure functions is already
            // the shell.
            let decides = conds.iter().any(|(c, names)| {
                inline_logic(c) && names.iter().any(|n| state.contains(n)) && !needs_io(c)
            });
            if !decides && updates.iter().all(|e| simple_update(e)) {
                continue;
            }
            // A `let` of the loop that reads no state is an input of the
            // step (`kind = kinds[k]`); one that does is inside the step.
            let reads_state = |e: &Expr| {
                let mut found = false;
                each_expr(e, &mut |x| {
                    if matches!(&x.kind, ExprKind::Var(v) if state.contains(&v.name.as_str())) {
                        found = true;
                    }
                });
                found
            };
            let input = |n: &str| {
                lets.iter()
                    .any(|(m, init, mutable)| *m == n && !*mutable && !reads_state(init))
            };
            // A `let mut` of the loop that a deciding branch assigns is a
            // result of the step (the value printed).
            let results: Vec<&str> = lets
                .iter()
                .filter(|(n, _, mutable)| {
                    *mutable
                        && conds
                            .iter()
                            .any(|(c, names)| inline_logic(c) && names.contains(n))
                })
                .map(|(n, _, _)| *n)
                .collect();
            let mut reads: Vec<String> = state.iter().map(|s| s.to_string()).collect();
            let mut read = |x: &Expr| {
                if let ExprKind::Var(v) = &x.kind
                    && !reads.contains(&v.name)
                    && (!declared.contains(v.name.as_str()) || input(&v.name))
                    && !counter(&v.name)
                    && var != Some(v.name.as_str())
                {
                    reads.push(v.name.clone());
                }
            };
            for (c, names) in &conds {
                if names
                    .iter()
                    .any(|n| state.contains(n) || results.contains(n))
                {
                    each_expr(c, &mut read);
                }
            }
            for (n, e) in &assigns {
                if state.contains(n) || results.contains(n) {
                    each_expr(e, &mut read);
                }
            }
            let locals = t.locals.get(path);
            let typed = |n: &str| match locals.and_then(|l| l.get(n)) {
                Some(ty) => format!("{n}: {}", crate::check::ty_str(ty)),
                None => n.to_string(),
            };
            let outs: Vec<&str> = state.iter().chain(&results).copied().collect();
            let ret = match &outs[..] {
                [one] => locals
                    .and_then(|l| l.get(*one))
                    .map_or_else(|| one.to_string(), crate::check::ty_str),
                many => format!(
                    "({})",
                    many.iter()
                        .map(|n| {
                            locals
                                .and_then(|l| l.get(*n))
                                .map_or_else(|| n.to_string(), crate::check::ty_str)
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            };
            let name = format!("{}_step", f.name.name);
            out.push(
                Diag::new(
                    "W0910",
                    span,
                    format!(
                        "this loop of `{path}` prints and updates its state `{}` with pure expressions only: the update can be a pure function, proved by its contract, so that only the printing needs a test",
                        state.join("`, `")
                    ),
                )
                .fix(
                    format!(
                        "move the update into `fn {name}({}) -> {ret}` with `#[effects(pure)]` and a contract, and assign its result here",
                        reads.iter().map(|r| typed(r)).collect::<Vec<_>>().join(", ")
                    ),
                    None,
                ),
            );
        }
    }
    out
}

type LoopRef<'a> = (Span, &'a Block, Option<&'a str>, Option<&'a Expr>);

/// The outermost `while` and `for` loops of a block (not inside another):
/// span, body, the `for` variable, the `while` condition.
fn outer_loops<'a>(b: &'a Block, out: &mut Vec<LoopRef<'a>>) {
    for s in &b.stmts {
        match s {
            Stmt::While {
                body, span, cond, ..
            } => out.push((*span, body, None, Some(cond))),
            Stmt::For {
                var, body, span, ..
            } => out.push((*span, body, Some(var.name.as_str()), None)),
            Stmt::Let { init: e, .. } | Stmt::Expr(e) | Stmt::Assign { value: e, .. } => {
                expr_loops(e, out)
            }
            _ => {}
        }
    }
    if let Some(e) = &b.tail {
        expr_loops(e, out);
    }
}

fn expr_loops<'a>(e: &'a Expr, out: &mut Vec<LoopRef<'a>>) {
    match &e.kind {
        ExprKind::If { then, els, .. } => {
            outer_loops(then, out);
            if let Some(x) = els {
                expr_loops(x, out);
            }
        }
        ExprKind::Block(b) => outer_loops(b, out),
        ExprKind::Match { arms, .. } => {
            for a in arms {
                expr_loops(&a.body, out);
            }
        }
        _ => {}
    }
}

/// What a loop body declares, assigns (to a plain local), the conditions
/// of its branches with the locals each branch assigns, and whether it calls
/// something that needs `io`; nested blocks and loops included.
#[derive(Default)]
struct LoopParts<'a> {
    declared: BTreeSet<&'a str>,
    /// The loop's own `let`s: name, initializer, `mut`.
    lets: Vec<(&'a str, &'a Expr, bool)>,
    assigns: Vec<(&'a str, &'a Expr)>,
    conds: Vec<(&'a Expr, Vec<&'a str>)>,
    prints: bool,
}

fn loop_parts<'a>(b: &'a Block, lp: &mut LoopParts<'a>, needs_io: &dyn Fn(&Expr) -> bool) {
    for s in &b.stmts {
        match s {
            Stmt::Let {
                name,
                init,
                mutable,
                ..
            } => {
                lp.declared.insert(name.name.as_str());
                lp.lets.push((name.name.as_str(), init, *mutable));
                lp.prints |= needs_io(init);
                expr_parts(init, lp, needs_io);
            }
            Stmt::Assign { target, value, .. } => {
                if let ExprKind::Var(v) = &target.kind {
                    lp.assigns.push((v.name.as_str(), value));
                }
                lp.prints |= needs_io(value);
                expr_parts(value, lp, needs_io);
            }
            Stmt::While { cond, body, .. } => {
                lp.prints |= needs_io(cond);
                loop_parts(body, lp, needs_io);
            }
            Stmt::For { var, body, .. } => {
                lp.declared.insert(var.name.as_str());
                loop_parts(body, lp, needs_io);
            }
            Stmt::Expr(e) => {
                lp.prints |= needs_io(e);
                expr_parts(e, lp, needs_io);
            }
            _ => {}
        }
    }
    if let Some(e) = &b.tail {
        lp.prints |= needs_io(e);
        expr_parts(e, lp, needs_io);
    }
}

fn expr_parts<'a>(e: &'a Expr, lp: &mut LoopParts<'a>, needs_io: &dyn Fn(&Expr) -> bool) {
    let before = lp.assigns.len();
    let cond = match &e.kind {
        ExprKind::If { cond, then, els } => {
            loop_parts(then, lp, needs_io);
            if let Some(x) = els {
                expr_parts(x, lp, needs_io);
            }
            Some(&**cond)
        }
        ExprKind::Block(b) | ExprKind::Loop { body: b, .. } => {
            loop_parts(b, lp, needs_io);
            None
        }
        ExprKind::Match { scrut, arms } => {
            for a in arms {
                expr_parts(&a.body, lp, needs_io);
            }
            Some(&**scrut)
        }
        _ => None,
    };
    if let Some(c) = cond {
        let names = lp.assigns[before..].iter().map(|(n, _)| *n).collect();
        lp.conds.push((c, names));
    }
}

/// A branch condition that is a decision of its own: not a call (an
/// extracted pure predicate), a local, the negation of one, or the test of
/// one value against a literal (`n < 0`: the shell reading a result).
fn inline_logic(c: &Expr) -> bool {
    let read = |x: &Expr| {
        matches!(
            x.kind,
            ExprKind::Var(_)
                | ExprKind::Index { .. }
                | ExprKind::Field { .. }
                | ExprKind::Call { .. }
        )
    };
    let lit = |x: &Expr| match &x.kind {
        ExprKind::Int(..) | ExprKind::Bool(_) | ExprKind::Float(_) => true,
        ExprKind::Unary(UnOp::Neg, y) => matches!(y.kind, ExprKind::Int(..) | ExprKind::Float(_)),
        _ => false,
    };
    match &c.kind {
        ExprKind::Call { .. } | ExprKind::Var(_) | ExprKind::Bool(_) => false,
        ExprKind::Unary(UnOp::Not, x) => inline_logic(x),
        ExprKind::Binary(op, l, r) if op.is_cmp() => !((read(l) && lit(r)) || (lit(l) && read(r))),
        _ => true,
    }
}

/// A state update that is a step, not a computation: a literal, a local,
/// an element or field read, a call, or `v + k` / `v - k` with a literal.
fn simple_update(e: &Expr) -> bool {
    let atom = |x: &Expr| {
        matches!(
            x.kind,
            ExprKind::Int(..) | ExprKind::Bool(_) | ExprKind::Var(_) | ExprKind::Float(_)
        )
    };
    match &e.kind {
        ExprKind::Call { .. } | ExprKind::Index { .. } | ExprKind::Field { .. } => true,
        ExprKind::Binary(BinOp::Add | BinOp::Sub, l, r) => {
            atom(l) && matches!(r.kind, ExprKind::Int(..))
        }
        _ => atom(e),
    }
}

/// Calls `f` on `e` and every subexpression of it, blocks included.
pub(crate) fn each_expr(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(e);
    for c in children(e) {
        each_expr(c, f);
    }
}

// ---------- self-matching triggers (W0906, backlog 1B-45) ----------

/// The subexpressions of `e`, specification parts included.
fn children(e: &Expr) -> Vec<&Expr> {
    match &e.kind {
        ExprKind::Call { args, .. } | ExprKind::Array(args) => args.iter().collect(),
        ExprKind::Method { recv, args, .. } => std::iter::once(&**recv).chain(args).collect(),
        ExprKind::Index { base, index } => vec![base, index],
        ExprKind::Range { lo: l, hi: r } => vec![l, r],
        ExprKind::Binary(_, l, r) => vec![l, r],
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Deref(x)
        | ExprKind::Try(x)
        | ExprKind::Trigger(x)
        | ExprKind::Repeat { value: x, .. }
        | ExprKind::Field { base: x, .. }
        | ExprKind::Quant { body: x, .. } => vec![x],
        ExprKind::If { cond, then, els } => {
            let mut v = vec![&**cond];
            v.extend(block_exprs(then));
            v.extend(els.as_deref());
            v
        }
        ExprKind::Block(b) => block_exprs(b),
        ExprKind::StructLit { fields, rest, .. } => fields
            .iter()
            .map(|(_, x)| x)
            .chain(rest.as_deref())
            .collect(),
        ExprKind::Match { scrut, arms } => std::iter::once(&**scrut)
            .chain(arms.iter().map(|a| &a.body))
            .collect(),
        ExprKind::Return(x) | ExprKind::Break { value: x, .. } => {
            x.as_deref().into_iter().collect()
        }
        ExprKind::Loop {
            invariants,
            decreases,
            body,
            ..
        } => invariants
            .iter()
            .chain(decreases.as_deref())
            .chain(block_exprs(body))
            .collect(),
        ExprKind::Continue { .. } => Vec::new(),
        ExprKind::Path(_)
        | ExprKind::Int(..)
        | ExprKind::ByteLit(_)
        | ExprKind::Float(_)
        | ExprKind::ByteStr(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Var(_)
        | ExprKind::Error => Vec::new(),
    }
}

/// The expressions of a block, loop invariants, measures and `assert`
/// conditions included.
pub(crate) fn block_exprs(b: &Block) -> Vec<&Expr> {
    let mut v = Vec::new();
    for s in &b.stmts {
        match s {
            Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => v.push(init),
            Stmt::Assign { target, value, .. } => v.extend([target, value]),
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                v.push(cond);
                v.extend(invariants);
                v.extend(decreases);
                v.extend(block_exprs(body));
            }
            Stmt::For {
                lo,
                hi,
                invariants,
                body,
                ..
            } => {
                v.extend([lo, hi]);
                v.extend(invariants);
                v.extend(block_exprs(body));
            }
            Stmt::Assert { cond, .. } | Stmt::Assume { cond, .. } => v.push(cond),
            Stmt::Expr(e) => v.push(e),
            Stmt::Proof { body, .. } => v.extend(block_exprs(body)),
        }
    }
    v.extend(b.tail.as_deref());
    v
}

/// Every quantifier (`forall`, `exists`) in `e`, nested ones included.
fn foralls<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
    if matches!(e.kind, ExprKind::Quant { .. }) {
        out.push(e);
    }
    for c in children(e) {
        foralls(c, out);
    }
}

/// The terms of a `forall` body, outside nested `forall`s: with `in_trigger`
/// false, those outside `#[trigger]` (`out` gets every node); with it true,
/// the trigger terms themselves.
fn body_terms<'e>(e: &'e Expr, in_trigger: bool, out: &mut Vec<&'e Expr>) {
    match &e.kind {
        ExprKind::Quant { .. } => {}
        ExprKind::Trigger(x) => {
            if in_trigger {
                out.push(x);
            }
        }
        _ => {
            if !in_trigger {
                out.push(e);
            }
            for c in children(e) {
                body_terms(c, in_trigger, out);
            }
        }
    }
}

fn mentions(e: &Expr, names: &BTreeSet<&str>) -> bool {
    match &e.kind {
        ExprKind::Var(v) => names.contains(v.name.as_str()),
        _ => children(e).into_iter().any(|c| mentions(c, names)),
    }
}

/// What a trigger term matches on, and its arguments as text: an index
/// `a[i]` (the array) or a call `f(x, y)` (the function).
fn term_key(e: &Expr, text: &str) -> Option<(String, String)> {
    let t = |sp: Span| squash(&text[sp.lo as usize..sp.hi as usize]);
    match &e.kind {
        ExprKind::Index { base, index } => Some((format!("{}[]", t(base.span)), t(index.span))),
        ExprKind::Call { path, args, .. } => Some((
            path.iter()
                .map(|i| i.name.as_str())
                .collect::<Vec<_>>()
                .join("::")
                + "()",
            args.iter()
                .map(|a| t(a.span))
                .collect::<Vec<_>>()
                .join(", "),
        )),
        _ => None,
    }
}

/// W0906 for every quantifier of `f` (contracts, body, invariants, `assert`).
fn self_matching(f: &FnDef, text: &str) -> Vec<Diag> {
    let mut roots: Vec<&Expr> = f.requires.iter().chain(&f.ensures).collect();
    roots.extend(&f.decreases);
    roots.extend(block_exprs(&f.body));
    let mut all = Vec::new();
    for r in roots {
        foralls(r, &mut all);
    }
    let mut out = Vec::new();
    for q in all {
        let ExprKind::Quant {
            q: kind,
            binders,
            body,
        } = &q.kind
        else {
            continue;
        };
        let mut trig = Vec::new();
        body_terms(body, true, &mut trig);
        let [t] = trig[..] else { continue };
        let Some((head, targs)) = term_key(t, text) else {
            continue;
        };
        let bound: BTreeSet<&str> = binders.iter().map(|b| b.name.name.as_str()).collect();
        let mut terms = Vec::new();
        body_terms(body, false, &mut terms);
        for e in terms {
            let Some((h, args)) = term_key(e, text) else {
                continue;
            };
            if h != head || args == targs || !mentions(e, &bound) {
                continue;
            }
            let raw = &text[e.span.lo as usize..e.span.hi as usize];
            let shown = |sp: Span| squash(&text[sp.lo as usize..sp.hi as usize]);
            out.push(
                Diag::new(
                    "W0906",
                    e.span,
                    format!(
                        "`{}` matches the same pattern as this `{}`'s only trigger `{}` at other arguments: each instance yields a term the trigger matches again (a matching loop), which can run the solver to its limit",
                        shown(e.span),
                        kind.keyword(),
                        shown(t.span)
                    ),
                )
                .fix(
                    "mark it as a trigger too: the two terms then form one multi-pattern, which an instance does not feed",
                    Some(format!("#[trigger] {raw}")),
                ),
            );
        }
    }
    out
}

// ---------- canonical counting loops (W0907, backlog 2K-04) ----------

/// The blocks of `b`, nested ones included.
fn blocks_in<'b>(b: &'b Block, out: &mut Vec<&'b Block>) {
    out.push(b);
    for s in &b.stmts {
        match s {
            Stmt::While { cond, body, .. } => {
                expr_blocks(cond, out);
                blocks_in(body, out);
            }
            Stmt::For { lo, hi, body, .. } => {
                expr_blocks(lo, out);
                expr_blocks(hi, out);
                blocks_in(body, out);
            }
            Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => expr_blocks(init, out),
            Stmt::Assign { target, value, .. } => {
                expr_blocks(target, out);
                expr_blocks(value, out);
            }
            Stmt::Assert { .. } | Stmt::Assume { .. } | Stmt::Proof { .. } => {}
            Stmt::Expr(e) => expr_blocks(e, out),
        }
    }
    if let Some(t) = &b.tail {
        expr_blocks(t, out);
    }
}

fn expr_blocks<'b>(e: &'b Expr, out: &mut Vec<&'b Block>) {
    match &e.kind {
        ExprKind::If { cond, then, els } => {
            expr_blocks(cond, out);
            blocks_in(then, out);
            if let Some(x) = els {
                expr_blocks(x, out);
            }
        }
        ExprKind::Block(b) | ExprKind::Loop { body: b, .. } => blocks_in(b, out),
        _ => {
            for c in children(e) {
                expr_blocks(c, out);
            }
        }
    }
}

/// The variable a place expression is rooted at.
fn place_root(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Var(v) => Some(&v.name),
        ExprKind::Field { base: x, .. }
        | ExprKind::Index { base: x, .. }
        | ExprKind::Deref(x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Method { recv: x, .. } => place_root(x),
        _ => None,
    }
}

/// Whether `b` may change a variable of `names`: an assignment to a place
/// rooted there, a `&mut` borrow of one, or a method other than a read
/// called on one.
fn writes(b: &Block, names: &BTreeSet<&str>) -> bool {
    fn in_expr(e: &Expr, names: &BTreeSet<&str>) -> bool {
        let here = match &e.kind {
            ExprKind::Ref {
                mutable: true,
                expr,
            } => place_root(expr).is_some_and(|r| names.contains(r)),
            ExprKind::Method { recv, name, .. } => {
                !matches!(name.name.as_str(), "len" | "as_slice")
                    && place_root(recv).is_some_and(|r| names.contains(r))
            }
            _ => false,
        };
        here || children(e).into_iter().any(|c| in_expr(c, names))
    }
    let mut blocks = Vec::new();
    blocks_in(b, &mut blocks);
    let assigns = blocks.iter().flat_map(|b| &b.stmts).any(|s| {
        matches!(s, Stmt::Assign { target, .. }
            if place_root(target).is_some_and(|r| names.contains(r)))
    });
    assigns || block_exprs(b).into_iter().any(|e| in_expr(e, names))
}

/// Whether a `continue` in `b` goes to the loop labeled `label` around it
/// (an unlabeled one outside any inner loop does).
fn continues_to(b: &Block, label: Option<&str>) -> bool {
    fn in_block(b: &Block, label: Option<&str>, inner: bool) -> bool {
        b.stmts.iter().any(|s| match s {
            Stmt::While { cond, body, .. } => {
                in_expr(cond, label, inner) || in_block(body, label, true)
            }
            Stmt::For { lo, hi, body, .. } => {
                in_expr(lo, label, inner)
                    || in_expr(hi, label, inner)
                    || in_block(body, label, true)
            }
            Stmt::Let { init: e, .. } | Stmt::LetFields { init: e, .. } | Stmt::Expr(e) => {
                in_expr(e, label, inner)
            }
            Stmt::Assign { target, value, .. } => {
                in_expr(target, label, inner) || in_expr(value, label, inner)
            }
            Stmt::Assert { .. } | Stmt::Assume { .. } | Stmt::Proof { .. } => false,
        }) || b.tail.as_ref().is_some_and(|t| in_expr(t, label, inner))
    }
    fn in_expr(e: &Expr, label: Option<&str>, inner: bool) -> bool {
        match &e.kind {
            ExprKind::Continue { label: l } => match l {
                Some(l) => label == Some(l.name.as_str()),
                None => !inner,
            },
            ExprKind::Loop { body, .. } => in_block(body, label, true),
            ExprKind::If { cond, then, els } => {
                in_expr(cond, label, inner)
                    || in_block(then, label, inner)
                    || els.as_ref().is_some_and(|x| in_expr(x, label, inner))
            }
            ExprKind::Block(b) => in_block(b, label, inner),
            _ => children(e).into_iter().any(|c| in_expr(c, label, inner)),
        }
    }
    in_block(b, label, false)
}

/// A bound that names the same value at every head: variables, constants,
/// fields, arithmetic and `len()`.
fn plain_bound(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Var(_) | ExprKind::Int(..) | ExprKind::Path(_) => true,
        ExprKind::Field { base: x, .. } | ExprKind::Deref(x) | ExprKind::Unary(_, x) => {
            plain_bound(x)
        }
        ExprKind::Binary(op, l, r) => op.is_arith() && plain_bound(l) && plain_bound(r),
        ExprKind::Method { recv, name, args } => {
            name.name == "len" && args.is_empty() && plain_bound(recv)
        }
        _ => false,
    }
}

fn int_literal(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(..) => true,
        ExprKind::Unary(UnOp::Neg, x) => int_literal(x),
        _ => false,
    }
}

/// W0907 for each counting `while` of `f` (ADR-0049 §2, ADR-0057): `let mut
/// i: T = lo;` right before `while i < hi .. { ..; i = i + 1; }`, where
/// only that last statement changes `i`, nothing in the body changes what
/// `hi` reads, no `continue` goes to the loop and nothing after it names
/// `i`. That is `for i in lo..hi` with the same invariants: the range
/// `for` is encoded as exactly this expansion plus its head invariant and
/// measure (backlog 2K-01d), so the proof carries over. The fix splices the
/// source, so comments stay.
pub fn counting_whiles(f: &FnDef, text: &str) -> Vec<Diag> {
    let mut blocks = Vec::new();
    blocks_in(&f.body, &mut blocks);
    let mut out = Vec::new();
    for b in blocks {
        for k in 0..b.stmts.len().saturating_sub(1) {
            let Stmt::Let {
                mutable: true,
                name,
                ty,
                init: lo,
                span: let_span,
                ..
            } = &b.stmts[k]
            else {
                continue;
            };
            let Stmt::While {
                label,
                cond,
                invariants: _,
                decreases: Some(dec),
                body,
                span: w_span,
            } = &b.stmts[k + 1]
            else {
                continue;
            };
            let i = name.name.as_str();
            let ExprKind::Binary(BinOp::Lt, iv, hi) = &cond.kind else {
                continue;
            };
            let is_i = |e: &Expr| matches!(&e.kind, ExprKind::Var(v) if v.name == i);
            let step_ok = |s: &Stmt| match s {
                Stmt::Assign { target, value, .. } => {
                    is_i(target)
                        && matches!(&value.kind, ExprKind::Binary(BinOp::Add, a, one)
                            if is_i(a) && matches!(one.kind, ExprKind::Int(1, _)))
                }
                _ => false,
            };
            let Some((step, rest)) = body.stmts.split_last() else {
                continue;
            };
            let only_i: BTreeSet<&str> = [i].into();
            let mut hi_vars = BTreeSet::new();
            vars_in(hi, &mut hi_vars);
            let after = Block {
                stmts: b.stmts[k + 2..].to_vec(),
                tail: b.tail.clone(),
                span: b.span,
            };
            let body_rest = Block {
                stmts: rest.to_vec(),
                tail: None,
                span: body.span,
            };
            let convertible = is_i(iv)
                && matches!(
                    ty.kind,
                    TypeKind::Int(IntTy::I64) | TypeKind::Int(IntTy::U64) | TypeKind::Int(IntTy::U8) | TypeKind::Int(IntTy::Usize)
                )
                // Two literal bounds would make `i` an `i64`.
                && !(int_literal(lo) && int_literal(hi) && ty.kind != TypeKind::Int(IntTy::I64))
                && body.tail.is_none()
                && step_ok(step)
                && plain_bound(hi)
                && !hi_vars.contains(i)
                && !writes(&body_rest, &only_i)
                && !writes(body, &hi_vars.iter().map(String::as_str).collect())
                && !continues_to(body, label.as_ref().map(|l| l.name.as_str()))
                && !block_exprs(&after).into_iter().any(|e| mentions(e, &only_i));
            if !convertible {
                continue;
            }
            out.push(w0907(
                text,
                i,
                lo,
                hi,
                label.as_ref(),
                dec,
                step,
                body,
                *let_span,
                *w_span,
            ));
        }
    }
    out
}

/// W0907 for every function of `p` (methods and default bodies included),
/// in source order; `files` holds the source texts by file id.
pub fn canonical_loops(p: &Program, files: &[(String, String)]) -> Vec<Diag> {
    fn walk(m: &Mod, files: &[(String, String)], out: &mut Vec<Diag>) {
        let fns = m
            .fns
            .iter()
            .chain(m.impls.iter().flat_map(|i| &i.fns))
            .chain(m.trait_impls.iter().flat_map(|i| &i.fns))
            .chain(
                m.traits
                    .iter()
                    .flat_map(|t| &t.fns)
                    .filter(|t| t.has_body)
                    .map(|t| &t.f),
            );
        for f in fns {
            out.extend(counting_whiles(f, &files[f.span.file as usize].1));
        }
        for s in &m.mods {
            walk(s, files, out);
        }
    }
    let mut out = Vec::new();
    for m in &p.mods {
        walk(m, files, &mut out);
    }
    out.sort_by_key(|d| (d.span.file, d.span.lo));
    out
}

fn vars_in(e: &Expr, out: &mut BTreeSet<String>) {
    if let ExprKind::Var(v) = &e.kind {
        out.insert(v.name.clone());
    }
    for c in children(e) {
        vars_in(c, out);
    }
}

/// The start of the line of byte `at`, and the end of the line of byte
/// `to` past its newline, when only blanks surround `at..to` there.
fn whole_lines(text: &str, at: usize, to: usize) -> Option<(usize, usize)> {
    let start = text[..at].rfind('\n').map_or(0, |n| n + 1);
    let end = text[to..].find('\n').map_or(text.len(), |n| to + n + 1);
    (text[start..at].trim().is_empty() && text[to..end].trim().is_empty()).then_some((start, end))
}

#[allow(clippy::too_many_arguments)]
fn w0907(
    text: &str,
    i: &str,
    lo: &Expr,
    hi: &Expr,
    label: Option<&Ident>,
    dec: &Expr,
    step: &Stmt,
    body: &Block,
    let_span: Span,
    w_span: Span,
) -> Diag {
    let src = |sp: Span| &text[sp.lo as usize..sp.hi as usize];
    let Stmt::Assign {
        span: step_span, ..
    } = step
    else {
        unreachable!("checked step")
    };
    let head_at = label.map_or(w_span.lo, |l| l.span.lo) as usize;
    // Comments between the `let` and the loop stay before it.
    let between = text[let_span.hi as usize..head_at].trim_start();
    let label_text = &text[head_at..w_span.lo as usize];
    // The clauses without `decreases ..,`.
    let cond_end = hi.span.hi as usize;
    let dec_kw = text[..dec.span.lo as usize]
        .rfind("decreases")
        .expect("a `decreases` keyword");
    let mut dec_end = dec.span.hi as usize;
    if text[dec_end..].starts_with(',') {
        dec_end += 1;
    }
    let (cut_lo, cut_hi) = whole_lines(text, dec_kw, dec_end).unwrap_or_else(|| {
        let n = text[dec_end..].len() - text[dec_end..].trim_start_matches(' ').len();
        (dec_kw, dec_end + n)
    });
    let mut clauses = format!(
        "{}{}",
        &text[cond_end..cut_lo],
        &text[cut_hi..body.span.lo as usize]
    );
    if clauses.trim().is_empty() {
        clauses = " ".into();
    }
    // The body without the step.
    let (s_lo, s_hi) = whole_lines(text, step_span.lo as usize, step_span.hi as usize)
        .unwrap_or((step_span.lo as usize, step_span.hi as usize));
    let body_text = format!(
        "{}{}",
        &text[body.span.lo as usize..s_lo],
        &text[s_hi..body.span.hi as usize]
    );
    let replacement = format!(
        "{between}{label_text}for {i} in {}..{}{clauses}{body_text}",
        src(lo.span),
        src(hi.span)
    );
    Diag::new(
        "W0907",
        Span {
            file: let_span.file,
            lo: let_span.lo,
            hi: w_span.hi,
        },
        format!(
            "`{i}` counts from `{}` up to `{}` by one: the canonical form is `for {i} in {}..{}`",
            squash(src(lo.span)),
            squash(src(hi.span)),
            squash(src(lo.span)),
            squash(src(hi.span))
        ),
    )
    .fix(
        "write the range `for` (the invariants stay; its measure is automatic)",
        Some(replacement),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Detection is tested in `tests/verify.rs` (`proofs_are_not_vacuous`,
    // which fails when the 2G-05 loop fault is put back).
    #[test]
    fn w0905_names_the_point_and_the_function() {
        let d = w0905("m::f", Span::default(), "exit of the loop");
        assert_eq!(d.code, "W0905");
        assert!(d.message.starts_with("the exit of the loop of `m::f`"));
        assert!(d.fix.is_some());
    }
}
