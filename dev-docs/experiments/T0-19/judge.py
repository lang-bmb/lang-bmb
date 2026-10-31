#!/usr/bin/env python3
"""Replays the frozen BVB judge on a contract variant (backlog T0-19).

  judge.py TASK_ID TASKS.bmb      one JSON line
  judge.py --emit DIR TASK_ID TASKS.bmb
                                  also keeps DIR/ws and DIR/accept.bmb (the
                                  hidden check as the harness wrote it), for
                                  `bmb verify --emit-smt OUT DIR/ws DIR/accept.bmb`
  judge.py --all                  every variants/<task>/*.bmb plus the task's
                                  reference and weak_reference, one line each

The workspace is the set's seed plus TASKS.bmb as `src/tasks.bmb`. The verdict
comes from `bench/m1/harness.py` as it is (`judge_write`, unchanged). The
hidden check's own row is reported too (`check`, `check_rlimit`, the first
diagnostic): when the agent body does not verify, `judge_target` is called with
`verified` forced so the contract form is still judged. `tasks_rlimit` is the
workspace's proof cost minus the seed's. `bmb` is $BMB (the harness default
otherwise); T0-19 used the T0-10 compiler (`condition.py build f83cd47d`).
Standard library only.
"""
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
sys.path.insert(0, os.path.join(ROOT, "bench", "m1"))
import harness  # noqa: E402

harness.SET = os.path.join(ROOT, "bench", "bvb")
_bmb = harness.bmb


def reference_unit(task, key):
    code = [o for o in task[key] if o["op"] == "put"][0]["code"]
    return '#[summary("x")]\nmod tasks {\n' + code + "\n}\n"


def judge(tid, text, seed_cost, emit=None):
    task = json.load(open(os.path.join(harness.SET, "tasks", tid + ".json")))
    tmp = tempfile.mkdtemp(prefix="t019-")
    try:
        ws = os.path.join(tmp, "ws")
        shutil.copytree(os.path.join(harness.SET, "seed"), ws)
        with open(os.path.join(ws, "src", "tasks.bmb"), "w") as f:
            f.write(text)
        log = []

        def spy(*a, **k):
            r = _bmb(*a, **k)
            accs = [x for x in a if str(x).endswith(".bmb")]
            if a[0] == "verify" and accs:
                log.append(r[1])
                if emit:
                    shutil.copytree(ws, os.path.join(emit, "ws"), dirs_exist_ok=True,
                                    ignore=shutil.ignore_patterns(".bmb-cache"))
                    shutil.copy(accs[-1], os.path.join(emit, "accept.bmb"))
            return r
        harness.bmb = spy
        v = harness.judge_write(task, ws)
        out = {"task": tid, "verified": v["verified"], "accepted": v["spec_accepted"],
               "reason": [x["reason"] for x in v.get("spec_rejections", [])],
               "tasks_rlimit": v["rlimit_total"] - seed_cost}
        if not v["verified"]:
            log.clear()
            harness.judge_target(task, ws, True)
        for o in log:
            rs = harness.rows(o)
            chk = [r for r in rs if r.get("def") == "bmb_accept::check" and "status" in r]
            out["check"] = chk[0]["status"] if chk else None
            out["check_rlimit"] = chk[0].get("rlimit") if chk else None
            msgs = [r["message"] for r in rs if "bmb_accept::check" in r.get("message", "")]
            if msgs:
                out["check_msg"] = msgs[0]
        return out
    finally:
        harness.bmb = _bmb
        shutil.rmtree(tmp)


def main():
    _, seed_out = _bmb("verify", os.path.join(harness.SET, "seed"))
    seed_cost = harness.proof_cost(seed_out)[0]
    if sys.argv[1:] == ["--all"]:
        vdir = os.path.join(HERE, "variants")
        for tid in sorted(os.listdir(vdir)):
            task = json.load(open(os.path.join(harness.SET, "tasks", tid + ".json")))
            cases = [("reference", reference_unit(task, "reference")),
                     ("weak_reference", reference_unit(task, "weak_reference"))]
            cases += [(f[:-4], open(os.path.join(vdir, tid, f)).read())
                      for f in sorted(os.listdir(os.path.join(vdir, tid)))]
            for name, text in cases:
                print(json.dumps({"variant": name, **judge(tid, text, seed_cost)}), flush=True)
    else:
        args, emit = sys.argv[1:], None
        if args[0] == "--emit":
            emit, args = args[1], args[2:]
            os.makedirs(emit, exist_ok=True)
        tid, path = args
        print(json.dumps(judge(tid, open(path).read(), seed_cost, emit)))


if __name__ == "__main__":
    main()
