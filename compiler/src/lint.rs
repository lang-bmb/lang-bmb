//! Contract lint (DESIGN.md §8.2 "계약 조정", backlog 2D-08).
//!
//! Five deterministic reports, as warning diagnostics (`W09xx`):
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

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::ast::{BinOp, Block, Expr, ExprKind, FnDef, FnKind, Mod, Program, Stmt, UnOp};
use crate::cache::ProofCache;
use crate::diag::{Diag, Diags};
use crate::incremental::Session;
use crate::source::Span;

/// Upper bound on re-verifications for unused-precondition checks (W0902).
pub const MAX_PRE_TRIALS: usize = 32;
/// Upper bound on re-verifications for unused-postcondition checks (W0903).
pub const MAX_POST_TRIALS: usize = 32;
/// Upper bound on mutants verified per run (W0904).
pub const MAX_MUTANTS: usize = 64;

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
    /// False when the proof gate could not run.
    pub ok: bool,
}

impl LintOutput {
    /// JSONL: the diagnostics, then
    /// `{"summary":true,"defs","reports","verifications","skipped","mutants","surviving","mutants_skipped"}`.
    pub fn report(&self) -> String {
        format!(
            "{}{{\"summary\":true,\"defs\":{},\"reports\":{},\"verifications\":{},\"skipped\":{},\"mutants\":{},\"surviving\":{},\"mutants_skipped\":{}}}\n",
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
    let mut fe = crate::frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let mut cache = cache_dir.map(ProofCache::load).unwrap_or_default();
    // Only the first verification covers the whole program; the restricted
    // ones below each change the definitions they prove, so the incremental
    // cache serves this one alone (and records it, with `cache_dir`).
    let base = match cache_dir {
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

    // W0905: points of verified functions their own proof cannot reach.
    for (path, span, what) in crate::verify::vacuous_ends(&fe.program, &fe.tables) {
        if unsat.contains(&path) || !verified(&path) {
            continue;
        }
        out.push(w0905(&path, span, what));
    }
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

    // W0904: body mutations the specification does not distinguish.
    let mut plan: Vec<(&String, Vec<Site>)> = Vec::new();
    for path in &exec {
        let f = defs[*path];
        if f.ensures.is_empty() || unsat.contains(*path) || !verified(path) {
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
        ExprKind::Int(c) if *c <= MAX_LIT => {
            let ctx = parent
                .filter(|p| p.lo <= e.span.lo && e.span.hi <= p.hi)
                .unwrap_or(e.span);
            splice(ctx, e.span.lo, e.span.hi, &(c + 1).to_string())
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
                        kind: ExprKind::Int(1),
                        span,
                    }),
                ),
                span,
            };
        }
        ExprKind::Binary(op, _, _) => *op = swap_op(*op).expect("a mutation site"),
        ExprKind::Int(c) => *c += 1,
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
            Stmt::Assert { .. } => {}
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
        ExprKind::Field { base, .. } | ExprKind::Deref(base) => walk_expr(base, None, f),
        ExprKind::StructLit { fields, .. } => {
            fields.iter_mut().for_each(|(_, x)| walk_expr(x, None, f))
        }
        ExprKind::Match { scrut, arms } => {
            walk_expr(scrut, None, f);
            arms.iter_mut()
                .for_each(|a| walk_expr(&mut a.body, None, f));
        }
        ExprKind::Path(_)
        | ExprKind::Old(_)
        | ExprKind::Forall { .. }
        | ExprKind::Trigger(_)
        | ExprKind::Int(_)
        | ExprKind::ByteLit(_)
        | ExprKind::ByteStr(_)
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
