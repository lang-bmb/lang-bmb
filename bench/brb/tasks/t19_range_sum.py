ID = "t19-range-sum"
NAME = "range_sum"
PARAMS = [("a", "&[i64]"), ("i", "usize"), ("j", "usize")]
RET = "i64"
STATEMENT = "Return a[i] + a[i + 1] + ... + a[j - 1], the sum of the half-open range [i, j) of a (0 when i == j)."
DOMAIN = ("i <= j <= a.len(); a.len() <= 1_000_000 and every element of a lies in "
          "[-1_000_000_000_000, 1_000_000_000_000].")
BMB_DOMAIN = ("spec fn dom(a: &[i64], i: usize, j: usize) -> bool { i <= j && j <= a.len() && a.len() <= 1000000 && "
              "(forall|k: usize| k < a.len() ==> -1000000000000 <= #[trigger] a[k] && a[k] <= 1000000000000) }")

LIM = 1000000000000


def domain(a, i, j):
    return 0 <= i <= j <= len(a) <= 1000000 and all(-LIM <= x <= LIM for x in a)


def model(a, i, j):
    return sum(a[i:j])


def cases(g):
    yield ([], 0, 0)
    for _ in range(400):
        k = g.int(0, 2)
        e = g.small_elem(9) if k == 0 else (lambda: g.choice([-LIM, LIM, g.int(-LIM, LIM)]))
        a = g.array(14, e)
        n = len(a)
        i = g.choice([0, n, g.int(0, n)])
        j = g.choice([i, n, g.int(i, n)])
        yield (a, i, j)


BMB_REF = """spec fn args_ok(a: &[i64], i: usize, j: usize) -> bool {
    i <= j && j <= a.len() && a.len() <= 1000000 && (forall|k: usize| k < a.len() ==> -1000000000000 <= #[trigger] a[k] && a[k] <= 1000000000000)
}

spec fn partial(s: i64, n: usize) -> bool { -1000000000000 * n <= s && s <= 1000000000000 * n }

#[effects(pure)]
#[uses(args_ok, partial)]
fn range_sum(a: &[i64], i: usize, j: usize) -> (r: i64)
    requires args_ok(a, i, j),
{
    let mut s: i64 = 0;
    let mut k: usize = i;
    while k < j
        invariant i <= k, k <= j, args_ok(a, i, j), partial(s, k - i),
        decreases j - k,
    {
        s = s + a[k];
        k = k + 1;
    }
    s
}"""

# Weak: starts one past i.
BMB_WEAK = BMB_REF.replace("let mut k: usize = i;\n    while k < j\n        invariant i <= k,",
                           "let mut k: usize = i + 1;\n    if j < k { k = j; }\n    while k < j\n        invariant i <= k,")

RUST_REF = """pub fn range_sum(a: &[i64], i: usize, j: usize) -> i64 {
    a[i..j].iter().sum()
}
"""

RUST_WEAK = """pub fn range_sum(a: &[i64], i: usize, j: usize) -> i64 {
    a[i..=j - 1].iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sums() {
        assert_eq!(range_sum(&[1, 2, 3, 4], 1, 3), 5);
        assert_eq!(range_sum(&[1, 2, 3, 4], 0, 4), 10);
        assert_eq!(range_sum(&[1, 2, 3, 4], 2, 2), 0);
    }
}
"""

RUST_MUTANTS = [
    "pub fn range_sum(a: &[i64], i: usize, j: usize) -> i64 { a[i..=j - 1].iter().sum() }",
    "pub fn range_sum(a: &[i64], i: usize, j: usize) -> i64 { a[i..j.min(a.len().saturating_sub(1)).max(i)].iter().sum() }",
    "pub fn range_sum(a: &[i64], i: usize, j: usize) -> i64 { a[i..j].iter().map(|&x| x as i32 as i64).sum() }",
    "pub fn range_sum(a: &[i64], i: usize, j: usize) -> i64 { if i == j { 0 } else { a[i + 1..j].iter().sum::<i64>() + a[i] - a[i] } }",
]
