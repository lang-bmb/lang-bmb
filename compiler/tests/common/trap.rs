//! Trap programs for the soundness fuzzers (backlog 1B-51, from 2N-04).
//!
//! A fuzzer that only compares the output of calls with a value cannot see a
//! check that a wrong proof erased, or one that code generation forgot,
//! inside a function that was not proved: the interpreter says the call
//! panics, so the call is never run. Here such a call runs as a program of
//! its own (a trap ends the run): `main` prints `1`, makes the call, prints
//! `2`. Both builds, proved checks removed and every check kept, must print
//! `1` and end by `SIGILL` (`llvm.trap` after flushing stdout, backlog
//! 1B-52), as Rust prints the lines before a panic.
//!
//! The program shares its definitions with the fuzzer's batch program, so
//! the first build proves through the proof cache the batch filled and
//! nothing is proved again.

use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Command;

/// Trapping calls per batch (each one is two builds and two runs).
pub const TRAPS: usize = 3;

/// `SIGILL`: what `llvm.trap` ends a process with on x86-64.
const SIGILL: i32 = 4;

/// Chooses up to `TRAPS` trapping calls, one per function, preferring a
/// different kind of fault each time (`kind` of a call).
pub fn choose<T: Clone, K: PartialEq>(
    calls: &[(usize, T)],
    kind: impl Fn(&T) -> K,
) -> Vec<(usize, T)> {
    let mut chosen: Vec<(usize, T)> = Vec::new();
    for (i, t) in calls {
        if chosen.len() < TRAPS && !chosen.iter().any(|(j, c)| *j == *i || kind(c) == kind(t)) {
            chosen.push((*i, t.clone()));
        }
    }
    for (i, t) in calls {
        if chosen.len() < TRAPS && !chosen.iter().any(|(j, _)| *j == *i) {
            chosen.push((*i, t.clone()));
        }
    }
    chosen
}

/// `main`'s body for a trapping call: `1`, the call, `2`.
pub fn body(call: &str) -> String {
    format!("        print_i64(1);\n{call}        print_i64(2);\n")
}

/// Builds `files` twice and runs each build. Returns `None` when both
/// printed `1` and ended by `SIGILL`, or what went wrong.
pub fn check(files: &[(String, String)], cache: &Path, exe: &Path) -> Option<String> {
    for all_checks in [false, true] {
        let out = if all_checks {
            bmb::emit_llvm_sources_with(files, true)
        } else {
            bmb::emit_llvm_sources_cached(files, Some(cache))
        };
        let Some(ir) = out.ir else {
            return Some(format!(
                "no IR for the trap (all_checks = {all_checks})\n{}",
                out.verify.output
            ));
        };
        if let Err(e) = bmb::toolchain::build_executable(&ir, exe.to_str().unwrap()) {
            return Some(e);
        }
        let run = Command::new(exe).output().unwrap();
        let _ = fs::remove_file(exe);
        let printed = String::from_utf8_lossy(&run.stdout);
        if printed != "1\n" || run.status.signal() != Some(SIGILL) {
            return Some(format!(
                "the call panics in Rust, but the build (all_checks = {all_checks}) printed {printed:?} and ended with {}",
                run.status
            ));
        }
    }
    None
}
