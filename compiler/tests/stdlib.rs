//! The standard library as the tools see it (ADR-0092 §1, backlog 4B-02):
//! added to a program that names it and only then, proved and run with it,
//! shown apart in the map, and never written by a tool. Each unit is proved
//! on its own by `tests/verify.rs` `libraries_are_proved`.

mod common;

use std::fs;
use std::process::Command;

use common::cli;

fn files(text: &str) -> Vec<(String, String)> {
    vec![("app.bmb".to_string(), text.to_string())]
}

const USES: &str = r#"#[summary("std 사용")]
mod app {
    #[effects(pure)]
    fn begins(s: &str) -> bool {
        std::str::starts_with(s, "ab")
    }
}
"#;

const PLAIN: &str = r#"#[summary("std 없음")]
mod app {
    #[effects(pure)]
    fn one() -> i64 {
        1
    }
}
"#;

/// A program that names `std::str` gets it, proved with the program; one
/// that does not, does not.
#[test]
fn the_library_is_added_where_it_is_named() {
    let (ok, out) = cli::run(&["verify"], &files(USES));
    assert!(ok, "{out}");
    assert!(
        out.contains("\"def\":\"std::str::starts_with\",\"status\":\"verified\""),
        "{out}"
    );
    let (ok, out) = cli::run(&["verify"], &files(PLAIN));
    assert!(ok, "{out}");
    assert!(!out.contains("std::"), "{out}");
}

/// The normal form and `fix --write` hold the program's units only.
#[test]
fn fmt_and_fix_leave_the_library_out() {
    let (ok, out) = cli::run(&["fmt"], &files(USES));
    assert!(ok, "{out}");
    assert!(
        !out.contains("mod std") && !out.contains("mod str"),
        "{out}"
    );
    let dir = cli::fresh_dir();
    let app = dir.join("app.bmb");
    // A fixable habit in the program: `x.len() as u64`-style casts are
    // E0205 with a replacement; the library has none to apply.
    fs::write(
        &app,
        USES.replace(
            "std::str::starts_with(s, \"ab\")",
            "std::str::starts_with(s, \"ab\") && s.len() as u64 > 0",
        ),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["fix", "--write"])
        .arg(&app)
        .output()
        .unwrap();
    let text = fs::read_to_string(&app).unwrap();
    assert!(text.contains("u64::try_from(s.len())"), "{text}\n{out:?}");
    let written: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(written, vec!["app.bmb".to_string()]);
    assert!(!dir.join("std").exists());
    let _ = fs::remove_dir_all(&dir);
}

/// The map shows the library's units and definitions apart.
#[test]
fn the_map_marks_the_library() {
    let (ok, out) = cli::run(&["map", "L0"], &files(USES));
    assert!(ok, "{out}");
    let lib: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("\"library\":\"std\""))
        .collect();
    assert!(
        lib.iter()
            .any(|l| l.contains("\"path\":\"std::str::starts_with\"")),
        "{out}"
    );
    assert!(
        out.lines()
            .filter(|l| l.contains("\"path\":\"app"))
            .all(|l| !l.contains("\"library\"")),
        "{out}"
    );
}

/// A transaction writes the program's units, reports its own changes, and
/// does not change the library.
#[test]
fn transactions_do_not_touch_the_library() {
    let root = cli::fresh_dir();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/app.bmb"), PLAIN).unwrap();
    let apply = |ops: &str| {
        let f = root.join("ops.json");
        fs::write(&f, ops).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("apply")
            .arg(&root)
            .arg(&f)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    let out = apply(concat!(
        r#"[{"op":"put","unit":"app","code":""#,
        "#[effects(pure)] fn begins(s: &str) -> bool { std::str::starts_with(s, \\\"ab\\\") }",
        r#""}]"#
    ));
    assert!(out.contains("\"committed\":true"), "{out}");
    assert!(out.contains("\"changed\":[\"app::begins\"]"), "{out}");
    let names: Vec<_> = fs::read_dir(root.join("src"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(names, vec!["app.bmb".to_string()]);
    for op in [
        r#"[{"op":"remove","def":"std::str::starts_with"}]"#,
        r#"[{"op":"rename","def":"std::str::starts_with","to":"begins"}]"#,
        concat!(
            r#"[{"op":"put","unit":"std::str","code":""#,
            "#[effects(pure)] fn f() -> i64 { 0 }",
            r#""}]"#
        ),
        r#"[{"op":"unit","path":"std","summary":"x"}]"#,
    ] {
        let out = apply(op);
        assert!(out.contains("\"committed\":false"), "{op}: {out}");
        assert!(out.contains("standard library"), "{op}: {out}");
    }
    let _ = fs::remove_dir_all(&root);
}

/// What `tests/data/std_str_diff.bmb` prints for one line `s\tp`, by Rust's
/// own `str` methods (the positions BMB returns where Rust returns slices).
fn rust_str(s: &str, p: &str) -> String {
    let opt = |r: Option<usize>| r.map_or("-".to_string(), |i| i.to_string());
    let mut out = vec![
        u8::from(s.starts_with(p)).to_string(),
        u8::from(s.ends_with(p)).to_string(),
        opt(s.find(p)),
        opt(s.strip_prefix(p).map(|_| p.len())),
        opt(s.strip_suffix(p).map(|r| r.len())),
        (s.len() - s.trim_ascii_start().len()).to_string(),
        s.trim_ascii_end().len().to_string(),
    ];
    for i in 0..=s.len() {
        match s.get(i..).and_then(|t| t.chars().next()) {
            Some(c) => {
                out.push((c as u32).to_string());
                out.push((i + c.len_utf8()).to_string());
            }
            None => out.push("-".to_string()),
        }
    }
    out.join("\n") + "\n"
}

/// `std::str` against Rust's `str` on random text (ADR-0092: each unit has
/// a Rust differential test and a fuzzer target): ASCII letters and
/// whitespace and characters of two, three and four bytes; patterns cut
/// from the text (whole characters) or made the same way. The BMB program
/// is fully proved (its slices at the tab and before the line feed too)
/// and built once; `BMB_SOUND_ITERS` sets the lines (default 3,000).
#[test]
fn std_str_agrees_with_rust_str() {
    let n: usize = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(3000, |k: usize| k * 750);
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/std_str_diff.bmb");
    let dir = cli::fresh_dir();
    let exe = dir.join("diff");
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("build")
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    const PARTS: &[&str] = &[
        "a", "b", "ab", " ", "\r", "\u{c}", "\u{b}", "é", "€", "😀", "ß", "x", "\u{7f}",
    ];
    let mut rng = common::scalar::Rng(0x9E37_79B9_7F4A_7C15);
    let text = |rng: &mut common::scalar::Rng, len: usize| -> String {
        (0..len).map(|_| PARTS[rng.below(PARTS.len())]).collect()
    };
    let mut input = String::new();
    let mut expected = String::new();
    for _ in 0..n {
        let len = rng.below(9);
        let s = text(&mut rng, len);
        let p = if rng.chance(60) && !s.is_empty() {
            // A run of whole characters of `s`.
            let cuts: Vec<usize> = (0..=s.len()).filter(|&i| s.is_char_boundary(i)).collect();
            let a = cuts[rng.below(cuts.len())];
            let b = cuts[rng.below(cuts.len())];
            s[a.min(b)..a.max(b)].to_string()
        } else {
            let len = rng.below(3);
            text(&mut rng, len)
        };
        input.push_str(&format!("{s}\t{p}\n"));
        expected.push_str(&rust_str(&s, &p));
    }
    let mut child = Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write as _;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    let run = child.wait_with_output().unwrap();
    assert!(run.status.success());
    let got = String::from_utf8(run.stdout).unwrap();
    let _ = fs::remove_dir_all(&dir);
    if got != expected {
        let (g, e): (Vec<&str>, Vec<&str>) = (got.lines().collect(), expected.lines().collect());
        let k = g
            .iter()
            .zip(&e)
            .position(|(a, b)| a != b)
            .unwrap_or(g.len().min(e.len()));
        panic!(
            "std::str differs from Rust's str at output line {k}: {:?} vs {:?}",
            g.get(k),
            e.get(k)
        );
    }
}

/// The hint of an unknown text method lists every function of `std::str`.
#[test]
fn the_text_method_hint_names_every_std_str_function() {
    let lib = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../lib/std/str.bmb"),
    )
    .unwrap();
    let fns: Vec<&str> = lib
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("fn "))
        .map(|l| &l[..l.find('(').unwrap()])
        .filter(|f| *f != "ascii_space")
        .collect();
    assert!(fns.len() >= 8, "{fns:?}");
    let prog = "#[summary(\"x\")]\nmod t {\n    #[effects(pure)]\n    fn f(s: &str) -> usize {\n        s.trim().len()\n    }\n}\n";
    let (_, out) = cli::run(&["check"], &files(prog));
    for f in fns {
        assert!(out.contains(&format!("`{f}`")), "`{f}` missing: {out}");
    }
}

/// What `tests/data/std_num_diff.bmb` prints for line `t`, by Rust's
/// `parse`, `from_str_radix` and `Display`/`{:x}`.
fn rust_num(t: &str) -> String {
    let u = t.parse::<u64>().map_or("-".to_string(), |v| v.to_string());
    let i = t.parse::<i64>().map_or("-".to_string(), |v| v.to_string());
    let h = u64::from_str_radix(t, 16).map_or("-".to_string(), |v| format!("{v:x}"));
    format!("{u} {i} {h}\n")
}

/// `std::num` against Rust (ADR-0092: Rust differential test and fuzzer
/// target): decimal and hexadecimal texts near the limits of `u64` and
/// `i64`, signs, leading zeros, empty and stray bytes; each value read is
/// written back, so the round trip is in the comparison too.
/// `BMB_SOUND_ITERS` sets the lines (default 3,000).
#[test]
fn std_num_agrees_with_rust() {
    let n: usize = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(3000, |k: usize| k * 750);
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/std_num_diff.bmb");
    let dir = cli::fresh_dir();
    let exe = dir.join("diff");
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("build")
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let mut rng = common::scalar::Rng(0xD1B5_4A32_D192_ED03);
    let edges: Vec<String> = [
        "0",
        "1",
        "9",
        "10",
        "+0",
        "-0",
        "+",
        "-",
        "",
        "00",
        "007",
        "18446744073709551615",
        "18446744073709551616",
        "99999999999999999999",
        "9223372036854775807",
        "9223372036854775808",
        "-9223372036854775808",
        "-9223372036854775809",
        "ffffffffffffffff",
        "10000000000000000",
        "FfFf",
        "0x1f",
        " 1",
        "1 ",
        "+-1",
        "--1",
        "1_000",
        "１",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let mut input = String::new();
    let mut expected = String::new();
    for k in 0..n {
        let t = if k < edges.len() {
            edges[k].clone()
        } else {
            let value = match rng.below(4) {
                0 => rng.next(),
                1 => rng.next() >> rng.below(64),
                2 => u64::MAX - rng.below(1000) as u64,
                _ => rng.below(100) as u64,
            };
            let body = if rng.chance(30) {
                format!("{value:x}")
            } else {
                value.to_string()
            };
            let sign = ["", "", "", "+", "-"][rng.below(5)];
            let zeros = "0".repeat(if rng.chance(10) { rng.below(3) } else { 0 });
            let mut t = format!("{sign}{zeros}{body}");
            if rng.chance(10) {
                let at = rng.below(t.len() + 1);
                t.insert(at, ['x', ' ', 'g', 'G', '9', '+'][rng.below(6)]);
            }
            t
        };
        input.push_str(&t);
        input.push('\n');
        expected.push_str(&rust_num(&t));
    }
    let mut child = Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write as _;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    let run = child.wait_with_output().unwrap();
    assert!(run.status.success());
    let got = String::from_utf8(run.stdout).unwrap();
    let _ = fs::remove_dir_all(&dir);
    let lines: Vec<&str> = input.lines().collect();
    for (k, (g, e)) in got.lines().zip(expected.lines()).enumerate() {
        assert_eq!(g, e, "std::num differs from Rust on {:?}", lines[k]);
    }
    assert_eq!(got.lines().count(), expected.lines().count());
}

/// What `tests/data/std_slice_diff.bmb` prints for keys `v`: Rust's stable
/// `sort_by_key` as positions, then `binary_search` of 0..=10 in the
/// sorted keys (`y` for any match, as Rust may return any; the insertion
/// point otherwise, which is unique).
fn rust_slice(v: &[u64]) -> String {
    let mut r: Vec<usize> = (0..v.len()).collect();
    r.sort_by_key(|&i| v[i]);
    let mut out: String = r.iter().map(|i| format!("{i} ")).collect();
    out.push_str("|\n");
    let sorted: Vec<u64> = r.iter().map(|&i| v[i]).collect();
    for x in 0..=10u64 {
        match sorted.binary_search(&x) {
            Ok(_) => out.push_str("y\n"),
            Err(i) => out.push_str(&format!("{i}\n")),
        }
    }
    out
}

/// `std::slice` against Rust: random keys with many repeats (so stability
/// shows), sorted by position, and searched. `BMB_SOUND_ITERS` sets the
/// lines (default 1,500).
#[test]
fn std_slice_agrees_with_rust() {
    let n: usize = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(1500, |k: usize| k * 375);
    let src =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/std_slice_diff.bmb");
    let dir = cli::fresh_dir();
    let exe = dir.join("diff");
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("build")
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let mut rng = common::scalar::Rng(0x2545_F491_4F6C_DD1D);
    let mut input = String::new();
    let mut expected = String::new();
    for _ in 0..n {
        let len = rng.below(12);
        let range = [3, 6, 11, 1000][rng.below(4)];
        let v: Vec<u64> = (0..len).map(|_| rng.below(range) as u64).collect();
        let text: Vec<String> = v.iter().map(|k| k.to_string()).collect();
        input.push_str(&text.join(" "));
        input.push('\n');
        expected.push_str(&rust_slice(&v));
    }
    let mut child = Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write as _;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    let run = child.wait_with_output().unwrap();
    assert!(run.status.success());
    let got = String::from_utf8(run.stdout).unwrap();
    let _ = fs::remove_dir_all(&dir);
    if got != expected {
        let (g, e): (Vec<&str>, Vec<&str>) = (got.lines().collect(), expected.lines().collect());
        let k = g
            .iter()
            .zip(&e)
            .position(|(a, b)| a != b)
            .unwrap_or(g.len().min(e.len()));
        panic!(
            "std::slice differs from Rust at output line {k}: {:?} vs {:?}",
            g.get(k),
            e.get(k)
        );
    }
}

/// `std::collections` against Rust's `BTreeMap` and `BTreeSet`: random
/// inserts (new keys and replaced values), removes and lookups over a small
/// key range, so keys repeat, then the values and the keys in order. The BMB
/// program (`tests/data/std_collections_diff.bmb`) is fully proved.
/// `BMB_SOUND_ITERS` sets the operations (default 2,000).
#[test]
fn std_collections_agree_with_rust() {
    let keys: Vec<u64> = (0..40).collect();
    collections_diff("std_collections_diff.bmb", &keys, 0x5DEE_CE66_D1B5_4A32);
}

/// The same with text keys (`BTreeMap<String, u64>`, backlog 4B-04): the
/// keys are prefixes of each other, differ in case, hold characters of two
/// to four bytes and include the empty text, so the order is Rust's
/// byte-lexicographic one (`"B" < "a" < "ab" < "é"`), not a numeric or
/// character-class one (`tests/data/std_text_keys_diff.bmb`).
#[test]
fn std_collections_text_keys_agree_with_rust() {
    let keys: Vec<String> = [
        "", "a", "aa", "ab", "aB", "b", "B", "ba", "é", "éa", "e", "€", "😀", "a😀", "z", "Z",
        "10", "9", "a\u{7f}", "ü",
    ]
    .iter()
    .map(|k| k.to_string())
    .collect();
    collections_diff("std_text_keys_diff.bmb", &keys, 0x1B5_4A32_5DEE_CE66);
}

/// Builds `tests/data/{file}`, feeds it random operations on `keys` and
/// compares its output with Rust's `BTreeMap` and `BTreeSet` doing the same.
fn collections_diff<K: Ord + Clone + std::fmt::Display>(file: &str, keys: &[K], seed: u64) {
    use std::collections::{BTreeMap, BTreeSet};
    let n: usize = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(2000, |k: usize| k * 500);
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(file);
    let dir = cli::fresh_dir();
    let exe = dir.join("diff");
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("build")
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let mut rng = common::scalar::Rng(seed);
    let mut m: BTreeMap<K, u64> = BTreeMap::new();
    let mut s: BTreeSet<K> = BTreeSet::new();
    let mut input = String::new();
    let mut expected = String::new();
    let opt = |r: Option<u64>| r.map_or("-\n".to_string(), |v| format!("{v}\n"));
    let flag = |b: bool| if b { "1\n" } else { "0\n" };
    for _ in 0..n {
        let k = keys[rng.below(keys.len())].clone();
        match rng.below(10) {
            0..=4 => {
                let v = rng.next() % 1000;
                input.push_str(&format!("i {k} {v}\n"));
                expected.push_str(&opt(m.insert(k.clone(), v)));
                expected.push_str(flag(s.insert(k)));
            }
            5..=7 => {
                input.push_str(&format!("r {k}\n"));
                expected.push_str(&opt(m.remove(&k)));
                expected.push_str(flag(s.remove(&k)));
            }
            _ => {
                input.push_str(&format!("g {k}\n"));
                expected.push_str(&opt(m.get(&k).copied()));
                expected.push_str(flag(s.contains(&k)));
            }
        }
    }
    for v in m.values() {
        expected.push_str(&format!("{v}\n"));
    }
    for k in m.keys() {
        expected.push_str(&format!("{k}\n"));
    }
    for k in &s {
        expected.push_str(&format!("{k}\n"));
    }
    let mut child = Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write as _;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    let run = child.wait_with_output().unwrap();
    assert!(run.status.success());
    let got = String::from_utf8(run.stdout).unwrap();
    let _ = fs::remove_dir_all(&dir);
    if got != expected {
        let (g, e): (Vec<&str>, Vec<&str>) = (got.lines().collect(), expected.lines().collect());
        let k = g
            .iter()
            .zip(&e)
            .position(|(a, b)| a != b)
            .unwrap_or(g.len().min(e.len()));
        panic!(
            "{file} differs from Rust at output line {k}: {:?} vs {:?}",
            g.get(k),
            e.get(k)
        );
    }
}
