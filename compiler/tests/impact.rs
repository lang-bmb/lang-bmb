//! The `impact` query (backlog 2D-02) is complete: when a definition's body
//! changes, every definition whose proof script changes is in the reported
//! set. The proof scripts are exactly what the solver sees, so they are the
//! ground truth for "this proof must be redone".

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn files(text: &str) -> Vec<(String, String)> {
    vec![("c.bmb".to_string(), text.to_string())]
}

fn rows(out: &str) -> Vec<bmb::json::Value> {
    out.lines()
        .filter(|l| !l.is_empty())
        .map(|l| bmb::json::parse(l).unwrap())
        .collect()
}

fn s<'a>(v: &'a bmb::json::Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(|x| x.as_str())
}

/// Index of the `{` that opens the body of `unit::name`.
fn body_open(text: &str, unit: &str, name: &str) -> usize {
    let m = text.find(&format!("mod {unit} {{")).unwrap();
    let f = m + text[m..].find(&format!("fn {name}(")).unwrap();
    f + text[f..].find('{').unwrap()
}

fn matching_close(text: &str, open: usize) -> usize {
    let mut depth = 0;
    for (i, c) in text[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + i;
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced braces")
}

/// The same program with the body of `path` changed but its meaning kept.
fn change_body(text: &str, path: &str, spec: bool) -> String {
    let (unit, name) = path.rsplit_once("::").unwrap();
    let open = body_open(text, unit, name);
    if spec {
        let close = matching_close(text, open);
        let body = &text[open + 1..close];
        // The other branch is never taken; a second copy of `body` would bind
        // its quantified names twice.
        let header = &text[..open];
        let other = if header[header.rfind("fn ").unwrap()..].contains("-> bool") {
            "false"
        } else {
            "0"
        };
        format!(
            "{}{{ if true {{ {body} }} else {{ {other} }} }}{}",
            &text[..open],
            &text[close + 1..]
        )
    } else {
        format!(
            "{} let impact_probe: i64 = 0;{}",
            &text[..=open],
            &text[open + 1..]
        )
    }
}

#[test]
fn body_changes_reach_only_the_reported_definitions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut reached_others = 0;
    for rel in ["run/search.bmb"] {
        let text = fs::read_to_string(root.join(rel)).unwrap();
        let base = bmb::verify_sources(&files(&text));
        assert!(base.ok, "{rel}: {}", base.output);
        let l0 = bmb::map_request(&files(&text), &["map".into(), "L0".into()]).unwrap();
        let defs: Vec<(String, bool)> = rows(&l0)
            .iter()
            .filter(|r| s(r, "kind") == Some("def"))
            .map(|r| {
                (
                    s(r, "path").unwrap().to_string(),
                    s(r, "def_kind") == Some("spec fn"),
                )
            })
            .collect();
        assert!(defs.len() > 10, "{rel}");
        for (path, spec) in &defs {
            let changed = change_body(&text, path, *spec);
            let after = bmb::verify_sources(&files(&changed));
            assert!(
                after.ok,
                "{rel}: changing {path} broke it:\n{changed}\n{}",
                after.output
            );
            let touched: BTreeSet<&String> = base
                .scripts
                .keys()
                .chain(after.scripts.keys())
                .filter(|k| base.scripts.get(*k) != after.scripts.get(*k))
                .collect();
            let impact = bmb::map_request(
                &files(&text),
                &["query".into(), "impact".into(), path.clone(), "body".into()],
            )
            .unwrap();
            let reported: BTreeSet<String> = rows(&impact)
                .iter()
                .filter_map(|r| s(r, "path").map(str::to_string))
                .collect();
            reached_others += touched.iter().filter(|t| **t != path).count();
            for t in &touched {
                assert!(
                    reported.contains(*t),
                    "{rel}: changing the body of {path} changed the proof of {t}, which impact does not report ({reported:?})"
                );
            }
        }
    }
    // Not vacuous: spec fn bodies do reach other proofs.
    assert!(reached_others > 0);
}

#[test]
fn contract_changes_reach_callers_and_spec_bodies_reach_revealers() {
    let text = r#"
#[summary("u")]
mod u {
    spec fn p(x: i64) -> bool { x > 0 }

    spec fn q(x: i64) -> bool { p(x) }

    #[effects(pure)]
    #[uses(p)]
    fn f(x: i64) -> i64
        requires p(x),
    {
        x - 1
    }

    #[effects(pure)]
    #[uses(p)]
    fn g(x: i64) -> i64
        requires p(x),
    {
        f(x)
    }

    #[effects(pure)]
    fn h(x: i64) -> i64
        requires q(x),
    {
        x
    }
}
"#;
    let ask = |def: &str, change: &str| -> Vec<(String, String)> {
        let out = bmb::map_request(
            &files(text),
            &["query".into(), "impact".into(), def.into(), change.into()],
        )
        .unwrap();
        rows(&out)
            .iter()
            .filter_map(|r| Some((s(r, "path")?.to_string(), s(r, "reason")?.to_string())))
            .collect()
    };
    let pairs = |xs: &[(&str, &str)]| -> Vec<(String, String)> {
        xs.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    };
    assert_eq!(ask("u::f", "body"), pairs(&[("u::f", "changed")]));
    assert_eq!(
        ask("u::f", "iface"),
        pairs(&[("u::f", "changed"), ("u::g", "calls")])
    );
    // `h` names `q`, whose body calls `p`, but reveals neither: unaffected.
    assert_eq!(
        ask("u::p", "body"),
        pairs(&[("u::f", "uses"), ("u::g", "uses")])
    );
    assert_eq!(
        ask("u::p", "iface"),
        pairs(&[
            ("u::f", "uses"),
            ("u::g", "uses"),
            ("u::p", "changed"),
            ("u::q", "names")
        ])
    );
    let usage = bmb::map_request(
        &files(text),
        &[
            "query".into(),
            "impact".into(),
            "u::f".into(),
            "both".into(),
        ],
    );
    assert!(usage.is_err());
}
