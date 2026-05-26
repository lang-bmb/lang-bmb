#!/usr/bin/env python3
"""Analyze chained_comparison warnings from bmb check output on stdin."""
import sys
import json
import re
from collections import defaultdict

warnings = []
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        w = json.loads(line)
        if w.get('type') == 'warning' and w.get('kind') == 'chained_comparison':
            warnings.append(w)
    except Exception:
        pass

# Group by 'end' byte — each unique chain shares the same end position
groups = defaultdict(list)
for w in warnings:
    groups[w['end']].append(w)

print(f"Total chained_comparison warnings: {len(warnings)}")
print(f"Unique chains (by end-byte): {len(groups)}")

# For each group, find the root (smallest start = earliest = largest chain)
roots = []
for end, ws in groups.items():
    root = min(ws, key=lambda x: x['start'])
    m = re.search(r'(\d+) chained', root['message'])
    n = int(m.group(1)) if m else 0
    var = re.search(r'on `([^`]+)`', root['message'])
    v = var.group(1) if var else '?'
    roots.append((root['line'], v, n, root['start'], end))

roots.sort()
print("\nTop 20 chains by line:")
for line, var, n, start, end in roots[:20]:
    print(f"  line={line:6d}  var={var:20s}  chain_len={n:3d}  bytes={start}-{end}")

# Variable name frequency
var_freq = defaultdict(int)
for _, var, n, _, _ in roots:
    var_freq[var] += 1
print("\nTop variable names:")
for var, cnt in sorted(var_freq.items(), key=lambda x: -x[1])[:20]:
    print(f"  {var:30s}: {cnt} chains")
