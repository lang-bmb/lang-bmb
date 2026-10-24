//! Code generation (backlog 1C-05, 1C-06): built programs print what they
//! should, and runtime checks exist exactly where proofs are missing.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/run")
}

fn src(name: &str, text: &str) -> Vec<(String, String)> {
    vec![(name.to_string(), text.to_string())]
}

#[test]
fn run_corpus_prints_expected_output() {
    let dir = std::env::temp_dir().join(format!("bmb-run-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut files: Vec<PathBuf> = fs::read_dir(root())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    for p in files {
        let out = bmb::emit_llvm_sources(&src("run.bmb", &fs::read_to_string(&p).unwrap()));
        assert!(
            out.verify.ok,
            "{} must be fully proved:\n{}",
            p.display(),
            out.verify.output
        );
        assert_eq!(out.checks, 0, "{}", p.display());
        let exe = dir.join(p.file_stem().unwrap());
        bmb::toolchain::build_executable(&out.ir.unwrap(), exe.to_str().unwrap()).unwrap();
        let run = Command::new(&exe).output().unwrap();
        assert!(
            run.status.success(),
            "{} exited with {}",
            p.display(),
            run.status
        );
        let expected = fs::read_to_string(p.with_extension("out")).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{}",
            p.display()
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

const PROVED: &str = r#"
#[summary("범위가 증명된 인덱스")]
mod m {
    spec fn idx_ok(a: &[i64], i: usize) -> bool { i < a.len() }

    #[effects(pure)]
    #[uses(idx_ok)]
    fn get(a: &[i64], i: usize) -> i64
        requires idx_ok(a, i),
    {
        a[i]
    }

    #[effects(pure)]
    fn inc(x: u64) -> u64 { x.wrapping_add(1) }
}
"#;

#[test]
fn proved_index_has_no_check_and_unproved_keeps_one() {
    let proved = bmb::emit_llvm_sources(&src("p.bmb", PROVED));
    assert!(proved.verify.ok, "{}", proved.verify.output);
    let ir = proved.ir.unwrap();
    assert_eq!(proved.checks, 0);
    assert!(!ir.contains("unproved"));
    let gep = ir
        .lines()
        .find(|l| l.contains("getelementptr inbounds i64, ptr %\"a.ptr\""))
        .expect("indexing emitted");
    assert!(gep.ends_with("; proved E0805"), "{gep}");

    // The same function without its precondition: the proof fails, the check stays.
    let unproved_src = PROVED.replace("        requires idx_ok(a, i),\n", "");
    let unproved = bmb::emit_llvm_sources(&src("u.bmb", &unproved_src));
    assert!(!unproved.verify.ok);
    let ir = unproved.ir.unwrap();
    assert_eq!(unproved.checks, 1);
    assert!(
        ir.contains("; unproved: index may be out of bounds"),
        "{ir}"
    );
    assert!(ir.contains("call void @llvm.trap()"));
}

#[test]
fn proved_arithmetic_carries_no_wrap_flags() {
    let text = r#"
#[summary("산술")]
mod m {
    spec fn small(x: i64) -> bool { x < 100 && x > -100 }

    #[effects(pure)]
    #[uses(small)]
    fn f(x: i64) -> i64
        requires small(x),
    {
        x * 3 + 1
    }

    #[effects(pure)]
    fn g(x: i64) -> i64 { x + 1 }
}
"#;
    let out = bmb::emit_llvm_sources(&src("a.bmb", text));
    let ir = out.ir.unwrap();
    let f = &ir[ir.find("@\"m::f\"").unwrap()..ir.find("@\"m::g\"").unwrap()];
    assert!(
        f.contains("mul nsw i64") && f.contains("add nsw i64"),
        "{f}"
    );
    let g = &ir[ir.find("@\"m::g\"").unwrap()..];
    assert!(g.contains("@llvm.sadd.with.overflow.i64"), "{g}");
    assert_eq!(out.checks, 1);
}

#[test]
fn mut_slices_are_noalias_and_shared_ones_readonly() {
    let text = r#"
#[summary("별칭")]
mod m {
    #[effects(pure)]
    fn f(a: &mut [i64], b: &[i64]) -> () {}
}
"#;
    let ir = bmb::emit_llvm_sources(&src("n.bmb", text)).ir.unwrap();
    assert!(ir.contains("ptr noalias %\"a.ptr\""), "{ir}");
    assert!(ir.contains("ptr readonly %\"b.ptr\""), "{ir}");
    assert!(
        !ir.contains("llvm.assume"),
        "assumes must come only from proofs"
    );
}

#[test]
fn bool_arrays_are_bytes_in_memory() {
    // `i1` in memory blocks LLVM's fill and vectorization idioms (backlog 1E-09).
    let text = r#"
#[summary("bool 배열")]
mod m {
    #[effects(pure)]
    fn set_all(a: &mut [bool]) -> () {
        let mut i: usize = 0;
        while i < a.len()
            invariant i <= a.len(),
            decreases a.len() - i,
        {
            a[i] = !a[i];
            i = i + 1;
        }
    }
}
"#;
    let ir = bmb::emit_llvm_sources(&src("b.bmb", text)).ir.unwrap();
    assert!(!ir.contains("store i1 ") || !ir.contains("ptr %t"), "{ir}");
    assert!(
        ir.contains("getelementptr inbounds i8, ptr %\"a.ptr\""),
        "{ir}"
    );
    assert!(ir.contains("zext i1") && ir.contains("trunc i8"), "{ir}");
}
