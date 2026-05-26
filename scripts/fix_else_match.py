#!/usr/bin/env python3
"""Fix 'else match VAR { ... }' -> 'else { match VAR { ... } }' in compiler.bmb."""
import re

TARGET = 'bootstrap/compiler.bmb'

def find_brace_end(text, pos):
    """Find position after matching '}' given pos is after opening '{'."""
    depth = 1
    i = pos
    in_str = False
    sc = None
    while i < len(text):
        c = text[i]
        if in_str:
            if c == '\\':
                i += 2
                continue
            if c == sc:
                in_str = False
        elif c in ('"', "'"):
            in_str = True
            sc = c
        elif c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return i

with open(TARGET, 'rb') as f:
    content = f.read().decode('utf-8', errors='replace')

# Process from right to left to preserve offsets
pat = re.compile(r'(?<!\w)else (match \w+ \{)')
matches = list(pat.finditer(content))

print(f"Found {len(matches)} 'else match' patterns")

# Process in reverse order
result = content
for m in reversed(matches):
    # Find the opening brace position
    brace_offset = m.group(1).index('{')
    brace_pos = m.start(1) + brace_offset

    # Find matching closing '}'
    match_end = find_brace_end(result, brace_pos + 1)

    # Build replacement: 'else { match VAR { ... } }'
    match_expr = result[m.start(1):match_end]  # 'match VAR { ... }'
    old = result[m.start():match_end]           # 'else match VAR { ... }'
    new = 'else { ' + match_expr + ' }'

    print(f"  pos={m.start()}: {old[:50]!r} -> {new[:50]!r}")
    result = result[:m.start()] + new + result[match_end:]

with open(TARGET, 'wb') as f:
    f.write(result.encode('utf-8'))

print(f"\nFixed {len(matches)} patterns.")
