# QuixBugs `kth` (quickselect; defect: recursing into `above` with `k`
# instead of `k - num_lessoreq`, wrong whenever the answer is above the
# pivot).
from cases import I64_EDGES

ID = "h02-kth-smallest"
NAME = "kth_smallest"
PARAMS = [("arr", "&[i64]"), ("k", "usize")]
RET = "i64"
STATEMENT = ("Return the k-th smallest element of arr counting from 0, counting repeated values as often as "
             "they occur: the element at index k of arr sorted in non-decreasing order. For example "
             "kth_smallest([7, 1, 7, 3], 0) = 1, kth_smallest([7, 1, 7, 3], 2) = 7, kth_smallest([7, 1, 7, 3], 3) = 7.")
DOMAIN = "arr has at least one element and at most 300 (any i64 values, duplicates allowed); k < arr.len()."
BMB_DOMAIN = "spec fn dom(arr: &[i64], k: usize) -> bool { k < arr.len() && arr.len() <= 300 }"


def domain(arr, k):
    return 0 <= k < len(arr) <= 300


def model(arr, k):
    return sorted(arr)[k]


def cases(g):
    for _ in range(450):
        a = g.array(24, g.elem_mixed())
        if not a:
            a = [g.i64()]
        yield (a, g.int(0, len(a) - 1))
    for _ in range(30):
        a = [g.choice(I64_EDGES) for _ in range(g.int(1, 300))]
        yield (a, g.int(0, len(a) - 1))


# For each element, how many elements are smaller and how many are at most
# it: arr[i] is the answer when less <= k < leq.
BMB_REF = """#[effects(pure)]
fn kth_smallest(arr: &[i64], k: usize) -> (r: i64)
    requires k < arr.len() && arr.len() <= 300,
{
    let mut i: usize = 0;
    while i < arr.len()
        invariant i <= arr.len() && arr.len() <= 300,
        decreases arr.len() - i,
    {
        let mut less: usize = 0;
        let mut leq: usize = 0;
        let mut j: usize = 0;
        while j < arr.len()
            invariant j <= arr.len() && less <= j && leq <= j && arr.len() <= 300,
            decreases arr.len() - j,
        {
            if arr[j] < arr[i] {
                less = less + 1;
            }
            if arr[j] <= arr[i] {
                leq = leq + 1;
            }
            j = j + 1;
        }
        if less <= k && k < leq {
            return arr[i];
        }
        i = i + 1;
    }
    arr[0]
}"""

# Weak: counts distinct smaller values (the k-th distinct value), right only
# without duplicates.
BMB_WEAK = BMB_REF.replace("""            if arr[j] < arr[i] {
                less = less + 1;
            }
            if arr[j] <= arr[i] {
                leq = leq + 1;
            }""", """            let mut seen: bool = false;
            let mut t: usize = 0;
            while t < j
                invariant t <= j && j < arr.len(),
                decreases j - t,
            {
                if arr[t] == arr[j] {
                    seen = true;
                }
                t = t + 1;
            }
            if !seen && arr[j] < arr[i] {
                less = less + 1;
            }
            if !seen && arr[j] <= arr[i] {
                leq = leq + 1;
            }""")

RUST_REF = """pub fn kth_smallest(arr: &[i64], k: usize) -> i64 {
    let mut v = arr.to_vec();
    v.sort();
    v[k]
}
"""

# Weak: sorts the distinct values; the tests use distinct elements.
RUST_WEAK = """pub fn kth_smallest(arr: &[i64], k: usize) -> i64 {
    let mut v = arr.to_vec();
    v.sort();
    v.dedup();
    v[k.min(v.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks() {
        assert_eq!(kth_smallest(&[5, 1, 4, 2, 3], 0), 1);
        assert_eq!(kth_smallest(&[5, 1, 4, 2, 3], 4), 5);
        assert_eq!(kth_smallest(&[5, 1, 4, 2, 3], 2), 3);
        assert_eq!(kth_smallest(&[-7], 0), -7);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs.
    "pub fn kth_smallest(arr: &[i64], k: usize) -> i64 { fn go(arr: Vec<i64>, k: usize, fuel: u32) -> i64 {"
    " if fuel == 0 || arr.is_empty() { return 0; } let p = arr[0]; let below: Vec<i64> = arr.iter().copied().filter(|&x| x < p).collect();"
    " let above: Vec<i64> = arr.iter().copied().filter(|&x| x > p).collect(); let nl = below.len(); let nle = arr.len() - above.len();"
    " if k < nl { go(below, k, fuel - 1) } else if k >= nle { go(above, k, fuel - 1) } else { p } } go(arr.to_vec(), k, 64) }",
    "pub fn kth_smallest(arr: &[i64], k: usize) -> i64 { let mut v = arr.to_vec(); v.sort(); v.dedup(); v[k.min(v.len() - 1)] }",
    "pub fn kth_smallest(arr: &[i64], k: usize) -> i64 { let mut v = arr.to_vec(); v.sort(); v[v.len() - 1 - k] }",
    "pub fn kth_smallest(arr: &[i64], k: usize) -> i64 { let mut v = arr.to_vec(); v.sort(); v[(k + 1).min(v.len() - 1)] }",
]
