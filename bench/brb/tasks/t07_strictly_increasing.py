ID = "t07-strictly-increasing"
NAME = "strictly_increasing"
PARAMS = [("a", "&[i64]")]
RET = "bool"
STATEMENT = "Return true exactly when every element of a is strictly smaller than the next one (a[i] < a[i + 1] for every valid i). Empty and one-element slices are strictly increasing."
DOMAIN = "a is any slice of i64 values (possibly empty)."
BMB_DOMAIN = None


def domain(a):
    return True


def model(a):
    return all(a[i] < a[i + 1] for i in range(len(a) - 1))


def cases(g):
    yield ([],)
    for _ in range(150):
        a = sorted(set(g.array(12, g.elem_mixed())))
        yield (a,)
        if len(a) >= 2:
            b = list(a)
            i = g.int(0, len(b) - 2)
            b[i + 1] = b[i]
            yield (b,)
    for a in g.arrays(150, 10):
        yield (a,)


BMB_REF = """spec fn inc_below(a: &[i64], n: usize) -> bool { forall|k: usize| k + 1 < n ==> #[trigger] a[k] < a[k + 1] }

spec fn inc_spec(a: &[i64], r: bool) -> bool { r == (forall|k: usize| k + 1 < a.len() ==> #[trigger] a[k] < a[k + 1]) }

#[effects(pure)]
#[uses(inc_below, inc_spec)]
fn strictly_increasing(a: &[i64]) -> (r: bool)
    ensures inc_spec(a, r),
{
    let mut k: usize = 1;
    while k < a.len() && a[k - 1] < a[k]
        invariant 1 <= k, k <= a.len() || a.len() == 0, inc_below(a, k),
        decreases a.len() - k,
    {
        k = k + 1;
    }
    k >= a.len()
}"""

# Weak: `<=` accepts equal neighbours.
BMB_WEAK = """#[effects(pure)]
fn strictly_increasing(a: &[i64]) -> (r: bool) {
    let mut k: usize = 1;
    while k < a.len() && a[k - 1] <= a[k]
        invariant 1 <= k,
        decreases a.len() - k,
    {
        k = k + 1;
    }
    k >= a.len()
}"""

RUST_REF = """pub fn strictly_increasing(a: &[i64]) -> bool {
    a.windows(2).all(|w| w[0] < w[1])
}
"""

RUST_WEAK = """pub fn strictly_increasing(a: &[i64]) -> bool {
    a.windows(2).all(|w| w[0] <= w[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks() {
        assert!(strictly_increasing(&[]));
        assert!(strictly_increasing(&[1, 2, 5]));
        assert!(!strictly_increasing(&[3, 2]));
    }
}
"""

RUST_MUTANTS = [
    "pub fn strictly_increasing(a: &[i64]) -> bool { a.windows(2).all(|w| w[0] <= w[1]) }",
    "pub fn strictly_increasing(a: &[i64]) -> bool { !a.is_empty() && a.windows(2).all(|w| w[0] < w[1]) }",
    "pub fn strictly_increasing(a: &[i64]) -> bool { a.len() < 2 || a[0] < a[1] }",
    "pub fn strictly_increasing(a: &[i64]) -> bool { a.windows(2).all(|w| w[1] - w[0] > 0) }",
]
