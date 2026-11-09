# QuixBugs `pascal` (rows of Pascal's triangle; defect: each row one entry
# short, `range(0, r)` for `range(0, r + 1)`), restated as one coefficient.
# The hard part is the width: C(n, k) fits in u64 for every n <= 67, but the
# multiplicative formula's intermediate `r * (n - i)` does not.
from cases import U64_MAX

ID = "h12-binomial"
NAME = "binomial"
PARAMS = [("n", "u64"), ("k", "u64")]
RET = "u64"
STATEMENT = ("Return the binomial coefficient C(n, k), the number of ways to choose k items out of n "
             "(entry k of row n of Pascal's triangle), and 0 when k > n. For example binomial(5, 2) = 10, "
             "binomial(4, 0) = 1, binomial(3, 4) = 0.")
DOMAIN = "n and k are u64 values with n <= 67 (so the result fits in u64); k may be any u64."
BMB_DOMAIN = "spec fn dom(n: u64, k: u64) -> bool { n <= 67 }"


def domain(n, k):
    return n <= 67


def model(n, k):
    if k > n:
        return 0
    r = 1
    for i in range(k):
        r = r * (n - i) // (i + 1)
    return r


def cases(g):
    for n in range(68):
        for k in range(n + 2):
            yield (n, k)
    for n in (0, 1, 33, 60, 66, 67):
        for k in (U64_MAX, 1 << 32, 68, 100):
            yield (n, k)


# Row by row (Pascal's rule) in a fixed array; entries never exceed C(67, 33),
# so the additions are saturating only to spare the proof the bound.
BMB_REF = """#[effects(pure)]
fn binomial(n: u64, k: u64) -> (r: u64)
    requires n <= 67,
{
    if k > n {
        return 0;
    }
    let mut row: [u64; 68] = [0; 68];
    row[0] = 1;
    let mut i: u64 = 1;
    while i <= n
        invariant i >= 1 && n <= 67,
        decreases n + 1 - i,
    {
        let mut j: u64 = i;
        while j >= 1
            invariant j <= i && i <= 67,
            decreases j,
        {
            let ju: usize = j as usize;
            row[ju] = row[ju].saturating_add(row[ju - 1]);
            j = j - 1;
        }
        i = i + 1;
    }
    row[k as usize]
}"""

# Weak (QuixBugs): each row one entry short, so the last entry of a row
# (k == n) stays 0 instead of 1.
BMB_WEAK = """#[effects(pure)]
fn binomial(n: u64, k: u64) -> (r: u64)
    requires n <= 67,
{
    if k > n {
        return 0;
    }
    let mut row: [u64; 68] = [0; 68];
    row[0] = 1;
    let mut i: u64 = 1;
    while i <= n
        invariant i >= 1 && n <= 67,
        decreases n + 1 - i,
    {
        let mut j: u64 = i - 1;
        while j >= 1
            invariant j < i && i <= 67,
            decreases j,
        {
            let ju: usize = j as usize;
            row[ju] = row[ju].saturating_add(row[ju - 1]);
            j = j - 1;
        }
        i = i + 1;
    }
    row[k as usize]
}"""

RUST_REF = """pub fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r: u128 = 1;
    for i in 0..k {
        r = r * (n - i) as u128 / (i + 1) as u128;
    }
    r as u64
}
"""

# Weak: the multiplicative formula in u64, whose tests stay at small n.
RUST_WEAK = """pub fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    let mut r: u64 = 1;
    for i in 0..k {
        r = r * (n - i) / (i + 1);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small() {
        assert_eq!(binomial(5, 2), 10);
        assert_eq!(binomial(4, 0), 1);
        assert_eq!(binomial(3, 4), 0);
        assert_eq!(binomial(10, 10), 1);
        assert_eq!(binomial(20, 10), 184756);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs: each row one entry short.
    "pub fn binomial(n: u64, k: u64) -> u64 { if k > n { return 0; } let mut row = vec![1u64];"
    " for r in 1..=n { let mut next = vec![1u64]; for c in 1..r { next.push(row[c as usize - 1] + row[c as usize]); }"
    " row = next; } row.get(k as usize).copied().unwrap_or(0) }",
    "pub fn binomial(n: u64, k: u64) -> u64 { if k > n { return 0; } let mut r: u64 = 1;"
    " for i in 0..k { r = r * (n - i) / (i + 1); } r }",
    "pub fn binomial(n: u64, k: u64) -> u64 { let mut r: u128 = 1; for i in 0..k.min(n) {"
    " r = r * (n - i) as u128 / (i + 1) as u128; } r as u64 }",
    "pub fn binomial(n: u64, k: u64) -> u64 { if k > n { return 0; } let k = k.min(n - k); let mut r: u64 = 1;"
    " for i in 0..k { r = r / (i + 1) * (n - i); } r }",
]
