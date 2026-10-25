from collections import Counter

from cases import I64_MIN

ID = "t20-majority"
NAME = "majority"
PARAMS = [("a", "&[i64]")]
RET = "i64"
STATEMENT = "Return the value that occurs in more than half of the positions of a (strictly more than a.len() / 2 times), or i64::MIN if no value does."
DOMAIN = "a is any slice of i64 values (possibly empty)."
BMB_DOMAIN = None


def domain(a):
    return True


def model(a):
    for v, c in Counter(a).items():
        if 2 * c > len(a):
            return v
    return I64_MIN


def cases(g):
    yield ([],)
    yield ([1, 2],)
    yield ([1, 1, 2, 2],)
    yield ([3, 3, 4],)
    for _ in range(350):
        e = g.elem_mixed()
        n = g.int(0, 15)
        if g.int(0, 1):
            v = e()
            a = [v if g.int(0, 2) else e() for _ in range(n)]
        else:
            a = [e() for _ in range(n)]
        yield (a,)


BMB_REF = """spec fn count_le(c: usize, k: usize) -> bool { c <= k }

#[effects(pure)]
#[uses(count_le)]
fn majority(a: &[i64]) -> (r: i64) {
    let mut cand: i64 = 0;
    let mut cnt: usize = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), count_le(cnt, k),
        decreases a.len() - k,
    {
        if cnt == 0 {
            cand = a[k];
            cnt = 1;
        } else {
            if a[k] == cand { cnt = cnt + 1; } else { cnt = cnt - 1; }
        }
        k = k + 1;
    }
    let mut c: usize = 0;
    let mut i: usize = 0;
    while i < a.len()
        invariant i <= a.len(), count_le(c, i),
        decreases a.len() - i,
    {
        if a[i] == cand {
            c = c + 1;
        }
        i = i + 1;
    }
    if c > a.len() / 2 { cand } else { -9223372036854775807 - 1 }
}"""

# Weak: trusts the Boyer-Moore candidate without counting it.
BMB_WEAK = """spec fn count_le(c: usize, k: usize) -> bool { c <= k }

#[effects(pure)]
#[uses(count_le)]
fn majority(a: &[i64]) -> (r: i64) {
    let mut cand: i64 = 0;
    let mut cnt: usize = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), count_le(cnt, k),
        decreases a.len() - k,
    {
        if cnt == 0 {
            cand = a[k];
            cnt = 1;
        } else {
            if a[k] == cand { cnt = cnt + 1; } else { cnt = cnt - 1; }
        }
        k = k + 1;
    }
    if cnt > 0 { cand } else { -9223372036854775807 - 1 }
}"""

RUST_REF = """pub fn majority(a: &[i64]) -> i64 {
    let (mut cand, mut cnt) = (0i64, 0usize);
    for &x in a {
        if cnt == 0 {
            cand = x;
            cnt = 1;
        } else if x == cand {
            cnt += 1;
        } else {
            cnt -= 1;
        }
    }
    let c = a.iter().filter(|&&x| x == cand).count();
    if c > a.len() / 2 { cand } else { i64::MIN }
}
"""

RUST_WEAK = """pub fn majority(a: &[i64]) -> i64 {
    let (mut cand, mut cnt) = (0i64, 0usize);
    for &x in a {
        if cnt == 0 {
            cand = x;
            cnt = 1;
        } else if x == cand {
            cnt += 1;
        } else {
            cnt -= 1;
        }
    }
    if cnt > 0 { cand } else { i64::MIN }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn majorities() {
        assert_eq!(majority(&[3, 3, 4, 3]), 3);
        assert_eq!(majority(&[1, 2]), i64::MIN);
        assert_eq!(majority(&[]), i64::MIN);
        assert_eq!(majority(&[5]), 5);
    }
}
"""

RUST_MUTANTS = [
    "pub fn majority(a: &[i64]) -> i64 { let (mut cand, mut cnt) = (0i64, 0usize); for &x in a { if cnt == 0 { cand = x; cnt = 1; } else if x == cand { cnt += 1; } else { cnt -= 1; } } if cnt > 0 { cand } else { i64::MIN } }",
    "pub fn majority(a: &[i64]) -> i64 { let (mut cand, mut cnt) = (0i64, 0usize); for &x in a { if cnt == 0 { cand = x; cnt = 1; } else if x == cand { cnt += 1; } else { cnt -= 1; } } let c = a.iter().filter(|&&x| x == cand).count(); if c >= a.len() / 2 && c > 0 { cand } else { i64::MIN } }",
    "pub fn majority(a: &[i64]) -> i64 { let mut v = a.to_vec(); v.sort(); if v.is_empty() { i64::MIN } else { v[v.len() / 2] } }",
    "pub fn majority(a: &[i64]) -> i64 { let (mut cand, mut cnt) = (0i64, 0usize); for &x in a { if cnt == 0 { cand = x; cnt = 1; } else if x == cand { cnt += 1; } else { cnt -= 1; } } let c = a.iter().filter(|&&x| x == cand).count(); if 2 * c >= a.len() && c > 0 { cand } else { i64::MIN } }",
]
