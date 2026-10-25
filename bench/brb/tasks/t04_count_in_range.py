from cases import I64_EDGES

ID = "t04-count-in-range"
NAME = "count_in_range"
PARAMS = [("a", "&[i64]"), ("lo", "i64"), ("hi", "i64")]
RET = "usize"
STATEMENT = "Return how many elements x of a satisfy lo <= x <= hi (both ends inclusive). If lo > hi the answer is 0."
DOMAIN = "a is any slice of i64 values; lo and hi are any i64 values."
BMB_DOMAIN = None


def domain(a, lo, hi):
    return True


def model(a, lo, hi):
    return sum(1 for x in a if lo <= x <= hi)


def cases(g):
    for a in g.arrays(250, 15):
        pool = a + [g.int(-4, 4), g.choice(I64_EDGES)]
        lo, hi = g.choice(pool), g.choice(pool)
        yield (a, min(lo, hi), max(lo, hi))
        if g.int(0, 3) == 0:
            yield (a, max(lo, hi), min(lo, hi))
    yield ([I64_EDGES[0], I64_EDGES[-1]], I64_EDGES[0], I64_EDGES[-1])


BMB_REF = """spec fn counted(c: usize, k: usize) -> bool { c <= k }

#[effects(pure)]
#[uses(counted)]
fn count_in_range(a: &[i64], lo: i64, hi: i64) -> (r: usize) {
    let mut c: usize = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), counted(c, k),
        decreases a.len() - k,
    {
        if lo <= a[k] && a[k] <= hi {
            c = c + 1;
        }
        k = k + 1;
    }
    c
}"""

# Weak: the upper end is exclusive.
BMB_WEAK = BMB_REF.replace("a[k] <= hi", "a[k] < hi")

RUST_REF = """pub fn count_in_range(a: &[i64], lo: i64, hi: i64) -> usize {
    a.iter().filter(|&&x| lo <= x && x <= hi).count()
}
"""

RUST_WEAK = """pub fn count_in_range(a: &[i64], lo: i64, hi: i64) -> usize {
    a.iter().filter(|&&x| x >= lo && x < hi).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts() {
        assert_eq!(count_in_range(&[1, 5, 9, 12], 0, 10), 3);
        assert_eq!(count_in_range(&[], 0, 10), 0);
        assert_eq!(count_in_range(&[3, 3], 5, 1), 0);
    }
}
"""

RUST_MUTANTS = [
    "pub fn count_in_range(a: &[i64], lo: i64, hi: i64) -> usize { a.iter().filter(|&&x| x >= lo && x < hi).count() }",
    "pub fn count_in_range(a: &[i64], lo: i64, hi: i64) -> usize { a.iter().filter(|&&x| x > lo && x <= hi).count() }",
    "pub fn count_in_range(a: &[i64], lo: i64, hi: i64) -> usize { a.iter().filter(|&&x| (x - lo) as u64 <= (hi - lo) as u64).count() }",
    "pub fn count_in_range(a: &[i64], lo: i64, hi: i64) -> usize { a.iter().filter(|&&x| lo <= x || x <= hi).count() }",
]
