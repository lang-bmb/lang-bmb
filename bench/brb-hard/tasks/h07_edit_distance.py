# QuixBugs `levenshtein` (defect: `1 + levenshtein(source[1:], target[1:])`
# when the first characters match; a match costs nothing).

ID = "h07-edit-distance"
NAME = "edit_distance"
PARAMS = [("a", "&[i64]"), ("b", "&[i64]")]
RET = "usize"
STATEMENT = ("Return the Levenshtein distance between the sequences a and b: the least number of single-element "
             "edits (insert an element, delete an element, or replace an element by another) that turn a into b. "
             "For example edit_distance([1, 2, 3], [1, 3]) = 1, edit_distance([5, 6], [7, 8]) = 2, "
             "edit_distance([], [4, 4]) = 2.")
DOMAIN = "a and b have at most 64 elements each (any i64 values)."
BMB_DOMAIN = "spec fn dom(a: &[i64], b: &[i64]) -> bool { a.len() <= 64 && b.len() <= 64 }"


def domain(a, b):
    return len(a) <= 64 and len(b) <= 64


def model(a, b):
    prev = list(range(len(b) + 1))
    for i in range(1, len(a) + 1):
        cur = [i] + [0] * len(b)
        for j in range(1, len(b) + 1):
            cur[j] = min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (a[i - 1] != b[j - 1]))
        prev = cur
    return prev[len(b)]


def cases(g):
    yield ([], [])
    yield ([1, 2, 3], [1, 3])
    for _ in range(450):
        k = g.int(0, 2)
        if k == 0:
            a = [g.int(0, 2) for _ in range(g.int(0, 10))]
            b = [g.int(0, 2) for _ in range(g.int(0, 10))]
        elif k == 1:
            a = g.array(20, g.elem_mixed())
            b = list(a)
            for _ in range(g.int(0, 4)):
                op = g.int(0, 2)
                if op == 0 and b:
                    del b[g.int(0, len(b) - 1)]
                elif op == 1:
                    b.insert(g.int(0, len(b)), g.int(-3, 3))
                elif b:
                    b[g.int(0, len(b) - 1)] = g.int(-3, 3)
        else:
            a = [g.int(0, 3) for _ in range(g.int(40, 64))]
            b = [g.int(0, 3) for _ in range(g.int(40, 64))]
        yield (a, b)


# Two rows of the distance table. Entries are at most 128, so the
# increments saturate only to spare the proof the bound.
BMB_REF = """#[effects(pure)]
fn edit_distance(a: &[i64], b: &[i64]) -> (r: usize)
    requires a.len() <= 64 && b.len() <= 64,
{
    let mut prev: [usize; 65] = [0; 65];
    let mut cur: [usize; 65] = [0; 65];
    let mut j: usize = 0;
    while j <= b.len()
        invariant j <= b.len() + 1 && b.len() <= 64,
        decreases b.len() + 1 - j,
    {
        prev[j] = j;
        j = j + 1;
    }
    let mut i: usize = 1;
    while i <= a.len()
        invariant i >= 1 && a.len() <= 64 && b.len() <= 64,
        decreases a.len() + 1 - i,
    {
        cur[0] = i;
        let mut k: usize = 1;
        while k <= b.len()
            invariant k >= 1 && i <= a.len() && a.len() <= 64 && b.len() <= 64,
            decreases b.len() + 1 - k,
        {
            let del: usize = prev[k].saturating_add(1);
            let ins: usize = cur[k - 1].saturating_add(1);
            let sub: usize = if a[i - 1] == b[k - 1] { prev[k - 1] } else { prev[k - 1].saturating_add(1) };
            let mut m: usize = del;
            if ins < m {
                m = ins;
            }
            if sub < m {
                m = sub;
            }
            cur[k] = m;
            k = k + 1;
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
    prev[b.len()]
}"""

# Weak (QuixBugs): a match still costs one edit.
BMB_WEAK = BMB_REF.replace("if a[i - 1] == b[k - 1] { prev[k - 1] } else { prev[k - 1].saturating_add(1) }",
                           "prev[k - 1].saturating_add(1)")

RUST_REF = """pub fn edit_distance(a: &[i64], b: &[i64]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let sub = prev[j - 1] + usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(sub);
        }
        prev = cur;
    }
    prev[b.len()]
}
"""

# Weak: only insertions and deletions (a replacement counts two); the tests
# never need a replacement.
RUST_WEAK = """pub fn edit_distance(a: &[i64], b: &[i64]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let keep = if a[i - 1] == b[j - 1] { prev[j - 1] } else { usize::MAX };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(keep);
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distances() {
        assert_eq!(edit_distance(&[1, 2, 3], &[1, 3]), 1);
        assert_eq!(edit_distance(&[], &[4, 4]), 2);
        assert_eq!(edit_distance(&[1, 2], &[1, 2]), 0);
        assert_eq!(edit_distance(&[1, 2], &[1, 2, 3]), 1);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs.
    "pub fn edit_distance(a: &[i64], b: &[i64]) -> usize { let mut p: Vec<usize> = (0..=b.len()).collect(); for i in 1..=a.len() {"
    " let mut c = vec![i; b.len() + 1]; for j in 1..=b.len() { c[j] = (p[j] + 1).min(c[j - 1] + 1).min(p[j - 1] + 1); } p = c; } p[b.len()] }",
    "pub fn edit_distance(a: &[i64], b: &[i64]) -> usize { let mut p: Vec<usize> = (0..=b.len()).collect(); for i in 1..=a.len() {"
    " let mut c = vec![i; b.len() + 1]; for j in 1..=b.len() { let k = if a[i - 1] == b[j - 1] { p[j - 1] } else { usize::MAX };"
    " c[j] = (p[j] + 1).min(c[j - 1] + 1).min(k); } p = c; } p[b.len()] }",
    "pub fn edit_distance(a: &[i64], b: &[i64]) -> usize { let n = a.len().min(b.len());"
    " (0..n).filter(|&i| a[i] != b[i]).count() + a.len().max(b.len()) - n }",
    "pub fn edit_distance(a: &[i64], b: &[i64]) -> usize { let mut p: Vec<usize> = (0..=b.len()).collect(); for i in 1..=a.len() {"
    " let mut c = vec![0; b.len() + 1]; for j in 1..=b.len() { let s = p[j - 1] + usize::from(a[i - 1] != b[j - 1]);"
    " c[j] = (p[j] + 1).min(c[j - 1] + 1).min(s); } p = c; } p[b.len()] }",
]
