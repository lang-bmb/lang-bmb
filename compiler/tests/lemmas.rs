//! The division facts the verifier assumes are theorems of its own SMT
//! definitions (ADR-0017 §3). If this test fails, the gate is unsound.

use std::io::Write;
use std::process::{Command, Stdio};

use bmb::verify::{PRELUDE, SIGNED_DIV_LEMMAS, UNSIGNED_DIV_LEMMAS, instantiate};

fn prove_all(header: &str, lemmas: &[&str], q: &str, r: &str) {
    for l in lemmas {
        let goal = instantiate(l, "a", "b", q, r);
        let script = format!(
            "{PRELUDE}\n(declare-const a Int)\n(declare-const b Int)\n{header}\n(assert (not {goal}))\n(check-sat)\n"
        );
        let mut child = Command::new("z3")
            .args(["-in", "-T:120"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("z3 must be installed (scripts/setup.sh)");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let answer = String::from_utf8_lossy(&out.stdout);
        assert_eq!(answer.trim(), "unsat", "lemma not proved: {l}");
    }
}

#[test]
fn signed_division_lemmas_hold() {
    prove_all("", SIGNED_DIV_LEMMAS, "(bmb_tdiv a b)", "(bmb_trem a b)");
}

#[test]
fn unsigned_division_lemmas_hold() {
    prove_all(
        "(assert (>= a 0))",
        UNSIGNED_DIV_LEMMAS,
        "(div a b)",
        "(mod a b)",
    );
}
