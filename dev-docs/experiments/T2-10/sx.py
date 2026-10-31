import re, sys
def tok(s):
    return re.findall(r'\|[^|]*\||\(|\)|"[^"]*"|[^\s()]+', s)
def parse(s):
    ts = tok(s); pos = 0; out = []
    def rd():
        nonlocal pos
        t = ts[pos]; pos += 1
        if t == '(':
            l = []
            while ts[pos] != ')':
                l.append(rd())
            pos += 1
            return l
        return t
    while pos < len(ts):
        out.append(rd())
    return out
def show(e):
    if isinstance(e, list):
        return '(' + ' '.join(show(x) for x in e) + ')'
    return e
def strip_comments(s):
    return '\n'.join(l for l in s.split('\n') if not l.startswith(';'))
