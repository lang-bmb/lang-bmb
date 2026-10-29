//! Contract weakening between two versions (backlog 2D-19, `bmb weaken`),
//! through the command line: the corpus `tests/weaken/<case>/{old,new}.bmb`
//! with its expected output (`out.jsonl`, `BMB_BLESS=1` rewrites it), the
//! verdict of every definition, determinism, and exit codes.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/weaken")
}

/// `bmb weaken OLD NEW` run in the corpus directory: exit code and output.
fn weaken(old: &str, new: &str) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(corpus())
        .args(["weaken", old, new])
        .output()
        .unwrap();
    (out.status.code(), String::from_utf8(out.stdout).unwrap())
}

fn case(c: &str) -> (Option<i32>, String) {
    weaken(&format!("{c}/old.bmb"), &format!("{c}/new.bmb"))
}

/// The string value of `key` in one JSON line, `None` for `null`.
fn field(line: &str, key: &str) -> Option<String> {
    let at = line.find(&format!("\"{key}\":"))? + key.len() + 3;
    let v = line[at..].strip_prefix('"')?;
    Some(v[..v.find('"')?].to_string())
}

/// The rows by definition (the new path, or the old one when removed).
fn rows(out: &str) -> BTreeMap<String, String> {
    out.lines()
        .filter(|l| l.contains("\"verdict\":"))
        .map(|l| {
            let d = field(l, "def").or_else(|| field(l, "old")).unwrap();
            (d, l.to_string())
        })
        .collect()
}

#[test]
fn the_corpus_gives_its_expected_output() {
    let bless = std::env::var("BMB_BLESS").is_ok();
    let mut cases: Vec<PathBuf> = fs::read_dir(corpus())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    cases.sort();
    assert!(cases.len() >= 8, "{cases:?}");
    for dir in cases {
        let c = dir.file_name().unwrap().to_str().unwrap().to_string();
        let (_, got) = case(&c);
        let want = dir.join("out.jsonl");
        if bless {
            fs::write(&want, &got).unwrap();
            continue;
        }
        assert_eq!(
            fs::read_to_string(&want).unwrap_or_default(),
            got,
            "{c}: output differs (BMB_BLESS=1 rewrites it)"
        );
    }
}

/// (definition, verdict, requires, ensures, effects); `-` is `null`.
type Want = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

#[test]
fn every_definition_gets_its_verdict() {
    let cases: &[(&str, &[Want])] = &[
        (
            "requires",
            &[
                ("m::inc", "weakened", "stronger", "same", "same"),
                ("m::inc_wide", "refined", "weaker", "same", "same"),
            ],
        ),
        (
            "ensures",
            &[
                ("m::inc", "weakened", "same", "weaker", "same"),
                ("m::up", "refined", "same", "stronger", "same"),
                ("m::parity", "weakened", "same", "incomparable", "same"),
            ],
        ),
        (
            "same",
            &[
                ("m::inc", "same", "same", "same", "same"),
                ("m::inc_body", "same", "same", "same", "same"),
                ("m::inc_order", "same", "same", "same", "same"),
                ("m::bump", "same", "same", "same", "same"),
                ("m::twice", "same", "same", "same", "same"),
            ],
        ),
        (
            "opaque",
            &[("m::inc", "weakened", "incomparable", "same", "same")],
        ),
        (
            "rename",
            &[
                ("m::step", "weakened", "same", "weaker", "same"),
                ("n::up", "refined", "weaker", "same", "same"),
                ("m::inc3", "same", "same", "same", "same"),
                ("m::dec", "weakened", "stronger", "same", "same"),
                ("m::fresh", "added", "-", "-", "-"),
                ("m::gone", "removed", "-", "-", "-"),
            ],
        ),
        (
            "spec",
            &[
                ("m::inc", "refined", "weaker", "same", "same"),
                ("m::clamp", "weakened", "same", "weaker", "same"),
                ("m::keep", "weakened", "weaker", "weaker", "same"),
                ("m::clamp2", "weakened", "same", "weaker", "same"),
                ("m::inc_tiny", "same", "same", "same", "same"),
            ],
        ),
        (
            "shapes",
            &[
                ("m::noisy", "weakened", "same", "same", "weaker"),
                ("m::quiet", "refined", "same", "same", "stronger"),
                ("m::widen", "signature", "-", "-", "-"),
                ("m::zero_first", "weakened", "same", "weaker", "same"),
                ("m::area", "weakened", "stronger", "same", "same"),
                ("m::keep_pair", "refined", "weaker", "same", "same"),
                ("m::below_more", "weakened", "stronger", "same", "same"),
                ("m::pick", "same", "same", "same", "same"),
                ("m::Pair::make", "weakened", "stronger", "same", "same"),
            ],
        ),
        (
            "undecided",
            &[("m::total", "undecided", "same", "undecided", "same")],
        ),
    ];
    for (c, want) in cases {
        let (_, out) = case(c);
        let got = rows(&out);
        assert_eq!(got.len(), want.len(), "{c}: {out}");
        for (def, v, req, ens, eff) in *want {
            let row = &got[*def];
            let f = |k: &str| field(row, k).unwrap_or("-".into());
            assert_eq!(
                (f("verdict"), f("requires"), f("ensures"), f("effects")),
                (
                    v.to_string(),
                    req.to_string(),
                    ens.to_string(),
                    eff.to_string()
                ),
                "{c} {def}"
            );
        }
    }
}

/// A stronger precondition with every postcondition the same is a weakening
/// (the gate-power mutant `w-requires-direction-dropped` ignores the
/// `requires` direction and calls it the same), with the counterexample
/// and a `W0908` at the new clause.
#[test]
fn a_stronger_precondition_alone_is_a_weakening() {
    let (code, out) = case("requires");
    assert_eq!(code, Some(1), "{out}");
    let inc = &rows(&out)["m::inc"];
    assert_eq!(field(inc, "verdict").as_deref(), Some("weakened"));
    assert_eq!(field(inc, "ensures").as_deref(), Some("same"));
    assert!(
        inc.contains(
            "\"lost\":[{\"clause\":\"new requires pos(x)\",\"counterexample\":\"x = 0\",\"opaque\":[]}]"
        ),
        "{inc}"
    );
    let w: Vec<&str> = out.lines().filter(|l| l.contains("\"W0908\"")).collect();
    assert_eq!(w.len(), 1, "{out}");
    assert!(w[0].contains("\"severity\":\"warning\""), "{}", w[0]);
    assert!(
        w[0].contains("\"file\":\"requires/new.bmb\",\"line\":15"),
        "{}",
        w[0]
    );
}

/// A counterexample is the input (and the result, or the value behind a
/// `&mut` parameter after the call) where the old contract promised what
/// the new one does not; spec fns the query saw only by name are listed.
#[test]
fn counterexamples_name_the_lost_promise() {
    let (_, out) = case("ensures");
    assert!(
        rows(&out)["m::inc"].contains(
            "{\"clause\":\"old ensures is_succ(x, r)\",\"counterexample\":\"x = 0, r = 2\""
        ),
        "{out}"
    );
    let (_, out) = case("shapes");
    assert!(
        rows(&out)["m::zero_first"].contains("a = [0] (len 1), a after = [0, 0] (len 2)"),
        "{out}"
    );
    let (_, out) = case("opaque");
    assert!(
        rows(&out)["m::inc"].contains("\"opaque\":[\"m::nonneg\"]"),
        "{out}"
    );
}

/// The same input gives the same bytes, and a version against itself is the
/// same everywhere without a query.
#[test]
fn deterministic_and_identity_is_free() {
    for c in ["shapes", "spec", "rename"] {
        assert_eq!(case(c), case(c), "{c}");
        let n = format!("{c}/new.bmb");
        let (code, out) = weaken(&n, &n);
        assert_eq!(code, Some(0), "{out}");
        assert!(
            rows(&out)
                .values()
                .all(|r| field(r, "verdict").as_deref() == Some("same")),
            "{out}"
        );
        assert!(out.ends_with("\"queries\":0}\n"), "{out}");
    }
}

/// A JSON string's text (the escapes `json_str` writes).
fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let h: String = it.by_ref().take(4).collect();
                out.push(char::from_u32(u32::from_str_radix(&h, 16).unwrap()).unwrap());
            }
            Some(c) => out.push(c),
            None => {}
        }
    }
    out
}

/// The aligned program holds both versions of real programs (structs with
/// `impl`s, two structs of one `id`, traits, generics, recursive types): a
/// body mutant (`bmb mutate`) changes no contract, so every definition is
/// `same`, also the callers whose `id` changed with it. The sweep that this
/// samples found the two structs of one `id` and the unshared traits.
#[test]
fn body_mutants_of_real_programs_are_the_same() {
    let bmb = env!("CARGO_BIN_EXE_bmb");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = std::env::temp_dir().join(format!("bmb-weaken-mut-{}", std::process::id()));
    let run = |args: &[&str]| {
        let o = Command::new(bmb).args(args).output().unwrap();
        String::from_utf8(o.stdout).unwrap()
    };
    for (file, def) in [
        ("tests/verify/pass/structs.bmb", "geo::twice"),
        ("tests/run/trait_contracts.bmb", "sc::best"),
        ("tests/verify/pass/invariants.bmb", "inv::make"),
        ("tests/verify/pass/invariants.bmb", "inv::two"),
    ] {
        let (a, b) = (dir.join("a"), dir.join("b"));
        let _ = fs::remove_dir_all(&dir);
        let src = root.join(file);
        run(&["import", a.to_str().unwrap(), src.to_str().unwrap()]);
        let out = run(&["mutate", "--def", def, a.to_str().unwrap()]);
        let m = out
            .lines()
            .find(|l| l.contains("\"text\":"))
            .unwrap_or_else(|| panic!("{file}: no mutant of {def}: {out}"));
        let unit = field(m, "unit").unwrap();
        let at = m.find("\"text\":\"").unwrap() + 8;
        let text = unescape(&m[at..m.len() - 2]);
        fs::create_dir_all(b.join("src")).unwrap();
        for e in fs::read_dir(a.join("src")).unwrap() {
            let e = e.unwrap().path();
            fs::copy(&e, b.join("src").join(e.file_name().unwrap())).unwrap();
        }
        fs::write(b.join("src").join(format!("{unit}.bmb")), text).unwrap();
        let (code, out) = weaken(a.to_str().unwrap(), b.to_str().unwrap());
        assert_eq!(code, Some(0), "{file}: {out}");
        let rs = rows(&out);
        assert!(!rs.is_empty(), "{file}: {out}");
        assert!(
            rs.values()
                .all(|r| field(r, "verdict").as_deref() == Some("same")),
            "{file}: {out}"
        );
        assert!(
            !out.ends_with("\"queries\":0}\n"),
            "{file}: nothing compared: {out}"
        );
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn exit_codes_and_workspaces() {
    // Only refinements and rewrites pass; an undecided comparison fails.
    let (code, out) = case("same");
    assert_eq!(code, Some(0), "{out}");
    let (code, out) = case("undecided");
    assert_eq!(code, Some(1), "{out}");
    // Workspace directories.
    let dir = std::env::temp_dir().join(format!("bmb-weaken-{}", std::process::id()));
    for (v, c) in [("a", "old"), ("b", "new")] {
        fs::create_dir_all(dir.join(v).join("src")).unwrap();
        fs::copy(
            corpus().join("spec").join(format!("{c}.bmb")),
            dir.join(v).join("src").join("m.bmb"),
        )
        .unwrap();
    }
    let (code, out) = weaken(
        dir.join("a").to_str().unwrap(),
        dir.join("b").to_str().unwrap(),
    );
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("\"weakened\":3"), "{out}");
    // A version that does not check: its diagnostics.
    fs::write(
        dir.join("bad.bmb"),
        "#[summary(\"x\")]\nmod m {\n    #[effects(pure)]\n    fn f(x: i64) -> i64 {\n        y\n    }\n}\n",
    )
    .unwrap();
    let (code, out) = weaken("spec/old.bmb", dir.join("bad.bmb").to_str().unwrap());
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("\"code\":\"E0301\""), "{out}");
    let _ = fs::remove_dir_all(&dir);
    // Usage.
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["weaken", "x"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
