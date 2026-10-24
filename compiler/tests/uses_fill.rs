//! `#[uses]` auto-fill (backlog 2D-05): a failing definition gets the
//! smallest `#[uses(..)]` list, built from its entries and the hidden
//! predicates its diagnostics name, that makes it prove; nothing else changes.

use std::process::Command;

use bmb::uses_fill::{UsesStatus, fill_uses};

const SRC: &str = r#"#[summary("uses fill")]
mod m {
    spec fn is_succ(x: i64, r: i64) -> bool { r == x + 1 }

    spec fn small(x: i64) -> bool { x < 100 && x > 0 }

    spec fn tiny(x: i64) -> bool { small(x) }

    spec fn pos(x: i64) -> bool { x > 0 }

    // Needs `small` (overflow) besides the `is_succ` it already names.
    #[effects(pure)]
    #[uses(is_succ)]
    fn inc(x: i64) -> (r: i64)
        requires small(x),
        ensures is_succ(x, r),
    {
        x + 1
    }

    // No attribute yet; `pos` is not needed.
    #[effects(pure)]
    fn bump(x: i64) -> i64
        requires small(x), pos(x),
    {
        x + 1
    }

    // `pos` is superfluous and the body is wrong: no list can prove it.
    #[effects(pure)]
    #[uses(pos)]
    fn wrong(x: i64) -> (r: i64)
        requires small(x),
        ensures is_succ(x, r),
    {
        x
    }

    // `small` appears only in the body of `tiny`.
    #[effects(pure)]
    fn deep(x: i64) -> i64
        requires tiny(x),
    {
        x + 1
    }

    // `pos` is superfluous here; `small` is what the proof needs.
    #[effects(pure)]
    #[uses(pos)]
    fn swap_in(x: i64) -> i64
        requires small(x),
    {
        x + 1
    }

    #[effects(pure)]
    fn fine(x: i64) -> i64 { x }
}
"#;

fn one(text: &str) -> Vec<(String, String)> {
    vec![("u.bmb".to_string(), text.to_string())]
}

fn row<'a>(out: &'a bmb::uses_fill::UsesOutput, def: &str) -> &'a bmb::uses_fill::UsesRow {
    out.rows.iter().find(|r| r.def == def).unwrap()
}

fn verified(files: &[(String, String)], def: &str) -> bool {
    bmb::verify_sources(files)
        .output
        .contains(&format!("{{\"def\":\"{def}\",\"status\":\"verified\""))
}

#[test]
fn missing_predicates_are_added_and_prove() {
    let input = one(SRC);
    assert!(!verified(&input, "m::inc"));
    assert!(!verified(&input, "m::bump"));
    let out = fill_uses(&input).unwrap();

    let inc = row(&out, "m::inc");
    assert_eq!(inc.status, UsesStatus::Proved);
    assert_eq!(inc.uses, ["is_succ", "small"]);
    assert!(
        out.files[0]
            .1
            .contains("    #[uses(is_succ, small)]\n    fn inc(")
    );

    // Inserted where none was, on its own line before `fn`.
    let bump = row(&out, "m::bump");
    assert_eq!(bump.status, UsesStatus::Proved);
    assert_eq!(bump.uses, ["small"]);
    assert!(
        out.files[0]
            .1
            .contains("    #[effects(pure)]\n    #[uses(small)]\n    fn bump(")
    );

    // A superfluous entry is dropped while the needed one is added.
    let swap = row(&out, "m::swap_in");
    assert_eq!(swap.status, UsesStatus::Proved);
    assert_eq!(swap.uses, ["small"]);

    assert_eq!(row(&out, "m::fine").status, UsesStatus::Unchanged);
    for def in ["m::inc", "m::bump", "m::swap_in", "m::deep", "m::fine"] {
        assert!(verified(&out.files, def), "{def}\n{}", out.files[0].1);
    }
}

#[test]
fn closure_reaches_predicates_named_only_in_revealed_bodies() {
    // The first diagnostics name only `tiny`; `small` shows up once `tiny`'s
    // body is revealed, and both are needed.
    let first = bmb::verify_sources(&one(SRC)).output;
    let deep_hint = first
        .lines()
        .find(|l| l.contains("in `m::deep`"))
        .unwrap()
        .to_string();
    assert!(deep_hint.contains("#[uses(tiny)]"), "{deep_hint}");
    let out = fill_uses(&one(SRC)).unwrap();
    let deep = row(&out, "m::deep");
    assert_eq!(deep.status, UsesStatus::Proved);
    assert_eq!(deep.uses, ["small", "tiny"]);
    // reveal {tiny}, reveal {small, tiny}, then try dropping each.
    assert_eq!(deep.verifications, 4);
}

#[test]
fn no_false_success_when_another_failure_remains() {
    let out = fill_uses(&one(SRC)).unwrap();
    let wrong = row(&out, "m::wrong");
    assert_eq!(wrong.status, UsesStatus::NotFound);
    assert_eq!(wrong.uses, ["pos"]);
    assert!(wrong.edit.is_none());
    // Untouched in the output text, and still failing.
    assert!(out.files[0].1.contains("    #[uses(pos)]\n    fn wrong("));
    assert!(!verified(&out.files, "m::wrong"));
    assert!(!out.ok());
}

#[test]
fn edits_are_replacements_that_reproduce_the_output() {
    let input = one(SRC);
    let out = fill_uses(&input).unwrap();
    let text = &input[0].1;
    let offset = |line: u32, col: u32| {
        let start: usize = text
            .split_inclusive('\n')
            .take(line as usize - 1)
            .map(str::len)
            .sum();
        start
            + text[start..]
                .chars()
                .take(col as usize - 1)
                .map(char::len_utf8)
                .sum::<usize>()
    };
    let mut edits: Vec<(usize, usize, String)> = out
        .rows
        .iter()
        .filter_map(|r| r.edit.as_ref())
        .map(|e| {
            (
                offset(e.line, e.col),
                offset(e.end_line, e.end_col),
                e.replacement.clone(),
            )
        })
        .collect();
    edits.sort();
    let mut fixed = text.clone();
    for (lo, hi, r) in edits.into_iter().rev() {
        fixed.replace_range(lo..hi, &r);
    }
    assert_eq!(fixed, out.files[0].1);
    let report = out.report(&input);
    assert!(report.contains("\"fix\":{\"message\":"), "{report}");
    assert!(report.contains("\"replacement\":\"#[uses(is_succ, small)]\""));
}

#[test]
fn deterministic_and_idempotent() {
    let input = one(SRC);
    let a = fill_uses(&input).unwrap();
    let b = fill_uses(&input).unwrap();
    assert_eq!(a.report(&input), b.report(&input));
    assert_eq!(a.files, b.files);
    // Verifications: the first, four lockstep rounds (`m::deep` takes the
    // most), and the final confirmation of all found lists together.
    assert!(
        a.report(&input).ends_with(
            "{\"summary\":true,\"defs\":6,\"proved\":4,\"unchanged\":1,\"not_found\":1,\"verifications\":6,\"changed\":[\"u.bmb\"]}\n"
        ),
        "{}",
        a.report(&input)
    );

    // On its own output nothing changes.
    let again = fill_uses(&a.files).unwrap();
    assert_eq!(again.files, a.files);
    assert!(
        again
            .rows
            .iter()
            .all(|r| r.edit.is_none() && r.status != UsesStatus::Proved)
    );
    assert!(again.report(&a.files).ends_with(
        "{\"summary\":true,\"defs\":6,\"proved\":0,\"unchanged\":5,\"not_found\":1,\"verifications\":2,\"changed\":[]}\n"
    ));

    // A program that proves is left alone.
    let proving = "#[summary(\"p\")]\nmod p {\n    spec fn small(x: i64) -> bool { x < 100 && x > 0 }\n\n    #[effects(pure)]\n    #[uses(small)]\n    fn inc(x: i64) -> i64\n        requires small(x),\n    {\n        x + 1\n    }\n}\n";
    let out = fill_uses(&one(proving)).unwrap();
    assert!(out.ok());
    assert_eq!(out.files, one(proving));
    assert_eq!(out.verifications, 1);
    assert_eq!(row(&out, "p::inc").uses, ["small"]);
}

#[test]
fn qualified_predicates_from_other_units() {
    let text = "#[summary(\"a\")]\nmod a {\n    spec fn small(x: i64) -> bool { x < 100 && x > 0 }\n}\n\n#[summary(\"b\")]\nmod b {\n    #[effects(pure)] fn inc(x: i64) -> i64\n        requires a::small(x),\n    {\n        x + 1\n    }\n}\n";
    let out = fill_uses(&one(text)).unwrap();
    let inc = row(&out, "b::inc");
    assert_eq!(inc.status, UsesStatus::Proved);
    assert_eq!(inc.uses, ["a::small"]);
    // `fn` shares its line with another attribute: inserted inline.
    assert!(
        out.files[0]
            .1
            .contains("#[effects(pure)] #[uses(a::small)] fn inc(")
    );
    assert!(bmb::verify_sources(&out.files).ok);
}

#[test]
fn program_that_does_not_check_is_an_error() {
    let err = fill_uses(&one("mod m {}\n")).err().unwrap();
    assert!(err.contains("E0704"), "{err}");
}

#[test]
fn cli_reports_and_writes_only_with_write() {
    let dir = std::env::temp_dir().join(format!("bmb-uses-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("u.bmb");
    std::fs::write(&path, SRC).unwrap();
    let bmb = env!("CARGO_BIN_EXE_bmb");

    let report = Command::new(bmb).arg("uses").arg(&path).output().unwrap();
    // `m::wrong` cannot be proved.
    assert_eq!(report.status.code(), Some(1));
    let stdout = String::from_utf8(report.stdout).unwrap();
    assert!(stdout.lines().any(|l| l.starts_with(
        "{\"def\":\"m::inc\",\"uses\":[\"is_succ\",\"small\"],\"status\":\"proved\""
    )));
    assert!(stdout.lines().last().unwrap().contains("\"proved\":4"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), SRC);

    let written = Command::new(bmb)
        .args(["uses", "--write"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(written.status.code(), Some(1));
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(after, fill_uses(&one(SRC)).unwrap().files[0].1);

    let again = Command::new(bmb)
        .args(["uses", "--write"])
        .arg(&path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(again.stdout).unwrap();
    assert!(stdout.lines().last().unwrap().contains("\"changed\":[]"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
    std::fs::remove_dir_all(&dir).unwrap();
}
