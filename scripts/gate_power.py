#!/usr/bin/env python3
"""Detection power of the gate against prover and code-generator defects
(backlog 1B-38, HEALTH.md §7).

  gate_power.py check                  every mutant applies to the current sources
  gate_power.py run --work DIR --out FILE.jsonl [--only ID,..]
  gate_power.py table FILE.jsonl       Markdown table and the catch rate

Each mutant is one plausible defect in `compiler/src/verify.rs` or
`compiler/src/codegen.rs`: an encoding with the wrong sign or bound, an
obligation that is never sent, a havoc that forgets a variable, a code
generator that emits the wrong instruction or drops a check it may not drop.
`run` applies them one at a time to a detached worktree of HEAD (DIR) and runs
the whole gate there (`scripts/check.sh`) after `cargo fmt` (formatting
must not be what catches a defect); the gate catches a mutant when it exits
non-zero, and `detected_by` names the gate jobs that failed (empty when an
earlier step such as clippy or the build failed; `tail` shows which). Classes: `unsound` (a false program could be proved),
`incomplete` (a true program could fail), `miscompile` (a proved program could
run wrongly). A mutant the gate misses is either a gap (a test is added for
it) or equivalent (no program can tell, reasoned in `note`).
Standard library only.
"""
import argparse
import json
import os
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
V, C = "compiler/src/verify.rs", "compiler/src/codegen.rs"

# (id, class, file, old, new, what)
MUTANTS = [
    ("v-read-index-le", "unsound", V,
     '''                    e.span,
                    "index may be out of bounds",
                    format!("(and (<= 0 {i}) (< {i} {len}))"),''',
     '''                    e.span,
                    "index may be out of bounds",
                    format!("(and (<= 0 {i}) (<= {i} {len}))"),''',
     "reading a[len] is allowed"),
    ("v-store-index-le", "unsound", V,
     '''                        e.span,
                        "index may be out of bounds",
                        format!("(and (<= 0 {i}) (< {i} {len}))"),''',
     '''                        e.span,
                        "index may be out of bounds",
                        format!("(and (<= 0 {i}) (<= {i} {len}))"),''',
     "writing a[len] is allowed"),
    ("v-div-zero-dropped", "unsound", V,
     '''                            "divisor may be zero",
                            format!("(not (= {rt} 0))"),''',
     '''                            "divisor may be zero",
                            "true".to_string(),''',
     "division by zero is never an obligation"),
    ("v-min-rem-dropped", "unsound", V,
     '''format!("(not (and (= {lt} {I64_MIN}) (= {rt} (- 1))))"),''',
     '''"true".to_string(),''',
     "i64::MIN % -1 is never an obligation"),
    ("v-neg-overflow-dropped", "unsound", V,
     '''self.oblige("E0803", e.span, "negation may overflow", r);''',
     '''let _ = ("E0803", e.span, "negation may overflow", r);''',
     "-i64::MIN is never an obligation"),
    ("v-sub-swapped", "unsound", V,
     '''                let t = match op {
                    BinOp::Add => format!("(+ {lt} {rt})"),
                    BinOp::Sub => format!("(- {lt} {rt})"),''',
     '''                let t = match op {
                    BinOp::Add => format!("(+ {lt} {rt})"),
                    BinOp::Sub => format!("(- {rt} {lt})"),''',
     "a - b is encoded as b - a"),
    ("v-mul-as-add", "unsound", V,
     '''                    BinOp::Mul => {
                        self.mul_facts(&lt, &rt);
                        format!("(* {lt} {rt})")''',
     '''                    BinOp::Mul => {
                        self.mul_facts(&lt, &rt);
                        format!("(+ {lt} {rt})")''',
     "a * b is encoded as a + b"),
    ("v-spec-lt-as-le", "unsound", V,
     '''                BinOp::Lt => format!("(and ({le} {a} {b}) (not (= {a} {b})))"),''',
     '''                BinOp::Lt => format!("({le} {a} {b})"),''',
     "for a type parameter bounded by Ord, a < b means a <= b (contracts and code share `ord_term` since 2T-02)"),
    ("v-lt-as-le", "unsound", V,
     '''                BinOp::Lt => "<",
                BinOp::Le => "<=",''',
     '''                BinOp::Lt => "<=",
                BinOp::Le => "<=",''',
     "a < b is encoded as a <= b"),
    ("v-ge-as-gt", "unsound", V,
     '''                BinOp::Gt => ">",
                _ => ">=",''',
     '''                BinOp::Gt => ">",
                _ => ">",''',
     "a >= b is encoded as a > b"),
    ("v-or-guard", "unsound", V,
     '''(format!("(not {lt})"), "or")''',
     '''(lt.clone(), "or")''',
     "the right operand of || is assumed to run when the left is true"),
    ("v-element-range-dropped", "incomplete", V,
     '''                if let Some(r) = range(&self.ty(e), &t) {
                    self.pc.push(r);
                }''',
     '''                if let Some(r) = range(&self.ty(e), &t) {
                    let _ = r;
                }''',
     "array elements read lose their type range"),
    ("v-i64-range-plus-one", "unsound", V,
     '''TypeKind::I64 => Some(format!("(and (<= {I64_MIN} {x}) (<= {x} {I64_MAX}))")),''',
     '''TypeKind::I64 => Some(format!("(and (<= {I64_MIN} {x}) (<= {x} (+ {I64_MAX} 1)))")),''',
     "i64 overflow by one is allowed"),
    ("v-u64-range-minus-one", "unsound", V,
     '''TypeKind::U64 | TypeKind::Usize => Some(format!("(and (<= 0 {x}) (<= {x} {U64_MAX}))")),''',
     '''TypeKind::U64 | TypeKind::Usize => Some(format!("(and (<= (- 1) {x}) (<= {x} {U64_MAX}))")),''',
     "unsigned underflow to -1 is allowed"),
    ("v-invariant-entry-dropped", "unsound", V,
     '''self.oblige("E0806", inv.span, "loop invariant may not hold on entry", g);''',
     '''let _ = ("E0806", inv.span, "loop invariant may not hold on entry", g);''',
     "invariants are assumed without holding on entry"),
    ("v-invariant-preserved-dropped", "unsound", V,
     '''self.oblige("E0807", inv.span, "loop invariant may not be preserved", g);''',
     '''let _ = ("E0807", inv.span, "loop invariant may not be preserved", g);''',
     "invariants are assumed without being preserved"),
    ("v-decreases-unbounded", "unsound", V,
     '''                "`decreases` may not decrease or may be negative",
                format!("(and (<= 0 {d0}) (< {d1} {d0}))"),''',
     '''                "`decreases` may not decrease or may be negative",
                format!("(< {d1} {d0})"),''',
     "a loop measure may go below zero"),
    ("v-havoc-skipped", "unsound", V,
     '''        for v in &modified {
            let Some(cur) = self.vars.get(v).cloned() else {''',
     '''        for v in modified.iter().take(0) {
            let Some(cur) = self.vars.get(v).cloned() else {''',
     "variables a loop assigns keep their entry values"),
    ("v-mut-param-havoc-skipped", "unsound", V,
     '''Some(n) => mut_params.contains(n).then(|| n.to_string()),''',
     '''Some(n) => mut_params.contains(n).then(|| n.to_string()).filter(|_| false),''',
     "a loop that writes through a &mut parameter keeps its entry contents"),
    ("v-call-requires-dropped", "unsound", V,
     '''            self.oblige(
                "E0801",''',
     '''            let _ = (
                "E0801",''',
     "callee preconditions are never obligations"),
    ("v-ensures-dropped", "unsound", V,
     '''                    enc.oblige(
                        "E0802",''',
     '''                    let _ = (
                        "E0802",''',
     "postconditions are never obligations"),
    ("v-assert-dropped", "unsound", V,
     '''self.oblige("E0810", *span, "assertion may not hold", g);''',
     '''let _ = ("E0810", *span, "assertion may not hold", g);''',
     "assertions are never obligations"),
    ("v-recursion-measure-dropped", "unsound", V,
     '''            self.oblige(
                "E0809",''',
     '''            let _ = (
                "E0809",''',
     "recursive calls need not decrease their measure"),
    ("v-signed-div-rem-swapped", "unsound", V,
     '''                                format!("(bmb_tdiv {lt} {rt})"),
                                format!("(bmb_trem {lt} {rt})"),''',
     '''                                format!("(bmb_trem {lt} {rt})"),
                                format!("(bmb_tdiv {lt} {rt})"),''',
     "signed / and % are swapped in the encoding"),
    ("c-add-as-sub", "miscompile", C,
     '''BinOp::Add => self.arith(e, "add", &a, &b, signed),''',
     '''BinOp::Add => self.arith(e, "sub", &a, &b, signed),''',
     "+ is compiled as -"),
    ("c-slt-as-sle", "miscompile", C,
     '''(BinOp::Lt, true) => "slt",''',
     '''(BinOp::Lt, true) => "sle",''',
     "signed < is compiled as <="),
    ("c-ult-as-ule", "miscompile", C,
     '''(BinOp::Lt, false) => "ult",''',
     '''(BinOp::Lt, false) => "ule",''',
     "unsigned < is compiled as <="),
    ("c-sdiv-as-udiv", "miscompile", C,
     '''(BinOp::Div, true) => "sdiv",''',
     '''(BinOp::Div, true) => "udiv",''',
     "signed / is compiled as unsigned"),
    ("c-bounds-check-ule", "miscompile", C,
     '''        let i = self.expr(index).imm();
        let note = if self.proved("E0805", e) {
            " ; proved E0805"
        } else {
            let c = self.t();
            self.emit(format!("{c} = icmp ult i64 {i}, {len}"));''',
     '''        let i = self.expr(index).imm();
        let note = if self.proved("E0805", e) {
            " ; proved E0805"
        } else {
            let c = self.t();
            self.emit(format!("{c} = icmp ule i64 {i}, {len}"));''',
     "the runtime bounds check (unproved indexing) lets a[len] through"),
    ("c-overflow-check-removed", "miscompile", C,
     '''        if self.proved("E0803", e) {
            let flag = if signed { "nsw" } else { "nuw" };''',
     '''        if self.proved("E0803", e) || !self.proved("E0803", e) {
            let flag = if signed { "nsw" } else { "nuw" };''',
     "overflow checks are dropped without a proof (nsw/nuw on unproved arithmetic)"),
    ("c-bounds-check-removed", "miscompile", C,
     '''        let i = self.expr(index).imm();
        let note = if self.proved("E0805", e) {''',
     '''        let i = self.expr(index).imm();
        let note = if self.proved("E0805", e) || !self.proved("E0805", e) {''',
     "bounds checks are dropped without a proof"),
    ("v-refine-post-dropped", "unsound", V,
     '''                    format!("postcondition `{}` of `{decl}` may not hold", pred_name(en)),
                    format!("(=> {hyp} {g})"),''',
     '''                    format!("postcondition `{}` of `{decl}` may not hold", pred_name(en)),
                    format!("(or (=> {hyp} {g}) true)"),''',
     "a trait impl's postconditions need not imply the trait's (E0817)"),
    ("v-refine-pre-dropped", "unsound", V,
     '''                    pred_name(r)
                ),
                format!("(=> {pre} {g})"),''',
     '''                    pred_name(r)
                ),
                format!("(or (=> {pre} {g}) true)"),''',
     "a trait impl may require more than the trait (E0817)"),
    ("v-default-spec-leaks", "unsound", V,
     '''                                    .is_some_and(|(p, _)| p == u)''',
     '''                                    .is_none_or(|(p, _)| p == u)''',
     "a default spec body is revealed for a bounded `Self`, whose impl may override it"),
    ("c-trait-call-undispatched", "miscompile", C,
     '''                .zip(a)
                .collect();
            target = t;''',
     '''                .zip(a)
                .collect();
            if t.is_empty() {
                target = t;
            }''',
     "a trait call with a known `Self` is not lowered to its impl's function"),
    # Owned types (ADR-0047, phase 2O, backlog 2O-05).
    ("c-box-free-dropped", "miscompile", C,
     r'''                body.push_str("  call void @free(ptr %h)\n");''',
     r'''                body.push_str("");''',
     "a dropped `Box` is never freed (a leak at every drop)"),
    ("v-replace-returns-new", "unsound", V,
     '''            return old;''',
     '''            return self.eval(&p0);''',
     "`std::mem::replace` is taken to return the new value"),
    ("v-decrease-any-term", "unsound", V,
     '''                    if cur == a {
                        return and(&testers);
                    }
                }
                "false".into()''',
     '''                    if cur == a {
                        return and(&testers);
                    }
                }
                "true".into()''',
     "a structural `decreases` holds for any argument, not only a part of the caller's value"),
    ("c-mut-match-write-back-dropped", "miscompile", C,
     '''            for (tmp, fp, t) in write_back {''',
     '''            for (tmp, fp, t) in write_back.into_iter().take(0) {''',
     "a scalar written through a `&mut` match binding is not stored back to the payload"),
]


def apply(root, m):
    _, _, path, old, new, _ = m
    p = os.path.join(root, path)
    text = open(p).read()
    n = text.count(old)
    if n != 1:
        sys.exit(f"{m[0]}: the old text occurs {n} times in {path}")
    with open(p, "w") as f:
        f.write(text.replace(old, new))


def check(_args):
    for m in MUTANTS:
        for path in {m[2]}:
            n = open(os.path.join(ROOT, path)).read().count(m[3])
            if n != 1:
                sys.exit(f"{m[0]}: the old text occurs {n} times in {path}")
    ids = [m[0] for m in MUTANTS]
    assert len(set(ids)) == len(ids), "duplicate ids"
    print(f"gate_power: {len(MUTANTS)} mutants apply")


def git(*args, cwd=ROOT):
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def run(args):
    head = git("rev-parse", "HEAD")
    if not os.path.isdir(args.work):
        git("worktree", "add", "--detach", args.work, head)
    else:
        # A stopped run may leave a mutant applied: discard it.
        git("checkout", "--force", "--detach", head, cwd=args.work)
    only = set(args.only.split(",")) if args.only else None
    done = set()
    if os.path.exists(args.out):
        done = {json.loads(line)["id"] for line in open(args.out) if line.strip()}
    with open(args.out, "a") as out:
        for m in MUTANTS:
            if (only and m[0] not in only) or m[0] in done:
                continue
            git("checkout", "--", "compiler/src", cwd=args.work)
            apply(args.work, m)
            subprocess.run(["cargo", "fmt"], cwd=args.work, capture_output=True)
            t0 = time.monotonic()
            env = dict(os.environ, BMB_GATE_POWER_RUN="1")
            r = subprocess.run(["timeout", "3600", "scripts/check.sh"], cwd=args.work, capture_output=True, text=True, env=env)
            lines = (r.stdout + r.stderr).strip().splitlines()
            tail = lines[-12:]
            failed = next((x for x in reversed(lines) if x.startswith("failed: ")), "")
            detected_by = [j.strip() for j in failed[len("failed: "):].split(",") if j.strip()]
            row = {"id": m[0], "class": m[1], "file": m[2], "what": m[5], "commit": head,
                   "caught": r.returncode != 0, "exit": r.returncode, "detected_by": detected_by,
                   "seconds": round(time.monotonic() - t0, 1), "tail": tail}
            out.write(json.dumps(row, ensure_ascii=False) + "\n")
            out.flush()
            print(f"gate_power: {m[0]}: {'caught' if row['caught'] else 'MISSED'} ({row['seconds']}s)", flush=True)
        git("checkout", "--", "compiler/src", cwd=args.work)


def table(args):
    rows = [json.loads(line) for line in open(args.results) if line.strip()]
    out = ["| 변이체 | 종류 | 결함 | 게이트 |", "| --- | --- | --- | --- |"]
    for r in rows:
        by = ", ".join(r.get("detected_by") or []) or "앞 단계"
        out.append(f"| `{r['id']}` | {r['class']} | {r['what']} | {('잡음: ' + by) if r['caught'] else '**놓침**'} |")
    caught = sum(r["caught"] for r in rows)
    out.append("")
    out.append(f"검출: {caught}/{len(rows)}")
    for cls in ("unsound", "incomplete", "miscompile"):
        rs = [r for r in rows if r["class"] == cls]
        if rs:
            out.append(f"- {cls}: {sum(r['caught'] for r in rs)}/{len(rs)}")
    print("\n".join(out))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("check")
    r = sub.add_parser("run")
    r.add_argument("--work", required=True)
    r.add_argument("--out", required=True)
    r.add_argument("--only")
    t = sub.add_parser("table")
    t.add_argument("results")
    args = ap.parse_args()
    {"check": check, "run": run, "table": table}[args.cmd](args)


if __name__ == "__main__":
    main()
