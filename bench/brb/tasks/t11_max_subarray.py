ID = "t11-max-subarray"
NAME = "max_subarray"
PARAMS = [("a", "&[i64]")]
RET = "i64"
STATEMENT = "Return the largest sum of a non-empty contiguous run a[i], a[i + 1], ..., a[j] (i <= j) of a."
DOMAIN = "a is a non-empty slice with a.len() <= 1_000_000 whose elements all lie in [-1_000_000_000, 1_000_000_000]."
BMB_DOMAIN = ("spec fn dom(a: &[i64]) -> bool { a.len() > 0 && a.len() <= 1000000 && "
              "(forall|k: usize| k < a.len() ==> -1000000000 <= #[trigger] a[k] && a[k] <= 1000000000) }")

LIM = 1000000000


def domain(a):
    return 0 < len(a) <= 1000000 and all(-LIM <= x <= LIM for x in a)


def model(a):
    best = cur = a[0]
    for x in a[1:]:
        cur = max(x, cur + x)
        best = max(best, cur)
    return best


def cases(g):
    for _ in range(400):
        k = g.int(0, 3)
        if k == 0:
            e = g.small_elem(5)
        elif k == 1:
            e = (lambda: g.int(-LIM, -1))
        elif k == 2:
            e = (lambda: g.choice([-LIM, LIM, -1, 0, 1]))
        else:
            e = (lambda: g.int(-LIM, LIM))
        yield (g.array(18, e),)


BMB_REF = """spec fn elems_ok(a: &[i64]) -> bool {
    a.len() > 0 && a.len() <= 1000000 && (forall|k: usize| k < a.len() ==> -1000000000 <= #[trigger] a[k] && a[k] <= 1000000000)
}

spec fn bounded(v: i64, k: usize) -> bool { -1000000000 <= v && v <= 1000000000 * k }

#[effects(pure)]
#[uses(elems_ok, bounded)]
fn max_subarray(a: &[i64]) -> (r: i64)
    requires elems_ok(a),
{
    let mut cur: i64 = a[0];
    let mut best: i64 = a[0];
    let mut k: usize = 1;
    while k < a.len()
        invariant 1 <= k, k <= a.len(), elems_ok(a), bounded(cur, k), bounded(best, k),
        decreases a.len() - k,
    {
        let x: i64 = a[k];
        if cur > 0 {
            cur = cur + x;
        } else {
            cur = x;
        }
        if cur > best {
            best = cur;
        }
        k = k + 1;
    }
    best
}"""

# Weak: the best sum starts at 0 (wrong when every element is negative).
BMB_WEAK = """spec fn elems_ok(a: &[i64]) -> bool {
    a.len() > 0 && a.len() <= 1000000 && (forall|k: usize| k < a.len() ==> -1000000000 <= #[trigger] a[k] && a[k] <= 1000000000)
}

spec fn bounded0(v: i64, k: usize) -> bool { 0 <= v && v <= 1000000000 * k }

#[effects(pure)]
#[uses(elems_ok, bounded0)]
fn max_subarray(a: &[i64]) -> (r: i64)
    requires elems_ok(a),
{
    let mut cur: i64 = 0;
    let mut best: i64 = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), elems_ok(a), bounded0(cur, k), bounded0(best, k),
        decreases a.len() - k,
    {
        cur = cur + a[k];
        if cur < 0 {
            cur = 0;
        }
        if cur > best {
            best = cur;
        }
        k = k + 1;
    }
    best
}"""

RUST_REF = """pub fn max_subarray(a: &[i64]) -> i64 {
    let mut cur = a[0];
    let mut best = a[0];
    for &x in &a[1..] {
        cur = if cur > 0 { cur + x } else { x };
        best = best.max(cur);
    }
    best
}
"""

RUST_WEAK = """pub fn max_subarray(a: &[i64]) -> i64 {
    let mut cur = 0;
    let mut best = 0;
    for &x in a {
        cur = (cur + x).max(0);
        best = best.max(cur);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sums() {
        assert_eq!(max_subarray(&[-2, 1, -3, 4, -1, 2, 1, -5, 4]), 6);
        assert_eq!(max_subarray(&[5]), 5);
        assert_eq!(max_subarray(&[1, 2, 3]), 6);
    }
}
"""

RUST_MUTANTS = [
    "pub fn max_subarray(a: &[i64]) -> i64 { let mut cur = 0; let mut best = 0; for &x in a { cur = (cur + x).max(0); best = best.max(cur); } best }",
    "pub fn max_subarray(a: &[i64]) -> i64 { *a.iter().max().unwrap() }",
    "pub fn max_subarray(a: &[i64]) -> i64 { let s: i64 = a.iter().filter(|&&x| x > 0).sum(); if s > 0 { s } else { *a.iter().max().unwrap() } }",
    "pub fn max_subarray(a: &[i64]) -> i64 { let mut cur = a[0]; let mut best = a[0]; for &x in a { cur = if cur > 0 { cur + x } else { x }; best = best.max(cur); } best }",
]
