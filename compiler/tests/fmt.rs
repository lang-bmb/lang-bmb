//! Normal form and definition hashes (ADR-0019, backlog 1D-02, 1D-03).
//! Through the CLI (backlog 4A-04): `bmb fmt`, `bmb verify`, `bmb hash`.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::cli::run as bmb;

/// `bmb verify` of `files`: success and output.
fn verify(files: &[(String, String)]) -> (bool, String) {
    bmb(&["verify"], files)
}

fn corpus() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut v = Vec::new();
    for dir in ["check/ok", "verify/pass", "verify/fail", "run"] {
        for e in fs::read_dir(root.join(dir)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "bmb") {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

fn one(name: &str, text: &str) -> Vec<(String, String)> {
    vec![(name.to_string(), text.to_string())]
}

fn normal(text: &str) -> String {
    let (ok, out) = bmb(&["fmt"], &one("x.bmb", text));
    assert!(ok, "{out}");
    out
}

fn statuses(out: &str) -> Vec<String> {
    out.lines()
        .filter(|l| l.starts_with("{\"def\""))
        .map(String::from)
        .collect()
}

#[test]
fn formatting_is_idempotent() {
    for p in corpus() {
        let once = normal(&fs::read_to_string(&p).unwrap());
        assert_eq!(normal(&once), once, "{}", p.display());
    }
}

#[test]
fn formatting_preserves_proof_results() {
    for p in corpus() {
        let text = fs::read_to_string(&p).unwrap();
        let before = verify(&one("x.bmb", &text)).1;
        let after = verify(&one("x.bmb", &normal(&text))).1;
        assert_eq!(statuses(&before), statuses(&after), "{}", p.display());
    }
}

fn hashes(files: &[(String, String)]) -> Vec<String> {
    let (ok, out) = bmb(&["hash"], files);
    assert!(ok, "{out}");
    let mut v: Vec<String> = out.lines().map(String::from).collect();
    v.sort();
    v
}

const PROG: &str = r#"
#[summary("해시 예제")]
mod h {
    spec fn pos(x: i64) -> bool { x > 0 }

    #[effects(pure)]
    #[uses(pos)]
    fn dec(x: i64) -> i64
        requires pos(x),
    {
        x - 1
    }
}

#[summary("다른 단위")]
mod g {
    #[effects(pure)]
    fn id(x: i64) -> i64 { x }
}
"#;

#[test]
fn hashes_ignore_layout_comments_and_order() {
    let base = hashes(&one("a.bmb", PROG));
    // Normal form, comments, blank lines.
    assert_eq!(hashes(&one("a.bmb", &normal(PROG))), base);
    let commented = PROG.replace("x - 1", "// note\n        x   -   1 // trailing");
    assert_eq!(hashes(&one("a.bmb", &commented)), base);
    // Item order inside a unit and unit order.
    let (h_part, g_part) = PROG.split_at(PROG.find("#[summary(\"다른").unwrap());
    let reordered = format!("{g_part}\n{h_part}");
    assert_eq!(hashes(&one("a.bmb", &reordered)), base);
    // Split across files.
    let files = vec![
        ("one.bmb".to_string(), h_part.to_string()),
        ("two.bmb".to_string(), g_part.to_string()),
    ];
    assert_eq!(hashes(&files), base);
}

fn field(line: &str, key: &str) -> String {
    let k = format!("\"{key}\":\"");
    let i = line.find(&k).unwrap() + k.len();
    line[i..i + line[i..].find('"').unwrap()].to_string()
}

fn def_line(files: &[(String, String)], def: &str) -> String {
    hashes(files)
        .into_iter()
        .find(|l| l.contains(&format!("\"def\":\"{def}\"")))
        .unwrap()
}

#[test]
fn body_change_keeps_iface_contract_change_moves_both() {
    let a = def_line(&one("a.bmb", PROG), "h::dec");
    let body_changed = PROG.replace("x - 1", "x - 1 + 0");
    let b = def_line(&one("a.bmb", &body_changed), "h::dec");
    assert_ne!(field(&a, "body"), field(&b, "body"));
    assert_eq!(field(&a, "iface"), field(&b, "iface"));

    let contract_changed = PROG.replace("requires pos(x),", "requires pos(x), pos(x),");
    let c = def_line(&one("a.bmb", &contract_changed), "h::dec");
    assert_ne!(field(&a, "iface"), field(&c, "iface"));
}

#[test]
fn identity_follows_what_names_resolve_to_not_the_names() {
    // `pos` resolves to `h::pos` here and to `k::pos` below. Both are the
    // same definition, so `dec` is too (ADR-0059 §1: references by id).
    let other = PROG.replace("mod h {", "mod k {");
    let a = def_line(&one("a.bmb", PROG), "h::dec");
    let b = def_line(&one("a.bmb", &other), "k::dec");
    for f in ["id", "body", "iface"] {
        assert_eq!(field(&a, f), field(&b, f), "{f}");
    }
    // Another `pos` makes another `dec`, though its text is the same.
    let changed = other.replace("x > 0", "x > 1");
    let c = def_line(&one("a.bmb", &changed), "k::dec");
    for f in ["id", "body", "iface"] {
        assert_ne!(field(&a, f), field(&c, f), "{f}");
    }
}

#[test]
fn renamed_definition_keeps_its_body_hash() {
    let renamed = PROG.replace("fn id(", "fn ident(");
    let a = def_line(&one("a.bmb", PROG), "g::id");
    let b = def_line(&one("a.bmb", &renamed), "g::ident");
    assert_eq!(field(&a, "body"), field(&b, "body"));
}

/// The data type surface (ADR-0026, backlog 2B-03) parses and prints in
/// normal form before the checker accepts it: types by name, each followed by
/// its `impl` (functions by name), then functions; multi-line `match`; a
/// condition holding a struct literal in parentheses (as Rust requires).
/// A definition's summary (ADR-0117, backlog 3P-08): `///` is its input
/// form, the normal form writes the attribute first, and no hash reads it.
#[test]
fn summaries_are_kept_by_the_normal_form_and_read_by_no_hash() {
    let body = "    #[effects(pure)]\n    fn inc(x: i64) -> (y: i64)\n        requires x < 100,\n        ensures y == x + 1,\n    {\n        x + 1\n    }\n";
    let plain = format!("#[summary(\"s\")]\nmod u {{\n    const K: i64 = 1;\n\n{body}}}\n");
    let doc = format!(
        "#[summary(\"s\")]\nmod u {{\n    /// The\n    /// limit.\n    const K: i64 = 1;\n\n    /// Adds one.\n{body}}}\n"
    );
    let attr = format!(
        "#[summary(\"s\")]\nmod u {{\n    #[summary(\"The limit.\")]\n    const K: i64 = 1;\n\n    #[summary(\"Adds one.\")]\n{body}}}\n"
    );
    assert_eq!(normal(&doc), attr);
    assert_eq!(normal(&attr), attr);
    assert_eq!(hashes(&one("x.bmb", &doc)), hashes(&one("x.bmb", &plain)));
    let other = attr.replace("Adds one.", "Something else.");
    assert_eq!(hashes(&one("x.bmb", &other)), hashes(&one("x.bmb", &plain)));
}

#[test]
fn data_types_have_a_normal_form() {
    const SRC: &str = r#"#[summary("2B surface")]
mod geo {
    impl Point {
        #[effects(pure)]
        fn norm1(self: &Point) -> i64 {
            let s: Shape = Shape::Dot;
            match s {
                Shape::Dot => 0,
                Shape::Rect(w, _) => w,
            }
        }

        #[effects(pure)]
        fn at(self: &mut Point, x: i64) -> () {
            self.x = x;
            *self = Point { x: x, y: self.y };
            if *self == Point { x: 0, y: 0 } { self.y = 1; }
        }
    }

    #[effects(pure)]
    fn main_like(o: Option<i64>) -> Result<i64, Why> {
        let r: i64 = match o { Some(v) => v, None => 0 };
        Ok(r)
    }

    enum Why { Bad, Worse(i64, bool) }

    struct Point { x: i64, y: i64 }
}
"#;
    const NORMAL: &str = r#"#[summary("2B surface")]
mod geo {
    struct Point { x: i64, y: i64 }

    impl Point {
        #[effects(pure)]
        fn at(self: &mut Point, x: i64) -> () {
            self.x = x;
            *self = Point { x: x, y: self.y };
            if (*self == Point { x: 0, y: 0 }) {
                self.y = 1;
            }
        }

        #[effects(pure)]
        fn norm1(self: &Point) -> i64 {
            let s: Shape = Shape::Dot;
            match s {
                Shape::Dot => 0,
                Shape::Rect(w, _) => w,
            }
        }
    }

    enum Why { Bad, Worse(i64, bool) }

    #[effects(pure)]
    fn main_like(o: Option<i64>) -> Result<i64, Why> {
        let r: i64 = match o {
            Some(v) => v,
            None => 0,
        };
        Ok(r)
    }
}
"#;
    // `bmb dump parse`: the parsed program in normal form (backlog 4A-03).
    let parse = |name: &str, text: &str| bmb(&["dump", "parse"], &one(name, text));
    assert_eq!(parse("x.bmb", SRC), (true, NORMAL.to_string()));
    assert_eq!(parse("x.bmb", NORMAL), (true, NORMAL.to_string()));
    // The checker rejects this sample (unknown `Shape`, values not checked yet).
    assert!(!bmb(&["fmt"], &one("x.bmb", SRC)).0);
    // A wildcard arm parses; the checker rejects it with the arms it stands
    // for (ADR-0026: one arm per variant).
    let wild = "#[summary(\"w\")]\nmod w {\n    #[effects(pure)]\n    fn f(o: Option<i64>) -> i64 { match o { Some(v) => v, _ => 0 } }\n}\n";
    assert!(parse("w.bmb", wild).0);
    let (ok, err) = bmb(&["fmt"], &one("w.bmb", wild));
    assert!(
        !ok && err.contains("E0224") && err.contains("None => 0"),
        "{err}"
    );
}

/// A literal keeps its base; separators, leading zeros and the case of hex
/// digits are not part of the normal form (ADR-0048 §3, ADR-0019 §1).
/// Bit operators print with Rust's precedence: only the parentheses it
/// needs (backlog 2N-02).
#[test]
fn literal_bases_and_bit_operators_have_one_normal_form() {
    let text = "#[summary(\"b\")]\nmod b {\n    #[effects(pure)]\n    fn f(x: u32, k: u32) -> bool {\n        let m: u32 = 0xFF_00 | 0o0_17 | 0b0000_0101 | 1_000;\n        let n: u32 = (x & m) ^ ((x << 2) | (x >> (k + 1)));\n        let p: u32 = ((x | m) & !m) << (k - 1);\n        (m & n) == p & (n | 1 ^ 2)\n    }\n}\n";
    let out = normal(text);
    assert!(
        out.contains("let m: u32 = 0xff00 | 0o17 | 0b101 | 1000;"),
        "{out}"
    );
    assert!(
        out.contains("let n: u32 = x & m ^ (x << 2 | x >> k + 1);"),
        "{out}"
    );
    assert!(
        out.contains("let p: u32 = ((x | m) & !m) << k - 1;"),
        "{out}"
    );
    assert!(out.contains("m & n == p & (n | 1 ^ 2)"), "{out}");
    assert_eq!(normal(&out), out);
    let before = verify(&one("x.bmb", text));
    let after = verify(&one("x.bmb", &out));
    assert_eq!(statuses(&before.1), statuses(&after.1));
}

/// `assert(e) by (bit_vector);` has one normal form (backlog 2N-03), and a
/// change of the prover clause changes the definition's hash.
#[test]
fn bit_vector_assertions_have_one_normal_form() {
    let text = "#[summary(\"b\")]\nmod b {\n    #[effects(pure)]\n    fn f(x: u32) -> u32 {\n        assert((x ^ x)==0)by(bit_vector) ;\n        proof { assert(x.rotate_left(3).rotate_right(3)==x)   by  ( bit_vector ); }\n        x.count_ones()\n    }\n}\n";
    let out = normal(text);
    assert!(
        out.contains("        assert(x ^ x == 0) by (bit_vector);\n"),
        "{out}"
    );
    assert!(
        out.contains("assert(x.rotate_left(3).rotate_right(3) == x) by (bit_vector);"),
        "{out}"
    );
    assert_eq!(normal(&out), out);
    let before = verify(&one("x.bmb", text));
    let after = verify(&one("x.bmb", &out));
    assert_eq!(statuses(&before.1), statuses(&after.1));
    assert!(before.0, "{}", before.1);
    let plain = out.replace(" by (bit_vector);", ";");
    assert_ne!(
        def_line(&one("x.bmb", &out), "b::f"),
        def_line(&one("x.bmb", &plain), "b::f")
    );
}

/// A float literal's normal form is the shortest text that reads back as
/// the same `f64` (ADR-0061 §2, Rust's `{:?}` with a point in the
/// mantissa): `0.10`, `1_0.0e-2` and `0.1` are one text and one hash.
#[test]
fn float_literals_have_one_normal_form() {
    let prog = |lit: &str| {
        format!(
            "#[summary(\"f\")]\nmod f {{\n    #[effects(pure)]\n    fn g() -> f64 {{\n        {lit} + 2.5E-3 * 1.0e16 - 100000000000000000.0 + 6.02e23 * 1.0e-9\n    }}\n}}\n"
        )
    };
    let out = normal(&prog("0.10"));
    assert!(
        out.contains("0.1 + 0.0025 * 1.0e16 - 1.0e17 + 6.02e23 * 1.0e-9"),
        "{out}"
    );
    assert_eq!(normal(&out), out);
    let h = |lit: &str| hashes(&one("x.bmb", &prog(lit)));
    assert_eq!(h("0.10"), h("1_0.0e-2"));
    assert_eq!(h("0.10"), h("0.1"));
    assert_ne!(h("0.1"), h("0.2"));
    // Every `f64` (bit patterns of each class) reads back from its text:
    // the lexer gives each written literal and its normal text the same
    // bits (`bmb dump tokens`), and that text is Rust's value.
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    let mut bits: Vec<u64> = vec![1, 0x000f_ffff_ffff_ffff, 0x0010_0000_0000_0000];
    bits.extend([0x7fef_ffff_ffff_ffff, 0x3ff0_0000_0000_0001]);
    for _ in 0..2000 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        bits.push(seed & 0x7fff_ffff_ffff_ffff);
    }
    bits.retain(|b| f64::from_bits(*b).is_finite());
    // Rust's shortest text, with a point in the mantissa.
    let written: Vec<String> = bits
        .iter()
        .map(|b| {
            let s = format!("{:?}", f64::from_bits(*b));
            match s.split_once('e') {
                Some((m, e)) if !m.contains('.') => format!("{m}.0e{e}"),
                _ => s,
            }
        })
        .collect();
    let (ok, toks) = bmb(&["dump", "tokens"], &one("t.bmb", &written.join(" ")));
    assert!(ok, "{toks}");
    let rows: Vec<&str> = toks
        .lines()
        .filter(|l| l.contains("\"kind\":\"float\""))
        .collect();
    assert_eq!(rows.len(), bits.len());
    let normal: Vec<String> = rows.iter().map(|r| field(r, "text")).collect();
    for ((b, row), text) in bits.iter().zip(&rows).zip(&normal) {
        assert_eq!(field(row, "bits"), format!("{b:#018x}"), "{row}");
        assert_eq!(text.parse::<f64>().unwrap().to_bits(), *b, "{text}");
    }
    let (ok, again) = bmb(&["dump", "tokens"], &one("t.bmb", &normal.join(" ")));
    assert!(ok, "{again}");
    let back: Vec<String> = again
        .lines()
        .filter(|l| l.contains("\"kind\":\"float\""))
        .map(|r| field(r, "bits"))
        .collect();
    let want: Vec<String> = bits.iter().map(|b| format!("{b:#018x}")).collect();
    assert_eq!(back, want);
}

const BY_PATH: &str = r#"
#[summary("경로 호출")]
mod k {
    struct P { x: i64 }

    impl P {
        #[effects(pure)]
        fn name(self: &P) -> i64 { self.x }

        #[effects(pure)]
        fn pick<U: Copy>(self: &P, u: U) -> U { u }

        #[effects(pure)]
        fn make(x: i64) -> P { P { x: x } }
    }

    trait A {
        #[effects(pure)]
        fn name(self: &Self) -> i64;
    }

    trait B {
        #[effects(pure)]
        fn tag(self: &Self) -> i64;
    }

    trait C {
        #[effects(pure)]
        fn tag(self: &Self) -> i64;
    }

    trait D {
        #[effects(pure)]
        fn only(self: &Self) -> i64;
    }

    impl A for P {
        #[effects(pure)]
        fn name(self: &P) -> i64 { 1 }
    }

    impl B for P {
        #[effects(pure)]
        fn tag(self: &P) -> i64 { 2 }
    }

    impl C for P {
        #[effects(pure)]
        fn tag(self: &P) -> i64 { 3 }
    }

    impl D for P {
        #[effects(pure)]
        fn only(self: &P) -> i64 { 4 }
    }

    #[effects(pure)]
    fn f(p: P) -> i64 {
        let q: P = P::make(1);
        A::name(&p) + P::name(&q) + B::tag(&p) + C::tag(&q) + P::pick::<i64>(&p, 4) + D::only(&q)
    }
}
"#;

/// The normal form writes a method called by path as a method call where
/// that names the same function (ADR-0051, backlog 2M-02): not a trait
/// method the type's own one shadows, not a name of two traits, not a
/// function with type parameters of its own or without a receiver. The
/// identities and the proof results are those of the program as written.
#[test]
fn methods_called_by_path_are_written_as_method_calls() {
    let src = one("k.bmb", BY_PATH);
    let out = normal(BY_PATH);
    for want in [
        "let q: P = P::make(1);",
        "A::name(&p) + q.name() + B::tag(&p) + C::tag(&q) + P::pick::<i64>(&p, 4) + q.only()",
    ] {
        assert!(out.contains(want), "{want}\n{out}");
    }
    let formatted = one("k.bmb", &out);
    assert_eq!(hashes(&formatted), hashes(&src));
    assert_eq!(statuses(&verify(&formatted).1), statuses(&verify(&src).1));
    assert_eq!(normal(&out), out, "idempotent");
}

/// `bmb fix` makes the same rewrite in place (W0911), splicing the
/// receiver's and the arguments' text: a comment between them stays.
#[test]
fn fix_writes_method_calls_in_place_with_their_comments() {
    let src = BY_PATH
        .replace("D::only(&q)", "D::only(\n            &q,\n        )")
        .replace(
            "fn only(self: &Self) -> i64;",
            "fn only(self: &Self, k: i64) -> i64;",
        )
        .replace(
            "fn only(self: &P) -> i64 { 4 }",
            "fn only(self: &P, k: i64) -> i64 { k }",
        )
        .replace(
            "D::only(\n            &q,\n        )",
            "D::only(\n            &q,\n            // the argument\n            4,\n        )",
        );
    let (ok, out) = bmb(&["fix", "--print"], &one("k.bmb", &src));
    assert!(ok, "{out}");
    assert!(
        out.contains("q.only(\n            // the argument\n            4)"),
        "{out}"
    );
    assert!(out.contains("A::name(&p) + q.name() + B::tag(&p)"), "{out}");
}
