//! Normal form and definition hashes (ADR-0019).
//! Through the CLI: `bmb fmt`, `bmb verify`, `bmb hash`.

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

const CALLER: &str = "#[summary(\"a\")]\nmod a {\n    #[summary(\"f\")]\n    #[effects(pure)]\n    fn g() -> i64 {\n        b::f()\n    }\n}\n";
const CALLEE: &str = "#[summary(\"b\")]\nmod b {\n    #[summary(\"f\")]\n    #[effects(pure)]\n    fn f() -> i64 {\n        1\n    }\n}\n";

/// `fmt --check` holds each file to the units it declares, in name order,
/// whatever the other files are and in whatever order they come.
#[test]
fn check_holds_each_file_to_its_own_units() {
    let file = |n: &str, t: &str| (n.to_string(), t.to_string());
    let check = |files: &[(String, String)]| bmb(&["fmt", "--check"], files).0;
    let (a, b) = (file("a.bmb", CALLER), file("b.bmb", CALLEE));
    assert!(check(&[a.clone(), b.clone()]));
    assert!(check(&[b.clone(), a.clone()]));
    assert!(check(&[
        file("e.bmb", ""),
        file("ab.bmb", &format!("{CALLER}\n{CALLEE}"))
    ]));
    assert!(!check(&[file("ba.bmb", &format!("{CALLEE}\n{CALLER}"))]));
    let loose = file(
        "c.bmb",
        &CALLER.replace("mod a", "mod c").replace("{\n", "{\n\n"),
    );
    assert!(!check(&[b.clone(), a.clone(), loose.clone()]));
    assert!(!check(&[loose, a.clone(), b.clone()]));
    assert_eq!(
        bmb(&["fmt"], &[a.clone(), b.clone()]),
        bmb(&["fmt"], &[b, a])
    );
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
#[summary("hash example")]
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

#[summary("another unit")]
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
    let (h_part, g_part) = PROG.split_at(PROG.find("#[summary(\"another").unwrap());
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

/// The data type surface (ADR-0026) parses and prints in
/// normal form before the checker accepts it: types by name, each followed by
/// its `impl` (functions by name), then functions; multi-line `match`; a
/// condition holding a struct literal in parentheses (as Rust requires).
/// A definition's summary (ADR-0117): `///` is its input
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
    // `bmb dump parse`: the parsed program in normal form.
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

/// `bmb dump types|moves`: what the checker's bodies
/// compute, the same text on every run.
#[test]
fn table_dumps_show_what_the_bodies_compute() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    const N: i64 = 3;
    struct P { x: i64 }
    impl P { #[effects(pure)] fn get(self: &P) -> i64 { self.x } }
    #[effects(pure)]
    fn id<T: Copy>(v: T) -> T { v }
    #[effects(alloc)]
    fn keep(v: Vec<i64>, w: Vec<i64>) -> Vec<i64> { v }
    #[effects(pure)]
    fn down(n: u8) -> u8
        decreases n,
    { if n == 0 { 0 } else { down(n - 1) } }
    #[effects(io)]
    fn main() -> () {
        let p: P = P { x: N };
        let r: i64 = id::<i64>(p.get());
        print_i64(r);
    }
}
"#;
    let dump = |stage: &str, text: &str| bmb(&["dump", stage], &one("x.bmb", text));
    for stage in ["types", "moves"] {
        let (ok, out) = dump(stage, SRC);
        assert!(ok && out.starts_with("---\n"), "{stage}: {out}");
        assert_eq!(dump(stage, SRC).1, out, "{stage}");
    }
    let types = dump("types", SRC).1;
    for row in [
        // Resolved: the `const` is its value (the normal form writes a
        // resolved method call as a method call again).
        "id::<i64>(p.get())",
        "P { x: 3 }",
        r#"{"locals":"d::main","vars":[["p","d::P"],["r","i64"]]}"#,
        r#""targs":["i64"]}"#,
        r#"{"calls":"d::down","to":["d::down"]}"#,
        r#"{"needs":"d::main","own":"pure","body":"io"}"#,
        r#"{"recursive":"d::down","scc":"#,
        r#"{"reads":"d::main","consts":["d::N"]}"#,
    ] {
        assert!(types.contains(row), "{row}\n{types}");
    }
    assert!(types.contains(r#""type":"d::P"}"#), "{types}");
    let moves = dump("moves", SRC).1;
    assert!(
        moves.contains(r#"{"moves":"d::keep","kind":"drops","file":"x.bmb","#)
            && moves.contains(r#""names":["w"]}"#),
        "{moves}"
    );
    // The checker goes on after a finding: the rows are there with it.
    let bad = SRC.replace("let r: i64 = id::<i64>(p.get());", "let r: i64 = true;");
    let (ok, out) = dump("types", &bad);
    assert!(
        !ok && out.contains("\"code\":\"E") && out.contains(r#"{"locals":"d::main""#),
        "{out}"
    );
}

/// `bmb dump texts`: the qualified texts the incremental
/// keys read. Another definition is written by its full path, the
/// definition's own name `_`; a caller's surface leaves `#[uses]` out.
#[test]
fn text_dump_shows_the_qualified_texts() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    struct P { x: i64 }
    impl P { #[effects(pure)] fn get(self: &P) -> i64 { self.x } }
    spec fn small(x: i64) -> bool { x < 10 }
    #[effects(pure)]
    #[uses(small)]
    fn low(p: &P) -> (r: i64)
        requires small(p.x),
        ensures small(r),
    { P::get(p) }
}
"#;
    let dump = |text: &str| bmb(&["dump", "texts"], &one("x.bmb", text));
    let (ok, out) = dump(SRC);
    assert!(ok && out.starts_with("---\n"), "{out}");
    assert_eq!(dump(SRC).1, out);
    for row in [
        r##"{"def":"d::low","text":"#[effects(pure)]\n#[uses(d::small)]\nfn _(p: &P) -> (r: i64)\n    requires d::small(p.x),\n    ensures d::small(r),\n{\n    d::P::get(p)\n}\n","caller":"#[effects(pure)]\nfn _(p: &P) -> (r: i64)\n    requires d::small(p.x),\n    ensures d::small(r),\n"}"##,
        r#"{"item":"d::P","hash":"sha256:"#,
        r#"{"item":"d::low","hash":"sha256:"#,
    ] {
        assert!(out.contains(row), "{row}\n{out}");
    }
    // A program that does not check has diagnostics and no rows.
    let (ok, out) = dump(&SRC.replace("P::get(p)", "true"));
    assert!(!ok && out.ends_with("---\n"), "{out}");
}

/// `bmb dump lowered`: the tables the code generator
/// reads, text as bytes and a `char` as a `u32`.
#[test]
fn lowered_dump_shows_text_as_bytes() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    struct P<T> { name: String, x: T }
    enum E { A, B(char, String) }
    #[effects(alloc)]
    fn f(s: &str, c: char) -> String {
        let p: P<char> = P { name: String::from(s), x: c };
        let o: Option<char> = Some(g(&p));
        String::new()
    }
    #[effects(pure)]
    fn g(p: &P<char>) -> char { p.x }
}
"#;
    let dump = |text: &str| bmb(&["dump", "lowered"], &one("x.bmb", text));
    let (ok, out) = dump(SRC);
    assert!(ok && out.starts_with("---\n"), "{out}");
    assert_eq!(dump(SRC).1, out);
    for row in [
        r#"{"fn":"d::f","params":["&[u8]","u32"],"ret":"Vec<u8>"}"#,
        r#"{"adt":"d::E","kind":"enum","body":[["A",[]],["B",["u32","Vec<u8>"]]]}"#,
        r#"{"adt":"d::P","kind":"struct","body":[["name","Vec<u8>"],["x","T"]]}"#,
        r#"{"locals":"d::f","vars":[["c","u32"],["o","Option<u32>"],["p","d::P<u32>"],["s","&[u8]"]]}"#,
        r#""text":true}"#,
        r#"{"of":"&[u8]","value":"&[u8]"}"#,
        r#"{"of":"&d::P<u32>","value":"d::P<u32>","fields":[["name","Vec<u8>"],["x","u32"]]}"#,
        r#"{"of":"Option<u32>","value":"Option<u32>","variants":[["None",[]],["Some",["u32"]]]}"#,
    ] {
        assert!(out.contains(row), "{row}\n{out}");
    }
    // The checker goes on after a finding: the rows are there with it.
    let (ok, out) = dump(&SRC.replace("String::new()", "true"));
    assert!(!ok && out.contains(r#"{"fn":"d::f""#), "{out}");
}

/// `bmb dump layout`: the LLVM types, sizes and type
/// definitions the code generator gives the program's types. `layout`
/// sizes an enum's payload area: a scalar counts as a word there, whatever
/// its width in memory.
#[test]
fn layout_dump_shows_the_type_definitions() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    struct P { on: bool, x: u16, name: String }
    enum E { A, B(u8, P), C(i64) }
    #[effects(alloc, io)]
    fn main() -> () {
        let p: P = P { on: true, x: 1, name: String::new() };
        let e: E = E::B(2, p);
        let a: [u8; 3] = [1, 2, 3];
    }
}
"#;
    let dump = |text: &str| bmb(&["dump", "layout"], &one("x.bmb", text));
    let (ok, out) = dump(SRC);
    assert!(ok && out.starts_with("---\n"), "{out}");
    assert_eq!(dump(SRC).1, out);
    for row in [
        r#"{"of":"d::P","reg":"i64","mem":"%\"d::P\"","size":40,"align":8,"is":"data mem_elem mem holds_vec move","elem_of":"d::P","elem":{"kind":"i64","mem":"i64","align":8}}"#,
        r#"{"of":"d::E","reg":"i64","mem":"%\"d::E\"","size":56,"align":8,"#,
        r#"{"of":"[u8; 3]","reg":"i64","mem":"[3 x i8]","size":24,"align":8,"is":"mem_elem","elem_of":"u8","#,
        r#"{"of":"u8","reg":"i64","mem":"i8","size":8,"align":8,"is":"scalar","elem_of":"u8","elem":{"kind":"u8","mem":"i8","align":1}}"#,
        r#"{"uses_vec":true,"uses_heap":true,"main":"d::main"}"#,
        "%\"d::P\" = type { i8, i16, %bmb.vec }\n",
        "%\"d::E\" = type { i64, [6 x i64] }\n",
        "%\"d::E::B\" = type { i64, i8, %\"d::P\" }\n",
        "%\"d::E::C\" = type { i64, i64 }\n",
    ] {
        assert!(out.contains(row), "{row}\n{out}");
    }
    // A program that does not check has diagnostics and no rows.
    let (ok, out) = dump(&SRC.replace("E::B(2, p)", "true"));
    assert!(!ok && out.ends_with("---\n"), "{out}");
}

/// `bmb dump smt-decls`: what the proof encoder derives before it reads a
/// body, with no solver: the instances of generic spec fns, the sorts of
/// parameters and results in the encoder's view (a `Vec` and text are an
/// array and a length, a struct its leaves), the SMT functions of a spec fn,
/// the declarations of type parameters, the datatypes, which parameters are
/// text and the default contexts.
#[test]
fn smt_decls_dump_shows_the_encoders_declarations() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    struct Node { v: i64, kids: Vec<Node> }
    struct P { x: i64, on: bool }
    enum E { A, B(P) }
    spec fn same<T: Copy>(a: T, b: T) -> bool { a == b }
    spec fn top<T: Ord + Copy>(a: T, b: T) -> bool { a <= b }
    spec fn mid(p: P) -> P { p }
    #[effects(pure)]
    fn id<T: Copy>(x: T) -> (r: T)
        ensures same::<T>(r, x),
    {
        x
    }
    #[effects(pure)]
    fn f(s: &str, n: &Node, e: E) -> (r: u8)
        requires top::<usize>(s.len(), 9),
    {
        id::<u8>(3)
    }
}
"#;
    let dump = |text: &str| bmb(&["dump", "smt-decls"], &one("x.bmb", text));
    let (ok, out) = dump(SRC);
    assert!(ok && out.starts_with("---\n"), "{out}");
    assert_eq!(dump(SRC).1, out);
    for row in [
        // An instance in the generic caller's own terms, one from the
        // callee's contract at the call, one from a clause.
        r#"{"inst":"d::same<T>","base":"d::same","sub":[["T","T"]]}"#,
        r#"{"inst":"d::same<u8>","base":"d::same","sub":[["T","u8"]]}"#,
        r#"{"inst":"d::top<usize>","base":"d::top","sub":[["T","usize"]]}"#,
        r#"{"inst_type":"usize"}"#,
        r#"{"fn":"d::f","params":["(Array Int Int)","Int","|d::Node|","|d::E|"],"ret":["Int"],"funs":[],"sorts":""}"#,
        r#"{"fn":"d::id","params":["|'T|"],"ret":["|'T|"],"funs":[],"sorts":"(declare-sort |'T| 0)\n(declare-const |'T.default| |'T|)\n"}"#,
        r#"{"fn":"d::mid","params":["Int","Bool"],"ret":["Int","Bool"],"funs":[["|d::mid.x|","Int"],["|d::mid.on|","Bool"]],"sorts":""}"#,
        r#"{"fn":"d::same<u8>","params":["Int","Int"],"ret":["Bool"],"funs":[["|d::same<u8>|","Bool"]],"sorts":""}"#,
        r#"(declare-fun |'T.le| (|'T| |'T|) Bool)\n(assert (forall ((a |'T|)) (! (|'T.le| a a) :pattern ((|'T.le| a a)))))\n"#,
        r#"{"datatype":"|d::Node::","name":"|d::Node|","decl":"(define-sort |d::Node| () Int)"}"#,
        r#"{"datatype":"|d::E::","name":"|d::E|","decl":"(declare-datatypes ((|d::E| 0)) (((|d::E::A|) (|d::E::B| (|d::E::B.0.x| Int) (|d::E::B.0.on| Bool)))))"}"#,
        r#"(declare-fun |d::Node::Node.1.e| (|d::Node|) (Array Int |d::Node|))\n"#,
        r#"{"text_sig":"d::f","params":[true,false,false],"ret":false}"#,
        r#"{"context":"d::f","specs":["d::same","d::top"]}"#,
        r#"{"context":"d::same","specs":[]}"#,
    ] {
        assert!(out.contains(row), "{row}\n{out}");
    }
    // A program that does not check has diagnostics and no rows.
    let (ok, out) = dump(&SRC.replace("id::<u8>(3)", "true"));
    assert!(!ok && out.ends_with("---\n"), "{out}");
}

/// `bmb dump smt`: what the proof encoder builds of each definition, with
/// no solver: the declarations of spec fns (one SMT function per leaf of a
/// struct result), a row per definition and instance with its proof context
/// and what it reveals, a spec fn's definition as the queries take it (a
/// macro form before the marker for one that is not recursive), the entry
/// facts of a proof fn's parameters, and the termination obligations of a
/// recursive spec fn (under `forall` for every value of the bound variable;
/// with the struct facts of a recursive struct's selectors).
#[test]
fn smt_dump_shows_spec_definitions_and_termination() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    struct Node { v: u64, kids: Vec<Node> }
    struct P { x: i64, on: bool }
    spec fn same<T: Copy>(a: T, b: T) -> bool { a == b }
    spec fn mid(p: P) -> P { P { x: p.x + 1, on: !p.on } }
    spec fn below(n: usize) -> bool
        decreases n,
    {
        n == 0 || (forall|k: usize| k < n ==> #[trigger] below(k))
    }
    spec fn spine(n: &Node) -> u64
        decreases n,
    {
        if n.kids.len() == 0 { n.v } else { spine(&n.kids[0]) }
    }
    #[uses(below)]
    proof fn grows(a: &[u64], p: P)
        ensures same::<bool>(p.on, p.on),
    {
    }
    #[effects(pure)]
    #[uses(mid)]
    fn f(p: P, n: &Node) -> (r: u8)
        requires same::<i64>(p.x, 0),
    {
        3
    }
}
"#;
    let dump = |text: &str| bmb(&["dump", "smt"], &one("x.bmb", text));
    let (ok, out) = dump(SRC);
    assert!(ok && out.starts_with("---\n"), "{out}");
    assert_eq!(dump(SRC).1, out);
    for row in [
        r#"{"spec_decl":"d::mid","key":"|d::mid.","decl":"(declare-fun |d::mid.x| (Int Bool) Int)\n(declare-fun |d::mid.on| (Int Bool) Bool)"}"#,
        r#"{"def":"d::f","kind":"fn","inst":false,"unit":"d","sub":[],"uses":["d::mid"],"context":["d::mid","d::same"],"contract":["d::same"],"revealed":["d::mid","d::same<bool>","d::same<i64>"],"arrays":[false,true,false],"fields":[[0,1],[0,1]],"encoded":true}"#,
        // A proof fn's body is encoded as an executable function's.
        r#""revealed":["d::below","d::same<bool>","d::same<i64>"],"arrays":[true,false,false],"fields":[[],[0,1]],"encoded":true}"#,
        // A generic spec fn has no definition of its own; its instance has.
        r#"{"def":"d::same","kind":"spec fn","inst":false,"unit":"d","sub":[],"uses":[],"context":[],"contract":[],"revealed":[],"arrays":[false,false,false],"fields":[[],[]],"encoded":true}"#,
        r#"{"axiom":"d::same<bool>","text":"(define-fun |d::same<bool>| ((|a!p| Bool) (|b!p| Bool)) Bool (= |a!p| |b!p|))\n; quantified form outside the probe\n(assert (forall ((|a!p| Bool) (|b!p| Bool)) (! (=> true (= (|d::same<bool>| |a!p| |b!p|) (= |a!p| |b!p|))) :pattern ((|d::same<bool>| |a!p| |b!p|)))))","facts":"","parts":[{"sym":"|d::same<bool>|","params":["|a!p|","|b!p|"],"binders":["(|a!p| Bool)","(|b!p| Bool)"],"guard":"true","body":"(= |a!p| |b!p|)","sort":"Bool","recursive":false}]}"#,
        r#"(define-fun |d::mid.on| ((|p!p.x| Int) (|p!p.on| Bool)) Bool (ite (and (<= (- 9223372036854775808) |p!p.x|) (<= |p!p.x| 9223372036854775807)) (not |p!p.on|) (|d::mid.on!ext| |p!p.x| |p!p.on|)))"#,
        r#"{"proof_params":"d::grows","decls":["(declare-const |a@0| (Array Int Int))","(declare-const |a.len@0| Int)","(declare-const |p.x@0| Int)","(declare-const |p.on@0| Bool)"],"facts":["(forall ((k Int)) (! (and (<= 0 (select |a@0| k)) (<= (select |a@0| k) 18446744073709551615)) :pattern ((select |a@0| k))))","(and (<= 0 |a.len@0|) (<= |a.len@0| 1152921504606846975))","(and (<= (- 9223372036854775808) |p.x@0|) (<= |p.x@0| 9223372036854775807))"]}"#,
        r#"{"termination":"d::below","decls":["(declare-const |n@0| Int)"],"trivial":[]}"#,
        r#""msg":"recursive call to `d::below` may not decrease its `decreases` measure","shared":0,"assumptions":["(and (<= 0 |n@0|) (<= |n@0| 18446744073709551615))","(not (= |n@0| 0))"],"goal":"(forall ((|k!0| Int)) (=> (and (and (<= 0 |k!0|) (<= |k!0| 18446744073709551615)) (< |k!0| |n@0|)) (and (<= 0 |n@0|) (< |k!0| |n@0|))))","facts":"","shown":null,"sub":null}"#,
        r#""assumptions":["(not (= (|d::Node::Node.1.len| |n@0|) 0))"],"goal":"(and (<= 0 0) (< 0 (|d::Node::Node.1.len| |n@0|)))","facts":"(assert (and (<= 0 (|d::Node::Node.1.len| |n@0|)) (<= (|d::Node::Node.1.len| |n@0|) 9223372036854775807)))\n","shown":null,"sub":null}"#,
    ] {
        assert!(out.contains(row), "{row}\n{out}");
    }
    // A program that does not check has diagnostics and no rows.
    let (ok, out) = dump(&SRC.replace("{ a == b }", "{ a }"));
    assert!(!ok && out.ends_with("---\n"), "{out}");
}

/// `bmb dump smt` on executable and proof functions: a body has an `exec`
/// row (declarations, the path condition at the end, the entry values, the
/// reach points with their path conditions) and an `obl` row per
/// obligation (its assumptions as the number it shares with the row before
/// and the rest), built by the encoder the proof pass runs, with no solver.
/// The rows below
/// show a `return` on one side of an `if`, a call with a `&mut` argument
/// (its precondition, the new value and the callee's `ensures` with `old`),
/// the obligations and lemmas of `%`, a conversion, a type invariant proved
/// for the result with the value a counterexample would show, a `for` (its
/// bounds on entry, its havocked variable, the obligations of going back to
/// the head and its exit as a reach point), a proof fn's body, a `proof`
/// block that calls it, a `push` on a `Vec` and on a `String` with a
/// comparison of the text, and a sub-proof with its query.
#[test]
fn smt_dump_shows_executable_and_proof_bodies() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    #[invariant(low)]
    struct Cap { n: u8 }
    spec fn low(c: &Cap) -> bool { c.n < 9 }
    spec fn small(a: i64) -> bool { a < 100 }
    #[effects(pure)]
    fn inc(a: &mut i64) -> ()
        requires small(*a),
        ensures small(*a - 1) && *a == *old(a) + 1,
    {
        *a = *a + 1;
    }
    #[effects(pure)]
    fn f(x: i64, c: Cap) -> (r: Cap)
        requires small(x),
    {
        let mut y: i64 = if x < 0 { return c; } else { x };
        inc(&mut y);
        Cap { n: (y % 4) as u8 }
    }
    #[effects(pure)]
    fn sum(a: &[u8]) -> (r: u64) {
        let mut s: u64 = 0;
        for i in 0..a.len() {
            s = s.wrapping_add(a[i] as u64);
        }
        s
    }
    proof fn lemma(a: i64)
        requires small(a),
        ensures small(a - 1),
    {
        if a > 0 {
            assert(a - 1 < a);
        }
    }
    #[effects(alloc)]
    fn grow(v: &mut Vec<u8>, s: &mut String, a: i64) -> bool
        requires small(a),
    {
        proof {
            lemma(a);
        }
        v.push(1);
        s.push('é');
        s.as_str() < "m"
    }
    #[effects(pure)]
    fn masked(x: u32) -> u32 {
        assert(x & 0 == 0) by (bit_vector);
        x
    }
}
"#;
    let dump = |text: &str| bmb(&["dump", "smt"], &one("x.bmb", text));
    let (ok, out) = dump(SRC);
    assert!(ok && out.starts_with("---\n"), "{out}");
    assert_eq!(dump(SRC).1, out);
    for row in [
        r#"{"def":"d::f","kind":"fn","inst":false,"unit":"d","sub":[],"uses":[],"context":["d::small"],"contract":["d::small"],"revealed":["d::small"],"arrays":[false,false,false],"fields":[[],[0]],"encoded":true}"#,
        r#"{"exec":"d::f","decls":["(declare-const |x@0| Int)","(declare-const |c.n@0| Int)","(declare-const |y@0| Int)"],"trivial":[],"dead":false,"pc":["(and (<= (- 9223372036854775808) |x@0|) (<= |x@0| 9223372036854775807))","(and (<= 0 |c.n@0|) (<= |c.n@0| 255))","(|d::low| |c.n@0|)","(|d::small| |x@0|)","(not (< |x@0| 0))","(|d::small| |x@0|)","(and (<= (- 9223372036854775808) |y@0|) (<= |y@0| 9223372036854775807))","(and (|d::small| (- |y@0| 1)) (= |y@0| (+ |x@0| 1)))","(not (= 4 0))","#,
        r#""(|d::low| (bmb_trem |y@0| 4))"],"inputs":[["x",["|x@0|"],"i64"],["c",["|c.n@0|"],"d::Cap"]],"texts":[],"valid":[],"reach":[[0,354,355,"end of the body",["(and (<= (- 9223372036854775808) |x@0|) (<= |x@0| 9223372036854775807))","#,
        r#"{"obl":"d::f","code":"E0801","file":0,"lo":485,"hi":496,"msg":"precondition `small` of `d::inc` may not hold","shared":0,"assumptions":["(and (<= (- 9223372036854775808) |x@0|) (<= |x@0| 9223372036854775807))","(and (<= 0 |c.n@0|) (<= |c.n@0| 255))","(|d::low| |c.n@0|)","(|d::small| |x@0|)","(not (< |x@0| 0))"],"goal":"(|d::small| |x@0|)","facts":"","shown":null,"sub":null}"#,
        r#""goal":"(not (and (= |y@0| (- 9223372036854775808)) (= 4 (- 1))))","facts":"","shown":null,"sub":null}"#,
        r#""goal":"(and (<= 0 (bmb_trem |y@0| 4)) (<= (bmb_trem |y@0| 4) 255))","facts":"","shown":null,"sub":null}"#,
        r#""goal":"(|d::low| (bmb_trem |y@0| 4))","facts":"","shown":["result",["(bmb_trem |y@0| 4)"],"d::Cap"],"sub":null}"#,
        r#"{"exec":"d::inc","decls":["(declare-const |a@0| Int)"],"trivial":[],"dead":false,"pc":["(and (<= (- 9223372036854775808) |a@0|) (<= |a@0| 9223372036854775807))","(|d::small| |a@0|)","(and (<= (- 9223372036854775808) (+ |a@0| 1)) (<= (+ |a@0| 1) 9223372036854775807))","(and (|d::small| (- (+ |a@0| 1) 1)) (= (+ |a@0| 1) (+ |a@0| 1)))"],"inputs":[["a",["|a@0|"],"&mut i64"]],"texts":[],"valid":[],"reach":[[0,190,193,"end of the body",["(and (<= (- 9223372036854775808) |a@0|) (<= |a@0| 9223372036854775807))","(|d::small| |a@0|)","(and (<= (- 9223372036854775808) (+ |a@0| 1)) (<= (+ |a@0| 1) 9223372036854775807))"]]]}"#,
        r#""msg":"postcondition `small(*a - 1) && *a == *old(a) + 1` may not hold""#,
        r#"{"def":"d::sum","kind":"fn","inst":false,"unit":"d","sub":[],"uses":[],"context":[],"contract":[],"revealed":[],"arrays":[true,false],"fields":[[]],"encoded":true}"#,
        r#"{"exec":"d::sum","decls":["(declare-const |a@0| (Array Int Int))","(declare-const |a.len@0| Int)","(declare-const |s@0| Int)","(declare-const |i@0| Int)"],"trivial":[],"dead":false,"pc":["#,
        r#""(and (<= 0 |i@0|) (or (<= |i@0| |a.len@0|) (= |i@0| 0)))","(not (< |i@0| |a.len@0|))"],"inputs":[["a",["|a@0|","|a.len@0|"],"&[u8]"]],"texts":[],"valid":[],"reach":[[0,629,705,"exit of the loop",["#,
        r#""(not (< |i@0| |a.len@0|))"]],[0,565,568,"end of the body",["#,
        r#"{"obl":"d::sum","code":"E0806","file":0,"lo":629,"hi":705,"msg":"loop invariant may not hold on entry","shared":0,"assumptions":["#,
        r#""msg":"loop invariant may not be preserved","#,
        r#""goal":"(and (<= 0 (- |a.len@0| |i@0|)) (< (- |a.len@0| (+ |i@0| 1)) (- |a.len@0| |i@0|)))","facts":"","shown":null,"sub":null}"#,
        r#"{"def":"d::lemma","kind":"proof fn","inst":false,"unit":"d","sub":[],"uses":[],"context":["d::small"],"contract":["d::small"],"revealed":["d::small"],"arrays":[false,false],"fields":[[]],"encoded":true}"#,
        r#"{"exec":"d::lemma","decls":["(declare-const |a@0| Int)"],"trivial":[],"dead":false,"pc":["(and (<= (- 9223372036854775808) |a@0|) (<= |a@0| 9223372036854775807))","(|d::small| |a@0|)","(=> (> |a@0| 0) (< (- |a@0| 1) |a@0|))","(|d::small| (- |a@0| 1))"],"inputs":[["a",["|a@0|"],"i64"]],"texts":[],"valid":[],"reach":[[0,735,740,"end of the body",["#,
        r#""msg":"precondition `small` of `d::lemma` may not hold","#,
        r#""(|d::small| |a@0|)","(|d::small| (- |a@0| 1))","(< |v.len@0| 9223372036854775807)","(or (and (<= 0 233) (<= 233 55295)) (and (<= 57344 233) (<= 233 1114111)))","#,
        r#"(store (store (store (store |s@0| (+ |s.len@0| 0) 195) (+ |s.len@0| 1) 169) (+ |s.len@0| 2) 0) (+ |s.len@0| 3) 0)"#,
        r#""(<= 0 |lcp@0|)","#,
        r#""texts":[["|s@0|","|s.len@0|"]],"valid":[],"reach":[[0,907,911,"end of the body",["#,
        r#"{"def":"d::masked","kind":"fn","inst":false,"unit":"d","sub":[],"uses":[],"context":[],"contract":[],"revealed":[],"arrays":[false,false],"fields":[[]],"encoded":true}"#,
        r#"{"obl":"d::masked","code":"E0818","file":0,"lo":1174,"hi":1209,"msg":"bit-vector assertion may not hold","shared":0,"assumptions":[],"goal":"(= (_ bv0 1) (_ bv0 1))","facts":"","shown":null,"sub":{"logic":"QF_BV","decls":["(declare-const |x| (_ BitVec 32))"],"hyps":[],"goal":"(= (_ bv0 1) (_ bv0 1))","vars":[["|x|","u32"]],"guards":[]}}"#,
    ] {
        assert!(out.contains(row), "{row}\n{out}");
    }
    // The invariant of the `Cap` returned early is on the path already: no
    // obligation.
    assert_eq!(out.matches("\"code\":\"E0823\"").count(), 1, "{out}");
}

#[path = "../../tests/port/smt_terms_ref.rs"]
mod smt_terms_ref;

/// The stage0 side of the differential case `smt-terms`
/// (`tests/port/smt_terms_ref.rs`, also an example of this crate): the
/// proof encoder's term builders that need no encoder state and the
/// translations of sub-proofs (`subproof`), each over a grid of arguments
/// written there, with no solver: one row per call, the
/// same rows every time. The port prints the same rows from its own
/// builders (`tests/port/smt_terms.bmb`). The rows below
/// are one of each kind of builder: numerals past a machine word, a text
/// reader, a linear and a bit-vector form of a bit operation, a bit method
/// with a constant amount, an `f64` literal, a lemma, a value of a slice
/// of structs, a default value read from the tables, an assignment into
/// a recursive struct (a new constant with the facts of its fields), the
/// definition of a bit helper, a model value as a counterexample shows it,
/// the counterexample of a sub-proof, a script in canonical symbols with
/// its legend, an answer named back in the program's symbols, the
/// canonical text of a query, and the bit-vector query of a term that converts an integer
/// to `f64`.
#[test]
fn smt_terms_reference_shows_the_term_builders() {
    let out = smt_terms_ref::rows();
    assert_eq!(smt_terms_ref::rows(), out);
    for row in [
        r#"{"fn":"numeral","args":"(- 170141183460469231731687303715884105727)","out":"(- 170141183460469231731687303715884105727)"}"#,
        r#"{"fn":"numeral","args":"170141183460469231731687303715884105728","out":null}"#,
        r#"{"fn":"numeral","args":"(- 0)","out":"0"}"#,
        r#"{"fn":"fixed_len","args":"+5","out":true}"#,
        r#"{"fn":"select_app","args":"(select |a b| |i (|)","out":["|a b|","|i (|"]}"#,
        r#"{"fn":"implication","args":"(=> (and a b) (or c d))","out":["(and a b)","(or c d)"]}"#,
        r#"{"fn":"range_int","args":"i64 x","out":"(and (<= (- 9223372036854775808) x) (<= x 9223372036854775807))"}"#,
        r#"{"fn":"wrap_int","args":"u64 x","out":"(mod x 18446744073709551616)"}"#,
        r#"{"fn":"bit_op","args":"BitAnd u64 a 18446744073709551615","out":["(mod a 18446744073709551616)",[]]}"#,
        r#"{"fn":"bit_op","args":"Shl i8 a 3","out":["(- (mod (+ (* a 8) 128) 256) 128)",[]]}"#,
        r#"{"fn":"bit_op","args":"BitXor u8 a b","out":["(bv2nat (bvxor ((_ int2bv 8) a) ((_ int2bv 8) b)))",["(=> (and (and (<= 0 a) (<= a 255)) (and (<= 0 b) (<= b 255))) (and (<= 0 (bv2nat (bvxor ((_ int2bv 8) a) ((_ int2bv 8) b)))) (<= (bv2nat (bvxor ((_ int2bv 8) a) ((_ int2bv 8) b))) 255)))"]]}"#,
        r#"{"fn":"bit_method","args":"rotate_right u8 (- 3)","out":["(+ (* (mod a 32) 8) (div a 32))",[]]}"#,
        r#"{"fn":"f64_lit","args":"9221120237041090560","out":"(_ NaN 11 53)"}"#,
        r#"{"fn":"f64_lit","args":"9223372036854775808","out":"(fp #b1 #b00000000000 #x0000000000000)"}"#,
        r#"{"fn":"instantiate","args":"unsigned 1","out":"(=> (> (- y) 0) (and (<= 0 {a}{q}) (< {a}{q} (- y))))"}"#,
        r#"{"fn":"select_val","args":"5","out":{"arr":{"struct":[{"s":"(select xe0 i)"},{"s":"(select xe1 i)"}]},"len":"2"}}"#,
        r#"{"fn":"default_val","args":"&[g::R]","out":{"a":"((as const (Array Int |g::R|)) |g::R::'arb|)","len":"0"}}"#,
        r#"{"fn":"leaf_fact","args":"&[u8] v.len","out":"(and (<= 0 v.len) (<= v.len 9223372036854775807))"}"#,
        r#"{"fn":"set_at","args":"11 DtField(0,&g::R)","out":{"val":{"s":"|R!new@0|"},"decls":["(declare-const |R!new@0| |g::R|)","(assert (= (|g::R::R.0| |R!new@0|) new))","(assert (= (|g::R::R.1.e| |R!new@0|) (|g::R::R.1.e| r)))","(assert (= (|g::R::R.1.len| |R!new@0|) (|g::R::R.1.len| r)))"]}}"#,
        r#"{"fn":"helper_def","args":"bmb_rotl8","out":["(define-fun bmb_rotl8 ((x (_ BitVec 8)) (s (_ BitVec 8))) (_ BitVec 8) (bvor (bvshl x (bvand s (_ bv7 8))) (bvlshr x (bvsub (_ bv8 8) (bvand s (_ bv7 8))))))",8]}"#,
        r#"{"fn":"render_value","args":"(- 3)","out":"-3"}"#,
        r##"{"fn":"sub_values","args":"#xff;i8","out":"x@0 = -1, b = true, f = #xff"}"##,
        r#"{"fn":"canon_back","args":"$4!val!0","out":"T!val!0"}"#,
        r#"{"fn":"canon_back","args":"|$5|","out":"|x y|"}"#,
        r#"{"fn":"canonical","args":"no reset |a|\n   ; indented comment |a|\n\t(reset)\t\n|a|","out":"(reset)"}"#,
        r#"{"fn":"canonical_script","args":"(reset)\n(declare-const |a| Int)\n(assert (> |a| |b c|))\n(assert (> |b c| |a|))\n(check-sat)\n","out":{"text":"(reset)\n(declare-const |$0| Int)\n(assert (> |$0| |$1|))\n(assert (> |$1| |$0|))\n(check-sat)\n","legend":"; query 1\n|$0| |a|\n|$1| |b c|\n"}}"#,
        r#"{"fn":"bit_vector","args":"d:i64 s:u64 o:bool x:f64 y:f64;(f64.lt (f64.from d) (fp #b0 #b01111111111 #x0000000000000))","out":{"logic":"QF_FPBV","decls":["(define-fun f64.lt ((a Float64) (b Float64)) Bool (fp.lt a b))","(declare-const d (_ BitVec 64))","(declare-const s (_ BitVec 64))","(declare-const o Bool)","(declare-const x Float64)","(declare-const y Float64)"],"hyps":[],"goal":"(f64.lt ((_ to_fp 11 53) RNE d) (fp #b0 #b01111111111 #x0000000000000))","guards":[]}}"#,
    ] {
        assert!(out.contains(&format!("{row}\n")), "{row}");
    }
    // Every row names its builder, and no builder is left out.
    let builders: std::collections::BTreeSet<&str> = out
        .lines()
        .filter_map(|l| l.strip_prefix("{\"fn\":\""))
        .filter_map(|l| l.split('"').next())
        .collect();
    assert_eq!(builders.len(), 84, "{builders:?}");
    assert_eq!(out.lines().count(), 6591);
}

/// `bmb dump runtime`: the code generator's constant IR
/// texts by name (the port's module is made of them, and its IR is compared
/// with this compiler's: `tests/port/cases.json`, case `emit`).
#[test]
fn runtime_dump_lists_the_constant_texts() {
    let (ok, out) = bmb(&["dump", "runtime"], &[]);
    assert!(ok, "{out}");
    let names: Vec<&str> = out
        .lines()
        .filter_map(|l| l.strip_prefix("{\"name\":\""))
        .filter_map(|l| l.split('"').next())
        .collect();
    assert_eq!(
        names,
        [
            "head",
            "head_rest",
            "heap_runtime",
            "vec_runtime",
            "print_f64",
            "text_cmp",
            "text_grow",
            "text_utf8",
            "sys_args",
            "sys_child",
            "sys_clock",
            "sys_cpath",
            "sys_dirs",
            "sys_eprint",
            "sys_exit",
            "sys_fs",
            "sys_io_common",
            "sys_meta",
            "sys_proc",
            "sys_run",
            "sys_stdin"
        ]
    );
    assert!(
        out.starts_with("{\"name\":\"head\",\"text\":\"; generated by bmb"),
        "{out}"
    );
}

/// `bmb dump std|lower|decls`: the stages between the
/// parser and the checker's bodies, each the same text on every run.
#[test]
fn stage_dumps_show_the_lowering_and_the_declarations() {
    const SRC: &str = r#"#[summary("d")]
mod d {
    const N: usize = 3;
    const T: [u8; N] = [1, 2, 3];
    struct P { x: i64 }
    trait Sz { #[effects(pure)] fn sz(self: &Self) -> i64; }
    impl Sz for P { #[effects(pure)] fn sz(self: &P) -> i64 { self.x } }
    #[effects(pure)]
    fn ap<F: Fn(i64) -> i64>(f: F, v: i64) -> i64 { f(v) }
    #[effects(alloc, io)]
    fn main() -> () {
        let r: i64 = ap::<_>(|a: i64| -> i64 { a }, 2);
        let mut s: String = String::new();
        std::num::push_i64(&mut s, r);
        print_str(s.as_str());
    }
}
"#;
    let dump = |stage: &str, text: &str| bmb(&["dump", stage], &one("x.bmb", text));
    let (ok, out) = bmb(&["check"], &one("x.bmb", SRC));
    assert!(ok, "{out}");
    for stage in ["std", "lower", "decls"] {
        let (ok, out) = dump(stage, SRC);
        assert!(ok && out.starts_with("---\n"), "{stage}: {out}");
        assert_eq!(dump(stage, SRC).1, out, "{stage}");
    }
    // The library units the program names come with it.
    let std = dump("std", SRC).1;
    assert!(
        std.contains("mod d {") && std.contains("\nmod std {"),
        "{std}"
    );
    assert!(
        std.contains("const T: [u8; N]") && std.contains("|a: i64|"),
        "{std}"
    );
    // Lowered: the constant's length is its value, the closure is a struct.
    let lower = dump("lower", SRC).1;
    assert!(lower.contains("const T: [u8; 3]"), "{lower}");
    for row in [
        r#"{"made":"d::__closure$0","name":"d::main::{closure#0}","file":"x.bmb","#,
        r#"{"passes":"<d::__closure$0 as d::__fn$0_0>::call","file":"x.bmb","#,
        r#"{"const":"d::T","unit":"d","type":"[u8; 3]","value":"[1, 2, 3]"}"#,
    ] {
        assert!(lower.contains(row), "{row}\n{lower}");
    }
    let decls = dump("decls", SRC).1;
    for row in [
        r#"{"type":"d::P","generics":[],"kind":"struct","body":[["x","i64"]],"inv":null}"#,
        r#"{"fn":"d::ap","kind":"exec","generics":[{"name":"F","bounds":["d::__fn$0_0"]}],"impl_generics":0,"params":["F","i64"],"ret":"i64","effects":"pure","vacuous":false,"builtin":false,"in_impl":false,"trait_of":null,"default":false}"#,
        r#"{"trait":"d::Sz","methods":["sz"]}"#,
        r#"{"impl":"d::Sz","generics":[],"for":"d::P","unit":"d"}"#,
    ] {
        assert!(decls.contains(row), "{row}\n{decls}");
    }
    // A syntax error ends every stage as it ends `dump parse`; a later
    // error comes before the `---` line.
    let (ok, out) = dump("decls", "mod {");
    assert!(!ok && !out.contains("---"), "{out}");
    let dup = "#[summary(\"a\")]\nmod a {\n    struct S { x: i64 }\n    struct S { y: i64 }\n}\n";
    let (ok, out) = dump("decls", dup);
    assert!(
        !ok && out.contains("\"code\":\"E") && out.contains("\n---\n"),
        "{out}"
    );
}

/// A literal keeps its base; separators, leading zeros and the case of hex
/// digits are not part of the normal form (ADR-0048 §3, ADR-0019 §1).
/// Bit operators print with Rust's precedence: only the parentheses it
/// needs.
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

/// `assert(e) by (bit_vector);` has one normal form, and a
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
#[summary("path calls")]
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
/// that names the same function (ADR-0051): not a trait
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
