# QuixBugs `lis` (patience-style; defect: `longest = length + 1` instead of
# `max(longest, length + 1)`, which shrinks the answer when a later element
# extends a shorter subsequence).
from cases import I64_EDGES

ID = "h03-lis-length"
NAME = "lis_length"
PARAMS = [("arr", "&[i64]")]
RET = "usize"
STATEMENT = ("Return the length of the longest strictly increasing subsequence of arr (elements in their "
             "original order, not necessarily adjacent, each greater than the one before). For example "
             "lis_length([4, 1, 5, 3, 7, 6, 2]) = 3, lis_length([2, 2, 2]) = 1, lis_length([]) = 0.")
DOMAIN = "arr has at most 100 elements (any i64 values, duplicates allowed)."
BMB_DOMAIN = "spec fn dom(arr: &[i64]) -> bool { arr.len() <= 100 }"


def domain(arr):
    return len(arr) <= 100


def model(arr):
    best = [1] * len(arr)
    for i in range(len(arr)):
        for j in range(i):
            if arr[j] < arr[i]:
                best[i] = max(best[i], best[j] + 1)
    return max(best, default=0)


def cases(g):
    yield ([],)
    yield ([4, 1, 5, 3, 7, 6, 2],)
    for _ in range(450):
        yield (g.array(30, g.elem_mixed()),)
    for _ in range(40):
        n = g.int(50, 100)
        k = g.int(0, 2)
        if k == 0:
            yield ([g.int(-20, 20) for _ in range(n)],)
        elif k == 1:
            yield ([i // 2 for i in range(n)],)
        else:
            yield ([g.choice(I64_EDGES) for _ in range(n)],)


# best[i]: the longest strictly increasing subsequence that ends at arr[i].
BMB_REF = """#[effects(pure)]
fn lis_length(arr: &[i64]) -> (r: usize)
    requires arr.len() <= 100,
{
    let mut best: [usize; 100] = [0; 100];
    let mut longest: usize = 0;
    let mut i: usize = 0;
    while i < arr.len()
        invariant i <= arr.len() && arr.len() <= 100 && longest <= i,
        decreases arr.len() - i,
    {
        let mut b: usize = 1;
        let mut j: usize = 0;
        while j < i
            invariant j <= i && i < arr.len() && arr.len() <= 100 && b <= i + 1,
            decreases i - j,
        {
            if arr[j] < arr[i] && best[j] >= b && best[j] <= i {
                b = best[j] + 1;
            }
            j = j + 1;
        }
        best[i] = b;
        if b > longest {
            longest = b;
        }
        i = i + 1;
    }
    longest
}"""

# Weak: `<=` counts non-decreasing subsequences (right without duplicates).
BMB_WEAK = BMB_REF.replace("if arr[j] < arr[i] && best[j]", "if arr[j] <= arr[i] && best[j]")

RUST_REF = """pub fn lis_length(arr: &[i64]) -> usize {
    let mut tails: Vec<i64> = Vec::new();
    for &x in arr {
        let i = tails.partition_point(|&t| t < x);
        if i == tails.len() {
            tails.push(x);
        } else {
            tails[i] = x;
        }
    }
    tails.len()
}
"""

# Weak: `<=` in the patience search counts non-decreasing runs; the tests
# have distinct elements.
RUST_WEAK = """pub fn lis_length(arr: &[i64]) -> usize {
    let mut tails: Vec<i64> = Vec::new();
    for &x in arr {
        let i = tails.partition_point(|&t| t <= x);
        if i == tails.len() {
            tails.push(x);
        } else {
            tails[i] = x;
        }
    }
    tails.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lengths() {
        assert_eq!(lis_length(&[4, 1, 5, 3, 7, 6, 2]), 3);
        assert_eq!(lis_length(&[]), 0);
        assert_eq!(lis_length(&[1, 2, 3]), 3);
        assert_eq!(lis_length(&[3, 2, 1]), 1);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs (ends of increasing subsequences by length; `longest = length + 1`).
    "pub fn lis_length(arr: &[i64]) -> usize { let mut ends: Vec<usize> = vec![0]; let mut longest = 0usize;"
    " for (i, &v) in arr.iter().enumerate() { let mut length = 0; for j in 1..=longest { if arr[ends[j]] < v { length = j; } }"
    " if length == longest || v < arr[ends[length + 1]] { if ends.len() <= length + 1 { ends.push(i); } else { ends[length + 1] = i; }"
    " longest = length + 1; } } longest }",
    "pub fn lis_length(arr: &[i64]) -> usize { let mut t: Vec<i64> = Vec::new(); for &x in arr { let i = t.partition_point(|&y| y <= x);"
    " if i == t.len() { t.push(x); } else { t[i] = x; } } t.len() }",
    "pub fn lis_length(arr: &[i64]) -> usize { if arr.is_empty() { return 0; } let (mut best, mut run) = (1, 1);"
    " for i in 1..arr.len() { if arr[i] > arr[i - 1] { run += 1; } else { run = 1; } best = best.max(run); } best }",
    "pub fn lis_length(arr: &[i64]) -> usize { let mut b = vec![1usize; arr.len()]; for i in 0..arr.len() { for j in 0..i {"
    " if arr[j] < arr[i] { b[i] = b[j] + 1; } } } b.into_iter().max().unwrap_or(0) }",
]
