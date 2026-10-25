from cases import I64_EDGES

ID = "t02-midpoint"
NAME = "midpoint"
PARAMS = [("a", "i64"), ("b", "i64")]
RET = "i64"
STATEMENT = "Return the mean of a and b rounded toward negative infinity, that is floor((a + b) / 2) computed exactly (the exact sum a + b may not fit in i64)."
DOMAIN = "a and b are any i64 values."
BMB_DOMAIN = None


def domain(a, b):
    return True


def model(a, b):
    return (a + b) // 2


def cases(g):
    for a in I64_EDGES:
        for b in I64_EDGES:
            yield (a, b)
    for _ in range(300):
        yield (g.i64(), g.i64())


BMB_REF = """spec fn is_mid(a: i64, b: i64, r: i64) -> bool { 2 * r <= a + b && a + b <= 2 * r + 1 }

#[effects(pure)]
#[uses(is_mid)]
fn midpoint(a: i64, b: i64) -> (r: i64)
    ensures is_mid(a, b, r),
{
    if (a >= 0) == (b >= 0) {
        if a <= b { a + (b - a) / 2 } else { b + (a - b) / 2 }
    } else {
        let s: i64 = a + b;
        let q: i64 = s / 2;
        if s % 2 != 0 && s < 0 { q - 1 } else { q }
    }
}"""

# Weak: halves each argument first (loses the carry of two odd halves).
BMB_WEAK = """#[effects(pure)]
fn midpoint(a: i64, b: i64) -> (r: i64) {
    a / 2 + b / 2
}"""

RUST_REF = """pub fn midpoint(a: i64, b: i64) -> i64 {
    (a >> 1) + (b >> 1) + (a & b & 1)
}
"""

RUST_WEAK = """pub fn midpoint(a: i64, b: i64) -> i64 {
    (a + b) / 2
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small() {
        assert_eq!(midpoint(2, 4), 3);
        assert_eq!(midpoint(1, 4), 2);
        assert_eq!(midpoint(0, 10), 5);
        assert_eq!(midpoint(-4, -2), -3);
    }
}
"""

RUST_MUTANTS = [
    "pub fn midpoint(a: i64, b: i64) -> i64 { (a + b) / 2 }",
    "pub fn midpoint(a: i64, b: i64) -> i64 { a / 2 + b / 2 }",
    "pub fn midpoint(a: i64, b: i64) -> i64 { (a >> 1) + (b >> 1) }",
    "pub fn midpoint(a: i64, b: i64) -> i64 { ((a as i128 + b as i128) / 2) as i64 }",
]
