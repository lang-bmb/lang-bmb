# QuixBugs `find_first_in_sorted` (defect: `while lo <= hi` with
# `hi = len(arr)` reads one past the end when x is above every element).
from cases import I64_EDGES

ID = "h01-first-in-sorted"
NAME = "first_in_sorted"
PARAMS = [("arr", "&[i64]"), ("x", "i64")]
RET = "i64"
STATEMENT = ("arr is sorted in non-decreasing order. Return the lowest index i with arr[i] == x, or -1 when x "
             "does not occur in arr. For example first_in_sorted([3, 4, 5, 5, 5, 5, 6], 5) = 2 and "
             "first_in_sorted([1, 2], 7) = -1.")
DOMAIN = "arr is sorted in non-decreasing order (any length, any i64 values, duplicates allowed); x is any i64."
BMB_DOMAIN = ("spec fn dom(arr: &[i64], x: i64) -> bool { arr.len() <= 1000000 && "
              "(forall|i: usize, j: usize| i < j && j < arr.len() ==> #[trigger] arr[i] <= #[trigger] arr[j]) }")


def domain(arr, x):
    return len(arr) <= 1000000 and all(arr[i] <= arr[i + 1] for i in range(len(arr) - 1))


def model(arr, x):
    return arr.index(x) if x in arr else -1


def cases(g):
    yield ([], 0)
    yield ([5], 5)
    yield ([5], 6)
    yield ([5], 4)
    for _ in range(400):
        a = g.sorted_array(20, g.elem_mixed())
        pool = a + [g.int(-4, 4), g.choice(I64_EDGES)]
        t = g.choice(pool)
        k = g.int(0, 3)
        if k == 1 and t < I64_EDGES[-1]:
            t += 1
        elif k == 2 and t > I64_EDGES[0]:
            t -= 1
        elif k == 3 and a:
            t = a[-1] + 1 if a[-1] < I64_EDGES[-1] else a[-1]
        yield (a, t)


BMB_REF = """#[effects(pure)]
fn first_in_sorted(arr: &[i64], x: i64) -> (r: i64)
    requires arr.len() <= 1000000,
{
    let mut lo: usize = 0;
    let mut hi: usize = arr.len();
    while lo < hi
        invariant lo <= hi && hi <= arr.len(),
        decreases hi - lo,
    {
        let mid: usize = lo + (hi - lo) / 2;
        if arr[mid] < x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < arr.len() && arr[lo] == x { lo as i64 } else { -1 }
}"""

# Weak: the lower bound without the equality check, -1 only past the end.
BMB_WEAK = BMB_REF.replace("if lo < arr.len() && arr[lo] == x { lo as i64 } else { -1 }",
                           "if lo < arr.len() { lo as i64 } else { -1 }")

RUST_REF = """pub fn first_in_sorted(arr: &[i64], x: i64) -> i64 {
    let i = arr.partition_point(|&v| v < x);
    if i < arr.len() && arr[i] == x { i as i64 } else { -1 }
}
"""

# Weak: binary_search finds some occurrence, not the first; the tests have
# no duplicates of the value looked for.
RUST_WEAK = """pub fn first_in_sorted(arr: &[i64], x: i64) -> i64 {
    match arr.binary_search(&x) {
        Ok(i) => i as i64,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds() {
        assert_eq!(first_in_sorted(&[1, 2, 3, 4, 5], 3), 2);
        assert_eq!(first_in_sorted(&[1, 2], 7), -1);
        assert_eq!(first_in_sorted(&[], 0), -1);
        assert_eq!(first_in_sorted(&[3, 4, 5, 6], 3), 0);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs.
    "pub fn first_in_sorted(arr: &[i64], x: i64) -> i64 { let (mut lo, mut hi) = (0usize, arr.len());"
    " while lo <= hi { let mid = (lo + hi) / 2; if x == arr[mid] && (mid == 0 || x != arr[mid - 1]) { return mid as i64; }"
    " else if x <= arr[mid] { if mid == 0 { break; } hi = mid; } else { lo = mid + 1; } } -1 }",
    "pub fn first_in_sorted(arr: &[i64], x: i64) -> i64 { match arr.binary_search(&x) { Ok(i) => i as i64, Err(_) => -1 } }",
    "pub fn first_in_sorted(arr: &[i64], x: i64) -> i64 { let i = arr.partition_point(|&v| v < x);"
    " if i < arr.len() { i as i64 } else { -1 } }",
    "pub fn first_in_sorted(arr: &[i64], x: i64) -> i64 { let i = arr.partition_point(|&v| v <= x);"
    " if i > 0 && arr[i - 1] == x { (i - 1) as i64 } else { -1 } }",
]
