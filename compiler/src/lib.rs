//! BMB v2 compiler. Milestone 1: parse, check, prove.

pub mod ast;
pub mod ast_json;
pub mod cache;
pub mod check;
pub mod closures;
pub mod codegen;
pub mod consts;
pub mod diag;
pub mod escape;
pub mod fmt;
pub mod identity;
pub mod incremental;
pub mod json;
pub mod lexer;
pub mod lint;
pub mod map;
pub mod mcp;
pub mod moves;
pub mod parser;
pub mod probe;
pub(crate) mod refs;
pub mod sha256;
pub mod smt;
pub mod source;
pub mod stdlib;
pub mod subproof;
pub mod toolchain;
pub mod udiff;
pub mod uses_fill;
pub mod verify;
pub mod view;
pub mod weaken;
pub mod workspace;

use std::collections::BTreeMap;

use diag::Diags;
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
    /// The program as parsed: the normal form prints it.
    source: ast::Program,
    /// The program the later stages read: method calls of user functions
    /// rewritten to calls by path (ADR-0051, `check::check_resolved`).
    program: ast::Program,
    tables: verify::Tables,
}

/// `bmb examples`: examples of the contracts of `defs` (backlog T1-44c,
/// `verify::examples`), `count` inputs each, as JSONL: per definition its
/// rows and a summary row. The program is checked, not proved: the examples
/// come from the contracts alone. The queries are advisory
/// (`smt::advisory`): a report, never a verdict of the gate. `Err` is the
/// diagnostics of a program that does not check, or names a path that is
/// not an executable function.
pub fn examples_sources(
    files: &[(String, String)],
    defs: &[String],
    count: usize,
) -> Result<String, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let found = smt::advisory(|| verify::examples(&fe.program, &fe.tables, defs, count))
        .map_err(|e| format!("{e}\n"))?;
    Ok(found.iter().map(|e| e.jsonl()).collect())
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
    // The library units it names, after its own (ADR-0092 §1).
    stdlib::add(&mut sm, &mut program, &mut diags);
    let (resolved, tables) = check::check_resolved(program.clone(), &mut diags);
    Frontend {
        sm,
        diags,
        source: program,
        program: resolved,
        tables,
    }
}

/// The reach points the program's own proofs refute (backlog 1B-44,
/// `verify::vacuous_ends`), as `path: what`, for every definition; empty
/// when the program does not check. `bmb lint` reports the ones of verified
/// functions as W0905.
pub fn vacuous_points(files: &[(String, String)]) -> Vec<String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Vec::new();
    }
    verify::vacuous_ends(&fe.program, &fe.tables)
        .0
        .into_iter()
        .map(|(path, _, what)| format!("{path}: {what}"))
        .collect()
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

/// `report` for the command line: the summary row also says whether the
/// changed files were written (`--write`), since `changed` lists them
/// either way (backlog 2E-05: read as written when they were not).
pub fn with_written(report: String, written: bool) -> String {
    let last = report.trim_end().rfind('\n').map_or(0, |i| i + 1);
    match report.strip_suffix("}\n") {
        Some(head) if report[last..].starts_with("{\"summary\":true") => {
            format!("{head},\"written\":{written}}}\n")
        }
        _ => report,
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
    fix_sources_with(files, false)
}

/// As `fix_sources`; on a program that checks it also writes the hash of
/// each `#[rationale(..)]` that carries none (ADR-0029 §1: the tools fill
/// it in) and, with `confirm`, of each whose hash is no longer current
/// (`bmb fix --confirm`: confirming is a judgment, §3).
pub fn fix_sources_with(files: &[(String, String)], confirm: bool) -> FixOutput {
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
        // A program that checks also takes its canonical-form rewrites
        // (W0907, ADR-0057).
        let canon = if fe.diags.has_errors() {
            Vec::new()
        } else {
            let mut c = lint::canonical_loops(&fe.program, &texts);
            c.extend(lint::method_calls(&fe, &texts));
            c.extend(lint::conversions(&fe.source, &texts));
            c.extend(lint::default_uses(&fe, &texts));
            c.extend(lint::unconfirmed_rationales(
                &fe.program,
                &fe.tables,
                confirm,
            ));
            c
        };
        let mut cand: Vec<(usize, usize, usize, &'static str, String)> = fe
            .diags
            .items
            .iter()
            .chain(&fe.diags.aliases)
            .chain(&canon)
            .filter_map(|d| {
                let r = d.fix.as_ref()?.replacement.as_ref()?;
                let (f, lo, hi) = (d.span.file as usize, d.span.lo as usize, d.span.hi as usize);
                // Only the program's files: the library is not written.
                let text = &texts.get(f)?.1;
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
            let (hi, r) = layout(&texts[*f].1, *lo, *hi, r);
            let lo = layout_start(&texts[*f].1, *lo, hi, &r);
            next[*f].1.replace_range(lo..hi, &r);
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

/// A replacement in the layout of the text around it (backlog 2R-04e:
/// `pub fn` and `(mut x` left a space behind, and an attribute added on the
/// line of its `fn`): a deletion takes the spaces after it on the same line
/// when a space, `(`, `&` or `<` is before it (`(mut x`, `&'a T`), and a line
/// of an insertion that has no indentation of its own gets the line's.
fn layout(text: &str, lo: usize, hi: usize, r: &str) -> (usize, String) {
    let line_start = text[..lo].rfind('\n').map_or(0, |i| i + 1);
    if r.is_empty() {
        // A deletion that leaves its line blank takes the line (an
        // attribute, W0916); `layout_start` moves the start to the line's.
        let line_end = text[hi..].find('\n').map_or(text.len(), |i| hi + i);
        if lo > line_start
            && text[line_start..lo].trim().is_empty()
            && text[hi..line_end].trim().is_empty()
            && line_end < text.len()
        {
            return (line_end + 1, String::new());
        }
        let before = text[..lo].chars().next_back();
        // The only entry of a `<..>` list goes with its brackets; the first
        // of several with the comma after it (a lifetime, E0208).
        if before == Some('<') {
            let rest = text[hi..].trim_start_matches(' ');
            if rest.starts_with('>') {
                // Signal the caller to take the `<` too: returned as the
                // end past `>`, the start fixed by `layout_start`.
                return (text.len() - rest.len() + 1, String::new());
            }
            if let Some(after) = rest.strip_prefix(',') {
                let skip = after.len() - after.trim_start_matches(' ').len();
                return (text.len() - rest.len() + 1 + skip, String::new());
            }
        }
        if matches!(before, Some(' ' | '(' | '&' | '<')) || lo == line_start {
            let after = text[hi..].len() - text[hi..].trim_start_matches(' ').len();
            return (hi + after, String::new());
        }
        return (hi, String::new());
    }
    if r.contains('\n') {
        let indent: String = text[line_start..lo]
            .chars()
            .take_while(|c| *c == ' ')
            .collect();
        let mut out = String::new();
        for (k, line) in r.split('\n').enumerate() {
            if k > 0 {
                out.push('\n');
                // A line that brings its own indentation keeps it.
                if !line.starts_with(' ') && !line.is_empty() {
                    out.push_str(&indent);
                }
            }
            out.push_str(line);
        }
        return (hi, out);
    }
    (hi, r.to_string())
}

/// The start of a deletion `layout` widened over a whole `<..>` list:
/// the `<` goes too.
fn layout_start(text: &str, lo: usize, hi: usize, r: &str) -> usize {
    let line_start = text[..lo].rfind('\n').map_or(0, |i| i + 1);
    if r.is_empty()
        && lo > line_start
        && text[line_start..lo].trim().is_empty()
        && text[..hi].ends_with('\n')
    {
        line_start
    } else if lo > 0 && text[..lo].ends_with('<') && text[..hi].ends_with('>') {
        lo - 1
    } else {
        lo
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
    verify_frontend(&frontend(files), cache_dir, reuse, None)
}

/// As `verify_sources_opts`, proving only the definitions in `only` (full
/// paths, `bmb verify --def`; backlog T1-29, ADR-0065 §4: re-proving the
/// overlapping reads of two edits in their merge). The whole program is
/// still checked, and each listed definition gets the status of a
/// whole-program run (`verify::verify_only`); the output has status lines for
/// the listed definitions alone. `Err` names the listed paths that are not
/// definitions of a program that checks.
pub fn verify_sources_only(
    files: &[(String, String)],
    cache_dir: Option<&std::path::Path>,
    reuse: bool,
    only: &std::collections::BTreeSet<String>,
) -> Result<VerifyOutput, String> {
    let fe = frontend(files);
    let (out, proved) = verify_frontend_defs(&fe, cache_dir, reuse, Some(only));
    let missing: Vec<&str> = only
        .iter()
        .filter(|d| !fe.diags.has_errors() && !proved.contains(*d))
        .map(String::as_str)
        .collect();
    if missing.is_empty() {
        Ok(out)
    } else {
        Err(format!("no definition {}", missing.join(", ")))
    }
}

fn verify_frontend(
    fe: &Frontend,
    cache_dir: Option<&std::path::Path>,
    reuse: bool,
    only: Option<&std::collections::BTreeSet<String>>,
) -> VerifyOutput {
    verify_frontend_defs(fe, cache_dir, reuse, only).0
}

/// `verify_frontend` and the paths of the definitions it gave a status.
fn verify_frontend_defs(
    fe: &Frontend,
    cache_dir: Option<&std::path::Path>,
    reuse: bool,
    only: Option<&std::collections::BTreeSet<String>>,
) -> (VerifyOutput, std::collections::BTreeSet<String>) {
    if fe.diags.has_errors() {
        let out = VerifyOutput {
            ok: false,
            output: fe.diags.to_jsonl(&fe.sm),
            scripts: BTreeMap::new(),
            cache: (0, 0),
            incr: incremental::Stats::default(),
        };
        return (out, Default::default());
    }
    let mut diags = fe.diags.clone();
    let mut incr = cache_dir.filter(|_| reuse).map(|d| match only {
        Some(only) => incremental::Session::new_for(d, &fe.program, &fe.tables, &fe.sm, only),
        None => incremental::Session::new(d, &fe.program, &fe.tables, &fe.sm),
    });
    let (outcome, cache) = prove_defs(fe, &mut diags, cache_dir, incr.as_mut(), only);
    let mut output = diags.to_jsonl(&fe.sm);
    output.push_str(&status_lines(&outcome));
    if let Some(s) = &mut incr {
        s.save();
    }
    let defs = outcome.statuses.iter().map(|s| s.path.clone()).collect();
    let out = VerifyOutput {
        ok: !diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
        output,
        scripts: outcome.scripts,
        cache,
        incr: incr.map(|s| s.stats).unwrap_or_default(),
    };
    (out, defs)
}

/// `hi / lo` in hundredths, rounded half up (`lo > 0`).
fn hundredths(hi: u64, lo: u64) -> u128 {
    let (hi, lo) = (u128::from(hi), u128::from(lo));
    (hi * 200 + lo) / (2 * lo)
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
    // The largest `hi / lo` in hundredths, rounded half up: integer fixed
    // point, so no float formatting is involved (backlog 4A-03).
    let mut spread_max: u128 = 100;
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
            spread_max = spread_max.max(hundredths(hi, lo));
        }
        out.push_str(&format!(
            "{{\"def\":{},\"stable\":{stable},\"verdicts\":[{}],\"rlimit_min\":{lo},\"rlimit_max\":{hi}}}\n",
            diag::json_str(&s.path),
            verdicts.join(","),
        ));
    }
    out.push_str(&format!(
        "{{\"summary\":true,\"seeds\":{},\"defs\":{},\"unstable\":{unstable},\"rlimit_spread_max\":{}.{:02}}}\n",
        runs.len(),
        runs[0].len(),
        spread_max / 100,
        spread_max % 100
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
    let usable = usable_proofs(fe, &outcome);
    let module = codegen::emit_with(&fe.program, &fe.tables, &usable, incr.as_mut());
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
        // Checks kept below an escape are not missing proofs (ADR-0029 §4).
        checks: module
            .checks_by
            .iter()
            .filter(|(p, _)| !outcome.checked.contains(*p))
            .map(|(_, n)| n)
            .sum(),
        has_main: module.main.is_some(),
        ir: Some(module.ir),
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
        // A proof with trusted facts says so (ADR-0029 §5); a `proved` line
        // is as before.
        let trust = match s.trust {
            verify::Trust::Proved => String::new(),
            t if s.failed == 0 => format!(",\"trust\":\"{}\"", t.name()),
            _ => String::new(),
        };
        // Only when the backstop ended a query (backlog INF-26).
        let trust = if s.backstop > 0 {
            format!("{trust},\"backstop\":{}", s.backstop)
        } else {
            trust
        };
        out.push_str(&format!(
            "{{\"def\":{},\"status\":\"{status}\",\"obligations\":{},\"failed\":{},\"rlimit\":{},\"rlimit_max\":{}{trust}}}\n",
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
    // Method calls by path become method syntax (ADR-0051, backlog 2M-02).
    let normal = check::methodize(&fe.source, &fe.program, &fe.tables);
    // The program's units: the library is not its text (ADR-0092 §1).
    let mut units = fmt::format_units(&normal);
    units.remove(stdlib::UNIT);
    Ok(units)
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

/// The tokens of `files` as JSONL (`bmb dump tokens`, backlog 4A-03): one
/// row per token with its file, byte span, kind and value, for comparing
/// another lexer with this one. A literal's `text` is its normal form
/// (ADR-0019 §1); a float also gives the bits of its `f64`, a byte string
/// its bytes. Lexer errors are the diagnostics instead.
pub fn dump_tokens(files: &[(String, String)]) -> Result<String, String> {
    use json::Value;
    use lexer::Tok;
    let q = |s: &str| Value::Str(s.to_string()).to_json();
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut out = String::new();
    for (name, text) in files {
        let id = sm.add(name.clone(), text.clone());
        for t in lexer::lex(id, text, &mut diags) {
            let (kind, rest) = match &t.tok {
                Tok::Ident(x) => ("ident", format!(",\"text\":{}", q(x))),
                Tok::Int(v, r) => (
                    "int",
                    format!(",\"text\":{},\"value\":\"{v}\"", q(&r.text(*v))),
                ),
                Tok::Float(b) => (
                    "float",
                    format!(
                        ",\"text\":{},\"bits\":\"{b:#018x}\"",
                        q(&lexer::float_text(*b))
                    ),
                ),
                Tok::Str(x) => ("str", format!(",\"text\":{}", q(x))),
                Tok::Char(c) => ("char", format!(",\"value\":{c}")),
                Tok::Byte(v) => ("byte", format!(",\"value\":{v}")),
                Tok::Bytes(bs) => {
                    let vs: Vec<String> = bs.iter().map(u8::to_string).collect();
                    ("bytes", format!(",\"value\":[{}]", vs.join(",")))
                }
                Tok::P(x) => ("punct", format!(",\"text\":{}", q(x))),
                Tok::Label(x) => ("label", format!(",\"text\":{}", q(x))),
                Tok::Eof => ("eof", String::new()),
            };
            out.push_str(&format!(
                "{{\"file\":{},\"lo\":{},\"hi\":{},\"kind\":\"{kind}\"{rest}}}\n",
                q(name),
                t.span.lo,
                t.span.hi
            ));
        }
    }
    if diags.has_errors() {
        return Err(diags.to_jsonl(&sm));
    }
    Ok(out)
}

/// Definition identities and hashes as JSONL (ADR-0059 §1, §4).
pub fn hash_sources(files: &[(String, String)]) -> Result<String, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let mut out = String::new();
    for h in fmt::hashes(&fe.program, &fe.tables) {
        out.push_str(&format!(
            "{{\"def\":{},\"id\":\"{}\",\"body\":\"{}\",\"iface\":\"{}\"}}\n",
            diag::json_str(&h.path),
            h.id,
            h.body,
            h.iface
        ));
    }
    Ok(out)
}

/// `bmb reads` (backlog T1-27, ADR-0065 §2): each definition's proof key as
/// rows, the read set of its proof (`incremental::proof_reads`), then each
/// data type's digest. JSONL, by path:
/// `{"def","kind","key","contract","decl","uses":[..],"reads":[{"kind","path","digest"},..]}`
/// and `{"type","digest"}`.
pub fn reads_sources(files: &[(String, String)]) -> Result<String, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let (defs, types) = incremental::proof_reads(&fe.program, &fe.tables, &fe.sm);
    let mut out = String::new();
    for (path, d) in &defs {
        let uses: Vec<String> = d.uses.iter().map(|u| diag::json_str(u)).collect();
        let reads: Vec<String> = d
            .reads
            .iter()
            .map(|r| {
                format!(
                    "{{\"kind\":\"{}\",\"path\":{},\"digest\":\"{}\"}}",
                    r.kind,
                    diag::json_str(&r.path),
                    r.digest
                )
            })
            .collect();
        let kind = match d.kind {
            ast::FnKind::Exec => "fn",
            ast::FnKind::Spec => "spec fn",
            ast::FnKind::Proof => "proof fn",
        };
        out.push_str(&format!(
            "{{\"def\":{},\"kind\":\"{kind}\",\"key\":\"{}\",\"contract\":\"{}\",\"decl\":{},\"uses\":[{}],\"reads\":[{}]}}\n",
            diag::json_str(path),
            d.key,
            d.contract,
            d.decl.as_ref().map_or("null".to_string(), |x| format!("\"{x}\"")),
            uses.join(","),
            reads.join(",")
        ));
    }
    for (path, digest) in &types {
        out.push_str(&format!(
            "{{\"type\":{},\"digest\":\"{digest}\"}}\n",
            diag::json_str(path)
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
enum ProofNeed {
    None,
    /// The rows of these definitions: the L1 row of one (`query surface
    /// DEF`), the definitions in a scope (`query status PATH..`, backlog
    /// 2D-17).
    Defs(std::collections::BTreeSet<String>),
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
        // With a scope, `status` and `tests` show the definitions in it
        // alone, so only those are proved (backlog 2D-17).
        (Some("query"), Some("status" | "tests")) => match ix.scope(&request[2..]) {
            Ok(None) => ProofNeed::All,
            Ok(Some(scope)) => ProofNeed::Defs(
                ix.defs
                    .values()
                    .filter(|d| scope.has(&d.path, &d.unit))
                    .map(|d| d.path.clone())
                    .collect(),
            ),
            // A usage error: nothing is shown, so nothing is proved.
            Err(_) => ProofNeed::None,
        },
        (Some("query"), Some("surface")) => match request.get(2) {
            Some(d) if ix.defs.contains_key(d) => ProofNeed::Defs([d.to_string()].into()),
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
        ProofNeed::Defs(only) => {
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
    /// The input aliases its parser took (`Diags::aliases`).
    aliases: Vec<diag::Diag>,
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
            let mut it = fe.source.mods.into_iter();
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
            let (mods, file_diags, file_aliases) = match prev {
                Some((pf, mods)) if pf.name == name && pf.text == text => {
                    self.stats.reused += 1;
                    (mods, pf.diags, pf.aliases)
                }
                _ => {
                    self.stats.parsed += 1;
                    let mut d = Diags::default();
                    let toks = lexer::lex(id, &text, &mut d);
                    let mut p = parser::Parser::new(toks, &mut d, &text);
                    (p.parse_program().mods, d.items, d.aliases)
                }
            };
            diags.items.extend(file_diags.iter().cloned());
            diags.aliases.extend(file_aliases.iter().cloned());
            self.files.push(ParsedFile {
                name,
                text,
                mods: mods.len(),
                diags: file_diags,
                aliases: file_aliases,
            });
            program.mods.extend(mods);
        }
        // After the files' units (`update` takes them apart by count).
        stdlib::add(&mut sm, &mut program, &mut diags);
        self.stats.checks += 1;
        let (resolved, tables) = check::check_resolved(program.clone(), &mut diags);
        self.fe = Some(Frontend {
            sm,
            diags,
            source: program,
            program: resolved,
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
        verify_frontend(self.fe(), cache_dir, true, None)
    }

    /// As `emit_llvm_sources_cached` on the files of the last `update`.
    pub fn emit_llvm(&self, cache_dir: Option<&std::path::Path>) -> LlvmOutput {
        emit_llvm_frontend(self.fe(), false, cache_dir)
    }
}

#[cfg(test)]
mod spread_tests {
    /// The integer form prints what `{:.2}` of the quotient printed, away
    /// from exact ties.
    #[test]
    fn hundredths_are_the_rounded_quotient() {
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        for _ in 0..10_000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let lo = seed % 5_000_000 + 1;
            let hi = lo + (seed >> 20) % (3 * lo);
            let x = super::hundredths(hi, lo);
            let q = hi as f64 / lo as f64;
            if ((q * 100.0).fract() - 0.5).abs() < 1e-6 {
                continue;
            }
            assert_eq!(
                format!("{}.{:02}", x / 100, x % 100),
                format!("{q:.2}"),
                "{hi}/{lo}"
            );
        }
    }
}
