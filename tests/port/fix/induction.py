#!/usr/bin/env python3
"""`bmb fix --induction` on the programs of the Rust compiler's own tests of it.

    induction.py BMB ROOT

The first program is `tests/run/max_list.bmb` without its induction lemma
and the lemma's call, cut out as `compiler/tests/fix_cmd.rs` cuts them: the
gate does not prove `max_list` then, and the draft from `contains_prefix`
to the quantified predicate closes it. It is fixed without the flag (the
prover does not run and nothing changes), with it (the report), with
`--print`, with `--write`, and once more over what was written (no round).
The second is the program of that file whose drafts the gate does not
prove (`../lint/fix-27.txt`): nothing is written. Then two programs of the
corpus whose failures no draft answers, and one that does not check.
Prints each command's output and exit code; the files are left as written.
"""
import shutil
import subprocess
import sys

bmb, root = sys.argv[1], sys.argv[2]


def run(args):
    p = subprocess.run([bmb] + args, capture_output=True)
    sys.stdout.write(p.stdout.decode("utf-8", "replace"))
    sys.stdout.write(p.stderr.decode("utf-8", "replace"))
    print("exit", p.returncode)
    sys.stdout.flush()


text = open(root + "tests/run/max_list.bmb", encoding="utf-8").read()
start = text.index("    // Recursion to quantifier")
end = text.index("    // The hidden contract of the task")
text = text[:start] + text[end:]
call = "        proof {\n            contains_has_index(lst, lst.len(), m);\n        }\n"
assert call in text
with open("max_list.bmb", "w", encoding="utf-8") as f:
    f.write(text.replace(call, ""))
run(["fix", "max_list.bmb"])
run(["fix", "--induction", "max_list.bmb"])
run(["fix", "--induction", "--print", "max_list.bmb"])
run(["fix", "--write", "--induction", "max_list.bmb"])
run(["fix", "--write", "--induction", "max_list.bmb"])
run(["verify", "max_list.bmb"])
shutil.copy(root + "tests/port/lint/fix-27.txt", "unproved.bmb")
run(["fix", "--induction", "--write", "unproved.bmb"])
shutil.copy(root + "tests/verify/fail/loops.bmb", "loops.bmb")
run(["fix", "--induction", "--write", "loops.bmb"])
run(["fix", "--induction", root + "tests/verify/fail/contracts.bmb"])
run(["fix", "--induction", root + "tests/check/err/types.bmb"])
