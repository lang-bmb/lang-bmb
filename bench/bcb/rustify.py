"""BMB executable definition -> Rust twin definition, for the generated subset
(backlog 2A-14b, ADR-0042 §4).

The Rust twin (bench/bnb/twin.py) renders from the plan, so it cannot render a
mutated body. This translator takes the BMB text of one executable definition
of a bench/m3 workspace (the normal form gen.py writes and `bmb mutate`
returns) and writes the twin's text for it, following twin.py's mapping.
Mutants are translated the same way, so the two languages get the same
mutation at the same place. `check_unit` confirms that translating every
original definition of a unit gives the twin's text byte for byte; callers
use it before trusting a translation.
"""

import re

ZERO_MINUS = re.compile(r"(?<![\w)\]])0 - (\w+)")
COMPOUND = re.compile(r"^(\w+) = \1 ([+-]) (.+);$")
LET = re.compile(r"^let (mut )?(\w+): [\w\[\]; &]+ = (.+)$")


def expr(e):
    """`0 - x` -> `-x` (twin.py writes negation), `print_i64(e)` -> `println!`."""
    e = ZERO_MINUS.sub(r"-\1", e)
    if e.startswith("print_i64(") and e.endswith(");"):
        return 'println!("{}", ' + e[len("print_i64("):-2] + ");"
    return e


def split_top(s):
    """Top-level comma split of a contract list (`a, f(x, r)` -> two items)."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def blocks(unit_text):
    """Top-level blocks of a unit's body (definitions with their attributes)."""
    body = unit_text.split("{", 1)[1].rsplit("}", 1)[0]
    return [b for b in body.strip("\n").split("\n\n") if b.strip()]


def def_name(block):
    for line in block.splitlines():
        s = line.strip()
        if s.startswith("fn ") or s.startswith("spec fn "):
            return s.split("fn ", 1)[1].split("(")[0]
    raise ValueError("no fn in block")


def is_exec(block):
    return any(line.strip().startswith("fn ") for line in block.splitlines())


def translate(block):
    """Rust twin text of one executable BMB definition block."""
    lines = [x[4:] if x.startswith("    ") else x for x in block.splitlines()]
    i = 0
    while not lines[i].startswith("fn "):
        i += 1
    sig = lines[i]
    name, rest = sig[3:].split("(", 1)
    params, ret = rest.rsplit(") -> ", 1)
    m = re.match(r"\((\w+): (.+)\)$", ret)
    rtype = m.group(2) if m else ret
    i += 1
    requires, ensures = [], []
    while lines[i].strip() != "{":
        s = lines[i].strip().rstrip(",")
        if s.startswith("requires "):
            requires += split_top(s[len("requires "):])
        elif s.startswith("ensures "):
            ensures += split_top(s[len("ensures "):])
        else:
            raise ValueError(f"unexpected header line {lines[i]!r}")
        i += 1
    body = [x[4:] for x in lines[i + 1:-1]]  # inside the braces, at column 0
    doc = []
    if requires:
        doc.append("/// Requires: " + ", ".join(f"`{c}`" for c in requires) + ".")
    if ensures:
        doc.append("/// Ensures: " + ", ".join(f"`{c}`" for c in ensures) + ".")
    head = f"pub fn {name}({params}) -> {rtype} {{" if rtype != "()" else f"pub fn {name}({params}) {{"
    out = []
    if requires:
        out.append("debug_assert!(" + " && ".join(requires) + ");")
    # The tail expression: the last line if it is not a statement, or a
    # trailing `if .. { } else { }` block that starts at column 0.
    tail = []
    if rtype != "()":
        j = len(body) - 1
        if body[j] == "}":
            while not body[j].startswith("if "):
                j -= 1
        tail, body = body[j:], body[:j]
    k = 0
    while k < len(body):
        s = body[k]
        ind = len(s) - len(s.lstrip())
        t = s.strip()
        pad = " " * ind
        if t.startswith("while ") and not t.endswith("{"):
            cond = t
            k += 1
            inv = None
            while body[k].strip() != "{":
                u = body[k].strip().rstrip(",")
                if u.startswith("invariant "):
                    inv = u[len("invariant "):]
                k += 1
            if inv is not None:
                out.append(f"{pad}// invariant: {inv}")
            out.append(f"{pad}{expr(cond)} {{")
        elif LET.match(t):
            mm = LET.match(t)
            out.append(f"{pad}let {mm.group(1) or ''}{mm.group(2)} = {expr(mm.group(3))}")
        elif COMPOUND.match(t):
            mm = COMPOUND.match(t)
            out.append(f"{pad}{mm.group(1)} {mm.group(2)}= {expr(mm.group(3))};")
        else:
            out.append(pad + expr(t))
        k += 1
    if tail:
        tail = [expr(x) for x in tail]
        tail[0] = "let r = " + tail[0]
        tail[-1] += ";"
        out += tail
    if ensures:
        out.append("debug_assert!(" + " && ".join(ensures) + ");")
    if tail:
        out.append("r")
    return "\n".join(doc + [head] + ["    " + x for x in out] + ["}"]) + "\n"


def rust_blocks(rust_text):
    """name -> block (doc lines included) of a twin unit file."""
    out = {}
    for b in rust_text.split("\n\n"):
        for line in b.splitlines():
            if line.startswith("pub fn "):
                out[line[len("pub fn "):].split("(")[0]] = b.rstrip("\n") + "\n"
    return out


def check_unit(bmb_text, rust_text):
    """Names of executable definitions whose translation differs from the twin."""
    rb = rust_blocks(rust_text)
    return [def_name(b) for b in blocks(bmb_text) if is_exec(b) and translate(b) != rb.get(def_name(b))]


def replace_rust(rust_text, name, new_block):
    """The twin unit file with definition `name` replaced by `new_block`."""
    old = rust_blocks(rust_text)[name]
    assert rust_text.count(old) == 1, name
    return rust_text.replace(old, new_block)
