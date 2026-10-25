import math

from cases import I64_MAX

ID = "t17-isqrt"
NAME = "isqrt"
PARAMS = [("n", "i64")]
RET = "i64"
STATEMENT = "Return the integer square root of n: the largest r >= 0 with r * r <= n (computed exactly)."
DOMAIN = "n is any i64 value with n >= 0."
BMB_DOMAIN = "spec fn dom(n: i64) -> bool { n >= 0 }"


def domain(n):
    return n >= 0


def model(n):
    return math.isqrt(n)


def cases(g):
    for n in range(0, 40):
        yield (n,)
    yield (I64_MAX,)
    yield (I64_MAX - 1,)
    for _ in range(300):
        k = g.int(0, 3)
        r = g.int(1, 3037000499) if k < 3 else g.int(1, 100000)
        sq = r * r
        yield (g.choice([sq, sq - 1, min(sq + 1, I64_MAX), min(sq + 2 * r, I64_MAX)]),)
    for _ in range(100):
        yield (g.int(0, I64_MAX),)


BMB_REF = """spec fn nonneg(n: i64) -> bool { n >= 0 }

#[effects(pure)]
#[uses(nonneg)]
fn isqrt(n: i64) -> (r: i64)
    requires nonneg(n),
{
    let mut lo: i64 = 0;
    let mut hi: i64 = 3037000499;
    if hi > n {
        hi = n;
    }
    while lo < hi
        invariant 0 <= lo, lo <= hi, hi <= 3037000499,
        decreases hi - lo,
    {
        let mid: i64 = lo + (hi - lo + 1) / 2;
        if mid <= n / mid {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}"""

# Weak: `<` misses exact squares.
BMB_WEAK = BMB_REF.replace("if mid <= n / mid {", "if mid < n / mid {")

RUST_REF = """pub fn isqrt(n: i64) -> i64 {
    let (mut lo, mut hi) = (0i64, n.min(3_037_000_499));
    while lo < hi {
        let mid = lo + (hi - lo + 1) / 2;
        if mid <= n / mid {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}
"""

RUST_WEAK = """pub fn isqrt(n: i64) -> i64 {
    (n as f64).sqrt() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roots() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(15), 3);
        assert_eq!(isqrt(16), 4);
        assert_eq!(isqrt(1_000_000), 1000);
    }
}
"""

RUST_MUTANTS = [
    "pub fn isqrt(n: i64) -> i64 { (n as f64).sqrt() as i64 }",
    "pub fn isqrt(n: i64) -> i64 { let (mut lo, mut hi) = (0i64, n.min(3_037_000_499)); while lo < hi { let mid = lo + (hi - lo + 1) / 2; if mid < n / mid { lo = mid; } else { hi = mid - 1; } } lo }",
    "pub fn isqrt(n: i64) -> i64 { let (mut lo, mut hi) = (0i64, n); while lo < hi { let mid = lo + (hi - lo + 1) / 2; if mid * mid <= n { lo = mid; } else { hi = mid - 1; } } lo }",
    "pub fn isqrt(n: i64) -> i64 { let r = (n as f64).sqrt() as i64; if (r + 1) * (r + 1) <= n { r + 1 } else { r } }",
]
