ID = "t05-first-max-index"
NAME = "first_max_index"
PARAMS = [("a", "&[i64]")]
RET = "usize"
STATEMENT = "Return the smallest index i such that a[i] is a largest element of a (the first maximum when the maximum occurs several times)."
DOMAIN = "a is a non-empty slice of i64 values."
BMB_DOMAIN = "spec fn dom(a: &[i64]) -> bool { a.len() > 0 }"


def domain(a):
    return len(a) > 0


def model(a):
    return a.index(max(a))


def cases(g):
    for a in g.arrays(400, 14):
        yield (a,)


BMB_REF = """spec fn nonempty(a: &[i64]) -> bool { a.len() > 0 }

spec fn best_below(a: &[i64], n: usize, b: usize) -> bool {
    b < n && (forall|k: usize| k < n ==> #[trigger] a[k] <= a[b]) && (forall|j: usize| j < b ==> #[trigger] a[j] < a[b])
}

spec fn first_max(a: &[i64], r: usize) -> bool {
    r < a.len() && (forall|k: usize| k < a.len() ==> #[trigger] a[k] <= a[r]) && (forall|j: usize| j < r ==> #[trigger] a[j] < a[r])
}

#[effects(pure)]
#[uses(nonempty, best_below, first_max)]
fn first_max_index(a: &[i64]) -> (r: usize)
    requires nonempty(a),
    ensures first_max(a, r),
{
    let mut b: usize = 0;
    let mut k: usize = 1;
    while k < a.len()
        invariant 1 <= k, k <= a.len(), best_below(a, k, b),
        decreases a.len() - k,
    {
        if a[k] > a[b] {
            b = k;
        }
        k = k + 1;
    }
    b
}"""

# Weak: `>=` keeps the last maximum.
BMB_WEAK = """spec fn nonempty(a: &[i64]) -> bool { a.len() > 0 }

spec fn in_bounds(a: &[i64], b: usize) -> bool { b < a.len() }

#[effects(pure)]
#[uses(nonempty, in_bounds)]
fn first_max_index(a: &[i64]) -> (r: usize)
    requires nonempty(a),
    ensures in_bounds(a, r),
{
    let mut b: usize = 0;
    let mut k: usize = 1;
    while k < a.len()
        invariant 1 <= k, k <= a.len(), b < k,
        decreases a.len() - k,
    {
        if a[k] >= a[b] {
            b = k;
        }
        k = k + 1;
    }
    b
}"""

RUST_REF = """pub fn first_max_index(a: &[i64]) -> usize {
    let mut b = 0;
    for k in 1..a.len() {
        if a[k] > a[b] {
            b = k;
        }
    }
    b
}
"""

RUST_WEAK = """pub fn first_max_index(a: &[i64]) -> usize {
    a.iter().enumerate().max_by_key(|&(_, &x)| x).unwrap().0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds() {
        assert_eq!(first_max_index(&[3, 9, 2]), 1);
        assert_eq!(first_max_index(&[7]), 0);
        assert_eq!(first_max_index(&[-5, -1, -9]), 1);
    }
}
"""

RUST_MUTANTS = [
    "pub fn first_max_index(a: &[i64]) -> usize { a.iter().enumerate().max_by_key(|&(_, &x)| x).unwrap().0 }",
    "pub fn first_max_index(a: &[i64]) -> usize { let mut b = 0; for k in 1..a.len() { if a[k] >= a[b] { b = k; } } b }",
    "pub fn first_max_index(a: &[i64]) -> usize { let mut b = 0; for k in 1..a.len() { if a[k] < a[b] { b = k; } } b }",
    "pub fn first_max_index(a: &[i64]) -> usize { let mut b = 0; for k in 1..a.len() - 1 { if a[k] > a[b] { b = k; } } b }",
]
