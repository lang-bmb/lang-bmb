"""What a reviewer is shown (frozen with the sets that list this file in
their MANIFEST). Standard library only; the `weaken` part runs the compiler.

An arm of a set (`set.json`, `arms`) is an ordered list of parts. Each part
is a text made from one change (`changes/<id>.json`), and the view is the
parts in order, each under its title. The hidden label is never read here.

  contract_diff   unified diff of the contract text before -> after
  weaken          `bmb weaken before after`, stdout as the tool prints it
  code_diff       unified diff of the program file before -> after
  contract        the contract text after the change
  code            the program file after the change
  field:NAME      the change's own frozen text `NAME` (e.g. generated examples)

The contract text of a program is, in source order, each function's signature
line with its `requires` and `ensures` lines, and each `spec fn` those lines
name, directly or through another such `spec fn` (whole definition). Bodies,
attributes, `decreases`, loop invariants and `spec fn`s only the proof uses
are not contract text. `spec fn`s of the seed (`given::..`) are in neither
diff: the seed does not change, and a task that gives them shows them in the
intent.

Sizes are bytes of the view and ceil(bytes / 4) tokens (ADR-0021 §4).
"""
import difflib
import json
import os
import re
import shutil
import subprocess
import tempfile

TITLES = {
    "contract_diff": "Contract diff (signatures, requires/ensures, and the spec fns they name)",
    "weaken": "Output of `bmb weaken before after`",
    "code_diff": "Code diff",
    "contract": "Contract (signatures, requires/ensures, and the spec fns they name)",
    "code": "Code",
}
NONE = "(no difference)\n"


def load_set(set_dir):
    with open(os.path.join(set_dir, "set.json")) as f:
        return json.load(f)


def load_changes(set_dir):
    d = os.path.join(set_dir, "changes")
    out = []
    for name in sorted(os.listdir(d)):
        if name.endswith(".json"):
            with open(os.path.join(d, name)) as f:
                out.append(json.load(f))
    return out


# ---------- contract text ----------

def definitions(text):
    """The top-level definitions of a one-unit program as lists of lines
    (dedented), in source order. A definition starts at a line of the unit's
    indentation that opens an attribute or a function and ends at its closing
    brace: the end of a one-line `spec fn`, or a `}` line of that indentation."""
    lines = text.split("\n")
    start = next(i for i, l in enumerate(lines) if re.match(r"mod \w+ \{$", l))
    end = max(i for i, l in enumerate(lines) if l == "}")
    defs, cur = [], None
    for raw in lines[start + 1:end]:
        if raw.strip() and not raw.startswith("    "):
            raise ValueError(f"line outside the unit's indentation: {raw!r}")
        line = raw[4:]
        if cur is None:
            if not line:
                continue
            if not re.match(r"(#\[|spec fn |proof fn |fn )", line):
                raise ValueError(f"not the start of a definition: {line!r}")
            cur = [line]
        else:
            cur.append(line)
        code = [l for l in cur if not l.startswith("#[")]
        if not code:
            continue
        one_line = len(code) == 1 and line.rstrip().endswith("}")
        if one_line or line == "}":
            defs.append(cur)
            cur = None
    if cur is not None:
        raise ValueError("unterminated definition")
    return defs


def _name(lines):
    head = next(l for l in lines if not l.startswith("#["))
    m = re.match(r"(spec fn|proof fn|fn) (\w+)", head)
    return m.group(1), m.group(2)


def _contract_lines(lines):
    """Signature, `requires` and `ensures` lines of a function definition."""
    body = [l for l in lines if not l.startswith("#[")]
    out = [body[0]]
    for l in body[1:]:
        if l == "{":
            break
        word = l.strip().split(" ")[0]
        if not l.startswith("    ") or l.startswith("     ") or word not in ("requires", "ensures", "decreases"):
            raise ValueError(f"not a one-line contract clause: {l!r}")
        if word != "decreases":
            out.append(l)
    return out


def contract_text(text):
    defs = definitions(text)
    kinds = [(_name(d), d) for d in defs]
    specs = {name: d for (kind, name), d in kinds if kind == "spec fn"}
    funcs = {name: _contract_lines(d) for (kind, name), d in kinds if kind != "spec fn"}

    def named(lines):
        # A call of a `spec fn` of this unit: its name, not after `::`.
        found = set()
        for l in lines:
            for m in re.finditer(r"(?<![\w:])(\w+)\s*\(", l):
                if m.group(1) in specs:
                    found.add(m.group(1))
        return found

    shown, todo = set(), set()
    for lines in funcs.values():
        todo |= named(lines[1:])
    while todo:
        n = todo.pop()
        if n not in shown:
            shown.add(n)
            # The definition's own head names itself; that adds nothing new.
            todo |= named(specs[n]) - shown
    out = []
    for (kind, name), d in kinds:
        if kind != "spec fn":
            out.append("\n".join(funcs[name]))
        elif name in shown:
            out.append("\n".join(d))
    return "\n\n".join(out) + "\n"


# ---------- parts ----------

def unified(a, b, name):
    out = list(difflib.unified_diff(a.splitlines(), b.splitlines(), f"a/{name}", f"b/{name}", lineterm=""))
    return "\n".join(out) + "\n" if out else NONE


def workspace(set_dir, change, which, root):
    """Workspace `root`: the change's seed and its file as `which` has it."""
    src = os.path.join(root, "src")
    os.makedirs(src)
    if change.get("seed"):
        seed = os.path.join(set_dir, change["seed"], "src")
        for f in sorted(os.listdir(seed)):
            shutil.copy(os.path.join(seed, f), os.path.join(src, f))
    path = os.path.join(root, change["file"])
    with open(path, "w") as f:
        f.write(change[which])
    return root


def weaken(set_dir, change, bmb):
    """stdout of `bmb weaken before after`, run where the two workspaces are
    `before` and `after`, so the output names no temporary path. The tool
    exits 1 when a contract got weaker or a comparison was not decided; that
    is an answer. No rows at all is not."""
    tmp = tempfile.mkdtemp(prefix="bmb-review-")
    try:
        for which in ("before", "after"):
            workspace(set_dir, change, which, os.path.join(tmp, which))
        r = subprocess.run([bmb, "weaken", "before", "after"], cwd=tmp, capture_output=True, text=True)
        if r.returncode not in (0, 1) or '"summary":true' not in r.stdout:
            raise RuntimeError(f"{change['id']}: bmb weaken failed ({r.returncode}): {r.stdout}{r.stderr}")
        return r.stdout
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def part(name, set_dir, change, bmb):
    if name == "contract_diff":
        return unified(contract_text(change["before"]), contract_text(change["after"]), "contract")
    if name == "weaken":
        return weaken(set_dir, change, bmb)
    if name == "code_diff":
        return unified(change["before"], change["after"], change["file"])
    if name == "contract":
        return contract_text(change["after"])
    if name == "code":
        return change["after"]
    if name.startswith("field:"):
        return change[name[6:]]
    raise ValueError(f"unknown view part {name}")


def title(name, cfg):
    if name.startswith("field:"):
        return cfg["titles"][name]
    return TITLES[name]


def join(cfg, names, texts):
    """The parts in order, each under its title."""
    out = []
    for name, text in zip(names, texts):
        out.append(f"## {title(name, cfg)}\n{text}" + ("" if text.endswith("\n") else "\n"))
    return "\n".join(out)


def view(set_dir, cfg, change, arm, bmb):
    """The text arm `arm` shows of `change`."""
    names = cfg["arms"][arm]
    return join(cfg, names, [part(n, set_dir, change, bmb) for n in names])


def size(text):
    n = len(text.encode())
    return n, -(-n // 4)


def message(change, view_text):
    """The reviewer's user message: the intent, then the view."""
    return f"## Intent\n{change['intent']}\n\n{view_text}"
