#!/usr/bin/env python3
"""T2-10: what keeps Z3 from deciding the undecided queries of a check.

usage: ablate.py OUT CHECK     (OUT as measure.sh wrote it, CHECK vac or pre)

Takes the final undecided queries (`CHECK.log` rows with the check's own limit
and answer unknown), one per distinct text, removes one kind of assertion at a
time and solves again under rlimit 1,000,000:
  range  the element-range quantifiers of arrays (`:pattern ((select a k))`)
  def    the quantified definitions of non-recursive spec fns
  rec    the quantified definitions of recursive spec fns
  defs   both, all    both and the ranges
A query is attributed to the first removal that lets Z3 decide; one no
removal decides is attributed to its arithmetic (`bv2nat` terms, else plain).
Removing assertions weakens the query, so this attributes the cost; it decides
nothing about the query itself."""
import collections, hashlib, json, os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sx import parse, show  # noqa: E402

LIMIT = {"vac": 1000000, "pre": 5000000}


def kinds(cmd):
    if cmd[0] != 'assert':
        return 'cmd'
    a = cmd[1]
    if isinstance(a, list) and a[0] == 'forall' and isinstance(a[2], list) and a[2][0] == '!':
        pat = show(a[2][3]) if len(a[2]) > 3 else ''
        m = re.match(r'\(\((\|[^|]+::[^|]+\|)', pat)
        if m:
            return 'rec' if show(a[2][1]).count(m.group(1)) > 1 else 'def'
        if pat.startswith('((select'):
            return 'range'
    return 'fact'


def solve(cmds, limit=1000000):
    txt = '\n'.join(show(c) for c in cmds if not (c[0] == 'set-option' and c[1] == ':rlimit') and c[0] != 'check-sat')
    txt = f'(set-option :rlimit {limit})\n{txt}\n(check-sat)\n(get-info :rlimit)\n'
    out = subprocess.run(['z3', '-in'], input=txt, capture_output=True, text=True).stdout
    m = re.search(r':rlimit (\d+)', out)
    return (out.split() or ['?'])[0], int(m.group(1)) if m else -1


def ablate(text):
    cmds = [c for c in parse('\n'.join(l for l in text.split('\n') if not l.startswith(';'))) if c[0] != 'reset']
    ks = [kinds(c) for c in cmds]
    have = collections.Counter(ks)
    res = {}
    for name, what in [('range', ('range',)), ('def', ('def',)), ('rec', ('rec',)),
                       ('defs', ('def', 'rec')), ('all', ('def', 'rec', 'range'))]:
        if any(have[w] for w in what):
            res[name] = solve([c for c, k in zip(cmds, ks) if k not in what])
    decided = lambda k: k in res and res[k][0] in ('sat', 'unsat')  # noqa: E731
    if decided('range'):
        cause = 'element ranges'
    elif decided('def') and not have['rec']:
        cause = 'non-recursive definitions'
    elif decided('rec') and not have['def']:
        cause = 'recursive definitions'
    elif decided('def'):
        cause = 'non-recursive definitions (recursive present)'
    elif decided('rec'):
        cause = 'recursive definitions (non-recursive present)'
    elif decided('defs'):
        cause = 'definitions (both kinds)'
    elif decided('all'):
        cause = 'definitions and ranges'
    else:
        cause = 'arithmetic: bit-vector' if 'bv2nat' in text or 'int2bv' in text else 'arithmetic'
    return cause, res


def main():
    out, chk = sys.argv[1], sys.argv[2]
    rows = [json.loads(line) for line in open(f'{out}/{chk}.log')]
    groups = {}
    for r in rows:
        if r['limit'] == LIMIT[chk] and r['answer'] == 'unknown':
            body = open(r['file']).read().split('\n', 1)[1]
            g = groups.setdefault(hashlib.sha256(body.encode()).hexdigest(), {'body': body, 'rows': []})
            g['rows'].append(r)
    with ThreadPoolExecutor(int(os.environ.get('JOBS', '2'))) as ex:
        results = list(ex.map(lambda g: ablate(g['body']), groups.values()))
    count, cost = collections.Counter(), collections.Counter()
    for g, (cause, res) in zip(groups.values(), results):
        count[cause] += len(g['rows'])
        cost[cause] += sum(r['rlimit'] for r in g['rows'])
        print(json.dumps({'cause': cause, 'copies': len(g['rows']), 'label': g['rows'][0]['label'],
                          'rlimit': g['rows'][0]['rlimit'], 'ablations': res}))
    print(f'# {chk}: {sum(count.values())} undecided, {len(groups)} distinct')
    for c in sorted(count, key=lambda c: -cost[c]):
        print(f'# {c}: {count[c]} undecided, rlimit {cost[c]:,}')


if __name__ == '__main__':
    main()
