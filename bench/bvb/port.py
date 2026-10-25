#!/usr/bin/env python3
"""Builds the frozen BVB task files from their port sources (backlog 2A-08, ADR-0034).

  port.py build     write tasks/<id>.json from port/<id>.task
  port.py check     the task files are exactly what `build` writes (no drift)

A port source is a sequence of sections. A line `@@ NAME` starts section NAME;
`@@ NAME: VALUE` is a one-line section. Sections:

  id, benchmark, origin, origin_path, rules  where the task comes from and the
                                             conversion rules applied (SOURCES.md)
  description                                the natural-language task, shared
                                             word for word by the BMB and Verus prompts
  signature                                  required BMB signature (unit `tasks`)
  accept.specs / .pre / .post / .args        hidden reference contract: the
                                             original Verus spec translated (ADR-0021 §2)
  reference, weak                            BMB code of unit `tasks`: a solution that
                                             passes, and a weak control that must fail
  verus.signature, verus.target              the original Verus signature and name
  verus.specs / .pre / .post / .args         the original spec, verbatim up to renaming
                                             helpers to `bvb_ref_*` (2A-09 spec check)
  verus.reference                            a verified Verus solution (the original
                                             when one exists, see verus.reference_origin)
  verus.reference_origin                     `upstream`, `upstream-triggers` or
                                             `upstream-migrated` (edited, README) or `bvb`
  frozen, given, verus.given                 (optional, together; rule G1) the given
                                             specification functions: their paths, BMB code
                                             (must be in seed/src/given.bmb) and Verus code

`none` as a one-line value means JSON null. Standard library only.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
PORT = os.path.join(HERE, "port")
TASKS = os.path.join(HERE, "tasks")

ONE_LINE = {"frozen", "id", "benchmark", "origin", "origin_path", "rules", "accept.pre", "accept.post", "accept.args",
            "verus.target", "verus.args", "verus.reference_origin"}
OPTIONAL = {"frozen", "given", "verus.given"}
REQUIRED = ONE_LINE | {"description", "signature", "accept.specs", "reference", "weak", "verus.signature",
                       "verus.specs", "verus.pre", "verus.post", "verus.reference"}
BMB_PREFIX = "Create unit `tasks` for this task.\n\n"
BMB_SIGNATURE_NOTE = "\n\nRequired signature (in unit `tasks`):\n"
VERUS_SIGNATURE_NOTE = "\n\nRequired signature:\n"


def parse(path):
    sections, name, buf = {}, None, []

    def flush():
        if name is not None:
            sections[name] = "\n".join(buf).strip("\n")

    for line in open(path, encoding="utf-8").read().splitlines():
        m = re.match(r"^@@ ([a-z_.]+)(?::\s*(.*))?$", line)
        if m:
            flush()
            name, buf = m.group(1), []
            if m.group(2) is not None:
                sections[name] = m.group(2).strip()
                name = None
            continue
        if name is None:
            if line.strip():
                sys.exit(f"{path}: text outside a section: {line!r}")
            continue
        buf.append(line)
    flush()
    missing = REQUIRED - OPTIONAL - set(sections)
    unknown = set(sections) - REQUIRED - OPTIONAL
    if ("frozen" in sections) != ("given" in sections) or ("given" in sections) != ("verus.given" in sections):
        sys.exit(f"{path}: frozen, given and verus.given go together")
    if missing or unknown:
        sys.exit(f"{path}: missing {sorted(missing)}, unknown {sorted(unknown)}")
    return sections


def value(s):
    return None if s == "none" else s


def unit_ops(code):
    return [{"op": "unit", "path": "tasks", "summary": "과제 구현"},
            {"op": "put", "unit": "tasks", "code": code}]


def target_name(signature):
    return re.match(r"fn (\w+)\(", signature.strip()).group(1)


def given_note(intro, code):
    return f"\n\n{intro} (use them, do not change them):\n```\n{code.strip()}\n```"


def squash(text):
    return re.sub(r"\s+", " ", text).strip()


def build_task(s):
    sig = s["signature"].strip()
    vsig = s["verus.signature"].strip()
    bmb_given = verus_given = ""
    if "given" in s:
        seed = open(os.path.join(HERE, "seed", "src", "given.bmb"), encoding="utf-8").read()
        for part in re.split(r"\n(?=spec fn )", s["given"].strip()):
            if squash(part) not in squash(seed):
                sys.exit(f"{s['id']}: `given` is not in seed/src/given.bmb: {part.splitlines()[0]}")
        paths = sorted("given::" + n for n in re.findall(r"spec fn (\w+)", s["given"]))
        if paths != sorted(s["frozen"].split()):
            sys.exit(f"{s['id']}: `frozen` must name exactly the given definitions {paths}")
        bmb_given = given_note("Given specification functions in unit `given`", s["given"])
        verus_given = given_note("The file starts with these given specification functions", s["verus.given"])
    task = {
        "id": s["id"],
        "kind": "write",
        "prompt": BMB_PREFIX + s["description"] + bmb_given + BMB_SIGNATURE_NOTE + sig,
        "unit": "tasks",
        "def": "tasks::" + target_name(sig),
        "signature": sig,
        "accept": {"specs": s["accept.specs"], "pre": value(s["accept.pre"]),
                   "post": s["accept.post"], "args": s["accept.args"]},
        "reference": unit_ops(s["reference"]),
        "weak_reference": unit_ops(s["weak"]),
        "source": {"benchmark": s["benchmark"], "origin": s["origin"], "path": s["origin_path"],
                   "rules": s["rules"].split()},
        "verus": {
            "prompt": s["description"] + verus_given + VERUS_SIGNATURE_NOTE + vsig,
            "signature": vsig,
            "target": s["verus.target"],
            "specs": s["verus.specs"],
            "pre": value(s["verus.pre"]),
            "post": s["verus.post"],
            "args": s["verus.args"],
            "reference": s["verus.reference"],
            "reference_origin": s["verus.reference_origin"],
        },
    }
    if "given" in s:
        task["frozen"] = s["frozen"].split()
        task["verus"]["given"] = s["verus.given"]
    return task


def rendered():
    out = {}
    for f in sorted(os.listdir(PORT)):
        if f.endswith(".task"):
            s = parse(os.path.join(PORT, f))
            if s["id"] + ".task" != f:
                sys.exit(f"{f}: id {s['id']} does not match the file name")
            out[s["id"] + ".json"] = json.dumps(build_task(s), ensure_ascii=False, indent=2) + "\n"
    return out


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    want = rendered()
    if cmd == "build":
        os.makedirs(TASKS, exist_ok=True)
        for f in os.listdir(TASKS):
            if f not in want:
                os.unlink(os.path.join(TASKS, f))
        for f, text in want.items():
            with open(os.path.join(TASKS, f), "w", encoding="utf-8") as fh:
                fh.write(text)
        print(f"{len(want)} tasks written")
    elif cmd == "check":
        have = sorted(os.listdir(TASKS)) if os.path.isdir(TASKS) else []
        bad = [f for f in sorted(set(have) | set(want))
               if f not in want or f not in have or open(os.path.join(TASKS, f), encoding="utf-8").read() != want[f]]
        if bad:
            sys.exit("tasks/ differs from port/ (run port.py build): " + ", ".join(bad))
        print(f"{len(want)} tasks match their port sources")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
