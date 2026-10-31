#!/usr/bin/env python3
"""The same measurement condition across commits (ADR-0070).

  condition.py build COMMIT                      build `bmb` from COMMIT's compiler
  condition.py same --base COMMIT --sets a,b     is HEAD's harness the same instrument?
  condition.py probe --tree DIR --set NAME ...    (internal: one probe run, one tree per process)

A run's condition is what the model is shown and how the result is judged:
the compiler (diagnostics, proofs, tool descriptions), the solver, the frozen
set, the agent settings, and the harness code that turns tool results into
model requests and workspaces into verdicts. Two runs are under the same
condition when (1) they use a `bmb` built from the same `compiler/` tree,
(2) their sets have the same MANIFEST, and (3) the harness of the later commit
is equivalent to the earlier one's, checked here: both harnesses are driven by
the same scripted model, and every request body the model would receive and
every verdict field must be byte-identical.

`same` checks (3). For each set, provider (`openai`, `anthropic`) and script
(`oracle`: the reference solution, then a stop; `stress`: the reference, an
unknown tool, malformed arguments, then calls past the tool-call limit) it runs
the set's harness from a worktree at `--base` and from this checkout, both with
the same `bmb`, records the request bodies through the real adapters
(`model_api.OpenAIAgent`, `anthropic_agent.AnthropicAgent`) with a fake
transport, and compares them and the result rows. Fields the later harness
adds are allowed; changed or missing fields, and `*_seconds` timings aside,
are not. The report (`bench/conditions/BASE-HEAD.json`) is the evidence a run
made with `run_models.py --same-as BASE` refers to.

`build` makes `target/conditions/COMMIT/bmb` from a temporary worktree with
`provenance.json` (commit, `compiler/` tree, binary hash), so the compiler of
a run is known by construction.

What the scripts do not reach is not checked: an adapter path only a real
model takes (a `length` stop, an API error) is compared only as code. Standard
library only.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import types

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
PY = sys.executable
SETS = ("m1", "m1b", "bvb", "bvb-verus", "brb-bmb", "brb-rust")
PROVIDERS = {"openai": "openai:qwen3.8-27b", "anthropic": "anthropic:claude-sonnet-5-5"}
SCRIPTS = ("oracle", "stress")


def git(*args, cwd=ROOT):
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def sha256_file(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


# ---------- pinned compiler ----------

def build_dir(commit):
    return os.path.join(ROOT, "target", "conditions", commit)


def provenance(commit):
    """The provenance of the `bmb` built from `commit`, or None."""
    path = os.path.join(build_dir(commit), "provenance.json")
    if not os.path.exists(path):
        return None
    with open(path) as f:
        p = json.load(f)
    if sha256_file(os.path.join(build_dir(commit), "bmb")) != p["bmb_sha256"]:
        return None
    return p


def build(commit):
    commit = git("rev-parse", commit)
    if p := provenance(commit):
        return p
    tmp = tempfile.mkdtemp(prefix="bmb-condition-")
    tree = os.path.join(tmp, "tree")
    try:
        git("worktree", "add", "--detach", tree, commit)
        target = os.path.join(tmp, "target")
        env = {**os.environ, "CARGO_TARGET_DIR": target}
        subprocess.run(["cargo", "build", "--bin", "bmb"], cwd=os.path.join(tree, "compiler"), env=env, check=True)
        os.makedirs(build_dir(commit), exist_ok=True)
        out = os.path.join(build_dir(commit), "bmb")
        shutil.copy2(os.path.join(target, "debug", "bmb"), out)
        p = {"commit": commit, "compiler_tree": git("rev-parse", f"{commit}:compiler"), "bmb_sha256": sha256_file(out)}
        with open(os.path.join(build_dir(commit), "provenance.json"), "w") as f:
            json.dump(p, f, indent=1)
        return p
    finally:
        subprocess.run(["git", "worktree", "remove", "--force", tree], cwd=ROOT, capture_output=True)
        shutil.rmtree(tmp, ignore_errors=True)


# ---------- probe (one tree, one process) ----------

def harness_of(tree, set_name):
    """(harness file, argv after `run`'s options) for a set in `tree`."""
    if set_name in ("m1", "m1b", "bvb"):
        return os.path.join(tree, "bench", "m1", "harness.py"), ["--set", os.path.join(tree, "bench", set_name), "run"]
    if set_name == "bvb-verus":
        return os.path.join(tree, "bench", "bvb", "verus", "harness.py"), ["run"]
    arm = set_name.split("-")[1]
    return os.path.join(tree, "bench", "brb", "harness.py"), ["run", "--arm", arm] + (
        ["--proptest", "off"] if arm == "rust" else [])


def task_id(task):
    return task["id"] if isinstance(task, dict) else task.ID


def script_turns(oracle, script):
    """The model's turns: lists of (tool, arguments) where arguments is a dict
    or, for malformed arguments, a string. The last turn is empty (a stop)."""
    calls = []
    while step := oracle.next_calls([]):
        calls += step
    turns = [[c] for c in calls]
    if script == "stress":
        name = calls[-1][0] if calls else "verify"
        turns.append([("no_such_tool", {}), (name, "{not json")])
        repeat = calls[-1] if calls else ("verify", {})
        turns += [[repeat] * 4 for _ in range(12)]  # past the 40-call limit
    return turns + [[]]


class Transcript:
    def __init__(self):
        self.bodies = {}

    def add(self, tid, body):
        self.bodies.setdefault(tid, []).append(json.loads(json.dumps(body, default=vars)))


def fake_openai(model_api, model, system_prompt, task_prompt, turns, record):
    n = [0]

    def post(_url, body, *_a, **_k):
        record(body)
        calls = turns.pop(0) if turns else []
        if not calls:
            return {"choices": [{"finish_reason": "stop", "message": {"content": "done"}}], "usage": {}}
        tcs = []
        for name, args in calls:
            n[0] += 1
            raw = args if isinstance(args, str) else json.dumps(args)
            tcs.append({"id": f"call_{n[0]}", "type": "function", "function": {"name": name, "arguments": raw}})
        return {"choices": [{"finish_reason": "tool_calls", "message": {"content": "", "tool_calls": tcs}}],
                "usage": {}}

    return model_api.OpenAIAgent(model, system_prompt, task_prompt, post=post)


def fake_anthropic(anthropic_mod, model, system_prompt, task_prompt, turns, record):
    ns = types.SimpleNamespace
    n = [0]

    def create(**kw):
        record(kw)
        calls = turns.pop(0) if turns else []
        usage = ns(input_tokens=0, output_tokens=0)
        if not calls:
            return ns(usage=usage, stop_reason="end_turn", content=[ns(type="text", text="done")])
        blocks = []
        for name, args in calls:
            n[0] += 1
            blocks.append(ns(type="tool_use", id=f"toolu_{n[0]}", name=name,
                             input=args if isinstance(args, dict) else {"arguments": args}))
        return ns(usage=usage, stop_reason="tool_use", content=blocks)

    client = ns(messages=ns(create=create))
    return anthropic_mod.AnthropicAgent(model, system_prompt, task_prompt, client=client)


def probe(tree, set_name, provider, script, out, tasks=None):
    """Runs `set_name`'s harness from `tree` with the scripted model; writes
    the results file `out` and the request bodies `out + '.bodies.json'`."""
    m1_dir = os.path.join(tree, "bench", "m1")
    sys.path.insert(0, m1_dir)
    import model_api  # noqa: E402  (the tree's own)
    import anthropic_agent as anthropic_mod  # noqa: E402
    path, argv = harness_of(tree, set_name)
    spec = importlib.util.spec_from_file_location("probed_harness", path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["probed_harness"] = mod
    spec.loader.exec_module(mod)
    kind = PROVIDERS[provider]
    transcript = Transcript()
    orig_make, orig_model = mod.make_agent, model_api.make_model_agent

    def make_agent(k, task, *rest):
        if k != kind:
            return orig_make(k, task, *rest)
        turns = script_turns(orig_make("oracle", task, *rest), script)
        tid = task_id(task)

        def make_model(_kind, system_prompt, task_prompt):
            model = kind.split(":", 1)[1]
            if provider == "openai":
                return fake_openai(model_api, model, system_prompt, task_prompt, turns,
                                    lambda b: transcript.add(tid, b))
            return fake_anthropic(anthropic_mod, model, system_prompt, task_prompt, turns,
                                   lambda b: transcript.add(tid, b))

        model_api.make_model_agent = make_model
        try:
            return orig_make(k, task, *rest)
        finally:
            model_api.make_model_agent = orig_model

    mod.make_agent = make_agent
    sys.argv = [path] + argv + ["--agent", kind, "--out", out] + (["--tasks", tasks] if tasks else [])
    try:
        mod.main()
    except SystemExit as e:  # `--expect` is not passed; anything else is a failure
        if e.code not in (0, None):
            raise
    with open(out + ".bodies.json", "w") as f:
        json.dump(transcript.bodies, f, ensure_ascii=False, sort_keys=True)


# ---------- compare ----------

def normalize(text, tree):
    """Temporary directories and the tree's location are not part of the condition."""
    text = text.replace(tree, "<tree>")
    return re.sub(re.escape(tempfile.gettempdir()) + r"/[^/\"\\\s]+", "<tmp>", text)


def result_rows(path):
    rows = {}
    with open(path) as f:
        for line in f:
            if line.strip():
                r = json.loads(line)
                if "id" in r:
                    rows[r["id"]] = r
    return rows


def compare(base_out, base_tree, head_out, head_tree):
    """Differences (strings) between two probe runs; empty when equivalent."""
    diffs = []
    with open(base_out + ".bodies.json") as f:
        a = json.loads(normalize(f.read(), base_tree))
    with open(head_out + ".bodies.json") as f:
        b = json.loads(normalize(f.read(), head_tree))
    for tid in sorted(set(a) | set(b)):
        x, y = a.get(tid, []), b.get(tid, [])
        if len(x) != len(y):
            diffs.append(f"{tid}: {len(x)} requests vs {len(y)}")
            continue
        for i, (p, q) in enumerate(zip(x, y)):
            if p != q:
                keys = sorted(k for k in set(p) | set(q) if p.get(k) != q.get(k))
                diffs.append(f"{tid}: request {i + 1} differs in {keys}")
                break
    ra = json.loads(normalize(json.dumps(result_rows(base_out)), base_tree))
    rb = json.loads(normalize(json.dumps(result_rows(head_out)), head_tree))
    for tid in sorted(set(ra) | set(rb)):
        if tid not in ra or tid not in rb:
            diffs.append(f"{tid}: result row only in {'head' if tid in rb else 'base'}")
            continue
        for k, v in ra[tid].items():
            if k.endswith("_seconds"):
                continue
            if k not in rb[tid]:
                diffs.append(f"{tid}: field {k} missing in head")
            elif rb[tid][k] != v:
                diffs.append(f"{tid}: field {k}: {json.dumps(v)[:80]} vs {json.dumps(rb[tid][k])[:80]}")
    return diffs


def run_probe(tree, set_name, provider, script, out, bmb, tasks):
    env = {**os.environ, "BMB": bmb}
    env.pop("BMB_CONTEXT_LOG", None)
    argv = [PY, os.path.join(HERE, "condition.py"), "probe", "--tree", tree, "--set", set_name,
            "--provider", provider, "--script", script, "--out", out] + (["--tasks", tasks] if tasks else [])
    r = subprocess.run(argv, cwd=tree, env=env, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"probe {set_name} {provider} {script} in {tree} failed:\n{r.stderr[-2000:]}")


def same(base, sets, providers, scripts, bmb, tasks=None, head_tree=ROOT):
    """The equivalence report of `head_tree`'s harness against `base`'s."""
    base = git("rev-parse", base)
    if harness_changed("HEAD"):
        raise SystemExit("uncommitted harness changes: the report must name the harness that was checked")
    tmp = tempfile.mkdtemp(prefix="bmb-same-")
    base_tree = os.path.join(tmp, "base")
    report = {"base": base, "head": git("rev-parse", "HEAD", cwd=head_tree), "bmb_sha256": sha256_file(bmb),
              "scripts": list(scripts), "providers": list(providers), "sets": {}}
    try:
        git("worktree", "add", "--detach", base_tree, base)
        for s in sets:
            entry = {"same": True, "diffs": []}
            if manifests(base, s) != manifests(report["head"], s):
                entry["same"] = False
                entry["diffs"].append("MANIFEST differs (frozen inputs or hidden mutants)")
            for provider in providers:
                for script in scripts:
                    outs = []
                    for label, tree in (("base", base_tree), ("head", head_tree)):
                        out = os.path.join(tmp, f"{s}-{provider}-{script}-{label}.jsonl")
                        run_probe(tree, s, provider, script, out, bmb, tasks)
                        outs.append((out, tree))
                    diffs = compare(outs[0][0], outs[0][1], outs[1][0], outs[1][1])
                    entry["tasks"] = len(result_rows(outs[1][0]))
                    if diffs:
                        entry["same"] = False
                        entry["diffs"] += [f"{provider}/{script}: {d}" for d in diffs[:20]]
            report["sets"][s] = entry
            print(f"condition: {s}: {'same' if entry['same'] else 'DIFFERENT'}", file=sys.stderr)
    finally:
        subprocess.run(["git", "worktree", "remove", "--force", base_tree], cwd=ROOT, capture_output=True)
        shutil.rmtree(tmp, ignore_errors=True)
    report["same"] = all(e["same"] for e in report["sets"].values())
    return report


SET_DIRS = {"m1": "bench/m1", "m1b": "bench/m1b", "bvb": "bench/bvb", "bvb-verus": "bench/bvb",
            "brb-bmb": "bench/brb", "brb-rust": "bench/brb"}
# The code and frozen inputs of a run: the four harness directories (with
# their sets) and the orchestrator. Results, reports and tests may differ.
HARNESS_PATHS = ["bench/m1", "bench/m1b", "bench/bvb", "bench/brb", "bench/run_models.py",
                 ":(exclude,glob)bench/**/results/**", ":(exclude,glob)bench/**/test_*.py"]


# Files a MANIFEST freezes that are instrument code, not task inputs: the
# replay check stands for their identity (ADR-0070 §2). Prompts, tasks, seeds,
# hidden cases and templates must stay byte-identical.
INSTRUMENT = {"bench/brb/MANIFEST": {"harness.py", "judge.py", "rust_tools.py"}}


def manifests(commit, set_name):
    """{path: frozen task-input lines} of every MANIFEST of the set (frozen
    inputs, hidden mutants) at `commit`."""
    files = git("ls-tree", "-r", "--name-only", commit, "--", SET_DIRS[set_name]).split("\n")
    out = {}
    for f in files:
        if os.path.basename(f) == "MANIFEST":
            skip = INSTRUMENT.get(f, set())
            lines = git("show", f"{commit}:{f}").split("\n")
            out[f] = [x for x in lines if x.split("  ", 1)[-1] not in skip]
    return out


def harness_changed(commit):
    """Whether the harnesses, their sets or the orchestrator differ between
    `commit` and the working tree (uncommitted changes included)."""
    r = subprocess.run(["git", "diff", "--quiet", commit, "--", *HARNESS_PATHS], cwd=ROOT)
    return r.returncode != 0


def report_path(base, head):
    return os.path.join(HERE, "conditions", f"{base[:12]}-{head[:12]}.json")


def find_report(base, set_name, provider):
    """The committed report that makes this checkout's harness the same
    instrument as `base`'s for the set and provider (whole set, both scripts,
    and no harness change since the report's commit), or None."""
    d = os.path.join(HERE, "conditions")
    for name in sorted(os.listdir(d)) if os.path.isdir(d) else []:
        path = os.path.join(d, name)
        with open(path) as f:
            r = json.load(f)
        e = r.get("sets", {}).get(set_name)
        if (r.get("base") == base and e and e.get("same") and provider in r.get("providers", [])
                and set(SCRIPTS) <= set(r.get("scripts", [])) and not r.get("tasks_limit")
                and not harness_changed(r["head"])):
            return path
    return None


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build")
    b.add_argument("commit")
    s = sub.add_parser("same")
    s.add_argument("--base", required=True)
    s.add_argument("--sets", required=True)
    s.add_argument("--providers", default=",".join(PROVIDERS))
    s.add_argument("--scripts", default=",".join(SCRIPTS))
    s.add_argument("--bmb", help="default: the build of --base (`build`)")
    s.add_argument("--tasks", help="comma list (a partial check; its report cannot be cited by a run)")
    s.add_argument("--report", help="default: bench/conditions/BASE-HEAD.json")
    p = sub.add_parser("probe")
    p.add_argument("--tree", required=True)
    p.add_argument("--set", required=True, choices=SETS)
    p.add_argument("--provider", required=True, choices=list(PROVIDERS))
    p.add_argument("--script", required=True, choices=SCRIPTS)
    p.add_argument("--out", required=True)
    p.add_argument("--tasks")
    args = ap.parse_args()

    if args.cmd == "build":
        print(json.dumps(build(args.commit)))
    elif args.cmd == "probe":
        probe(os.path.abspath(args.tree), args.set, args.provider, args.script, args.out, args.tasks)
    else:
        sets = [x for x in args.sets.split(",") if x]
        if unknown := [x for x in sets if x not in SETS]:
            sys.exit(f"unknown sets: {unknown}")
        base = git("rev-parse", args.base)
        bmb = args.bmb or os.path.join(build_dir(base), "bmb")
        if not args.bmb and not provenance(base):
            sys.exit(f"no build of {base[:12]}: run `condition.py build {base[:12]}` first")
        report = same(base, sets, args.providers.split(","), args.scripts.split(","), bmb, args.tasks)
        if args.tasks:
            report["tasks_limit"] = args.tasks
        out = args.report or report_path(report["base"], report["head"])
        os.makedirs(os.path.dirname(out), exist_ok=True)
        with open(out, "w") as f:
            json.dump(report, f, ensure_ascii=False, indent=1)
            f.write("\n")
        print(out)
        if not report["same"]:
            sys.exit("the harnesses differ: not the same condition")


if __name__ == "__main__":
    main()
