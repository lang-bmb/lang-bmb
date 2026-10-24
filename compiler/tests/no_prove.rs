//! Requests that do not report proof results do not run the solver (backlog
//! 1D-17), and answer the same. This file counts solver starts and mutates the
//! process environment, so it holds a single test and runs as its own process.

use std::path::Path;

fn req(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn only_proof_status_requests_start_the_solver() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/run");
    let files: Vec<(String, String)> = ["search.bmb"]
        .iter()
        .map(|n| (n.to_string(), std::fs::read_to_string(dir.join(n)).unwrap()))
        .collect();
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
        req(&["map", "L9"]),
    ];
    let with_proof = [
        req(&["map", "L1"]),
        req(&["map", "all"]),
        req(&["query", "surface", "app::main"]),
        req(&["query", "status"]),
    ];

    // Reference answers with the solver available.
    let reference: Vec<_> = without_proof
        .iter()
        .map(|r| bmb::map_request(&files, r))
        .collect();
    for (r, out) in without_proof.iter().zip(&reference) {
        if !matches!(r[1].as_str(), "nosuch" | "L9") {
            assert!(out.is_ok(), "{r:?}: {out:?}");
        }
    }

    // The unproved L0 and L2 maps are exactly the rows of the proved full map.
    let all = bmb::map_request(&files, &req(&["map", "all"])).unwrap();
    for (level, out) in [("L0", &reference[0]), ("L2", &reference[1])] {
        let rows: String = all
            .lines()
            .filter(|l| {
                l.starts_with("{\"schema\"") || l.contains(&format!("\"level\":\"{level}\""))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(out.as_ref().unwrap(), &rows, "{level}");
    }

    // No solver on PATH: the same answers, and no start was attempted.
    let path = std::env::var("PATH").unwrap();
    unsafe { std::env::set_var("PATH", "/nonexistent") };
    let before = bmb::smt::starts();
    for (r, expected) in without_proof.iter().zip(&reference) {
        assert_eq!(&bmb::map_request(&files, r), expected, "{r:?}");
    }
    assert_eq!(
        bmb::smt::starts(),
        before,
        "a request without proof status started the solver"
    );

    // Requests that report proof status do try to start it (the counter works),
    // and without a solver they report failure rather than success.
    for r in &with_proof {
        let before = bmb::smt::starts();
        let out = bmb::map_request(&files, r).unwrap();
        assert!(
            bmb::smt::starts() > before,
            "{r:?} did not start the solver"
        );
        assert!(!out.contains("\"status\":\"verified\""), "{r:?}: {out}");
    }
    unsafe { std::env::set_var("PATH", path) };
}
