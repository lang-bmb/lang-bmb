#!/usr/bin/env python3
"""Classify chained_comparison chains by RHS pattern type."""
import sys
import json
import re
from collections import defaultdict

PRELUDE = 2885

with open('bootstrap/compiler.bmb', 'rb') as f:
    content = f.read()

import subprocess
result = subprocess.run(
    ['./target/release/bmb', 'check', 'bootstrap/compiler.bmb'],
    capture_output=True, encoding='utf-8', errors='replace'
)

warnings = []
for line in (result.stdout + result.stderr).splitlines():
    try:
        w = json.loads(line.strip())
        if w.get('type') == 'warning' and w.get('kind') == 'chained_comparison':
            warnings.append(w)
    except Exception:
        pass

groups = defaultdict(list)
for w in warnings:
    groups[w['end']].append(w)

roots = []
for end, ws in groups.items():
    root = min(ws, key=lambda x: x['start'])
    m = re.search(r'(\d+) chained', root['message'])
    n = int(m.group(1)) if m else 0
    var = re.search(r'on `([^`]+)`', root['message'])
    v = var.group(1) if var else '?'
    roots.append((root['line'], v, n, root['start'], end))

roots.sort()

literal_chains = []
fn_call_chains = []

for line, var, n, raw_start_pp, raw_end_pp in roots:
    raw_start = raw_start_pp - PRELUDE
    raw_end = raw_end_pp - PRELUDE
    segment = content[raw_start:raw_end + 200].decode('utf-8', errors='replace')

    # Check first comparison on the right side
    # Pattern: "if VAR == RHS { ..."
    # Find the RHS of the first == VAR
    pat = re.compile(r'if\s+' + re.escape(var) + r'\s*==\s*([^\s{]+)')
    m = pat.search(segment)
    if m:
        rhs = m.group(1)
        # Is it a literal? (integer, string, char)
        is_lit = bool(re.match(r'^-?\d+$', rhs)) or rhs.startswith('"') or rhs.startswith("'")
        is_fn = bool(re.match(r'^[A-Za-z_]\w*\(', rhs))
        if is_lit:
            literal_chains.append((line, var, n, rhs))
        elif is_fn:
            fn_call_chains.append((line, var, n, rhs))
        else:
            print(f"UNKNOWN: line={line} var={var} rhs={rhs!r}")

print(f"Total unique chains: {len(roots)}")
print(f"Literal RHS (can convert to match): {len(literal_chains)}")
print(f"Function call RHS (needs different approach): {len(fn_call_chains)}")

print("\n--- Literal chains (first 20) ---")
for line, var, n, rhs in literal_chains[:20]:
    print(f"  line={line:6d}  var={var:20s}  n={n:3d}  rhs_sample={rhs!r}")

print("\n--- Function-call chains (first 20) ---")
for line, var, n, rhs in fn_call_chains[:20]:
    print(f"  line={line:6d}  var={var:20s}  n={n:3d}  rhs_sample={rhs!r}")
