from cases import I64_EDGES

ID = "t18-count-digits"
NAME = "count_digits"
PARAMS = [("n", "i64")]
RET = "i64"
STATEMENT = "Return the number of decimal digits of |n| (the sign does not count); 0 has one digit."
DOMAIN = "n is any i64 value."
BMB_DOMAIN = None


def domain(n):
    return True


def model(n):
    return len(str(abs(n)))


def cases(g):
    for e in I64_EDGES:
        yield (e,)
    for d in range(0, 19):
        p = 10 ** d
        for v in (p - 1, p, p + 1):
            yield (v,)
            yield (-v,)
    for _ in range(200):
        yield (g.i64(),)


BMB_REF = """#[effects(pure)]
fn count_digits(n: i64) -> (r: i64) {
    let m: i64 = if n > 0 { 0 - n } else { n };
    if m > -10 { 1 } else if m > -100 { 2 } else if m > -1000 { 3 } else if m > -10000 { 4 }
    else if m > -100000 { 5 } else if m > -1000000 { 6 } else if m > -10000000 { 7 }
    else if m > -100000000 { 8 } else if m > -1000000000 { 9 } else if m > -10000000000 { 10 }
    else if m > -100000000000 { 11 } else if m > -1000000000000 { 12 } else if m > -10000000000000 { 13 }
    else if m > -100000000000000 { 14 } else if m > -1000000000000000 { 15 } else if m > -10000000000000000 { 16 }
    else if m > -100000000000000000 { 17 } else if m > -1000000000000000000 { 18 } else { 19 }
}"""

# Weak: `>=` is off by one at every power of ten.
BMB_WEAK = BMB_REF.replace("m > -", "m >= -")

RUST_REF = """pub fn count_digits(n: i64) -> i64 {
    let mut m = n.unsigned_abs();
    let mut c = 1;
    while m >= 10 {
        m /= 10;
        c += 1;
    }
    c
}
"""

RUST_WEAK = """pub fn count_digits(n: i64) -> i64 {
    n.abs().to_string().len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digits() {
        assert_eq!(count_digits(0), 1);
        assert_eq!(count_digits(9), 1);
        assert_eq!(count_digits(10), 2);
        assert_eq!(count_digits(-12345), 5);
    }
}
"""

RUST_MUTANTS = [
    "pub fn count_digits(n: i64) -> i64 { n.abs().to_string().len() as i64 }",
    "pub fn count_digits(n: i64) -> i64 { let mut m = n.unsigned_abs(); let mut c = 1; while m > 10 { m /= 10; c += 1; } c }",
    "pub fn count_digits(n: i64) -> i64 { if n == 0 { 1 } else { (n.unsigned_abs() as f64).log10().floor() as i64 + 1 } }",
    "pub fn count_digits(n: i64) -> i64 { n.to_string().len() as i64 }",
]
