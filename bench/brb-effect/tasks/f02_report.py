from cases import I64_EDGES, I64_MAX, I64_MIN

ID = "f02-report"
NAME = "report"
PARAMS = [("a", "&[i64]"), ("t", "i64")]
RET = "i64"
STATEMENT = (
    "Print a short report on a. If a is empty, print nothing and return 0. Otherwise print four lines, in this "
    "order: the smallest element; the largest element; the midpoint of those two, floor((smallest + largest) / 2) "
    "computed exactly and rounded toward negative infinity; and the number of elements strictly greater than t "
    "(print each value with print_i64 in BMB, or println!(\"{}\", value) in Rust). Return the length of the longest "
    "run of consecutive elements that are all strictly greater than t (0 if there is none).")
DOMAIN = "a is any slice of i64 values (possibly empty); t is any i64."
BMB_DOMAIN = None


def domain(a, t):
    return True


def model(a, t):
    if not a:
        return 0, []
    lo, hi = min(a), max(a)
    best = run = 0
    for x in a:
        run = run + 1 if x > t else 0
        best = max(best, run)
    return best, [lo, hi, (lo + hi) // 2, sum(1 for x in a if x > t)]


def cases(g):
    yield ([], 0)
    yield ([I64_MAX, I64_MAX - 1], 0)
    yield ([I64_MIN, I64_MIN + 1], 0)
    yield ([I64_MIN, I64_MAX], I64_MIN)
    yield ([-3, 0], -1)
    yield ([-1, -2, 5, 5, 5], 4)
    yield ([1, 1, 0, 1, 1, 1], 0)
    for a in g.arrays(330, 14):
        ts = [0, -1] + ([g.choice(a), min(a) - 1 if min(a) > I64_MIN else I64_MIN] if a else []) + [g.i64()]
        yield (a, g.choice(ts))
        yield (a, g.choice(I64_EDGES))


BMB_REF = """spec fn nonempty(a: &[i64]) -> bool { a.len() > 0 }

#[effects(pure)]
#[uses(nonempty)]
fn min_of(a: &[i64]) -> (r: i64)
    requires nonempty(a),
{
    let mut m: i64 = a[0];
    let mut k: usize = 1;
    while k < a.len()
        invariant k <= a.len(),
        decreases a.len() - k,
    {
        if a[k] < m { m = a[k]; }
        k = k + 1;
    }
    m
}

#[effects(pure)]
#[uses(nonempty)]
fn max_of(a: &[i64]) -> (r: i64)
    requires nonempty(a),
{
    let mut m: i64 = a[0];
    let mut k: usize = 1;
    while k < a.len()
        invariant k <= a.len(),
        decreases a.len() - k,
    {
        if a[k] > m { m = a[k]; }
        k = k + 1;
    }
    m
}

spec fn halved(d: i64, r: i64) -> bool { 2 * r <= d && d < 2 * r + 2 }

#[effects(pure)]
#[uses(halved)]
fn floor_half(d: i64) -> (r: i64)
    ensures halved(d, r),
{
    if d < 0 && d % 2 != 0 { d / 2 - 1 } else { d / 2 }
}

#[effects(pure)]
#[uses(halved)]
fn midpoint(lo: i64, hi: i64) -> (r: i64) {
    if (lo >= 0) == (hi >= 0) {
        lo + floor_half(hi - lo)
    } else {
        floor_half(lo + hi)
    }
}

#[effects(pure)]
fn count_above(a: &[i64], t: i64) -> (r: i64) {
    let mut c: i64 = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), c <= k,
        decreases a.len() - k,
    {
        if a[k] > t { c = c + 1; }
        k = k + 1;
    }
    c
}

#[effects(pure)]
fn longest_above(a: &[i64], t: i64) -> (r: i64) {
    let mut best: i64 = 0;
    let mut run: i64 = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), run <= k, best <= k,
        decreases a.len() - k,
    {
        if a[k] > t {
            run = run + 1;
            if run > best { best = run; }
        } else {
            run = 0;
        }
        k = k + 1;
    }
    best
}

#[effects(io)]
#[uses(nonempty)]
fn report(a: &[i64], t: i64) -> (r: i64) {
    if a.len() == 0 {
        0
    } else {
        let lo: i64 = min_of(a);
        let hi: i64 = max_of(a);
        print_i64(lo);
        print_i64(hi);
        print_i64(midpoint(lo, hi));
        print_i64(count_above(a, t));
        longest_above(a, t)
    }
}"""

# Weak: the longest run is updated only when a run ends, so a run that
# reaches the end of the slice is never counted (the printed lines are right).
BMB_WEAK = """spec fn nonempty(a: &[i64]) -> bool { a.len() > 0 }

#[effects(pure)]
#[uses(nonempty)]
fn min_of(a: &[i64]) -> (r: i64)
    requires nonempty(a),
{
    let mut m: i64 = a[0];
    let mut k: usize = 1;
    while k < a.len()
        invariant k <= a.len(),
        decreases a.len() - k,
    {
        if a[k] < m { m = a[k]; }
        k = k + 1;
    }
    m
}

#[effects(pure)]
#[uses(nonempty)]
fn max_of(a: &[i64]) -> (r: i64)
    requires nonempty(a),
{
    let mut m: i64 = a[0];
    let mut k: usize = 1;
    while k < a.len()
        invariant k <= a.len(),
        decreases a.len() - k,
    {
        if a[k] > m { m = a[k]; }
        k = k + 1;
    }
    m
}

spec fn halved(d: i64, r: i64) -> bool { 2 * r <= d && d < 2 * r + 2 }

#[effects(pure)]
#[uses(halved)]
fn floor_half(d: i64) -> (r: i64)
    ensures halved(d, r),
{
    if d < 0 && d % 2 != 0 { d / 2 - 1 } else { d / 2 }
}

#[effects(pure)]
#[uses(halved)]
fn midpoint(lo: i64, hi: i64) -> (r: i64) {
    if (lo >= 0) == (hi >= 0) {
        lo + floor_half(hi - lo)
    } else {
        floor_half(lo + hi)
    }
}

#[effects(pure)]
fn count_above(a: &[i64], t: i64) -> (r: i64) {
    let mut c: i64 = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), c <= k,
        decreases a.len() - k,
    {
        if a[k] > t { c = c + 1; }
        k = k + 1;
    }
    c
}

#[effects(pure)]
fn longest_above(a: &[i64], t: i64) -> (r: i64) {
    let mut best: i64 = 0;
    let mut run: i64 = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), run <= k, best <= k,
        decreases a.len() - k,
    {
        if a[k] > t {
            run = run + 1;
        } else {
            if run > best { best = run; }
            run = 0;
        }
        k = k + 1;
    }
    best
}

#[effects(io)]
#[uses(nonempty)]
fn report(a: &[i64], t: i64) -> (r: i64) {
    if a.len() == 0 {
        0
    } else {
        let lo: i64 = min_of(a);
        let hi: i64 = max_of(a);
        print_i64(lo);
        print_i64(hi);
        print_i64(midpoint(lo, hi));
        print_i64(count_above(a, t));
        longest_above(a, t)
    }
}"""

RUST_REF = """pub fn report(a: &[i64], t: i64) -> i64 {
    if a.is_empty() {
        return 0;
    }
    let lo = *a.iter().min().unwrap();
    let hi = *a.iter().max().unwrap();
    println!("{}", lo);
    println!("{}", hi);
    println!("{}", ((lo as i128 + hi as i128).div_euclid(2)) as i64);
    println!("{}", a.iter().filter(|&&x| x > t).count());
    let (mut best, mut run) = (0i64, 0i64);
    for &x in a {
        run = if x > t { run + 1 } else { 0 };
        best = best.max(run);
    }
    best
}
"""

# Weak: the midpoint as (lo + hi) / 2 (overflows, and rounds toward zero);
# its tests use small positive values.
RUST_WEAK = """pub fn report(a: &[i64], t: i64) -> i64 {
    if a.is_empty() {
        return 0;
    }
    let lo = *a.iter().min().unwrap();
    let hi = *a.iter().max().unwrap();
    println!("{}", lo);
    println!("{}", hi);
    println!("{}", (lo + hi) / 2);
    println!("{}", a.iter().filter(|&&x| x > t).count());
    let (mut best, mut run) = (0i64, 0i64);
    for &x in a {
        run = if x > t { run + 1 } else { 0 };
        best = best.max(run);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs() {
        assert_eq!(report(&[], 0), 0);
        assert_eq!(report(&[1, 5, 6, 0, 7], 4), 2);
        assert_eq!(report(&[3, 3, 3], 2), 3);
        assert_eq!(report(&[3, 3, 3], 3), 0);
    }
}
"""


def _rust(mid, cmp=">", flush=False):
    upd = ("if x " + cmp + " t { run += 1; } else { best = best.max(run); run = 0; }" if flush else
           "run = if x " + cmp + " t { run + 1 } else { 0 }; best = best.max(run);")
    return ("pub fn report(a: &[i64], t: i64) -> i64 {\n"
            "    if a.is_empty() { return 0; }\n"
            "    let lo = *a.iter().min().unwrap();\n    let hi = *a.iter().max().unwrap();\n"
            "    println!(\"{}\", lo);\n    println!(\"{}\", hi);\n"
            "    println!(\"{}\", " + mid + ");\n"
            "    println!(\"{}\", a.iter().filter(|&&x| x " + cmp + " t).count());\n"
            "    let (mut best, mut run) = (0i64, 0i64);\n"
            "    for &x in a { " + upd + " }\n    best\n}")


_EXACT = "((lo as i128 + hi as i128).div_euclid(2)) as i64"

RUST_MUTANTS = [
    _rust("(lo + hi) / 2"),                                # overflow, rounds toward zero
    _rust("((lo as i128 + hi as i128) / 2) as i64"),       # rounds toward zero
    _rust(_EXACT, cmp=">="),                               # counts elements equal to t
    _rust(_EXACT, flush=True),                             # a run at the end is not counted
]
