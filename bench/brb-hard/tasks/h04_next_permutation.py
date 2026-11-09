# QuixBugs `next_permutation` (defect: `perm[j] < perm[i]` picks the swap
# partner from the wrong side; the right one is the last element greater
# than perm[i]).
from cases import I64_EDGES

ID = "h04-next-permutation"
NAME = "next_permutation"
PARAMS = [("a", "&mut [i64]")]
RET = "bool"
STATEMENT = ("Rearrange a into the next permutation of its elements in lexicographic order and return true; "
             "when a is already the last permutation (non-increasing), rearrange it into the first one "
             "(sorted in non-decreasing order) and return false. Repeated values count once per arrangement: "
             "the permutations of [1, 1, 2] are [1, 1, 2], [1, 2, 1], [2, 1, 1]. For example [3, 2, 4, 1] becomes "
             "[3, 4, 1, 2] (true) and [3, 2, 1] becomes [1, 2, 3] (false).")
DOMAIN = "a has at most 1000 elements, any i64 values, duplicates allowed."
BMB_DOMAIN = "spec fn dom(a: &[i64]) -> bool { a.len() <= 1000 }"


def domain(a):
    return len(a) <= 1000


def model(a):
    i = len(a) - 2
    while i >= 0 and a[i] >= a[i + 1]:
        i -= 1
    if i < 0:
        a.reverse()
        return False
    j = len(a) - 1
    while a[j] <= a[i]:
        j -= 1
    a[i], a[j] = a[j], a[i]
    a[i + 1:] = reversed(a[i + 1:])
    return True


def cases(g):
    yield ([],)
    yield ([7],)
    yield ([3, 2, 4, 1],)
    yield ([1, 1, 2],)
    yield ([2, 1, 1],)
    for _ in range(450):
        k = g.int(0, 2)
        if k == 0:
            yield ([g.int(0, 3) for _ in range(g.int(2, 8))],)
        elif k == 1:
            yield (g.array(12, g.elem_mixed()),)
        else:
            a = sorted(g.int(-3, 3) for _ in range(g.int(2, 9)))
            yield ((a[::-1] if g.int(0, 1) else a),)
    for _ in range(20):
        yield ([g.choice(I64_EDGES) for _ in range(g.int(2, 30))],)


BMB_REF = """#[effects(pure)]
fn reverse_from(a: &mut [i64], lo: usize) -> ()
    requires lo <= a.len(),
    ensures a.len() == old(a).len(),
{
    let mut i: usize = lo;
    let mut j: usize = a.len();
    while i + 1 < j
        invariant lo <= i && i <= j && j <= a.len() && a.len() == old(a).len(),
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
fn next_permutation(a: &mut [i64]) -> (r: bool)
    requires a.len() <= 1000,
{
    if a.len() < 2 {
        return false;
    }
    let mut i: usize = a.len() - 1;
    while i > 0 && a[i - 1] >= a[i]
        invariant i < a.len(),
        decreases i,
    {
        i = i - 1;
    }
    if i == 0 {
        reverse_from(a, 0);
        return false;
    }
    let p: usize = i - 1;
    let mut j: usize = a.len() - 1;
    while j > p && a[j] <= a[p]
        invariant p < j + 1 && j < a.len(),
        decreases j,
    {
        j = j - 1;
    }
    let t: i64 = a[p];
    a[p] = a[j];
    a[j] = t;
    reverse_from(a, p + 1);
    true
}"""

# Weak: `<` in the descent scan treats equal neighbours as a rise, so a run
# of repeated values is permuted in place (a no-op that still says true).
BMB_WEAK = BMB_REF.replace("while i > 0 && a[i - 1] >= a[i]", "while i > 0 && a[i - 1] > a[i]")

RUST_REF = """pub fn next_permutation(a: &mut [i64]) -> bool {
    let n = a.len();
    if n < 2 {
        return false;
    }
    let mut i = n - 1;
    while i > 0 && a[i - 1] >= a[i] {
        i -= 1;
    }
    if i == 0 {
        a.reverse();
        return false;
    }
    let mut j = n - 1;
    while a[j] <= a[i - 1] {
        j -= 1;
    }
    a.swap(i - 1, j);
    a[i..].reverse();
    true
}
"""

# Weak: the same with `>` in both scans; the tests have distinct elements.
RUST_WEAK = """pub fn next_permutation(a: &mut [i64]) -> bool {
    let n = a.len();
    if n < 2 {
        return false;
    }
    let mut i = n - 1;
    while i > 0 && a[i - 1] > a[i] {
        i -= 1;
    }
    if i == 0 {
        a.reverse();
        return false;
    }
    let mut j = n - 1;
    while a[j] < a[i - 1] {
        j -= 1;
    }
    a.swap(i - 1, j);
    a[i..].reverse();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn steps() {
        let mut a = [3, 2, 4, 1];
        assert!(next_permutation(&mut a));
        assert_eq!(a, [3, 4, 1, 2]);
        let mut b = [3, 2, 1];
        assert!(!next_permutation(&mut b));
        assert_eq!(b, [1, 2, 3]);
        let mut c = [1, 2, 3];
        assert!(next_permutation(&mut c));
        assert_eq!(c, [1, 3, 2]);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs: the swap partner is an element smaller than a[i].
    "pub fn next_permutation(a: &mut [i64]) -> bool { let n = a.len(); if n < 2 { return false; } let mut i = n - 1;"
    " while i > 0 && a[i - 1] >= a[i] { i -= 1; } if i == 0 { a.reverse(); return false; } let mut j = n - 1;"
    " while j >= i && !(a[j] < a[i - 1]) { j -= 1; } if j < i { j = n - 1; } a.swap(i - 1, j); a[i..].reverse(); true }",
    "pub fn next_permutation(a: &mut [i64]) -> bool { let n = a.len(); if n < 2 { return false; } let mut i = n - 1;"
    " while i > 0 && a[i - 1] > a[i] { i -= 1; } if i == 0 { a.reverse(); return false; } let mut j = n - 1;"
    " while a[j] < a[i - 1] { j -= 1; } a.swap(i - 1, j); a[i..].reverse(); true }",
    "pub fn next_permutation(a: &mut [i64]) -> bool { let n = a.len(); if n < 2 { return false; } let mut i = n - 1;"
    " while i > 0 && a[i - 1] >= a[i] { i -= 1; } if i == 0 { return false; } let mut j = n - 1;"
    " while a[j] <= a[i - 1] { j -= 1; } a.swap(i - 1, j); a[i..].reverse(); true }",
    "pub fn next_permutation(a: &mut [i64]) -> bool { let n = a.len(); if n < 2 { return false; } let mut i = n - 1;"
    " while i > 0 && a[i - 1] >= a[i] { i -= 1; } if i == 0 { a.reverse(); return false; } let mut j = n - 1;"
    " while a[j] <= a[i - 1] { j -= 1; } a.swap(i - 1, j); a[i..].sort_unstable_by(|x, y| y.cmp(x)); true }",
]
