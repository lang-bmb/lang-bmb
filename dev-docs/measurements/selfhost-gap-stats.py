"""Self-hosting gap inventory of the Rust compiler (backlog 4A-01).

Usage: python3 -I dev-docs/measurements/selfhost-gap-stats.py compiler/src

Counts are regular-expression approximations over the non-test part of each
`compiler/src/*.rs` file (text before the first `#[cfg(test)]`, `//` comment
lines removed). They size the porting work; they are not an AST analysis.
Output: per-file sizes, one table row per construct (total and the five files
with the most uses), and a few derived counts. The numbers are quoted in
`dev-docs/measurements/selfhost-gap.md`.
"""

import glob
import os
import re
import sys
from collections import Counter

pats = {
    'String': r'\bString\b',
    '&str': r"&('[a-z]+ )?str\b",
    'char': r'\bchar\b',
    '.chars()/char_indices': r'\.(chars|char_indices)\(\)',
    'u8/bytes': r"\bu8\b|\.as_bytes\(\)|\.bytes\(\)|\bb'",
    'format!': r'\bformat!\(',
    'write!/writeln!': r'\bwrite(ln)?!\(',
    'to_string/to_owned/into': r'\.(to_string|to_owned)\(\)',
    'push_str/push(char)': r'\.push_str\(',
    'str methods': r'\.(starts_with|ends_with|strip_prefix|strip_suffix|split|trim|contains|lines|replace|split_once|parse|split_whitespace|to_lowercase|is_ascii_\w+|is_alphanumeric|is_whitespace)\b',
    'Vec<': r'\bVec<',
    'vec!': r'\bvec!\[',
    '.push(': r'\.push\(',
    'slice &[': r'&(mut )?\[',
    'BTreeMap': r'\bBTreeMap\b',
    'BTreeSet': r'\bBTreeSet\b',
    'HashMap': r'\bHashMap\b',
    'HashSet': r'\bHashSet\b',
    'VecDeque': r'\bVecDeque\b',
    'map ops(.get/.insert/.entry/.contains_key/.remove)': r'\.(get|get_mut|insert|entry|contains_key|remove|range)\(',
    'Option<': r'\bOption<',
    'Some(': r'\bSome\(',
    'None': r'\bNone\b',
    'Result<': r'\bResult<',
    'Ok(/Err(': r'\b(Ok|Err)\(',
    '? op': r'[\)\]\w]\?(?=[;\.\),\s])',
    'unwrap/expect': r'\.(unwrap|expect)\(',
    'unwrap_or*/map_or/ok_or': r'\.(unwrap_or|unwrap_or_else|unwrap_or_default|map_or|map_or_else|ok_or|ok_or_else|and_then|ok)\(',
    'Box<': r'\bBox<',
    'Box::new': r'\bBox::new\(',
    'Rc/RefCell/Cell/Arc/Mutex': r'\b(Rc|RefCell|Cell|Arc|Mutex)\b',
    'struct decl': r'^\s*(pub(\(crate\))? )?struct ',
    'enum decl': r'^\s*(pub(\(crate\))? )?enum ',
    'impl block': r'^\s*impl\b',
    'trait decl': r'^\s*(pub )?trait ',
    'impl Trait for': r'^\s*impl(<[^>]*>)? [\w:]+(<[^>]*>)? for ',
    '#[derive]': r'#\[derive',
    'fn decl': r'^\s*(pub(\(crate\))? )?fn ',
    'generic fn': r'\bfn \w+<',
    'lifetime': r"'[a-z]\b(?!')",
    'dyn/impl Fn|Iterator': r'\b(dyn|impl) (Fn|FnMut|FnOnce|Iterator)',
    'closure': r'[(,=]\s*(move\s+)?\|[^|\n]*\|',
    'move closure': r'\bmove \|',
    'match': r'\bmatch\b',
    'if let/while let/let-else': r'\bif let\b|\bwhile let\b|\blet [^=;]*=[^;]*\belse \{',
    'for loop': r'^\s*for .* in ',
    'while': r'^\s*while\b',
    'loop': r'^\s*(\'\w+: )?loop\b',
    'return': r'\breturn\b',
    'break/continue': r'\b(break|continue)\b',
    '.iter()/into_iter': r'\.(iter|iter_mut|into_iter|values|keys|drain)\(\)',
    'iter adapters': r'\.(map|filter|filter_map|flat_map|enumerate|zip|rev|any|all|find|position|fold|sum|max|min|count|take|skip|chain|peekable|windows|chunks|cloned|copied|last|max_by_key|min_by_key|take_while|skip_while|join)\(',
    'collect': r'\.collect(::<[^>]*>)?\(\)',
    'sort/dedup/binary_search': r'\.(sort|sort_by|sort_by_key|sort_unstable|sort_unstable_by|dedup|dedup_by|dedup_by_key|binary_search)\w*\(',
    'clone': r'\.clone\(\)',
    'as cast': r'\bas (u8|u16|u32|u64|i64|i32|usize|isize|char|i128|u128|f64)\b',
    'i128/u128': r'\b[iu]128\b',
    'u32': r'\bu32\b',
    'f64': r'\bf64\b',
    'bit ops(<< >> ^ & | ! rotate wrapping, excl. generics)': r'(\s<<\s|\s>>\s|\s\^\s|rotate_right|rotate_left|wrapping_\w+|\(\w+ & \w+\)|!\w+ &)',
    'checked_/saturating_': r'\.(checked|saturating|overflowing)_\w+\(',
    'std::fs': r'\bfs::\w+|\bFile::',
    'process (Command/Stdio/exit)': r'\b(Command|Stdio|process::exit|ExitCode)\b',
    'env (args/var)': r'\benv::(args|var|vars|var_os|current_dir|temp_dir)',
    'stdin/stdout/stderr': r'\b(stdin|stdout|stderr)\(\)',
    'print!/eprint!': r'\b(e?println|e?print)!\(',
    'thread': r'\bthread::\w+',
    'time (Instant/Duration)': r'\b(Instant|Duration|SystemTime)\b',
    'Path/PathBuf': r'\b(Path|PathBuf)\b',
    'io traits (Write/BufRead/Read)': r'\b(BufRead|BufReader|BufWriter)\b|\bio::(Write|Read)\b',
}


def load(path):
    full = open(path, encoding='utf-8').read()
    i = full.find('#[cfg(test)]')
    txt = full if i < 0 else full[:i]
    body = '\n'.join(l for l in txt.split('\n') if not l.strip().startswith('//'))
    nolit = re.sub(r'"((?:[^"\\]|\\.)*)"', '""', body)
    return full, txt, body, nolit


def fn_bodies(txt):
    for m in re.finditer(r'\bfn (\w+)\s*(<[^>]*>)?\s*\(', txt):
        j = txt.find('{', m.end())
        if j < 0:
            continue
        depth, k = 0, j
        while k < len(txt):
            if txt[k] == '{':
                depth += 1
            elif txt[k] == '}':
                depth -= 1
                if depth == 0:
                    break
            k += 1
        yield m.group(1), txt[j + 1:k]


def main():
    src = sys.argv[1]
    files = sorted(glob.glob(os.path.join(src, '*.rs')))
    names = [os.path.basename(f)[:-3] for f in files]
    rows, extra, inst, fmt_specs = {}, Counter(), Counter(), Counter()
    print('| 파일 | 전체 줄 | 테스트 제외 줄 | 바이트 | fn | 자기 재귀 fn |')
    print('| --- | --- | --- | --- | --- | --- |')
    tot = Counter()
    for f, n in zip(files, names):
        full, txt, body, nolit = load(f)
        rows[n] = {k: len(re.findall(p, body, re.M)) for k, p in pats.items()}
        fns = list(fn_bodies(txt))
        rec = sorted({name for name, b in fns
                      if re.search(r'(?<![\w.])(self\.|Self::)?' + re.escape(name) + r'\(', b)})
        row = (full.count('\n'), txt.count('\n'), len(full.encode()), len(fns), len(rec))
        for k, v in zip(('lines', 'non_test', 'bytes', 'fns', 'rec'), row):
            tot[k] += v
        print(f'| {n} | ' + ' | '.join(str(v) for v in row) + ' |')
        lits = re.findall(r'"((?:[^"\\]|\\.)*)"', body)
        extra['string literals'] += len(lits)
        extra['string literal bytes'] += sum(len(x) for x in lits)
        extra['method calls .f(..)'] += len(re.findall(r'\.[a-z_][a-z_0-9]*\s*(::<[^>]*>)?\(', nolit))
        extra['self.f(..) calls'] += len(re.findall(r'\bself\.[a-z_][a-z_0-9]*\(', nolit))
        extra['&self/&mut self methods'] += len(re.findall(r'fn \w+(<[^>]*>)?\(&(\'\w+ )?(mut )?self', nolit))
        extra['hex literals'] += len(re.findall(r'\b0x[0-9a-fA-F_]+', nolit))
        extra['char literals'] += len(re.findall(r"(?<![b\w])'(\\.|[^'\\])'", nolit))
        extra['byte literals'] += len(re.findall(r"\bb'(\\.|[^'\\])'", nolit))
        extra['slice ranges [a..b]'] += len(re.findall(r'\[[^\[\]]*\.\.[^\[\]]*\]', nolit))
        extra['wildcard arms _ =>'] += len(re.findall(r'(^|\|)\s*_\s*(if [^=]*)?=>', nolit, re.M))
        extra['let'] += len(re.findall(r'\blet\b', nolit))
        extra['let with type'] += len(re.findall(r'\blet\s+(mut\s+)?\w+\s*:', nolit))
        extra['matches!'] += len(re.findall(r'\bmatches!\(', nolit))
        extra['assert!/panic!/unreachable!'] += len(re.findall(r'\b(assert|assert_eq|panic|unreachable|debug_assert|todo)!\(', nolit))
        extra['executable enum ==/!= (X::Y)'] += len(re.findall(r'[=!]=\s*&?(Some\()?[A-Z]\w*::[A-Z]', nolit))
        extra['const/static items'] += len(re.findall(r'^\s*(pub )?(const|static) ', nolit, re.M))
        extra['std::mem take/replace/swap'] += len(re.findall(r'\bmem::(take|replace|swap)', nolit))
        for m in re.finditer(r'(format|write|writeln|println|eprintln|print|eprint)!\(\s*(?:[^,"]*,\s*)?"((?:[^"\\]|\\.)*)"', body):
            for s in re.findall(r'\{([^{}]*)\}', m.group(2)):
                fmt_specs[(s.split(':', 1)[1] if ':' in s else '') or '{}'] += 1
        for m in re.finditer(r'\b(BTreeMap|BTreeSet|Vec|Option)<', nolit):
            j, d = m.end(), 1
            while j < len(nolit) and d:
                d += {'<': 1, '>': -1}.get(nolit[j], 0)
                j += 1
            inst[(m.group(1), re.sub(r'\s+', ' ', nolit[m.end():j - 1]))] += 1
    print(f"| 합계 | {tot['lines']} | {tot['non_test']} | {tot['bytes']} | {tot['fns']} | {tot['rec']} |")
    print()
    print('| 구성 | 합계 | 많이 쓰는 파일(상위 5) |')
    print('| --- | --- | --- |')
    for k in pats:
        total = sum(rows[n][k] for n in names)
        top = sorted(((rows[n][k], n) for n in names if rows[n][k]), reverse=True)[:5]
        print(f'| {k} | {total} | ' + ', '.join(f'{n} {c}' for c, n in top) + ' |')
    print()
    for k, v in extra.items():
        print(f'{k}: {v}')
    print('format placeholders:', dict(fmt_specs.most_common()))
    vec_uses = [(t, v) for (k, t), v in inst.items() if k == 'Vec']
    non_copy = r'String|Vec|Expr|Block|Mod|FnDef|Diag|Value|J\b|Sexp|Param|Ident|Attr|Stmt|Type\b|&'
    print('Vec uses whose element is not Copy (String, tree node, reference; approx):',
          sum(v for t, v in vec_uses if re.search(non_copy, t)), 'of', sum(v for _, v in vec_uses))
    print('Option of a reference (Option<&..>):',
          sum(v for (k, t), v in inst.items() if k == 'Option' and t.startswith('&')))
    for kind in ('BTreeMap', 'BTreeSet', 'Vec', 'Option'):
        ks = sorted(((v, t) for (k, t), v in inst.items() if k == kind), reverse=True)
        print(f'{kind}: {len(ks)} distinct instantiations, {sum(v for v, _ in ks)} uses; top: '
              + '; '.join(f'{t} x{v}' for v, t in ks[:8]))


main()
