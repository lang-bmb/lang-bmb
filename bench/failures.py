#!/usr/bin/env python3
"""Where each failed session ended, from the result rows (backlog T0-16, T0-10 report).

  failures.py RESULTS.jsonl..     one table per file, then the failed sessions

Every failed session gets one ending (how the session stopped) and one verdict
(the first judge check it did not pass). The ending is `api_error`, `length`
(a model response used its whole output limit without a tool call), `limit`
(the tool-call limit or, BRB, the session output-token limit), `refusal`, or
`stop` (the model stopped on its own). The verdict is, for write tasks (m1,
m1b, BVB, Verus): `no-commit` (BMB: no transaction committed), `unverified`,
`signature`, `given-changed` (BVB given definitions), `spec-refuted`,
`spec-undecided`, `spec`; for BRB: `not-delivered (<why>)`, `hidden-cases`,
`domain`; for navigation: `navigate`. This is attribution to the judge step,
not to a layer: the layer needs the session record (backlog INF-17), and runs
without records (T0-10) stop here. Standard library only.
"""
import collections
import json
import os
import sys


def rows(path):
    with open(path) as f:
        recs = [json.loads(line) for line in f if line.strip()]
    head = next((r for r in recs if r.get("summary") or r.get("partial")), {})
    return head, [r for r in recs if "id" in r]


def ending(r):
    if (r.get("model_usage") or {}).get("api_error") or r.get("stop") == "api_error":
        return "api_error"
    if r.get("limit_hit"):
        return "limit"
    if r.get("stop") in ("length", "max_tokens"):
        return "length"
    if r.get("stop") == "refusal":
        return "refusal"
    return "stop"


def verdict(r):
    if r.get("kind") == "navigate":
        return "navigate"
    if "delivered" in r:  # BRB
        if not r["delivered"]:
            return f"not-delivered ({r.get('why', '?')[:40]})"
        if r.get("failed"):
            return "hidden-cases"
        return "domain"
    if r.get("commits") == 0:
        return "no-commit"
    if not r.get("verified"):
        return "unverified"
    if not r.get("signature_ok"):
        return "signature"
    if r.get("given_intact") is False:
        return "given-changed"
    reasons = {j.get("reason") for j in r.get("spec_rejections", [])}
    if "refuted" in reasons:
        return "spec-refuted"
    if "undecided" in reasons:
        return "spec-undecided"
    return "spec"


def table(path):
    head, rs = rows(path)
    failed = [r for r in rs if not r.get("success")]
    name = head.get("task_set", os.path.basename(path)) + (f" ({head['arm']})" if head.get("arm") else "")
    out = [f"## {name}: 실패 {len(failed)}/{len(rs)}", "", "| 끝남 \\ 판정 | " +
           " | ".join(vs := sorted({verdict(r) for r in failed})) + " | 합 |",
           "| --- | " + " | ".join("---" for _ in vs) + " | --- |"]
    grid = collections.Counter((ending(r), verdict(r)) for r in failed)
    for e in sorted({ending(r) for r in failed}):
        cells = [str(grid[(e, v)] or "") for v in vs]
        out.append(f"| {e} | " + " | ".join(cells) + f" | {sum(grid[(e, v)] for v in vs)} |")
    out += ["", "| 과제 | 끝남 | 판정 | 도구 호출 | 출력 토큰 |", "| --- | --- | --- | --- | --- |"]
    for r in failed:
        out.append(f"| {r['id']} | {ending(r)} | {verdict(r)} | {r.get('tool_calls', '')} | "
                   f"{(r.get('model_usage') or {}).get('output_tokens', '')} |")
    return "\n".join(out) + "\n"


def main(argv=None):
    paths = (argv if argv is not None else sys.argv[1:])
    if not paths:
        sys.exit(__doc__)
    sys.stdout.write("\n".join(table(p) for p in paths))


if __name__ == "__main__":
    main()
