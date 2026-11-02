from cases import I64_EDGES, I64_MAX

ID = "f04-rle"
NAME = "rle"
PARAMS = [("a", "&[i64]"), ("limit", "i64")]
RET = "i64"
STATEMENT = (
    "Print a run-length encoding of a. Split a into maximal runs of equal consecutive elements, and split every "
    "run longer than limit into pieces of exactly limit elements followed by one shorter piece for the rest (if "
    "any rest remains), in order. For each piece print two lines: the element's value, then the piece's length "
    "(print each value with print_i64 in BMB, or println!(\"{}\", value) in Rust). Return the number of pieces "
    "(0 for an empty slice).")
DOMAIN = "a is any slice of i64 values (possibly empty); limit >= 1."
BMB_DOMAIN = """spec fn dom(a: &[i64], limit: i64) -> bool {
    limit >= 1
}"""


def domain(a, limit):
    return limit >= 1


def model(a, limit):
    out, pieces, k = [], 0, 0
    while k < len(a):
        j = k
        while j < len(a) and a[j] == a[k]:
            j += 1
        n = j - k
        while n > 0:
            m = min(n, limit)
            out += [a[k], m]
            pieces += 1
            n -= m
        k = j
    return pieces, out


def cases(g):
    yield ([], 1)
    yield ([], I64_MAX)
    yield ([7], 1)
    yield ([7, 7, 7], 3)
    yield ([7, 7, 7, 7], 2)
    yield ([7, 7, 7, 7, 7], 2)
    yield ([1, 1, 2, 2, 2, 1], I64_MAX)
    yield ([I64_MAX, I64_MAX, -1, -1, -1], 1)
    for _ in range(440):
        a = g.array(24, g.small_elem(g.choice([0, 1, 1, 2, 3])))
        if a and g.rng.randrange(3) == 0:
            big = g.choice(I64_EDGES)
            a = [big if x == 0 else x for x in a]
        yield (a, g.choice([1, 1, 2, 2, 3, 4, 5, g.int(1, 30), 1 << 33, I64_MAX]))


BMB_REF = """spec fn piece(n: i64, limit: i64) -> bool { 1 <= n && n <= limit }

spec fn room(n: i64, limit: i64, r: bool) -> bool { r ==> n < limit }

#[effects(pure)]
#[uses(room)]
fn continues(cur: i64, n: i64, x: i64, limit: i64) -> (r: bool)
    ensures room(n, limit, r),
{
    x == cur && n < limit
}

#[effects(io)]
#[uses(piece, room)]
fn rle(a: &[i64], limit: i64) -> (r: i64)
    requires piece(1, limit),
{
    if a.len() == 0 {
        0
    } else {
        let mut cur: i64 = a[0];
        let mut n: i64 = 1;
        let mut pieces: i64 = 0;
        let mut k: usize = 1;
        while k < a.len()
            invariant 1 <= k, k <= a.len(), piece(n, limit), pieces < k,
            decreases a.len() - k,
        {
            if continues(cur, n, a[k], limit) {
                n = n + 1;
            } else {
                print_i64(cur);
                print_i64(n);
                pieces = pieces + 1;
                cur = a[k];
                n = 1;
            }
            k = k + 1;
        }
        print_i64(cur);
        print_i64(n);
        pieces + 1
    }
}"""

# Weak: a piece that ends because the next element differs or the piece is
# full prints its length before its value (the last piece is right).
BMB_WEAK = """spec fn piece(n: i64, limit: i64) -> bool { 1 <= n && n <= limit }

spec fn room(n: i64, limit: i64, r: bool) -> bool { r ==> n < limit }

#[effects(pure)]
#[uses(room)]
fn continues(cur: i64, n: i64, x: i64, limit: i64) -> (r: bool)
    ensures room(n, limit, r),
{
    x == cur && n < limit
}

#[effects(io)]
#[uses(piece, room)]
fn rle(a: &[i64], limit: i64) -> (r: i64)
    requires piece(1, limit),
{
    if a.len() == 0 {
        0
    } else {
        let mut cur: i64 = a[0];
        let mut n: i64 = 1;
        let mut pieces: i64 = 0;
        let mut k: usize = 1;
        while k < a.len()
            invariant 1 <= k, k <= a.len(), piece(n, limit), pieces < k,
            decreases a.len() - k,
        {
            if continues(cur, n, a[k], limit) {
                n = n + 1;
            } else {
                print_i64(n);
                print_i64(cur);
                pieces = pieces + 1;
                cur = a[k];
                n = 1;
            }
            k = k + 1;
        }
        print_i64(cur);
        print_i64(n);
        pieces + 1
    }
}"""

RUST_REF = """pub fn rle(a: &[i64], limit: i64) -> i64 {
    let mut pieces = 0i64;
    let mut k = 0usize;
    while k < a.len() {
        let v = a[k];
        let mut n = 0i64;
        while k < a.len() && a[k] == v {
            if n == limit {
                println!("{}", v);
                println!("{}", n);
                pieces += 1;
                n = 0;
            }
            n += 1;
            k += 1;
        }
        println!("{}", v);
        println!("{}", n);
        pieces += 1;
    }
    pieces
}
"""

# Weak: a piece may hold limit + 1 elements; its tests use limits longer
# than the runs.
RUST_WEAK = """pub fn rle(a: &[i64], limit: i64) -> i64 {
    let mut pieces = 0i64;
    let mut k = 0usize;
    while k < a.len() {
        let v = a[k];
        let mut n = 0i64;
        while k < a.len() && a[k] == v {
            if n > limit {
                println!("{}", v);
                println!("{}", n);
                pieces += 1;
                n = 0;
            }
            n += 1;
            k += 1;
        }
        println!("{}", v);
        println!("{}", n);
        pieces += 1;
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pieces() {
        assert_eq!(rle(&[], 3), 0);
        assert_eq!(rle(&[1, 1, 2], 5), 2);
        assert_eq!(rle(&[4, 4, 4, 4], 10), 1);
        assert_eq!(rle(&[1, 2, 1], 1), 3);
    }
}
"""


def _rust(full="n == limit", order=("v", "n"), last=True):
    pr = "println!(\"{}\", " + order[0] + "); println!(\"{}\", " + order[1] + ");"
    tail = (pr + " pieces += 1;") if last else "pieces += 1;"
    return ("pub fn rle(a: &[i64], limit: i64) -> i64 {\n"
            "    let mut pieces = 0i64; let mut k = 0usize;\n"
            "    while k < a.len() {\n        let v = a[k]; let mut n = 0i64;\n"
            "        while k < a.len() && a[k] == v {\n"
            "            if " + full + " { " + pr + " pieces += 1; n = 0; }\n"
            "            n += 1; k += 1;\n        }\n"
            "        if k < a.len() { " + pr + " pieces += 1; } else { " + tail + " }\n"
            "    }\n    pieces\n}")


RUST_MUTANTS = [
    _rust(full="n > limit"),          # pieces of limit + 1
    _rust(full="false"),              # limit ignored
    _rust(order=("n", "v")),          # length before value
    _rust(last=False),                # the last piece is not printed
]
