#!/usr/bin/env python3
"""Reference for the port-shape experiment: one expression per input line,
its normal form (or `error`) per output line.

  expr  := term (('+' | '-') term)*
  term  := unary ('*' unary)*
  unary := '-' unary | atom
  atom  := digit+ | name | name '(' [expr (',' expr)*] ')' | '(' expr ')'
  name  := [a-z]+

Spaces between tokens are skipped. Nesting deeper than DEPTH is an error:
the depth is 1 at the top `expr` and grows by one at each `-` operand, each
parenthesised expression and each call argument. Anything else on the line
after the expression is an error; an empty line is an error.

Normal form: a number and a name as written, `(a+b)` `(a-b)` `(a*b)`,
`(-a)`, `f(a,b)`; no spaces.

  ref.py < input           the reference output
  ref.py gen SEED N        N random lines (valid and broken)
"""
import random
import sys

DEPTH = 64


class Bad(Exception):
    pass


def parse(t):
    i = 0

    def skip():
        nonlocal i
        while i < len(t) and t[i] == " ":
            i += 1

    def expr(d):
        nonlocal i
        if d > DEPTH:
            raise Bad
        a = term(d)
        while True:
            skip()
            if i < len(t) and t[i] in "+-":
                op = t[i]
                i += 1
                a = f"({a}{op}{term(d)})"
            else:
                return a

    def term(d):
        nonlocal i
        a = unary(d)
        while True:
            skip()
            if i < len(t) and t[i] == "*":
                i += 1
                a = f"({a}*{unary(d)})"
            else:
                return a

    def unary(d):
        nonlocal i
        skip()
        if i < len(t) and t[i] == "-":
            i += 1
            if d + 1 > DEPTH:
                raise Bad
            return f"(-{unary(d + 1)})"
        return atom(d)

    def atom(d):
        nonlocal i
        skip()
        j = i
        if i < len(t) and t[i].isdigit():
            while i < len(t) and t[i] in "0123456789":
                i += 1
            return t[j:i]
        if i < len(t) and "a" <= t[i] <= "z":
            while i < len(t) and "a" <= t[i] <= "z":
                i += 1
            name = t[j:i]
            skip()
            if i < len(t) and t[i] == "(":
                i += 1
                args = []
                skip()
                if i < len(t) and t[i] == ")":
                    i += 1
                    return f"{name}()"
                while True:
                    args.append(expr(d + 1))
                    skip()
                    if i < len(t) and t[i] == ",":
                        i += 1
                    elif i < len(t) and t[i] == ")":
                        i += 1
                        return f"{name}({','.join(args)})"
                    else:
                        raise Bad
            return name
        if i < len(t) and t[i] == "(":
            i += 1
            a = expr(d + 1)
            skip()
            if i < len(t) and t[i] == ")":
                i += 1
                return a
            raise Bad
        raise Bad

    try:
        a = expr(1)
        skip()
        if i != len(t):
            raise Bad
        return a
    except Bad:
        return "error"


def gen(rng, d):
    r = rng.random()
    if d > 5 or r < 0.25:
        return str(rng.randrange(100000)) if rng.random() < 0.5 else rng.choice(["x", "y", "foo", "g"])
    if r < 0.55:
        return gen(rng, d + 1) + rng.choice([" + ", "-", "*", " * "]) + gen(rng, d + 1)
    if r < 0.65:
        return "-" + gen(rng, d + 1)
    if r < 0.8:
        return "(" + gen(rng, d + 1) + ")"
    return rng.choice(["f", "max"]) + "(" + ", ".join(gen(rng, d + 1) for _ in range(rng.randrange(4))) + ")"


def lines(seed, n):
    rng = random.Random(seed)
    out = []
    for _ in range(n):
        s = gen(rng, 0)
        if rng.random() < 0.2 and s:  # break it
            k = rng.randrange(len(s))
            s = s[:k] + rng.choice(["", "(", ")", ",", "+", "A", "  "]) + s[k + 1:]
        out.append(s)
    out += ["", " ", "(" * 63 + "1" + ")" * 63, "(" * 64 + "1" + ")" * 64, "-" * 63 + "1", "-" * 64 + "1", "f()", "f(,)", "1 2"]
    return out


if __name__ == "__main__":
    if sys.argv[1:2] == ["gen"]:
        print("\n".join(lines(int(sys.argv[2]), int(sys.argv[3]))))
    else:
        for line in sys.stdin.read().split("\n")[:-1]:
            print(parse(line))
