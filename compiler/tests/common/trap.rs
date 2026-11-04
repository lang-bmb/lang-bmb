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

/// Builds `files` twice through the CLI (`bmb emit-llvm`, then clang as
/// `bmb build` does) and runs each build: the first proves through the proof
/// cache in `cache`, the second keeps every check (`--all-checks`). Returns
/// `None` when both printed `1` and ended by `SIGILL`, or what went wrong.
/// The CLI, so the fuzzers that use it test whichever compiler is built
/// (backlog 4A-04).
pub fn check(files: &[(String, String)], cache: &Path, exe: &Path) -> Option<String> {
    let dir = exe.with_extension("src");
    fs::create_dir_all(&dir).unwrap();
    let paths: Vec<_> = files
        .iter()
        .map(|(name, text)| {
            let p = dir.join(name);
            fs::write(&p, text).unwrap();
            p
        })
        .collect();
    let result = (|| {
        for all_checks in [false, true] {
            let ll = exe.with_extension("ll");
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
            cmd.arg("emit-llvm");
            if all_checks {
                cmd.arg("--all-checks");
            } else {
                cmd.arg("--cache").arg(cache);
            }
            let out = cmd.args(&paths).arg("-o").arg(&ll).output().unwrap();
            let Ok(ir) = fs::read_to_string(&ll) else {
                return Some(format!(
                    "no IR for the trap (all_checks = {all_checks})\n{}",
                    String::from_utf8_lossy(&out.stdout)
                ));
            };
            let _ = fs::remove_file(&ll);
            let build = exe.with_extension("build.ll");
            fs::write(&build, ir).unwrap();
            let cc = Command::new("clang")
                .args(["-O2", "-Wno-override-module", "-o"])
                .arg(exe)
                .arg(&build)
                .output()
                .unwrap();
            let _ = fs::remove_file(&build);
            if !cc.status.success() {
                return Some(String::from_utf8_lossy(&cc.stderr).into_owned());
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
    })();
    let _ = fs::remove_dir_all(&dir);
    result
}
