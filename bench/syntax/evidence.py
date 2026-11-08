#!/usr/bin/env python3
"""Measured evidence per construct from the recorded model runs (backlog 2R-03).

Deterministic, offline: reads the session logs the frozen harnesses keep
(`bench/*/results/**/*.sessions/*.json.gz`, the official path of ADR-0066),
no model runs. A session is one task attempt: the agent's tool calls with
the tool results. Every diagnostic the tools returned to the agent of a BMB
arm (sessions whose directory name ends in `-rust` or `-verus` are the other
arms and are left out) is counted by code:

  hits        diagnostics with that code, over all tool results
  sessions    sessions where the code appeared at least once
  first       sessions where it appeared in the first tool result that held
              any diagnostic (the agent's first attempt)
  unresolved  sessions where it appeared in the last such result (the
              attempt the agent ended on)

The codes are grouped by construct with the table CONSTRUCTS (the E02xx
Rust habits by the habit, the others by their family), and each row has
the share of sessions and the tokens of those sessions. One JSON line per
code with its construct, then one per construct; --write appends them.

Some habits share one diagnostic (E0103, a syntax error, holds most of
them), so the code the agent wrote (`tx_put` and `write`) is also matched
against HABITS, patterns of Rust, Verus and Dafny forms the surface does
not take: one JSON line per habit with the sessions whose code shows it.

Usage: bench/syntax/evidence.py [--write FILE]
"""

import glob
import gzip
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CODE = re.compile(r'"code"\s*:\s*"([EW]\d{4})"')

# code prefix or code -> construct (syntax-eval-1.md's rows)
CONSTRUCTS = [
    ("E0103", "구문(일반)"),
    ("E0104", "리터럴 접미사"),
    ("E0201", "`let` 타입"),
    ("E0202", "매크로"),
    ("E0205", "`as` 변환"),
    ("E0206", "`?`"),
    ("E0207", "M1 밖 구성(`pub`·`const`·..)"),
    ("E0209", "`-> ()`"),
    ("E0210", "`mut` 매개변수"),
    ("E0211", "비교 사슬"),
    ("E0212", "정수 넓힘"),
    ("E0229", "`unwrap`"),
    ("E0237", "타입 인자"),
    ("E02", "그 밖의 Rust 습관(E02xx)"),
    ("E0303", "셰도잉"),
    ("E0304", "알 수 없는 함수"),
    ("E0305", "알 수 없는 타입(`int`·`nat`)"),
    ("E03", "이름(E03xx)"),
    ("E0401", "타입 불일치"),
    ("E0404", "알 수 없는 메서드"),
    ("E0413", "이동 뒤 사용"),
    ("E04", "타입·소유(E04xx)"),
    ("E0501", "계약 절 = 이름 붙은 술어"),
    ("E0509", "`#[uses]` 항목"),
    ("E0511", "`#[trigger]` 필요"),
    ("E05", "명세 형태(E05xx)"),
    ("E0601", "`#[effects]` 없음"),
    ("E06", "효과(E06xx)"),
    ("E08", "증명 실패(E08xx)"),
    ("W09", "린트(W09xx)"),
    ("W", "그 밖의 경고"),
    ("E", "그 밖의 오류"),
]


HABITS = [
    ("repeated clause keyword (`invariant a, invariant b`, Dafny)",
     re.compile(r"\b(invariant|ensures|requires)\b[^\n{]*,\s*\n?\s*\1\b")),
    ("loop invariant inside the body (`while c { invariant .. }`)",
     re.compile(r"\bwhile\b[^{\n]*\{\s*\n\s*(invariant|decreases)\b")),
    ("`while` without `decreases`",
     re.compile(r"\bwhile\b(?:(?!\bdecreases\b)[^{])*\{")),
    ("inline contract expression (`requires x > 0`, Verus)",
     re.compile(r"\b(requires|ensures)\s+[^,\n]*?(<=|>=|==|<|>|!=)")),
    ("compound assignment (`x += 1`)", re.compile(r"\b\w+\s*(\+=|-=|\*=|/=|%=|\^=|\|=|&=|<<=|>>=)")),
    ("`as` cast", re.compile(r"\bas\s+(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize|int|nat)\b")),
    ("`int`/`nat` spec type (Verus)", re.compile(r":\s*(int|nat)\b")),
    ("Verus `@` view (`a@.len()`)", re.compile(r"\w@")),
    ("attribute `#[decreases(..)]`/`#[verifier..]`", re.compile(r"#\[(decreases|verifier)")),
    ("`exists` in a specification", re.compile(r"\bexists\s*\|")),
    ("`T::MAX`/`T::MIN`", re.compile(r"\b(u8|u16|u32|u64|usize|i8|i16|i32|i64)::(MAX|MIN)\b")),
    ("shadowing `let x` twice", None),
]


def shadows(code):
    """A name bound twice by `let` in one function body (E0303)."""
    for body in re.split(r"\n\s*(?:#\[[^\n]*\]\s*)*(?:spec |proof )?fn ", code):
        names = re.findall(r"\blet\s+(?:mut\s+)?(\w+)", body)
        if len(names) != len(set(names)):
            return True
    return False


def construct(code):
    for p, name in CONSTRUCTS:
        if code.startswith(p):
            return name
    return "?"


def sessions():
    for d in sorted(glob.glob(os.path.join(ROOT, "bench", "**", "*.sessions"), recursive=True)):
        name = os.path.basename(d)[: -len(".sessions")]
        if name.endswith("-rust") or name.endswith("-verus"):
            continue
        bench = os.path.relpath(d, ROOT).split(os.sep)[1]
        model = "claude-sonnet-5-5" if "claude-sonnet-5-5" in name else ("qwen3.8-27b" if "qwen3.8-27b" in name else "?")
        for f in sorted(glob.glob(os.path.join(d, "*.json.gz"))):
            with gzip.open(f, "rt", encoding="utf-8") as g:
                yield bench, model, json.load(g)


def main():
    write = sys.argv[sys.argv.index("--write") + 1] if "--write" in sys.argv else None
    per = {}
    n = 0
    tokens_all = 0
    habit = {}
    for bench, model, s in sessions():
        n += 1
        code = "\n".join(
            (c.get("args") or {}).get("code") or ""
            for c in s.get("calls", [])
            if c.get("name") in ("tx_put", "write") and isinstance(c.get("args"), dict)
        )
        for name, rx in HABITS:
            hit = shadows(code) if rx is None else bool(rx.search(code))
            if hit:
                h = habit.setdefault(name, {"sessions": 0, "models": {}})
                h["sessions"] += 1
                h["models"][model] = h["models"].get(model, 0) + 1
        tok = sum(r.get("input_tokens", 0) + r.get("output_tokens", 0) for r in s.get("requests", []))
        tokens_all += tok
        results = [CODE.findall(c.get("text") or "") for c in s.get("calls", [])]
        results = [r for r in results if r]
        seen = set()
        for r in results:
            for c in r:
                p = per.setdefault(c, {"hits": 0, "sessions": 0, "first": 0, "unresolved": 0, "tokens": 0, "models": {}, "benches": {}})
                p["hits"] += 1
            seen |= set(r)
        for c in seen:
            p = per[c]
            p["sessions"] += 1
            p["tokens"] += tok
            p["models"][model] = p["models"].get(model, 0) + 1
            p["benches"][bench] = p["benches"].get(bench, 0) + 1
        if results:
            for c in set(results[0]):
                per[c]["first"] += 1
            for c in set(results[-1]):
                per[c]["unresolved"] += 1
    out = []
    for name, hs in sorted(habit.items(), key=lambda kv: -kv[1]["sessions"]):
        out.append({"kind": "habit", "habit": name, **hs, "share": round(hs["sessions"] / n, 4)})
    for c in sorted(per, key=lambda k: -per[k]["sessions"]):
        out.append({"kind": "code", "code": c, "construct": construct(c), **per[c], "share": round(per[c]["sessions"] / n, 4)})
    by = {}
    for r in (x for x in out if x["kind"] == "code"):
        b = by.setdefault(r["construct"], {"codes": [], "hits": 0, "first": 0, "unresolved": 0})
        b["codes"].append(r["code"])
        b["hits"] += r["hits"]
        b["first"] += r["first"]
        b["unresolved"] += r["unresolved"]
    for k, v in sorted(by.items(), key=lambda kv: -kv[1]["hits"]):
        out.append({"kind": "construct", "construct": k, **v})
    out.append({"kind": "summary", "sessions": n, "tokens": tokens_all})
    for r in out:
        print(json.dumps(r, ensure_ascii=False))
        if write:
            with open(write, "a", encoding="utf-8") as w:
                w.write(json.dumps(r, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
