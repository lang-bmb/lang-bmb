//! Command-line behaviour that no other test covers.

use std::process::{Command, Stdio};

/// Output into a closed pipe (`bmb codes | head -1` once `head` has exited)
/// ends quietly with status 141, as a process killed by SIGPIPE, instead of
/// panicking with a backtrace (backlog 1D-24).
#[test]
fn a_closed_stdout_ends_quietly() {
    let bmb = env!("CARGO_BIN_EXE_bmb");
    for args in [&["codes"][..], &["codes", "--md"]] {
        let (reader, writer) = std::io::pipe().unwrap();
        drop(reader);
        let out = Command::new(bmb)
            .args(args)
            .stdout(writer)
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(141), "{args:?}: {err}");
        assert!(err.is_empty(), "{args:?}: {err}");
    }
}

/// An unknown local name suggests the one visible local within edit
/// distance 2, as a replacement `bmb fix` can apply (backlog 2D-18); two
/// equally near names suggest nothing.
#[test]
fn an_unknown_local_suggests_the_nearest_one() {
    let dir = std::env::temp_dir().join(format!("bmb-cli-near-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let check = |body: &str| {
        let f = dir.join("a.bmb");
        std::fs::write(
            &f,
            format!("#[summary(\"t\")]\nmod a {{\n    #[effects(pure)]\n    fn f(count: i64, other: i64) -> i64 {{\n        {body}\n    }}\n}}\n"),
        )
        .unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("check")
            .arg(&f)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    let out = check("cuont");
    assert!(out.contains("\"code\":\"E0301\""), "{out}");
    assert!(
        out.contains("\"fix\":{\"message\":\"did you mean `count`?\",\"replacement\":\"count\"}"),
        "{out}"
    );
    let out = check("zzzzzz");
    assert!(
        out.contains("\"code\":\"E0301\"") && !out.contains("did you mean"),
        "{out}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Data types in signatures, impl functions and type-qualified calls pass the
/// checker (backlog 2B-04); the proof gate reports them as E0814 until they
/// are encoded (2B-05).
#[test]
fn data_types_in_signatures_prove_but_do_not_compile_yet() {
    const SRC: &str = r#"#[summary("data types in signatures (ADR-0026, backlog 2B-04)")]
mod geo {
    enum Shape { Dot, Square(i64), Rect(Point, Point) }

    struct Point { x: i64, y: i64 }

    impl Point {
        #[effects(pure)]
        fn origin_dist(self: &Point, other: &Point) -> i64 {
            0
        }

        #[effects(pure)]
        fn keep(self: Point) -> Point {
            self
        }
    }

    #[effects(pure)]
    fn pass(s: Shape, o: Option<i64>, r: Result<Point, Shape>) -> Option<i64> {
        o
    }

    #[effects(pure)]
    fn call(p: Point, q: &Point) -> i64 {
        Point::origin_dist(&p, q)
    }
}

#[summary("another unit names the types by full path")]
mod user {
    #[effects(pure)]
    fn far(p: geo::Point) -> geo::Point {
        geo::Point::keep(p)
    }
}
"#;
    let dir = std::env::temp_dir().join(format!("bmb-cli-types-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("t.bmb");
    std::fs::write(&f, SRC).unwrap();
    let bmb = env!("CARGO_BIN_EXE_bmb");
    let check = Command::new(bmb).arg("check").arg(&f).output().unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stdout)
    );
    assert!(
        check.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&check.stdout)
    );
    let verify = Command::new(bmb).arg("verify").arg(&f).output().unwrap();
    let out = String::from_utf8_lossy(&verify.stdout);
    assert!(verify.status.success(), "{out}");
    // Code generation waits for backlog 2B-06.
    let ll = dir.join("t.ll");
    let emit = Command::new(bmb)
        .arg("emit-llvm")
        .arg(&f)
        .arg("-o")
        .arg(&ll)
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&emit.stdout);
    assert!(
        !emit.status.success() && out.contains("not compiled yet"),
        "{out}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_spec_reaching_an_unencoded_data_type_is_not_sent_to_the_solver() {
    // Integer signatures, but the revealed spec fn reaches one returning a
    // struct (no single SMT result yet, backlog 2B-05d): the caller is E0814,
    // not a solver error.
    const SRC: &str = r#"#[summary("a spec reaching a struct result")]
mod p {
    struct Point { x: i64, y: i64 }

    spec fn mid(a: i64) -> Point { Point { x: a, y: a } }

    spec fn same(a: i64, b: i64) -> bool { mid(a).x == b }

    spec fn also(a: i64, b: i64) -> bool { same(a, b) }

    #[effects(pure)]
    #[uses(also)]
    fn id(a: i64) -> (r: i64)
        ensures also(a, r),
    {
        a
    }
}
"#;
    let dir = std::env::temp_dir().join(format!("bmb-cli-spec-variants-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("t.bmb");
    std::fs::write(&f, SRC).unwrap();
    let bmb = env!("CARGO_BIN_EXE_bmb");
    let check = Command::new(bmb).arg("check").arg(&f).output().unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stdout)
    );
    let verify = Command::new(bmb).arg("verify").arg(&f).output().unwrap();
    let out = String::from_utf8_lossy(&verify.stdout);
    assert!(
        !verify.status.success() && out.contains("\"code\":\"E0814\""),
        "{out}"
    );
    assert!(!out.contains("E0812"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn data_values_fields_and_variants_check() {
    const SRC: &str = r#"#[summary("data type values (ADR-0026, backlog 2B-04b)")]
mod geo {
    enum Shape { Dot, Square(i64), Rect(Point, Point) }

    struct Point { x: i64, y: i64 }

    spec fn same(a: Point, b: Point) -> bool { a == b && a.x == b.x }

    #[effects(pure)]
    fn make(x: i64) -> Point {
        Point { y: 0, x: x }
    }

    #[effects(pure)]
    fn sum(p: &Point) -> i64 {
        p.x + p.y
    }

    #[effects(pure)]
    fn shapes(p: Point, q: Point) -> Shape {
        let d: Shape = Shape::Dot;
        let s: Shape = Shape::Square(p.x);
        Shape::Rect(p, q)
    }

    #[effects(pure)]
    fn first(a: i64) -> Option<i64> {
        let n: Option<i64> = None;
        Some(a)
    }

    #[effects(pure)]
    fn checked(a: i64) -> Result<Point, Shape> {
        let e: Result<Point, Shape> = Err(Shape::Dot);
        Ok(Point { x: a, y: a })
    }

    #[effects(pure)]
    fn shift(p: &mut Point, d: i64) -> () {
        p.x = d;
        p.y = d;
    }

    #[effects(pure)]
    fn local() -> i64 {
        let mut p: Point = make(1);
        p.y = 2;
        p.y
    }

    spec fn is_some(o: Option<i64>) -> bool { match o { None => false, Some(_) => true } }

    #[effects(pure)]
    fn area(s: &Shape) -> i64 {
        match *s {
            Shape::Dot => 0,
            Shape::Square(n) => n,
            Shape::Rect(a, b) => {
                let w: i64 = b.x;
                w
            }
        }
    }

    #[effects(pure)]
    fn literal_first(o: Option<u64>) -> bool {
        match o {
            None => 0,
            Some(v) => v,
        } == 3
    }

    #[effects(io)]
    fn report(r: Result<i64, Shape>) -> () {
        match r {
            Ok(v) => print_i64(v),
            Err(e) => {
                match e {
                    Shape::Dot => print_i64(0),
                    Shape::Square(_) => print_i64(1),
                    Shape::Rect(_, _) => print_i64(2),
                }
            }
        }
        print_i64(9);
    }
}
"#;
    let dir = std::env::temp_dir().join(format!("bmb-cli-values-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("t.bmb");
    std::fs::write(&f, SRC).unwrap();
    let check = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("check")
        .arg(&f)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stdout)
    );
    assert!(
        check.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&check.stdout)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
