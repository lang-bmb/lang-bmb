ID = "t14-dedup-sorted"
NAME = "dedup_sorted"
PARAMS = [("a", "&mut [i64]")]
RET = "usize"
STATEMENT = ("a is sorted in non-decreasing order. In place, move the distinct values of a to the front in increasing order, "
             "set every remaining position to 0, and return the number of distinct values.")
DOMAIN = "a is any slice of i64 values sorted in non-decreasing order (possibly empty)."
BMB_DOMAIN = "spec fn dom(a: &[i64]) -> bool { forall|i: usize, j: usize| i < j && j < a.len() ==> #[trigger] a[i] <= #[trigger] a[j] }"


def domain(a):
    return all(a[i] <= a[i + 1] for i in range(len(a) - 1))


def model(a):
    d = sorted(set(a))
    a[:] = d + [0] * (len(a) - len(d))
    return len(d)


def cases(g):
    yield ([],)
    for _ in range(350):
        yield (g.sorted_array(14, g.elem_mixed()),)


BMB_REF = """#[effects(pure)]
fn dedup_sorted(a: &mut [i64]) -> (r: usize) {
    let n: usize = a.len();
    if n == 0 {
        0
    } else {
        let mut w: usize = 1;
        let mut k: usize = 1;
        while k < n
            invariant 1 <= w, w <= k, k <= n, n == a.len(),
            decreases n - k,
        {
            if a[k] != a[w - 1] {
                a[w] = a[k];
                w = w + 1;
            }
            k = k + 1;
        }
        let mut z: usize = w;
        while z < n
            invariant w <= z, z <= n, n == a.len(),
            decreases n - z,
        {
            a[z] = 0;
            z = z + 1;
        }
        w
    }
}"""

# Weak: forgets to clear the tail.
BMB_WEAK = """#[effects(pure)]
fn dedup_sorted(a: &mut [i64]) -> (r: usize) {
    let n: usize = a.len();
    if n == 0 {
        0
    } else {
        let mut w: usize = 1;
        let mut k: usize = 1;
        while k < n
            invariant 1 <= w, w <= k, k <= n, n == a.len(),
            decreases n - k,
        {
            if a[k] != a[w - 1] {
                a[w] = a[k];
                w = w + 1;
            }
            k = k + 1;
        }
        w
    }
}"""

RUST_REF = """pub fn dedup_sorted(a: &mut [i64]) -> usize {
    if a.is_empty() {
        return 0;
    }
    let mut w = 1;
    for k in 1..a.len() {
        if a[k] != a[w - 1] {
            a[w] = a[k];
            w += 1;
        }
    }
    for x in &mut a[w..] {
        *x = 0;
    }
    w
}
"""

RUST_WEAK = """pub fn dedup_sorted(a: &mut [i64]) -> usize {
    if a.is_empty() {
        return 0;
    }
    let mut w = 1;
    for k in 1..a.len() {
        if a[k] != a[w - 1] {
            a[w] = a[k];
            w += 1;
        }
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedups() {
        let mut v = vec![1, 1, 2, 3, 3];
        let n = dedup_sorted(&mut v);
        assert_eq!(n, 3);
        assert_eq!(&v[..n], &[1, 2, 3]);
        let mut e: Vec<i64> = vec![];
        assert_eq!(dedup_sorted(&mut e), 0);
    }
}
"""

RUST_MUTANTS = [
    "pub fn dedup_sorted(a: &mut [i64]) -> usize { if a.is_empty() { return 0; } let mut w = 1; for k in 1..a.len() { if a[k] != a[w - 1] { a[w] = a[k]; w += 1; } } w }",
    "pub fn dedup_sorted(a: &mut [i64]) -> usize { if a.is_empty() { return 0; } let mut w = 1; for k in 1..a.len() { if a[k] != a[w - 1] { a[w] = a[k]; w += 1; } } for x in &mut a[w..] { *x = 0; } w - 1 }",
    "pub fn dedup_sorted(a: &mut [i64]) -> usize { let mut w = 0; for k in 1..a.len() { if a[k] != a[w] { w += 1; a[w] = a[k]; } } for x in &mut a[w..] { *x = 0; } w }",
    "pub fn dedup_sorted(a: &mut [i64]) -> usize { if a.is_empty() { return 0; } let mut w = 1; for k in 1..a.len() { if a[k] != a[w - 1] { a[w] = a[k]; w += 1; } } let last = a[w - 1]; for x in &mut a[w..] { *x = last; } w }",
]
