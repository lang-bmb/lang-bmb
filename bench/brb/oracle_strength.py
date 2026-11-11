#!/usr/bin/env python3
"""Oracle strength by the tasks' mutants (ADR-0119, backlog T0-40).

  oracle_strength.py run RESULTS.jsonl --out OUT.jsonl [--only ID ..]
  oracle_strength.py summarize BMB_OUT.jsonl RUST_OUT.jsonl [--aux BMB RUST]
  oracle_strength.py check [--record] reference and weak solutions (the gate)

H-ORACLE asks whether a proof gate lets fewer defects through than tests do.
The session results only tell that when a model writes a defect. This reads
the stored final sources of each session (`final_sources`, no model run) and
asks of every mutant of the task (`RUST_MUTANTS`: defective implementations
the hidden cases all kill) whether the session's own check would have caught
it:

  rust  the mutant replaces the agent's function in the final crate and the
        agent's tests run (`cargo test`, PROPTEST_RNG_SEED fixed): killed
        when they fail. A mutant the crate does not build with is left out.
  bmb   witnesses are hidden cases on which the mutant differs from the
        model. The agent's contract kills the mutant when `bmb verify`
        proves, from the contract alone, that the function cannot return the
        mutant's outputs on the witnesses (`bench/m1/bsb.py`). A mutant that
        only panics, overflows or hangs is killed by the gate when the
        contract admits the witness input: a proved program has no such run.
        Undecided is not killed.

Only sessions whose own check passed at the end count (`final_check_ok`).
Not frozen: it reads results, it produces none a model saw.
Standard library only.
"""
import argparse
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.append(os.path.join(os.path.dirname(HERE), "m1"))
sys.dont_write_bytecode = True
import bsb  # noqa: E402
import harness  # noqa: E402
import judge  # noqa: E402

MAX_WITNESSES = 4  # output witnesses per mutant, in case order
PROPTEST_SEED = "0"
TRAPS = ("panic", "crash", "timeout")
T975 = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262, 10: 2.228,
        11: 2.201, 12: 2.179, 13: 2.160, 14: 2.145, 15: 2.131, 16: 2.120, 17: 2.110, 18: 2.101, 19: 2.093,
        20: 2.086, 21: 2.080, 22: 2.074, 23: 2.069, 24: 2.064, 25: 2.060, 26: 2.056, 27: 2.052, 28: 2.048,
        29: 2.045, 30: 2.042}


# ---------------------------------------------------------------- mutants


def value(ty, tok):
    return tok != "0" if ty == "bool" else int(tok)


def case_input(task, args):
    return {n: (list(v) if ty in judge.SLICES else (bool(v) if ty == "bool" else int(v)))
            for (n, ty), v in zip(task.PARAMS, args)}


def case_output(task, toks):
    """The tokens a driver printed for one case, as a kill-check output."""
    toks = list(toks)
    out = {}
    if task.RET != "()":
        out["r"] = value(task.RET, toks.pop(0))
    for n, ty in task.PARAMS:
        if ty == "&mut [i64]":
            k = int(toks.pop(0))
            out[n] = [int(x) for x in toks[:k]]
            del toks[:k]
    return out


def mutant_witnesses(task, tmp):
    """For each mutant: {"outputs": [{"input", "output"}], "trap": input or None}.
    `outputs` are the first hidden cases where it returns something the model
    does not; `trap` is the first case where it panics, crashes or hangs."""
    cs = judge.hidden_cases(task)
    crate = os.path.join(tmp, "mutants")
    judge.mutant_crate(task, crate)
    callees = [f"solution::m{i}::{task.NAME}" for i in range(len(task.RUST_MUTANTS))]
    exe, err = judge.build_rust_judge(task, crate, tmp, callees)
    if exe is None:
        raise RuntimeError(f"{task.ID}: mutants do not build: {err}")
    cfile = os.path.join(tmp, "cases.txt")
    judge.write_case_file(task, cs, cfile)
    out = []
    for i in range(len(callees)):
        got = judge.run_cases(lambda s, i=i: [exe, cfile, str(s), str(i)], len(cs))
        w = {"outputs": [], "trap": None}
        for k, args in enumerate(cs):
            g = got.get(k, ("not run",))
            if g[0] == "ok" and g[1] != judge.expected(task, args):
                if len(w["outputs"]) < MAX_WITNESSES:
                    w["outputs"].append({"input": case_input(task, args), "output": case_output(task, g[1])})
            elif g[0] in TRAPS and w["trap"] is None:
                w["trap"] = case_input(task, args)
        if not w["outputs"] and w["trap"] is None:
            raise RuntimeError(f"{task.ID}: mutant {i} passes every hidden case")
        out.append(w)
    return out


# ---------------------------------------------------------------- BMB arm


def admits_fn(name, uses, def_path, signature, inp):
    """Proved exactly when the contract's precondition admits `inp`."""
    attr = f"#[uses({', '.join(uses)})]\n" if uses else ""
    body = bsb.call_lines(def_path, signature, inp)
    return f"#[effects(pure)]\n{attr}fn {name}() -> () {{\n" + "".join(f"    {l}\n" for l in body) + "}\n"


def bmb_kills(task, sources, witnesses, tmp):
    """[killed] per mutant for the BMB sources of a session."""
    ws = os.path.join(tmp, "bmb")
    files = []
    for rel, text in sorted(sources.items()):
        p = os.path.join(ws, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w", encoding="utf-8") as f:
            f.write(text)
        files.append(p)
    bmb = harness.bmb_bin()
    l0 = subprocess.run([bmb, "map", "L0", *files], capture_output=True, text=True).stdout
    rows = judge.bmb_rows(l0)
    specs = sorted(r["path"] for r in rows if r.get("def_kind") == "spec fn")
    units = {r["path"] for r in rows if r.get("kind") == "unit"}
    unit = bsb.hidden_unit_name(units)
    def_path, sig = f"tasks::{task.NAME}", judge.sig_bmb(task)
    fns = []
    for i, w in enumerate(witnesses):
        if w["outputs"]:
            fns.append(bsb.kill_fn(f"k{i}", specs, def_path, sig, w["outputs"]))
        else:
            fns.append(admits_fn(f"k{i}", specs, def_path, sig, w["trap"]))
    kpath = os.path.join(tmp, "kill.bmb")
    with open(kpath, "w", encoding="utf-8") as f:
        f.write(f'#[summary("hidden mutant kill check")]\nmod {unit} {{\n' + "\n".join(fns) + "}\n")
    out = subprocess.run([bmb, "verify", *files, kpath], capture_output=True, text=True).stdout
    st = bsb.statuses(out, unit)
    return [bool(st.get(f"k{i}", False)) for i in range(len(witnesses))]


# ---------------------------------------------------------------- Rust arm


def with_mutant(lib, name, mutant):
    """The agent's `lib.rs` with its function `name` renamed away and the
    mutant in its place (in a module of its own, so its helpers clash with
    nothing)."""
    renamed = re.sub(rf"\bfn {re.escape(name)}\b(?=\s*[(<])", f"fn {name}__agent", lib)
    return (f"#![allow(warnings)]\n{renamed}\n"
            f"mod brb_mutant {{\n{mutant}\n}}\npub use brb_mutant::{name};\n")


def cargo_test(crate, target, build_only=False):
    env = {**os.environ, "CARGO_TARGET_DIR": target, "CARGO_TERM_COLOR": "never",
           "PROPTEST_RNG_SEED": PROPTEST_SEED}
    args = ["cargo", "test", "--offline", "--quiet"] + (["--no-run"] if build_only else [])
    try:
        return subprocess.run(args, cwd=crate, env=env, capture_output=True, text=True,
                              timeout=600).returncode
    except subprocess.TimeoutExpired:
        return 1


def rust_kills(task, sources, tmp, target):
    """[killed | None] per mutant (None: the crate does not build with it),
    or a string saying why the session is left out."""
    crate = os.path.join(tmp, "solution")
    harness.make_crate(crate, True)
    for rel, text in sources.items():
        p = os.path.join(crate, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w", encoding="utf-8") as f:
            f.write(text)
    lib_path = os.path.join(crate, "src", "lib.rs")
    with open(lib_path, encoding="utf-8") as f:
        lib = f.read()
    if cargo_test(crate, target) != 0:
        return "the session's tests fail under the fixed seed"
    out = []
    for m in task.RUST_MUTANTS:
        with open(lib_path, "w", encoding="utf-8") as f:
            f.write(with_mutant(lib, task.NAME, m))
        if cargo_test(crate, target, build_only=True) != 0:
            out.append(None)
        else:
            out.append(cargo_test(crate, target) != 0)
    return out


# ---------------------------------------------------------------- run


def judge_session(task, arm, sources, target):
    tmp = tempfile.mkdtemp(prefix="brb-oracle-")
    try:
        ws = mutant_witnesses(task, tmp)
        classes = ["output" if w["outputs"] else "trap" for w in ws]
        if arm == "bmb":
            killed = bmb_kills(task, sources, ws, tmp)
        else:
            killed = rust_kills(task, sources, tmp, target)
        if isinstance(killed, str):
            return {"excluded": killed}
        return {"killed": killed, "classes": classes}
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def load_rows(path):
    with open(path, encoding="utf-8") as f:
        return [r for r in (json.loads(l) for l in f if l.strip()) if "id" in r and "arm" in r]


def cmd_run(a):
    tasks = {t.ID: t for t in harness.tasks()}
    rows = load_rows(a.results)
    target = tempfile.mkdtemp(prefix="brb-oracle-target-")
    out = []
    try:
        if any(r["arm"] == "rust" for r in rows):
            harness.warm_rust(target, True)
        for r in rows:
            if a.only and r["id"] not in a.only:
                continue
            row = {"id": r["id"], "arm": r["arm"], "agent": r.get("agent"), "source": os.path.basename(a.results)}
            if not r.get("final_check_ok") or not r.get("final_sources"):
                row["excluded"] = "the session's own check did not pass at the end"
            else:
                row.update(judge_session(tasks[r["id"]], r["arm"], r["final_sources"], target))
            out.append(row)
            print(json.dumps(row, ensure_ascii=False), file=sys.stderr)
    finally:
        shutil.rmtree(target, ignore_errors=True)
    with open(a.out, "w", encoding="utf-8") as f:
        for row in out:
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    return 0


# ---------------------------------------------------------------- summary


def rate(row, only=None):
    """(killed, counted) of a row; `only` keeps one mutant class."""
    ks = [k for k, c in zip(row["killed"], row["classes"]) if k is not None and (only is None or c == only)]
    return sum(ks), len(ks)


def paired(bmb, rust, only=None):
    """Per-task rates of the tasks both arms have, as [(id, bmb, rust)]."""
    b = {r["id"]: r for r in bmb if "killed" in r}
    u = {r["id"]: r for r in rust if "killed" in r}
    out = []
    for tid in sorted(set(b) & set(u)):
        (kb, nb), (ku, nu) = rate(b[tid], only), rate(u[tid], only)
        if nb and nu:
            out.append((tid, kb / nb, ku / nu))
    return out


def sign_flip_p(diffs):
    """Exact two-sided p of |mean| under flipping the sign of each paired
    difference (the distribution of sums, by convolution)."""
    obs = abs(sum(diffs))
    dist = {0.0: 1}
    for d in diffs:
        nxt = {}
        for s, c in dist.items():
            for v in (round(s + d, 9), round(s - d, 9)):
                nxt[v] = nxt.get(v, 0) + c
        dist = nxt
    total = sum(dist.values())
    return sum(c for s, c in dist.items() if abs(s) >= obs - 1e-9) / total


def interval(diffs):
    """(mean, lo, hi): the t interval of the paired differences."""
    n = len(diffs)
    m = sum(diffs) / n
    if n < 2:
        return m, m, m
    sd = math.sqrt(sum((d - m) ** 2 for d in diffs) / (n - 1))
    h = T975[min(n - 1, 30)] * sd / math.sqrt(n)
    return m, m - h, m + h


def verdict(main, aux=None):
    """ADR-0119 decision 4, on the reference model's pairs (and the sign of
    the auxiliary model's difference)."""
    diffs = [b - u for _, b, u in main]
    if not diffs:
        return "판정 불가", "짝지은 과제가 없다"
    rust_mean = sum(u for _, _, u in main) / len(main)
    m, lo, hi = interval(diffs)
    p = sign_flip_p(diffs)
    if rust_mean >= 0.9:
        return "변별 불가", f"기준 모델의 Rust 팔 평균 살해율 {rust_mean:.3f} ≥ 0.9"
    if lo > 0 and p < 0.05:
        if aux is None:
            return "판정 불가", "보조 모델의 결과가 없다"
        am = sum(b - u for _, b, u in aux) / len(aux) if aux else 0.0
        if am > 0:
            return "성립", f"차이 {m:+.3f} [{lo:+.3f}, {hi:+.3f}], p = {p:.4f}, 보조 모델 차이 {am:+.3f}"
        return "판정 불가", f"기준 모델은 구간이 0 위이나 보조 모델 차이 {am:+.3f}의 부호가 다르다"
    if hi < 0 and p < 0.05:
        return "반증", f"차이 {m:+.3f} [{lo:+.3f}, {hi:+.3f}], p = {p:.4f}"
    return "판정 불가", f"차이 {m:+.3f} [{lo:+.3f}, {hi:+.3f}], p = {p:.4f}"


def describe(label, bmb, rust):
    lines = [f"### {label}", ""]
    for arm, rows in (("BMB", bmb), ("Rust", rust)):
        ex = [r for r in rows if "excluded" in r]
        unb = sum(k is None for r in rows if "killed" in r for k in r["killed"])
        lines.append(f"- {arm}: 세션 {len(rows)}, 대상 {len(rows) - len(ex)}, 제외 {len(ex)}"
                     + (f", 빌드되지 않아 뺀 변이체 {unb}" if unb else ""))
    lines += ["", "| 범위 | 짝지은 과제 | BMB 평균 살해율 | Rust 평균 살해율 | 차이(BMB − Rust) | 95% 구간 | p(순열) |",
              "| --- | --- | --- | --- | --- | --- | --- |"]
    for name, only in (("모든 변이체", None), ("출력이 다른 변이체만", "output"), ("패닉·넘침·멈춤만", "trap")):
        ps = paired(bmb, rust, only)
        if not ps:
            lines.append(f"| {name} | 0 | — | — | — | — | — |")
            continue
        diffs = [b - u for _, b, u in ps]
        m, lo, hi = interval(diffs)
        lines.append(f"| {name} | {len(ps)} | {sum(b for _, b, _ in ps) / len(ps):.3f} | "
                     f"{sum(u for _, _, u in ps) / len(ps):.3f} | {m:+.3f} | {lo:+.3f} ~ {hi:+.3f} | {sign_flip_p(diffs):.4f} |")
    kb, nb = map(sum, zip(*[rate(r) for r in bmb if "killed" in r])) if any("killed" in r for r in bmb) else (0, 0)
    ku, nu = map(sum, zip(*[rate(r) for r in rust if "killed" in r])) if any("killed" in r for r in rust) else (0, 0)
    lines += ["", f"합친 수(짝짓지 않음): BMB {kb}/{nb}, Rust {ku}/{nu}.", ""]
    return lines


def per_task(bmb, rust):
    b = {r["id"]: r for r in bmb}
    u = {r["id"]: r for r in rust}
    lines = ["| 과제 | 변이체 | BMB 죽임 | Rust 죽임 | 살아남은 변이체(BMB / Rust) |", "| --- | --- | --- | --- | --- |"]

    def cell(r):
        if r is None:
            return "—", ""
        if "excluded" in r:
            return "제외", ""
        k, n = rate(r)
        return f"{k}/{n}", ",".join(str(i) for i, x in enumerate(r["killed"]) if x is False)

    for tid in sorted(set(b) | set(u)):
        (cb, sb), (cu, su) = cell(b.get(tid)), cell(u.get(tid))
        n = len((b.get(tid) or u.get(tid)).get("killed", []))
        lines.append(f"| {tid} | {n or '—'} | {cb} | {cu} | {sb or '—'} / {su or '—'} |")
    return lines


def cmd_summarize(a):
    bmb, rust = load_rows(a.bmb), load_rows(a.rust)
    aux = (load_rows(a.aux[0]), load_rows(a.aux[1])) if a.aux else None
    out = describe("기준 모델", bmb, rust)
    if aux:
        out += describe("보조 모델", *aux)
    v, why = verdict(paired(bmb, rust), paired(*aux) if aux else None)
    out += [f"**판정(ADR-0119 결정 4): {v}.** {why}", "", "### 과제별(기준 모델)", ""] + per_task(bmb, rust)
    if aux:
        out += ["", "### 과제별(보조 모델)", ""] + per_task(*aux)
    print("\n".join(out))
    return 0


# ---------------------------------------------------------------- gate


REF = os.path.join(HERE, "oracle_strength_ref.json")


def cmd_check(a):
    """The reference and weak solutions (ADR-0119 decision 7). The reference
    contracts are not complete specifications (many state no `ensures`), so
    what each kills is recorded (`check --record`) and compared here: a
    change in the tool, the prover or the hidden cases shows as a difference.
    Controls: a weak BMB solution that states no contract kills no mutant
    that differs by output; a weak Rust solution's tests do not kill a mutant
    that is that weak solution."""
    bad = []
    target = tempfile.mkdtemp(prefix="brb-oracle-target-")
    try:
        tasks = [t for t in harness.tasks() if not a.only or t.ID in a.only]
        rust_done = 0
        got_ref = {}
        for t in tasks:
            def unit(body):
                return {"src/tasks.bmb": f'#[summary("{t.ID}")]\nmod tasks {{\n{body}\n}}\n'}

            ref = judge_session(t, "bmb", unit(t.BMB_REF), target)
            got_ref[t.ID] = {"killed": ref["killed"], "classes": ref["classes"]}
            if "ensures" not in t.BMB_WEAK:
                weak = judge_session(t, "bmb", unit(t.BMB_WEAK), target)
                got = [i for i, (k, c) in enumerate(zip(weak["killed"], weak["classes"])) if k and c == "output"]
                if got:
                    bad.append(f"{t.ID}: a weak solution without a contract kills output mutants {got}")
            same = [i for i, m in enumerate(t.RUST_MUTANTS)
                    if " ".join(m.split()) == " ".join(t.RUST_WEAK.split("#[cfg(test)]")[0].split())]
            if same and rust_done < a.rust_tasks:
                if rust_done == 0:
                    harness.warm_rust(target, True)
                rust_done += 1
                r = judge_session(t, "rust", {"src/lib.rs": t.RUST_WEAK}, target)
                if isinstance(r.get("killed"), list) and any(r["killed"][i] for i in same):
                    bad.append(f"{t.ID}: the weak solution's tests kill the mutant that is the weak solution")
        if a.record:
            with open(REF, "w", encoding="utf-8") as f:
                json.dump(got_ref, f, indent=1, sort_keys=True)
                f.write("\n")
        else:
            with open(REF, encoding="utf-8") as f:
                want = json.load(f)
            for tid, g in got_ref.items():
                if want.get(tid) != g:
                    bad.append(f"{tid}: the reference contract kills {g['killed']}, recorded {want.get(tid, {}).get('killed')}")
        for b in bad:
            print(f"oracle_strength: {b}", file=sys.stderr)
        print(f"ok: {len(tasks)} tasks, rust weak controls {rust_done}" if not bad else f"{len(bad)} problems")
        return 1 if bad else 0
    finally:
        shutil.rmtree(target, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("results")
    r.add_argument("--out", required=True)
    r.add_argument("--only", nargs="*")
    s = sub.add_parser("summarize")
    s.add_argument("bmb")
    s.add_argument("rust")
    s.add_argument("--aux", nargs=2)
    c = sub.add_parser("check")
    c.add_argument("--only", nargs="*")
    c.add_argument("--rust-tasks", type=int, default=3)
    c.add_argument("--record", action="store_true")
    a = ap.parse_args()
    return {"run": cmd_run, "summarize": cmd_summarize, "check": cmd_check}[a.cmd](a)


if __name__ == "__main__":
    sys.exit(main())
