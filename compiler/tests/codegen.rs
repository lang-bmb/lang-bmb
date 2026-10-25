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
fn an_unproved_index_traps_exactly_from_the_length() {
    // `get` has no precondition: its bounds check stays. Index 2 of three
    // elements reads; index 3 (the length) must trap (backlog 1B-38).
    let dir = std::env::temp_dir().join(format!("bmb-trap-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for (i, want) in [(2, Some("3\n")), (3, None)] {
        let text = format!(
            "#[summary(\"trap\")]\nmod m {{\n    #[effects(pure)]\n    fn get(a: &[i64], i: usize) -> i64 {{\n        a[i]\n    }}\n\n    #[effects(io)]\n    fn main() -> () {{\n        let a: [i64; 3] = [1, 2, 3];\n        print_i64(get(&a, {i}));\n    }}\n}}\n"
        );
        let out = bmb::emit_llvm_sources(&src("t.bmb", &text));
        assert!(!out.verify.ok);
        let exe = dir.join(format!("t{i}"));
        bmb::toolchain::build_executable(&out.ir.unwrap(), exe.to_str().unwrap()).unwrap();
        let run = Command::new(&exe).output().unwrap();
        match want {
            Some(o) => {
                assert!(run.status.success(), "index {i}: {}", run.status);
                assert_eq!(String::from_utf8_lossy(&run.stdout), o);
            }
            None => {
                assert!(!run.status.success(), "index {i} must trap");
                assert!(run.stdout.is_empty());
            }
        }
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_unproved_conversion_traps_exactly_outside_the_range() {
    // No precondition: the range checks of ADR-0028 §1 stay. Each pair is
    // the last value in range and the first one outside it (backlog 2C-02).
    let dir = std::env::temp_dir().join(format!("bmb-conv-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let cases = [
        ("u64", "i64", "0", Some("0\n")),
        ("u64", "i64", "-1", None),
        (
            "i64",
            "u64",
            "9223372036854775807",
            Some("9223372036854775807\n"),
        ),
        ("i64", "u64", "9223372036854775808", None),
    ];
    for (k, (to, from, v, want)) in cases.into_iter().enumerate() {
        let print = if to == "u64" {
            "print_u64"
        } else {
            "print_i64"
        };
        let text = format!(
            "#[summary(\"trap\")]\nmod m {{\n    #[effects(pure)]\n    fn conv(x: {from}) -> {to} {{\n        {to}::try_from(x).unwrap()\n    }}\n\n    #[effects(io)]\n    fn main() -> () {{\n        {print}(conv({v}));\n    }}\n}}\n"
        );
        let out = bmb::emit_llvm_sources(&src("t.bmb", &text));
        assert!(!out.verify.ok);
        let exe = dir.join(format!("c{k}"));
        bmb::toolchain::build_executable(&out.ir.unwrap(), exe.to_str().unwrap()).unwrap();
        let run = Command::new(&exe).output().unwrap();
        match want {
            Some(o) => {
                assert!(run.status.success(), "{from} {v} -> {to}: {}", run.status);
                assert_eq!(String::from_utf8_lossy(&run.stdout), o);
            }
            None => {
                assert!(!run.status.success(), "{from} {v} -> {to} must trap");
                assert!(run.stdout.is_empty());
            }
        }
    }
    let _ = fs::remove_dir_all(&dir);
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
    let body = &ir[ir.find("@\"m::set_all\"").unwrap()..];
    assert!(!body.contains("store i1 "), "{body}");
    assert!(
        ir.contains("getelementptr inbounds i8, ptr %\"a.ptr\""),
        "{ir}"
    );
    assert!(ir.contains("zext i1") && ir.contains("trunc i8"), "{ir}");
    assert!(
        body.contains("load i8, ptr") && body.contains(", align 1"),
        "{body}"
    );
}

#[test]
fn word_elements_are_eight_byte_aligned() {
    let ir = bmb::emit_llvm_sources(&src("p.bmb", PROVED)).ir.unwrap();
    let body = &ir[ir.find("@\"m::get\"").unwrap()..];
    assert!(
        body.lines()
            .any(|l| l.contains("load i64") && l.ends_with("align 8")),
        "{body}"
    );
}

#[test]
fn proofs_resting_on_failed_ones_keep_their_checks() {
    // `caller` proves its index in bounds only through `pick`'s `ensures`,
    // which `pick` itself fails to prove: the check must stay (1B-31).
    let text = r#"
#[summary("부분 증명")]
mod m {
    spec fn below(r: usize, n: usize) -> bool { r < n }

    spec fn nonempty(a: &[i64]) -> bool { a.len() > 0 }

    #[effects(pure)]
    #[uses(below)]
    fn pick(n: usize) -> (r: usize)
        ensures below(r, n),
    {
        n
    }

    #[effects(pure)]
    #[uses(below, nonempty)]
    fn caller(a: &[i64]) -> i64
        requires nonempty(a),
    {
        a[pick(a.len())]
    }

    #[effects(pure)]
    #[uses(nonempty)]
    fn unrelated(a: &[i64]) -> i64
        requires nonempty(a),
    {
        a[0]
    }
}
"#;
    let out = bmb::emit_llvm_sources(&src("p.bmb", text));
    assert!(!out.verify.ok);
    let ir = out.ir.unwrap();
    let body = |name: &str| -> String {
        let start = ir.find(&format!("@\"m::{name}\"(")).unwrap();
        let rest = &ir[start..];
        rest[..rest.find("\n}").unwrap()].to_string()
    };
    assert!(
        body("caller").contains("; unproved: index may be out of bounds"),
        "{ir}"
    );
    assert!(!body("unrelated").contains("unproved"), "{ir}");
}

#[test]
fn only_the_entry_point_is_external() {
    // Closed world (backlog 2D-07): LLVM may inline and drop every function.
    let text = r#"
#[summary("연결")]
mod m {
    #[effects(pure)]
    fn unused(x: i64) -> i64 { x }

    #[effects(pure)]
    fn one() -> i64 { 1 }

    #[effects(io)]
    fn main() -> () {
        print_i64(one());
    }
}
"#;
    let out = bmb::emit_llvm_sources(&src("l.bmb", text));
    let ir = out.ir.unwrap();
    let defines: Vec<&str> = ir.lines().filter(|l| l.starts_with("define ")).collect();
    assert_eq!(defines.len(), 6, "{ir}");
    for d in &defines {
        assert!(
            d.starts_with("define internal ") || d.starts_with("define i32 @main()"),
            "{d}"
        );
    }
    assert_eq!(
        defines.iter().filter(|d| !d.contains(" internal ")).count(),
        1
    );
    for f in [
        "@\"m::unused\"",
        "@\"m::one\"",
        "@\"m::main\"",
        "@bmb_print_i64",
    ] {
        assert!(
            defines
                .iter()
                .any(|d| d.starts_with("define internal ") && d.contains(f)),
            "{f}: {ir}"
        );
    }
    // The built program still runs.
    let dir = std::env::temp_dir().join(format!("bmb-link-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("l");
    bmb::toolchain::build_executable(&ir, exe.to_str().unwrap()).unwrap();
    let run = Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn vec_is_gated_until_its_encoding_and_lowering_exist() {
    // Backlog 2G-02/03: the checker and the prover accept `Vec`; the code
    // generator (2G-04) refuses it with E0814 instead of guessing.
    let text = "#[summary(\"g\")]\nmod m {\n    #[effects(pure)]\n    fn n() -> usize {\n        let mut v: Vec<i64> = Vec::new();\n        v.push(1);\n        v.len()\n    }\n\n    #[effects(io)]\n    fn main() -> () {\n        print_u64(u64::try_from(n()).unwrap());\n    }\n}\n";
    assert!(bmb::check_sources(&src("g.bmb", text)).ok);
    let out = bmb::emit_llvm_sources(&src("g.bmb", text));
    assert!(out.ir.is_none());
    assert!(!out.verify.ok);
    assert!(
        out.verify.output.contains("\"code\":\"E0814\""),
        "{}",
        out.verify.output
    );
}
