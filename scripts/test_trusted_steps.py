#!/usr/bin/env python3
"""Self-test of scripts/trusted_steps.py: each bound fails when it is
broken, and the port as it is passes."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import trusted_steps as ts

OK = '{"def":"a::f","status":"verified","obligations":1,"failed":0,"rlimit":1,"rlimit_max":1}\n'
SOLVER = """mod solver {
    fn check_sat(s: &str) -> Answer {
        match verdict_word(s) { None => Answer::Sat, Some(a) => a, }
    }

    fn is_unsat(a: &Answer) -> bool {
        match a { Answer::Unsat => true, Answer::Sat => false, }
    }

    fn verdict_word(got: &str) -> Option<Answer> {
        if got == "unsat" { return Some(Answer::Unsat); }
        None
    }
}
"""
PROVE = """mod prove {
    fn decide(v: &mut Verdict) -> () {
        v.proved = true;
    }

    fn decide_sub(v: &mut Verdict) -> () {
        v.proved = true;
    }

    fn fresh() -> Verdict {
        Verdict { proved: false, by: 9 }
    }
}
"""
CODEGEN = """mod codegen {
    fn index(self: &mut Gen, e: &ast::Expr) -> () {
        if self.proved("E0805", e) {
            self.emit("; proved E0805");
        }
    }
}
"""


class Rows(unittest.TestCase):
    def test_verified_rows_pass(self):
        self.assertEqual(ts.bad_rows(OK + OK), [])

    def test_each_key_and_status_breaks_the_bound(self):
        for extra in (',"trust":"assumed"', ',"escapes":1', ',"termination":"unproved"'):
            self.assertEqual(len(ts.bad_rows(OK + OK.replace("}", extra + "}"))), 1, extra)
        self.assertEqual(len(ts.bad_rows(OK.replace("verified", "failed"))), 1)

    def test_no_rows_and_other_lines_fail(self):
        self.assertEqual(ts.bad_rows(""), ["no status rows"])
        self.assertEqual(len(ts.bad_rows(OK + '{"code":"E0802"}\n')), 1)


class Steps(unittest.TestCase):
    def base(self):
        return {"solver.bmb": SOLVER, "prove.bmb": PROVE, "codegen.bmb": CODEGEN}

    def test_the_closed_list_passes(self):
        self.assertEqual(ts.bad_steps(self.base()), [])

    def test_a_second_maker_of_unsat_fails(self):
        s = self.base()
        s["solver.bmb"] = SOLVER.replace("None => Answer::Sat", "None => Answer::Unsat")
        self.assertEqual(len(ts.bad_steps(s)), 1)
        s = self.base()
        s["lint.bmb"] = "mod lint {\n    fn guess() -> solver::Answer {\n        solver::Answer::Unsat\n    }\n}\n"
        self.assertEqual(len(ts.bad_steps(s)), 1)

    def test_a_second_setter_of_proved_fails(self):
        s = self.base()
        s["prove.bmb"] = PROVE.replace("proved: false", "proved: true")
        self.assertEqual(len(ts.bad_steps(s)), 1)

    def test_a_listed_place_that_is_gone_fails(self):
        s = self.base()
        s["prove.bmb"] = PROVE.replace("fn decide_sub", "fn decide_other")
        self.assertEqual(len(ts.bad_steps(s)), 2)  # one new place, one missing

    def test_a_dropped_check_without_the_question_fails(self):
        s = self.base()
        s["codegen.bmb"] = CODEGEN.replace('self.proved("E0805", e)', "true")
        self.assertEqual(len(ts.bad_steps(s)), 1)
        s["codegen.bmb"] = CODEGEN.replace('self.proved("E0805", e)', 'self.proved("E0803", e)')
        self.assertEqual(len(ts.bad_steps(s)), 1)

    def test_the_port_passes(self):
        d = ts.ROOT / "selfhost"
        self.assertEqual(ts.bad_steps({p.name: p.read_text(encoding="utf-8") for p in d.glob("*.bmb")}), [])


if __name__ == "__main__":
    unittest.main()
