"""BSB: specification mutant kill rate (ADR-0034 G3, backlog 2A-10).

A task set may carry hidden mutant sets in `<set>/mutants/<task-id>.json`
(made by `bsb_gen.py`, checked by `<set>/mutants/MANIFEST`). A mutant is a
single W0904 change (`bmb mutate`) to the reference solution: to the target
or to a function it calls. It is kept only with witnesses: inputs that the
task's hidden precondition allows on which the mutant terminates with an
output that differs from the reference's and that the hidden reference
contract rejects (so the mutant is a bug, not an equivalent or allowed
variant).

A contract kills a mutant when `bmb verify` proves, from the contract alone
(the function is seen only through its `requires`/`ensures`, as every caller
sees it, ADR-0017 §4), that the witness inputs are allowed and that the
function cannot return the mutant's outputs on all of them:

    fn k0_3() -> () {
        let mut a_0: [i64; 3] = [1, 5, 2];          // witness 1
        let bsb_r0: bool = tasks::f(&mut a_0, 4);   // precondition proved here
        let mut a_1: [i64; 1] = [7];                // witness 2
        let bsb_r1: bool = tasks::f(&mut a_1, 0);
        assert(!((bsb_r0 && a_0[0] == 2 && ..) && (!bsb_r1 && a_1[0] == 7)));
    }

The calls are independent (each result is only what the contract allows), so
the assertion holds exactly when the contract rules out the mutant's output on
at least one witness. A contract whose precondition rejects a witness input
(stronger than the hidden `ref_pre`, which also fails the spec-strength check)
does not kill that mutant. One kill function per mutant, each judged by its
own status line, all in one `bmb verify` run per task. The kill rate is
killed / kept mutants. Standard library only.
"""
import hashlib
import json
import os
import re
import tempfile

HIDDEN = "bmb_bsb"


def mutants_dir(set_dir):
    return os.path.join(set_dir, "mutants")


def manifest_text(set_dir):
    d = mutants_dir(set_dir)
    lines = []
    for f in sorted(os.listdir(d)):
        if f.endswith(".json"):
            with open(os.path.join(d, f), "rb") as fh:
                lines.append(f"{hashlib.sha256(fh.read()).hexdigest()}  {f}")
    return "\n".join(lines) + "\n"


def check_manifest(set_dir):
    """True when the set has mutant files and they match their MANIFEST;
    False when it has none. Exits on a mismatch (frozen like the tasks)."""
    d = mutants_dir(set_dir)
    if not os.path.isdir(d):
        return False
    path = os.path.join(d, "MANIFEST")
    if not os.path.exists(path) or open(path).read() != manifest_text(set_dir):
        raise SystemExit("mutant files differ from mutants/MANIFEST: refusing to run (ADR-0021 §1)")
    return True


def load(set_dir, task_id):
    p = os.path.join(mutants_dir(set_dir), task_id + ".json")
    if not os.path.exists(p):
        return None
    with open(p) as f:
        return json.load(f)


# ---------- signatures and values ----------

def params_of(signature):
    """[(name, type)] and the result type of `fn f(a: T, ..) -> (r: R)`."""
    m = re.match(r"fn \w+\((.*)\) -> (.*)$", signature.strip())
    params = []
    for p in m.group(1).split(", ") if m.group(1) else []:
        name, ty = p.split(": ", 1)
        params.append((name, ty))
    ret = m.group(2)
    m2 = re.match(r"\((\w+): (.*)\)$", ret)
    return params, (m2.group(2) if m2 else ret)


def lit(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    return str(v)


def call_lines(def_path, signature, inp, result="bsb_r", suffix=""):
    """Statements that bind the input `inp` and call `def_path` on it (array
    locals are the parameter names plus `suffix`)."""
    params, ret = params_of(signature)
    lines, args = [], []
    for name, ty in params:
        if ty in ("&[i64]", "&mut [i64]"):
            mut = "mut " if ty.startswith("&mut") else ""
            vals = inp[name]
            lines.append(f"let {mut}{name}{suffix}: [i64; {len(vals)}] = [{', '.join(lit(x) for x in vals)}];")
            args.append(("&mut " if mut else "&") + name + suffix)
        else:
            args.append(lit(inp[name]))
    call = f"{def_path}({', '.join(args)})"
    if ret == "()":
        lines.append(call + ";")
    else:
        lines.append(f"let {result}: {ret} = {call};")
    return lines


def output_terms(signature, out, result="bsb_r", suffix=""):
    """Conjuncts stating that the observed output equals `out`."""
    params, ret = params_of(signature)
    terms = []
    if ret != "()":
        v = out["r"]
        if ret == "bool":
            terms.append(result if v else f"!{result}")
        else:
            terms.append(f"{result} == {lit(v)}")
    for name, ty in params:
        if ty == "&mut [i64]":
            terms += [f"{name}{suffix}[{k}] == {lit(x)}" for k, x in enumerate(out[name])]
    return terms


def kill_fn(name, uses, def_path, signature, witnesses):
    """The kill check of one mutant (see the module comment)."""
    body, conj = [], []
    one = len(witnesses) == 1
    for j, w in enumerate(witnesses):
        res, sfx = ("bsb_r", "") if one else (f"bsb_r{j}", f"_{j}")
        body += call_lines(def_path, signature, w["input"], result=res, suffix=sfx)
        terms = " && ".join(output_terms(signature, w["output"], result=res, suffix=sfx))
        conj.append(terms if one else f"({terms})")
    body.append(f"assert(!({' && '.join(conj)}));")
    attr = f"#[uses({', '.join(uses)})]\n" if uses else ""
    return (f"#[effects(pure)]\n{attr}fn {name}() -> () {{\n"
            + "".join(f"    {l}\n" for l in body) + "}\n")


def statuses(verify_out, unit):
    st = {}
    for line in verify_out.splitlines():
        line = line.strip()
        if line.startswith("{"):
            r = json.loads(line)
            if "status" in r and r.get("def", "").startswith(unit + "::"):
                st[r["def"][len(unit) + 2:]] = r["status"] == "verified"
    return st


def hidden_unit_name(taken):
    name = HIDDEN
    while name in taken:
        name += "_"
    return name


def kill_check(targets, uses, taken_units, verify, prelude=""):
    """Kill verdicts for every target's mutants against the contracts that
    `verify` sees.

    targets: [(def path, signature, mutants)] — the function each kill check
    calls, its signature and its kept mutants (with witnesses).
    uses: the `#[uses]` list of every kill function (the predicates that
    reveal the contract). `prelude` is more text for the hidden unit (the
    generator's stub with the hidden reference contract). taken_units: unit
    names the hidden unit must avoid. verify(extra_file) runs `bmb verify`
    on the program plus that file and returns its stdout.
    Returns [[killed: bool per mutant] per target]."""
    unit = hidden_unit_name(taken_units)
    fns = [kill_fn(f"k{t}_{i}", uses, def_path, sig, m["witnesses"])
           for t, (def_path, sig, mutants) in enumerate(targets) for i, m in enumerate(mutants)]
    if not fns:
        return [[] for _ in targets]
    text = (f'#[summary("hidden mutant kill check")]\nmod {unit} {{\n' + prelude
            + "\n".join(fns) + "}\n")
    with tempfile.NamedTemporaryFile("w", suffix=".bmb", delete=False) as f:
        f.write(text)
    try:
        st = statuses(verify(f.name), unit)
    finally:
        os.unlink(f.name)
    return [[st.get(f"k{t}_{i}", False) for i in range(len(mutants))]
            for t, (_, _, mutants) in enumerate(targets)]


def rate_text(killed, total):
    return f"{killed}/{total} ({100.0 * killed / total:.1f}%)" if total else "—"
