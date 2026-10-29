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

/// Every run program that owns heap memory (`Vec`, `Box`) frees each
/// allocation exactly once and touches no freed memory (ADR-0047 §4,
/// backlog 2O-01): built at -O0 (optimization may delete an allocation
/// that does not escape, and with it a missing free) and run under
/// valgrind memcheck, or AddressSanitizer where valgrind is missing.
#[test]
fn run_corpus_frees_every_allocation_once() {
    let dir = std::env::temp_dir().join(format!("bmb-free-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let ok = |c: &mut Command| c.output().is_ok_and(|o| o.status.success());
    let valgrind = ok(Command::new("valgrind").arg("--version"));
    let mut files: Vec<PathBuf> = fs::read_dir(root())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    files.sort();
    let mut checked = 0;
    for p in files {
        let text = fs::read_to_string(&p).unwrap();
        if !text.contains("Box<") && !text.contains("Vec<") {
            continue;
        }
        let out = bmb::emit_llvm_sources(&src("run.bmb", &text));
        let ll = dir.join(p.file_stem().unwrap()).with_extension("ll");
        fs::write(&ll, out.ir.unwrap()).unwrap();
        let exe = dir.join(p.file_stem().unwrap());
        let mut cc = Command::new("clang");
        cc.args(["-O0", "-Wno-override-module"]);
        if !valgrind {
            cc.arg("-fsanitize=address");
        }
        let built = cc.arg("-o").arg(&exe).arg(&ll).output().unwrap();
        assert!(
            built.status.success(),
            "{}: {}",
            p.display(),
            String::from_utf8_lossy(&built.stderr)
        );
        let run = if valgrind {
            Command::new("valgrind")
                .args([
                    "-q",
                    "--leak-check=full",
                    "--show-leak-kinds=definite,indirect,possible",
                    "--errors-for-leak-kinds=definite,indirect,possible",
                    "--error-exitcode=99",
                ])
                .arg(&exe)
                .output()
                .unwrap()
        } else {
            Command::new(&exe)
                .env("ASAN_OPTIONS", "detect_leaks=1:exitcode=99")
                .output()
                .unwrap()
        };
        assert!(
            run.status.success(),
            "{} reports a memory error:\n{}",
            p.display(),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            fs::read_to_string(p.with_extension("out")).unwrap(),
            "{}",
            p.display()
        );
        checked += 1;
    }
    assert!(checked > 0);
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

/// An unproved shift amount is checked against the width (ADR-0048 §2, as
/// Rust's debug build panics): the last amount below it shifts, the width
/// and a negative amount trap (backlog 2N-02). A literal amount below the
/// width needs no check.
#[test]
fn an_unproved_shift_traps_exactly_from_the_width() {
    let dir = std::env::temp_dir().join(format!("bmb-shift-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let cases = [
        ("u8", "u32", "<<", "1", "7", Some("128\n")),
        ("u8", "u32", "<<", "1", "8", None),
        ("i32", "i64", ">>", "-8", "31", Some("-1\n")),
        ("i32", "i64", ">>", "-8", "32", None),
        ("i32", "i64", ">>", "-8", "-1", None),
        (
            "u64",
            "usize",
            "<<",
            "3",
            "63",
            Some("9223372036854775808\n"),
        ),
        ("u64", "usize", "<<", "3", "64", None),
    ];
    for (k, (t, a, op, x, s, want)) in cases.into_iter().enumerate() {
        let print = if t.starts_with('u') {
            "print_u64(u64::try_from(sh(x, s)).unwrap())"
        } else {
            "print_i64(i64::try_from(sh(x, s)).unwrap())"
        };
        let text = format!(
            "#[summary(\"trap\")]\nmod m {{\n    #[effects(pure)]\n    fn sh(x: {t}, s: {a}) -> {t} {{\n        x {op} s\n    }}\n\n    #[effects(io)]\n    fn main() -> () {{\n        let x: {t} = {x};\n        let s: {a} = {s};\n        {print};\n    }}\n}}\n"
        );
        let out = bmb::emit_llvm_sources(&src("t.bmb", &text));
        assert!(!out.verify.ok);
        let exe = dir.join(format!("s{k}"));
        bmb::toolchain::build_executable(&out.ir.unwrap(), exe.to_str().unwrap()).unwrap();
        let run = Command::new(&exe).output().unwrap();
        match want {
            Some(o) => {
                assert!(run.status.success(), "{x} {op} {s} on {t}: {}", run.status);
                assert_eq!(String::from_utf8_lossy(&run.stdout), o);
            }
            None => {
                assert!(!run.status.success(), "{x} {op} {s} on {t} must trap");
                assert!(run.stdout.is_empty());
            }
        }
    }
    let text = "#[summary(\"lit\")]\nmod m {\n    #[effects(pure)]\n    fn f(x: u16) -> u16 {\n        (x << 3) | (x >> 15)\n    }\n}\n";
    let out = bmb::emit_llvm_sources(&src("t.bmb", text));
    assert!(out.verify.ok, "{}", out.verify.output);
    assert_eq!(out.checks, 0);
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

/// A narrow product is computed in 64 bits before its range check: the
/// flag must hold for every operand pair, or the check reads a poison
/// value. `u32` products reach `(2^32 - 1)^2 > i64::MAX` (backlog 2N-04).
#[test]
fn narrow_products_claim_only_what_holds() {
    let text = r#"
#[summary("곱")]
mod m {
    #[effects(pure)]
    fn a(x: u32, y: u32) -> u32 { x * y }

    #[effects(pure)]
    fn b(x: i32, y: i32) -> i32 { x * y }

    #[effects(pure)]
    fn c(x: u16, y: u16) -> u16 { x * y }
}
"#;
    let ir = bmb::emit_llvm_sources(&src("p.bmb", text)).ir.unwrap();
    let body = |f: &str, next: Option<&str>| {
        let s = ir.find(&format!("@\"m::{f}\"(")).unwrap();
        let e = next.map_or(ir.len(), |n| ir.find(&format!("@\"m::{n}\"(")).unwrap());
        ir[s..e].to_string()
    };
    let (a, b, c) = (body("a", Some("b")), body("b", Some("c")), body("c", None));
    assert!(a.contains("mul nuw i64") && !a.contains("mul nsw"), "{a}");
    assert!(b.contains("mul nsw i64"), "{b}");
    assert!(c.contains("mul nuw i64"), "{c}");
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
    assert_eq!(defines.len(), 7, "{ir}");
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
fn a_proved_push_has_no_check_and_an_unproved_one_keeps_it() {
    // Backlog 2G-04: E0816 like every obligation (ADR-0031 §4).
    let text = "#[summary(\"g\")]\nmod m {\n    #[effects(pure)]\n    fn grow(v: &mut Vec<i64>) -> () {\n        v.push(2);\n    }\n\n    #[effects(pure)]\n    fn n() -> usize {\n        let mut v: Vec<i64> = Vec::new();\n        v.push(1);\n        v.len()\n    }\n}\n";
    let out = bmb::emit_llvm_sources(&src("g.bmb", text));
    let ir = out.ir.unwrap();
    assert_eq!(ir.matches("; proved E0816").count(), 1, "{ir}");
    assert_eq!(ir.matches("unproved: `push` may exceed").count(), 1, "{ir}");
}

#[test]
fn an_unproved_vec_index_traps_exactly_from_the_length() {
    // As for slices (1B-38): index 2 of three elements reads, 3 traps.
    let dir = std::env::temp_dir().join(format!("bmb-vtrap-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for (i, want) in [(2, Some("3\n")), (3, None)] {
        let text = format!(
            "#[summary(\"trap\")]\nmod m {{\n    #[effects(pure)]\n    fn get(v: &Vec<i64>, i: usize) -> i64 {{\n        v[i]\n    }}\n\n    #[effects(io)]\n    fn main() -> () {{\n        let mut v: Vec<i64> = Vec::new();\n        v.push(1);\n        v.push(2);\n        v.push(3);\n        print_i64(get(&v, {i}));\n    }}\n}}\n"
        );
        let out = bmb::emit_llvm_sources(&src("t.bmb", &text));
        assert!(!out.verify.ok);
        let exe = dir.join(format!("v{i}"));
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

/// `tests/lib_run`: programs that use the libraries (`lib/`: bytes, ADR-0044
/// §4; UTF-8, CRC-32, SHA-256, base64, T4) are built with every library file, fully proved, without
/// runtime checks, and print their `.out`.
#[test]
fn library_programs_print_expected_output() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut libs: Vec<PathBuf> = fs::read_dir(repo.join("lib"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    libs.sort();
    let libs: Vec<(String, String)> = libs
        .iter()
        .map(|p| {
            (
                format!("lib/{}", p.file_name().unwrap().to_string_lossy()),
                fs::read_to_string(p).unwrap(),
            )
        })
        .collect();
    let dir = std::env::temp_dir().join(format!("bmb-lib-run-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut files: Vec<PathBuf> = fs::read_dir(repo.join("tests").join("lib_run"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    for p in files {
        let mut program = vec![("run.bmb".to_string(), fs::read_to_string(&p).unwrap())];
        program.extend(libs.iter().cloned());
        let out = bmb::emit_llvm_sources(&program);
        assert!(out.verify.ok, "{}:\n{}", p.display(), out.verify.output);
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

/// The program without its ghost code: every `proof { .. }` block and
/// every `proof fn` item (with the attribute lines above it) removed.
fn strip_ghost(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut keep = vec![true; lines.len()];
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim_start();
        let indent = lines[i].len() - t.len();
        if t == "proof {" || t.starts_with("proof fn ") {
            let mut start = i;
            while t.starts_with("proof fn ")
                && start > 0
                && lines[start - 1].trim_start().starts_with("#[")
            {
                start -= 1;
            }
            let close = format!("{}}}", " ".repeat(indent));
            let mut end = i;
            while lines[end] != close {
                end += 1;
            }
            keep[start..=end].iter_mut().for_each(|k| *k = false);
            i = end;
        }
        i += 1;
    }
    let kept: Vec<&str> = lines
        .iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(l, _)| *l)
        .collect();
    kept.join("\n") + "\n"
}

#[test]
fn ghost_code_leaves_no_trace_in_the_ir() {
    // ADR-0062 §2: code generation erases `proof fn`s and `proof` blocks.
    let text = fs::read_to_string(root().join("max_list.bmb")).unwrap();
    let stripped = strip_ghost(&text);
    assert!(text.contains("proof fn ") && text.contains("proof {"));
    assert!(
        !stripped.contains("proof fn ") && !stripped.contains("proof {"),
        "{stripped}"
    );
    // Unproved (every obligation a check), the IR is the same bytes as the
    // program's without its ghost code.
    let with = bmb::emit_llvm_sources_with(&src("m.bmb", &text), true);
    let without = bmb::emit_llvm_sources_with(&src("m.bmb", &stripped), true);
    assert_eq!(with.ir.unwrap(), without.ir.unwrap());
    // Proved, no lemma appears and no check is left: a lemma's facts reach
    // code generation only as proved obligations.
    let proved = bmb::emit_llvm_sources(&src("m.bmb", &text));
    assert!(proved.verify.ok, "{}", proved.verify.output);
    assert_eq!(proved.checks, 0);
    let ir = proved.ir.unwrap();
    for name in [
        "contains_has_index",
        "sum_grows",
        "contains_prefix",
        "sum_to",
    ] {
        assert!(!ir.contains(name), "{name}: {ir}");
    }
    // Without the lemmas the running sum of `total` keeps its overflow check.
    let unproved = bmb::emit_llvm_sources(&src("m.bmb", &stripped));
    assert!(!unproved.verify.ok);
    assert!(unproved.checks > 0);
}

/// The bit methods (ADR-0048 §2, backlog 2N-03) compute what Rust's do: for
/// every integer type, a program applies each method to edge and pattern
/// values (and amounts below, at and past the width) and its output is
/// compared with the same methods run by this test (compiled by rustc).
#[test]
fn bit_methods_compute_rusts_results() {
    use bmb::ast::IntTy;
    let dir = std::env::temp_dir().join(format!("bmb-bitm-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let amounts: [u32; 9] = [0, 1, 3, 7, 8, 31, 33, 64, u32::MAX];
    for i in IntTy::ALL {
        let (lo, hi) = (i.min_value(), i.max_value());
        let size = 1i128 << i.bits();
        let mut xs: Vec<i128> = vec![lo, lo + 1, -1, 0, 1, 2, 0x5a, 0x80, hi - 1, hi];
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        for _ in 0..6 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            xs.push(seed as i128);
        }
        // Into the type's range by its two's complement image.
        let mut xs: Vec<i128> = xs
            .into_iter()
            .map(|v| {
                let u = v.rem_euclid(size);
                if u > hi { u - size } else { u }
            })
            .collect();
        xs.dedup();
        let mut want = String::new();
        macro_rules! on {
            ($t:ty) => {{
                for &x in &xs {
                    let x = x as $t;
                    for &k in &amounts {
                        for v in [
                            x.rotate_left(k) as i128,
                            x.rotate_right(k) as i128,
                            x.wrapping_shl(k) as i128,
                            x.wrapping_shr(k) as i128,
                        ] {
                            want.push_str(&format!("{v}\n"));
                        }
                    }
                    for v in [
                        x.count_ones() as i128,
                        x.count_zeros() as i128,
                        x.leading_zeros() as i128,
                        x.trailing_zeros() as i128,
                        x.swap_bytes() as i128,
                    ] {
                        want.push_str(&format!("{v}\n"));
                    }
                }
            }};
        }
        match i {
            IntTy::I8 => on!(i8),
            IntTy::I16 => on!(i16),
            IntTy::I32 => on!(i32),
            IntTy::I64 => on!(i64),
            IntTy::Isize => on!(isize),
            IntTy::U8 => on!(u8),
            IntTy::U16 => on!(u16),
            IntTy::U32 => on!(u32),
            IntTy::U64 => on!(u64),
            IntTy::Usize => on!(usize),
        }
        let t = i.name();
        let (print, conv) = if i.signed() {
            ("print_i64", "i64")
        } else {
            ("print_u64", "u64")
        };
        let lits: Vec<String> = xs.iter().map(|v| v.to_string()).collect();
        let ks: Vec<String> = amounts.iter().map(|v| v.to_string()).collect();
        let text = format!(
            "#[summary(\"bit methods\")]
mod m {{
    #[effects(io)]
    fn show(v: {t}) -> () {{
        {print}({conv}::try_from(v).unwrap());
    }}

    #[effects(io)]
    fn count(v: u32) -> () {{
        print_u64(u64::try_from(v).unwrap());
    }}

    #[effects(io)]
    fn main() -> () {{
        let xs: [{t}; {nx}] = [{xs}];
        let ks: [u32; {nk}] = [{ks}];
        for a in 0..xs.len() {{
            let x: {t} = xs[a];
            for b in 0..ks.len() {{
                let k: u32 = ks[b];
                show(x.rotate_left(k));
                show(x.rotate_right(k));
                show(x.wrapping_shl(k));
                show(x.wrapping_shr(k));
            }}
            count(x.count_ones());
            count(x.count_zeros());
            count(x.leading_zeros());
            count(x.trailing_zeros());
            show(x.swap_bytes());
        }}
    }}
}}
",
            nx = xs.len(),
            xs = lits.join(", "),
            nk = amounts.len(),
            ks = ks.join(", "),
        );
        let out = bmb::emit_llvm_sources(&src("t.bmb", &text));
        assert!(out.verify.ok, "{t}: {}", out.verify.output);
        assert_eq!(out.checks, 0, "{t}");
        let exe = dir.join(format!("m_{t}"));
        bmb::toolchain::build_executable(&out.ir.unwrap(), exe.to_str().unwrap()).unwrap();
        let run = Command::new(&exe).output().unwrap();
        assert!(run.status.success(), "{t}: {}", run.status);
        assert_eq!(String::from_utf8_lossy(&run.stdout), want, "{t}");
    }
    let _ = fs::remove_dir_all(&dir);
}

/// A BMB `f64` literal (or constant) for `v` (ADR-0061 §2).
fn f64_lit(v: f64) -> String {
    if v.is_nan() {
        return "f64::NAN".into();
    }
    if v.is_infinite() {
        return if v > 0.0 {
            "f64::INFINITY"
        } else {
            "f64::NEG_INFINITY"
        }
        .into();
    }
    let s = format!("{:?}", v.abs());
    let s = match s.split_once('e') {
        Some((m, e)) if !m.contains('.') => format!("{m}.0e{e}"),
        _ => s,
    };
    if v.is_sign_negative() {
        format!("-{s}")
    } else {
        s
    }
}

/// Builds a fully proved program, runs it and returns its output.
fn run_program(tag: &str, text: &str) -> String {
    let out = bmb::emit_llvm_sources(&src("f.bmb", text));
    assert!(out.verify.ok, "{tag}: {}", out.verify.output);
    assert_eq!(out.checks, 0, "{tag}");
    let dir = std::env::temp_dir().join(format!("bmb-{tag}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("p");
    bmb::toolchain::build_executable(&out.ir.unwrap(), exe.to_str().unwrap()).unwrap();
    let run = Command::new(&exe).output().unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert!(run.status.success(), "{tag}: {}", run.status);
    String::from_utf8_lossy(&run.stdout).into_owned()
}

/// The `f64` operations (ADR-0061 §6) compute what Rust's do over a table
/// of edge values: `+ - * /`, unary `-`, `sqrt` and `abs` bit for bit
/// (printed with 1074 decimals, which tell every `f64` apart and show the
/// sign of zero), the six comparisons (NaN, signed zeros, equal operands),
/// `min`, `max`, `is_nan`, `is_finite`, and `f64::from` of `i64`, `i32` and
/// `u16`. Catches an `fadd`/`fsub` or `olt`/`ole` mix-up in code generation
/// (gate-power mutants `c-fadd-as-fsub`, `c-olt-as-ole`).
#[test]
fn f64_operations_compute_rusts_results() {
    let xs: [f64; 14] = [
        0.0,
        -0.0,
        1.0,
        -1.5,
        0.1,
        1.0 / 3.0,
        1.0e308,
        -1.0e308,
        5.0e-324,
        2.2250738585072014e-308,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        123456789.125,
    ];
    let exact = |v: f64| format!("{v:.1074}\n");
    let flag = |b: bool| format!("{}\n", u64::from(b));
    let mut want = String::new();
    for &a in &xs {
        for &b in &xs {
            for v in [a + b, a - b, a * b, a / b] {
                want.push_str(&exact(v));
            }
            for c in [a < b, a <= b, a > b, a >= b, a == b, a != b] {
                want.push_str(&flag(c));
            }
            // Rust leaves the sign of `min`/`max` of two zeros open.
            if !(a == 0.0 && b == 0.0) {
                want.push_str(&exact(a.min(b)));
                want.push_str(&exact(a.max(b)));
            }
        }
        for v in [a.sqrt(), a.abs(), -a] {
            want.push_str(&exact(v));
        }
        want.push_str(&flag(a.is_nan()));
        want.push_str(&flag(a.is_finite()));
    }
    let ks: [i64; 6] = [
        0,
        -1,
        2147483647,
        -2147483648,
        9007199254740992,
        -9007199254740991,
    ];
    for &k in &ks {
        want.push_str(&exact(k as f64));
        want.push_str(&exact(f64::from(k as i32)));
        want.push_str(&exact(f64::from(k as u16)));
    }
    let lits: Vec<String> = xs.iter().map(|v| f64_lit(*v)).collect();
    let text = format!(
        "#[summary(\"f64 ops\")]
mod m {{
    spec fn exact(k: i64) -> bool {{ -9007199254740992 <= k && k <= 9007199254740992 }}

    #[effects(io)]
    fn show(x: f64) -> () {{
        print_f64(x, 1074);
    }}

    #[effects(io)]
    fn flag(b: bool) -> () {{
        if b {{
            print_u64(1);
        }} else {{
            print_u64(0);
        }}
    }}

    #[effects(io)]
    #[uses(exact)]
    fn conv(k: i64, w: i32, u: u16) -> ()
        requires exact(k),
    {{
        show(f64::from(k));
        show(f64::from(w));
        show(f64::from(u));
    }}

    #[effects(io)]
    #[uses(exact)]
    fn main() -> () {{
        let xs: [f64; {n}] = [{xs}];
        for i in 0..xs.len() {{
            let a: f64 = xs[i];
            for j in 0..xs.len() {{
                let b: f64 = xs[j];
                show(a + b);
                show(a - b);
                show(a * b);
                show(a / b);
                flag(a < b);
                flag(a <= b);
                flag(a > b);
                flag(a >= b);
                flag(a == b);
                flag(a != b);
                if !(a == 0.0 && b == 0.0) {{
                    show(a.min(b));
                    show(a.max(b));
                }}
            }}
            show(a.sqrt());
            show(a.abs());
            show(-a);
            flag(a.is_nan());
            flag(a.is_finite());
        }}
        {convs}
    }}
}}
",
        n = xs.len(),
        xs = lits.join(", "),
        convs = ks
            .iter()
            .map(|k| format!("conv({k}, {}, {});", *k as i32, *k as u16))
            .collect::<Vec<_>>()
            .join("\n        "),
    );
    assert_eq!(run_program("f64ops", &text), want);
}

/// `print_f64(x, n)` prints what Rust's `println!("{:.n}", x)` does
/// (ADR-0061 §6): exact ties round to even (`k / 16` at 0 to 3 decimals),
/// values that are not ties, large and tiny magnitudes, signed zero, NaN
/// and the infinities, at precisions up to 40.
#[test]
fn print_f64_formats_as_rust() {
    let mut xs: Vec<f64> = (-40..=40).map(|k| f64::from(k) / 16.0).collect();
    xs.extend([
        2.675,
        1.005,
        0.045,
        1.0e21,
        1.0e22,
        123456789012345680.0,
        1.0e23,
        1.0e-7,
        5.0e-324,
        f64::MAX,
        -f64::MIN_POSITIVE,
        -0.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.1 + 0.2,
    ]);
    let ds: [u32; 7] = [0, 1, 2, 3, 6, 17, 40];
    let mut want = String::new();
    for &x in &xs {
        for &d in &ds {
            want.push_str(&format!("{x:.*}\n", d as usize));
        }
    }
    let lits: Vec<String> = xs.iter().map(|v| f64_lit(*v)).collect();
    let text = format!(
        "#[summary(\"print_f64\")]
mod m {{
    #[effects(io)]
    fn main() -> () {{
        let xs: [f64; {n}] = [{xs}];
        let ds: [u32; {nd}] = [{ds}];
        for i in 0..xs.len() {{
            for j in 0..ds.len() {{
                if ds[j] <= 65535 {{
                    print_f64(xs[i], ds[j]);
                }}
            }}
        }}
    }}
}}
",
        n = xs.len(),
        xs = lits.join(", "),
        nd = ds.len(),
        ds = ds.map(|d| d.to_string()).join(", "),
    );
    assert_eq!(run_program("printf64", &text), want);
}

/// Floating point leaves no room for the optimizer (ADR-0061 §6, C2): no
/// fast-math flag and no `contract` on any `f64` instruction, and a
/// multiply-add stays two rounded operations (an FMA would round once).
#[test]
fn f64_code_has_no_fast_math_or_contraction() {
    let text = "#[summary(\"fma\")]
mod m {
    #[effects(pure)]
    fn madd(a: f64, b: f64, c: f64) -> f64 {
        a * b + c
    }

    #[effects(io)]
    fn main() -> () {
        let a: f64 = 0.1;
        print_f64(madd(a, 10.0, -1.0), 30);
    }
}
";
    let out = bmb::emit_llvm_sources(&src("fma.bmb", text));
    let ir = out.ir.unwrap();
    for line in ir.lines().filter(|l| l.contains(" double ")) {
        for flag in [
            "fast", "contract", "reassoc", "nnan", "ninf", "nsz", "arcp", "afn",
        ] {
            assert!(!line.contains(&format!(" {flag} ")), "{line}");
        }
    }
    let a: f64 = std::hint::black_box(0.1);
    let want = format!("{:.30}\n", a * 10.0 - 1.0);
    assert_eq!(run_program("fma", text), want);
    assert_ne!(want, format!("{:.30}\n", a.mul_add(10.0, -1.0)));
}

/// The Benchmarks Game programs that have a Rust reference
/// (`bench/game/ref/*.rs`, backlog T3-04: n-body, spectral-norm) print the
/// benchmark's check output for its small argument (`bench/game/*.out`),
/// and so does the Rust reference built by rustc.
#[test]
fn game_programs_match_their_rust_references() {
    let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../bench/game");
    let dir = std::env::temp_dir().join(format!("bmb-game-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut refs: Vec<PathBuf> = fs::read_dir(game.join("ref"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    refs.sort();
    assert!(refs.len() >= 2);
    for r in refs {
        let stem = r.file_stem().unwrap().to_str().unwrap().to_string();
        let want = fs::read_to_string(game.join(&stem).with_extension("out")).unwrap();
        let text = fs::read_to_string(game.join(&stem).with_extension("bmb")).unwrap();
        assert_eq!(run_program(&stem, &text), want, "{stem}");
        let exe = dir.join(format!("{stem}-rs"));
        let built = Command::new("rustc")
            .args(["-O", "--edition", "2021", "-o"])
            .arg(&exe)
            .arg(&r)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{stem}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new(&exe).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&run.stdout), want, "{stem} (Rust)");
    }
    let _ = fs::remove_dir_all(&dir);
}
