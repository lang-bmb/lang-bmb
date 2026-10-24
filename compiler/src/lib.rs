//! BMB v2 compiler. Milestone 1: parse, check, prove.

pub mod ast;
pub mod cache;
pub mod check;
pub mod codegen;
pub mod diag;
pub mod fmt;
pub mod json;
pub mod lexer;
pub mod map;
pub mod mcp;
pub mod parser;
pub mod sha256;
pub mod smt;
pub mod source;
pub mod toolchain;
pub mod verify;
pub mod workspace;

use std::collections::BTreeMap;

use diag::Diags;
use source::SourceMap;

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
    /// Proof cache (hits, misses); zero without a cache.
    pub cache: (usize, usize),
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

/// Parses, checks and proves the given files. Proof runs only on a program
/// that checks cleanly.
pub fn verify_sources(files: &[(String, String)]) -> VerifyOutput {
    verify_sources_cached(files, None)
}

/// As `verify_sources`, with the proof cache in `cache_dir` (backlog 2D-01):
/// obligations proved before with the same solver input are not sent to the
/// solver again, and new proofs are added. The output is the same as without
/// the cache.
pub fn verify_sources_cached(
    files: &[(String, String)],
    cache_dir: Option<&std::path::Path>,
) -> VerifyOutput {
    let mut fe = frontend(files);
    if fe.diags.has_errors() {
        return VerifyOutput {
            ok: false,
            output: fe.diags.to_jsonl(&fe.sm),
            scripts: BTreeMap::new(),
            cache: (0, 0),
        };
    }
    let (outcome, cache) = prove(&mut fe, cache_dir);
    let mut output = fe.diags.to_jsonl(&fe.sm);
    output.push_str(&status_lines(&outcome));
    VerifyOutput {
        ok: !fe.diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
        output,
        scripts: outcome.scripts,
        cache,
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
    let mut fe = frontend(files);
    if fe.diags.has_errors() {
        return LlvmOutput {
            verify: VerifyOutput {
                ok: false,
                output: fe.diags.to_jsonl(&fe.sm),
                scripts: BTreeMap::new(),
                cache: (0, 0),
            },
            ir: None,
            checks: 0,
            has_main: false,
        };
    }
    let (outcome, cache) = if all_checks {
        let none = verify::Outcome {
            statuses: Vec::new(),
            proven: Default::default(),
            scripts: BTreeMap::new(),
        };
        (none, (0, 0))
    } else {
        prove(&mut fe, cache_dir)
    };
    let usable = usable_proofs(&fe, &outcome);
    let module = codegen::emit(&fe.program, &fe.tables, &usable);
    let mut output = fe.diags.to_jsonl(&fe.sm);
    output.push_str(&status_lines(&outcome));
    LlvmOutput {
        verify: VerifyOutput {
            ok: !fe.diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
            output,
            scripts: outcome.scripts,
            cache,
        },
        ir: Some(module.ir),
        checks: module.checks,
        has_main: module.main.is_some(),
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
    let mut spans: Vec<(u32, u32, u32)> = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        if broken.contains(&root(&mut parent, i)) {
            let sp = model.defs[*p].f.span;
            spans.push((sp.file, sp.lo, sp.hi));
        }
    }
    outcome
        .proven
        .iter()
        .filter(|(_, file, lo, hi)| {
            !spans
                .iter()
                .any(|(f, a, b)| f == file && a <= lo && hi <= b)
        })
        .cloned()
        .collect()
}

/// Proves a checked program, through the proof cache in `cache_dir` when
/// given (backlog 2D-01). Returns the outcome and the cache (hits, misses).
fn prove(
    fe: &mut Frontend,
    cache_dir: Option<&std::path::Path>,
) -> (verify::Outcome, (usize, usize)) {
    let mut cache = cache_dir.map(cache::ProofCache::load);
    let outcome =
        verify::verify_program_with(&fe.program, &fe.tables, &mut fe.diags, 0, cache.as_mut());
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
    let mut fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let (outcome, _) = prove(&mut fe, cache_dir);
    let model = map::Model::build(&fe.program, &fe.tables, &fe.sm, &outcome);
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
