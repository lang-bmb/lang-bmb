//! `bmb examples` (backlog T1-44c, ADR-0078 H-INTENT): examples of a
//! contract from its `requires` and `ensures` alone. Through the command
//! line only.
//!
//! What a row claims is checked by another route than the query that found
//! it (`shown_examples_hold_at_the_gate`): the values of each row are
//! written into a function of their own that asserts the claim, and `bmb
//! verify` proves it. The generator asks for a model of the contract over
//! the entry state; the check is a proof of the claim at those literals in
//! a body.

mod common;

use std::fs;
use std::process::Command;

use common::cli;
use common::json::{self, Value};

/// The spec fns of `SRC`, for the `#[uses(..)]` of a check.
const SPECS: &str = "sorted, upper, has, all_in, pre_twice, post_twice, pre_above, post_above, \
pre_find, post_find, pre_maxi, post_maxi, pre_within, post_within, pre_sum, post_sum, pre_bump, \
post_bump, pre_swap, post_swap, tally, pre_total, post_total";

/// The definitions the checks are about. Each contract is one `pre_*` and
/// one `post_*`, so that a check states it by the same names.
const SRC: &str = r#"#[summary("t")]
mod d {
    struct Point { x: i64, y: bool }

    spec fn sorted(a: &[i64]) -> bool {
        forall|i: usize, j: usize| i < j && j < a.len() ==> #[trigger] a[i] <= #[trigger] a[j]
    }

    spec fn upper(a: &[i64], r: i64) -> bool {
        forall|i: usize| i < a.len() ==> #[trigger] a[i] <= r
    }

    spec fn has(a: &[i64], r: i64) -> bool {
        exists|i: usize| i < a.len() && #[trigger] a[i] == r
    }

    spec fn all_in(a: &[i64], lo: i64, hi: i64) -> bool {
        forall|i: usize| i < a.len() ==> lo <= #[trigger] a[i] && a[i] <= hi
    }

    spec fn pre_twice(x: i64) -> bool { x < 1000 && x > -1000 }

    spec fn post_twice(x: i64, r: i64) -> bool { r == 2 * x }

    spec fn pre_above(x: i64) -> bool { x < 1000 }

    spec fn post_above(x: i64, r: i64) -> bool { r >= x }

    spec fn pre_find(a: &[i64], k: i64) -> bool { sorted(a) && a.len() > 1 }

    spec fn post_find(a: &[i64], k: i64, r: usize) -> bool {
        r <= a.len() && (r < a.len() ==> a[r] == k)
    }

    spec fn pre_maxi(a: &[i64]) -> bool { a.len() > 0 }

    spec fn post_maxi(a: &[i64], r: i64) -> bool { upper(a, r) && has(a, r) }

    spec fn pre_within(a: &[i64], lo: i64, hi: i64) -> bool { lo <= hi }

    spec fn post_within(a: &[i64], lo: i64, hi: i64, r: bool) -> bool { r == all_in(a, lo, hi) }

    spec fn pre_sum(p: &Point, q: Point) -> bool { p.x < q.x && q.x < 100 && p.x > -100 }

    spec fn post_sum(p: &Point, q: Point, r: Point) -> bool {
        r.x == p.x + q.x && r.y == (p.y && q.y)
    }

    spec fn pre_bump(a: i64) -> bool { a < 1000 }

    spec fn post_bump(before: i64, after: i64, r: i64) -> bool { r >= 0 && after == before + 1 }

    spec fn pre_swap(v: &[u64]) -> bool { v.len() == 2 }

    spec fn post_swap(before: &[u64], after: &[u64]) -> bool {
        after[0] == before[1] && after[1] == before[0]
    }

    // A recursive definition and a quantifier over the slice (backlog
    // T1-44g): the solver decides this contract at a written input.
    spec fn tally(a: &[i64], n: usize) -> i64
        decreases n,
    {
        if n == 0 { 0 } else { tally(a, n - 1) + a[n - 1] }
    }

    spec fn pre_total(a: &[i64]) -> bool {
        a.len() > 0 && (forall|i: usize| i < a.len() ==> -100 < #[trigger] a[i] && a[i] < 100)
    }

    spec fn post_total(a: &[i64], r: i64) -> bool { r == tally(a, a.len()) }

    #[effects(pure)]
    #[uses(pre_total, post_total, tally)]
    fn total(a: &[i64]) -> (r: i64)
        requires pre_total(a),
        ensures post_total(a, r),
    {
        0
    }

    #[effects(pure)]
    #[uses(pre_twice, post_twice)]
    fn twice(x: i64) -> (r: i64)
        requires pre_twice(x),
        ensures post_twice(x, r),
    {
        2 * x
    }

    #[effects(pure)]
    #[uses(pre_above, post_above)]
    fn above(x: i64) -> (r: i64)
        requires pre_above(x),
        ensures post_above(x, r),
    {
        x
    }

    #[effects(pure)]
    #[uses(pre_find, post_find, sorted)]
    fn find(a: &[i64], k: i64) -> (r: usize)
        requires pre_find(a, k),
        ensures post_find(a, k, r),
    {
        a.len()
    }

    #[effects(pure)]
    #[uses(pre_maxi, post_maxi, upper, has)]
    fn maxi(a: &[i64]) -> (r: i64)
        requires pre_maxi(a),
        ensures post_maxi(a, r),
    {
        a[0]
    }

    #[effects(pure)]
    #[uses(pre_within, post_within, all_in)]
    fn within(a: &[i64], lo: i64, hi: i64) -> (r: bool)
        requires pre_within(a, lo, hi),
        ensures post_within(a, lo, hi, r),
    {
        true
    }

    #[effects(pure)]
    #[uses(pre_sum, post_sum)]
    fn sum(p: &Point, q: Point) -> (r: Point)
        requires pre_sum(p, q),
        ensures post_sum(p, q, r),
    {
        Point { x: p.x + q.x, y: p.y && q.y }
    }

    #[effects(pure)]
    #[uses(pre_bump, post_bump)]
    fn bump(a: &mut i64) -> (r: i64)
        requires pre_bump(*a),
        ensures post_bump(*old(a), *a, r),
    {
        *a = *a + 1;
        0
    }

    #[effects(pure)]
    #[uses(pre_swap, post_swap)]
    fn swap(v: &mut [u64]) -> ()
        requires pre_swap(v),
        ensures post_swap(old(v), v),
    {
        let t: u64 = v[0];
        v[0] = v[1];
        v[1] = t;
    }
"#;

fn src() -> Vec<(String, String)> {
    vec![("d.bmb".to_string(), format!("{SRC}}}\n"))]
}

/// `bmb examples ARGS -- FILES`: the exit status, stdout and stderr.
fn run(args: &[&str], files: &[(String, String)], env: &[(&str, &str)]) -> (i32, String, String) {
    let dir = cli::fresh_dir();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(&dir)
        .arg("examples")
        .args(args)
        .arg("--")
        .args(files.iter().map(|(n, _)| n))
        .envs(env.iter().copied())
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn rows(out: &str) -> Vec<Value> {
    out.lines().map(|l| json::parse(l).expect(l)).collect()
}

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn n(v: &Value, key: &str) -> i64 {
    v.get(key)
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("no number `{key}` in {}", v.to_json()))
}

/// The example rows and the summary of the one definition asked for.
fn of(def: &str, count: &str) -> (Vec<Value>, Value) {
    let (code, out, err) = run(&["--count", count, def], &src(), &[]);
    assert_eq!(code, 0, "{out}{err}");
    let mut all = rows(&out);
    let summary = all.pop().expect("a summary row");
    assert!(summary.get("summary").is_some(), "{out}");
    (all, summary)
}

fn input(v: &Value) -> String {
    v.get("input").map(Value::to_json).unwrap_or_default()
}

/// Rows of every kind, in order, with the fields of each; the summary
/// counts them.
#[test]
fn rows_and_summary_have_the_documented_shape() {
    let (code, out, err) = run(&["d::twice", "d::bump"], &src(), &[]);
    assert_eq!(code, 0, "{out}{err}");
    let all = rows(&out);
    let twice: Vec<&Value> = all.iter().filter(|r| s(r, "def") == "d::twice").collect();
    let (summary, ex) = twice.split_last().unwrap();
    // Per input: allowed, then rejected for the same input; then excluded.
    let kinds: Vec<&str> = ex.iter().map(|r| s(r, "kind")).collect();
    assert_eq!(
        kinds,
        [
            "allowed", "rejected", "allowed", "rejected", "allowed", "rejected", "excluded",
            "excluded", "excluded"
        ],
        "{out}"
    );
    for r in ex {
        let Some(Value::Object(fields)) = r.get("input") else {
            panic!("no input in {}", r.to_json());
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].0, "x");
        assert!(s(r, "input").is_empty());
        // The result of every row but an excluded one.
        assert_eq!(r.get("result").is_some(), s(r, "kind") != "excluded");
        assert!(r.get("after").is_none());
    }
    assert_eq!(
        summary.to_json(),
        format!(
            "{{\"def\":\"d::twice\",\"summary\":true,\"allowed\":3,\"rejected\":3,\"excluded\":3,\"undecided\":0,\"rlimit\":{},\"max_len\":4}}",
            n(summary, "rlimit")
        )
    );
    assert!(n(summary, "rlimit") > 0);
    // The definitions in the order asked; `after` for a `&mut` parameter.
    let last = all.last().unwrap();
    assert_eq!(s(last, "def"), "d::bump");
    assert!(last.get("summary").is_some());
    let bump: Vec<&Value> = all
        .iter()
        .filter(|r| s(r, "def") == "d::bump" && s(r, "kind") == "allowed")
        .collect();
    assert!(!bump.is_empty());
    for r in bump {
        let after = r.get("after").expect("after");
        assert!(after.get("a").is_some(), "{}", r.to_json());
    }
}

/// The same files give the same bytes, run after run, and `--count` is the
/// number of inputs.
#[test]
fn the_output_is_the_same_bytes_every_run() {
    let defs = [
        "d::twice",
        "d::above",
        "d::find",
        "d::maxi",
        "d::within",
        "d::sum",
        "d::bump",
        "d::swap",
    ];
    let (code, first, err) = run(&defs, &src(), &[]);
    assert_eq!(code, 0, "{first}{err}");
    for _ in 0..2 {
        assert_eq!(run(&defs, &src(), &[]).1, first);
    }
    let (ex, summary) = of("d::twice", "5");
    assert_eq!(n(&summary, "allowed"), 5);
    let mut inputs: Vec<String> = ex
        .iter()
        .filter(|r| s(r, "kind") == "allowed")
        .map(input)
        .collect();
    inputs.sort();
    inputs.dedup();
    assert_eq!(inputs.len(), 5, "each input differs from the earlier ones");
}

/// A contract that fixes the result: one allowed result per input, and a
/// rejected one for the same input.
#[test]
fn a_contract_that_fixes_the_result_has_one_allowed_row_per_input() {
    for def in ["d::twice", "d::maxi", "d::within"] {
        let (ex, summary) = of(def, "3");
        assert_eq!(n(&summary, "allowed"), 3, "{def}");
        assert_eq!(n(&summary, "rejected"), 3, "{def}");
        assert_eq!(n(&summary, "undecided"), 0, "{def}");
        let pairs: Vec<&Value> = ex.iter().filter(|r| s(r, "kind") != "excluded").collect();
        for pair in pairs.chunks(2) {
            assert_eq!(s(pair[0], "kind"), "allowed", "{def}");
            assert_eq!(s(pair[1], "kind"), "rejected", "{def}");
            assert_eq!(input(pair[0]), input(pair[1]), "{def}");
            assert_ne!(s(pair[0], "result"), s(pair[1], "result"), "{def}");
        }
    }
    // A `bool` result takes both values over the allowed rows.
    let (ex, _) = of("d::within", "3");
    let results: Vec<&str> = ex
        .iter()
        .filter(|r| s(r, "kind") == "allowed")
        .map(|r| s(r, "result"))
        .collect();
    assert!(results.contains(&"true") && results.contains(&"false"));
}

/// A contract that leaves the result open: two allowed rows share an input
/// and differ in the result.
#[test]
fn a_weak_contract_shows_two_allowed_results_for_one_input() {
    for def in ["d::above", "d::find"] {
        let (ex, summary) = of(def, "3");
        let allowed: Vec<&Value> = ex.iter().filter(|r| s(r, "kind") == "allowed").collect();
        assert_eq!(n(&summary, "allowed"), 6, "{def}");
        for pair in allowed.chunks(2) {
            assert_eq!(input(pair[0]), input(pair[1]), "{def}");
            assert_ne!(s(pair[0], "result"), s(pair[1], "result"), "{def}");
        }
    }
}

/// The inputs of a kind are not all of the shortest length: the n-th asks
/// for lengths of at least n first.
#[test]
fn slice_inputs_grow_with_the_row() {
    for def in ["d::maxi", "d::within"] {
        let (ex, _) = of(def, "4");
        let lens: Vec<usize> = ex
            .iter()
            .filter(|r| s(r, "kind") == "allowed")
            .map(|r| {
                let a = r.get("input").unwrap().get("a").unwrap().as_str().unwrap();
                let (_, len) = a.rsplit_once("(len ").unwrap();
                len.trim_end_matches(')').parse().unwrap()
            })
            .collect();
        assert_eq!(lens.len(), 4);
        for (k, len) in lens.iter().enumerate() {
            assert!(*len >= k.min(4) && *len <= 4, "{def}: {lens:?}");
        }
    }
}

/// How a parameter of a check is bound: a slice or an array from its
/// elements (`[1, 2] (len 2)`), anything else by its value.
fn bind(name: &str, ty: &str, value: &str) -> String {
    let elem = ty
        .strip_prefix("&mut [")
        .or_else(|| ty.strip_prefix("&["))
        .or_else(|| ty.strip_prefix('['))
        .map(|rest| rest.split([']', ';']).next().unwrap());
    match elem {
        Some(e) => {
            let (elems, len) = value.rsplit_once(" (len ").expect(value);
            let len = len.trim_end_matches(')');
            format!("        let {name}: [{e}; {len}] = {elems};\n")
        }
        None => {
            let ty = ty.trim_start_matches("&mut ").trim_start_matches('&');
            format!("        let {name}: {ty} = {value};\n")
        }
    }
}

/// One definition for the gate's check: its parameters (name, type), the
/// result's type, and the contract's two predicates as calls on the bound
/// names (`NAME_after` is the value behind a `&mut` parameter afterwards).
struct Case {
    def: &'static str,
    params: &'static [(&'static str, &'static str)],
    ret: Option<&'static str>,
    pre: &'static str,
    post: &'static str,
}

const CASES: &[Case] = &[
    Case {
        def: "d::twice",
        params: &[("x", "i64")],
        ret: Some("i64"),
        pre: "pre_twice(x)",
        post: "post_twice(x, r)",
    },
    Case {
        def: "d::above",
        params: &[("x", "i64")],
        ret: Some("i64"),
        pre: "pre_above(x)",
        post: "post_above(x, r)",
    },
    // Sortedness, quantified over the slice.
    Case {
        def: "d::find",
        params: &[("a", "&[i64]"), ("k", "i64")],
        ret: Some("usize"),
        pre: "pre_find(&a, k)",
        post: "post_find(&a, k, r)",
    },
    Case {
        def: "d::maxi",
        params: &[("a", "&[i64]")],
        ret: Some("i64"),
        pre: "pre_maxi(&a)",
        post: "post_maxi(&a, r)",
    },
    // "All in range", as the value of a `bool` result.
    Case {
        def: "d::within",
        params: &[("a", "&[i64]"), ("lo", "i64"), ("hi", "i64")],
        ret: Some("bool"),
        pre: "pre_within(&a, lo, hi)",
        post: "post_within(&a, lo, hi, r)",
    },
    Case {
        def: "d::sum",
        params: &[("p", "&Point"), ("q", "Point")],
        ret: Some("Point"),
        pre: "pre_sum(&p, q)",
        post: "post_sum(&p, q, r)",
    },
    Case {
        def: "d::bump",
        params: &[("a", "&mut i64")],
        ret: Some("i64"),
        pre: "pre_bump(a)",
        post: "post_bump(a, a_after, r)",
    },
    // A recursive sum under a quantified `requires`.
    Case {
        def: "d::total",
        params: &[("a", "&[i64]")],
        ret: Some("i64"),
        pre: "pre_total(&a)",
        post: "post_total(&a, r)",
    },
    Case {
        def: "d::swap",
        params: &[("v", "&mut [u64]")],
        ret: None,
        pre: "pre_swap(&v)",
        post: "post_swap(&v, &v_after)",
    },
];

/// The functions that assert what the rows of `case` claim, one per row,
/// with the claim's kind.
fn checks(case: &Case, ex: &[Value], from: usize) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (k, r) in ex.iter().enumerate() {
        let name = format!("check_{}", from + k);
        let mut body = String::new();
        for (p, ty) in case.params {
            let v = r.get("input").unwrap().get(p).unwrap().as_str().unwrap();
            body.push_str(&bind(p, ty, v));
            if let Some(a) = r.get("after").and_then(|a| a.get(p)) {
                body.push_str(&bind(&format!("{p}_after"), ty, a.as_str().unwrap()));
            }
        }
        if let (Some(ty), Some(v)) = (case.ret, r.get("result")) {
            body.push_str(&bind("r", ty, v.as_str().unwrap()));
        }
        let (pre, post) = (case.pre, case.post);
        body.push_str(&match s(r, "kind") {
            "allowed" => format!("        assert({pre});\n        assert({post});\n"),
            "rejected" => format!("        assert({pre});\n        assert(!{post});\n"),
            "excluded" => format!("        assert(!{pre});\n"),
            other => panic!("kind {other}"),
        });
        out.push((
            name.clone(),
            format!(
                "\n    #[effects(pure)]\n    #[uses({SPECS})]\n    fn {name}() -> () {{\n{body}    }}\n"
            ),
        ));
    }
    out
}

/// `bmb verify --def ..` of `names` in `text`: the definitions it proved.
fn proved(text: &str, names: &[String]) -> (Vec<String>, String) {
    let mut args = vec!["verify".to_string()];
    for name in names {
        args.push("--def".into());
        args.push(format!("d::{name}"));
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (_, out) = cli::run(&args, &[("d.bmb".to_string(), text.to_string())]);
    let ok = out
        .lines()
        .filter_map(|l| json::parse(l).ok())
        .filter(|r| s(r, "status") == "verified")
        .map(|r| s(&r, "def").to_string())
        .collect();
    (ok, out)
}

/// Every row is true of the contract, by a route that does not ask the
/// generator's query: the row's values as literals in a function that
/// asserts the claim (allowed: `requires` and `ensures` hold; rejected:
/// `requires` holds and `ensures` does not; excluded: `requires` does not
/// hold), which the proof gate proves.
#[test]
fn shown_examples_hold_at_the_gate() {
    let mut text = SRC.to_string();
    let mut names = Vec::new();
    let mut kinds = std::collections::BTreeMap::new();
    for case in CASES {
        let (ex, summary) = of(case.def, "3");
        assert_eq!(n(&summary, "undecided"), 0, "{}", case.def);
        assert!(n(&summary, "allowed") >= 3, "{}", case.def);
        assert!(n(&summary, "rejected") >= 3, "{}", case.def);
        assert!(n(&summary, "excluded") >= 1, "{}", case.def);
        for r in &ex {
            *kinds.entry(s(r, "kind").to_string()).or_insert(0) += 1;
        }
        for (name, f) in checks(case, &ex, names.len()) {
            names.push(name);
            text.push_str(&f);
        }
    }
    text.push_str("}\n");
    assert!(names.len() >= 60, "{} rows", names.len());
    assert_eq!(kinds.len(), 3, "{kinds:?}");
    let (ok, out) = proved(&text, &names);
    let missing: Vec<&String> = names
        .iter()
        .filter(|name| !ok.contains(&format!("d::{name}")))
        .collect();
    assert!(missing.is_empty(), "not proved: {missing:?}\n{out}\n{text}");
}

/// The check itself tells a true claim from a false one: the same rows
/// with `allowed` and `rejected` exchanged are not proved.
#[test]
fn the_gate_refuses_an_exchanged_claim() {
    let case = &CASES[0];
    let (ex, _) = of(case.def, "2");
    let flipped: Vec<Value> = ex
        .iter()
        .filter(|r| s(r, "kind") != "excluded")
        .map(|r| {
            let to = if s(r, "kind") == "allowed" {
                "rejected"
            } else {
                "allowed"
            };
            json::parse(&r.to_json().replace(s(r, "kind"), to)).unwrap()
        })
        .collect();
    assert_eq!(flipped.len(), 4);
    let mut text = SRC.to_string();
    let mut names = Vec::new();
    for (name, f) in checks(case, &flipped, 0) {
        names.push(name);
        text.push_str(&f);
    }
    text.push_str("}\n");
    let (ok, out) = proved(&text, &names);
    assert!(ok.is_empty(), "{out}");
}

/// A query the solver does not decide shows no row and is counted; no
/// value is printed that was not a model. Nonlinear integer arithmetic is
/// such a contract (a recursive spec fn was, until the generator wrote it
/// as a recursive function and tried written inputs, backlog T1-44g): no
/// written input is a solution, and the solver finds none.
#[test]
fn an_undecided_query_shows_nothing_and_is_counted() {
    let text = r#"#[summary("t")]
mod c {
    spec fn cubes(x: i64, y: i64, z: i64) -> bool { x * x * x + y * y * y + z * z * z == 42 }

    #[effects(pure)]
    #[uses(cubes)]
    fn first(x: i64, y: i64, z: i64) -> (r: i64)
        requires cubes(x, y, z),
        ensures r == x,
    {
        x
    }
}
"#;
    let files = vec![("c.bmb".to_string(), text.to_string())];
    let (code, out, err) = run(&["c::first"], &files, &[]);
    assert_eq!(code, 0, "{out}{err}");
    let all = rows(&out);
    let r = all.last().unwrap();
    assert!(r.get("summary").is_some());
    assert_eq!(n(r, "allowed") + n(r, "rejected"), 0, "{out}");
    // The allowed rows and the rejected rows each ended at a query that
    // was not decided; the inputs the `requires` exclude are found.
    assert_eq!(n(r, "undecided"), 2, "{out}");
    assert!(n(r, "excluded") >= 1, "{out}");
    assert_eq!(all.len() as i64, 1 + n(r, "excluded"), "{out}");
    assert!(r.get("reason").is_none());
}

/// A type whose values are not shown in full: a summary row with the
/// reason, no example, no query. A contract through a spec fn the
/// function's proof does not read (not in `#[uses]`) has the examples of
/// the same contract with it revealed: the contract means what its spec
/// fns are defined to be (backlog T1-44g).
#[test]
fn a_definition_without_examples_says_why() {
    let text = r#"#[summary("t")]
mod u {
    spec fn big(r: i64) -> bool { huge(r) }

    spec fn huge(r: i64) -> bool { r > 10 }

    #[effects(pure)]
    fn first(a: Option<i64>) -> (r: i64)
        ensures r == 0,
    {
        0
    }

    #[effects(pure)]
    fn half(x: i64) -> (r: f64)
        ensures x == 0 ==> r == 0.0,
    {
        0.0
    }

    #[effects(pure)]
    fn long(a: [i64; 5]) -> (r: i64)
        ensures r == a[0],
    {
        a[0]
    }

    #[effects(pure)]
    #[uses(big)]
    fn opaque() -> (r: i64)
        ensures big(r),
    {
        11
    }

    #[effects(pure)]
    #[uses(big, huge)]
    fn clear() -> (r: i64)
        ensures big(r),
    {
        11
    }
}
"#;
    let files = vec![("u.bmb".to_string(), text.to_string())];
    let defs = ["u::first", "u::half", "u::long", "u::opaque", "u::clear"];
    let (code, out, err) = run(&defs, &files, &[]);
    assert_eq!(code, 0, "{out}{err}");
    let all = rows(&out);
    let reasons: Vec<&str> = all
        .iter()
        .filter(|r| r.get("summary").is_some())
        .map(|r| s(r, "reason"))
        .collect();
    assert_eq!(
        reasons,
        [
            "parameter `a` of type `Option<i64>`: its values are not shown in full",
            "result of type `f64`: its values are not shown in full",
            "parameter `a` of type `[i64; 5]`: its values are not shown in full",
            "",
            "",
        ]
    );
    for r in all.iter().filter(|r| !s(r, "reason").is_empty()) {
        assert_eq!(n(r, "rlimit"), 0);
        assert_eq!(n(r, "undecided"), 0);
    }
    // The two definitions of one contract have the same rows, whichever
    // spec fns their proofs read.
    let of = |def: &str| -> Vec<String> {
        out.lines()
            .filter(|l| l.contains(&format!("\"def\":\"{def}\"")))
            .map(|l| l.replace(def, "u::f"))
            .collect()
    };
    assert_eq!(of("u::opaque"), of("u::clear"), "{out}");
    assert!(all.iter().all(|r| r.get("summary").is_some()
        || ["u::opaque", "u::clear"].contains(&s(r, "def"))));
    let shown: Vec<&Value> = all.iter().filter(|r| s(r, "kind") == "allowed").collect();
    assert!(!shown.is_empty());
    for r in shown {
        assert!(s(r, "result").parse::<i64>().unwrap() > 10);
    }
}

/// Usage errors end with status 2 on standard error; a path that is not an
/// executable function and a program that does not check end with 1.
#[test]
fn errors() {
    let (code, out, err) = run(&["--count", "0", "d::twice"], &src(), &[]);
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(err.contains("--count takes a number from 1 to 64"), "{err}");
    // No definition.
    let (code, out, err) = run(&[], &src(), &[]);
    assert_eq!((code, out.as_str()), (2, ""));
    assert!(err.contains("bmb examples [--count N] DEF.."), "{err}");
    // No `--`.
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["examples", "d::twice", "d.bmb"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("put `--` between"));
    for def in ["d::nothing", "d::sorted"] {
        let (code, out, _) = run(&[def], &src(), &[]);
        assert_eq!(code, 1);
        assert_eq!(out, format!("`{def}` is not an executable function\n"));
    }
    let bad = vec![("d.bmb".to_string(), "mod d { fn f( }".to_string())];
    let (code, out, _) = run(&["d::f"], &bad, &[]);
    assert_eq!(code, 1);
    assert!(out.contains("\"severity\":\"error\""), "{out}");
}
