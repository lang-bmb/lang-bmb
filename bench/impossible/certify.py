#!/usr/bin/env python3
"""Checks that every BMB-Impossible task is what it claims (backlog 2A-13).

  certify.py [--tasks ID,..]

For each `given` task, on the seed plus the task's `setup`:
  - the starting workspace commits and only the given definition fails to
    prove (the placeholder body), so the gate fails at the start;
  - `feasible: false`: the task's hidden `certificate` (unit `bmb_cert`)
    proves. `refute` calls the given definition on witness inputs and
    ensures an opaque `false`, so the given contract has no result there;
    `reach` calls `refute` on concrete inputs, so the witness inputs exist
    and satisfy the given `requires`. Together: no implementation of the
    contract exists, and the gate can pass only if the contract changes.
  - `feasible: true` (the control): no certificate, a `reference`, and a
    `twin` that is an infeasible task of the set.

One JSON line per task, then a summary; exit code 1 when a task fails.
Uses `bmb` from $BMB like the harness. Standard library only.
"""
import argparse
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(HERE), "m1"))
import harness  # noqa: E402

CERT_UNIT = "bmb_cert"


def load_tasks(set_dir=HERE):
    tdir = os.path.join(set_dir, "tasks")
    tasks = []
    for name in sorted(os.listdir(tdir)):
        with open(os.path.join(tdir, name)) as f:
            tasks.append(json.load(f))
    return tasks


def certify(task, ids, set_dir=HERE):
    """The problems found with `task` (empty when it is what it claims)."""
    problems = []
    tmp = tempfile.mkdtemp(prefix=f"bmb-certify-{task['id']}-")
    try:
        ws = os.path.join(tmp, "ws")
        shutil.copytree(os.path.join(set_dir, "seed"), ws)
        try:
            harness.apply_setup(task, ws)
        except RuntimeError as e:
            return [str(e)]
        code, out = harness.bmb("verify", ws)
        status = {r["def"]: r["status"] for r in harness.rows(out) if "status" in r}
        if code == 0 or status.get(task["def"]) != "failed":
            problems.append(f"the given {task['def']} does not fail to prove at the start")
        others = sorted(d for d, s in status.items() if d != task["def"] and s != "verified")
        if others:
            problems.append(f"other definitions fail at the start: {others}")
        if task["feasible"]:
            if "certificate" in task:
                problems.append("a feasible task has a certificate")
            if not task.get("reference"):
                problems.append("a feasible task has no reference")
            if task.get("twin") not in ids:
                problems.append(f"twin {task.get('twin')} is not an infeasible task of the set")
            return problems
        cert = task.get("certificate", "")
        if not cert:
            return problems + ["an infeasible task has no certificate"]
        if f"{task['def']}(" not in cert or "refute(" not in cert:
            problems.append(f"the certificate does not call {task['def']} and refute")
        path = os.path.join(tmp, "cert.bmb")
        with open(path, "w") as f:
            f.write(cert)
        _, out = harness.bmb("verify", ws, path)
        rs = harness.rows(out)
        errors = [r for r in rs if r.get("severity") == "error" and r.get("file") == path]
        cert_status = {r["def"]: r["status"] for r in rs if r.get("def", "").startswith(CERT_UNIT + "::")}
        if errors:
            problems.append(f"certificate errors: {[e['message'] for e in errors]}")
        for d in ("refute", "reach"):
            if cert_status.get(f"{CERT_UNIT}::{d}") != "verified":
                problems.append(f"{CERT_UNIT}::{d} is not verified")
        bad = sorted(d for d, s in cert_status.items() if s != "verified")
        if bad:
            problems.append(f"unproved certificate definitions: {bad}")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return problems


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tasks")
    args = ap.parse_args()
    harness.SET = HERE
    harness.check_manifest()
    tasks = load_tasks()
    infeasible = {t["id"] for t in tasks if not t["feasible"]}
    if args.tasks:
        wanted = set(args.tasks.split(","))
        tasks = [t for t in tasks if t["id"] in wanted]
    failed = 0
    for t in tasks:
        if t.get("kind") != "given":
            problems = ["not a `given` task"]
        else:
            problems = certify(t, infeasible)
        failed += bool(problems)
        print(json.dumps({"id": t["id"], "feasible": t.get("feasible"), "ok": not problems, "problems": problems},
                         ensure_ascii=False))
    print(json.dumps({"summary": True, "tasks": len(tasks), "failed": failed,
                      "infeasible": sum(1 for t in tasks if not t.get("feasible")),
                      "feasible": sum(1 for t in tasks if t.get("feasible"))}))
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
