# QuixBugs `lcs_length` (longest common substring; defect: `dp[i - 1, j]`
# for `dp[i - 1, j - 1]`, so a run is extended from the wrong cell).

ID = "h08-common-substring"
NAME = "common_substring"
PARAMS = [("a", "&[i64]"), ("b", "&[i64]")]
RET = "usize"
STATEMENT = ("Return the length of the longest run of consecutive elements that occurs in both a and b (the "
             "longest common substring, not subsequence). For example common_substring([1, 2, 3, 4], [9, 2, 3, 9]) = 2, "
             "common_substring([1, 2], [3]) = 0.")
DOMAIN = "a and b have at most 64 elements each (any i64 values)."
BMB_DOMAIN = "spec fn dom(a: &[i64], b: &[i64]) -> bool { a.len() <= 64 && b.len() <= 64 }"


def domain(a, b):
    return len(a) <= 64 and len(b) <= 64


def model(a, b):
    best = 0
    prev = [0] * (len(b) + 1)
    for i in range(1, len(a) + 1):
        cur = [0] * (len(b) + 1)
        for j in range(1, len(b) + 1):
            if a[i - 1] == b[j - 1]:
                cur[j] = prev[j - 1] + 1
                best = max(best, cur[j])
        prev = cur
    return best


def cases(g):
    yield ([], [])
    yield ([1, 2, 3, 4], [9, 2, 3, 9])
    for _ in range(450):
        k = g.int(0, 2)
        if k == 0:
            a = [g.int(0, 2) for _ in range(g.int(0, 12))]
            b = [g.int(0, 2) for _ in range(g.int(0, 12))]
        elif k == 1:
            core = [g.int(-5, 5) for _ in range(g.int(0, 8))]
            a = [g.int(-5, 5) for _ in range(g.int(0, 6))] + core + [g.int(-5, 5) for _ in range(g.int(0, 6))]
            b = [g.int(-5, 5) for _ in range(g.int(0, 6))] + core + [g.int(-5, 5) for _ in range(g.int(0, 6))]
        else:
            a = [g.int(0, 1) for _ in range(g.int(40, 64))]
            b = [g.int(0, 1) for _ in range(g.int(40, 64))]
        yield (a, b)


# Two rows: run[j] is the length of the common run ending at a[i - 1] and
# b[j - 1]; runs are at most 64, the increment saturates only to spare the
# proof the bound.
BMB_REF = """#[effects(pure)]
fn common_substring(a: &[i64], b: &[i64]) -> (r: usize)
    requires a.len() <= 64 && b.len() <= 64,
{
    let mut prev: [usize; 65] = [0; 65];
    let mut cur: [usize; 65] = [0; 65];
    let mut best: usize = 0;
    let mut i: usize = 1;
    while i <= a.len()
        invariant i >= 1 && a.len() <= 64 && b.len() <= 64,
        decreases a.len() + 1 - i,
    {
        cur[0] = 0;
        let mut j: usize = 1;
        while j <= b.len()
            invariant j >= 1 && i <= a.len() && a.len() <= 64 && b.len() <= 64,
            decreases b.len() + 1 - j,
        {
            if a[i - 1] == b[j - 1] {
                cur[j] = prev[j - 1].saturating_add(1);
                if cur[j] > best {
                    best = cur[j];
                }
            } else {
                cur[j] = 0;
            }
            j = j + 1;
        }
        let mut t: usize = 0;
        while t <= b.len()
            invariant t <= b.len() + 1 && b.len() <= 64,
            decreases b.len() + 1 - t,
        {
            prev[t] = cur[t];
            t = t + 1;
        }
        i = i + 1;
    }
    best
}"""

# Weak (QuixBugs): the run is extended from the cell above.
BMB_WEAK = BMB_REF.replace("cur[j] = prev[j - 1].saturating_add(1);", "cur[j] = prev[j].saturating_add(1);")

RUST_REF = """pub fn common_substring(a: &[i64], b: &[i64]) -> usize {
    let mut best = 0;
    let mut prev = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        let mut cur = vec![0usize; b.len() + 1];
        for j in 1..=b.len() {
            if a[i - 1] == b[j - 1] {
                cur[j] = prev[j - 1] + 1;
                best = best.max(cur[j]);
            }
        }
        prev = cur;
    }
    best
}
"""

# Weak: the longest common subsequence; the tests have no gaps.
RUST_WEAK = """pub fn common_substring(a: &[i64], b: &[i64]) -> usize {
    let mut prev = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        let mut cur = vec![0usize; b.len() + 1];
        for j in 1..=b.len() {
            cur[j] = if a[i - 1] == b[j - 1] { prev[j - 1] + 1 } else { prev[j].max(cur[j - 1]) };
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs() {
        assert_eq!(common_substring(&[1, 2, 3, 4], &[9, 2, 3, 9]), 2);
        assert_eq!(common_substring(&[1, 2], &[3]), 0);
        assert_eq!(common_substring(&[], &[1]), 0);
        assert_eq!(common_substring(&[5, 6, 7], &[5, 6, 7]), 3);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs.
    "pub fn common_substring(a: &[i64], b: &[i64]) -> usize { let mut best = 0; let mut p = vec![0usize; b.len() + 1];"
    " for i in 1..=a.len() { let mut c = vec![0usize; b.len() + 1]; for j in 1..=b.len() { if a[i - 1] == b[j - 1] {"
    " c[j] = p[j] + 1; best = best.max(c[j]); } } p = c; } best }",
    "pub fn common_substring(a: &[i64], b: &[i64]) -> usize { let mut p = vec![0usize; b.len() + 1]; for i in 1..=a.len() {"
    " let mut c = vec![0usize; b.len() + 1]; for j in 1..=b.len() { c[j] = if a[i - 1] == b[j - 1] { p[j - 1] + 1 } else { p[j].max(c[j - 1]) }; }"
    " p = c; } p[b.len()] }",
    "pub fn common_substring(a: &[i64], b: &[i64]) -> usize { let mut best = 0; let mut p = vec![0usize; b.len() + 1];"
    " for i in 1..=a.len() { let mut c = vec![0usize; b.len() + 1]; for j in 1..=b.len() { if a[i - 1] == b[j - 1] {"
    " c[j] = p[j - 1] + 1; } } best = best.max(c[b.len()]); p = c; } best }",
    "pub fn common_substring(a: &[i64], b: &[i64]) -> usize { let mut best = 0; let mut p = vec![0usize; b.len() + 1];"
    " for i in 1..a.len() { let mut c = vec![0usize; b.len() + 1]; for j in 1..=b.len() { if a[i - 1] == b[j - 1] {"
    " c[j] = p[j - 1] + 1; best = best.max(c[j]); } } p = c; } best }",
]
