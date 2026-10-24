//! Proof cache (backlog 2D-01): the same output with or without it, warm runs
//! skip the solver for proved obligations, and a change to anything a proof
//! read invalidates that proof.

use std::fs;
use std::path::{Path, PathBuf};

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("bmb-cache-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

fn files(text: &str) -> Vec<(String, String)> {
    vec![("c.bmb".to_string(), text.to_string())]
}

#[test]
fn cached_output_is_identical_and_warm_runs_skip_proved_obligations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let dir = tmp("corpus");
    for sub in ["verify/pass", "verify/fail", "run"] {
        let mut paths: Vec<PathBuf> = fs::read_dir(root.join(sub))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
            .collect();
        paths.sort();
        for p in paths {
            let f = files(&fs::read_to_string(&p).unwrap());
            let plain = bmb::verify_sources(&f);
            let cold = bmb::verify_sources_cached(&f, Some(&dir));
            let warm = bmb::verify_sources_cached(&f, Some(&dir));
            for (name, run) in [("cold", &cold), ("warm", &warm)] {
                assert_eq!(plain.output, run.output, "{} ({name})", p.display());
                assert_eq!(plain.ok, run.ok, "{} ({name})", p.display());
            }
            // Warm: only obligations that were not proved go to the solver.
            let failed: usize = plain
                .output
                .lines()
                .filter(|l| l.contains("\"def\""))
                .map(|l| {
                    let i = l.find("\"failed\":").unwrap() + 9;
                    l[i..]
                        .split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap()
                        .parse::<usize>()
                        .unwrap()
                })
                .sum();
            assert_eq!(cold.cache.0 + cold.cache.1, warm.cache.0 + warm.cache.1);
            assert!(
                warm.cache.1 <= failed,
                "{}: {:?} misses, {failed} failed",
                p.display(),
                warm.cache
            );
        }
    }
    let _ = fs::remove_dir_all(&dir);
}

const BASE: &str = r#"
#[summary("u")]
mod u {
    spec fn small(x: i64) -> bool { x < 100 }

    #[effects(pure)]
    #[uses(small)]
    fn inc(x: i64) -> i64
        requires small(x),
    {
        x + 1
    }

    spec fn big(r: i64) -> bool { r > 0 }

    #[effects(pure)]
    #[uses(big)]
    fn one() -> (r: i64)
        ensures big(r),
    {
        1
    }

    #[effects(pure)]
    fn user() -> i64 {
        inc(one())
    }
}
"#;

#[test]
fn a_change_a_proof_read_invalidates_it() {
    let dir = tmp("invalidate");
    let first = bmb::verify_sources_cached(&files(BASE), Some(&dir));
    assert!(
        !first.ok,
        "user cannot see small(one()): `small` is opaque to it"
    );
    // Reveal `small` and give `one` a stronger contract: `user` is proved.
    let strong = BASE
        .replace(
            "spec fn big(r: i64) -> bool { r > 0 }",
            "spec fn big(r: i64) -> bool { r > 0 && r < 10 }",
        )
        .replace(
            "    #[effects(pure)]\n    fn user()",
            "    #[effects(pure)]\n    #[uses(small, big)]\n    fn user()",
        );
    let proved = bmb::verify_sources_cached(&files(&strong), Some(&dir));
    assert!(proved.ok, "{}", proved.output);
    // Weaken the revealed definition again: the cached proof of `user` must not
    // be reused, because the query text it was keyed on changed.
    let weak = strong.replace("r > 0 && r < 10", "r > 0");
    let again = bmb::verify_sources_cached(&files(&weak), Some(&dir));
    assert!(!again.ok, "a stale proof was reused:\n{}", again.output);
    assert_eq!(again.output, bmb::verify_sources(&files(&weak)).output);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn foreign_or_damaged_cache_files_are_ignored() {
    let dir = tmp("foreign");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("proofs"), "not a cache\nsha256:00 1 1\n").unwrap();
    let out = bmb::verify_sources_cached(&files(BASE), Some(&dir));
    assert_eq!(out.cache.0, 0);
    assert_eq!(out.output, bmb::verify_sources(&files(BASE)).output);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn builds_and_maps_through_the_cache_match_the_uncached_ones() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let dir = tmp("paths");
    let f = files(&fs::read_to_string(root.join("run/search.bmb")).unwrap());
    let plain = bmb::emit_llvm_sources(&f);
    for _ in 0..2 {
        let cached = bmb::emit_llvm_sources_cached(&f, Some(&dir));
        assert_eq!(plain.verify.output, cached.verify.output);
        assert_eq!(plain.ir, cached.ir, "same proofs, same checks removed");
        assert_eq!(plain.checks, cached.checks);
    }
    let req = ["query".to_string(), "status".to_string()];
    assert_eq!(
        bmb::map_request(&f, &req).unwrap(),
        bmb::map_request_cached(&f, &req, Some(&dir)).unwrap()
    );
    let _ = fs::remove_dir_all(&dir);
}
