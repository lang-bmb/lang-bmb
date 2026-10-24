//! Z3 process connection (ADR-0017 §5, backlog 1B-09).
//!
//! The solver runs as a child process speaking SMT-LIB on stdin/stdout. Every
//! answer other than `unsat` is a failure for the proof gate.

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
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    started: Instant,
}

/// Wall-clock backstop in seconds. Results are governed by the deterministic
/// `rlimit`; this only stops a runaway process.
const BACKSTOP_SECS: u64 = 300;

/// A process may be reused for a new definition only this long after it
/// started (backlog 1D-18). Its `-T` limit covers this window plus the full
/// backstop, so every definition still gets at least `BACKSTOP_SECS`, as it
/// did when each definition started its own process.
const REUSE_SECS: u64 = 60;

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

impl Solver {
    /// Starts the pinned Z3. Fails if it is missing or a different version.
    pub fn start() -> Result<Solver, String> {
        STARTS.fetch_add(1, Ordering::Relaxed);
        toolchain::check_z3()?;
        let mut child = Command::new("z3")
            .args([
                "-in",
                "-smt2",
                &format!("-T:{}", BACKSTOP_SECS + REUSE_SECS),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start z3: {e}"))?;
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut s = Solver {
            child,
            stdin,
            stdout,
            started: Instant::now(),
        };
        // Every command answers, so an error is seen at the command that caused it.
        s.send("(set-option :print-success true)")?;
        s.expect_success("(set-option :print-success true)")?;
        Ok(s)
    }

    /// Whether this process may take on another definition (see `REUSE_SECS`).
    /// Every obligation starts with `(reset)` and its full base (ADR-0018),
    /// so a reused process answers as a new one would.
    pub fn reusable(&self) -> bool {
        self.started.elapsed() < Duration::from_secs(REUSE_SECS)
    }

    fn expect_success(&mut self, cmd: &str) -> Result<(), String> {
        let r = self.read_response()?;
        if r == "success" {
            Ok(())
        } else {
            Err(format!("{r} (in `{}`)", truncate(cmd, 160)))
        }
    }

    /// Runs a sequence of top-level commands that answer `success`.
    pub fn run(&mut self, script: &str) -> Result<(), String> {
        for cmd in split_commands(script) {
            self.send(&cmd)?;
            self.expect_success(&cmd)?;
        }
        Ok(())
    }

    fn send(&mut self, text: &str) -> Result<(), String> {
        self.stdin
            .write_all(text.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|e| format!("z3 input closed: {e}"))
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
                return Err("z3 exited unexpectedly".into());
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

    /// Resource units spent since the last `(reset)` (deterministic for a
    /// given solver version and query).
    pub fn rlimit_used(&mut self) -> Result<u64, String> {
        self.send("(get-info :rlimit)")?;
        let r = self.read_response()?;
        r.trim_matches(|c| c == '(' || c == ')')
            .split_whitespace()
            .nth(1)
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("unexpected rlimit answer: {r}"))
    }

    pub fn check_sat(&mut self) -> Result<Answer, String> {
        self.send("(check-sat)")?;
        let r = self.read_response()?;
        match r.as_str() {
            "unsat" => Ok(Answer::Unsat),
            "sat" => Ok(Answer::Sat),
            "unknown" => {
                self.send("(get-info :reason-unknown)")?;
                let why = self.read_response()?;
                Ok(Answer::Unknown(reason_text(&why)))
            }
            other => Err(format!("unexpected solver response: {other}")),
        }
    }

    /// Values of the given terms in the current model, rendered for humans.
    pub fn get_values(&mut self, terms: &[String]) -> Result<Vec<String>, String> {
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        // Values are only meaningful right after `sat`.
        self.send(&format!("(get-value ({}))", terms.join(" ")))?;
        let r = self.read_response()?;
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

impl Drop for Solver {
    fn drop(&mut self) {
        let _ = self.send("(exit)");
        let _ = self.child.wait();
    }
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

/// `5` -> `5`, `(- 5)` -> `-5`, `true` -> `true`; other terms are printed back.
pub fn render_value(v: &Sexp) -> String {
    match v {
        Sexp::Atom(a) => a.clone(),
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
    fn renders_values() {
        let t = parse("((x 5) (y (- 3)) (b true))").unwrap();
        let Sexp::List(pairs) = t else { panic!() };
        let vals: Vec<String> = pairs
            .iter()
            .map(|p| match p {
                Sexp::List(kv) => render_value(&kv[1]),
                _ => panic!(),
            })
            .collect();
        assert_eq!(vals, ["5", "-3", "true"]);
    }

    #[test]
    fn splits_commands() {
        let v = split_commands("(a (b))\n(c |x)y|) \"(\"");
        assert_eq!(v, ["(a (b))", "(c |x)y|)"]);
    }

    #[test]
    fn reason() {
        assert_eq!(reason_text("(:reason-unknown \"canceled\")"), "canceled");
    }
}
