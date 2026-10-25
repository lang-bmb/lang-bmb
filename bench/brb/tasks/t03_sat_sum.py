from cases import I64_MAX, I64_MIN

ID = "t03-sat-sum"
NAME = "sat_sum"
PARAMS = [("a", "&[i64]")]
RET = "i64"
STATEMENT = ("Add the elements of a from left to right, starting from 0, where every single addition saturates: "
             "if the exact running total would exceed i64::MAX it becomes i64::MAX, if it would fall below i64::MIN "
             "it becomes i64::MIN. Return the final running total.")
DOMAIN = "a is any slice of i64 values (possibly empty)."
BMB_DOMAIN = None


def domain(a):
    return True


def model(a):
    s = 0
    for x in a:
        s = max(I64_MIN, min(I64_MAX, s + x))
    return s


def cases(g):
    yield ([],)
    yield ([I64_MAX, 1, -1],)
    yield ([I64_MIN, -1, 1],)
    yield ([I64_MAX, I64_MAX, I64_MIN],)
    yield ([I64_MIN, I64_MIN, I64_MAX, I64_MAX],)
    for a in g.arrays(350, 12):
        yield (a,)


BMB_REF = """#[effects(pure)]
fn sat_add(s: i64, x: i64) -> (r: i64) {
    if x > 0 && s > 9223372036854775807 - x {
        9223372036854775807
    } else {
        if x < 0 && s < -9223372036854775807 - 1 - x { -9223372036854775807 - 1 } else { s + x }
    }
}

#[effects(pure)]
fn sat_sum(a: &[i64]) -> (r: i64) {
    let mut s: i64 = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(),
        decreases a.len() - k,
    {
        s = sat_add(s, a[k]);
        k = k + 1;
    }
    s
}"""

# Weak: an addition that would overflow is skipped instead of saturating.
BMB_WEAK = """#[effects(pure)]
fn sat_sum(a: &[i64]) -> (r: i64) {
    let mut s: i64 = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(),
        decreases a.len() - k,
    {
        let x: i64 = a[k];
        if x > 0 && s > 9223372036854775807 - x {
            s = s;
        } else {
            if x < 0 && s < -9223372036854775807 - 1 - x { s = s; } else { s = s + x; }
        }
        k = k + 1;
    }
    s
}"""

RUST_REF = """pub fn sat_sum(a: &[i64]) -> i64 {
    a.iter().fold(0i64, |s, &x| s.saturating_add(x))
}
"""

RUST_WEAK = """pub fn sat_sum(a: &[i64]) -> i64 {
    let s: i128 = a.iter().map(|&x| x as i128).sum();
    s.clamp(i64::MIN as i128, i64::MAX as i128) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sums() {
        assert_eq!(sat_sum(&[]), 0);
        assert_eq!(sat_sum(&[1, 2, 3]), 6);
        assert_eq!(sat_sum(&[i64::MAX, 1]), i64::MAX);
        assert_eq!(sat_sum(&[i64::MIN, -5]), i64::MIN);
    }
}
"""

RUST_MUTANTS = [
    "pub fn sat_sum(a: &[i64]) -> i64 { a.iter().sum() }",
    "pub fn sat_sum(a: &[i64]) -> i64 { a.iter().fold(0i64, |s, &x| s.wrapping_add(x)) }",
    "pub fn sat_sum(a: &[i64]) -> i64 { let s: i128 = a.iter().map(|&x| x as i128).sum(); s.clamp(i64::MIN as i128, i64::MAX as i128) as i64 }",
    "pub fn sat_sum(a: &[i64]) -> i64 { a.iter().fold(0i64, |s, &x| s.checked_add(x).unwrap_or(s)) }",
]
