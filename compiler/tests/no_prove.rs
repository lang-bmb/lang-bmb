//! Requests that do not report proof results do not run the solver (backlog
//! 1D-17), and answer the same. Through the CLI (backlog 4A-04): with a fake
//! `z3` first on `PATH` that records every start, a request without proof
//! status leaves the record empty and answers as with the real solver.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn req(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

/// The CLI form of a `map`/`query` request on `search.bmb`.
fn args(r: &[String]) -> Vec<String> {
    let mut a = r.to_vec();
    if r[0] == "query" {
        a.push("--".into());
    }
    a.push("search.bmb".into());
    a
}

/// Exit status, stdout and stderr of `bmb` on request `r`, with `path` as
/// `PATH` when given.
fn run(r: &[String], path: Option<&Path>) -> (Option<i32>, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
    cmd.args(args(r))
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/run"));
    if let Some(p) = path {
        cmd.env("PATH", p);
    }
    let out = cmd.output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn only_proof_status_requests_start_the_solver() {
    let without_proof = [
        req(&["map", "L0"]),
        req(&["map", "L2"]),
        req(&["query", "units"]),
        req(&["query", "edges", "app::main", "both", "2"]),
        req(&["query", "find", "main"]),
        req(&["query", "slice", "app::main"]),
        req(&["query", "contracts", "alg::is_sorted"]),
        req(&["query", "impact", "app::main", "body"]),
        req(&["query", "effects", "app::main"]),
        req(&["query", "effects"]),
        req(&["query", "nosuch"]),
    ];
    let with_proof = [
        req(&["map", "L1"]),
        req(&["map", "all"]),
        req(&["query", "surface", "app::main"]),
        req(&["query", "status"]),
        req(&["query", "tests"]),
    ];

    // Reference answers with the solver available.
    let reference: Vec<_> = without_proof.iter().map(|r| run(r, None)).collect();
    for (r, out) in without_proof.iter().zip(&reference) {
        if r[1] != "nosuch" {
            assert_eq!(out.0, Some(0), "{r:?}: {out:?}");
        }
    }

    // The unproved L0 and L2 maps are exactly the rows of the proved full map.
    let all = run(&req(&["map", "all"]), None).1;
    for (level, out) in [("L0", &reference[0]), ("L2", &reference[1])] {
        let rows: String = all
            .lines()
            .filter(|l| {
                l.starts_with("{\"schema\"") || l.contains(&format!("\"level\":\"{level}\""))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(out.1, rows, "{level}");
    }

    // A `z3` that records each start and is not a solver.
    let dir: PathBuf = std::env::temp_dir().join(format!("bmb-no-prove-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let log = dir.join("starts");
    let fake = dir.join("z3");
    fs::write(
        &fake,
        format!("#!/bin/sh\necho start >> {}\nexit 1\n", log.display()),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = std::env::join_paths(
        std::iter::once(dir.clone()).chain(std::env::split_paths(&std::env::var("PATH").unwrap())),
    )
    .unwrap();
    let path = PathBuf::from(path);
    let starts = || fs::read_to_string(&log).unwrap_or_default().lines().count();

    // The same answers, and no start was attempted.
    for (r, expected) in without_proof.iter().zip(&reference) {
        assert_eq!(&run(r, Some(&path)), expected, "{r:?}");
    }
    assert_eq!(
        starts(),
        0,
        "a request without proof status started the solver"
    );

    // Requests that report proof status do try to start it (the record works),
    // and without a solver they report failure rather than success.
    for r in &with_proof {
        let before = starts();
        let out = run(r, Some(&path));
        assert!(starts() > before, "{r:?} did not start the solver");
        assert!(
            !out.1.contains("\"status\":\"verified\""),
            "{r:?}: {}",
            out.1
        );
    }
    let _ = fs::remove_dir_all(&dir);
}
