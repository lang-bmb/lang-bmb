//! `bmb fix` (backlog 2D-04): applies every mechanical fix, re-checks and
//! repeats until none applies.

use std::process::Command;

const HABITS: &str = r#"#[summary("Rust habits")]
mod m {
    #[effects(pure)]
    pub fn add(x: i32, y: i64) -> i64 {
        let z = x + 1i64;
        z + y
    }

    #[effects(pure)]
    fn first(a: &'a [i64]) -> i64 {
        let v: [i64; 2] = vec![1, 2];
        v[0]
    }

    #[effects(io)]
    fn main() {
        let n = 3;
        print_i64(add(n, 2));
    }
}
"#;

const FIXED: &str = r#"#[summary("Rust habits")]
mod m {
    #[effects(pure)]
     fn add(x: i64, y: i64) -> i64 {
        let z: i64 = x + 1;
        z + y
    }

    #[effects(pure)]
    fn first(a: & [i64]) -> i64 {
        let v: [i64; 2] = [1, 2];
        v[0]
    }

    #[effects(io)]
    fn main() -> () {
        let n: i64 = 3;
        print_i64(add(n, 2));
    }
}
"#;

fn one(text: &str) -> Vec<(String, String)> {
    vec![("t.bmb".to_string(), text.to_string())]
}

#[test]
fn rust_habits_are_fixed_over_rounds() {
    let input = one(HABITS);
    assert!(!bmb::check_sources(&input).ok);
    let out = bmb::fix_sources(&input);
    assert_eq!(out.files[0].1, FIXED);
    assert!(out.ok, "{}", out.diagnostics);
    assert_eq!(out.diagnostics, "");
    assert!(bmb::check_sources(&out.files).ok);
    // `let z` gets its type only once `x` is `i64`: a second round.
    assert_eq!(out.rounds, 2);
    let mut codes: Vec<&str> = out.applied.iter().map(|a| a.code).collect();
    codes.sort();
    assert_eq!(
        codes,
        [
            "E0104", "E0201", "E0201", "E0202", "E0207", "E0208", "E0209", "E0402"
        ]
    );
    let report = out.report(&input);
    assert!(
        report.ends_with(
            "{\"summary\":true,\"rounds\":2,\"applied\":8,\"changed\":[\"t.bmb\"],\"remaining\":0,\"ok\":true}\n"
        ),
        "{report}"
    );
    // Deterministic.
    assert_eq!(bmb::fix_sources(&input).report(&input), report);
}

#[test]
fn fix_is_idempotent() {
    for text in [FIXED, HABITS] {
        let first = bmb::fix_sources(&one(text));
        let again = bmb::fix_sources(&first.files);
        assert_eq!(again.files, first.files);
        assert!(again.applied.is_empty());
        assert_eq!(again.rounds, 0);
        assert_eq!(
            again.report(&first.files),
            "{\"summary\":true,\"rounds\":0,\"applied\":0,\"changed\":[],\"remaining\":0,\"ok\":true}\n"
        );
    }
}

#[test]
fn non_mechanical_errors_remain() {
    // `return` has no mechanical fix; the suffix does.
    let text = "#[summary(\"x\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 {\n        return 5i64;\n    }\n}\n";
    let out = bmb::fix_sources(&one(text));
    assert!(!out.ok);
    assert!(out.files[0].1.contains("return 5;"), "{}", out.files[0].1);
    assert_eq!(out.applied.len(), 1);
    assert!(out.diagnostics.contains("\"code\":\"E0203\""));
    assert!(!out.diagnostics.contains("E0104"));
}

#[test]
fn fixes_apply_per_file_in_a_multi_file_program() {
    let files = vec![
        (
            "a.bmb".to_string(),
            "#[summary(\"a\")]\nmod a {\n    #[effects(pure)]\n    fn f(x: i32) -> i64 { x }\n}\n".to_string(),
        ),
        (
            "b.bmb".to_string(),
            "#[summary(\"b\")]\nmod b {\n    #[effects(io)]\n    fn main() -> () { print_i64(a::f(7i64)); }\n}\n".to_string(),
        ),
    ];
    let out = bmb::fix_sources(&files);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(out.files[0].1.contains("fn f(x: i64)"));
    assert!(out.files[1].1.contains("a::f(7)"));
    assert_eq!(out.changed(&files), ["a.bmb", "b.bmb"]);
}

#[test]
fn cli_reports_by_default_and_writes_only_with_write() {
    let dir = std::env::temp_dir().join(format!("bmb-fix-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("h.bmb");
    std::fs::write(&path, HABITS).unwrap();
    let bmb = env!("CARGO_BIN_EXE_bmb");

    let report = Command::new(bmb).arg("fix").arg(&path).output().unwrap();
    assert!(report.status.success());
    let stdout = String::from_utf8(report.stdout).unwrap();
    assert!(stdout.lines().last().unwrap().contains("\"applied\":8"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HABITS);

    let printed = Command::new(bmb)
        .args(["fix", "--print"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(printed.stdout).unwrap(), FIXED);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HABITS);

    let written = Command::new(bmb)
        .args(["fix", "--write"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(written.status.success());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), FIXED);

    let bad = dir.join("r.bmb");
    std::fs::write(
        &bad,
        "#[summary(\"x\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 {\n        return 5;\n    }\n}\n",
    )
    .unwrap();
    let failed = Command::new(bmb).arg("fix").arg(&bad).output().unwrap();
    assert_eq!(failed.status.code(), Some(1));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn literal_fixes_keep_the_value() {
    // `0x10` once lost its `x10` and became `0` (2R-02 finding).
    let text = "#[summary(\"t\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 { 0x10 + 0o17 + 0b101 + 5i64 }\n}\n";
    let out = bmb::fix_sources(&one(text));
    assert!(out.ok, "{}", out.diagnostics);
    assert!(
        out.files[0].1.contains("{ 16 + 15 + 5 + 5 }"),
        "{}",
        out.files[0].1
    );
    // An unknown suffix gets no mechanical fix.
    let odd = text.replace("5i64", "7abc");
    let out = bmb::fix_sources(&one(&odd));
    assert!(!out.ok);
    assert!(out.files[0].1.contains("7abc"));
}

/// The data type habits with a mechanical fix (ADR-0026 §5, backlog 2B-04d):
/// `bmb fix` turns them into a program that checks, in one round.
const DATA_HABITS: &str = r#"#[summary("habits with mechanical fixes")]
mod shapes {
    #[derive(Clone, Copy)]
    struct Point { x: i64, y: i64 }

    enum Shape { Dot, Square(i64), Rect(Point, Point) }

    impl Point {
        #[effects(pure)]
        fn sum(&self, k: i64) -> i64 {
            self.x + k
        }
    }

    #[effects(pure)]
    fn full_path(a: i64) -> Option<i64> {
        Option::Some(a)
    }

    #[effects(pure)]
    fn shorthand(x: i64, y: i64) -> Point {
        Point { x, y }
    }

    #[effects(pure)]
    fn update(p: Point) -> Point {
        Point { x: 1, ..p }
    }

    #[effects(pure)]
    fn wildcard(s: Shape) -> i64 {
        match s {
            Shape::Dot => 0,
            _ => 1,
        }
    }

    #[effects(pure)]
    fn if_let(o: Option<i64>) -> i64 {
        if let Some(v) = o { v } else { 0 }
    }

    #[effects(pure)]
    fn method(p: Point, q: &Point) -> i64 {
        p.sum(1) + q.sum(2)
    }

    #[effects(pure)]
    fn destructure(p: Point) -> i64 {
        let Point { x, y } = p;
        x + y
    }

    #[effects(pure)]
    fn deref_field(p: &mut Point) -> i64 {
        (*p).x = 1;
        (*p).y
    }
}
"#;

#[test]
fn data_type_habits_are_fixed_into_a_checked_program() {
    let input = one(DATA_HABITS);
    assert!(!bmb::check_sources(&input).ok);
    let out = bmb::fix_sources(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(bmb::check_sources(&out.files).ok);
    assert_eq!(out.rounds, 1);
    let mut codes: Vec<&str> = out.applied.iter().map(|a| a.code).collect();
    codes.sort();
    codes.dedup();
    assert_eq!(
        codes,
        [
            "E0220", "E0221", "E0222", "E0223", "E0224", "E0225", "E0227", "E0228", "E0230",
            "E0231"
        ]
    );
    let fixed = &out.files[0].1;
    for want in [
        "match o { None => { 0 }, Some(v) => { v } }",
        "match s { Shape::Dot => 0, Shape::Square(_) => 1, Shape::Rect(_, _) => 1 }",
        "Point::sum(&p, 1) + Point::sum(q, 2)",
        "let x: i64 = p.x; let y: i64 = p.y;",
        "Point { x: 1, y: p.y }",
        "fn sum(self: &Point, k: i64)",
    ] {
        assert!(fixed.contains(want), "{want}\n{fixed}");
    }
}

/// The conversion habits with a mechanical fix (ADR-0028 §8, backlog 2C-02):
/// each becomes `T::try_from(x).unwrap()`, which checks; the range is the
/// prover's (E0815), not the fix's.
const CONV_HABITS: &str = r#"#[summary("conversion habits")]
mod conv {
    #[effects(pure)]
    fn cast(x: u64, a: &[i64], k: i64) -> i64 {
        x as i64 + a.len() as i64 + a[k as usize] + (k + 1) as i64
    }

    #[effects(pure)]
    fn from(x: u64) -> i64 { i64::from(x) }

    #[effects(pure)]
    fn expects(x: u64) -> i64 { i64::try_from(x).expect("fits") }

    #[effects(pure)]
    fn bare(x: u64) -> i64 { i64::try_from(x) }

    #[effects(pure)]
    fn flag(b: bool) -> i64 { b as i64 }
}
"#;

#[test]
fn conversion_habits_are_fixed_into_a_checked_program() {
    let input = one(CONV_HABITS);
    assert!(!bmb::check_sources(&input).ok);
    let out = bmb::fix_sources(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(bmb::check_sources(&out.files).ok);
    let mut codes: Vec<&str> = out.applied.iter().map(|a| a.code).collect();
    codes.sort();
    codes.dedup();
    assert_eq!(codes, ["E0205", "E0212", "E0213", "E0412"]);
    let fixed = &out.files[0].1;
    for want in [
        "i64::try_from(x).unwrap() + i64::try_from(a.len()).unwrap() + a[usize::try_from(k).unwrap()] + i64::try_from(k + 1).unwrap()",
        "fn from(x: u64) -> i64 { i64::try_from(x).unwrap() }",
        "fn expects(x: u64) -> i64 { i64::try_from(x).unwrap() }",
        "fn bare(x: u64) -> i64 { i64::try_from(x).unwrap() }",
        "fn flag(b: bool) -> i64 { if b { 1 } else { 0 } }",
    ] {
        assert!(fixed.contains(want), "{want}\n{fixed}");
    }
}

/// `&v` / `&mut v` for a slice parameter (Rust's Deref coercion, ADR-0031
/// §7, backlog 2G-02) becomes `v.as_slice()` / `v.as_mut_slice()`.
#[test]
fn vec_coercions_are_fixed_into_a_checked_program() {
    let text = "#[summary(\"t\")]\nmod m {\n    #[effects(pure)]\n    fn read(a: &[i64]) -> usize { a.len() }\n\n    #[effects(pure)]\n    fn write(a: &mut [i64]) -> () {}\n\n    #[effects(pure)]\n    fn f(r: &Vec<i64>) -> usize {\n        let mut v: Vec<i64> = Vec::new();\n        write(&mut v);\n        read(&v) + read(r)\n    }\n}\n";
    let input = one(text);
    assert!(!bmb::check_sources(&input).ok);
    let out = bmb::fix_sources(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(bmb::check_sources(&out.files).ok);
    let codes: Vec<&str> = out.applied.iter().map(|a| a.code).collect();
    assert_eq!(codes, ["E0218", "E0218", "E0218"]);
    let fixed = &out.files[0].1;
    assert!(fixed.contains("write(v.as_mut_slice());"), "{fixed}");
    assert!(
        fixed.contains("read(v.as_slice()) + read(r.as_slice())"),
        "{fixed}"
    );
}
