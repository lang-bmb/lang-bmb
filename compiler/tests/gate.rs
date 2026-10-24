//! The proof gate fails closed (backlog 1B-13). This file mutates the process
//! environment, so it holds a single test and runs as its own process.

use std::path::Path;

#[test]
fn gate_fails_without_solver_or_resources() {
    let src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/check/ok/search.bmb"),
    )
    .unwrap();
    let files = [("search.bmb".to_string(), src)];
    assert!(bmb::verify_sources(&files).ok, "baseline must verify");

    // A starved solver proves nothing: `unknown` is a failure.
    unsafe { std::env::set_var("BMB_Z3_RLIMIT", "1") };
    let out = bmb::verify_sources(&files);
    assert!(!out.ok);
    assert!(out.output.contains("E0811"), "{}", out.output);
    unsafe { std::env::remove_var("BMB_Z3_RLIMIT") };

    // A raised limit is ignored: the pinned value stays in the queries.
    unsafe { std::env::set_var("BMB_Z3_RLIMIT", "999999999999") };
    let out = bmb::verify_sources(&files);
    assert!(out.ok);
    assert!(
        out.scripts
            .values()
            .filter(|s| !s.is_empty()) // definitions without obligations send nothing
            .all(|s| s.contains(&format!("(set-option :rlimit {})", bmb::smt::rlimit())))
    );
    assert!(!out.scripts.values().any(|s| s.contains("999999999999")));
    unsafe { std::env::remove_var("BMB_Z3_RLIMIT") };

    // No solver on PATH: no proof, no success.
    let path = std::env::var("PATH").unwrap();
    unsafe { std::env::set_var("PATH", "/nonexistent") };
    let out = bmb::verify_sources(&files);
    unsafe { std::env::set_var("PATH", &path) };
    assert!(!out.ok);
    assert!(out.output.contains("E0812"), "{}", out.output);

    // A solver that starts and then dies mid-run (one process now serves many
    // definitions, backlog 1D-18): a solver error, no hang, no success, and a
    // new process for the next definition.
    let dir = std::env::temp_dir().join(format!("bmb-gate-crash-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fake = dir.join("z3");
    std::fs::write(
        &fake,
        format!(
            "#!/bin/sh\n\
             if [ \"$1\" = --version ]; then echo 'Z3 version {} - 64 bit'; exit 0; fi\n\
             n=0\n\
             while read -r line; do\n\
             n=$((n+1)); if [ $n -gt 3 ]; then exit 0; fi\n\
             echo success\n\
             done\n",
            bmb::toolchain::z3_version()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    unsafe { std::env::set_var("PATH", format!("{}:{path}", dir.display())) };
    let starts = bmb::smt::starts();
    let out = bmb::verify_sources(&files);
    let started = bmb::smt::starts() - starts;
    unsafe { std::env::set_var("PATH", &path) };
    let _ = std::fs::remove_dir_all(&dir);
    assert!(!out.ok);
    assert!(out.output.contains("solver error"), "{}", out.output);
    // Only definitions without obligations count as verified.
    assert!(
        out.output
            .lines()
            .filter(|l| l.contains("\"status\":\"verified\""))
            .all(|l| l.contains("\"obligations\":0,")),
        "{}",
        out.output
    );
    let failed_defs = out.output.matches("\"status\":\"failed\"").count();
    assert!(failed_defs > 1, "{}", out.output);
    assert_eq!(
        started, failed_defs,
        "one start per definition after a crash"
    );

    // The real solver again: proofs succeed.
    assert!(bmb::verify_sources(&files).ok);
}
