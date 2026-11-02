"""Hidden judge of BRB-EFFECT (H-EFFECT, ADR-0078; backlog T1-43).

BRB's judge (`bench/brb/judge.py`, frozen), with one addition: a task's
behaviour includes the lines it prints. A task's `model(*args)` returns
`(result, lines)`, the return value and the integers it prints, one per line,
in order. A case passes when the return value, every `&mut` slice and the
printed lines all match.

How the lines are read: the BRB drivers print one header line `K ok ..` (or
`K panic`) after case K returns, and flush. The target prints through the
same stdout (BMB `print_i64` is the C library's `printf`, the driver's too;
Rust `println!` and the judge's `write!` share `std::io::stdout()`), so the
lines a case prints come right before its header. `run_cases` here keeps them
(a line is a header only when it names the next case). A case that crashes or
hangs loses its lines with its verdict.

Everything else is BRB's code, loaded as a private copy so that its
`expected`, `score`, `run_cases` and `domain_unit` can be replaced here
without touching the frozen file: BRB's `judge_bmb`, `judge_rust` and
`judge_mutants` call them through their module's globals. `domain_unit`
differs only in the effect of the hidden caller (`io`: it calls a target
that prints).

Standard library only.
"""
import copy
import importlib.util
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
BRB = os.path.join(os.path.dirname(HERE), "brb")


def _load():
    spec = importlib.util.spec_from_file_location("brb_effect_base_judge", os.path.join(BRB, "judge.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


base = _load()

load_tasks = base.load_tasks
task_prompt = base.task_prompt
hidden_cases = base.hidden_cases
encode = base.encode
not_delivered = base.not_delivered
params_text = base.params_text
sig_bmb = base.sig_bmb
sig_rust = base.sig_rust


def expected(task, args):
    """(tokens, lines): the return value and `&mut` slices as BRB's drivers
    print them, and the printed lines."""
    args = copy.deepcopy(list(args))
    r, lines = task.model(*args)
    toks = [] if task.RET == "()" else [str(int(r))]
    for (_, ty), v in zip(task.PARAMS, args):
        if ty == "&mut [i64]":
            toks.append(str(len(v)))
            toks += [str(x) for x in v]
    return toks, [str(int(x)) for x in lines]


def parse_output(out, start, n):
    """{k: ("ok", tokens, lines) | ("panic",)} for the cases a driver run
    reported, and the last case it reported (start - 1 when none)."""
    got, pending, nxt = {}, [], start
    for line in out.splitlines():
        parts = line.split()
        if nxt < n and len(parts) >= 2 and parts[0] == str(nxt) and parts[1] in ("ok", "panic"):
            got[nxt] = ("ok", parts[2:], pending) if parts[1] == "ok" else ("panic",)
            pending = []
            nxt += 1
        else:
            pending.append(line.strip())
    return got, nxt - 1


def run_cases(argv_for, n, timeout=base.PROCESS_TIMEOUT):
    """BRB's `run_cases` (restart after a case that kills or hangs the
    process), keeping each case's printed lines."""
    got = {}
    start = 0
    restarts = 0
    while start < n:
        timed = False
        try:
            p = base.spawn_retrying(argv_for(start), timeout)
            out = p.stdout
        except subprocess.TimeoutExpired as e:
            out, timed = e.stdout or b"", True
        part, last = parse_output(out.decode(errors="replace"), start, n)
        got.update(part)
        nxt = last + 1
        if nxt >= n:
            break
        got[nxt] = ("timeout",) if timed else ("crash",)
        start = nxt + 1
        restarts += 1
        if restarts > base.MAX_RESTARTS:
            for k in range(start, n):
                got[k] = ("not run",)
            break
    return got


def score(task, cs, got):
    """Per-case verdicts against the model; `wrong_output` counts the passing
    return values whose printed lines differ."""
    kinds = {"pass": 0, "wrong": 0, "panic": 0, "crash": 0, "timeout": 0, "not run": 0}
    failing = []
    lines_only = 0
    for k, args in enumerate(cs):
        g = got.get(k, ("not run",))
        want = expected(task, args)
        if g[0] == "ok":
            kind = "pass" if (g[1], g[2]) == want else "wrong"
            lines_only += int(g[1] == want[0] and g[2] != want[1])
        else:
            kind = g[0]
        kinds[kind] += 1
        if kind != "pass" and len(failing) < 3:
            failing.append({"case": encode(task, args)[:200], "kind": kind,
                            "want": (" ".join(want[0]) + " | " + " ".join(want[1]))[:160],
                            "got": (" ".join(g[1]) + " | " + " ".join(g[2]))[:160] if g[0] == "ok" else g[0]})
    return {"cases": len(cs), "passed": kinds["pass"], "failed": len(cs) - kinds["pass"],
            "fail_kinds": {k: v for k, v in kinds.items() if k != "pass" and v}, "failing": failing,
            "wrong_output": lines_only}


_base_domain_unit = base.domain_unit


def domain_unit(task, hidden, specs):
    return _base_domain_unit(task, hidden, specs).replace("#[effects(pure)]", "#[effects(io)]", 1)


base.expected = expected
base.run_cases = run_cases
base.score = score
base.domain_unit = domain_unit

judge_bmb = base.judge_bmb
judge_rust = base.judge_rust
judge_mutants = base.judge_mutants
