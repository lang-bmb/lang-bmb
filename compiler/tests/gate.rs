//! The proof gate fails closed (backlog 1B-13). Through the CLI (backlog
//! 4A-04): each case runs `bmb verify` with its own environment.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// `bmb verify` on `file` (relative to the repository root) with `env`:
/// whether it succeeded, and its stdout. With `smt`, the scripts are
/// written there (`--emit-smt`).
fn verify(file: &str, env: &[(&str, &str)], smt: Option<&Path>) -> (bool, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
    cmd.current_dir(root()).arg("verify");
    if let Some(d) = smt {
        cmd.arg("--emit-smt").arg(d);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.arg(file).output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// The scripts `--emit-smt` wrote into `dir`.
fn scripts(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "smt2"))
        .map(|p| fs::read_to_string(p).unwrap())
        .collect()
}

/// A line of `scripts/toolchain.env` (`KEY=value`).
fn pinned(key: &str) -> String {
    fs::read_to_string(root().join("scripts/toolchain.env"))
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{key}=")).map(str::to_string))
        .unwrap()
}

const SEARCH: &str = "tests/check/ok/search.bmb";

#[test]
fn gate_fails_without_solver_or_resources() {
    let tmp = std::env::temp_dir().join(format!("bmb-gate-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    assert!(verify(SEARCH, &[], None).0, "baseline must verify");

    // A starved solver proves nothing: `unknown` is a failure.
    let (ok, out) = verify(SEARCH, &[("BMB_Z3_RLIMIT", "1")], None);
    assert!(!ok);
    assert!(out.contains("E0811"), "{out}");

    // A raised limit is ignored: the pinned value stays in the queries.
    let smt = tmp.join("raised");
    let (ok, out) = verify(SEARCH, &[("BMB_Z3_RLIMIT", "999999999999")], Some(&smt));
    assert!(ok, "{out}");
    let texts = scripts(&smt);
    let limit = format!("(set-option :rlimit {})", pinned("Z3_RLIMIT"));
    // Definitions without obligations send nothing.
    assert!(texts.iter().any(|s| !s.is_empty()));
    assert!(
        texts
            .iter()
            .filter(|s| !s.is_empty())
            .all(|s| s.contains(&limit)),
        "{limit}"
    );
    assert!(!texts.iter().any(|s| s.contains("999999999999")));

    // A query the wall-clock backstop ends, before its rlimit, is marked:
    // that verdict depends on the machine (backlog INF-26). `pff::tri_le`
    // spends the whole rlimit (about a second); 50 ms ends it.
    let fail = "tests/verify/fail/proofs.bmb";
    let line = |out: &str| {
        out.lines()
            .find(|l| l.starts_with("{\"def\":\"pff::tri_le\""))
            .unwrap()
            .to_string()
    };
    assert!(!line(&verify(fail, &[], None).1).contains("backstop"));
    let out = verify(fail, &[("BMB_Z3_BACKSTOP_MS", "50")], None).1;
    assert!(line(&out).contains("\"backstop\":"), "{out}");

    // No solver on PATH: no proof, no success.
    let (ok, out) = verify(SEARCH, &[("PATH", "/nonexistent")], None);
    assert!(!ok);
    assert!(out.contains("E0812"), "{out}");

    // A solver that starts and then dies mid-run (one process now serves many
    // definitions, backlog 1D-18): a solver error, no hang, no success, and a
    // new process for the next definition. The fake records each start.
    let bin = tmp.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let log = tmp.join("starts");
    let fake = bin.join("z3");
    fs::write(
        &fake,
        format!(
            "#!/bin/sh\n\
             if [ \"$1\" = --version ]; then echo 'Z3 version {} - 64 bit'; exit 0; fi\n\
             echo start >> {}\n\
             n=0\n\
             while read -r line; do\n\
             n=$((n+1)); if [ $n -gt 3 ]; then exit 0; fi\n\
             echo success\n\
             done\n",
            pinned("Z3_VERSION"),
            log.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    let (ok, out) = verify(SEARCH, &[("PATH", &path)], None);
    let started = fs::read_to_string(&log).unwrap_or_default().lines().count();
    assert!(!ok);
    assert!(out.contains("solver error"), "{out}");
    // Only definitions without obligations count as verified.
    assert!(
        out.lines()
            .filter(|l| l.contains("\"status\":\"verified\""))
            .all(|l| l.contains("\"obligations\":0,")),
        "{out}"
    );
    let failed_defs = out.matches("\"status\":\"failed\"").count();
    assert!(failed_defs > 1, "{out}");
    assert_eq!(
        started, failed_defs,
        "one start per definition after a crash"
    );
    let _ = fs::remove_dir_all(&tmp);

    // The real solver again: proofs succeed.
    assert!(verify(SEARCH, &[], None).0);
}

/// A query that reaches the solver's memory limit is undecided, and only
/// that query: the process ends there and the next query starts a new one
/// (backlog T2-16). Before, the process grew until the system killed it
/// and every later obligation of the definition counted as failed.
#[test]
fn a_query_at_the_memory_limit_is_undecided_and_the_rest_go_on() {
    let tmp = std::env::temp_dir().join(format!("bmb-gate-mem-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    // The mechanism, with a solver that ends as Z3 does at `-memory` (exit
    // code 101) on its fourth command, every time it is started.
    let bin = tmp.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let log = tmp.join("starts");
    fs::write(
        bin.join("z3"),
        format!(
            "#!/bin/sh\n\
             if [ \"$1\" = --version ]; then echo 'Z3 version {} - 64 bit'; exit 0; fi\n\
             echo \"$@\" >> {}\n\
             n=0\n\
             while read -r line; do\n\
             n=$((n+1)); if [ $n -gt 3 ]; then exit 101; fi\n\
             echo success\n\
             done\n",
            pinned("Z3_VERSION"),
            log.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(bin.join("z3"), fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    let (ok, out) = verify(SEARCH, &[("PATH", &path)], None);
    assert!(!ok);
    let limit = pinned("Z3_MEMORY_MB");
    assert!(
        out.contains(&format!(
            "solver could not decide (memory limit reached, {limit} MB)"
        )),
        "{out}"
    );
    assert!(!out.contains("E0812"), "{out}");
    // The first process serves queries under the working limit (half); the
    // query that ends there is asked again alone under the deciding limit,
    // and so is each query after one that ended there (ADR-0129): each
    // undecided obligation ended a process of its own at the deciding limit.
    let starts = fs::read_to_string(&log).unwrap();
    let limit: u64 = limit.parse().unwrap();
    let (working, deciding) = (format!("-memory:{}", limit / 2), format!("-memory:{limit}"));
    let starts: Vec<&str> = starts.lines().collect();
    assert!(starts[0].contains(&working), "{starts:?}");
    assert!(
        starts
            .iter()
            .all(|l| l.contains(&working) || l.contains(&deciding)),
        "{starts:?}"
    );
    let undecided = out.matches("\"code\":\"E0811\"").count();
    assert!(
        undecided > 1 && starts.iter().filter(|l| l.contains(&deciding)).count() >= undecided,
        "{out}\n{starts:?}"
    );

    // The real solver, with a limit every query reaches: Z3 ends with the
    // code the gate reads as its memory limit, at each obligation anew.
    let (ok, out) = verify(SEARCH, &[("BMB_Z3_MEMORY_MB", "1")], None);
    assert!(!ok);
    assert!(
        out.contains("solver could not decide (memory limit reached, 1 MB)"),
        "{out}"
    );
    assert!(!out.contains("E0812"), "{out}");
    // A raised limit is ignored, as a raised rlimit is.
    assert!(verify(SEARCH, &[("BMB_Z3_MEMORY_MB", "999999")], None).0);
    let _ = fs::remove_dir_all(&tmp);
}

/// The pinned solver's path, and a `PATH` on which `z3` is the script `body`
/// (after a line that hands `--version` to the pinned solver; `$Z3` is its
/// path, `$LOG` a file in `dir` for the script's notes).
fn fake_solver(dir: &Path, shell: &str, body: &str) -> (String, PathBuf) {
    let path = std::env::var("PATH").unwrap();
    let z3 = std::env::split_paths(&path)
        .map(|d| d.join("z3"))
        .find(|p| p.is_file())
        .expect("z3 must be installed");
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let log = dir.join("log");
    fs::write(
        bin.join("z3"),
        format!(
            "#!{shell}\nZ3={}\nLOG={}\nif [ \"$1\" = --version ]; then exec $Z3 \"$@\"; fi\n{body}",
            z3.display(),
            log.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(bin.join("z3"), fs::Permissions::from_mode(0o755)).unwrap();
    (format!("{}:{path}", bin.display()), log)
}

/// A script that hands its input to the pinned solver line by line and ends
/// as Z3 does at `-memory` (exit code 101, no answer) where `cases` (arms
/// of a `case "$line"`) say `exit 101`. `$q` is 1 from a line with a
/// quantifier to the next `(reset)`.
fn ending_solver(cases: &str) -> String {
    format!(
        "set -o pipefail\nq=0\nwhile IFS= read -r line; do\n  case \"$line\" in\n    \"(exit)\") exit 0;;\n    \"(reset)\") q=0;;\n{cases}\n  esac\n  case \"$line\" in *\"(forall \"*) q=1;; esac\n  printf '%s\\n' \"$line\"\ndone | $Z3 \"$@\"\n"
    )
}

/// The probe's `rlimit` option (a hundredth of the pinned limit, ADR-0067).
fn probe_limit() -> String {
    let rlimit: u64 = pinned("Z3_RLIMIT").parse().unwrap();
    format!("(set-option :rlimit {})", rlimit / 100)
}

/// A process that has answered other queries, or runs under the working
/// limit, does not decide that a query is out of memory (ADR-0129, backlog
/// T2-17 a): Z3 counts what its process allocated, and a process keeps a
/// little of every query, so the same query fit or did not fit depending on
/// what was asked before it. Here every process under the working limit
/// ends at its first query; each query is asked again alone under the
/// deciding limit, and the output is byte for byte that of a run where no
/// process ended.
#[test]
fn a_memory_end_in_a_working_process_is_asked_again_alone() {
    let tmp = std::env::temp_dir().join(format!("bmb-gate-alone-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let limit: u64 = pinned("Z3_MEMORY_MB").parse().unwrap();
    let (path, log) = fake_solver(
        &tmp,
        "/bin/sh",
        &format!(
            "echo \"$@\" >> $LOG\n\
             case \" $* \" in *\" -memory:{} \"*) exec $Z3 -in -smt2 -memory:1;; esac\n\
             exec $Z3 \"$@\"\n",
            limit / 2
        ),
    );
    let (plain_ok, plain) = verify(SEARCH, &[], None);
    let (ok, out) = verify(SEARCH, &[("PATH", &path)], None);
    assert!(plain_ok && ok, "{out}");
    assert_eq!(out, plain);
    // Each query: a working process that ended, then a process of its own.
    let starts = fs::read_to_string(&log).unwrap();
    let starts: Vec<&str> = starts.lines().collect();
    let count = |limit: u64| {
        starts
            .iter()
            .filter(|l| l.contains(&format!("-memory:{limit}")))
            .count()
    };
    assert!(count(limit) > 10, "{starts:?}");
    assert_eq!(count(limit / 2), count(limit), "{starts:?}");
    let _ = fs::remove_dir_all(&tmp);
}

/// A probe that ends at the memory limit leaves its query to be asked
/// (backlog T2-17 b): the full query went to the process that had ended and
/// was reported undecided for memory without having been asked. Here every
/// query under the probe's `rlimit` ends its process; the full queries
/// prove every obligation.
#[test]
fn after_a_probe_at_the_memory_limit_the_query_is_asked() {
    let tmp = std::env::temp_dir().join(format!("bmb-gate-probe-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let (path, log) = fake_solver(
        &tmp,
        "/bin/bash",
        &ending_solver(&format!(
            "    \"{}\") echo probe >> $LOG; exit 101;;",
            probe_limit()
        )),
    );
    let (ok, out) = verify(SEARCH, &[("PATH", &path)], None);
    assert!(ok, "{out}");
    assert!(!out.contains("E0811") && !out.contains("E0812"), "{out}");
    assert!(fs::read_to_string(&log).unwrap().lines().count() > 2);
    let _ = fs::remove_dir_all(&tmp);
}

/// A query that ends at the memory limit gets the ground-instance retry
/// that a query the `rlimit` ends gets (backlog T2-17 c). Here every query
/// under the probe's `rlimit` and every query with a quantifier ends its
/// process; the retry, which has the instances of the recursive definition
/// in place of its quantified axiom, proves the obligation.
#[test]
fn a_query_at_the_memory_limit_gets_the_ground_instance_retry() {
    let tmp = std::env::temp_dir().join(format!("bmb-gate-retry-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("tri.bmb");
    fs::write(
        &file,
        r#"#[summary("삼각수")]
mod series {
    spec fn tri_spec(n: u64) -> u64
        decreases n,
    {
        if n == 0 { 0 } else { n + tri_spec(n - 1) }
    }

    #[effects(pure)]
    #[uses(tri_spec)]
    fn three() -> (r: u64)
        ensures r == tri_spec(2),
    {
        3
    }
}
"#,
    )
    .unwrap();
    let (path, log) = fake_solver(
        &tmp,
        "/bin/bash",
        &ending_solver(&format!(
            "    \"{}\") exit 101;;\n    \"(check-sat)\") if [ $q = 1 ]; then echo full >> $LOG; exit 101; fi;;",
            probe_limit()
        )),
    );
    let smt = tmp.join("smt");
    let (ok, out) = verify(file.to_str().unwrap(), &[("PATH", &path)], Some(&smt));
    assert!(ok, "{out}");
    assert!(!out.contains("E0811"), "{out}");
    // The full query was asked and ended (under the working limit, then
    // alone); the script shows the retry that proved it.
    assert!(fs::read_to_string(&log).unwrap().lines().count() >= 1);
    assert!(
        scripts(&smt)
            .iter()
            .any(|s| s.contains("; retry of obligation 0 with ground instances")),
    );
    let _ = fs::remove_dir_all(&tmp);
}
