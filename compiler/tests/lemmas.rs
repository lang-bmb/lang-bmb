//! The division and multiplication facts the verifier assumes are theorems of its own SMT
//! definitions (ADR-0017 §3). If this test fails, the gate is unsound.

use std::io::Write;
use std::process::{Command, Stdio};

use bmb::verify::{
    MUL_BOUND_LEMMA, MUL_LEMMAS, PRELUDE, SIGNED_DIV_LEMMAS, UNSIGNED_DIV_LEMMAS, instantiate,
    instantiate_mul,
};

fn prove_all(header: &str, lemmas: &[&str], q: &str, r: &str) {
    let goals: Vec<String> = lemmas
        .iter()
        .map(|l| instantiate(l, "a", "b", q, r))
        .collect();
    prove(header, &goals);
}

fn prove(header: &str, goals: &[String]) {
    for goal in goals {
        let script = format!(
            "{PRELUDE}\n(declare-const a Int)\n(declare-const b Int)\n(declare-const x Int)\n(declare-const y Int)\n{header}\n(assert (not {goal}))\n(check-sat)\n"
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
        assert_eq!(answer.trim(), "unsat", "lemma not proved: {goal}");
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

#[test]
fn multiplication_lemmas_hold() {
    let mut goals: Vec<String> = MUL_LEMMAS
        .iter()
        .map(|l| instantiate_mul(l, "a", "b", "(* a b)", "", ""))
        .collect();
    goals.push(instantiate_mul(
        MUL_BOUND_LEMMA,
        "a",
        "b",
        "(* a b)",
        "x",
        "y",
    ));
    prove("", &goals);
}
