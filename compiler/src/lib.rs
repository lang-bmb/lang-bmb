//! BMB v2 compiler. Milestone 1: parse, check, prove.

pub mod ast;
pub mod cache;
pub mod check;
pub mod codegen;
pub mod diag;
pub mod fmt;
pub mod incremental;
pub mod json;
pub mod lexer;
pub mod lint;
pub mod map;
pub mod mcp;
pub mod moves;
pub mod parser;
pub mod sha256;
pub mod smt;
pub mod source;
pub mod toolchain;
pub mod uses_fill;
pub mod verify;
pub mod workspace;

use std::collections::BTreeMap;

use diag::{Diag, Diags};
use source::SourceMap;

/// Stack size for compiler work. Every recursive pass is bounded by the
/// parser's nesting limit (`parser::MAX_NESTING`); this is enough for a program
/// at that limit even in a debug build (backlog 1A-11, `tests/nesting.rs`).
pub const STACK_BYTES: usize = 256 << 20;

/// Runs `f` on a thread with `STACK_BYTES` of stack. The CLI and the MCP
/// server run everything through this.
pub fn with_stack<T: Send>(f: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .stack_size(STACK_BYTES)
            .spawn_scoped(s, f)
            .expect("cannot spawn the compiler thread")
            .join()
            .unwrap_or_else(|e| std::panic::resume_unwind(e))
    })
}

/// Result of checking a set of source files.
pub struct CheckOutput {
    pub ok: bool,
    /// Diagnostics as deterministic JSONL.
    pub diagnostics: String,
}

/// Result of checking and proving a set of source files.
pub struct VerifyOutput {
    pub ok: bool,
    /// Diagnostics, then one status line per definition, as deterministic JSONL.
    pub output: String,
    /// SMT-LIB script per definition (empty when checking failed).
    pub scripts: BTreeMap<String, String>,
    /// Proof cache (hits, misses); zero without a cache. Obligations of a
    /// definition reused by the incremental cache count as hits.
    pub cache: (usize, usize),
    /// What the incremental cache did (backlog 2D-10); empty without a cache.
    pub incr: incremental::Stats,
}

struct Frontend {
    sm: SourceMap,
    diags: Diags,
    program: ast::Program,
    tables: verify::Tables,
}

fn frontend(files: &[(String, String)]) -> Frontend {
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut program = ast::Program { mods: Vec::new() };
    for (name, text) in files {
        let id = sm.add(name.clone(), text.clone());
        let toks = lexer::lex(id, text, &mut diags);
        let mut p = parser::Parser::new(toks, &mut diags, text);
        program.mods.extend(p.parse_program().mods);
    }
    let mut c = check::Checker::new(&mut diags);
    c.check_program(&program);
    let tables = c.into_tables();
    Frontend {
        sm,
        diags,
        program,
        tables,
    }
}

/// Parses and checks the given files as one program.
pub fn check_sources(files: &[(String, String)]) -> CheckOutput {
    let fe = frontend(files);
    CheckOutput {
        ok: !fe.diags.has_errors(),
        diagnostics: fe.diags.to_jsonl(&fe.sm),
    }
}

/// One replacement applied by `fix_sources`.
pub struct AppliedFix {
    /// 1-based round in which it was applied.
    pub round: usize,
    pub file: String,
    /// Position (1-based line, column in characters) in the text before that round.
    pub line: u32,
    pub col: u32,
    pub code: &'static str,
    pub replacement: String,
}

/// Result of `fix_sources`.
pub struct FixOutput {
    /// Every input file with its fixed text, in input order.
    pub files: Vec<(String, String)>,
    pub applied: Vec<AppliedFix>,
    /// Rounds that applied at least one replacement.
    pub rounds: usize,
    /// Diagnostics of the fixed program (front end only, no proof), as JSONL.
    pub diagnostics: String,
    pub ok: bool,
}

impl FixOutput {
    /// Files whose text changed, in input order.
    pub fn changed<'a>(&'a self, original: &[(String, String)]) -> Vec<&'a str> {
        self.files
            .iter()
            .zip(original)
            .filter(|(a, b)| a.1 != b.1)
            .map(|(a, _)| a.0.as_str())
            .collect()
    }

    /// JSONL report: one `{"round","file","line","col","code","replacement"}`
    /// row per applied replacement, the remaining diagnostics, then a summary
    /// row `{"summary":true,"rounds","applied","changed","remaining","ok"}`.
    pub fn report(&self, original: &[(String, String)]) -> String {
        let mut out = String::new();
        for a in &self.applied {
            out.push_str(&format!(
                "{{\"round\":{},\"file\":{},\"line\":{},\"col\":{},\"code\":{},\"replacement\":{}}}\n",
                a.round,
                diag::json_str(&a.file),
                a.line,
                a.col,
                diag::json_str(a.code),
                diag::json_str(&a.replacement)
            ));
        }
        out.push_str(&self.diagnostics);
        let changed: Vec<String> = self
            .changed(original)
            .into_iter()
            .map(diag::json_str)
            .collect();
        out.push_str(&format!(
            "{{\"summary\":true,\"rounds\":{},\"applied\":{},\"changed\":[{}],\"remaining\":{},\"ok\":{}}}\n",
            self.rounds,
            self.applied.len(),
            changed.join(","),
            self.diagnostics.lines().count(),
            self.ok
        ));
        out
    }
}

/// Upper bound on fix rounds; each round re-checks the program.
pub const FIX_MAX_ROUNDS: usize = 16;

/// Applies the mechanical fixes (diagnostic `replacement`s, DESIGN.md §8.2,
/// backlog 2D-04) until none applies. Each round checks the program, takes
/// the replacements in (file, start, end, code, text) order and keeps each
/// one that does not overlap one already kept (two insertions at the same
/// point overlap), applies them, and checks again. It stops when no
/// replacement changes the text, when a text repeats, or after
/// `FIX_MAX_ROUNDS` rounds. The result depends only on the input.
pub fn fix_sources(files: &[(String, String)]) -> FixOutput {
    let mut texts: Vec<(String, String)> = files.to_vec();
    let mut applied = Vec::new();
    let mut rounds = 0;
    let mut seen = std::collections::BTreeSet::new();
    seen.insert(texts.clone());
    loop {
        let fe = frontend(&texts);
        if rounds == FIX_MAX_ROUNDS {
            return fix_output(texts, applied, rounds, fe);
        }
        let mut cand: Vec<(usize, usize, usize, &'static str, String)> = fe
            .diags
            .items
            .iter()
            .filter_map(|d| {
                let r = d.fix.as_ref()?.replacement.as_ref()?;
                let (f, lo, hi) = (d.span.file as usize, d.span.lo as usize, d.span.hi as usize);
                let text = &texts[f].1;
                let ok = lo <= hi
                    && hi <= text.len()
                    && text.is_char_boundary(lo)
                    && text.is_char_boundary(hi)
                    && text[lo..hi] != **r;
                ok.then(|| (f, lo, hi, d.code, r.clone()))
            })
            .collect();
        cand.sort();
        cand.dedup_by(|a, b| (a.0, a.1, a.2, &a.4) == (b.0, b.1, b.2, &b.4));
        let mut kept: Vec<(usize, usize, usize, &'static str, String)> = Vec::new();
        for c in cand {
            if let Some(last) = kept.last()
                && last.0 == c.0
                && (c.1 < last.2 || (c.1 == c.2 && last.1 == last.2 && c.1 == last.1))
            {
                continue;
            }
            kept.push(c);
        }
        if kept.is_empty() {
            return fix_output(texts, applied, rounds, fe);
        }
        rounds += 1;
        let mut next = texts.clone();
        for (f, lo, hi, _, r) in kept.iter().rev() {
            next[*f].1.replace_range(*lo..*hi, r);
        }
        for (f, lo, _, code, r) in kept {
            let (line, col) = fe.sm.line_col(f as u32, lo as u32);
            applied.push(AppliedFix {
                round: rounds,
                file: texts[f].0.clone(),
                line,
                col,
                code,
                replacement: r,
            });
        }
        if !seen.insert(next.clone()) {
            let fe = frontend(&next);
            return fix_output(next, applied, rounds, fe);
        }
        texts = next;
    }
}

fn fix_output(
    files: Vec<(String, String)>,
    applied: Vec<AppliedFix>,
    rounds: usize,
    fe: Frontend,
) -> FixOutput {
    FixOutput {
        files,
        applied,
        rounds,
        ok: !fe.diags.has_errors(),
        diagnostics: fe.diags.to_jsonl(&fe.sm),
    }
}

/// Parses, checks and proves the given files. Proof runs only on a program
/// that checks cleanly.
pub fn verify_sources(files: &[(String, String)]) -> VerifyOutput {
    verify_sources_cached(files, None)
}

/// As `verify_sources`, with the proof cache in `cache_dir` (backlog 2D-01):
/// obligations proved before with the same solver input are not sent to the
/// solver again, and new proofs are added. Definitions whose incremental key
/// is unchanged are not even encoded (backlog 2D-10, `incremental.rs`). The
/// output is the same as without the cache; `scripts` then holds only the
/// definitions that were encoded.
pub fn verify_sources_cached(
    files: &[(String, String)],
    cache_dir: Option<&std::path::Path>,
) -> VerifyOutput {
    verify_sources_opts(files, cache_dir, true)
}

/// As `verify_sources_cached`; `reuse: false` keeps the proof cache but
/// encodes every definition (so `scripts` is complete, for `--emit-smt`).
pub fn verify_sources_opts(
    files: &[(String, String)],
    cache_dir: Option<&std::path::Path>,
    reuse: bool,
) -> VerifyOutput {
    verify_frontend(&frontend(files), cache_dir, reuse)
}

fn verify_frontend(
    fe: &Frontend,
    cache_dir: Option<&std::path::Path>,
    reuse: bool,
) -> VerifyOutput {
    if fe.diags.has_errors() {
        return VerifyOutput {
            ok: false,
            output: fe.diags.to_jsonl(&fe.sm),
            scripts: BTreeMap::new(),
            cache: (0, 0),
            incr: incremental::Stats::default(),
        };
    }
    let mut diags = fe.diags.clone();
    let mut incr = cache_dir
        .filter(|_| reuse)
        .map(|d| incremental::Session::new(d, &fe.program, &fe.tables, &fe.sm));
    let (outcome, cache) = prove(fe, &mut diags, cache_dir, incr.as_mut());
    let mut output = diags.to_jsonl(&fe.sm);
    output.push_str(&status_lines(&outcome));
    if let Some(s) = &mut incr {
        s.save();
    }
    VerifyOutput {
        ok: !diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
        output,
        scripts: outcome.scripts,
        cache,
        incr: incr.map(|s| s.stats).unwrap_or_default(),
    }
}

/// Proof stability (backlog 1B-19): proves the program once per solver seed
/// `0..seeds` and reports, per definition, the verdict and `rlimit` under each
/// seed. A definition is unstable when its verdict differs between seeds.
/// Returns JSONL (one line per definition, then a summary line) and whether
/// every definition was stable. `Err` carries diagnostics when the program
/// does not check.
pub fn stability_sources(files: &[(String, String)], seeds: u64) -> Result<(String, bool), String> {
    let mut runs: Vec<Vec<verify::FnStatus>> = Vec::new();
    for seed in 0..seeds.max(1) {
        let mut fe = frontend(files);
        if fe.diags.has_errors() {
            return Err(fe.diags.to_jsonl(&fe.sm));
        }
        runs.push(
            verify::verify_program_seeded(&fe.program, &fe.tables, &mut fe.diags, seed).statuses,
        );
    }
    let mut out = String::new();
    let mut unstable = 0;
    let mut spread_max = 1.0f64;
    for (i, s) in runs[0].iter().enumerate() {
        let per: Vec<&verify::FnStatus> = runs.iter().map(|r| &r[i]).collect();
        let verdicts: Vec<String> = per
            .iter()
            .map(|s| {
                if s.failed == 0 {
                    "\"verified\""
                } else {
                    "\"failed\""
                }
                .to_string()
            })
            .collect();
        let stable = per.iter().all(|p| (p.failed == 0) == (s.failed == 0));
        unstable += usize::from(!stable);
        let lo = per.iter().map(|p| p.rlimit_total).min().unwrap_or(0);
        let hi = per.iter().map(|p| p.rlimit_total).max().unwrap_or(0);
        if lo > 0 {
            spread_max = spread_max.max(hi as f64 / lo as f64);
        }
        out.push_str(&format!(
            "{{\"def\":{},\"stable\":{stable},\"verdicts\":[{}],\"rlimit_min\":{lo},\"rlimit_max\":{hi}}}\n",
            diag::json_str(&s.path),
            verdicts.join(","),
        ));
    }
    out.push_str(&format!(
        "{{\"summary\":true,\"seeds\":{},\"defs\":{},\"unstable\":{unstable},\"rlimit_spread_max\":{:.2}}}\n",
        runs.len(),
        runs[0].len(),
        spread_max
    ));
    Ok((out, unstable == 0))
}

/// Result of `emit_llvm_sources`.
pub struct LlvmOutput {
    /// Diagnostics and statuses, as from `verify_sources`.
    pub verify: VerifyOutput,
    /// LLVM IR, when the program checks (proof may be partial).
    pub ir: Option<String>,
    /// Runtime checks kept because their obligation is unproved.
    pub checks: u32,
    pub has_main: bool,
}

/// Checks, proves and lowers to LLVM IR. IR is produced even when some
/// obligations are unproved; those keep runtime checks. Building a binary
/// additionally requires `verify.ok`.
pub fn emit_llvm_sources(files: &[(String, String)]) -> LlvmOutput {
    emit_llvm_sources_with(files, false)
}

/// As `emit_llvm_sources`, proving through the proof cache in `cache_dir`.
pub fn emit_llvm_sources_cached(
    files: &[(String, String)],
    cache_dir: Option<&std::path::Path>,
) -> LlvmOutput {
    emit_llvm(files, false, cache_dir)
}

/// As `emit_llvm_sources`; with `all_checks`, the prover does not run and
/// every obligation keeps its runtime check (`verify` then carries only
/// front-end diagnostics). Used to test that proofs are sound: a proved
/// program must never trip one of these checks.
pub fn emit_llvm_sources_with(files: &[(String, String)], all_checks: bool) -> LlvmOutput {
    emit_llvm(files, all_checks, None)
}

fn emit_llvm(
    files: &[(String, String)],
    all_checks: bool,
    cache_dir: Option<&std::path::Path>,
) -> LlvmOutput {
    emit_llvm_frontend(&frontend(files), all_checks, cache_dir)
}

fn emit_llvm_frontend(
    fe: &Frontend,
    all_checks: bool,
    cache_dir: Option<&std::path::Path>,
) -> LlvmOutput {
    if fe.diags.has_errors() {
        return LlvmOutput {
            verify: VerifyOutput {
                ok: false,
                output: fe.diags.to_jsonl(&fe.sm),
                scripts: BTreeMap::new(),
                cache: (0, 0),
                incr: incremental::Stats::default(),
            },
            ir: None,
            checks: 0,
            has_main: false,
        };
    }
    let mut incr = cache_dir
        .filter(|_| !all_checks)
        .map(|d| incremental::Session::new(d, &fe.program, &fe.tables, &fe.sm));
    let mut diags = fe.diags.clone();
    let (outcome, cache) = if all_checks {
        (verify::Outcome::default(), (0, 0))
    } else {
        prove(fe, &mut diags, cache_dir, incr.as_mut())
    };
    // What the proof gate does not encode is not lowered either.
    let module = match verify::program_data_type_use(&fe.program, &fe.tables) {
        Some(sp) => {
            diags.push(Diag::new(
                "E0814",
                sp,
                "not compiled yet: `Vec` (backlog 2G-04), or elements holding an array",
            ));
            None
        }
        None => {
            let usable = usable_proofs(fe, &outcome);
            Some(codegen::emit_with(
                &fe.program,
                &fe.tables,
                &usable,
                incr.as_mut(),
            ))
        }
    };
    if let Some(s) = &mut incr {
        s.save();
    }
    let mut output = diags.to_jsonl(&fe.sm);
    output.push_str(&status_lines(&outcome));
    LlvmOutput {
        verify: VerifyOutput {
            ok: !diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
            output,
            scripts: outcome.scripts,
            cache,
            incr: incr.map(|s| s.stats).unwrap_or_default(),
        },
        checks: module.as_ref().map_or(0, |m| m.checks),
        has_main: module.as_ref().is_some_and(|m| m.main.is_some()),
        ir: module.map(|m| m.ir),
    }
}

/// Proved obligations whose runtime checks may be dropped (backlog 1B-31).
/// A proof assumes the function's `requires` (established by its callers),
/// its callees' `ensures` (proved by them, under their own callees), and
/// earlier facts in the same function. So a proof is usable only when every
/// function connected to its own by calls, in either direction, is fully
/// proved; otherwise the whole component keeps its checks.
fn usable_proofs(
    fe: &Frontend,
    outcome: &verify::Outcome,
) -> std::collections::BTreeSet<verify::ObKey> {
    let model = map::Model::build(&fe.program, &fe.tables, &fe.sm, outcome);
    let paths: Vec<&String> = model.defs.keys().collect();
    let index: BTreeMap<&String, usize> = paths.iter().enumerate().map(|(i, p)| (*p, i)).collect();
    let mut parent: Vec<usize> = (0..paths.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for e in model.edges.iter().filter(|e| e.kind == "calls") {
        if let (Some(&a), Some(&b)) = (index.get(&e.from), index.get(&e.to)) {
            let (ra, rb) = (root(&mut parent, a), root(&mut parent, b));
            parent[ra] = rb;
        }
    }
    let mut broken = std::collections::BTreeSet::new();
    for s in outcome.statuses.iter().filter(|s| s.failed > 0) {
        if let Some(&i) = index.get(&s.path) {
            broken.insert(root(&mut parent, i));
        }
    }
    let mut broken_spans: Vec<(u32, u32, u32)> = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        if broken.contains(&root(&mut parent, i)) {
            let sp = model.def(p).expect("indexed definition").f.span;
            broken_spans.push((sp.file, sp.lo, sp.hi));
        }
    }
    outcome
        .proven
        .iter()
        .filter(|(_, file, lo, hi)| {
            !broken_spans
                .iter()
                .any(|(f, a, b)| f == file && a <= lo && hi <= b)
        })
        .cloned()
        .collect()
}

/// Proves a checked program, through the proof cache in `cache_dir` when
/// given (backlog 2D-01). Returns the outcome and the cache (hits, misses).
fn prove(
    fe: &Frontend,
    diags: &mut Diags,
    cache_dir: Option<&std::path::Path>,
    incr: Option<&mut incremental::Session>,
) -> (verify::Outcome, (usize, usize)) {
    prove_defs(fe, diags, cache_dir, incr, None)
}

/// As `prove`; with `only`, proves just those definitions (backlog 3A-04).
/// A definition's status depends only on its own obligations, so it is the
/// same as in a whole-program run.
fn prove_defs(
    fe: &Frontend,
    diags: &mut Diags,
    cache_dir: Option<&std::path::Path>,
    incr: Option<&mut incremental::Session>,
    only: Option<&std::collections::BTreeSet<String>>,
) -> (verify::Outcome, (usize, usize)) {
    let mut cache = cache_dir.map(cache::ProofCache::load);
    let outcome = match (incr, only) {
        (incr, Some(only)) => {
            verify::verify_only(&fe.program, &fe.tables, diags, only, cache.as_mut(), incr)
        }
        (Some(s), None) => {
            verify::verify_program_incremental(&fe.program, &fe.tables, diags, cache.as_mut(), s)
        }
        (None, None) => {
            verify::verify_program_with(&fe.program, &fe.tables, diags, 0, cache.as_mut())
        }
    };
    if let (Some(c), Some(dir)) = (&cache, cache_dir)
        && let Err(e) = c.save(dir)
    {
        eprintln!("bmb: cannot write proof cache in {}: {e}", dir.display());
    }
    (outcome, cache.map(|c| (c.hits, c.misses)).unwrap_or((0, 0)))
}

fn status_lines(outcome: &verify::Outcome) -> String {
    let mut out = String::new();
    for s in &outcome.statuses {
        let status = if s.failed == 0 { "verified" } else { "failed" };
        out.push_str(&format!(
            "{{\"def\":{},\"status\":\"{status}\",\"obligations\":{},\"failed\":{},\"rlimit\":{},\"rlimit_max\":{}}}\n",
            diag::json_str(&s.path),
            s.obligations,
            s.failed,
            s.rlimit_total,
            s.rlimit_max
        ));
    }
    out
}

/// Normal form of a program (ADR-0019): top-level unit name -> its text.
/// `Err` carries diagnostics when the program does not check.
pub fn format_sources(files: &[(String, String)]) -> Result<BTreeMap<String, String>, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    Ok(fmt::format_units(&fe.program))
}

/// Normal form of parsed sources without checking them (backlog 2B-03: the
/// data type surface is parsed before the checker accepts it). Err holds the
/// syntax errors.
pub fn format_parsed(files: &[(String, String)]) -> Result<BTreeMap<String, String>, String> {
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut program = ast::Program { mods: Vec::new() };
    for (name, text) in files {
        let id = sm.add(name.clone(), text.clone());
        let toks = lexer::lex(id, text, &mut diags);
        let mut p = parser::Parser::new(toks, &mut diags, text);
        program.mods.extend(p.parse_program().mods);
    }
    if diags.has_errors() {
        return Err(diags.to_jsonl(&sm));
    }
    Ok(fmt::format_units(&program))
}

/// Definition hashes as JSONL (ADR-0019 §2).
pub fn hash_sources(files: &[(String, String)]) -> Result<String, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let mut out = String::new();
    for h in fmt::hashes(&fe.program, &fe.tables.fns) {
        out.push_str(&format!(
            "{{\"def\":{},\"body\":\"{}\",\"iface\":\"{}\"}}\n",
            diag::json_str(&h.path),
            h.body,
            h.iface
        ));
    }
    Ok(out)
}

/// Builds the maps and answers a request against them (ADR-0020).
/// `request` is `["map", level]` or `["query", name, args..]`.
pub fn map_request(files: &[(String, String)], request: &[String]) -> Result<String, String> {
    map_request_cached(files, request, None)
}

/// As `map_request`, proving through the proof cache in `cache_dir`.
pub fn map_request_cached(
    files: &[(String, String)],
    request: &[String],
    cache_dir: Option<&std::path::Path>,
) -> Result<String, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let ix = map::Index::build(&fe.program, &fe.tables, &fe.sm);
    map_answer(&fe, &ix, request, cache_dir)
}

/// What a map request needs proved (backlog 1D-17, 3A-04).
enum ProofNeed<'r> {
    None,
    /// The L1 row of one definition (`query surface DEF`).
    One(&'r str),
    All,
}

/// Answers a map request on a checked program and its index.
fn map_answer(
    fe: &Frontend,
    ix: &map::Index,
    request: &[String],
    cache_dir: Option<&std::path::Path>,
) -> Result<String, String> {
    // Proof results reach only the L1 rows (`map L1`/`all`, `query surface`),
    // `query status` (backlog 1D-17) and `query tests` (2D-21). Every other answer is built from
    // the checked program alone, so the solver does not run for it.
    // `surface DEF` shows the status of DEF alone, which depends only on
    // DEF's own obligations, so only DEF is proved (backlog 3A-04).
    let need = match (
        request.first().map(String::as_str),
        request.get(1).map(String::as_str),
    ) {
        (Some("map"), level) if matches!(level.unwrap_or("all"), "L1" | "all") => ProofNeed::All,
        (Some("query"), Some("status" | "tests")) => ProofNeed::All,
        (Some("query"), Some("surface")) => match request.get(2) {
            Some(d) if ix.defs.contains_key(d) => ProofNeed::One(d),
            // A usage error: nothing is shown, so nothing is proved.
            _ => ProofNeed::None,
        },
        _ => ProofNeed::None,
    };
    let outcome = match need {
        ProofNeed::None => verify::Outcome::default(),
        ProofNeed::All => {
            let mut incr =
                cache_dir.map(|d| incremental::Session::new(d, &fe.program, &fe.tables, &fe.sm));
            let o = prove(fe, &mut fe.diags.clone(), cache_dir, incr.as_mut()).0;
            if let Some(s) = &mut incr {
                s.save();
            }
            o
        }
        ProofNeed::One(d) => {
            let only: std::collections::BTreeSet<String> = [d.to_string()].into();
            let mut incr = cache_dir.map(|dir| {
                incremental::Session::new_for(dir, &fe.program, &fe.tables, &fe.sm, &only)
            });
            let o = prove_defs(
                fe,
                &mut fe.diags.clone(),
                cache_dir,
                incr.as_mut(),
                Some(&only),
            )
            .0;
            if let Some(s) = &mut incr {
                s.save();
            }
            o
        }
    };
    let model = map::Model::view(ix, &fe.program, &fe.tables, &outcome);
    match request.first().map(String::as_str) {
        Some("map") => {
            let level = request.get(1).map(String::as_str).unwrap_or("all");
            if !matches!(level, "L0" | "L1" | "L2" | "all") {
                return Err(format!("unknown map level `{level}`"));
            }
            Ok(model.map(level))
        }
        Some("query") => {
            let name = request.get(1).ok_or("missing query name")?;
            model
                .query(name, &request[2..])
                .map(|body| format!("{}\n{body}", map::HEADER))
        }
        _ => Err("expected `map` or `query`".into()),
    }
}

/// One workspace file as last read and parsed (backlog 3A-03).
struct ParsedFile {
    name: String,
    text: String,
    /// Number of top-level units it contributed to the program.
    mods: usize,
    /// Its lexer and parser diagnostics.
    diags: Vec<diag::Diag>,
}

/// Front-end results kept between calls by a long-lived process, the MCP
/// server (backlog 3A-03). `update` takes the files as just read and redoes
/// only what they changed: a file whose name, position and text are those of
/// the last call is not lexed or parsed again (its syntax tree and
/// diagnostics are reused; spans carry the file's position, so a file that
/// moved is parsed again). Any change re-runs the checker over the whole
/// program, whose tables are global, and drops the map index, which is
/// rebuilt on first use. When nothing changed, everything is reused. The
/// answers are those of `map_request_cached`, `verify_sources_cached` and
/// `emit_llvm_sources_cached` on the same files (tests/mcp.rs compares a
/// long-lived server with a fresh one per call).
#[derive(Default)]
pub struct FrontCache {
    files: Vec<ParsedFile>,
    fe: Option<Frontend>,
    index: Option<map::Index>,
    /// Counters for tests and measurement.
    pub stats: FrontStats,
}

/// What a `FrontCache` did since it was created.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct FrontStats {
    /// Files lexed and parsed.
    pub parsed: usize,
    /// Files whose syntax tree was reused.
    pub reused: usize,
    /// Checker runs.
    pub checks: usize,
    /// Map index builds.
    pub indexes: usize,
}

impl FrontCache {
    /// Brings the cache to `files` (name, text), in program order.
    pub fn update(&mut self, files: Vec<(String, String)>) {
        let same = self.fe.is_some()
            && files.len() == self.files.len()
            && files
                .iter()
                .zip(&self.files)
                .all(|(a, b)| a.0 == b.name && a.1 == b.text);
        if same {
            return;
        }
        // Take the old syntax trees apart, file by file.
        let mut old_mods: Vec<Vec<ast::Mod>> = Vec::new();
        if let Some(fe) = self.fe.take() {
            let mut it = fe.program.mods.into_iter();
            for f in &self.files {
                old_mods.push(it.by_ref().take(f.mods).collect());
            }
        }
        self.index = None;
        let mut old = std::mem::take(&mut self.files).into_iter().zip(old_mods);
        let mut sm = SourceMap::default();
        let mut diags = Diags::default();
        let mut program = ast::Program { mods: Vec::new() };
        for (name, text) in files {
            let prev = old.next();
            let id = sm.add(name.clone(), text.clone());
            let (mods, file_diags) = match prev {
                Some((pf, mods)) if pf.name == name && pf.text == text => {
                    self.stats.reused += 1;
                    (mods, pf.diags)
                }
                _ => {
                    self.stats.parsed += 1;
                    let mut d = Diags::default();
                    let toks = lexer::lex(id, &text, &mut d);
                    let mut p = parser::Parser::new(toks, &mut d, &text);
                    (p.parse_program().mods, d.items)
                }
            };
            diags.items.extend(file_diags.iter().cloned());
            self.files.push(ParsedFile {
                name,
                text,
                mods: mods.len(),
                diags: file_diags,
            });
            program.mods.extend(mods);
        }
        self.stats.checks += 1;
        let mut c = check::Checker::new(&mut diags);
        c.check_program(&program);
        let tables = c.into_tables();
        self.fe = Some(Frontend {
            sm,
            diags,
            program,
            tables,
        });
    }

    fn fe(&self) -> &Frontend {
        self.fe.as_ref().expect("FrontCache::update runs first")
    }

    /// As `map_request_cached` on the files of the last `update`.
    pub fn map_request(
        &mut self,
        request: &[String],
        cache_dir: Option<&std::path::Path>,
    ) -> Result<String, String> {
        let fe = self.fe.as_ref().expect("FrontCache::update runs first");
        if fe.diags.has_errors() {
            return Err(fe.diags.to_jsonl(&fe.sm));
        }
        if self.index.is_none() {
            self.stats.indexes += 1;
        }
        let ix = self
            .index
            .get_or_insert_with(|| map::Index::build(&fe.program, &fe.tables, &fe.sm));
        map_answer(fe, ix, request, cache_dir)
    }

    /// As `verify_sources_cached` on the files of the last `update`.
    pub fn verify(&self, cache_dir: Option<&std::path::Path>) -> VerifyOutput {
        verify_frontend(self.fe(), cache_dir, true)
    }

    /// As `emit_llvm_sources_cached` on the files of the last `update`.
    pub fn emit_llvm(&self, cache_dir: Option<&std::path::Path>) -> LlvmOutput {
        emit_llvm_frontend(self.fe(), false, cache_dir)
    }
}
