#!/usr/bin/env python3
"""Convert literal chained_comparison chains to match expressions in bootstrap/compiler.bmb."""
import sys
import json
import re
import subprocess
from collections import defaultdict

PRELUDE_LINES = 102
TARGET = 'bootstrap/compiler.bmb'

# ── Parsing helpers ──────────────────────────────────────────────────────────

def find_brace_end(text, pos):
    """Given text where pos is immediately AFTER opening '{', return position after '}'."""
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


def skip_ws(text, pos):
    while pos < len(text):
        c = text[pos]
        if c in (' ', '\t', '\n', '\r'):
            pos += 1
        elif text[pos:pos+2] == '//':
            while pos < len(text) and text[pos] not in ('\n', '\r'):
                pos += 1
        else:
            break
    return pos


def parse_literal(text, pos):
    """Return (literal_string, new_pos) or (None, pos) on failure.
    Also handles TK_XYZ() calls by substituting their integer values."""
    if pos >= len(text):
        return None, pos
    if text[pos] == '"':
        end = pos + 1
        while end < len(text):
            if text[end] == '\\':
                end += 2
                continue
            if text[end] == '"':
                end += 1
                break
            end += 1
        return text[pos:end], end
    if text[pos] == "'":
        end = pos + 1
        if end < len(text) and text[end] == '\\':
            end += 2
        else:
            end += 1
        if end < len(text):
            end += 1
        return text[pos:end], end
    if text[pos] == '-' or text[pos].isdigit():
        end = pos
        if text[end] == '-':
            end += 1
        while end < len(text) and text[end].isdigit():
            end += 1
        return text[pos:end], end
    if text[pos:pos+4] == 'true' and (pos+4 >= len(text) or not (text[pos+4].isalnum() or text[pos+4] == '_')):
        return 'true', pos + 4
    if text[pos:pos+5] == 'false' and (pos+5 >= len(text) or not (text[pos+5].isalnum() or text[pos+5] == '_')):
        return 'false', pos + 5
    # TK_*() function call → substitute integer value
    tk_m = re.match(r'(TK_\w+)\(\)', text[pos:])
    if tk_m:
        name = tk_m.group(1)
        if name in tk_map:
            return str(tk_map[name]), pos + len(tk_m.group(0))
    return None, pos


def strip_trailing_comment(body):
    """Strip trailing // comment from the last line of body (preserves string literals)."""
    lines = body.split('\n')
    last = lines[-1]
    in_str = False
    sc = None
    i = 0
    while i < len(last):
        c = last[i]
        if in_str:
            if c == '\\':
                i += 2
                continue
            if c == sc:
                in_str = False
        elif c in ('"', "'"):
            in_str = True
            sc = c
        elif last[i:i+2] == '//':
            lines[-1] = last[:i].rstrip()
            return '\n'.join(lines)
        i += 1
    return body


def parse_chain(text, var):
    """
    Parse: if var == LIT { BODY } else if ... else { BODY }
    Returns (arms, consumed) where arms = [(lit_or_None, body), ...].
    Bug fix: rollback pos to 'else' start when compound condition (and/or) detected.
    """
    arms = []
    pos = 0
    last_else_start = None  # position of 'else' before 'else if'
    while pos < len(text):
        pos = skip_ws(text, pos)
        if pos >= len(text):
            break

        if text[pos:pos+2] == 'if':
            # Require whitespace after 'if'
            if pos + 2 < len(text) and text[pos+2] not in (' ', '\t', '\n'):
                # rollback to else start if we came from 'else if'
                if last_else_start is not None:
                    pos = last_else_start
                break
            if_start = pos
            pos = skip_ws(text, pos + 2)
            # Match var name (exact word boundary)
            if not text[pos:].startswith(var):
                if last_else_start is not None:
                    pos = last_else_start
                break
            end_var = pos + len(var)
            if end_var < len(text) and (text[end_var].isalnum() or text[end_var] == '_'):
                if last_else_start is not None:
                    pos = last_else_start
                break
            pos = skip_ws(text, end_var)
            if text[pos:pos+2] != '==':
                if last_else_start is not None:
                    pos = last_else_start
                break
            pos = skip_ws(text, pos + 2)
            lit, pos = parse_literal(text, pos)
            if lit is None:
                if last_else_start is not None:
                    pos = last_else_start
                break
            pos = skip_ws(text, pos)
            if pos >= len(text) or text[pos] != '{':
                # Compound condition (and/or) — rollback to 'else' start
                if last_else_start is not None:
                    pos = last_else_start
                else:
                    pos = if_start
                break
            body_end = find_brace_end(text, pos + 1)
            body = text[pos+1:body_end-1].strip()
            pos = body_end
            arms.append((lit, body))
            last_else_start = None  # reset after successful arm

        elif text[pos:pos+4] == 'else':
            if pos + 4 < len(text) and text[pos+4] not in (' ', '\t', '\n'):
                break
            last_else_start = pos  # save 'else' position for rollback
            pos = skip_ws(text, pos + 4)
            if text[pos:pos+2] == 'if':
                continue  # let next iteration handle 'if var == LIT { body }'
            if pos < len(text) and text[pos] == '{':
                body_end = find_brace_end(text, pos + 1)
                body = text[pos+1:body_end-1].strip()
                pos = body_end
                arms.append((None, body))
                break
            break
        else:
            break

    return arms, pos


def build_match(var, arms):
    parts = []
    for lit, body in arms:
        pat = '_' if lit is None else lit
        # Strip trailing // comment to prevent it from absorbing arm separator
        safe_body = strip_trailing_comment(body)
        parts.append(f"{pat} => {safe_body}")
    return f"match {var} {{ {', '.join(parts)} }}"


def is_literal_rhs(rhs):
    if not rhs:
        return False
    if rhs.startswith('"') or rhs.startswith("'"):
        return True
    if re.match(r'^-?\d+$', rhs):
        return True
    if rhs in ('true', 'false'):
        return True
    # TK_*() calls are treated as literals (substituted with integers)
    if re.match(r'^TK_\w+\(\)$', rhs) and rhs[:-2] in tk_map:
        return True
    return False


def is_fn_call_rhs(rhs):
    return bool(re.match(r'^[A-Za-z_]\w*\(', rhs)) and not (re.match(r'^TK_\w+\(\)$', rhs) and rhs[:-2] in tk_map)


# ── Load file ────────────────────────────────────────────────────────────────

with open(TARGET, 'rb') as f:
    content_bytes = f.read()
content = content_bytes.decode('utf-8')

# Build line → byte-offset map
line_offsets = [0]
for i, b in enumerate(content_bytes):
    if b == ord('\n'):
        line_offsets.append(i + 1)

def line_start(n):
    """Byte offset where file line n (1-indexed) starts."""
    if n - 1 < len(line_offsets):
        return line_offsets[n - 1]
    return len(content_bytes)


# ── Extract TK_* integer values ───────────────────────────────────────────────

tk_map = {}
_tk_pat = re.compile(r'fn (TK_\w+)\(\)\s*->\s*i64.*?=\s*2000000000\s*\+\s*(\d+)\s*;', re.DOTALL)
for name, offset in _tk_pat.findall(content):
    tk_map[name] = 2000000000 + int(offset)
print(f"TK_* functions found: {len(tk_map)}")


# ── Get warnings ─────────────────────────────────────────────────────────────

result = subprocess.run(
    ['./target/release/bmb', 'check', TARGET],
    capture_output=True, encoding='utf-8', errors='replace'
)

warnings = []
for line in (result.stdout + result.stderr).splitlines():
    try:
        w = json.loads(line.strip())
        if w.get('kind') == 'chained_comparison':
            warnings.append(w)
    except:
        pass

print(f"Total chained_comparison warnings: {len(warnings)}")

# Group by end byte
groups = defaultdict(list)
for w in warnings:
    groups[w['end']].append(w)

# For each group, get root (outermost warning = min start)
raw_chains = []
for end_pp, ws in groups.items():
    root = min(ws, key=lambda x: x['start'])
    m = re.search(r'(\d+) chained', root['message'])
    count = int(m.group(1)) if m else 0
    v = re.search(r'on `([^`]+)`', root['message'])
    var = v.group(1) if v else '?'
    file_line_approx = root['line'] - PRELUDE_LINES
    raw_chains.append({
        'var': var,
        'count': count,
        'file_line_approx': file_line_approx,
        'warning_line': root['line'],
    })

print(f"Unique chains: {len(raw_chains)}")

# ── Find actual chain start byte for each chain ───────────────────────────────

def find_chain_start_byte(chain):
    """Locate the 'if var ==' in the file. Returns byte index or None."""
    var = chain['var']
    pat = re.compile(r'\bif\s+' + re.escape(var) + r'\s*==')

    # Search by line number with wider window (±50 lines)
    approx_line = chain['file_line_approx']
    for ln in range(max(1, approx_line - 50), min(len(line_offsets), approx_line + 50)):
        ls = line_start(ln)
        le = line_start(ln + 1) if ln + 1 <= len(line_offsets) else len(content)
        line_text = content[ls:le]
        m = pat.search(line_text)
        if m:
            return ls + m.start()

    return None


# Augment chains with actual start positions
resolved = []
for chain in raw_chains:
    start = find_chain_start_byte(chain)
    if start is None:
        print(f"  SKIP: could not find chain for var={chain['var']} near line ~{chain['file_line_approx']}")
        continue
    chain['actual_start'] = start
    resolved.append(chain)

# ── Classify as literal vs fn-call ───────────────────────────────────────────

for chain in resolved:
    start = chain['actual_start']
    seg = content[start:start + 200]
    pat = re.compile(r'if\s+' + re.escape(chain['var']) + r'\s*==\s*([^\s{(]+(?:\(\))?)')
    m = pat.search(seg)
    rhs = m.group(1) if m else ''
    chain['rhs'] = rhs
    chain['is_literal'] = is_literal_rhs(rhs)
    chain['is_fn_call'] = is_fn_call_rhs(rhs)

literal_chains = [c for c in resolved if c['is_literal']]
fn_chains = [c for c in resolved if c['is_fn_call']]
unknown = [c for c in resolved if not c['is_literal'] and not c['is_fn_call']]

print(f"Literal chains: {len(literal_chains)}")
print(f"Fn-call chains: {len(fn_chains)}")
if unknown:
    for c in unknown:
        print(f"  UNKNOWN: var={c['var']} rhs={c['rhs']!r}")

# ── Parse literal chains and compute actual end ───────────────────────────────

parsed = []
for chain in literal_chains:
    start = chain['actual_start']
    seg = content[start:start + 200000]
    arms, consumed = parse_chain(seg, chain['var'])
    if len(arms) < 2:
        print(f"  PARSE FAIL: var={chain['var']} start={start} arms={len(arms)}")
        continue
    # Require final else arm (becomes _ => in match); without it, remaining code would dangle
    has_else = arms[-1][0] is None
    if not has_else:
        print(f"  SKIP (no else): var={chain['var']} start={start} arms={len(arms)} — remaining code would dangle")
        continue
    chain['arms'] = arms
    chain['actual_end'] = start + consumed
    parsed.append(chain)

print(f"Parsed literal chains: {len(parsed)}")

# ── Deduplicate: remove inner chains that are contained in outer chains ────────

parsed.sort(key=lambda c: c['actual_start'])

def is_contained(inner, outer):
    return (outer['actual_start'] <= inner['actual_start'] and
            inner['actual_end'] <= outer['actual_end'])

kept = []
for i, chain in enumerate(parsed):
    dominated = False
    for other in parsed:
        if other is chain:
            continue
        if is_contained(chain, other) and other['count'] > chain['count']:
            dominated = True
            break
    if dominated:
        print(f"  SKIP (inner): var={chain['var']} count={chain['count']} start={chain['actual_start']}")
    else:
        kept.append(chain)

print(f"After dedup: {len(kept)} chains to convert")

# ── Build replacements and apply in reverse order ────────────────────────────

replacements = []
for chain in kept:
    match_text = build_match(chain['var'], chain['arms'])
    replacements.append((chain['actual_start'], chain['actual_end'], match_text))
    print(f"  var={chain['var']:20s} arms={len(chain['arms']):3d} "
          f"start={chain['actual_start']:6d} end={chain['actual_end']:6d}")

replacements.sort(key=lambda x: x[0], reverse=True)

new_content = content
for start, end, new_text in replacements:
    new_content = new_content[:start] + new_text + new_content[end:]

with open(TARGET, 'wb') as f:
    f.write(new_content.encode('utf-8'))

print(f"\nDone. Applied {len(replacements)} replacements.")
