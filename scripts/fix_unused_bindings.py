#!/usr/bin/env python3
"""
Fix unused_binding warnings in bootstrap/compiler.bmb
by prefixing unused variable declarations/parameters with '_'.

Handles two cases:
1. 'let var_name =' bindings: rename declaration only (var is truly unused)
2. Function parameters '(var: T' or ', var: T': rename param + all body usages
"""
import json
import subprocess
import re
import sys

DRY_RUN = '--dry-run' in sys.argv
COMPILER_PATH = 'bootstrap/compiler.bmb'

with open(COMPILER_PATH, 'rb') as f:
    content = f.read()

print(f"File size: {len(content)} bytes", flush=True)

result = subprocess.run(
    ['./target/release/bmb', 'check', COMPILER_PATH],
    capture_output=True, encoding='utf-8', errors='replace'
)

warnings = []
for src in (result.stdout, result.stderr):
    for line in src.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            w = json.loads(line)
            if w.get('type') == 'warning' and w.get('kind') == 'unused_binding':
                warnings.append(w)
        except json.JSONDecodeError:
            pass

print(f"Total unused_binding warnings: {len(warnings)}", flush=True)

def extract_var_name(message):
    m = re.search(r'unused (?:variable|binding): `([^`]+)`', message)
    return m.group(1) if m else None

FN_MARKER = b'\nfn '

# Two types of renames:
# let_renames: {byte_pos: var_name}  — rename 'let var ' at byte_pos
# param_renames: {byte_pos: (var_name, fn_end_pos)}  — rename param + all body usages
let_renames = {}
param_renames = {}
skipped_used = 0
not_found = 0
not_found_vars = {}

for w in warnings:
    var_name = extract_var_name(w.get('message', ''))
    if not var_name or var_name.startswith('_'):
        continue

    start_byte = w['start']

    # --- Try 'let var_name ' pattern first ---
    search_pattern = ('let ' + var_name + ' ').encode()
    idx = content[:start_byte].rfind(search_pattern)

    if idx >= 0:
        decl_end = idx + len(search_pattern)
        fn_end_pos = content.find(FN_MARKER, idx + 1)
        if fn_end_pos < 0:
            fn_end_pos = len(content)

        function_body = content[decl_end:fn_end_pos]
        var_bytes = var_name.encode()
        usage_pattern = re.compile(
            rb'(?<![a-zA-Z0-9_])' + re.escape(var_bytes) + rb'(?![a-zA-Z0-9_])'
        )
        if usage_pattern.search(function_body):
            skipped_used += 1
            continue

        if idx not in let_renames:
            let_renames[idx] = var_name
        continue

    # --- Try function parameter patterns ---
    # Look for '(var_name: ' or ', var_name: ' before start_byte
    param_patterns = [
        ('(' + var_name + ': ').encode(),
        (', ' + var_name + ': ').encode(),
        (' ' + var_name + ': ').encode(),
    ]
    param_idx = -1
    param_pat_len = 0
    var_offset_in_pat = 0

    for pat in param_patterns:
        idx2 = content[:start_byte].rfind(pat)
        if idx2 >= 0 and idx2 > param_idx:
            param_idx = idx2
            param_pat_len = len(pat)
            # Offset of var_name within the pattern
            var_offset_in_pat = pat.index(var_name.encode())

    if param_idx >= 0:
        # Find function end for body scope
        fn_end_pos = content.find(FN_MARKER, param_idx + 1)
        if fn_end_pos < 0:
            fn_end_pos = len(content)

        # Verify var_name is NOT used in the function body (it's a parameter)
        # Find function body start: '{' after the signature
        sig_end = content.find(b'{', param_idx)
        if sig_end < 0 or sig_end > fn_end_pos:
            sig_end = param_idx + 1
        fn_body = content[sig_end:fn_end_pos]

        # The actual position of var_name within the parameter pattern
        var_rename_pos = param_idx + var_offset_in_pat

        # Check if var is used in function body
        var_bytes = var_name.encode()
        usage_pattern = re.compile(
            rb'(?<![a-zA-Z0-9_])' + re.escape(var_bytes) + rb'(?![a-zA-Z0-9_])'
        )
        if usage_pattern.search(fn_body):
            skipped_used += 1
            continue

        if var_rename_pos not in param_renames:
            param_renames[var_rename_pos] = (var_name, fn_end_pos)
        continue

    not_found += 1
    not_found_vars[var_name] = not_found_vars.get(var_name, 0) + 1

print(f"Let renames identified:   {len(let_renames)}", flush=True)
print(f"Param renames identified: {len(param_renames)}", flush=True)
print(f"Skipped (var used):       {skipped_used}", flush=True)
print(f"Not found:                {not_found}", flush=True)
if not_found_vars:
    top = sorted(not_found_vars.items(), key=lambda x: -x[1])[:20]
    print(f"Not-found vars (top 20):  {top}", flush=True)

print("\n--- Let renames preview (first 10) ---", flush=True)
for pos, name in sorted(let_renames.items())[:10]:
    ls = content.rfind(b'\n', 0, pos) + 1
    le = content.find(b'\n', pos)
    dl = content[ls:le].decode('utf-8', errors='replace').rstrip()
    print(f"  byte={pos:7d}  {name!r:12s}  {dl[:70]}", flush=True)

print("\n--- Param renames preview (first 10) ---", flush=True)
for pos, (name, _) in sorted(param_renames.items())[:10]:
    ls = content.rfind(b'\n', 0, pos) + 1
    le = content.find(b'\n', pos)
    dl = content[ls:le].decode('utf-8', errors='replace').rstrip()
    print(f"  byte={pos:7d}  {name!r:12s}  {dl[:70]}", flush=True)

if DRY_RUN:
    print("\n[DRY RUN — no changes written]", flush=True)
    sys.exit(0)

# Build final content
result_content = bytearray(content)

# All renames are position-based; apply in reverse order
all_renames = []  # (pos, old_bytes, new_bytes)

for pos, var_name in let_renames.items():
    old = ('let ' + var_name + ' ').encode()
    new = ('let _' + var_name + ' ').encode()
    all_renames.append((pos, old, new))

for pos, (var_name, _) in param_renames.items():
    old = var_name.encode()
    new = ('_' + var_name).encode()
    all_renames.append((pos, old, new))

applied = 0
failed = 0
for pos, old_bytes, new_bytes in sorted(all_renames, key=lambda x: -x[0]):
    actual = bytes(result_content[pos:pos + len(old_bytes)])
    if actual == old_bytes:
        result_content[pos:pos + len(old_bytes)] = new_bytes
        applied += 1
    else:
        print(f"MISMATCH at {pos}: expected {old_bytes!r}, got {actual!r}", flush=True)
        failed += 1

print(f"\nApplied: {applied}, Failed: {failed}", flush=True)

with open(COMPILER_PATH, 'wb') as f:
    f.write(bytes(result_content))

print("Written successfully.", flush=True)
