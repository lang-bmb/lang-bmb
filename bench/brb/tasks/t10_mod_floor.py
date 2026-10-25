from cases import I64_EDGES

ID = "t10-mod-floor"
NAME = "mod_floor"
PARAMS = [("a", "i64"), ("b", "i64")]
RET = "i64"
STATEMENT = ("Return a - b * floor(a / b), the remainder of floor division: it is 0 or has the sign of b, and its absolute value is "
             "smaller than |b|. For example mod_floor(7, 3) = 1, mod_floor(-7, 3) = 2, mod_floor(7, -3) = -2, mod_floor(-7, -3) = -1.")
DOMAIN = "a and b are i64 values with b != 0."
BMB_DOMAIN = "spec fn dom(a: i64, b: i64) -> bool { b != 0 }"


def domain(a, b):
    return b != 0


def model(a, b):
    return a % b  # Python's % is the floor remainder


def cases(g):
    for a in I64_EDGES:
        for b in I64_EDGES:
            yield (a, b)
    for _ in range(300):
        k = g.int(0, 2)
        b = g.int(-12, 12) if k == 0 else g.i64()
        yield (g.i64(), b)


BMB_REF = """spec fn nonzero(b: i64) -> bool { b != 0 }

#[effects(pure)]
#[uses(nonzero)]
fn mod_floor(a: i64, b: i64) -> (r: i64)
    requires nonzero(b),
{
    if b == -1 {
        0
    } else {
        let m: i64 = a % b;
        if m != 0 && (m < 0) != (b < 0) { m + b } else { m }
    }
}"""

# Weak: fixes negative remainders for b > 0 only.
BMB_WEAK = """spec fn nonzero(b: i64) -> bool { b != 0 }

#[effects(pure)]
#[uses(nonzero)]
fn mod_floor(a: i64, b: i64) -> (r: i64)
    requires nonzero(b),
{
    if b == -1 {
        0
    } else {
        let m: i64 = a % b;
        if m < 0 && b > 0 { m + b } else { m }
    }
}"""

RUST_REF = """pub fn mod_floor(a: i64, b: i64) -> i64 {
    if b == -1 {
        return 0;
    }
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) { r + b } else { r }
}
"""

RUST_WEAK = """pub fn mod_floor(a: i64, b: i64) -> i64 {
    ((a % b) + b) % b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remainders() {
        assert_eq!(mod_floor(7, 3), 1);
        assert_eq!(mod_floor(-7, 3), 2);
        assert_eq!(mod_floor(7, -3), -2);
        assert_eq!(mod_floor(-7, -3), -1);
    }
}
"""

RUST_MUTANTS = [
    "pub fn mod_floor(a: i64, b: i64) -> i64 { ((a % b) + b) % b }",
    "pub fn mod_floor(a: i64, b: i64) -> i64 { a.rem_euclid(b) }",
    "pub fn mod_floor(a: i64, b: i64) -> i64 { let r = a % b; if r != 0 && ((r < 0) != (b < 0)) { r + b } else { r } }",
    "pub fn mod_floor(a: i64, b: i64) -> i64 { if b == -1 { return 0; } let r = a % b; if r < 0 && b > 0 { r + b } else { r } }",
]
