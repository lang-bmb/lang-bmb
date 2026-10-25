import math

from cases import I64_EDGES, I64_MIN

ID = "t16-gcd"
NAME = "gcd"
PARAMS = [("a", "i64"), ("b", "i64")]
RET = "i64"
STATEMENT = "Return the greatest common divisor of |a| and |b|, which is never negative; gcd(0, 0) = 0 and gcd(a, 0) = |a|."
DOMAIN = "a and b are i64 values greater than i64::MIN (so |a| and |b| fit in i64)."
BMB_DOMAIN = "spec fn dom(a: i64, b: i64) -> bool { a > -9223372036854775807 - 1 && b > -9223372036854775807 - 1 }"


def domain(a, b):
    return a > I64_MIN and b > I64_MIN


def model(a, b):
    return math.gcd(a, b)


def cases(g):
    for a in I64_EDGES:
        for b in I64_EDGES:
            yield (a, b)
    for _ in range(300):
        d = g.choice([1, 2, 6, 7, 1 << 20, 999983])
        k = g.int(0, 2)
        if k == 0:
            yield (d * g.int(-1000, 1000), d * g.int(-1000, 1000))
        elif k == 1:
            yield (g.int(-100, 100), g.int(-100, 100))
        else:
            yield (g.i64(), g.i64())


BMB_REF = """spec fn not_min(a: i64, b: i64) -> bool { a > -9223372036854775807 - 1 && b > -9223372036854775807 - 1 }

#[effects(pure)]
#[uses(not_min)]
fn gcd(a: i64, b: i64) -> (r: i64)
    requires not_min(a, b),
{
    let mut x: i64 = if a < 0 { 0 - a } else { a };
    let mut y: i64 = if b < 0 { 0 - b } else { b };
    while y > 0
        invariant x >= 0, y >= 0,
        decreases y,
    {
        let t: i64 = x % y;
        x = y;
        y = t;
    }
    x
}"""

# Weak: the sign of a leaks into the result.
BMB_WEAK = BMB_REF.replace("    x\n}", "    if a < 0 { 0 - x } else { x }\n}")

RUST_REF = """pub fn gcd(a: i64, b: i64) -> i64 {
    let (mut x, mut y) = (a.abs(), b.abs());
    while y > 0 {
        let t = x % y;
        x = y;
        y = t;
    }
    x
}
"""

RUST_WEAK = """pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gcds() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(gcd(7, 0), 7);
        assert_eq!(gcd(0, 0), 0);
        assert_eq!(gcd(17, 5), 1);
    }
}
"""

RUST_MUTANTS = [
    "pub fn gcd(mut a: i64, mut b: i64) -> i64 { while b != 0 { let t = a % b; a = b; b = t; } a }",
    "pub fn gcd(a: i64, b: i64) -> i64 { if a == 0 || b == 0 { return 0; } let (mut x, mut y) = (a.abs(), b.abs()); while y > 0 { let t = x % y; x = y; y = t; } x }",
    "pub fn gcd(a: i64, b: i64) -> i64 { let (mut x, mut y) = (a.abs(), b.abs()); while y > 1 { let t = x % y; x = y; y = t; } x }",
    "pub fn gcd(a: i64, b: i64) -> i64 { if a == 0 { return b; } let (mut x, mut y) = (a.abs(), b.abs()); while y > 0 { let t = x % y; x = y; y = t; } x }",
]
