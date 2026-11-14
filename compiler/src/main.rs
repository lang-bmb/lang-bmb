use std::process::ExitCode;

// `print!` and `println!` that end the process quietly when standard output
// is closed (`bmb hash DIR | head`, backlog 1D-24) instead of panicking with
// a backtrace: exit status 141, as a process killed by SIGPIPE. SIGPIPE
// itself stays ignored so that a solver that dies is an error, not a signal.
macro_rules! print {
    ($($t:tt)*) => { write_stdout(format_args!($($t)*)) };
}
macro_rules! println {
    () => { print!("\n") };
    ($($t:tt)*) => { write_stdout(format_args!("{}\n", format_args!($($t)*))) };
}

const CLOSED_STDOUT: u8 = 141;

fn write_stdout(args: std::fmt::Arguments) {
    use std::io::Write;
    match std::io::stdout().lock().write_fmt(args) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
            std::process::exit(CLOSED_STDOUT.into())
        }
        Err(e) => panic!("failed printing to stdout: {e}"),
    }
}

const USAGE: &str = "usage:
  bmb check <file.bmb>...   check sources; diagnostics as JSONL on stdout
  bmb verify [--emit-smt DIR] [--cache DIR] [--stats FILE] [--audit] [--bounded-search] [--def PATH]... <file.bmb>...
                            check and prove; diagnostics and per-definition status as JSONL;
                            --def proves only the named definitions (the whole program is
                            still checked; each gets its whole-program status);
                            --cache keeps proved obligations in DIR and skips them next time,
                            and reuses unchanged definitions without encoding them;
                            --stats writes what the caches did as one JSON line to FILE
                            (definitions encoded, reused, lowered, IR reused, proof cache
                            hits and misses, damaged cache lines; emit-llvm and build take
                            it too, and add the runtime checks the IR keeps);
                            --audit proves each definition the cache would reuse anyway and
                            compares (script as the solver receives it, status, proved
                            obligations); --stats then lists the differences (audit_mismatches);
                            --bounded-search asks each undecided obligation once more with the
                            inputs bounded (|integers| <= 8, lengths <= 4, ADR-0108): a model is
                            a counterexample, none is the E0811 field `bounded`
  bmb stability [--seeds N] <file.bmb>...
                            prove under solver seeds 0..N (default 5); per-definition verdicts
                            and rlimit as JSONL; fails if a verdict depends on the seed
  bmb emit-llvm [--cache DIR] [--stats FILE] [--audit] [--all-checks] <file.bmb>... -o OUT.ll
                            LLVM IR; unproved obligations keep runtime checks;
                            --all-checks skips the proof and keeps every check;
                            --audit as for verify, also for the IR the cache would reuse
  bmb build [--cache DIR] [--stats FILE] [--audit] <file.bmb>... -o OUT
                            check, prove (all obligations) and build an executable
  bmb fmt [--check] <file.bmb>...
                            print the normal form (ADR-0019); --check fails if a file differs
  bmb fix [--write|--print] [--confirm] [--induction] <file.bmb>...
                            apply mechanical fixes (diagnostic replacements, and on a program
                            that checks the canonical `for` of counting `while` loops, W0907,
                            and the hash of each `#[rationale(..)]` that carries none;
                            --confirm also rewrites the hashes that are no longer current, W1001)
                            and re-check until none applies; JSONL report of applied fixes,
                            remaining diagnostics
                            and a summary; --write rewrites changed files in place, --print
                            prints the fixed text instead of the report; fails if errors remain;
                            --induction also runs the prover: for a function it does not
                            prove, the `proof fn` from a recursive predicate to a predicate
                            of the same parameters that a postcondition reaches, and its
                            call, written only when the gate proves both (ADR-0135)
  bmb uses [--write] [--cache DIR] <file.bmb>...
                            for each definition that fails to prove, find a minimal #[uses(..)]
                            list from its current entries and the hidden predicates its
                            diagnostics name; JSONL row per definition (status proved, unchanged
                            or not_found, with the attribute replacement) and a summary;
                            --write rewrites changed files; fails if a definition stays unproved;
                            --cache uses and updates the caches in DIR as `verify --cache` does
  bmb lint [--cache DIR] [--select CODES] <file.bmb>...
                            contract lint (warnings W09xx as JSONL, then a summary): preconditions
                            that cannot hold together, preconditions the function's proof does
                            not need, postconditions no caller's proof uses, body mutations the
                            specification does not distinguish; re-verifications and mutants are
                            capped; the summary also gives the rlimit and undecided count of the
                            W0901 and W0905 satisfiability checks; fails only if the program does
                            not check or the gate cannot run; --cache uses and updates the caches
                            in DIR as `verify --cache` does; --select W0901,W0905 runs only the
                            listed checks; W0918 (postconditions that allow two results for one
                            input, with the witness) runs only when --select names it
  bmb probe --def PATH [--loop K] --candidate EXPR [--candidate EXPR].. <file.bmb>...
                            candidate invariants for loop K (from 0, source order) of PATH,
                            tried without changing the files (ADR-0109): per candidate its first
                            answer on entry and kept by the body, then the largest subset that
                            is inductive together (Houdini) and whether PATH proves with it
  bmb examples [--count N] DEF.. -- <file.bmb>...
                            examples of each named function's contract, from `requires` and
                            `ensures` alone (the body is not read, the program is checked, not
                            proved), as JSONL in a fixed order: per input (N of them, default 3,
                            each different from the earlier ones, small values first) a row
                            `allowed` (input and a result the contract allows), a second
                            `allowed` row when it allows another result for that input, and a
                            row `rejected` (a result it rejects for that input); then `rejected`
                            rows at other inputs while fewer than N are shown, then up to N rows
                            `excluded` (inputs `requires` rejects), then a summary row (rows per
                            kind, undecided queries, rlimit); `after` holds the values behind
                            `&mut` parameters; values are models of the gate's encoding, printed
                            as counterexamples are, lengths at most 4; a query that is not
                            decided by the solver is asked again at written small inputs, and
                            if still not decided shows nothing and is counted; the spec fns
                            a contract names are read by their definitions, in `#[uses]` or
                            not; a definition whose values cannot be shown in full gets only
                            a summary row with `reason`
  bmb mutate --def PATH [--def PATH].. <file.bmb>...
                            every W0904 body mutation of the named functions, unproved, as
                            JSONL rows with the changed unit's normal form, then a summary
  bmb weaken OLD NEW        compare the contracts of two versions (each a file or a workspace):
                            per definition same, refined, compatible, weakened, undecided,
                            signature, added or removed, by SMT implication with counterexamples;
                            W0908 for each weakened contract, then a summary; fails if a
                            contract got weaker or a comparison was not decided
  bmb weaken --readers FILE OLD NEW
                            the same comparison for each reader in FILE (JSONL lines
                            {\"def\",\"uses\"}), revealing only the spec fns that reader reveals,
                            forward direction only; one row per reader, in order
  bmb reads <file.bmb>...   each definition's proof key as rows (the read set of its proof:
                            own text, callee contracts, revealed spec fns, declarations, types)
                            and each type's digest, as JSONL
  bmb hash <file.bmb>...    nameless identities and hashes (id, body, iface) as JSONL
  bmb map [--cache DIR] [--budget BYTES] [--offset ROWS] [--limit ROWS] [L0|L1|L2|all] <file.bmb>...
                            standard maps as JSONL (ADR-0020); the window flags as for query
  bmb query [--cache DIR] [--budget BYTES] [--offset ROWS] [--limit ROWS]
            <units|overview|surface|edges|slice|find|contracts|status|impact|duplicates|effects|effect|tests> [args] -- <file.bmb>...
                            read queries as JSONL (ADR-0020); --cache as for verify (only
                            the L1 rows, `surface` and `status` prove);
                            units, status, duplicates, effects, effect E and tests take paths
                            (units, definitions, types) and answer for what those hold alone
                            (status and tests then prove only that; duplicates: the groups a
                            named definition is in; effects with one definition: what it
                            reaches); edges DEF [out|in|both] [DEPTH] [KIND,..] follows only
                            the edges of the listed kinds (calls, contract, uses, writes ..);
                            overview [PATH..] lists units with counts in place of their
                            definitions; --budget, --offset and --limit print a window of
                            the response (ADR-0127): the rows from the offset, whole rows
                            within the budget, and a last `truncated` row (rows shown, rows
                            and bytes of the whole response, `next`) when rows are left out
  bmb import DIR <file.bmb>...
                            create or replace workspace DIR from files, in normal form
  bmb apply DIR OPS.json    commit a transaction (JSON array of unit/put/remove/remove_unit/
                            rename/expect ops)
                            to workspace DIR; rejected as a whole on any structural or type error
  bmb mcp [--write-tool] [--compact] [--tool-docs] [--file-tools] [--ast-tools] [--item-docs] [--stats] [--bounded|--bounded-16k] DIR
                            MCP server (stdio) for workspace DIR: query, map, verify, build, tx_*;
                            --write-tool also offers `write` (unit, definitions, removals,
                            commit and proof in one call); --compact shows results in the
                            compact agent view (surfaces as Rust-shaped text without hashes
                            or positions, verified definitions as a count); --tool-docs gives
                            tool descriptions that document the tools on their own (what
                            each query returns, `write` as the only way to write);
                            --file-tools offers `read_file` and `edit_file` (a unified diff,
                            committed and proved in one call); --ast-tools offers `read_ast`
                            and `edit_ast` (a JSON Patch on the JSON syntax tree, committed and
                            proved in one call) and gives positions as pointers into the tree;
                            --bounded gives `query` and `map` results as windows of at most
                            32768 bytes with `offset` and `limit` arguments (ADR-0127);
                            --bounded-16k is the same at 16384 bytes (the default is
                            measured, backlog 2D-39)
  bmb ast <file.bmb>...     the syntax tree as one JSON document (the JSON AST edit surface's,
                            keyed by unit; positions are not part of it)
  bmb dump tokens <file.bmb>...
                            the lexer's tokens as JSONL (file, span, kind, normal text, value)
  bmb dump parse <file.bmb>...
                            the parsed program in normal form, before any check
  bmb codes [--md]          list diagnostic codes as JSONL (or the Markdown reference page)

Any <file.bmb> argument may also be a workspace directory (its src/*.bmb).
  bmb doctor                check the pinned toolchain (z3, LLVM) as JSONL";

/// A leading `--cache DIR` (proof cache and per-definition results, as for
/// `verify`); returns it and the remaining arguments.
fn cache_flag(args: &[String]) -> (Option<&str>, &[String]) {
    match args {
        [flag, dir, rest @ ..] if flag == "--cache" => (Some(dir.as_str()), rest),
        _ => (None, args),
    }
}

/// The window flags of `map` and `query` (ADR-0127), in any order before
/// the request: `--budget BYTES`, `--offset ROWS`, `--limit ROWS`. `None`
/// when there is none, and the response is then printed whole.
type Window = Option<(usize, usize, Option<usize>)>;

fn window_flags(mut args: &[String]) -> Result<(Window, &[String]), String> {
    let (mut budget, mut offset, mut limit) = (None, None, None);
    while let [flag, n, rest @ ..] = args {
        let slot = match flag.as_str() {
            "--budget" => &mut budget,
            "--offset" => &mut offset,
            "--limit" => &mut limit,
            _ => break,
        };
        *slot = Some(
            n.parse::<usize>()
                .map_err(|_| format!("{flag} takes a count, not `{n}`"))?,
        );
        args = rest;
    }
    let window = (budget.is_some() || offset.is_some() || limit.is_some())
        .then(|| (budget.unwrap_or(usize::MAX), offset.unwrap_or(0), limit));
    Ok((window, args))
}

/// `r` through the window, when there is one.
fn windowed(r: Result<String, String>, w: Window) -> Result<String, String> {
    match w {
        Some((budget, offset, limit)) => r.map(|t| bmb::map::bound(&t, budget, offset, limit)),
        None => r,
    }
}

fn main() -> ExitCode {
    bmb::with_stack(run)
}

fn run() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() > 1 => {
            let files = match read_files(&args[1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let out = bmb::check_sources(&files);
            print!("{}", out.diagnostics);
            if out.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("doctor") => {
            let mut ok = true;
            for (tool, want, res) in [
                (
                    "z3",
                    bmb::toolchain::z3_version(),
                    bmb::toolchain::check_z3(),
                ),
                (
                    "llvm",
                    bmb::toolchain::llvm_major(),
                    bmb::toolchain::check_llvm(),
                ),
            ] {
                let (status, detail) = match res {
                    Ok(v) => ("ok", v),
                    Err(e) => {
                        ok = false;
                        ("error", e)
                    }
                };
                println!(
                    "{{\"tool\":{},\"required\":{},\"status\":{},\"detail\":{}}}",
                    bmb::diag::json_str(tool),
                    bmb::diag::json_str(want),
                    bmb::diag::json_str(status),
                    bmb::diag::json_str(&detail)
                );
            }
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("stability") if args.len() > 1 => {
            let mut rest = &args[1..];
            let mut seeds = 5;
            if rest.first().map(String::as_str) == Some("--seeds") && rest.len() > 2 {
                match rest[1].parse() {
                    Ok(n) => seeds = n,
                    Err(_) => {
                        eprintln!("bmb: --seeds takes a number");
                        return ExitCode::from(2);
                    }
                }
                rest = &rest[2..];
            }
            let files = match read_files(rest) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::stability_sources(&files, seeds) {
                Ok((out, stable)) => {
                    print!("{out}");
                    if stable {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
            }
        }
        Some("verify") if args.len() > 1 => {
            let mut rest = &args[1..];
            let mut emit = None;
            let mut cache = None;
            let mut stats = None;
            let mut audit = false;
            let mut defs = std::collections::BTreeSet::new();
            loop {
                match rest.first().map(String::as_str) {
                    Some("--audit") if rest.len() > 1 => {
                        audit = true;
                        rest = &rest[1..];
                        continue;
                    }
                    Some("--bounded-search") if rest.len() > 1 => {
                        bmb::verify::set_bounded_search(true);
                        rest = &rest[1..];
                        continue;
                    }
                    Some("--emit-smt") if rest.len() > 2 => emit = Some(rest[1].clone()),
                    Some("--cache") if rest.len() > 2 => cache = Some(rest[1].clone()),
                    Some("--stats") if rest.len() > 2 => stats = Some(rest[1].clone()),
                    Some("--def") if rest.len() > 2 => {
                        defs.insert(rest[1].clone());
                    }
                    _ => break,
                }
                rest = &rest[2..];
            }
            let files = match read_files(rest) {
                Ok(f) => f,
                Err(code) => return code,
            };
            // `--emit-smt` needs every definition's script, so every
            // definition is encoded (the proof cache still applies).
            let cache_dir = cache.as_deref().map(std::path::Path::new);
            let run = || {
                if defs.is_empty() {
                    Ok(bmb::verify_sources_opts(&files, cache_dir, emit.is_none()))
                } else {
                    bmb::verify_sources_only(&files, cache_dir, emit.is_none(), &defs)
                }
            };
            let out = match if audit {
                bmb::incremental::with_audit(run)
            } else {
                run()
            } {
                Ok(out) => out,
                Err(e) => {
                    eprintln!("bmb: {e}");
                    return ExitCode::from(2);
                }
            };
            if let Some(dir) = emit {
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    eprintln!("bmb: cannot create {dir}: {e}");
                    return ExitCode::from(2);
                }
                for (def, script) in &out.scripts {
                    // What the solver received (canonical symbols, ADR-0063),
                    // and beside it the legend of each query's symbols.
                    let (text, legend) = bmb::smt::canonical_script(script);
                    let file = format!("{dir}/{}.smt2", def.replace("::", "."));
                    for (f, t) in [(file.clone(), text), (format!("{file}.legend"), legend)] {
                        if let Err(e) = std::fs::write(&f, t) {
                            eprintln!("bmb: cannot write {f}: {e}");
                            return ExitCode::from(2);
                        }
                    }
                }
            }
            if let Some(file) = stats
                && let Err(e) = std::fs::write(&file, verify_stats(&out, None))
            {
                eprintln!("bmb: cannot write {file}: {e}");
                return ExitCode::from(2);
            }
            print!("{}", out.output);
            if out.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some(cmd @ ("emit-llvm" | "build")) => {
            let Some(pos) = args.iter().position(|a| a == "-o") else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let Some(out_path) = args.get(pos + 1).cloned() else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let mut inputs: Vec<String> = args[1..pos]
                .iter()
                .chain(args[pos + 2..].iter())
                .cloned()
                .collect();
            let mut cache = None;
            if let Some(c) = inputs.iter().position(|a| a == "--cache")
                && c + 1 < inputs.len()
            {
                cache = Some(inputs.remove(c + 1));
                inputs.remove(c);
            }
            let mut stats = None;
            if let Some(c) = inputs.iter().position(|a| a == "--stats")
                && c + 1 < inputs.len()
            {
                stats = Some(inputs.remove(c + 1));
                inputs.remove(c);
            }
            // emit-llvm --all-checks: no proof, every obligation keeps its
            // runtime check (runs unproved programs such as mutants, 2A-10).
            let mut all_checks = false;
            if cmd == "emit-llvm"
                && let Some(c) = inputs.iter().position(|a| a == "--all-checks")
            {
                inputs.remove(c);
                all_checks = true;
            }
            // --audit: as for verify, every reused proof and IR is computed
            // again and compared (--stats lists the differences).
            let mut audit = false;
            if let Some(c) = inputs.iter().position(|a| a == "--audit") {
                inputs.remove(c);
                audit = true;
            }
            let files = match read_files(&inputs) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let cached = || {
                bmb::emit_llvm_sources_cached(&files, cache.as_deref().map(std::path::Path::new))
            };
            let out = if all_checks {
                bmb::emit_llvm_sources_with(&files, true)
            } else if audit {
                bmb::incremental::with_audit(cached)
            } else {
                cached()
            };
            if let Some(file) = stats
                && let Err(e) = std::fs::write(&file, verify_stats(&out.verify, Some(out.checks)))
            {
                eprintln!("bmb: cannot write {file}: {e}");
                return ExitCode::from(2);
            }
            print!("{}", out.verify.output);
            let Some(ir) = out.ir else {
                return ExitCode::from(1);
            };
            if cmd == "emit-llvm" {
                if let Err(e) = std::fs::write(&out_path, ir) {
                    eprintln!("bmb: cannot write {out_path}: {e}");
                    return ExitCode::from(2);
                }
                return if out.verify.ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                };
            }
            // build: no binary without a complete proof.
            if !out.verify.ok || out.checks > 0 {
                eprintln!("bmb: build refused: the program is not fully proved");
                return ExitCode::from(1);
            }
            if !out.has_main {
                eprintln!("bmb: build needs a `main`");
                return ExitCode::from(1);
            }
            match bmb::toolchain::build_executable(&ir, &out_path) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("bmb: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("fmt") if args.len() > 1 => {
            let check = args[1] == "--check";
            let paths = if check { &args[2..] } else { &args[1..] };
            let files = match read_files(paths) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::format_sources(&files) {
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
                Ok(units) => {
                    let text = units.values().cloned().collect::<Vec<_>>().join("\n");
                    if !check {
                        print!("{text}");
                        return ExitCode::SUCCESS;
                    }
                    let given = files
                        .iter()
                        .map(|(_, t)| t.as_str())
                        .collect::<Vec<_>>()
                        .join("\n");
                    if given == text {
                        ExitCode::SUCCESS
                    } else {
                        eprintln!("bmb: not in normal form");
                        ExitCode::from(1)
                    }
                }
            }
        }
        Some("fix") if args.len() > 1 => {
            let mut rest = &args[1..];
            let (mut write, mut print_text, mut confirm, mut induction) =
                (false, false, false, false);
            loop {
                match rest.first().map(String::as_str) {
                    Some("--write") => write = true,
                    Some("--print") => print_text = true,
                    Some("--confirm") => confirm = true,
                    Some("--induction") => induction = true,
                    _ => break,
                }
                rest = &rest[1..];
            }
            if rest.is_empty() || (write && print_text) {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            let files = match read_files(rest) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let out = bmb::fix_sources_opts(&files, confirm, induction);
            if print_text {
                let text = out
                    .files
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                print!("{text}");
            } else {
                print!("{}", bmb::with_written(out.report(&files), write));
            }
            if write {
                for (path, text) in out
                    .files
                    .iter()
                    .zip(&files)
                    .filter_map(|(a, b)| (a.1 != b.1).then_some((&a.0, &a.1)))
                {
                    if let Err(e) = std::fs::write(path, text) {
                        eprintln!("bmb: cannot write {path}: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
            if out.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("uses") if args.len() > 1 => {
            let mut paths = &args[1..];
            let mut write = false;
            let mut cache = None;
            loop {
                match paths.first().map(String::as_str) {
                    Some("--write") => {
                        write = true;
                        paths = &paths[1..];
                    }
                    Some("--cache") if paths.len() > 2 => {
                        cache = Some(paths[1].clone());
                        paths = &paths[2..];
                    }
                    _ => break,
                }
            }
            if paths.is_empty() {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            let files = match read_files(paths) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let out = match bmb::uses_fill::fill_uses_cached(
                &files,
                cache.as_deref().map(std::path::Path::new),
            ) {
                Ok(o) => o,
                Err(diags) => {
                    print!("{diags}");
                    return ExitCode::from(1);
                }
            };
            print!("{}", bmb::with_written(out.report(&files), write));
            if write {
                for (path, text) in out
                    .files
                    .iter()
                    .zip(&files)
                    .filter_map(|(a, b)| (a.1 != b.1).then_some((&a.0, &a.1)))
                {
                    if let Err(e) = std::fs::write(path, text) {
                        eprintln!("bmb: cannot write {path}: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
            if out.ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("lint") if args.len() > 1 => {
            let mut cache = None;
            let mut select: Vec<&str> = bmb::lint::default_codes();
            let mut paths = &args[1..];
            loop {
                match paths {
                    [flag, dir, tail @ ..] if flag == "--cache" => {
                        cache = Some(dir.clone());
                        paths = tail;
                    }
                    [flag, codes, tail @ ..] if flag == "--select" => {
                        select.clear();
                        for c in codes.split(',') {
                            let Some(known) = bmb::lint::CODES.iter().find(|k| **k == c) else {
                                eprintln!(
                                    "bmb: lint has no check {c:?} (the checks are {})",
                                    bmb::lint::CODES.join(", ")
                                );
                                return ExitCode::from(2);
                            };
                            select.push(known);
                        }
                        paths = tail;
                    }
                    _ => break,
                }
            }
            if paths.is_empty() {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            let files = match read_files(paths) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let cache = cache.as_deref().map(std::path::Path::new);
            match bmb::lint::lint_sources_selected(&files, cache, &select) {
                Ok(out) => {
                    print!("{}", out.report());
                    if out.ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
            }
        }
        Some("mutate") if args.len() > 1 => {
            let mut defs = Vec::new();
            let mut rest = &args[1..];
            while let [flag, def, tail @ ..] = rest
                && flag == "--def"
            {
                defs.push(def.clone());
                rest = tail;
            }
            if defs.is_empty() || rest.is_empty() {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            let files = match read_files(rest) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::lint::mutate_sources(&files, &defs) {
                Ok(ms) => {
                    for m in &ms {
                        print!("{}", m.row());
                    }
                    println!("{{\"summary\":true,\"mutants\":{}}}", ms.len());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    print!("{e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("examples") if args.len() > 1 => {
            let mut count = 3usize;
            let mut rest = &args[1..];
            if let [flag, n, tail @ ..] = rest
                && flag == "--count"
            {
                match n.parse() {
                    Ok(k) if (1..=bmb::verify::MAX_EXAMPLES).contains(&k) => count = k,
                    _ => {
                        eprintln!(
                            "bmb: --count takes a number from 1 to {}",
                            bmb::verify::MAX_EXAMPLES
                        );
                        return ExitCode::from(2);
                    }
                }
                rest = tail;
            }
            let Some(sep) = rest.iter().position(|a| a == "--") else {
                eprintln!("bmb: put `--` between the definitions and the files");
                return ExitCode::from(2);
            };
            let (defs, paths) = (&rest[..sep], &rest[sep + 1..]);
            if defs.is_empty() || paths.is_empty() {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
            let files = match read_files(paths) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::examples_sources(&files, defs, count) {
                Ok(out) => {
                    print!("{out}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    print!("{e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("probe") if args.len() > 1 => {
            let (mut def, mut at, mut cands) = (None, 0usize, Vec::new());
            let mut rest = &args[1..];
            loop {
                match rest {
                    [flag, v, tail @ ..] if flag == "--def" => {
                        def = Some(v.clone());
                        rest = tail;
                    }
                    [flag, v, tail @ ..] if flag == "--loop" => {
                        let Ok(n) = v.parse() else {
                            eprintln!("{USAGE}");
                            return ExitCode::from(2);
                        };
                        at = n;
                        rest = tail;
                    }
                    [flag, v, tail @ ..] if flag == "--candidate" => {
                        cands.push(v.clone());
                        rest = tail;
                    }
                    _ => break,
                }
            }
            let Some(def) = def.filter(|_| !rest.is_empty() && !cands.is_empty()) else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let files = match read_files(rest) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::probe::probe(&files, &def, at, &cands) {
                Ok(out) => {
                    print!("{out}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    println!("{e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("weaken") if args.len() == 3 => {
            let (old, new) = match (read_files(&args[1..2]), read_files(&args[2..3])) {
                (Ok(o), Ok(n)) => (o, n),
                (Err(code), _) | (_, Err(code)) => return code,
            };
            match bmb::weaken::weaken_sources(&old, &new) {
                Ok(out) => {
                    print!("{}", out.report());
                    if out.ok() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
            }
        }
        Some("weaken") if args.len() == 5 && args[1] == "--readers" => {
            let readers = match std::fs::read_to_string(&args[2]) {
                Ok(text) => match bmb::weaken::parse_readers(&text) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("bmb: {e}");
                        return ExitCode::from(2);
                    }
                },
                Err(e) => {
                    eprintln!("bmb: cannot read {}: {e}", args[2]);
                    return ExitCode::from(2);
                }
            };
            let (old, new) = match (read_files(&args[3..4]), read_files(&args[4..5])) {
                (Ok(o), Ok(n)) => (o, n),
                (Err(code), _) | (_, Err(code)) => return code,
            };
            match bmb::weaken::weaken_for_readers(&old, &new, &readers) {
                Ok(out) => {
                    print!("{}", out.report());
                    if out.ok() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
            }
        }
        Some("reads") if args.len() > 1 => {
            let files = match read_files(&args[1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            respond(bmb::reads_sources(&files))
        }
        Some("hash") if args.len() > 1 => {
            let files = match read_files(&args[1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::hash_sources(&files) {
                Ok(out) => {
                    print!("{out}");
                    ExitCode::SUCCESS
                }
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
            }
        }
        Some("map") if args.len() > 1 => {
            let (cache, args) = cache_flag(&args[1..]);
            let (window, args) = match window_flags(args) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("bmb: {e}");
                    return ExitCode::from(2);
                }
            };
            let Some(first) = args.first() else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let (level, paths) = if matches!(first.as_str(), "L0" | "L1" | "L2" | "all") {
                (first.clone(), &args[1..])
            } else {
                ("all".to_string(), args)
            };
            let files = match read_files(paths) {
                Ok(f) => f,
                Err(code) => return code,
            };
            respond(windowed(
                bmb::map_request_cached(
                    &files,
                    &["map".into(), level],
                    cache.map(std::path::Path::new),
                ),
                window,
            ))
        }
        Some("query") if args.len() > 2 => {
            let (cache, args) = cache_flag(&args[1..]);
            let (window, args) = match window_flags(args) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("bmb: {e}");
                    return ExitCode::from(2);
                }
            };
            let Some(sep) = args.iter().position(|a| a == "--") else {
                eprintln!("bmb: put `--` between the query and the files");
                return ExitCode::from(2);
            };
            let files = match read_files(&args[sep + 1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let mut req = vec!["query".to_string()];
            req.extend(args[..sep].iter().cloned());
            respond(windowed(
                bmb::map_request_cached(&files, &req, cache.map(std::path::Path::new)),
                window,
            ))
        }
        Some("import") if args.len() > 2 => {
            let files = match read_files(&args[2..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::format_sources(&files) {
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
                Ok(units) => {
                    match bmb::workspace::write_units(std::path::Path::new(&args[1]), &units) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("bmb: {e}");
                            ExitCode::from(2)
                        }
                    }
                }
            }
        }
        Some("apply") if args.len() == 3 => {
            let text = if args[2] == "-" {
                let mut s = String::new();
                if std::io::Read::read_to_string(&mut std::io::stdin(), &mut s).is_err() {
                    eprintln!("bmb: cannot read stdin");
                    return ExitCode::from(2);
                }
                s
            } else {
                match std::fs::read_to_string(&args[2]) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("bmb: cannot read {}: {e}", args[2]);
                        return ExitCode::from(2);
                    }
                }
            };
            let ops = match bmb::workspace::parse_ops(&text) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("bmb: {e}");
                    return ExitCode::from(2);
                }
            };
            match bmb::workspace::commit(std::path::Path::new(&args[1]), &ops) {
                Ok(c) => {
                    print!("{}", c.diagnostics);
                    println!("{}", commit_line(&c));
                    if c.ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(e) => {
                    eprintln!("bmb: {e}");
                    ExitCode::from(2)
                }
            }
        }
        Some("mcp")
            if args.len() >= 2
                && args[1..args.len() - 1]
                    .iter()
                    .all(|a| bmb::mcp::FLAGS.contains(&a.as_str())) =>
        {
            let flags: Vec<&str> = args[1..args.len() - 1].iter().map(String::as_str).collect();
            match bmb::mcp::serve(std::path::Path::new(&args[args.len() - 1]), &flags) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                    ExitCode::from(CLOSED_STDOUT)
                }
                Err(e) => {
                    eprintln!("bmb: {e}");
                    ExitCode::from(2)
                }
            }
        }
        // Each front-end stage's output, for comparing another
        // implementation stage by stage (backlog 4A-03): the tokens, and
        // the parsed program in normal form before any check.
        Some("dump") if args.len() > 2 && matches!(args[1].as_str(), "tokens" | "parse") => {
            let files = match read_files(&args[2..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let out = if args[1] == "tokens" {
                bmb::dump_tokens(&files)
            } else {
                bmb::format_parsed(&files)
                    .map(|units| units.values().cloned().collect::<Vec<_>>().join("\n"))
            };
            match out {
                Ok(text) => {
                    print!("{text}");
                    ExitCode::SUCCESS
                }
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
            }
        }
        Some("ast") if args.len() > 1 => {
            let files = match read_files(&args[1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let (program, sm, diags) = bmb::workspace::parse_files(&files);
            if diags.has_errors() {
                print!("{}", diags.to_jsonl(&sm));
                return ExitCode::from(1);
            }
            match bmb::ast_json::export(&program) {
                Ok(v) => {
                    println!("{}", v.to_json());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("bmb: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("codes") if args.get(1).map(String::as_str) == Some("--md") => {
            print!("{}", bmb::diag::codes_markdown());
            ExitCode::SUCCESS
        }
        Some("codes") => {
            print!("{}", bmb::diag::codes_jsonl());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn commit_line(c: &bmb::workspace::Commit) -> String {
    let list = |xs: &[String]| {
        xs.iter()
            .map(|x| bmb::diag::json_str(x))
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "{{\"committed\":{},\"changed\":[{}],\"removed\":[{}]}}",
        c.ok,
        list(&c.changed),
        list(&c.removed)
    )
}

fn read_files(paths: &[String]) -> Result<Vec<(String, String)>, ExitCode> {
    let mut files = Vec::new();
    for path in paths {
        if std::path::Path::new(path).is_dir() {
            match bmb::workspace::read_dir_sources(std::path::Path::new(path)) {
                Ok(fs) => files.extend(fs),
                Err(e) => {
                    eprintln!("bmb: {e}");
                    return Err(ExitCode::from(2));
                }
            }
            continue;
        }
        match std::fs::read_to_string(path) {
            Ok(text) => files.push((path.clone(), text)),
            Err(e) => {
                eprintln!("bmb: cannot read {path}: {e}");
                return Err(ExitCode::from(2));
            }
        }
    }
    Ok(files)
}

fn respond(r: Result<String, String>) -> ExitCode {
    match r {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            if e.starts_with('{') {
                print!("{e}");
            } else {
                eprintln!("bmb: {e}");
            }
            ExitCode::from(1)
        }
    }
}

/// `--stats` of `verify`, `emit-llvm` and `build` (backlog 2A-14, BCB): which
/// definitions were encoded and lowered, which were reused by the incremental
/// cache, and the proof cache counts; for `emit-llvm` and `build` also the
/// runtime checks the IR keeps because their obligations are unproved.
fn verify_stats(out: &bmb::VerifyOutput, checks: Option<u32>) -> String {
    let names = |xs: &[String]| {
        let mut xs: Vec<String> = xs.iter().map(|x| bmb::diag::json_str(x)).collect();
        xs.sort();
        format!("[{}]", xs.join(","))
    };
    format!(
        "{{\"encoded\":{},\"reused\":{},\"lowered\":{},\"ir_reused\":{},\"proof_cache_hits\":{},\"proof_cache_misses\":{},\"audit_mismatches\":{},\"damaged\":{}{}}}\n",
        names(&out.incr.encoded),
        names(&out.incr.reused),
        names(&out.incr.lowered),
        names(&out.incr.ir_reused),
        out.cache.0,
        out.cache.1,
        names(&out.incr.audit_mismatches),
        out.incr.damaged,
        checks.map_or(String::new(), |n| format!(",\"runtime_checks\":{n}"))
    )
}
