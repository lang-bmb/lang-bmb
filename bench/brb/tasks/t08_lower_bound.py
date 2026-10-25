import bisect

from cases import I64_EDGES

ID = "t08-lower-bound"
NAME = "lower_bound"
PARAMS = [("a", "&[i64]"), ("t", "i64")]
RET = "usize"
STATEMENT = "a is sorted in non-decreasing order. Return the smallest index i with a[i] >= t, or a.len() if every element is smaller than t. Use binary search."
DOMAIN = "a is any slice of i64 values sorted in non-decreasing order (possibly empty, duplicates allowed); t is any i64 value."
BMB_DOMAIN = "spec fn dom(a: &[i64], t: i64) -> bool { forall|i: usize, j: usize| i < j && j < a.len() ==> #[trigger] a[i] <= #[trigger] a[j] }"


def domain(a, t):
    return all(a[i] <= a[i + 1] for i in range(len(a) - 1))


def model(a, t):
    return bisect.bisect_left(a, t)


def cases(g):
    yield ([], 0)
    for _ in range(350):
        a = g.sorted_array(16, g.elem_mixed())
        pool = a + [g.int(-4, 4), g.choice(I64_EDGES)]
        t = g.choice(pool)
        yield (a, g.choice([t, t, t + 1 if t < I64_EDGES[-1] else t, t - 1 if t > I64_EDGES[0] else t]))


BMB_REF = """#[effects(pure)]
fn lower_bound(a: &[i64], t: i64) -> (r: usize) {
    let mut lo: usize = 0;
    let mut hi: usize = a.len();
    while lo < hi
        invariant lo <= hi, hi <= a.len(),
        decreases hi - lo,
    {
        let mid: usize = lo + (hi - lo) / 2;
        if a[mid] < t {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}"""

# Weak: `<=` finds the upper bound (right only without duplicates of t).
BMB_WEAK = BMB_REF.replace("a[mid] < t", "a[mid] <= t")

RUST_REF = """pub fn lower_bound(a: &[i64], t: i64) -> usize {
    a.partition_point(|&x| x < t)
}
"""

RUST_WEAK = """pub fn lower_bound(a: &[i64], t: i64) -> usize {
    match a.binary_search(&t) {
        Ok(i) => i,
        Err(i) => i,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds() {
        assert_eq!(lower_bound(&[1, 3, 5, 7], 5), 2);
        assert_eq!(lower_bound(&[1, 3, 5, 7], 4), 2);
        assert_eq!(lower_bound(&[1, 3, 5, 7], 9), 4);
        assert_eq!(lower_bound(&[], 9), 0);
    }
}
"""

RUST_MUTANTS = [
    "pub fn lower_bound(a: &[i64], t: i64) -> usize { match a.binary_search(&t) { Ok(i) => i, Err(i) => i } }",
    "pub fn lower_bound(a: &[i64], t: i64) -> usize { a.partition_point(|&x| x <= t) }",
    "pub fn lower_bound(a: &[i64], t: i64) -> usize { a.iter().position(|&x| x > t).unwrap_or(a.len()) }",
    "pub fn lower_bound(a: &[i64], t: i64) -> usize { let (mut lo, mut hi) = (0, a.len()); while lo + 1 < hi { let mid = (lo + hi) / 2; if a[mid] < t { lo = mid } else { hi = mid } } hi }",
]
