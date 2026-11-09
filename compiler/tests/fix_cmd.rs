//! `bmb fix` (backlog 2D-04): applies every mechanical fix, re-checks and
//! repeats until none applies. Through the command line (backlog 4A-04):
//! `bmb fix --write` in a directory of its own, then the files and the
//! report it printed.

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

mod common;

use common::cli;
use common::json;

/// What `bmb fix --write` did to `files`: the files after it, its exit
/// status, the report's rounds, applied codes (in report order), remaining
/// diagnostics, changed files and the whole report.
struct Fixed {
    files: Vec<(String, String)>,
    ok: bool,
    rounds: i64,
    codes: Vec<String>,
    diagnostics: String,
    changed: Vec<String>,
    report: String,
}

fn fix(files: &[(String, String)]) -> Fixed {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "bmb-fix-cli-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(&dir)
        .args(["fix", "--write"])
        .args(files.iter().map(|(n, _)| n))
        .output()
        .unwrap();
    let after = files
        .iter()
        .map(|(n, _)| (n.clone(), fs::read_to_string(dir.join(n)).unwrap()))
        .collect();
    let _ = fs::remove_dir_all(&dir);
    let report = String::from_utf8(out.stdout).unwrap();
    let mut fixed = Fixed {
        files: after,
        ok: out.status.success(),
        rounds: -1,
        codes: Vec::new(),
        diagnostics: String::new(),
        changed: Vec::new(),
        report: report.clone(),
    };
    for line in report.lines() {
        let v = json::parse(line).unwrap();
        if v.get("summary").is_some() {
            fixed.rounds = v.get("rounds").and_then(json::Value::as_i64).unwrap();
            if let Some(json::Value::Array(xs)) = v.get("changed") {
                fixed.changed = xs.iter().map(|x| x.as_str().unwrap().to_string()).collect();
            }
        } else if v.get("round").is_some() {
            fixed.codes.push(
                v.get("code")
                    .and_then(json::Value::as_str)
                    .unwrap()
                    .to_string(),
            );
        } else {
            fixed.diagnostics.push_str(line);
            fixed.diagnostics.push('\n');
        }
    }
    fixed
}

fn checks(files: &[(String, String)]) -> bool {
    cli::run(&["check"], files).0
}

fn verifies(files: &[(String, String)]) -> bool {
    cli::run(&["verify"], files).0
}

const HABITS: &str = r#"#[summary("Rust habits")]
mod m {
    #[effects(pure)]
    pub fn add(x: i128, y: i64) -> i64 {
        let z = x + 1i64;
        z + y
    }

    #[effects(pure)]
    fn first(a: &'a [i64]) -> i64 {
        let v: [i64; 2] = [1, 2];
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
    fn first(a: &[i64]) -> i64 {
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
    assert!(!checks(&input));
    let out = fix(&input);
    assert_eq!(out.files[0].1, FIXED);
    assert!(out.ok, "{}", out.diagnostics);
    assert_eq!(out.diagnostics, "");
    assert!(checks(&out.files));
    // `let z` gets its type only once `x` is `i64`: a second round.
    assert_eq!(out.rounds, 2);
    let mut codes: Vec<&str> = out.codes.iter().map(String::as_str).collect();
    codes.sort();
    assert_eq!(
        codes,
        [
            "E0104", "E0201", "E0201", "E0207", "E0208", "E0209", "E0402"
        ]
    );
    let report = out.report.clone();
    assert!(
        report.ends_with(
            "{\"summary\":true,\"rounds\":2,\"applied\":7,\"changed\":[\"t.bmb\"],\"remaining\":0,\"ok\":true,\"written\":true}\n"
        ),
        "{report}"
    );
    // Deterministic.
    assert_eq!(fix(&input).report, report);
}

#[test]
fn fix_is_idempotent() {
    for text in [FIXED, HABITS] {
        let first = fix(&one(text));
        let again = fix(&first.files);
        assert_eq!(again.files, first.files);
        assert!(again.codes.is_empty());
        assert_eq!(again.rounds, 0);
        assert_eq!(
            again.report.clone(),
            "{\"summary\":true,\"rounds\":0,\"applied\":0,\"changed\":[],\"remaining\":0,\"ok\":true,\"written\":true}\n"
        );
    }
}

#[test]
fn non_mechanical_errors_remain() {
    // A `while` without `decreases` has no mechanical fix; the suffix does.
    let text = "#[summary(\"x\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 {\n        let mut i: i64 = 5i64;\n        while i > 0 {\n            i = i - 1;\n        }\n        i\n    }\n}\n";
    let out = fix(&one(text));
    assert!(!out.ok);
    assert!(
        out.files[0].1.contains("let mut i: i64 = 5;"),
        "{}",
        out.files[0].1
    );
    assert_eq!(out.codes.len(), 1);
    assert!(out.diagnostics.contains("\"code\":\"E0506\""));
    assert!(!out.diagnostics.contains("E0104"));
}

#[test]
fn fixes_apply_per_file_in_a_multi_file_program() {
    let files = vec![
        (
            "a.bmb".to_string(),
            "#[summary(\"a\")]\nmod a {\n    #[effects(pure)]\n    fn f(x: i128) -> i64 { x }\n}\n".to_string(),
        ),
        (
            "b.bmb".to_string(),
            "#[summary(\"b\")]\nmod b {\n    #[effects(io)]\n    fn main() -> () { print_i64(a::f(7i64)); }\n}\n".to_string(),
        ),
    ];
    let out = fix(&files);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(out.files[0].1.contains("fn f(x: i64)"));
    assert!(out.files[1].1.contains("a::f(7)"));
    assert_eq!(out.changed, ["a.bmb", "b.bmb"]);
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
    // `changed` lists the files either way; `written` says whether they
    // were (backlog 2E-05: a report read as a write).
    let summary = stdout.lines().last().unwrap();
    assert!(summary.contains("\"applied\":7"), "{summary}");
    assert!(summary.ends_with(",\"written\":false}"), "{summary}");
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
    assert!(
        String::from_utf8(written.stdout)
            .unwrap()
            .trim_end()
            .ends_with(",\"written\":true}")
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), FIXED);

    let bad = dir.join("r.bmb");
    std::fs::write(
        &bad,
        "#[summary(\"x\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 {\n        let mut i: i64 = 5;\n        while i > 0 {\n            i = i - 1;\n        }\n        i\n    }\n}\n",
    )
    .unwrap();
    let failed = Command::new(bmb).arg("fix").arg(&bad).output().unwrap();
    assert_eq!(failed.status.code(), Some(1));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn literal_fixes_keep_the_value() {
    // `0x10` once lost its `x10` and became `0` (2R-02 finding). Bases are
    // literals now (ADR-0048 §3); a suffix goes, the base and digits stay.
    let text = "#[summary(\"t\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 { 0x10 + 0o17 + 0b1_01 + 5i64 + 0xfi64 }\n}\n";
    let out = fix(&one(text));
    assert!(out.ok, "{}", out.diagnostics);
    assert!(
        out.files[0]
            .1
            .contains("{ 0x10 + 0o17 + 0b1_01 + 5 + 0xf }"),
        "{}",
        out.files[0].1
    );
    // An unknown suffix gets no mechanical fix.
    let odd = text.replace("5i64", "7abc");
    let out = fix(&one(&odd));
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
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(checks(&out.files));
    assert_eq!(out.rounds, 1);
    let mut codes: Vec<&str> = out.codes.iter().map(String::as_str).collect();
    codes.sort();
    codes.dedup();
    assert_eq!(
        codes,
        [
            "E0220", "E0221", "E0222", "E0223", "E0224", "E0225", "E0228", "E0230", "E0231"
        ]
    );
    let fixed = &out.files[0].1;
    for want in [
        "match o { None => { 0 }, Some(v) => { v } }",
        "match s { Shape::Dot => 0, Shape::Square(_) => 1, Shape::Rect(_, _) => 1 }",
        "p.sum(1) + q.sum(2)",
        "let x: i64 = p.x; let y: i64 = p.y;",
        "Point { x: 1, y: p.y }",
        "fn sum(self: &Point, k: i64)",
    ] {
        assert!(fixed.contains(want), "{want}\n{fixed}");
    }
}

/// The conversion habits with a mechanical fix (ADR-0028 §8, backlog 2C-02):
/// each becomes the canonical `x as T` (ADR-0095 A4: the checker's fixes
/// write the alias `T::try_from(x).unwrap()` and W0915 rewrites it), which
/// checks; the range is the prover's (E0815), not the fix's. `x as T`
/// itself is the canonical form and stays.
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
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(checks(&out.files));
    let mut codes: Vec<&str> = out.codes.iter().map(String::as_str).collect();
    codes.sort();
    codes.dedup();
    assert_eq!(codes, ["E0212", "E0213", "E0412", "W0915"]);
    let fixed = &out.files[0].1;
    for want in [
        "x as i64 + a.len() as i64 + a[k as usize] + (k + 1) as i64",
        "fn from(x: u64) -> i64 { x as i64 }",
        "fn expects(x: u64) -> i64 { x as i64 }",
        "fn bare(x: u64) -> i64 { x as i64 }",
        "fn flag(b: bool) -> i64 { if b { 1 } else { 0 } }",
    ] {
        assert!(fixed.contains(want), "{want}\n{fixed}");
    }
}

/// `&v` / `&mut v` for a slice parameter (Rust's Deref coercion, ADR-0031
/// §7, backlog 2G-02) becomes `v.as_slice()` / `v.as_mut_slice()`.
#[test]
fn vec_coercions_are_fixed_into_a_checked_program() {
    let text = "#[summary(\"t\")]\nmod m {\n    #[effects(pure)]\n    fn read(a: &[i64]) -> usize { a.len() }\n\n    #[effects(pure)]\n    fn write(a: &mut [i64]) -> () {}\n\n    #[effects(alloc)]\n    fn f(r: &Vec<i64>) -> usize {\n        let mut v: Vec<i64> = Vec::new();\n        write(&mut v);\n        read(&v) + read(r)\n    }\n}\n";
    let input = one(text);
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(checks(&out.files));
    let codes: Vec<&str> = out.codes.iter().map(String::as_str).collect();
    assert_eq!(codes, ["E0218", "E0218", "E0218"]);
    let fixed = &out.files[0].1;
    assert!(fixed.contains("write(v.as_mut_slice());"), "{fixed}");
    assert!(
        fixed.contains("read(v.as_slice()) + read(r.as_slice())"),
        "{fixed}"
    );
}

/// Text habits (ADR-0044 §5): `"..."` becomes `b"..."` (non-ASCII as
/// `\xHH`), `'a'` becomes `b'a'`, `char` becomes `u8`.
#[test]
fn text_habits_are_fixed_into_a_checked_program() {
    let text = "#[summary(\"t\")]\nmod m {\n    #[effects(io)]\n    fn f(c: char) -> () {\n        print_bytes(\"caf\u{e9} \\\"x\\\"\\n\");\n        let n: u8 = 'a';\n    }\n}\n";
    let input = one(text);
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(checks(&out.files));
    let fixed = &out.files[0].1;
    // `char` is a type (ADR-0050); a string literal for bytes and a
    // character for a byte get their byte forms.
    for want in [
        "fn f(c: char) -> ()",
        "print_bytes(b\"caf\\xc3\\xa9 \\\"x\\\"\\n\");",
        "let n: u8 = b'a';",
    ] {
        assert!(fixed.contains(want), "{want}\n{fixed}");
    }
}

/// A `&T` given where a struct `T` is expected gets `*e` (backlog 2B-16):
/// `moved(old(self), self)` in an `ensures`, a `let` from a reference.
#[test]
fn reference_where_a_value_is_expected_is_fixed() {
    let text = "#[summary(\"t\")]\nmod d {\n    struct P { x: i64 }\n\n    spec fn small(p: P) -> bool { p.x < 100 }\n\n    spec fn moved(p: P, q: P) -> bool { p.x <= q.x }\n\n    impl P {\n        #[effects(pure)]\n        #[uses(small, moved)]\n        fn bump(self: &mut P) -> ()\n            requires small(*self),\n            ensures moved(old(self), self),\n        {\n            self.x = self.x + 1;\n        }\n    }\n\n    #[effects(pure)]\n    fn copy(r: &P) -> i64 {\n        let p: P = r;\n        p.x\n    }\n}\n";
    let input = one(text);
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(verifies(&out.files));
    let fixed = &out.files[0].1;
    assert!(
        fixed.contains("ensures moved(*old(self), *self),"),
        "{fixed}"
    );
    assert!(fixed.contains("let p: P = *r;"), "{fixed}");
}

#[test]
fn use_items_are_rewritten_to_full_paths() {
    // Backlog 2V-15: `use` is deleted and every name it imports is written
    // in full; the result checks and proves.
    let text = "#[summary(\"g\")]\nmod geo {\n    struct Point { x: i64, y: i64 }\n\n    #[effects(pure)]\n    fn origin() -> Point {\n        Point { x: 0, y: 0 }\n    }\n}\n\n#[summary(\"a\")]\nmod app {\n    use geo::Point;\n    use geo::origin;\n\n    #[effects(pure)]\n    fn x_of(p: Point) -> i64 {\n        p.x\n    }\n\n    #[effects(pure)]\n    fn go() -> i64 {\n        let p: Point = origin();\n        x_of(p)\n    }\n}\n";
    let input = one(text);
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(verifies(&out.files));
    let fixed = &out.files[0].1;
    assert!(!fixed.contains("use "), "{fixed}");
    assert!(fixed.contains("fn x_of(p: geo::Point) -> i64 {"), "{fixed}");
    assert!(
        fixed.contains("let p: geo::Point = geo::origin();"),
        "{fixed}"
    );
    assert!(
        fixed.contains("mod app {\n\n    #[effects(pure)]"),
        "{fixed}"
    );
}

#[test]
fn omitted_type_arguments_are_written_out() {
    // ADR-0046 §3: a generic call names its type arguments; E0237 infers
    // them from the arguments and the expected result and writes them, for
    // the function's and for an `impl`'s parameters.
    let text = "#[summary(\"g\")]\nmod g {\n    struct Cell<T: Copy> { v: T }\n\n    impl<T: Copy> Cell<T> {\n        #[effects(pure)]\n        fn new(v: T) -> Cell<T> {\n            Cell { v: v }\n        }\n    }\n\n    #[effects(pure)]\n    fn pick<A: Copy, B: Copy>(a: A, b: B) -> B {\n        b\n    }\n\n    #[effects(pure)]\n    fn go(x: u64) -> bool {\n        let c: Cell<bool> = Cell::new(true);\n        let y: u64 = pick(c, x);\n        pick(y, c.v)\n    }\n}\n";
    let input = one(text);
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(verifies(&out.files));
    let fixed = &out.files[0].1;
    assert!(fixed.contains("Cell::<bool>::new(true)"), "{fixed}");
    assert!(fixed.contains("pick::<Cell<bool>, u64>(c, x)"), "{fixed}");
    assert!(fixed.contains("pick::<u64, bool>(y, c.v)"), "{fixed}");
}

#[test]
fn where_clauses_move_to_the_parameters() {
    // Backlog 2T-03: a `where` clause (E0240) is rewritten to bounds next
    // to the type parameters, on a function, an `impl` and a type.
    let text = "#[summary(\"w\")]\nmod w {\n    trait Shape {\n        #[effects(pure)]\n        fn area(self: &Self) -> i64;\n    }\n\n    struct Cell<T> where T: Copy { v: T }\n\n    impl<T> Shape for Cell<T>\n    where\n        T: Copy + Shape,\n    {\n        #[effects(pure)]\n        fn area(self: &Cell<T>) -> i64 {\n            Shape::area(&self.v)\n        }\n    }\n\n    #[effects(pure)]\n    fn least<T: Copy, U>(a: T, b: T, u: U) -> T\n        where T: Ord, U: Copy\n    {\n        if a <= b { a } else { b }\n    }\n}\n";
    let input = one(text);
    assert!(!checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(verifies(&out.files));
    let fixed = &out.files[0].1;
    assert!(!fixed.contains("where"), "{fixed}");
    assert!(fixed.contains("struct Cell<T: Copy>"), "{fixed}");
    assert!(
        fixed.contains("impl<T: Copy + Shape> Shape for Cell<T>"),
        "{fixed}"
    );
    assert!(
        fixed.contains("fn least<T: Copy + Ord, U: Copy>(a: T, b: T, u: U) -> T"),
        "{fixed}"
    );
}

const COUNTING: &str = r#"#[summary("counting loops")]
mod c {
    spec fn small(a: &[u64]) -> bool {
        a.len() < 100 && (forall|k: usize| k < a.len() ==> #[trigger] a[k] < 100)
    }

    #[effects(pure)]
    fn sum(a: &[u64]) -> u64
        requires small(a),
    {
        let mut s: u64 = 0;
        // scan
        let mut i: usize = 0;
        'all: while i < a.len()
            invariant i <= a.len(), s <= 100 * i,
            decreases a.len() - i,
        {
            s = s + a[i];
            i = i + 1;
        }
        s
    }

    // `i` is read after the loop, `j` steps before the end, `k` meets a
    // `continue`, `m` runs to a bound the body assigns: none is a `for`.
    #[effects(pure)]
    fn others(n: u64) -> u64 {
        let mut i: u64 = 0;
        while i < n
            invariant i <= n,
            decreases n - i,
        {
            i = i + 1;
        }
        let mut j: u64 = 0;
        while j < n
            invariant j <= n,
            decreases n - j,
        {
            j = j + 1;
            let x: u64 = j;
        }
        let mut k: u64 = 0;
        while k < n
            invariant k <= n,
            decreases n - k,
        {
            if k == 3 {
                k = k + 1;
                continue;
            }
            k = k + 1;
        }
        let mut hi: u64 = n;
        let mut m: u64 = 0;
        while m < hi
            invariant m <= hi && hi == n,
            decreases hi - m,
        {
            hi = n;
            m = m + 1;
        }
        i
    }
}
"#;

/// W0907 (ADR-0057, backlog 2K-04): a counting `while` becomes a range
/// `for`, comments and invariants kept; the others stay.
#[test]
fn counting_while_becomes_a_range_for() {
    let input = one(COUNTING);
    assert!(verifies(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    let codes: Vec<&str> = out.codes.iter().map(String::as_str).collect();
    assert_eq!(codes, ["W0907"]);
    let text = &out.files[0].1;
    assert!(
        text.contains(
            "        let mut s: u64 = 0;\n        // scan\n        'all: for i in 0..a.len()\n            invariant i <= a.len(), s <= 100 * i,\n        {\n            s = s + a[i];\n        }\n        s\n"
        ),
        "{text}"
    );
    assert_eq!(text.matches("while").count(), 4, "{text}");
    let after = one(text);
    assert!(verifies(&after));
    // A fixed program has nothing left to rewrite.
    assert!(fix(&after).codes.is_empty());
}

/// The fixes leave the normal form around them (backlog 2R-04e): removing
/// `pub`, `mut` and a lifetime takes the space after it, and the effects
/// attribute goes on a line of its own at the function's indentation.
#[test]
fn fixed_habits_are_in_normal_form() {
    let input = one(r#"#[summary("habits")]
mod m {
    pub fn add(mut x: i64, y: i64) -> i64 {
        x + y
    }

    #[effects(pure)]
    fn first<'a>(a: &'a [i64]) -> i64 {
        0
    }
}
"#);
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    let text = &out.files[0].1;
    assert!(cli::run(&["fmt", "--check"], &out.files).0, "{text}");
    assert!(
        text.contains("    #[effects(pure)]\n    fn add(x: i64, y: i64) -> i64 {"),
        "{text}"
    );
    assert!(text.contains("fn first(a: &[i64]) -> i64"), "{text}");
}

/// E0511 (backlog 2R-04g): a quantifier over one variable gets its first
/// `name[var]` term as the trigger, the one the written corpora choose
/// (137 of 156); with two variables the fix only advises.
#[test]
fn a_missing_trigger_marks_the_first_indexed_term() {
    let input = one(r#"#[summary("triggers")]
mod t {
    spec fn pos(a: &[i64]) -> bool {
        forall |i: usize| i < a.len() ==> a[i] > 0
    }

    spec fn sorted(a: &[i64]) -> bool {
        forall |i: usize, j: usize| i < j && j < a.len() ==> a[i] <= a[j]
    }
}
"#);
    let out = fix(&input);
    let text = &out.files[0].1;
    assert!(
        text.contains("i < a.len() ==> #[trigger] a[i] > 0"),
        "{text}"
    );
    assert!(!text.contains("#[trigger] a[i] <="), "{text}");
    assert!(out.diagnostics.contains("E0511"), "{}", out.diagnostics);
}

/// The input aliases of ADR-0095 (B5, B6) and backlog 2N-07: a repeated
/// clause keyword, loop clauses at the start of the body and compound
/// assignments are read as the canonical syntax tree. So the program
/// checks, its nameless hashes and its proof verdicts are the canonical
/// program's, `bmb fmt` writes the canonical text, `bmb lint` reports each
/// alias and `bmb fix` rewrites them into that normal form.
const ALIASES: &str = r#"#[summary("aliases")]
mod a {
    #[effects(pure)]
    fn f(n: u64) -> u64
        requires small(n),
        requires tiny(n),
    {
        let mut i: u64 = 0;
        let mut h: u64 = 0;
        while i < n {
            invariant i <= n,
            invariant h <= i,
            decreases n - i,
            i += 1;
            h += 1;
        }
        let mut s: u64 = 0;
        loop
            invariant s <= n,
            invariant tiny(n),
            decreases n - s,
        {
            if s >= n {
                break;
            }
            s += 1 + 0;
        }
        h ^= 1;
        h
    }

    spec fn small(n: u64) -> bool {
        n < 1000
    }

    spec fn tiny(n: u64) -> bool {
        n < 100
    }
}
"#;

const CANONICAL: &str = r#"#[summary("aliases")]
mod a {
    #[effects(pure)]
    fn f(n: u64) -> u64
        requires small(n), tiny(n),
    {
        let mut i: u64 = 0;
        let mut h: u64 = 0;
        while i < n
            invariant i <= n, h <= i,
            decreases n - i,
        {
            i = i + 1;
            h = h + 1;
        }
        let mut s: u64 = 0;
        loop
            invariant s <= n, tiny(n),
            decreases n - s,
        {
            if s >= n {
                break;
            }
            s = s + (1 + 0);
        }
        h = h ^ 1;
        h
    }

    spec fn small(n: u64) -> bool {
        n < 1000
    }

    spec fn tiny(n: u64) -> bool {
        n < 100
    }
}
"#;

#[test]
fn input_aliases_are_the_canonical_program() {
    let alias = one(ALIASES);
    let canon = one(CANONICAL);
    assert!(checks(&alias));
    assert!(cli::run(&["fmt", "--check"], &canon).0);
    let (_, fmt) = cli::run(&["fmt"], &alias);
    assert_eq!(fmt, CANONICAL);
    assert_eq!(cli::run(&["hash"], &alias).1, cli::run(&["hash"], &canon).1);
    let (ok_a, va) = cli::run(&["verify"], &alias);
    let (ok_c, vc) = cli::run(&["verify"], &canon);
    assert_eq!((ok_a, &va), (ok_c, &vc));
    let (_, lint) = cli::run(&["lint", "--select", "W0912,W0913,W0914"], &alias);
    let count = |c: &str| lint.matches(&format!("\"code\":\"{c}\"")).count();
    assert_eq!(
        (count("W0912"), count("W0913"), count("W0914")),
        (3, 1, 4),
        "{lint}"
    );
    let out = fix(&alias);
    assert!(out.ok, "{}", out.diagnostics);
    assert_eq!(out.files[0].1, CANONICAL);
}

/// A compound assignment names its place twice in the canonical form, so
/// a place that calls a function is not taken (Rust evaluates it once).
#[test]
fn a_compound_assignment_to_a_computed_place_is_refused() {
    let input = one(r#"#[summary("c")]
mod c {
    #[effects(pure)]
    fn at(i: usize) -> usize {
        i
    }

    #[effects(pure)]
    fn f(a: &mut [u64]) {
        a[at(0)] += 1;
    }
}
"#);
    let (ok, out) = cli::run(&["check"], &input);
    assert!(!ok);
    assert!(
        out.contains("compound assignment `+=` to a place that calls a function"),
        "{out}"
    );
}

/// W0915 (ADR-0095 A4): `T::try_from(x).unwrap()` becomes `x as T`, in
/// parentheses where the cast binds looser than its place (the operand of
/// `-`, the left of `<`), and its operand in parentheses when it binds
/// looser than the cast. The result is the normal form.
#[test]
fn conversions_are_written_as_casts_in_their_context() {
    let input = one(r#"#[summary("casts")]
mod c {
    #[effects(pure)]
    fn f(x: u32, y: i64, a: &[i64], k: u8) -> bool
        requires a.len() > 300,
    {
        let n: i64 = -i64::try_from(x).unwrap();
        let s: i64 = i64::try_from(x + 1).unwrap() * 2;
        let t: i64 = a[usize::try_from(k).unwrap()];
        i64::try_from(x).unwrap() < y && n < s + t
    }
}
"#);
    assert!(checks(&input));
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    let text = &out.files[0].1;
    for want in [
        "let n: i64 = -(x as i64);",
        "let s: i64 = (x + 1) as i64 * 2;",
        "let t: i64 = a[k as usize];",
        "(x as i64) < y && n < s + t",
    ] {
        assert!(text.contains(want), "{want}\n{text}");
    }
    assert!(cli::run(&["fmt", "--check"], &out.files).0, "{text}");
    assert_eq!(
        cli::run(&["hash"], &input).1,
        cli::run(&["hash"], &out.files).1
    );
}

/// W0916 (ADR-0095 A1): a `#[uses]` entry the default context (rule r2c)
/// reveals already is left out; an attribute with nothing left goes. The
/// proofs are the same without them.
#[test]
fn uses_entries_of_the_default_context_are_removed() {
    let input = one(r#"#[summary("u")]
mod u {
    #[effects(pure)]
    #[uses(double, small)]
    fn make(n: u64) -> (r: u64)
        requires small(n),
        ensures double(n, r),
    {
        n + n
    }

    #[effects(pure)]
    #[uses(double, extra)]
    fn user(n: u64) -> u64
        requires n < 10,
    {
        let d: u64 = make(n);
        d + 1
    }

    spec fn double(n: u64, r: u64) -> bool {
        r == n * 2
    }

    spec fn extra(n: u64) -> bool {
        n < 5
    }

    spec fn small(n: u64) -> bool {
        n < 1000
    }
}
"#);
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    let text = &out.files[0].1;
    assert!(
        text.contains("    #[effects(pure)]\n    fn make(n: u64)"),
        "{text}"
    );
    assert!(
        text.contains("    #[uses(extra)]\n    fn user(n: u64)"),
        "{text}"
    );
    assert_eq!(out.codes, ["W0916", "W0916"]);
    let (ok, v) = cli::run(&["verify"], &out.files);
    assert!(ok, "{v}");
}

/// E0201's fix for `let x = 0;` and `let v = Vec::new();` takes the type
/// from the local's first use (backlog 2N-12): an index is `usize`, an
/// argument its parameter's type, a `Vec`'s first `push` its element. The
/// fixed program checks (before, `i64` and no fix left E0401 and E0201).
#[test]
fn untyped_lets_take_the_type_of_their_first_use() {
    let input = one(r#"#[summary("e")]
mod e {
    #[effects(pure)]
    fn sum(a: &[u64]) -> u64 {
        let mut i = 0;
        let mut s: u64 = 0;
        while i < a.len()
            invariant i <= a.len(),
            decreases a.len() - i,
        {
            s = s.wrapping_add(a[i]);
            i = i + 1;
        }
        s
    }

    #[effects(alloc)]
    fn build(n: u32) -> Vec<u32> {
        let mut v = Vec::new();
        let k = 3;
        v.push(n);
        take(k);
        v
    }

    #[effects(pure)]
    fn take(x: u8) -> u8 {
        x
    }

    struct Bag<T> { items: Vec<T>, n: u64 }

    #[effects(pure)]
    fn id<T>(x: T) -> T { x }

    #[effects(alloc)]
    fn passed() -> usize {
        let v = Vec::new();
        let w: Vec<i16> = id::<Vec<i16>>(v);
        w.len()
    }

    #[effects(alloc)]
    fn bag<T>(n: u64) -> Bag<T> {
        let items = Vec::new();
        Bag { items: items, n: n }
    }
}
"#);
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    let text = &out.files[0].1;
    for want in [
        "let mut i: usize = 0;",
        "let mut v: Vec<u32> = Vec::new();",
        "let k: u8 = 3;",
        // Through a type parameter (2N-13): the call's type argument, the
        // struct expression's instance.
        "let v: Vec<i16> = Vec::new();",
        "let items: Vec<T> = Vec::new();",
    ] {
        assert!(text.contains(want), "{want}\n{text}");
    }
    assert!(checks(&out.files));
}

/// A widening conversion in a specification is an input alias (ADR-0099,
/// backlog 2N-08): W0917, and `fix` writes the value, in parentheses where
/// the context needs them.
#[test]
fn widening_conversions_in_specifications_are_removed() {
    let input = one(r#"#[summary("w")]
mod w {
    spec fn byte_at(msg: &[u8], i: usize) -> u64 {
        if i < msg.len() { msg[i] as u64 } else { 256 }
    }

    spec fn twice(x: u8) -> u32 {
        (x + 1) as u32 * 2
    }
}
"#);
    let out = fix(&input);
    assert!(out.ok, "{}", out.diagnostics);
    let text = &out.files[0].1;
    for want in ["if i < msg.len() { msg[i] } else { 256 }", "(x + 1) * 2"] {
        assert!(text.contains(want), "{want}\n{text}");
    }
    assert!(!text.contains(" as "), "{text}");
    assert!(checks(&out.files));
}
