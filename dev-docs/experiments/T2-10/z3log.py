#!/usr/bin/env python3
# T2-10 measurement proxy, put on PATH as `z3` (measure.sh): forwards the
# compiler's commands to the real z3 (Z3_REAL) and after each (check-sat)
# asks (get-info :rlimit) itself, logging one JSON row per query to Z3_LOG.
import json, os, subprocess, sys
REAL = os.environ.get("Z3_REAL", "/usr/local/bin/z3")
if len(sys.argv) > 1 and sys.argv[1] == "--version":
    os.execv(REAL, [REAL] + sys.argv[1:])
log = open(os.environ["Z3_LOG"], "a")
qdir = os.environ.get("Z3_QDIR")
label = os.environ.get("Z3_LABEL", "")
p = subprocess.Popen([REAL] + sys.argv[1:], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
def resp():
    out, depth = "", 0
    while True:
        line = p.stdout.readline()
        if not line:
            return None
        depth += line.count("(") - line.count(")")
        out += line
        if depth <= 0 and out.strip():
            return out
seg, limit, n = [], None, [0]
for line in sys.stdin:
    cmd = line.strip()
    if cmd == "(reset)":
        seg, limit = [], None
    if cmd.startswith("(set-option :rlimit"):
        limit = int(cmd.split()[-1].rstrip(")"))
    seg.append(cmd)
    p.stdin.write(line); p.stdin.flush()
    r = resp()
    if r is None:
        break
    if cmd == "(check-sat)":
        p.stdin.write("(get-info :rlimit)\n"); p.stdin.flush()
        rl = int(resp().strip().strip("()").split()[1])
        ans = r.strip()
        why = ""
        if ans == "unknown":
            p.stdin.write("(get-info :reason-unknown)\n"); p.stdin.flush()
            why = resp().strip()
        n[0] += 1
        row = {"label": label, "pid": os.getpid(), "k": n[0], "limit": limit, "answer": ans, "rlimit": rl, "why": why}
        if qdir:
            fn = os.path.join(qdir, f"{os.getpid()}-{n[0]}.smt2")
            with open(fn, "w") as f:
                f.write(f"; {label} {ans} {rl}\n" + "\n".join(seg) + "\n")
            row["file"] = fn
        log.write(json.dumps(row) + "\n"); log.flush()
    sys.stdout.write(r); sys.stdout.flush()
    if cmd == "(exit)":
        break
p.stdin.close(); p.wait()
