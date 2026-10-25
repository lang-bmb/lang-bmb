from cases import U64_MAX

ID = "t13-rotate-right"
NAME = "rotate_right"
PARAMS = [("a", "&mut [i64]"), ("k", "usize")]
RET = "()"
STATEMENT = ("Rotate a to the right by k positions in place: the element at index i moves to index (i + k) mod a.len(). "
             "k may be larger than a.len(); an empty a stays empty.")
DOMAIN = "a is any slice of i64 values (possibly empty); k is any usize value (64-bit)."
BMB_DOMAIN = None


def domain(a, k):
    return 0 <= k <= U64_MAX


def model(a, k):
    n = len(a)
    if n:
        s = k % n
        a[:] = a[n - s:] + a[:n - s]


def cases(g):
    yield ([], 0)
    yield ([], 5)
    yield ([1, 2, 3], U64_MAX)
    for a in g.arrays(350, 12):
        n = len(a)
        k = g.choice([0, 1, n, n + 1, 2 * n + 1, g.int(0, 3 * n + 3), g.int(0, U64_MAX), U64_MAX, max(0, n - 1)])
        yield (a, k)


BMB_REF = """spec fn range_ok(a: &[i64], lo: usize, hi: usize) -> bool { lo <= hi && hi <= a.len() }

#[effects(pure)]
#[uses(range_ok)]
fn reverse_range(a: &mut [i64], lo: usize, hi: usize) -> ()
    requires range_ok(a, lo, hi),
{
    let mut i: usize = lo;
    let mut j: usize = hi;
    while i + 1 < j
        invariant i <= j, j <= a.len(),
        decreases j - i,
    {
        let t: i64 = a[i];
        a[i] = a[j - 1];
        a[j - 1] = t;
        i = i + 1;
        j = j - 1;
    }
}

#[effects(pure)]
#[uses(range_ok)]
fn rotate_right(a: &mut [i64], k: usize) -> () {
    let n: usize = a.len();
    if n > 0 {
        let s: usize = k % n;
        reverse_range(a, 0, n);
        reverse_range(a, 0, s);
        reverse_range(a, s, n);
    }
}"""

# Weak: the three reversals in the order that rotates left.
BMB_WEAK = BMB_REF.replace("""        reverse_range(a, 0, n);
        reverse_range(a, 0, s);
        reverse_range(a, s, n);""", """        reverse_range(a, 0, s);
        reverse_range(a, s, n);
        reverse_range(a, 0, n);""")

RUST_REF = """pub fn rotate_right(a: &mut [i64], k: usize) {
    if !a.is_empty() {
        let n = a.len();
        a.rotate_right(k % n);
    }
}
"""

RUST_WEAK = """pub fn rotate_right(a: &mut [i64], k: usize) {
    a.rotate_right(k);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotates() {
        let mut v = vec![1, 2, 3, 4];
        rotate_right(&mut v, 1);
        assert_eq!(v, vec![4, 1, 2, 3]);
        let mut w = vec![1, 2, 3];
        rotate_right(&mut w, 3);
        assert_eq!(w, vec![1, 2, 3]);
    }
}
"""

RUST_MUTANTS = [
    "pub fn rotate_right(a: &mut [i64], k: usize) { a.rotate_right(k); }",
    "pub fn rotate_right(a: &mut [i64], k: usize) { if !a.is_empty() { let n = a.len(); a.rotate_left(k % n); } }",
    "pub fn rotate_right(a: &mut [i64], k: usize) { let n = a.len(); a.rotate_right(k % n); }",
    "pub fn rotate_right(a: &mut [i64], k: usize) { if a.len() > 1 { let n = a.len(); a.rotate_right(k % (n - 1)); } }",
]
