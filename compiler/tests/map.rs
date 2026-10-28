//! Maps and read queries (ADR-0020, backlog 1D-05, 1D-06, 1D-12).

const A: &str = r#"
#[summary("탐색 대상")]
mod lib {
    spec fn pos(x: i64) -> bool { x > 0 }

    spec fn is_dec(x: i64, r: i64) -> bool { r == x - 1 }

    #[effects(pure)]
    #[uses(pos, is_dec)]
    fn dec(x: i64) -> (r: i64)
        requires pos(x),
        ensures is_dec(x, r),
    {
        x - 1
    }

    #[effects(pure)]
    fn zero(a: &mut [i64]) -> () {
        let mut i: usize = 0;
        while i < a.len()
            invariant i <= a.len(),
            decreases a.len() - i,
        {
            a[i] = 0;
            i = i + 1;
        }
    }
}
"#;

const B: &str = r#"
#[summary("호출자")]
mod app {
    #[effects(io)]
    #[uses(lib::pos)]
    fn main() -> () {
        let mut xs: [i64; 2] = [5, 6];
        lib::zero(&mut xs);
        print_i64(lib::dec(5));
    }
}
"#;

fn files(parts: &[(&str, &str)]) -> Vec<(String, String)> {
    parts
        .iter()
        .map(|(n, t)| (n.to_string(), t.to_string()))
        .collect()
}

fn run(f: &[(String, String)], req: &[&str]) -> String {
    let req: Vec<String> = req.iter().map(|s| s.to_string()).collect();
    bmb::map_request(f, &req).unwrap()
}

/// Drops the location keys, which follow the file layout by design.
fn without_locations(s: &str) -> String {
    s.lines()
        .map(|l| match l.find(",\"file\":") {
            Some(i) => format!("{}}}", &l[..i]),
            None => l.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn maps_are_deterministic_and_layout_independent() {
    let split = files(&[("lib.bmb", A), ("app.bmb", B)]);
    let joined = files(&[("all.bmb", &format!("{B}\n{A}"))]);
    let m1 = run(&split, &["map", "all"]);
    assert_eq!(m1, run(&split, &["map", "all"]));
    let m2 = run(&joined, &["map", "all"]);
    assert_eq!(without_locations(&m1), without_locations(&m2));
    assert!(m1.starts_with("{\"schema\":\"bmb.map\",\"version\":1}\n"));
}

#[test]
fn every_map_row_level_is_present() {
    let m = run(&files(&[("a.bmb", A), ("b.bmb", B)]), &["map", "all"]);
    for needle in [
        "\"level\":\"L0\",\"kind\":\"unit\",\"path\":\"app\",\"summary\":\"호출자\"",
        "\"level\":\"L0\",\"kind\":\"def\",\"path\":\"lib::dec\",\"def_kind\":\"fn\"",
        "\"level\":\"L1\",\"path\":\"lib::dec\",\"def_kind\":\"fn\",\"signature\":\"fn dec(x: i64) -> (r: i64)\",\"requires\":[\"pos(x)\"],\"ensures\":[\"is_dec(x, r)\"],\"decreases\":null,\"effects\":\"pure\",\"uses\":[\"lib::is_dec\",\"lib::pos\"],\"status\":\"verified\"",
        "{\"level\":\"L2\",\"from\":\"app::main\",\"to\":\"lib::zero\",\"kind\":\"writes\",\"arg\":0}",
        "{\"level\":\"L2\",\"from\":\"app::main\",\"to\":\"print_i64\",\"kind\":\"calls\"}",
        "{\"level\":\"L2\",\"from\":\"lib::dec\",\"to\":\"lib::pos\",\"kind\":\"contract\"}",
        "{\"level\":\"L2\",\"from\":\"app::main\",\"to\":\"lib::pos\",\"kind\":\"uses\"}",
    ] {
        assert!(m.contains(needle), "missing {needle}\n{m}");
    }
}

#[test]
fn queries_answer_from_the_maps() {
    let f = files(&[("a.bmb", A), ("b.bmb", B)]);
    let edges_in = run(&f, &["query", "edges", "lib::dec", "in"]);
    assert!(edges_in.contains("\"from\":\"app::main\",\"to\":\"lib::dec\",\"kind\":\"calls\""));
    assert!(!edges_in.contains("\"from\":\"lib::dec\""));

    let two = run(&f, &["query", "edges", "app::main", "out", "2"]);
    assert!(
        two.contains("\"from\":\"lib::dec\",\"to\":\"lib::is_dec\""),
        "{two}"
    );

    let slice = run(&f, &["query", "slice", "lib::dec"]);
    assert!(slice.contains("\"path\":\"lib::pos\""));
    assert!(
        slice.contains("spec fn pos(x: i64) -> bool {\\n    x > 0\\n}"),
        "{slice}"
    );
    assert!(slice.contains("\"approx_tokens\":"));

    let found = run(&f, &["query", "find", "zero"]);
    assert!(found.contains("\"path\":\"lib::zero\""));

    let c = run(&f, &["query", "contracts", "lib::pos"]);
    assert!(c.contains("\"required_by\":[\"lib::dec\"],\"ensured_by\":[],\"uses_by\":[\"app::main\",\"lib::dec\"]"), "{c}");

    let st = run(&f, &["query", "status"]);
    assert!(
        st.contains("{\"summary\":true,\"defs\":5,\"verified\":3,\"failed\":0,\"io_defs\":1}"),
        "{st}"
    );

    let surface = run(&f, &["query", "surface", "lib::zero"]);
    assert!(surface.contains("\"signature\":\"fn zero(a: &mut [i64]) -> ()\""));

    let units = run(&f, &["query", "units"]);
    assert_eq!(
        units
            .lines()
            .filter(|l| l.contains("\"kind\":\"unit\""))
            .count(),
        2
    );
}

#[test]
fn bad_queries_are_errors() {
    let f = files(&[("a.bmb", A), ("b.bmb", B)]);
    let req =
        |r: &[&str]| bmb::map_request(&f, &r.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    assert!(req(&["query", "surface", "nope::nope"]).is_err());
    assert!(req(&["query", "edges", "lib::dec", "sideways"]).is_err());
    assert!(req(&["query", "teleport"]).is_err());
}

#[test]
fn duplicates_groups_equal_bodies_across_names_and_units() {
    // `lib2::dec2` is `lib::dec` under another name and unit; the spec fns
    // repeat too. `lib2::other` has no twin.
    const C: &str = r#"
#[summary("복제")]
mod lib2 {
    spec fn pos(x: i64) -> bool { x > 0 }

    spec fn is_dec(x: i64, r: i64) -> bool { r == x - 1 }

    #[effects(pure)]
    #[uses(lib::pos, lib::is_dec)]
    fn dec2(x: i64) -> (r: i64)
        requires lib::pos(x),
        ensures lib::is_dec(x, r),
    {
        x - 1
    }

    #[effects(pure)]
    fn other(x: i64) -> i64 {
        x
    }
}
"#;
    let f = files(&[("a.bmb", A), ("b.bmb", B), ("c.bmb", C)]);
    let out = run(&f, &["query", "duplicates"]);
    let rows: Vec<&str> = out.lines().skip(1).collect();
    let groups: Vec<&str> = rows
        .iter()
        .copied()
        .filter(|r| r.contains("\"defs\":["))
        .collect();
    for want in [
        "\"defs\":[\"lib2::dec2\",\"lib::dec\"]}",
        "\"defs\":[\"lib2::pos\",\"lib::pos\"]}",
        "\"defs\":[\"lib2::is_dec\",\"lib::is_dec\"]}",
    ] {
        assert!(groups.iter().any(|r| r.ends_with(want)), "{want}\n{out}");
    }
    assert!(!out.contains("lib2::other"), "{out}");
    assert_eq!(
        *rows.last().unwrap(),
        "{\"summary\":true,\"groups\":3,\"duplicate_defs\":6,\"defs\":9}"
    );
    assert!(rows[0].starts_with("{\"hash\":\"sha256:"), "{out}");
    let hashes: Vec<&str> = groups.iter().map(|r| &r[..80]).collect();
    let mut sorted = hashes.clone();
    sorted.sort();
    assert_eq!(hashes, sorted);
    // Same answer for another file order.
    let g = files(&[("c.bmb", C), ("b.bmb", B), ("a.bmb", A)]);
    assert_eq!(run(&g, &["query", "duplicates"]), out);
    // No duplicates: only the summary row.
    let h = files(&[("a.bmb", A), ("b.bmb", B)]);
    let none = run(&h, &["query", "duplicates"]);
    assert_eq!(
        none.lines().nth(1).unwrap(),
        "{\"summary\":true,\"groups\":0,\"duplicate_defs\":0,\"defs\":5}"
    );
}

/// The alpha-equivalence corpus (`tests/duplicates/*.bmb`, ADR-0059 §6):
/// definitions that differ only in parameter, local, label, field or
/// definition names, recursive and mutually recursive ones and callers of
/// duplicates group together, exactly as each file's `// group:` lines say.
/// Each group shares one `id` in the L1 rows.
#[test]
fn duplicates_groups_alpha_equivalent_definitions() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/duplicates");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    for p in paths {
        let text = std::fs::read_to_string(&p).unwrap();
        let mut want: Vec<Vec<String>> = text
            .lines()
            .filter_map(|l| l.strip_prefix("// group: "))
            .map(|g| {
                let mut v: Vec<String> = g.split_whitespace().map(String::from).collect();
                v.sort();
                v
            })
            .collect();
        want.sort();
        let f = files(&[("x.bmb", &text)]);
        let out = run(&f, &["query", "duplicates"]);
        let mut got: Vec<Vec<String>> = out
            .lines()
            .filter(|r| r.contains("\"defs\":["))
            .map(|r| {
                let defs = &r[r.find("\"defs\":[").unwrap() + 8..r.rfind(']').unwrap()];
                defs.split(',')
                    .map(|d| d.trim_matches('"').to_string())
                    .collect()
            })
            .collect();
        got.sort();
        assert_eq!(got, want, "{}\n{out}", p.display());
        // The L1 `id` of a group's members is the group's.
        let l1 = run(&f, &["map", "L1"]);
        for g in &want {
            let ids: std::collections::BTreeSet<&str> = l1
                .lines()
                .filter(|r| g.iter().any(|d| r.contains(&format!("\"path\":\"{d}\""))))
                .map(|r| {
                    let i = r.find("\"id\":\"").unwrap() + 6;
                    &r[i..i + r[i..].find('"').unwrap()]
                })
                .collect();
            assert_eq!(ids.len(), 1, "{g:?}\n{l1}");
        }
    }
}

/// A huge depth stops once nothing new is reachable (backlog 1D-14).
#[test]
fn edges_with_a_huge_depth_terminate() {
    let f = files(&[("a.bmb", A), ("b.bmb", B)]);
    let t = std::time::Instant::now();
    let all = run(&f, &["query", "edges", "app::main", "both", "100000000000"]);
    let max = run(
        &f,
        &[
            "query",
            "edges",
            "app::main",
            "out",
            &usize::MAX.to_string(),
        ],
    );
    assert!(t.elapsed() < std::time::Duration::from_secs(5));
    assert_eq!(all, run(&f, &["query", "edges", "app::main", "both", "50"]));
    assert_eq!(max, run(&f, &["query", "edges", "app::main", "out", "50"]));
    assert!(
        max.contains("\"from\":\"lib::dec\",\"to\":\"lib::is_dec\""),
        "{max}"
    );
}

// Backlog 2D-11: call paths to `io` (DESIGN.md §7.2 effect view).
const IO: &str = r#"
#[summary("효과 경로")]
mod app {
    #[effects(io)]
    fn emit(x: i64) -> () {
        print_i64(x);
    }

    #[effects(io)]
    fn b_log(x: i64) -> () {
        emit(x);
    }

    #[effects(io)]
    fn a_log(x: i64) -> () {
        emit(x);
    }

    #[effects(io)]
    fn quiet() -> () {
        let x: i64 = twice(1);
    }

    #[effects(pure)]
    fn twice(x: i64) -> i64 {
        x * 2
    }

    #[effects(io)]
    fn main() -> () {
        b_log(twice(1));
        a_log(2);
        quiet();
    }
}
"#;

/// The remaining test surface (backlog 2D-21): definitions whose calls reach
/// a built-in `io` operation, and definitions with a failed obligation
/// (`twice` can overflow); `quiet` is declared `io` but reaches none, and
/// spec fns never count.
#[test]
fn tests_lists_the_definitions_that_still_need_tests() {
    let f = files(&[("io.bmb", IO)]);
    let out = run(&f, &["query", "tests"]);
    let expected = [
        "{\"schema\":\"bmb.map\",\"version\":1}",
        "{\"path\":\"app::a_log\",\"test_reasons\":[\"io\"],\"sites\":0,\"lines\":4}",
        "{\"path\":\"app::b_log\",\"test_reasons\":[\"io\"],\"sites\":0,\"lines\":4}",
        "{\"path\":\"app::emit\",\"test_reasons\":[\"io\"],\"sites\":1,\"lines\":4}",
        "{\"path\":\"app::main\",\"test_reasons\":[\"io\"],\"sites\":0,\"lines\":6}",
        "{\"path\":\"app::twice\",\"test_reasons\":[\"unproved\"],\"sites\":0,\"lines\":4}",
        "{\"summary\":true,\"exec_defs\":6,\"test_needed_defs\":5,\"io_defs\":4,\"unproved_defs\":1,\"test_sites\":1,\"test_lines\":22}",
    ];
    assert_eq!(out, expected.join("\n") + "\n");
    assert_eq!(out, run(&f, &["query", "tests"]));

    // Proved pure code needs no test: only `main`, with its one `print_i64`.
    let g = files(&[("a.bmb", A), ("b.bmb", B)]);
    let rows: Vec<String> = run(&g, &["query", "tests"])
        .lines()
        .skip(1)
        .map(str::to_string)
        .collect();
    assert_eq!(
        rows,
        [
            "{\"path\":\"app::main\",\"test_reasons\":[\"io\"],\"sites\":1,\"lines\":7}",
            "{\"summary\":true,\"exec_defs\":3,\"test_needed_defs\":1,\"io_defs\":1,\"unproved_defs\":0,\"test_sites\":1,\"test_lines\":7}",
        ]
    );
}

#[test]
fn effects_lists_shortest_paths_to_io() {
    let f = files(&[("io.bmb", IO)]);
    let one = run(&f, &["query", "effects", "app::main"]);
    let expected = [
        "{\"schema\":\"bmb.map\",\"version\":1}",
        "{\"def\":\"app::main\",\"effect\":\"io\",\"path\":[\"app::main\",\"app::a_log\"]}",
        "{\"def\":\"app::main\",\"effect\":\"io\",\"path\":[\"app::main\",\"app::b_log\"]}",
        // Both loggers reach `emit` in two steps: the smaller path wins.
        "{\"def\":\"app::main\",\"effect\":\"io\",\"path\":[\"app::main\",\"app::a_log\",\"app::emit\"]}",
        "{\"def\":\"app::main\",\"effect\":\"io\",\"path\":[\"app::main\"]}",
        "{\"def\":\"app::main\",\"effect\":\"io\",\"path\":[\"app::main\",\"app::quiet\"]}",
        "{\"def\":\"app::main\",\"effect\":\"io\",\"path\":[\"app::main\",\"app::a_log\",\"app::emit\",\"print_i64\"]}",
        "{\"summary\":true,\"def\":\"app::main\",\"io_reached\":6,\"defs\":6}",
    ];
    assert_eq!(one, expected.join("\n") + "\n");

    // A pure definition reaches no `io`.
    assert_eq!(
        run(&f, &["query", "effects", "app::twice"])
            .lines()
            .nth(1)
            .unwrap(),
        "{\"summary\":true,\"def\":\"app::twice\",\"io_reached\":0,\"defs\":6}"
    );

    // Without a definition: every definition that reaches an `io`
    // operation, with its nearest one; `quiet` is declared `io` but
    // reaches none.
    let all = run(&f, &["query", "effects"]);
    let expected = [
        "{\"schema\":\"bmb.map\",\"version\":1}",
        "{\"def\":\"app::a_log\",\"effect\":\"io\",\"path\":[\"app::a_log\",\"app::emit\",\"print_i64\"]}",
        "{\"def\":\"app::b_log\",\"effect\":\"io\",\"path\":[\"app::b_log\",\"app::emit\",\"print_i64\"]}",
        "{\"def\":\"app::emit\",\"effect\":\"io\",\"path\":[\"app::emit\",\"print_i64\"]}",
        "{\"def\":\"app::main\",\"effect\":\"io\",\"path\":[\"app::main\",\"app::a_log\",\"app::emit\",\"print_i64\"]}",
        "{\"summary\":true,\"io_reaching\":4,\"io_defs\":5,\"defs\":6}",
    ];
    assert_eq!(all, expected.join("\n") + "\n");

    // Deterministic, and independent of the order of definitions and calls.
    assert_eq!(one, run(&f, &["query", "effects", "app::main"]));
    let swapped = IO
        .replace("fn b_log", "fn TMP")
        .replace("fn a_log", "fn b_log")
        .replace("fn TMP", "fn a_log")
        .replace(
            "b_log(twice(1));\n        a_log(2);",
            "a_log(twice(1));\n        b_log(2);",
        );
    assert_ne!(swapped, IO);
    let g = files(&[("io.bmb", &swapped)]);
    assert_eq!(all, run(&g, &["query", "effects"]));
    assert_eq!(one, run(&g, &["query", "effects", "app::main"]));

    let req =
        |r: &[&str]| bmb::map_request(&f, &r.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    assert!(req(&["query", "effects", "app::nope"]).is_err());
}

/// Data types in the maps (ADR-0026 §6, backlog 2B-07): types are L0 `def`
/// rows with `def_kind` `struct`/`enum`, `impl` functions are `unit::T::f`,
/// L2 `types` edges, type impact, and variant constructors are no calls.
#[test]
fn data_types_are_mapped() {
    const SRC: &str = r#"#[summary("types")]
mod geo {
    struct Point { x: i64, y: i64 }

    enum Shape { Dot, At(Point) }

    struct Other { z: i64 }

    impl Point {
        #[effects(pure)]
        fn norm(self: &Point) -> i64 {
            0
        }
    }

    #[effects(pure)]
    fn make(p: Point) -> Shape {
        let n: i64 = Point::norm(&p);
        Shape::At(p)
    }

    #[effects(pure)]
    fn other(o: Other) -> i64 {
        o.z
    }

    #[effects(pure)]
    fn wrap(a: i64) -> i64 {
        let o: Option<i64> = Some(a);
        a
    }
}
"#;
    let f = files(&[("t.bmb", SRC)]);
    let l0 = run(&f, &["map", "L0"]);
    assert!(
        l0.contains(r#""path":"geo::Point","def_kind":"struct""#),
        "{l0}"
    );
    assert!(
        l0.contains(r#""path":"geo::Shape","def_kind":"enum""#),
        "{l0}"
    );
    assert!(
        l0.contains(r#""path":"geo::Point::norm","def_kind":"fn""#),
        "{l0}"
    );
    let l1 = run(&f, &["query", "surface", "geo::Shape"]);
    assert!(
        l1.contains(r#""signature":"enum Shape { Dot, At(Point) }""#),
        "{l1}"
    );
    let l2 = run(&f, &["map", "L2"]);
    assert!(
        l2.contains(r#""from":"geo::make","to":"geo::Point::norm","kind":"calls""#),
        "{l2}"
    );
    assert!(
        l2.contains(r#""from":"geo::make","to":"geo::Shape","kind":"types""#),
        "{l2}"
    );
    // `Some(a)` and `Shape::At(p)` build values: no `calls` edge.
    assert!(
        !l2.contains(r#""to":"Some""#) && !l2.contains("Shape::At"),
        "{l2}"
    );
    let hit = run(&f, &["query", "impact", "geo::Point", "iface"]);
    for d in ["geo::make", "geo::Point::norm"] {
        assert!(hit.contains(&format!(r#""path":"{d}""#)), "{hit}");
    }
    assert!(
        !hit.contains("geo::other") && !hit.contains("geo::wrap"),
        "{hit}"
    );
}

#[test]
fn proof_fns_and_their_callers_are_mapped() {
    // ADR-0062 §3: a lemma is a definition of its own kind, and an L2
    // `proof_calls` edge names every definition that relies on it (from a
    // `proof` block or another lemma, itself when it is an induction).
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/run/max_list.bmb"),
    )
    .unwrap();
    let f = files(&[("m.bmb", &text)]);
    let m = run(&f, &["map", "all"]);
    for needle in [
        "{\"level\":\"L0\",\"kind\":\"def\",\"path\":\"ml::contains_has_index\",\"def_kind\":\"proof fn\"",
        "{\"level\":\"L1\",\"path\":\"ml::contains_has_index\",\"def_kind\":\"proof fn\",\"signature\":\"proof fn contains_has_index(a: &[i64], n: usize, r: i64)\",\"requires\":[\"prefix_ok(a, n, r)\"],\"ensures\":[\"has_index_below(a, n, r)\"],\"decreases\":\"n\",\"effects\":\"proof\"",
        "{\"level\":\"L2\",\"from\":\"ml::max_of_list\",\"to\":\"ml::contains_has_index\",\"kind\":\"proof_calls\"}",
        "{\"level\":\"L2\",\"from\":\"ml::contains_has_index\",\"to\":\"ml::contains_has_index\",\"kind\":\"proof_calls\"}",
        "{\"level\":\"L2\",\"from\":\"ml::total\",\"to\":\"ml::sum_grows\",\"kind\":\"proof_calls\"}",
        // A spec fn named in a lemma's body is read as a specification.
        "{\"level\":\"L2\",\"from\":\"ml::contains_has_index\",\"to\":\"ml::prefix_ok\",\"kind\":\"contract\"}",
    ] {
        assert!(m.contains(needle), "missing {needle}\n{m}");
    }
    // The lemma's status is its own proof.
    assert!(m.contains("\"path\":\"ml::sum_grows\",\"def_kind\":\"proof fn\""));
    assert!(!m.contains("\"kind\":\"calls\",\"to\":\"ml::sum_grows\""));
    // Changing a lemma's contract reproves the definitions that call it.
    let hit = run(&f, &["query", "impact", "ml::contains_has_index"]);
    assert!(
        hit.contains("{\"path\":\"ml::max_of_list\",\"reason\":\"calls\"}"),
        "{hit}"
    );
    // A lemma needs no test: it does not run.
    let t = run(&f, &["query", "tests"]);
    assert!(!t.contains("contains_has_index"), "{t}");
}

#[test]
fn a_type_shows_its_invariant_and_slices_carry_the_predicate() {
    // ADR-0060 §1, §3: the invariant is on the type's L1 surface, and a
    // slice that holds the type holds the predicate it means.
    const SRC: &str = r#"#[summary("inv")]
mod iv {
    #[invariant(ordered)]
    struct Span { lo: i64, hi: i64 }

    spec fn ordered(s: &Span) -> bool { s.lo <= s.hi }

    #[effects(pure)]
    fn width(s: Span) -> i64 {
        0
    }
}
"#;
    let f = files(&[("t.bmb", SRC)]);
    let l1 = run(&f, &["query", "surface", "iv::Span"]);
    assert!(
        l1.contains(r##""signature":"#[invariant(ordered)] struct Span { lo: i64, hi: i64 }""##),
        "{l1}"
    );
    let slice = run(&f, &["query", "slice", "iv::width"]);
    assert!(
        slice.contains(r#""path":"iv::ordered","text":"spec fn ordered(s: &Span) -> bool {"#),
        "{slice}"
    );
}
