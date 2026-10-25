from cases import I64_MIN

ID = "t12-second-max"
NAME = "second_max"
PARAMS = [("a", "&[i64]")]
RET = "i64"
STATEMENT = ("Return the second largest distinct value of a: the largest value that is strictly smaller than the maximum of a. "
             "If a has fewer than two distinct values (including when a is empty), return i64::MIN.")
DOMAIN = "a is any slice of i64 values (possibly empty)."
BMB_DOMAIN = None


def domain(a):
    return True


def model(a):
    s = sorted(set(a))
    return s[-2] if len(s) >= 2 else I64_MIN


def cases(g):
    yield ([],)
    yield ([I64_MIN, 5],)
    yield ([5, 5, 5],)
    for a in g.arrays(400, 12):
        yield (a,)


BMB_REF = """#[effects(pure)]
fn second_max(a: &[i64]) -> (r: i64) {
    let mut first: i64 = -9223372036854775807 - 1;
    let mut second: i64 = -9223372036854775807 - 1;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(),
        decreases a.len() - k,
    {
        let x: i64 = a[k];
        if x > first {
            second = first;
            first = x;
        } else {
            if x < first && x > second {
                second = x;
            }
        }
        k = k + 1;
    }
    second
}"""

# Weak: a repeated maximum also counts as the second largest.
BMB_WEAK = BMB_REF.replace("if x > first {", "if x >= first {")

RUST_REF = """pub fn second_max(a: &[i64]) -> i64 {
    let mut first = i64::MIN;
    let mut second = i64::MIN;
    for &x in a {
        if x > first {
            second = first;
            first = x;
        } else if x < first && x > second {
            second = x;
        }
    }
    second
}
"""

RUST_WEAK = """pub fn second_max(a: &[i64]) -> i64 {
    let mut v = a.to_vec();
    v.sort();
    if v.len() < 2 { i64::MIN } else { v[v.len() - 2] }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn second() {
        assert_eq!(second_max(&[3, 9, 4]), 4);
        assert_eq!(second_max(&[1]), i64::MIN);
        assert_eq!(second_max(&[]), i64::MIN);
        assert_eq!(second_max(&[-1, -7]), -7);
    }
}
"""

RUST_MUTANTS = [
    "pub fn second_max(a: &[i64]) -> i64 { let mut v = a.to_vec(); v.sort(); if v.len() < 2 { i64::MIN } else { v[v.len() - 2] } }",
    "pub fn second_max(a: &[i64]) -> i64 { let mut f = i64::MIN; let mut s = i64::MIN; for &x in a { if x > f { s = f; f = x; } else if x > s { s = x; } } s }",
    "pub fn second_max(a: &[i64]) -> i64 { let mut v = a.to_vec(); v.sort(); v.dedup(); if v.len() < 2 { *v.last().unwrap_or(&i64::MIN) } else { v[v.len() - 2] } }",
    "pub fn second_max(a: &[i64]) -> i64 { let mut f = i64::MIN; let mut s = i64::MIN; for &x in a { if x > f { f = x; } else if x < f && x > s { s = x; } } s }",
]
