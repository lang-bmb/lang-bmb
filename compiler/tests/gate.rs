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
            .all(|s| s.contains(&format!("(set-option :rlimit {})", bmb::smt::rlimit())))
    );
    assert!(!out.scripts.values().any(|s| s.contains("999999999999")));
    unsafe { std::env::remove_var("BMB_Z3_RLIMIT") };

    // No solver on PATH: no proof, no success.
    let path = std::env::var("PATH").unwrap();
    unsafe { std::env::set_var("PATH", "/nonexistent") };
    let out = bmb::verify_sources(&files);
    unsafe { std::env::set_var("PATH", path) };
    assert!(!out.ok);
    assert!(out.output.contains("E0812"), "{}", out.output);
}
