//! Escapes (ADR-0029, backlog 2E-02): `assume` and `#[external_body]` with a
//! rationale. A definition is proved without trusted facts first, and only
//! a failure is proved again with them (`assumed`); an `#[external_body]`
//! function has no obligations (`external`). Code generation keeps every
//! check of the definitions that are not `proved` and of those they reach,
//! so a false trusted fact ends in a trap, never in a wrong value.

use std::path::Path;
use std::process::Command;

fn src(text: &str) -> Vec<(String, String)> {
    vec![("e.bmb".to_string(), text.to_string())]
}

fn program() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/run/escapes.bmb"))
        .unwrap()
}

/// `(def, trust)` of every status line; `proved` when the line has none.
fn trust(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .filter(|l| l.starts_with("{\"def\""))
        .map(|l| {
            let def = l.split('"').nth(3).unwrap().to_string();
            let t = l
                .split("\"trust\":\"")
                .nth(1)
                .map_or("proved", |r| r.split('"').next().unwrap());
            let t = if l.contains("\"status\":\"failed\"") {
                "failed"
            } else {
                t
            };
            (def, t.to_string())
        })
        .collect()
}

#[test]
fn trust_levels_follow_what_each_proof_needs() {
    let out = bmb::verify_sources(&src(&program()));
    assert!(out.ok, "{}", out.output);
    let t = trust(&out.output);
    let want = [
        // Needs `outside`'s `ensures` to rule out the overflow of `v + 1`.
        ("e::caller", "assumed"),
        // Does not need `square`'s `ensures`.
        ("e::ignores", "proved"),
        ("e::main", "proved"),
        // Proves without its `assume`.
        ("e::needless", "proved"),
        ("e::outside", "external"),
        // Needs its `assume` to bound `x * x`.
        ("e::square", "assumed"),
    ]
    .map(|(d, t)| (d.to_string(), t.to_string()));
    assert_eq!(t, want);
}

/// ADR-0029 §8 property 2: the `proved` set without any `assume` is the same.
#[test]
fn removing_every_assume_keeps_the_proved_set() {
    let text = program();
    let without: String = text
        .lines()
        .filter(|l| {
            !l.trim_start().starts_with("assume(")
                && !l.trim_start().starts_with("#[rationale(kind = nonlinear")
                && !l
                    .trim_start()
                    .starts_with("#[rationale(kind = solver_limit")
        })
        .map(|l| format!("{l}\n"))
        .collect();
    assert!(!without.contains("assume("));
    let proved = |s: &str| -> Vec<String> {
        trust(&bmb::verify_sources(&src(s)).output)
            .into_iter()
            .filter(|(_, t)| t == "proved")
            .map(|(d, _)| d)
            .collect()
    };
    assert_eq!(proved(&text), proved(&without));
}

/// ADR-0029 §4: the definitions that are not `proved`, and those they
/// reach, keep their checks; the others drop what their proofs show.
#[test]
fn checks_stay_below_every_escape() {
    let out = bmb::emit_llvm_sources(&src(&program()));
    assert!(out.verify.ok, "{}", out.verify.output);
    // Checks below an escape are not missing proofs: the build goes ahead.
    assert_eq!(out.checks, 0);
    let ir = out.ir.unwrap();
    let body = |name: &str| -> String {
        let from = ir
            .lines()
            .position(|l| l.starts_with("define") && l.contains(&format!("@\"e::{name}\"(")))
            .unwrap();
        ir.lines()
            .skip(from)
            .take_while(|l| *l != "}")
            .collect::<Vec<_>>()
            .join("\n")
    };
    for f in ["square", "caller", "outside"] {
        assert!(!body(f).contains("; proved"), "{f}: {}", body(f));
    }
    assert!(body("square").contains("@llvm.trap"), "{}", body("square"));
    assert!(
        body("needless").contains("; proved E0803"),
        "{}",
        body("needless")
    );
}

/// A false `assume` ends in a trap, not in a wrong result (ADR-0029 §4).
#[test]
fn a_false_assumption_traps() {
    let text = program().replace("print_i64(square(3));", "print_i64(square(4000000000));");
    let out = bmb::emit_llvm_sources(&src(&text));
    assert!(out.verify.ok, "{}", out.verify.output);
    let dir = std::env::temp_dir().join(format!("bmb-escapes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("e");
    bmb::toolchain::build_executable(&out.ir.unwrap(), exe.to_str().unwrap()).unwrap();
    let run = Command::new(&exe).output().unwrap();
    assert!(!run.status.success());
    // No wrong value: at most what ran before the trap (stdout is buffered,
    // so a trap may lose it).
    let printed = String::from_utf8_lossy(&run.stdout).to_string();
    assert!(printed.lines().all(|l| l == "8"), "{printed}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// L1 and `status` rows carry the trust keys (ADR-0029 §5, backlog 2E-03).
#[test]
fn maps_show_trust_and_escapes() {
    let f = src(&program());
    let st = bmb::map_request(&f, &["query".into(), "status".into()]).unwrap();
    let row = |d: &str| {
        st.lines()
            .find(|l| l.contains(&format!("\"path\":\"{d}\"")))
            .unwrap()
            .to_string()
    };
    let square = row("e::square");
    assert!(
        square.ends_with(r#""trust":"assumed","escapes":1,"assumed_obligations":2,"checked":true,"rationale":"confirmed","rationale_kinds":["nonlinear"]}"#),
        "{square}"
    );
    assert!(
        row("e::outside")
            .contains(r#""trust":"external","escapes":1,"assumed_obligations":0,"checked":true"#)
    );
    assert!(row("e::needless").contains(r#""trust":"proved","escapes":1"#));
    assert!(
        row("e::ignores")
            .contains(r#""trust":"proved","escapes":0,"assumed_obligations":0,"checked":false"#)
    );
    assert!(
        row("e::caller").contains(r#""test_reasons":["panic","trust"]"#),
        "{}",
        row("e::caller")
    );
    assert!(
        st.lines()
            .last()
            .unwrap()
            .ends_with(r#""proved":6,"assumed":2,"external":1,"escapes":3,"unconfirmed":0}"#),
        "{st}"
    );
}

/// A rationale is bound to the code it speaks for (ADR-0029 §3): editing
/// that code, or a predicate it names, makes it unconfirmed (W1001); its
/// own wording does not, and changes no identity or proof key. `bmb fix`
/// writes a missing hash, `bmb fix --confirm` a stale one.
#[test]
fn rationales_go_unconfirmed_with_their_code() {
    let text = program();
    let lint = |s: &str, code: &str| -> usize {
        bmb::lint::lint_sources(&src(s))
            .unwrap()
            .diagnostics
            .lines()
            .filter(|l| l.contains(&format!("\"code\":\"{code}\"")))
            .count()
    };
    assert_eq!(lint(&text, "W1001"), 0);
    let worded = text.replace("callers pass small values", "callers pass values up to ten");
    assert_eq!(lint(&worded, "W1001"), 0);
    assert_eq!(
        bmb::reads_sources(&src(&text)).unwrap(),
        bmb::reads_sources(&src(&worded)).unwrap(),
        "a rationale's wording is in no key"
    );
    let code = text.replace("        x * x\n", "        x * x + 0\n");
    assert_ne!(code, text);
    assert_eq!(lint(&code, "W1001"), 1);
    let predicate = text.replace(
        "spec fn small(x: i64) -> bool { 0 <= x && x <= 10 }",
        "spec fn small(x: i64) -> bool { 0 <= x && x <= 9 }",
    );
    assert_ne!(predicate, text);
    assert!(lint(&predicate, "W1001") >= 1);
    // `fix` keeps a stale hash, `--confirm` rewrites it.
    let kept = bmb::fix_sources(&src(&code));
    assert_eq!(lint(&kept.files[0].1, "W1001"), 1);
    let confirmed = bmb::fix_sources_with(&src(&code), true);
    assert_eq!(lint(&confirmed.files[0].1, "W1001"), 0);
    // A missing hash is written by `fix`.
    let bare = text.replace(", hash = \"99850c0c37931c6c\"", "");
    assert_eq!(lint(&bare, "W1001"), 1);
    assert_eq!(lint(&bmb::fix_sources(&src(&bare)).files[0].1, "W1001"), 0);
}

/// W1002: an `assume` the proof does not need; its fix removes it.
#[test]
fn needless_assumes_are_reported() {
    let out = bmb::lint::lint_sources(&src(&program())).unwrap();
    let w: Vec<&str> = out
        .diagnostics
        .lines()
        .filter(|l| l.contains("\"W1002\""))
        .collect();
    assert_eq!(w.len(), 1, "{}", out.diagnostics);
    assert!(w[0].contains("e::needless"), "{}", w[0]);
}
