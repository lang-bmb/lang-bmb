from cases import I64_EDGES, I64_MIN

ID = "t09-div-floor"
NAME = "div_floor"
PARAMS = [("a", "i64"), ("b", "i64")]
RET = "i64"
STATEMENT = "Return a divided by b rounded toward negative infinity (floor division; note that Rust's `/` rounds toward zero). For example div_floor(7, 2) = 3, div_floor(-7, 2) = -4, div_floor(7, -2) = -4, div_floor(-7, -2) = 3."
DOMAIN = "a and b are i64 values with b != 0, excluding the single pair a = i64::MIN, b = -1 (whose quotient does not fit in i64)."
BMB_DOMAIN = "spec fn dom(a: i64, b: i64) -> bool { b != 0 && (a != -9223372036854775807 - 1 || b != -1) }"


def domain(a, b):
    return b != 0 and not (a == I64_MIN and b == -1)


def model(a, b):
    return a // b


def cases(g):
    for a in I64_EDGES:
        for b in I64_EDGES:
            yield (a, b)
    for _ in range(300):
        k = g.int(0, 2)
        b = g.int(-12, 12) if k == 0 else g.i64()
        yield (g.i64(), b)


BMB_REF = """spec fn div_ok(a: i64, b: i64) -> bool { b != 0 && (a != -9223372036854775807 - 1 || b != -1) }

#[effects(pure)]
#[uses(div_ok)]
fn div_floor(a: i64, b: i64) -> (r: i64)
    requires div_ok(a, b),
{
    let q: i64 = a / b;
    if a % b != 0 && (a < 0) != (b < 0) && q > -9223372036854775807 - 1 { q - 1 } else { q }
}"""

# Weak: adjusts only for negative a (right for b > 0 only).
BMB_WEAK = """spec fn div_ok(a: i64, b: i64) -> bool { b != 0 && (a != -9223372036854775807 - 1 || b != -1) }

#[effects(pure)]
#[uses(div_ok)]
fn div_floor(a: i64, b: i64) -> (r: i64)
    requires div_ok(a, b),
{
    let q: i64 = a / b;
    if a % b != 0 && a < 0 && q > -9223372036854775807 - 1 { q - 1 } else { q }
}"""

RUST_REF = """pub fn div_floor(a: i64, b: i64) -> i64 {
    let q = a / b;
    if a % b != 0 && ((a < 0) != (b < 0)) { q - 1 } else { q }
}
"""

RUST_WEAK = """pub fn div_floor(a: i64, b: i64) -> i64 {
    a.div_euclid(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floors() {
        assert_eq!(div_floor(7, 2), 3);
        assert_eq!(div_floor(-7, 2), -4);
        assert_eq!(div_floor(6, 3), 2);
        assert_eq!(div_floor(-6, 3), -2);
    }
}
"""

RUST_MUTANTS = [
    "pub fn div_floor(a: i64, b: i64) -> i64 { a.div_euclid(b) }",
    "pub fn div_floor(a: i64, b: i64) -> i64 { a / b }",
    "pub fn div_floor(a: i64, b: i64) -> i64 { (a as f64 / b as f64).floor() as i64 }",
    "pub fn div_floor(a: i64, b: i64) -> i64 { let q = a / b; if a % b != 0 && a < 0 { q - 1 } else { q } }",
]
