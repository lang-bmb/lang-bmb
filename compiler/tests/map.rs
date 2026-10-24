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
