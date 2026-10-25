from cases import I64_EDGES

ID = "t01-clamp"
NAME = "clamp"
PARAMS = [("x", "i64"), ("lo", "i64"), ("hi", "i64")]
RET = "i64"
STATEMENT = "Return x limited to the closed range [lo, hi]: lo if x < lo, hi if x > hi, otherwise x."
DOMAIN = "x, lo, hi are any i64 values with lo <= hi."
BMB_DOMAIN = "spec fn dom(x: i64, lo: i64, hi: i64) -> bool { lo <= hi }"


def domain(x, lo, hi):
    return lo <= hi


def model(x, lo, hi):
    return min(max(x, lo), hi)


def cases(g):
    for x in I64_EDGES:
        for lo, hi in [(-1, 1), (0, 0), (I64_EDGES[0], I64_EDGES[-1]), (-10, -3), (3, 10), (I64_EDGES[-2], I64_EDGES[-1])]:
            yield (x, lo, hi)
    for _ in range(300):
        a, b = sorted([g.i64(), g.i64()])
        yield (g.choice([g.i64(), a, b, g.int(-12, 12)]), a, b)


BMB_REF = """spec fn ordered(lo: i64, hi: i64) -> bool { lo <= hi }

spec fn clamped(x: i64, lo: i64, hi: i64, r: i64) -> bool { (x < lo ==> r == lo) && (x > hi ==> r == hi) && ((lo <= x && x <= hi) ==> r == x) }

#[effects(pure)]
#[uses(ordered, clamped)]
fn clamp(x: i64, lo: i64, hi: i64) -> (r: i64)
    requires ordered(lo, hi),
    ensures clamped(x, lo, hi, r),
{
    if x < lo { lo } else { if x > hi { hi } else { x } }
}"""

# Weak: returns hi - 1 above the range (off by one), proved only in range.
BMB_WEAK = """spec fn ordered(lo: i64, hi: i64) -> bool { lo <= hi }

spec fn below_ok(x: i64, lo: i64, r: i64) -> bool { x < lo ==> r == lo }

#[effects(pure)]
#[uses(ordered, below_ok)]
fn clamp(x: i64, lo: i64, hi: i64) -> (r: i64)
    requires ordered(lo, hi),
    ensures below_ok(x, lo, r),
{
    if x < lo { lo } else { if x > hi && hi > lo { hi - 1 } else { if x > hi { hi } else { x } } }
}"""

RUST_REF = """pub fn clamp(x: i64, lo: i64, hi: i64) -> i64 {
    if x < lo { lo } else if x > hi { hi } else { x }
}
"""

# Weak: shifts into [0, hi - lo]; overflows for wide ranges. Its own tests use small values.
RUST_WEAK = """pub fn clamp(x: i64, lo: i64, hi: i64) -> i64 {
    (x - lo).max(0).min(hi - lo) + lo
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
        assert_eq!(clamp(-3, -5, -1), -3);
    }
}
"""

RUST_MUTANTS = [
    "pub fn clamp(x: i64, lo: i64, hi: i64) -> i64 { (if x < lo { lo } else if x > hi { hi } else { x }) - if x == hi && hi != lo { 1 } else { 0 } }",
    "pub fn clamp(x: i64, lo: i64, hi: i64) -> i64 { if x < lo { lo } else if x > hi { lo } else { x } }",
    "pub fn clamp(x: i64, lo: i64, hi: i64) -> i64 { (x - lo).max(0).min(hi - lo) + lo }",
    "pub fn clamp(x: i64, lo: i64, hi: i64) -> i64 { if x > hi { hi } else { x } }",
]
