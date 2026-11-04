//! Proof stability (backlog 1B-19): the verdict of every definition in the
//! corpora must not depend on the solver seed. Through the CLI (`bmb
//! stability`), so the test runs whichever compiler is built (backlog 4A-04).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `bmb stability --seeds N FILE`: its output and whether it succeeded
/// (it fails when a verdict depends on the seed).
fn stability(file: &Path, seeds: u32) -> (String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["stability", "--seeds", &seeds.to_string()])
        .arg(file)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.success(),
    )
}

fn corpus(dirs: &[&str]) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut files: Vec<PathBuf> = dirs
        .iter()
        .flat_map(|d| fs::read_dir(root.join(d)).unwrap())
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    files.sort();
    files
}

#[test]
fn corpus_verdicts_do_not_depend_on_the_seed() {
    for p in corpus(&["verify/pass", "verify/fail", "run", "check/ok"]) {
        let (out, stable) = stability(&p, 3);
        assert!(stable, "{}:\n{out}", p.display());
        let summary = out.lines().last().unwrap();
        assert!(summary.contains("\"seeds\":3"), "{summary}");
        // Every passing corpus file is fully verified under every seed.
        if !p.to_str().unwrap().contains("fail") {
            for line in out.lines().filter(|l| l.contains("\"def\"")) {
                assert!(!line.contains("\"failed\""), "{}: {line}", p.display());
            }
        }
    }
}

#[test]
fn stability_reports_one_verdict_per_seed() {
    let text = r#"
#[summary("s")]
mod m {
    #[effects(pure)]
    fn id(x: i64) -> i64 { x }
}
"#;
    let dir = std::env::temp_dir().join(format!("bmb-stability-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("s.bmb");
    fs::write(&file, text).unwrap();
    let (out, stable) = stability(&file, 4);
    let _ = fs::remove_dir_all(&dir);
    assert!(stable);
    assert!(
        out.starts_with(
            "{\"def\":\"m::id\",\"stable\":true,\"verdicts\":[\"verified\",\"verified\",\"verified\",\"verified\"]"
        ),
        "{out}"
    );
    assert!(
        out.ends_with("\"unstable\":0,\"rlimit_spread_max\":1.00}\n"),
        "{out}"
    );
}
