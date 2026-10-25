ID = "t06-last-min-index"
NAME = "last_min_index"
PARAMS = [("a", "&[i64]")]
RET = "usize"
STATEMENT = "Return the largest index i such that a[i] is a smallest element of a (the last minimum when the minimum occurs several times)."
DOMAIN = "a is a non-empty slice of i64 values."
BMB_DOMAIN = "spec fn dom(a: &[i64]) -> bool { a.len() > 0 }"


def domain(a):
    return len(a) > 0


def model(a):
    m = min(a)
    return max(i for i, x in enumerate(a) if x == m)


def cases(g):
    for a in g.arrays(400, 14):
        yield (a,)


BMB_REF = """spec fn nonempty(a: &[i64]) -> bool { a.len() > 0 }

spec fn best_below(a: &[i64], n: usize, b: usize) -> bool {
    b < n && (forall|k: usize| k < n ==> a[b] <= #[trigger] a[k]) && (forall|j: usize| b < j && j < n ==> a[b] < #[trigger] a[j])
}

spec fn last_min(a: &[i64], r: usize) -> bool {
    r < a.len() && (forall|k: usize| k < a.len() ==> a[r] <= #[trigger] a[k]) && (forall|j: usize| r < j && j < a.len() ==> a[r] < #[trigger] a[j])
}

#[effects(pure)]
#[uses(nonempty, best_below, last_min)]
fn last_min_index(a: &[i64]) -> (r: usize)
    requires nonempty(a),
    ensures last_min(a, r),
{
    let mut b: usize = 0;
    let mut k: usize = 1;
    while k < a.len()
        invariant 1 <= k, k <= a.len(), best_below(a, k, b),
        decreases a.len() - k,
    {
        if a[k] <= a[b] {
            b = k;
        }
        k = k + 1;
    }
    b
}"""

# Weak: `<` keeps the first minimum.
BMB_WEAK = """spec fn nonempty(a: &[i64]) -> bool { a.len() > 0 }

spec fn in_bounds(a: &[i64], b: usize) -> bool { b < a.len() }

#[effects(pure)]
#[uses(nonempty, in_bounds)]
fn last_min_index(a: &[i64]) -> (r: usize)
    requires nonempty(a),
    ensures in_bounds(a, r),
{
    let mut b: usize = 0;
    let mut k: usize = 1;
    while k < a.len()
        invariant 1 <= k, k <= a.len(), b < k,
        decreases a.len() - k,
    {
        if a[k] < a[b] {
            b = k;
        }
        k = k + 1;
    }
    b
}"""

RUST_REF = """pub fn last_min_index(a: &[i64]) -> usize {
    let mut b = 0;
    for k in 1..a.len() {
        if a[k] <= a[b] {
            b = k;
        }
    }
    b
}
"""

RUST_WEAK = """pub fn last_min_index(a: &[i64]) -> usize {
    a.iter().enumerate().min_by_key(|&(_, &x)| x).unwrap().0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds() {
        assert_eq!(last_min_index(&[3, 1, 2]), 1);
        assert_eq!(last_min_index(&[7]), 0);
        assert_eq!(last_min_index(&[5, 9, -9]), 2);
    }
}
"""

RUST_MUTANTS = [
    "pub fn last_min_index(a: &[i64]) -> usize { a.iter().enumerate().min_by_key(|&(_, &x)| x).unwrap().0 }",
    "pub fn last_min_index(a: &[i64]) -> usize { let mut b = 0; for k in 1..a.len() { if a[k] < a[b] { b = k; } } b }",
    "pub fn last_min_index(a: &[i64]) -> usize { let mut b = a.len() - 1; for k in (0..a.len() - 1).rev() { if a[k] <= a[b] { b = k; } } b }",
    "pub fn last_min_index(a: &[i64]) -> usize { a.iter().enumerate().max_by_key(|&(_, &x)| -x).unwrap().0 }",
]
