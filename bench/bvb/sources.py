#!/usr/bin/env python3
"""Upstream sources of BVB and the selection record (backlog 2A-08, ADR-0034 §2).

  sources.py fetch DIR     clone the pinned upstream commits into DIR (network)
  sources.py select DIR    recompute the selection from upstream: candidate counts,
                           Vericoding filter and sample order, every decision with its
                           reason; exit 1 if it disagrees with tasks/
  sources.py check DIR     provenance: vendored Verus text (references, given
                           functions) against upstream at the pinned commits
  sources.py table         the selection record as Markdown (no network)

Every upstream task gets one decision here: selected (the BVB task id) or an
exclusion reason (REASONS). The tables are the selection-bias disclosure that
ADR-0034's fairness rules ask for; SOURCES.md shows them. Standard library only.
"""
import hashlib
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TASKS = os.path.join(HERE, "tasks")

UPSTREAM = {
    "verus-proof-synthesis": ("https://github.com/microsoft/verus-proof-synthesis",
                              "cbf9c0c6337b224fd8e5b7cb4e01ae65c0f98bc1"),
    "vericoding-benchmark": ("https://github.com/Beneficial-AI-Foundation/vericoding-benchmark",
                             "387cd69996792d452ead7b0460f36ee4c5cdd148"),
}

REASONS = {
    "vec-out": "returns or grows a Vec (M1 has no heap or Vec)",
    "nested-vec": "takes nested Vecs (M1 slices hold only integers or bool)",
    "string": "takes a String or char (no strings in M1)",
    "tuple-option": "returns a tuple or an Option (no tuples, structs or enums in M1)",
    "i128": "result needs i128 (largest M1 integer is 64-bit)",
    "subrange-exists": "spec is an existential over Seq::subrange equality; in BMB it needs a named "
                       "window predicate the agent cannot know (trigger matching), so it is not portable faithfully",
    "signed-index": "returns an index as a signed integer with a -1 sentinel; M1 has no integer "
                    "conversion, so a spec cannot index with the result",
    "mut-scalar": "&mut scalar parameter (M1 references are slices only)",
    "permutation": "spec is a permutation of a Seq (needs sequence values)",
    "out-of-range": "spec quantifies over indices outside the array (Seq semantics)",
    "fixed-program": "invariant-synthesis task on a fixed program whose spec is a weak property of "
                     "its output; as spec-to-code it is solved by a constant",
    "no-exec": "proof-only task or no specification to implement",
    "escape-hatch": "the original uses assume",
    "dup": "same signature shape and spec as an earlier upstream task (kept the first)",
    "unsat": "unsolvable as stated: the spec forces a result outside the original return type for "
             "some valid input",
    "trivial": "postcondition holds for every value of the result type",
    "no-input": "no input (a constant)",
    "no-bmb-ref": "the author found no BMB reference proof: M1 has no lemma functions or "
                  "quantifier introduction (bias toward BMB, disclosed)",
    "upstream-issue": "listed in vericoding verus_issues.jsonl (upstream quality issue)",
}

# Verus-Bench (benchmarks/Verus-Bench/tasks.jsonl, 150 tasks): every task_id.
SEL = "selected"
VERUS_BENCH = {
    "cloverbench_all_digits_strong": "string", "cloverbench_array_append_strong": "vec-out",
    "cloverbench_array_concat_strong": "vec-out", "cloverbench_array_copy_strong": "vec-out",
    "cloverbench_array_product_strong": "vec-out", "cloverbench_array_sum_strong": "vec-out",
    "cloverbench_binary_search": "vb-clover-binary-search", "cloverbench_cal_div": "tuple-option",
    "cloverbench_is_prime": "vb-clover-is-prime", "cloverbench_linear_search2": "vb-clover-linear-search",
    "cloverbench_two_sum": "tuple-option",
    "misc_arg_free": "no-exec", "misc_basic_nonlinear": "no-exec", "misc_binary_search": "dup",
    "misc_bubble": "permutation", "misc_cell_2_sum": "fixed-program", "misc_choose_odd": "vb-misc-choose-odd",
    "misc_conditional_average": "vec-out", "misc_deduplicate": "vec-out", "misc_fib": "vec-out",
    "misc_filter": "vec-out", "misc_filter_weak": "vec-out", "misc_findmax": "vb-misc-findmax",
    "misc_havoc_inline_post": "escape-hatch", "misc_len_intersect": "no-exec",
    "misc_linearsearch": "signed-index", "misc_map": "vb-misc-map", "misc_max_index": "vb-misc-max-index",
    "misc_remove_all_greater": "vec-out", "misc_reverse": "vb-misc-reverse",
    "misc_simple_nested": "out-of-range", "misc_sum": "vb-misc-sum", "misc_tail_triangle": "mut-scalar",
    "misc_trigger": "out-of-range",
}
for n in ("105:vb-mbpp-105-count-true 113:vb-mbpp-113-is-integer 133:i128 142:vb-mbpp-142-count-identical "
          "145:vb-mbpp-145-max-difference 161:vec-out 170:i128 18:vec-out 2:vec-out 230:vec-out 240:vec-out "
          "249:vec-out 251:vec-out 261:vec-out 262:vec-out 273:vec-out 282:vec-out 284:vb-mbpp-284-all-equal "
          "290:nested-vec 3:vb-mbpp-3-is-non-prime 307:vec-out 399:vec-out 412:vec-out "
          "414:vb-mbpp-414-any-value-exists 424:vec-out 426:vec-out 431:dup 433:vb-mbpp-433-is-greater "
          "436:vec-out 445:vec-out 447:vec-out 454:vb-mbpp-454-contains-z 457:nested-vec 460:nested-vec "
          "461:vb-mbpp-461-count-uppercase 472:vb-mbpp-472-consecutive 474:vec-out 476:vb-mbpp-476-sum-min-max "
          "477:vec-out 554:vec-out 557:vec-out 567:vb-mbpp-567-is-sorted 572:vec-out 576:subrange-exists "
          "578:vec-out 579:vec-out 586:vec-out 588:vb-mbpp-588-max-min-diff 602:tuple-option 605:dup "
          "610:vec-out 616:vec-out 618:vec-out 62:vb-mbpp-62-smallest 624:vec-out 629:vec-out 644:vec-out "
          "69:subrange-exists 70:nested-vec 728:vec-out 732:vec-out 733:tuple-option 741:vb-mbpp-741-all-same "
          "743:vec-out 755:tuple-option 760:dup 764:vb-mbpp-764-count-digits 769:vec-out "
          "775:vb-mbpp-775-odd-at-odd 790:dup 798:i128 8:vec-out 804:vb-mbpp-804-product-even "
          "807:tuple-option 808:vb-mbpp-808-contains-k 809:vb-mbpp-809-is-smaller 94:nested-vec "
          "95:nested-vec").split():
    k, v = n.split(":")
    VERUS_BENCH["mbpp_task_id_" + k] = v
for n in ("brs1 brs2 brs3 brs4 brs5 conda condg condm condn ms1 ms2 ms3 ms4 ms5 res1 res1o res2 res2o "
          "s12if s1if s1lif s22if s2if s2lif s32if s3if s3lif s42if s4if s4lif s52if s5if s5lif "
          "sina1 sina2 sina3 sina4 sina5").split():
    VERUS_BENCH["diffy_" + n] = "fixed-program"
DUP_OF = {"misc_binary_search": "cloverbench_binary_search", "mbpp_task_id_431": "mbpp_task_id_414",
          "mbpp_task_id_605": "cloverbench_is_prime", "mbpp_task_id_760": "mbpp_task_id_741",
          "mbpp_task_id_790": "mbpp_task_id_775", "VJ0020": "misc_findmax", "VD0114": "misc_max_index",
          "VV0149": "cloverbench_linear_search2"}

# Vericoding Verus tasks that pass FILTER, in SAMPLE_ORDER, examined until 15 were selected.
VERICODING_TARGET = 15
VERICODING = [
    ("VJ0020", "dup"), ("VH0047", "unsat"), ("VD0791", "vc-vd0791-month-30-days"),
    ("VA0523", "vc-va0523-min-abs-diff"), ("VD0722", "vc-vd0722-last-digit"), ("VH0013", "no-bmb-ref"),
    ("VA0590", "vc-va0590-boring-apartment"), ("VA0676", "unsat"), ("VV0104", "unsat"), ("VA0205", "unsat"),
    ("VD0356", "unsat"), ("VT0094", "no-input"), ("VJ0055", "mut-scalar"), ("VD0114", "dup"),
    ("VV0128", "no-bmb-ref"), ("VV0033", "unsat"), ("VA0532", "vc-va0532-crackers"),
    ("VA0093", "vc-va0093-max-balance"), ("VA0053", "vc-va0053-aspect-pairs"),
    ("VD0727", "vc-vd0727-calculate-loss"), ("VV0039", "vc-vv0039-max-of-list"),
    ("VA0399", "vc-va0399-late-brother"), ("VD0131", "unsat"), ("VA0219", "vc-va0219-pyramid-height"),
    ("VH0043", "unsat"), ("VH0085", "unsat"), ("VD0126", "vc-vd0126-first-zero"), ("VV0100", "trivial"),
    ("VD0465", "vc-vd0465-increasing-run"), ("VD0670", "unsat"), ("VS0021", "unsat"),
    ("VV0135", "vc-vv0135-compare"), ("VD0199", "signed-index"), ("VV0073", "trivial"), ("VV0149", "dup"),
    ("VD0239", "signed-index"), ("VA0644", "vc-va0644-takahashi-dates"),
]
# Why each `unsat` task is unsolvable (an input in range whose required result does not fit).
UNSAT_WITNESS = {
    "VH0047": "base = height = 255: area 32512 > u8::MAX",
    "VA0676": "r = 0, g = 127: 2g - r = 254 > i8::MAX",
    "VV0104": "a = [i32::MAX - 1, i32::MIN + 1]: even - odd overflows i32",
    "VA0205": "n = 127, m = 2: 253 days > i8::MAX",
    "VD0356": "n = 2^63: f(n) = 3 * 2^63 - 1 > u64::MAX",
    "VV0033": "len >= 2^31: `nums.len() as i32` is negative, so result >= 0 && result <= it fails",
    "VD0131": "v = [i32::MAX, i32::MAX]: both are peeks, sum 2 * i32::MAX > i32::MAX",
    "VH0043": "n = 127: 127 * 127 > i8::MAX",
    "VH0085": "n = 3: 100 + 90 - 10 = 180 > i8::MAX",
    "VD0670": "a = [i32::MIN, -1]: sum < i32::MIN",
    "VS0021": "a = -128, b = 0: gcd 128 > i8::MAX",
}

SCALAR = r"(i8|i16|i32|i64|u8|u16|u32|u64|usize|bool)"
PARAM = re.compile(r"^\w+:\s*(&(mut )?)?(" + SCALAR + r"|\[" + SCALAR + r"\]|Vec<" + SCALAR + r">)$")


def vericoding_filter(t):
    """M1 shape filter: None if the task may be portable, else why not (automatic step)."""
    spec = t["vc-spec"]
    body = t["vc-preamble"] + t["vc-helpers"] + spec
    if len(re.findall(r"fn\s+\w+\s*\(", spec)) != 1:
        return "several functions to implement"
    m = re.search(r"fn\s+(\w+)\s*\((.*?)\)\s*(->\s*\((\w+):\s*([^)]*)\))?\s*(requires|ensures|$)", spec, re.S)
    if not m:
        return "signature not recognised"
    params = [p.strip() for p in m.group(2).split(",") if p.strip()]
    if not all(PARAM.match(p) for p in params):
        return "parameter type outside M1 (struct, String, nested Vec, float, ...)"
    if any("&mut Vec" in p for p in params):
        return "&mut Vec parameter (may grow)"
    if m.group(5) and not re.fullmatch(SCALAR, m.group(5).strip()):
        return "return type outside M1 (Vec, tuple, Option, ...)"
    if re.search(r"\b(struct|enum|Option|String|char|str|f32|f64|Map|Set|Multiset|impl|trait|fold_left|filter|"
                 r"map|to_multiset|i128|u128)\b", body):
        return "spec uses types or collections outside M1"
    if re.search(r"Seq<|seq!|subrange|\.take|\.skip|\.push|\.add\(", body):
        return "spec builds sequences"
    return None


def sample_key(task_id):
    return hashlib.sha256(("bvb:" + task_id).encode()).hexdigest()


# ---------- upstream access ----------

def git(*args, cwd=None):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout


def fetch(root):
    for name, (url, commit) in UPSTREAM.items():
        d = os.path.join(root, name)
        if not os.path.isdir(os.path.join(d, ".git")):
            git("init", "-q", d)
            git("remote", "add", "origin", url, cwd=d)
        git("fetch", "-q", "--depth", "1", "origin", commit, cwd=d)
        git("checkout", "-q", "FETCH_HEAD", cwd=d)
        print(f"{name}: {git('rev-parse', 'HEAD', cwd=d).strip()}")


def pinned(root, name):
    d = os.path.join(root, name)
    head = git("rev-parse", "HEAD", cwd=d).strip()
    if head != UPSTREAM[name][1]:
        sys.exit(f"{d} is at {head}, not the pinned {UPSTREAM[name][1]} (run fetch)")
    return d


def bvb_tasks():
    return {f[:-5]: json.load(open(os.path.join(TASKS, f))) for f in sorted(os.listdir(TASKS))}


# ---------- selection ----------

def selection(root):
    """Recomputes the selection from upstream; returns (report lines, problems)."""
    out, problems = [], []
    vb_dir = pinned(root, "verus-proof-synthesis")
    vb = [json.loads(line) for line in open(os.path.join(vb_dir, "benchmarks/Verus-Bench/tasks.jsonl"))]
    ids = [t["task_id"] for t in vb]
    if sorted(ids) != sorted(VERUS_BENCH):
        problems.append("Verus-Bench task list differs from VERUS_BENCH")
    out.append(f"Verus-Bench: {len(ids)} tasks")
    vc_dir = pinned(root, "vericoding-benchmark")
    vc = [json.loads(line) for line in open(os.path.join(vc_dir, "jsonl/verus_tasks.jsonl"))]
    issues = sum(1 for _ in open(os.path.join(vc_dir, "jsonl/verus_issues.jsonl")))
    passing = [t["id"] for t in vc if vericoding_filter(t) is None]
    why = {}
    for t in vc:
        r = vericoding_filter(t)
        if r:
            why[r] = why.get(r, 0) + 1
    out.append(f"Vericoding Verus: {len(vc)} tasks (+{issues} in verus_issues.jsonl), "
               f"{len(passing)} pass the M1 shape filter")
    for r, n in sorted(why.items(), key=lambda kv: -kv[1]):
        out.append(f"  filtered out, {r}: {n}")
    order = sorted(passing, key=sample_key)
    examined = [i for i, _ in VERICODING]
    if order[:len(examined)] != examined:
        problems.append("Vericoding sample order differs from VERICODING")
    selected = [d for _, d in VERICODING if d not in REASONS]
    if len(selected) != VERICODING_TARGET or VERICODING[-1][1] in REASONS:
        problems.append("VERICODING must end at its 15th selection")
    want = sorted([d for d in VERUS_BENCH.values() if d not in REASONS] + selected)
    if want != sorted(bvb_tasks()):
        problems.append("selected tasks differ from tasks/")
    return out, problems


def check(root):
    """Vendored Verus text equals upstream (references marked upstream, given functions)."""
    problems = []
    vb_dir = pinned(root, "verus-proof-synthesis")
    vc_dir = pinned(root, "vericoding-benchmark")
    squash = lambda s: re.sub(r"\s+", " ", s).strip()  # noqa: E731
    for tid, t in bvb_tasks().items():
        v, src = t["verus"], t["source"]
        if src["benchmark"] == "Verus-Bench":
            up = open(os.path.join(vb_dir, src["path"])).read()
        else:
            vid = src["path"].split("#")[1]
            p = os.path.join(vc_dir, "vericoded", f"{vid}_vericoded.rs")
            up = open(p).read() if os.path.exists(p) else None
        origin = v["reference_origin"]
        if origin == "upstream" and v["reference"].strip("\n") != up.strip("\n"):
            problems.append(f"{tid}: verus.reference differs from upstream")
        if origin.startswith("upstream-") and up is not None and v["reference"].strip("\n") == up.strip("\n"):
            problems.append(f"{tid}: marked {origin} but equal to upstream")
        if v.get("given") and up is not None and origin.startswith("upstream"):
            if squash(v["given"]) not in squash(up):
                problems.append(f"{tid}: verus.given is not upstream text")
    return problems


def table():
    """The selection record as Markdown."""
    tasks = bvb_tasks()
    lines = ["| 원본 | 결정 |", "| --- | --- |"]
    for k in sorted(VERUS_BENCH):
        d = VERUS_BENCH[k]
        lines.append(f"| Verus-Bench `{k}` | " + (f"`{d}`" if d in tasks else f"제외: {d}"
                     + (f" (= `{DUP_OF[k]}`)" if k in DUP_OF else "")) + " |")
    for k, d in VERICODING:
        note = f" ({UNSAT_WITNESS[k]})" if k in UNSAT_WITNESS else (f" (= `{DUP_OF[k]}`)" if k in DUP_OF else "")
        lines.append(f"| Vericoding `{k}` | " + (f"`{d}`" if d in tasks else f"제외: {d}{note}") + " |")
    return "\n".join(lines) + "\n"


def counts():
    c = {}
    for d in VERUS_BENCH.values():
        key = "selected" if d not in REASONS else d
        c[key] = c.get(key, 0) + 1
    v = {}
    for _, d in VERICODING:
        key = "selected" if d not in REASONS else d
        v[key] = v.get(key, 0) + 1
    return c, v


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    cmd = sys.argv[1]
    if cmd == "table":
        sys.stdout.write(table())
        vb, vc = counts()
        print("\nVerus-Bench:", json.dumps(vb, sort_keys=True))
        print("Vericoding (examined):", json.dumps(vc, sort_keys=True))
        return
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    root = sys.argv[2]
    if cmd == "fetch":
        os.makedirs(root, exist_ok=True)
        fetch(root)
    elif cmd == "select":
        out, problems = selection(root)
        print("\n".join(out))
        if problems:
            sys.exit("\n".join(problems))
        print("selection matches tasks/")
    elif cmd == "check":
        problems = check(root)
        if problems:
            sys.exit("\n".join(problems))
        print("vendored Verus text matches upstream")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
