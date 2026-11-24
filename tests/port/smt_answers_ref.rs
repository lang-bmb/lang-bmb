//! Differential test: sends scripts through stage0's solver connection
//! (`compiler/src/smt.rs`: the Z3 process, `Solver`, the limits; and
//! `compiler/src/toolchain.rs`: the version check) and prints everything
//! the connection shows of them. `smt_answers.bmb` does the same job with
//! the port's connection (`selfhost/solver.bmb`, `selfhost/toolchain.bmb`);
//! the cases `smt-answers*` compare the two outputs, and with the stand-in
//! solver of `tests/port/answers/fake/` also every byte each side sent.
//!
//! Each argument is a file of directives, each one parenthesised form:
//!
//! | Directive | What runs | What is printed |
//! | --- | --- | --- |
//! | `(!start gate)`, `refutations`, `models` | `Solver::start` for that asker, after the solver before it is dropped | `ok`, or the error |
//! | `(!run COMMANDS)` | `Solver::run` | `ok`, or the error |
//! | `(!ask COMMAND)` | `Solver::ask` (one command as sent, no canonical symbols) | `answer:` and the response, or the error |
//! | `(!check)` | `check_sat` | `unsat`, `sat`, or `unknown:` with the solver's reason and `reason:` with `unknown_reason` of it |
//! | `(!rlimit)` | `rlimit_used` | `rlimit:` and the number |
//! | `(!values TERM..)` | `get_values` | one `value:` per term |
//! | `(!terms TERM..)` | `get_value_terms` | one `term:` per term |
//! | `(!timeouts)` | `take_timeouts` | `timeouts:` and the count |
//! | `(!reusable)` | `reusable` | `reusable:` and the answer |
//! | `(!age MS)` | `age`: the process is that much older | nothing |
//! | `(!close)` | `close` | `closed:` and the exit status (128 + N for signal N), or `none` |
//! | `(!z3)` | `toolchain::check_z3` | `z3:` and the version, or the error |
//! | `(!limits)` | the limits of the asker | one line |
//!
//! An error is followed by `limit:` and `limit_reason` of it when it is a
//! limit, and any directive by `starts:` and the number of solver starts
//! so far when it started one. The clock is never printed: `(!age MS)`
//! moves a process past the reuse window in its place.
//!
//! An example of the `bmb` crate (`compiler/Cargo.toml`), so that it calls
//! the compiler's own functions.

use bmb::smt::{self, Advisory, Answer, Solver};
use bmb::toolchain;
use std::os::unix::process::ExitStatusExt;

struct State {
    solver: Option<Solver>,
    who: Option<Advisory>,
    starts: usize,
    out: String,
}

impl State {
    fn line(&mut self, a: &str, b: &str) {
        self.out.push_str(a);
        self.out.push_str(b);
        self.out.push('\n');
    }

    fn error(&mut self, e: &str) {
        self.line("error: ", e);
        if let Some(l) = smt::limit_reason(e) {
            self.line("limit: ", &l);
        }
    }

    fn directive(&mut self, word: &str, rest: &str) {
        match word {
            "start" => {
                self.solver = None;
                self.who = match rest {
                    "refutations" => Some(Advisory::Refutations),
                    "models" => Some(Advisory::Models),
                    _ => None,
                };
                return;
            }
            "z3" => {
                match toolchain::check_z3() {
                    Ok(v) => self.line("z3: ", &v),
                    Err(e) => self.error(&e),
                }
                return;
            }
            "limits" => {
                let l = smt::query_memory_mb();
                let text = format!(
                    "rlimit {} advisory {} query {} memory {} advisory {} query {} working {} backstop {}",
                    smt::rlimit(),
                    smt::advisory_rlimit(),
                    smt::query_rlimit(),
                    smt::memory_mb(),
                    smt::advisory_memory_mb(),
                    l,
                    smt::working_memory_mb(l),
                    smt::backstop_ms()
                );
                self.line("limits: ", &text);
                return;
            }
            _ => {}
        }
        let Some(s) = self.solver.as_mut() else {
            self.line("no solver", "");
            return;
        };
        match word {
            "run" => match s.run(rest) {
                Ok(()) => self.line("ok", ""),
                Err(e) => self.error(&e),
            },
            "ask" => match s.ask(rest) {
                Ok(r) => self.line("answer: ", &r),
                Err(e) => self.error(&e),
            },
            "check" => match s.check_sat() {
                Ok(Answer::Unsat) => self.line("unsat", ""),
                Ok(Answer::Sat) => self.line("sat", ""),
                Ok(Answer::Unknown(why)) => {
                    self.line("unknown: ", &why);
                    self.line("reason: ", &smt::unknown_reason(&why));
                }
                Err(e) => self.error(&e),
            },
            "rlimit" => match s.rlimit_used() {
                Ok(n) => self.line("rlimit: ", &n.to_string()),
                Err(e) => self.error(&e),
            },
            "values" => match s.get_values(&terms(rest)) {
                Ok(vs) => vs.iter().for_each(|v| self.line("value: ", v)),
                Err(e) => self.error(&e),
            },
            "terms" => match s.get_value_terms(&terms(rest)) {
                Ok(vs) => vs.iter().for_each(|v| self.line("term: ", &smt::print(v))),
                Err(e) => self.error(&e),
            },
            "timeouts" => {
                let n = s.take_timeouts();
                self.line("timeouts: ", &n.to_string());
            }
            "reusable" => {
                let r = s.reusable();
                self.line("reusable: ", if r { "true" } else { "false" });
            }
            "age" => s.age(rest.parse().unwrap_or(0)),
            "close" => {
                let status = s.close();
                self.closed(status);
            }
            _ => self.line("unknown directive", ""),
        }
    }

    fn closed(&mut self, status: Option<std::process::ExitStatus>) {
        let code = status.and_then(|s| s.code().or_else(|| s.signal().map(|n| 128 + n)));
        match code {
            Some(c) => self.line("closed: ", &c.to_string()),
            None => self.line("closed: none", ""),
        }
    }
}

/// The terms of `rest`, each as text.
fn terms(rest: &str) -> Vec<String> {
    match smt::parse(&format!("({rest})")) {
        Some(smt::Sexp::List(xs)) => xs.iter().map(smt::print).collect(),
        _ => Vec::new(),
    }
}

fn main() {
    let mut st = State {
        solver: None,
        who: None,
        starts: 0,
        out: String::new(),
    };
    for path in std::env::args().skip(1) {
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => {
                eprintln!("cannot read {path}");
                std::process::exit(2);
            }
        };
        let name = path.rsplit('/').next().unwrap_or("");
        st.line("== ", name);
        for d in smt::split_commands(&text) {
            st.line("> ", &d);
            let body = d.strip_prefix("(!").unwrap_or("");
            let body = body.strip_suffix(')').unwrap_or(body);
            let n = body
                .find(|c: char| c.is_ascii_whitespace())
                .unwrap_or(body.len());
            let (word, rest) = (&body[..n], body[n..].trim_ascii());
            let who = if word == "start" { None } else { st.who };
            match who {
                None => st.directive(word, rest),
                Some(a) => smt::advisory(a, || st.directive(word, rest)),
            }
            if word == "start" {
                let started = match st.who {
                    None => Solver::start(),
                    Some(a) => smt::advisory(a, Solver::start),
                };
                match started {
                    Ok(s) => {
                        st.solver = Some(s);
                        st.line("ok", "");
                    }
                    Err(e) => match st.who {
                        None => st.error(&e),
                        Some(a) => smt::advisory(a, || st.error(&e)),
                    },
                }
            }
            if smt::starts() != st.starts {
                st.starts = smt::starts();
                st.line("starts: ", &st.starts.to_string());
            }
            print!("{}", st.out);
            st.out.clear();
        }
        if let Some(s) = st.solver.as_mut() {
            let status = s.close();
            st.out.push_str("end ");
            st.closed(status);
        }
        st.solver = None;
        st.who = None;
        print!("{}", st.out);
        st.out.clear();
    }
}
