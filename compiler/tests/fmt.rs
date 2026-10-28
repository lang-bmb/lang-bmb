//! Normal form and definition hashes (ADR-0019, backlog 1D-02, 1D-03).

use std::fs;
use std::path::{Path, PathBuf};

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
    bmb::format_sources(&one("x.bmb", text))
        .unwrap()
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
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
        let before = bmb::verify_sources(&one("x.bmb", &text));
        let after = bmb::verify_sources(&one("x.bmb", &normal(&text)));
        assert_eq!(
            statuses(&before.output),
            statuses(&after.output),
            "{}",
            p.display()
        );
    }
}

fn hashes(files: &[(String, String)]) -> Vec<String> {
    let mut v: Vec<String> = bmb::hash_sources(files)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
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
    let once = bmb::format_parsed(&one("x.bmb", SRC)).unwrap();
    assert_eq!(once.values().cloned().collect::<String>(), NORMAL);
    let twice = bmb::format_parsed(&one("x.bmb", NORMAL)).unwrap();
    assert_eq!(twice.values().cloned().collect::<String>(), NORMAL);
    // The checker rejects this sample (unknown `Shape`, values not checked yet).
    assert!(bmb::format_sources(&one("x.bmb", SRC)).is_err());
    // A wildcard arm parses; the checker rejects it with the arms it stands
    // for (ADR-0026: one arm per variant).
    let wild = "#[summary(\"w\")]\nmod w {\n    #[effects(pure)]\n    fn f(o: Option<i64>) -> i64 { match o { Some(v) => v, _ => 0 } }\n}\n";
    assert!(bmb::format_parsed(&one("w.bmb", wild)).is_ok());
    let err = bmb::format_sources(&one("w.bmb", wild)).unwrap_err();
    assert!(err.contains("E0224") && err.contains("None => 0"), "{err}");
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
    let before = bmb::verify_sources(&one("x.bmb", text));
    let after = bmb::verify_sources(&one("x.bmb", &out));
    assert_eq!(statuses(&before.output), statuses(&after.output));
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
    let before = bmb::verify_sources(&one("x.bmb", text));
    let after = bmb::verify_sources(&one("x.bmb", &out));
    assert_eq!(statuses(&before.output), statuses(&after.output));
    assert!(before.ok, "{}", before.output);
    let plain = out.replace(" by (bit_vector);", ";");
    assert_ne!(
        def_line(&one("x.bmb", &out), "b::f"),
        def_line(&one("x.bmb", &plain), "b::f")
    );
}
