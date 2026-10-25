"""Rust tests of the BCB twin (backlog 2A-14b, ADR-0042 §4).

One `#[test]` per executable definition, in a `#[cfg(test)] mod tests` at the
end of its unit file. A test calls the function on fixed inputs that satisfy
its `requires`; the verdict is the twin's own contract checks (`debug_assert!`
on entry and before the result) and Rust's overflow and bounds panics, so a
test checks what the BMB contract states, no more. Inputs are a deterministic
candidate list per kind: boundary values (the bound B, 0, ±1, the
definition's constant ±1, i64 extremes where nothing restricts them) and a
few values from a generator seeded by the definition's name. `kills.py`
measures that the tests kill every mutant the BMB contract kills.
"""

import random

B = 1000000
MIN, MAX = "i64::MIN", "i64::MAX"


def lit(v):
    return v if isinstance(v, str) else str(v)


def arr(xs):
    return "&[" + ", ".join(lit(x) for x in xs) + "]"


def dedupe(xs):
    seen, out = set(), []
    for x in xs:
        key = lit(x) if not isinstance(x, (list, tuple)) else repr(x)
        if key not in seen:
            seen.add(key)
            out.append(x)
    return out


def scalars(rng, c=None):
    xs = [-B, -B + 1, -1, 0, 1, B - 1, B]
    if c is not None:
        xs += [-c - 1, -c, -c + 1, c - 1, c, c + 1]
    xs += [rng.randint(-B, B) for _ in range(6)]
    return dedupe([x for x in xs if -B <= x <= B])


def inb_arrays(rng, c=0):
    out = [[], [0], [-B], [B], [B, -B, 0], [c, -c, c + 1]]
    out += [[rng.randint(-B, B) for _ in range(rng.randint(1, 5))] for _ in range(3)]
    return dedupe(out)


def any_arrays(rng, c=0):
    out = [[], [0], [c], [c, c + 1, c - 1], [B, B, B], [-B, 0, B], [MAX, MIN, 0],
           [B + 1, -B - 1, B + 1, -B - 1]]
    out += [[rng.randint(-2 * B, 2 * B) for _ in range(rng.randint(2, 5))] for _ in range(3)]
    return dedupe(out)


POOL = [MIN, -B - 1, -B, -B + 1, -1, 0, 1, B - 1, B, B + 1, MAX]


def strong_cases(name, kind, info):
    """The oracle's inputs (kills.py): many more, to decide whether a mutant
    really breaks the contract on some valid input. Not part of the twin."""
    rng = random.Random(f"bcb-oracle:{name}")
    c = info.get("c")
    pool = POOL + ([c - 1, c, c + 1, -c - 1, -c, -c + 1] if c is not None else [])
    inb_pool = [x for x in pool if not isinstance(x, str) and -B <= x <= B]

    def arrays(vals, n):
        out = [[]] + [[v] for v in vals]
        out += [[rng.choice(vals) for _ in range(rng.randint(2, 6))] for _ in range(n)]
        out += [[rng.randint(-B, B) for _ in range(rng.randint(1, 6))] for _ in range(n // 2)]
        return dedupe(out)

    if kind in ("leaf", "chain", "show"):
        return [str(x) for x in dedupe(inb_pool + [rng.randint(-B, B) for _ in range(300)])]
    if kind == "scan_count":
        return [f"({arr(a)}, {lit(rng.choice(pool + [rng.randint(-B, B)]))})" for a in arrays(pool, 300)]
    if kind in ("scan_peak", "remap"):
        return [arr(a) for a in arrays(inb_pool, 300)]
    if kind in ("fill", "reset"):
        return [f"({arr(a)}, {rng.choice(inb_pool)})" for a in arrays(pool, 300)]
    raise ValueError(kind)


def cases(name, kind, info):
    """Rust expressions of the test inputs (one per call)."""
    rng = random.Random(f"bcb-tests:{name}")
    c = info.get("c")
    if kind in ("leaf", "chain", "show"):
        return [f"{x}" for x in scalars(rng, c if kind == "leaf" else None)]
    if kind == "scan_count":
        ts = [MIN, -1, 0, c, MAX]
        return [f"({arr(a)}, {lit(t)})" for a in any_arrays(rng, c) for t in ts]
    if kind in ("scan_peak", "remap"):
        return [arr(a) for a in inb_arrays(rng, c or 0)]
    if kind in ("fill", "reset"):
        vs = dedupe([-B, 0, B, rng.randint(-B, B)])
        return [f"({arr(a)}, {v})" for a in any_arrays(rng) for v in vs]
    raise ValueError(kind)


def test_fn(name, kind, info, target=None, strong=False):
    """One test of definition `name` (of `target`, a mutant copy, when given);
    `strong` is the oracle's test (prefix `s_` instead of `t_`)."""
    target = target or name
    cs = (strong_cases if strong else cases)(name, kind, info)
    rows = "".join(f"            {x},\n" for x in cs)
    if kind in ("leaf", "chain", "show"):
        decl, call = "&[i64]", f"for &x in cases {{\n            {target}(x);\n        }}"
    elif kind == "scan_count":
        decl, call = "&[(&[i64], i64)]", f"for &(a, t) in cases {{\n            {target}(a, t);\n        }}"
    elif kind == "scan_peak":
        decl, call = "&[&[i64]]", f"for &a in cases {{\n            {target}(a);\n        }}"
    elif kind == "remap":
        decl, call = "&[&[i64]]", (f"for &a in cases {{\n            let mut a = a.to_vec();\n"
                                   f"            {target}(&mut a);\n        }}")
    else:  # fill, reset
        decl, call = "&[(&[i64], i64)]", (f"for &(a, v) in cases {{\n            let mut a = a.to_vec();\n"
                                          f"            {target}(&mut a, v);\n        }}")
    return (f"    #[test]\n    fn {'s' if strong else 't'}_{target}() {{\n        let cases: {decl} = &[\n{rows}        ];\n"
            f"        {call}\n    }}\n")


EXEC = ("leaf", "chain", "scan_count", "scan_peak", "fill", "remap", "reset", "show")


def exec_defs(unit):
    return sorted((n, k, info) for n, k, info in unit.defs if k in EXEC)


def tests_block(unit, extra=()):
    """The unit file's test module; `extra` adds (name, kind, info, target,
    strong) tests."""
    fns = [test_fn(n, k, info) for n, k, info in exec_defs(unit)]
    fns += [test_fn(n, k, info, target, strong) for n, k, info, target, strong in extra]
    return "#[cfg(test)]\nmod tests {\n    use super::*;\n\n" + "\n".join(fns) + "}\n"
