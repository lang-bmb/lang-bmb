//! Z3 process connection (ADR-0017 §5, backlog 1B-09).
//!
//! The solver runs as a child process speaking SMT-LIB on stdin/stdout. Every
//! answer other than `unsat` is a failure for the proof gate.

use std::cell::Cell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::toolchain;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Unsat,
    Sat,
    Unknown(String),
}

pub struct Solver {
    /// The process that answers; `None` after a query ended at the memory
    /// limit in a process of its own, until the next `(reset)`.
    proc: Option<Proc>,
    /// The memory limit in megabytes that decides (`query_memory_mb` when
    /// the solver started): only a query that reaches it as the first query
    /// of its process is undecided for memory (ADR-0129).
    limit: u64,
    /// The commands of the query since its `(reset)`, as sent: what a new
    /// process is asked when the query's process ends at its memory limit.
    query: Vec<String>,
    /// Why there is no process (`proc` is `None`).
    down: String,
    /// The query before ended at a memory limit: the next one is asked
    /// alone at once (see `ask`).
    alone: bool,
    /// The canonical symbols of the query since the last `(reset)`
    /// (ADR-0063): what the solver sees names nothing of the program.
    canon: Canon,
    /// Queries since the last `take_timeouts` that the per-query backstop
    /// ended (`reason-unknown` "timeout", backlog INF-26).
    timeouts: usize,
}

/// One Z3 process.
struct Proc {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    started: Instant,
    /// Its `-memory` in megabytes.
    memory: u64,
    /// Queries (`(reset)`s) it was sent.
    queries: usize,
    /// Started to answer one query alone.
    alone: bool,
}

/// Canonical symbols of a query (ADR-0063, backlog T1-10e). Every quoted
/// symbol `|..|` (definitions, locals, callee results, selectors,
/// constructors, datatypes: everything the encoder names after the program)
/// becomes `|$k|`, numbered in order of first occurrence. Renaming the
/// program then leaves the query text the same, so the solver's work and
/// the proof cache key are the same too. `back` turns the solver's answers
/// (models, errors) into the program's names again. Comments and string
/// literals are kept as they are.
#[derive(Debug, Default, Clone)]
pub struct Canon {
    fwd: HashMap<String, String>,
    names: Vec<String>,
}

impl Canon {
    /// `text` with its quoted symbols in canonical form, numbering new ones.
    pub fn forward(&mut self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '|' => {
                    let mut sym = String::from("|");
                    for d in chars.by_ref() {
                        sym.push(d);
                        if d == '|' {
                            break;
                        }
                    }
                    match self.fwd.get(&sym) {
                        Some(canon) => out.push_str(canon),
                        None => {
                            let canon = format!("|${}|", self.names.len());
                            out.push_str(&canon);
                            self.fwd.insert(sym.clone(), canon);
                            self.names.push(sym);
                        }
                    }
                }
                '"' => {
                    out.push(c);
                    for d in chars.by_ref() {
                        out.push(d);
                        if d == '"' {
                            break;
                        }
                    }
                }
                ';' => {
                    out.push(c);
                    for d in chars.by_ref() {
                        out.push(d);
                        if d == '\n' {
                            break;
                        }
                    }
                }
                _ => out.push(c),
            }
        }
        out
    }

    /// `text` from the solver with canonical symbols named as in the program,
    /// written as the solver writes them: the solver prints `|$2|` as `$2`
    /// and the values of an uninterpreted sort `|$0|` as `$0!val!0`, so a
    /// token, quoted or not, that starts with `$k` gets the original name in
    /// place of `$k`, quoted only when it is not a simple symbol (SMT-LIB).
    pub fn back(&self, text: &str) -> String {
        if self.names.is_empty() || !text.contains('$') {
            return text.to_string();
        }
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::with_capacity(text.len());
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '|' {
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '|' {
                    j += 1;
                }
                let inner: String = chars[i + 1..j.min(chars.len())].iter().collect();
                match self.rename(&inner) {
                    Some(name) => out.push_str(&quote(&name)),
                    None => {
                        out.push('|');
                        out.push_str(&inner);
                        if j < chars.len() {
                            out.push('|');
                        }
                    }
                }
                i = j + 1;
            } else if c == '$' && (i == 0 || !is_symbol_char(chars[i - 1])) {
                let mut j = i;
                while j < chars.len() && is_symbol_char(chars[j]) {
                    j += 1;
                }
                let tok: String = chars[i..j].iter().collect();
                match self.rename(&tok) {
                    Some(name) => out.push_str(&quote(&name)),
                    None => out.push_str(&tok),
                }
                i = j;
            } else {
                out.push(c);
                i += 1;
            }
        }
        out
    }

    /// The unquoted name of token `tok` (`$k` then a suffix) in the program:
    /// the original symbol's name, then the suffix.
    fn rename(&self, tok: &str) -> Option<String> {
        let rest = tok.strip_prefix('$')?;
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        let k: usize = rest[..digits].parse().ok()?;
        let orig = self.names.get(k)?;
        let orig = orig.strip_prefix('|')?.strip_suffix('|')?;
        Some(format!("{orig}{}", &rest[digits..]))
    }

    /// The program's name of each canonical symbol, in order (`|$0|` first).
    pub fn legend(&self) -> &[String] {
        &self.names
    }
}

/// A character of an SMT-LIB simple symbol.
fn is_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "~!@$%^&*_-+=<>.?/".contains(c)
}

/// `name` as the solver prints a symbol: bare when it is a simple symbol.
fn quote(name: &str) -> String {
    let simple = !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name.chars().all(is_symbol_char);
    if simple {
        name.to_string()
    } else {
        format!("|{name}|")
    }
}

/// `text` (one query) as the solver receives it (ADR-0063): its commands
/// only (text between commands, such as comments, never reaches the
/// solver), with canonical symbols. The proof cache key's input.
pub fn canonical(text: &str) -> String {
    let mut canon = Canon::default();
    split_commands(text)
        .iter()
        .map(|cmd| canon.forward(cmd))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A script of queries that each start with `(reset)` (`--emit-smt`), each
/// query with its own canonical symbols, and the legend of every query in
/// order: `; query K` then one `|$k| NAME` line per symbol.
pub fn canonical_script(script: &str) -> (String, String) {
    let mut out = String::new();
    let mut legend = String::new();
    let mut canon = Canon::default();
    let mut query = 0;
    for line in script.split_inclusive('\n') {
        if line.trim() == "(reset)" {
            if !canon.legend().is_empty() {
                legend_of(&mut legend, query, &canon);
            }
            canon = Canon::default();
            query += 1;
        }
        if line.trim_start().starts_with(';') {
            out.push_str(line);
        } else {
            out.push_str(&canon.forward(line));
        }
    }
    if !canon.legend().is_empty() {
        legend_of(&mut legend, query, &canon);
    }
    (out, legend)
}

fn legend_of(out: &mut String, query: usize, canon: &Canon) {
    out.push_str(&format!("; query {query}\n"));
    for (k, name) in canon.legend().iter().enumerate() {
        out.push_str(&format!("|${k}| {name}\n"));
    }
}

/// Wall-clock backstop in seconds, per query (Z3's soft timeout `-t`).
/// Results are governed by the deterministic `rlimit`; this only stops a
/// query whose rlimit ticks slowly. Reaching it is reported as its own reason
/// (`unknown_reason`), as the verdict then depends on the machine: a ground
/// retry that spent its 5,000,000 in 194 s on one machine was cut off on a
/// slower one, and the two runs' rlimit totals differed (backlog INF-26).
/// Per query, so a query's share no longer depends on how long its process
/// had been running (with the process limit alone it was 300 to 360 s).
const BACKSTOP_SECS: u64 = 300;

/// The per-query backstop in milliseconds. `BMB_Z3_BACKSTOP_MS` may lower it
/// (for tests); it can never raise it.
pub fn backstop_ms() -> u64 {
    std::env::var("BMB_Z3_BACKSTOP_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map_or(BACKSTOP_SECS * 1000, |v| v.min(BACKSTOP_SECS * 1000))
}

/// A process may be reused for a new definition only this long after it
/// started (backlog 1D-18).
const REUSE_SECS: u64 = 60;

/// The process's hard limit (`-T`): only for a process that does not stop
/// at its soft timeout.
const HARD_SECS: u64 = REUSE_SECS + 2 * BACKSTOP_SECS;

/// The reason in an "undecided" diagnostic for Z3's `reason-unknown`.
pub fn unknown_reason(why: &str) -> String {
    if why.contains("timeout") {
        format!(
            "wall-clock backstop of {} ms reached before the rlimit; this verdict depends on the machine",
            backstop_ms()
        )
    } else if why.contains("canceled") || why.contains("resource") {
        format!("resource limit reached, rlimit {}", rlimit())
    } else {
        why.to_string()
    }
}

static STARTS: AtomicUsize = AtomicUsize::new(0);

/// Solver starts attempted by this process (successful or not). Tests use it
/// to show that a request did not need the solver.
pub fn starts() -> usize {
    STARTS.load(Ordering::Relaxed)
}

/// The pinned resource limit per obligation. `BMB_Z3_RLIMIT` may lower it
/// (for tests of the gate); it can never raise it.
pub fn rlimit() -> u64 {
    let pinned: u64 = toolchain::pinned("Z3_RLIMIT")
        .parse()
        .expect("Z3_RLIMIT must be an integer");
    std::env::var("BMB_Z3_RLIMIT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map_or(pinned, |v| v.min(pinned))
}

/// The pinned memory limit of a query in megabytes (Z3's `-memory`, which
/// counts a process: see `working_memory_mb` for how a query gets a count of
/// its own). A query that reaches it is undecided, as one that reaches the
/// `rlimit` is: without it such a query grew until the system killed the
/// process (backlog T2-16). `BMB_Z3_MEMORY_MB` may lower it (for tests); it
/// can never raise it.
pub fn memory_mb() -> u64 {
    let pinned: u64 = toolchain::pinned("Z3_MEMORY_MB")
        .parse()
        .expect("Z3_MEMORY_MB must be an integer");
    std::env::var("BMB_Z3_MEMORY_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map_or(pinned, |v| v.min(pinned))
}

/// The pinned memory limit in megabytes of an advisory query (`advisory`):
/// the lint's satisfiability checks and its re-verifications of changed
/// definitions (backlog T2-16). A query that is
/// false, or satisfiable, under a recursive predicate with a quantifier in
/// it makes Z3 grow without reaching its `rlimit`, and the time to the
/// limit grows with the limit: the port-shape experiment's `tree.bmb` has
/// 44 such queries, 248 s of lint at the gate's limit and 21 s at this one
/// (as measured then; the queries after one that ended at the limit are
/// asked since backlog T2-17, which about doubles the solver's time there).
/// Measured on `lib/std` and that experiment (13 files, 5,575 advisory
/// queries): no proof raised its process's peak above 21 MB, one model
/// raised it to 72 MB, and the lint's output is the same bytes at half
/// this limit. Never above `memory_mb`, so what lowers that lowers this.
pub fn advisory_memory_mb() -> u64 {
    let pinned: u64 = toolchain::pinned("Z3_ADVISORY_MEMORY_MB")
        .parse()
        .expect("Z3_ADVISORY_MEMORY_MB must be an integer");
    pinned.min(memory_mb())
}

thread_local! {
    /// Set while `advisory` runs.
    static ADVISORY: Cell<bool> = const { Cell::new(false) };
}

/// Runs `f` with every solver it starts under the advisory memory limit
/// (`advisory_memory_mb`) in place of the gate's. For queries whose
/// answer is a report, never a verdict of the gate: a query that reaches
/// the limit is undecided, as at the gate's limit, so the smaller limit
/// can only leave a report out. Deterministic like the gate's limit (Z3
/// counts its own allocations).
pub fn advisory<R>(f: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            ADVISORY.with(|a| a.set(self.0));
        }
    }
    let _restore = Restore(ADVISORY.with(|a| a.replace(true)));
    f()
}

/// The memory limit in megabytes that decides a query asked now: the
/// gate's, or the advisory one inside `advisory`. Part of the proof cache's
/// key (`cache::ProofCache::key`).
pub fn query_memory_mb() -> u64 {
    if ADVISORY.with(|a| a.get()) {
        advisory_memory_mb()
    } else {
        memory_mb()
    }
}

/// The memory limit in megabytes of a process that answers one query after
/// another, for the deciding limit `limit` (ADR-0129): half of it. Z3
/// counts the allocations of its process, and a process keeps a little of
/// each query it answered (measured on `lib/std`: 0.1 MB after the first
/// `(reset)`, 0.45 MB after 2,213 queries), so what a query's peak is
/// depends on what its process answered before: by less than 0.1 MB there,
/// in both directions. A query is therefore undecided for memory only when
/// it reaches `limit` as the first query of a new process, which is the
/// same state for every query; and a process that has answered others is
/// trusted only below this smaller limit, so that a query it answers would
/// also have been answered alone (its peak there would have to be larger
/// by the other half of the limit to differ). At least 1: Z3 reads 0 as no
/// limit.
pub fn working_memory_mb(limit: u64) -> u64 {
    (limit / 2).max(1)
}

/// Whether a solver error is the memory limit. The process has ended: the
/// next query needs a new one.
fn out_of_memory(e: &str) -> bool {
    e.contains("out of memory")
}

/// The error of a query that ended at the memory limit.
const OUT_OF_MEMORY: &str = "z3: out of memory";

/// The reason in an "undecided" diagnostic for a solver error that is a
/// limit (the `rlimit`, the memory limit); `None` for any other error.
pub fn limit_reason(e: &str) -> Option<String> {
    if out_of_memory(e) {
        Some(format!("memory limit reached, {} MB", query_memory_mb()))
    } else if e.contains("canceled") {
        Some(format!("resource limit reached, rlimit {}", rlimit()))
    } else {
        None
    }
}

impl Proc {
    /// Starts the pinned Z3 with memory limit `memory` (megabytes).
    fn start(memory: u64) -> Result<Proc, String> {
        STARTS.fetch_add(1, Ordering::Relaxed);
        let mut child = Command::new("z3")
            .args([
                "-in",
                "-smt2",
                &format!("-t:{}", backstop_ms()),
                &format!("-T:{HARD_SECS}"),
                &format!("-memory:{memory}"),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start z3: {e}"))?;
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut p = Proc {
            child,
            stdin,
            stdout,
            started: Instant::now(),
            memory,
            queries: 0,
            alone: false,
        };
        // Every command answers, so an error is seen at the command that caused it.
        match p.ask("(set-option :print-success true)")?.as_str() {
            "success" => Ok(p),
            r => Err(format!("{r} (in `(set-option :print-success true)`)")),
        }
    }

    /// Sends one command and reads its answer.
    fn ask(&mut self, text: &str) -> Result<String, String> {
        let sent = self
            .stdin
            .write_all(text.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush());
        if let Err(e) = sent {
            return Err(self
                .ended()
                .unwrap_or_else(|| format!("z3 input closed: {e}")));
        }
        self.read_response()
    }

    /// The error for a process that ended at its memory limit: Z3 ends with
    /// 101 at `-memory` and its message goes to its standard error.
    fn ended(&mut self) -> Option<String> {
        match self.child.wait().ok()?.code()? {
            101 => Some(OUT_OF_MEMORY.into()),
            _ => None,
        }
    }

    /// Reads one complete s-expression or atom from the solver.
    fn read_response(&mut self) -> Result<String, String> {
        let mut out = String::new();
        let mut depth: i64 = 0;
        loop {
            let mut line = String::new();
            let n = self
                .stdout
                .read_line(&mut line)
                .map_err(|e| format!("z3 output error: {e}"))?;
            if n == 0 {
                return Err(self
                    .ended()
                    .unwrap_or_else(|| "z3 exited unexpectedly".into()));
            }
            for c in line.chars() {
                match c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
            }
            out.push_str(&line);
            if depth <= 0 && !out.trim().is_empty() {
                return Ok(out.trim().to_string());
            }
        }
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        let _ = self
            .stdin
            .write_all(b"(exit)\n")
            .and_then(|_| self.stdin.flush());
        let _ = self.child.wait();
    }
}

impl Solver {
    /// Starts the pinned Z3. Fails if it is missing or a different version.
    pub fn start() -> Result<Solver, String> {
        if let Err(e) = toolchain::check_z3() {
            STARTS.fetch_add(1, Ordering::Relaxed);
            return Err(e);
        }
        let limit = query_memory_mb();
        Ok(Solver {
            proc: Some(Proc::start(working_memory_mb(limit))?),
            limit,
            query: Vec::new(),
            down: String::new(),
            alone: false,
            canon: Canon::default(),
            timeouts: 0,
        })
    }

    /// Whether this solver may take on another definition (see `REUSE_SECS`).
    /// Every obligation starts with `(reset)` and its full base (ADR-0018),
    /// so a reused process answers as a new one would; where its memory
    /// could make it answer otherwise, a new one is asked (`ask`).
    pub fn reusable(&self) -> bool {
        self.proc
            .as_ref()
            .is_none_or(|p| p.started.elapsed() < Duration::from_secs(REUSE_SECS))
    }

    /// Sends one command of the current query, as the solver receives it,
    /// and reads its answer. A `(reset)` starts a query.
    ///
    /// When the process ends at its memory limit, the query is undecided
    /// only if it was that process's first query and the limit was the
    /// deciding one (`limit`). Otherwise what the process held from earlier
    /// queries, or its smaller limit (`working_memory_mb`), may have ended
    /// it: the query is asked again, from its `(reset)`, alone (the first
    /// and only query of a new process under the deciding limit), and that
    /// answer stands (ADR-0129).
    ///
    /// After a query that ended at the deciding limit the next one is asked
    /// alone at once: the queries of one false obligation tend to end the
    /// same way, and asked alone each pays for its way to the limit once.
    /// Where a query is asked does not change its answer, so this is a
    /// matter of cost only.
    fn ask(&mut self, sent: &str) -> Result<String, String> {
        let reset = sent.trim() == "(reset)";
        if reset {
            self.query.clear();
            // A process that answered a query alone answers no other.
            if self.proc.as_ref().is_some_and(|p| p.alone) {
                self.proc = None;
                self.alone = false;
            }
            if self.proc.is_none() {
                let p = if self.alone {
                    Proc::start(self.limit).map(|mut p| {
                        p.alone = true;
                        p
                    })
                } else {
                    Proc::start(working_memory_mb(self.limit))
                };
                self.proc = Some(p.inspect_err(|e| self.down = e.clone())?);
            }
        }
        self.query.push(sent.to_string());
        let Some(p) = self.proc.as_mut() else {
            // The query has ended (at the memory limit).
            return Err(self.down.clone());
        };
        if reset {
            p.queries += 1;
        }
        let r = p.ask(sent);
        if !r.as_ref().is_err_and(|e| out_of_memory(e)) {
            return r;
        }
        let decided = p.queries == 1 && p.memory == self.limit;
        self.proc = None;
        self.down = OUT_OF_MEMORY.into();
        if decided {
            self.alone = true;
            return r;
        }
        let mut p = Proc::start(self.limit).inspect_err(|e| self.down = e.clone())?;
        p.alone = true;
        let mut answer = Ok(String::new());
        for cmd in &self.query {
            answer = p.ask(cmd);
            if answer.is_err() {
                break;
            }
        }
        match &answer {
            Ok(_) => self.proc = Some(p),
            Err(e) => {
                self.alone = out_of_memory(e);
                self.down = e.clone();
            }
        }
        answer
    }

    fn expect_success(&mut self, sent: &str, cmd: &str) -> Result<(), String> {
        let r = self.ask(sent)?;
        if r == "success" {
            Ok(())
        } else {
            Err(format!(
                "{} (in `{}`)",
                self.canon.back(&r),
                truncate(cmd, 160)
            ))
        }
    }

    /// Runs a sequence of top-level commands that answer `success`.
    pub fn run(&mut self, script: &str) -> Result<(), String> {
        for cmd in split_commands(script) {
            // Each query starts with `(reset)` and numbers its own symbols.
            if cmd.trim() == "(reset)" {
                self.canon = Canon::default();
            }
            let sent = self.canon.forward(&cmd);
            self.expect_success(&sent, &cmd)?;
        }
        Ok(())
    }

    /// Resource units spent since the last `(reset)` (deterministic for a
    /// given solver version and query).
    /// Queries the backstop ended since the last call (see `timeouts`).
    pub fn take_timeouts(&mut self) -> usize {
        std::mem::take(&mut self.timeouts)
    }

    pub fn rlimit_used(&mut self) -> Result<u64, String> {
        let r = self.ask("(get-info :rlimit)")?;
        r.trim_matches(|c| c == '(' || c == ')')
            .split_whitespace()
            .nth(1)
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("unexpected rlimit answer: {r}"))
    }

    pub fn check_sat(&mut self) -> Result<Answer, String> {
        let r = self.ask("(check-sat)")?;
        match r.as_str() {
            "unsat" => Ok(Answer::Unsat),
            "sat" => Ok(Answer::Sat),
            "unknown" => {
                let why = self.ask("(get-info :reason-unknown)")?;
                if reason_text(&why) == "timeout" {
                    self.timeouts += 1;
                }
                Ok(Answer::Unknown(self.canon.back(&reason_text(&why))))
            }
            other => Err(format!("unexpected solver response: {other}")),
        }
    }

    /// Values of the given terms in the current model, as the solver
    /// writes them.
    pub fn get_value_terms(&mut self, terms: &[String]) -> Result<Vec<Sexp>, String> {
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let asked = self
            .canon
            .forward(&format!("(get-value ({}))", terms.join(" ")));
        let raw = self.ask(&asked)?;
        let r = self.canon.back(&raw);
        if r.starts_with("(error") {
            return Err(r);
        }
        let tree = parse(&r).ok_or_else(|| format!("cannot parse model: {r}"))?;
        let Sexp::List(pairs) = tree else {
            return Err(format!("unexpected model: {r}"));
        };
        Ok(pairs
            .into_iter()
            .map(|p| match p {
                Sexp::List(mut kv) if kv.len() == 2 => kv.pop().expect("two"),
                other => other,
            })
            .collect())
    }

    /// Values of the given terms in the current model, rendered for humans.
    pub fn get_values(&mut self, terms: &[String]) -> Result<Vec<String>, String> {
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        // Values are only meaningful right after `sat`.
        let asked = self
            .canon
            .forward(&format!("(get-value ({}))", terms.join(" ")));
        let raw = self.ask(&asked)?;
        let r = self.canon.back(&raw);
        if r.starts_with("(error") {
            return Err(r);
        }
        let tree = parse(&r).ok_or_else(|| format!("cannot parse model: {r}"))?;
        let Sexp::List(pairs) = tree else {
            return Err(format!("unexpected model: {r}"));
        };
        Ok(pairs
            .iter()
            .map(|p| match p {
                Sexp::List(kv) if kv.len() == 2 => render_value(&kv[1]),
                other => render_value(other),
            })
            .collect())
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}..", s.chars().take(n).collect::<String>())
    }
}

/// Splits SMT-LIB text into top-level commands (respecting `|..|` and strings).
pub fn split_commands(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i64;
    let (mut in_bar, mut in_str) = (false, false);
    for c in text.chars() {
        if depth == 0 && c != '(' && !in_bar && !in_str {
            continue;
        }
        cur.push(c);
        match c {
            '|' if !in_str => in_bar = !in_bar,
            '"' if !in_bar => in_str = !in_str,
            '(' if !in_bar && !in_str => depth += 1,
            ')' if !in_bar && !in_str => {
                depth -= 1;
                if depth == 0 {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => {}
        }
    }
    out
}

fn reason_text(r: &str) -> String {
    // `(:reason-unknown "...")`
    match r.split_once('"') {
        Some((_, rest)) => rest.trim_end_matches(')').trim_end_matches('"').to_string(),
        None => r.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sexp {
    Atom(String),
    List(Vec<Sexp>),
}

pub fn parse(s: &str) -> Option<Sexp> {
    let toks = tokenize(s);
    let mut pos = 0;
    let e = parse_at(&toks, &mut pos)?;
    Some(e)
}

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_bar = false;
    for c in s.chars() {
        if in_bar {
            cur.push(c);
            if c == '|' {
                in_bar = false;
            }
            continue;
        }
        match c {
            '(' | ')' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                out.push(c.to_string());
            }
            '|' => {
                cur.push(c);
                in_bar = true;
            }
            c if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn parse_at(toks: &[String], pos: &mut usize) -> Option<Sexp> {
    let t = toks.get(*pos)?;
    *pos += 1;
    if t == "(" {
        let mut items = Vec::new();
        while toks.get(*pos)? != ")" {
            items.push(parse_at(toks, pos)?);
        }
        *pos += 1;
        Some(Sexp::List(items))
    } else if t == ")" {
        None
    } else {
        Some(Sexp::Atom(t.clone()))
    }
}

/// SMT-LIB text of an s-expression (inverse of `parse`).
pub fn print(e: &Sexp) -> String {
    match e {
        Sexp::Atom(a) => a.clone(),
        Sexp::List(xs) => format!("({})", xs.iter().map(print).collect::<Vec<_>>().join(" ")),
    }
}

/// The `f64` a model value of sort `Float64` denotes (ADR-0061 §5):
/// `(fp #b0 #b.. #x..)`, `(_ +zero 11 53)`, `(_ -oo 11 53)`, `(_ NaN 11
/// 53)` (Rust's `f64::NAN` for NaN).
pub fn fp_value(v: &Sexp) -> Option<f64> {
    let bits = |a: &Sexp, w: u32| -> Option<u64> {
        let Sexp::Atom(a) = a else { return None };
        let (digits, radix, n) = match a.strip_prefix("#b") {
            Some(b) => (b, 2, b.len() as u32),
            None => {
                let h = a.strip_prefix("#x")?;
                (h, 16, 4 * h.len() as u32)
            }
        };
        (n == w).then(|| u64::from_str_radix(digits, radix).ok())?
    };
    match v {
        Sexp::List(xs) => match xs.as_slice() {
            [Sexp::Atom(f), s, e, m] if f == "fp" => {
                let b = (bits(s, 1)? << 63) | (bits(e, 11)? << 52) | bits(m, 52)?;
                Some(f64::from_bits(b))
            }
            [Sexp::Atom(u), Sexp::Atom(k), Sexp::Atom(eb), Sexp::Atom(sb)]
                if u == "_" && eb == "11" && sb == "53" =>
            {
                Some(match k.as_str() {
                    "+zero" => 0.0,
                    "-zero" => -0.0,
                    "+oo" => f64::INFINITY,
                    "-oo" => f64::NEG_INFINITY,
                    "NaN" => f64::NAN,
                    _ => return None,
                })
            }
            _ => None,
        },
        Sexp::Atom(_) => None,
    }
}

/// `5` -> `5`, `(- 5)` -> `-5`, `true` -> `true`, a value of type
/// parameter `T` (an element of its sort, ADR-0046 §4) `T#1`, an `f64` in
/// Rust's shortest form (`0.1`, `-0.0`, `NaN`, `inf`); other terms are
/// printed back.
pub fn render_value(v: &Sexp) -> String {
    if let Some(x) = fp_value(v) {
        return if x.is_nan() {
            "NaN".into()
        } else if x.is_infinite() {
            format!("{x}")
        } else {
            crate::lexer::float_text(x.to_bits())
        };
    }
    match v {
        Sexp::Atom(a) => match a
            .strip_prefix("|'")
            .and_then(|r| r.strip_suffix('|'))
            .and_then(|r| r.split_once("!val!"))
        {
            Some((t, k)) => format!("{t}#{k}"),
            None => a.clone(),
        },
        Sexp::List(xs) if xs.len() == 2 && xs[0] == Sexp::Atom("-".into()) => {
            format!("-{}", render_value(&xs[1]))
        }
        Sexp::List(xs) => format!(
            "({})",
            xs.iter().map(render_value).collect::<Vec<_>>().join(" ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_symbols_number_quoted_symbols_by_first_use() {
        // ADR-0063: two queries that differ only in names are the same text.
        let a = "(declare-const |m::x@0| Int)\n(assert (> |m::x@0| |m::f.result@1|))";
        let b = "(declare-const |m::y@0| Int)\n(assert (> |m::y@0| |m::g.result@1|))";
        assert_eq!(canonical(a), canonical(b));
        assert_eq!(
            canonical(a),
            "(declare-const |$0| Int)\n(assert (> |$0| |$1|))"
        );
        // Text between commands (comments) never reaches the solver and is
        // not part of the key; strings keep their bars.
        let c = format!("; |m::x@0| is a local\n{a}\n; done");
        assert_eq!(canonical(&c), canonical(a));
        assert_eq!(canonical("(echo \"|a|\")"), "(echo \"|a|\")");
    }

    #[test]
    fn canonical_answers_are_named_back_as_the_solver_writes_them() {
        let mut c = Canon::default();
        let sent = c.forward("(declare-datatypes ((|Option<i64>| 0)) (((|Option<i64>::None|) (|Option<i64>::Some| (|v| Int)))))\n(declare-sort |T| 0)");
        assert!(sent.contains("|$0|") && sent.contains("|$4|"), "{sent}");
        // The solver prints simple symbols bare, and the values of an
        // uninterpreted sort with `!val!k`.
        assert_eq!(c.back("(($2 7))"), "((|Option<i64>::Some| 7))");
        assert_eq!(c.back("((x $1))"), "((x |Option<i64>::None|))");
        assert_eq!(c.back("$4!val!0"), "T!val!0");
        assert_eq!(c.back("|$4|"), "T");
        // What is not a canonical symbol stays.
        assert_eq!(c.back("$x $99 a$1 |$q|"), "$x $99 a$1 |$q|");
    }

    #[test]
    fn renders_values() {
        let t = parse("((x 5) (y (- 3)) (b true) (v |'T!val!1|))").unwrap();
        let Sexp::List(pairs) = t else { panic!() };
        let vals: Vec<String> = pairs
            .iter()
            .map(|p| match p {
                Sexp::List(kv) => render_value(&kv[1]),
                _ => panic!(),
            })
            .collect();
        assert_eq!(vals, ["5", "-3", "true", "T#1"]);
    }

    #[test]
    fn advisory_queries_run_under_their_own_memory_limit() {
        // Backlog T2-16: inside `advisory` a solver process starts with the
        // advisory limit, outside (and after it, nested or not) with the
        // gate's; the advisory limit is the smaller.
        let pinned = |k: &str| toolchain::pinned(k).parse::<u64>().unwrap();
        assert!(pinned("Z3_ADVISORY_MEMORY_MB") < pinned("Z3_MEMORY_MB"));
        assert!(advisory_memory_mb() <= memory_mb());
        assert_eq!(query_memory_mb(), memory_mb());
        advisory(|| {
            assert_eq!(query_memory_mb(), advisory_memory_mb());
            advisory(|| assert_eq!(query_memory_mb(), advisory_memory_mb()));
            assert_eq!(query_memory_mb(), advisory_memory_mb());
            assert_eq!(
                limit_reason("z3: out of memory"),
                Some(format!("memory limit reached, {} MB", advisory_memory_mb()))
            );
        });
        assert_eq!(query_memory_mb(), memory_mb());
        // A process that answers one query after another runs under half
        // the limit (ADR-0129), and never without one (Z3 reads 0 so).
        assert_eq!(working_memory_mb(memory_mb()), memory_mb() / 2);
        assert!(working_memory_mb(advisory_memory_mb()) < advisory_memory_mb());
        assert_eq!(working_memory_mb(1), 1);
    }

    #[test]
    fn splits_commands() {
        let v = split_commands("(a (b))\n(c |x)y|) \"(\"");
        assert_eq!(v, ["(a (b))", "(c |x)y|)"]);
    }

    #[test]
    fn reason() {
        assert_eq!(reason_text("(:reason-unknown \"canceled\")"), "canceled");
        assert!(unknown_reason("canceled").starts_with("resource limit reached"));
        assert!(unknown_reason("timeout").contains("depends on the machine"));
        assert_eq!(
            unknown_reason("incomplete quantifiers"),
            "incomplete quantifiers"
        );
    }
}
