from cases import I64_MAX

ID = "t24-pow-mod"
NAME = "pow_mod"
PARAMS = [("base", "i64"), ("exp", "i64"), ("m", "i64")]
RET = "i64"
STATEMENT = "Return base raised to the power exp, modulo m: the value (base^exp) mod m in [0, m), computed exactly (0^0 = 1, so pow_mod(0, 0, m) = 1 mod m)."
DOMAIN = "base >= 0, exp >= 0 and 1 <= m <= 3_000_000_000 (all i64; base and exp may be as large as i64::MAX)."
BMB_DOMAIN = "spec fn dom(base: i64, exp: i64, m: i64) -> bool { base >= 0 && exp >= 0 && 1 <= m && m <= 3000000000 }"


def domain(base, exp, m):
    return base >= 0 and exp >= 0 and 1 <= m <= 3000000000


def model(base, exp, m):
    return pow(base, exp, m)


def cases(g):
    for m in (1, 2, 3000000000, 2999999999, 1000000007):
        for base in (0, 1, 2, m - 1, m, I64_MAX):
            for exp in (0, 1, 2, 63, I64_MAX):
                yield (base, exp, m)
    for _ in range(250):
        m = g.choice([g.int(1, 20), g.int(1, 3000000000), 3000000000 - g.int(0, 10)])
        base = g.choice([g.int(0, 20), g.int(0, 3000000000), g.int(0, I64_MAX)])
        exp = g.choice([g.int(0, 10), g.int(0, 1000), g.int(0, I64_MAX)])
        yield (base, exp, m)


BMB_REF = """spec fn mod_ok(m: i64) -> bool { 1 <= m && m <= 3000000000 }

spec fn below(x: i64, m: i64) -> bool { 0 <= x && x < m }

spec fn pow_args(base: i64, exp: i64, m: i64) -> bool { base >= 0 && exp >= 0 && 1 <= m && m <= 3000000000 }

#[effects(pure)]
#[uses(mod_ok, below)]
fn mul_mod(x: i64, y: i64, m: i64) -> (r: i64)
    requires mod_ok(m), below(x, m), below(y, m),
    ensures below(r, m),
{
    let mut acc: i64 = 0;
    let mut xx: i64 = x;
    let mut yy: i64 = y;
    while yy > 0
        invariant mod_ok(m), below(acc, m), below(xx, m), yy >= 0,
        decreases yy,
    {
        if yy % 2 == 1 {
            acc = acc + xx;
            if acc >= m {
                acc = acc - m;
            }
        }
        xx = xx + xx;
        if xx >= m {
            xx = xx - m;
        }
        yy = yy / 2;
    }
    acc
}

#[effects(pure)]
#[uses(pow_args, mod_ok, below)]
fn pow_mod(base: i64, exp: i64, m: i64) -> (r: i64)
    requires pow_args(base, exp, m),
{
    let mut acc: i64 = 1 % m;
    let mut b: i64 = base % m;
    let mut e: i64 = exp;
    while e > 0
        invariant mod_ok(m), below(acc, m), below(b, m), e >= 0,
        decreases e,
    {
        if e % 2 == 1 {
            acc = mul_mod(acc, b, m);
        }
        b = mul_mod(b, b, m);
        e = e / 2;
    }
    acc
}"""

# Weak: stops before the highest bit of exp.
BMB_WEAK = BMB_REF.replace("    while e > 0\n        invariant mod_ok(m), below(acc, m), below(b, m), e >= 0,",
                           "    while e > 1\n        invariant mod_ok(m), below(acc, m), below(b, m), e >= 0,")

RUST_REF = """pub fn pow_mod(base: i64, exp: i64, m: i64) -> i64 {
    let m = m as i128;
    let mut acc: i128 = 1 % m;
    let mut b = base as i128 % m;
    let mut e = exp;
    while e > 0 {
        if e % 2 == 1 {
            acc = acc * b % m;
        }
        b = b * b % m;
        e /= 2;
    }
    acc as i64
}
"""

RUST_WEAK = """pub fn pow_mod(base: i64, exp: i64, m: i64) -> i64 {
    let mut acc = 1;
    let mut b = base;
    let mut e = exp;
    while e > 0 {
        if e % 2 == 1 {
            acc = acc * b % m;
        }
        b = b * b % m;
        e /= 2;
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn powers() {
        assert_eq!(pow_mod(2, 10, 1000), 24);
        assert_eq!(pow_mod(3, 0, 7), 1);
        assert_eq!(pow_mod(5, 3, 13), 8);
    }
}
"""

RUST_MUTANTS = [
    "pub fn pow_mod(base: i64, exp: i64, m: i64) -> i64 { let mut acc = 1; let mut b = base; let mut e = exp; while e > 0 { if e % 2 == 1 { acc = acc * b % m; } b = b * b % m; e /= 2; } acc }",
    "pub fn pow_mod(base: i64, exp: i64, m: i64) -> i64 { let m = m as i128; let mut acc: i128 = 1; let mut b = base as i128 % m; let mut e = exp; while e > 0 { if e % 2 == 1 { acc = acc * b % m; } b = b * b % m; e /= 2; } acc as i64 }",
    "pub fn pow_mod(base: i64, exp: i64, m: i64) -> i64 { let m = m as i128; let mut acc: i128 = 1 % m; let mut b = base as i128 % m; let mut e = exp; while e > 1 { if e % 2 == 1 { acc = acc * b % m; } b = b * b % m; e /= 2; } acc as i64 }",
    "pub fn pow_mod(base: i64, exp: i64, m: i64) -> i64 { let m = m as i128; let mut acc: i128 = 1 % m; let mut b = base as i128 % m; let mut e = exp; while e > 0 { if e % 2 == 1 { acc = acc * b % m; } b = b * b; e /= 2; } acc as i64 }",
]
