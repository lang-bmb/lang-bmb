ID = "e03-intervals"
SEED = "e03_intervals"
UNIT = "spans"
BMB_TYPE = "Spans"
RUST_TYPE = "Spans"
NAME = "remove"
KIND = "interval list"
STATEMENT = ("Add the operation `remove(x)`: take the single value `x` out of the set and return true. A span "
             "that holds only `x` disappears, `x` at either end of a span shortens it, and `x` inside a span "
             "splits it in two. A split needs a free place: when there are already 8 spans, leave the list "
             "unchanged and return false. When `x` is not in the set, return false.")
SIG_BMB = "fn remove(s: &mut Spans, x: i64) -> bool"
SIG_RUST = "pub fn remove(&mut self, x: i64) -> bool"

RAW = [("lo", "[i64; 8]"), ("hi", "[i64; 8]"), ("len", "usize")]

OPS = [
    ("push", ["i64", "i64"], "bool", "spans::Spans::push(&mut s, {a}, {b})", "s.push({a}, {b})"),
    ("remove", ["i64"], "bool", "spans::{name}(&mut s, {a})", "s.{name}({a})"),
    ("contains", ["i64"], "bool", "spans::Spans::contains(&s, {a})", "s.contains({a})"),
    ("count", [], "usize", "spans::Spans::count(&s)", "s.count()"),
    ("first", ["usize"], "opt", "spans::Spans::first(&s, {a})", "s.first({a})"),
    ("last", ["usize"], "opt", "spans::Spans::last(&s, {a})", "s.last({a})"),
]


def invariant(raw):
    n, lo, hi = raw["len"], raw["lo"], raw["hi"]
    return (0 <= n <= 8 and all(lo[i] <= hi[i] for i in range(n))
            and all(hi[i] + 1 < lo[i + 1] for i in range(n - 1)))


def view(raw):
    return [(raw["lo"][i], raw["hi"][i]) for i in range(raw["len"])]


def init():
    return []


def model_view(st):
    return list(st)


def step(st, op, a, b):
    if op == "push":
        if a > b or len(st) == 8 or (st and st[-1][1] + 1 >= a):
            return (0, 0)
        st.append((a, b))
        return (1, 0)
    if op == "contains":
        return (int(any(lo <= a <= hi for lo, hi in st)), 0)
    if op == "count":
        return (len(st), 0)
    if op in ("first", "last"):
        return (1, st[a][0 if op == "first" else 1]) if a < len(st) else (0, 0)
    for i, (lo, hi) in enumerate(st):
        if lo <= a <= hi:
            if lo == hi:
                del st[i]
            elif a == lo:
                st[i] = (lo + 1, hi)
            elif a == hi:
                st[i] = (lo, hi - 1)
            elif len(st) == 8:
                return (0, 0)
            else:
                st[i:i + 1] = [(lo, a - 1), (a + 1, hi)]
            return (1, 0)
    return (0, 0)


MIN, MAX = -(1 << 63), (1 << 63) - 1


def cases(g):
    for _ in range(400):
        # Spans with gaps of one to three values and widths of one to five,
        # sometimes starting at the bottom of `i64` or ending at its top.
        start = g.choice([MIN, MAX - 60, -20, -20, 0, 0])
        spans, x = [], start
        for _ in range(g.int(0, 8)):
            w = g.int(0, 4)
            if x + w > MAX:
                break
            spans.append((x, x + w))
            x = x + w + g.int(2, 4)
        script = [("push", lo, hi) for lo, hi in spans]
        pts = [v for lo, hi in spans for v in (lo - 1, lo, lo + 1, (lo + hi) // 2, hi - 1, hi, hi + 1)
               if MIN <= v <= MAX] or [0]
        for _ in range(g.int(1, 25)):
            k = g.int(0, 99)
            if k < 50:
                script.append(("remove", g.choice(pts), 0))
            elif k < 62:
                lo = g.choice(pts)
                script.append(("push", lo, max(MIN, min(MAX, lo + g.int(-1, 3)))))
            elif k < 77:
                script.append(("contains", g.choice(pts), 0))
            elif k < 85:
                script.append(("count", 0, 0))
            else:
                script.append((g.choice(["first", "last"]), g.int(0, 8), 0))
        yield script


# The reference: find the span, then delete it (shifting the later spans
# down), shorten it, or split it (shifting the later spans up), on copies of
# the arrays as the seed's sorted set does.
BMB_REF = """spec fn below(s: &Spans, p: usize, x: i64) -> bool {
    forall|k: usize| k < p ==> #[trigger] s.hi[k] < x
}

spec fn shifted(a: [i64; 8], b: [i64; 8], j: usize, n: usize) -> bool {
    (forall|k: usize| k <= j ==> #[trigger] b[k] == a[k]) && (forall|m: usize| j < m && m <= n ==> #[trigger] b[m] == a[m - 1])
}

spec fn closed(a: [i64; 8], b: [i64; 8], p: usize, j: usize, n: usize) -> bool {
    (forall|k: usize| k < p ==> #[trigger] b[k] == a[k]) && (forall|m: usize| p <= m && m < j ==> #[trigger] b[m] == a[m + 1]) && (forall|q: usize| j <= q && q < n ==> #[trigger] b[q] == a[q])
}

#[effects(pure)]
#[uses(below, canon, closed, shifted)]
fn remove(s: &mut Spans, x: i64) -> bool {
    let mut p: usize = 0;
    while p < s.len && s.hi[p] < x
        invariant p <= s.len, below(s, p, x),
        decreases s.len - p,
    {
        p = p + 1;
    }
    if p == s.len || x < s.lo[p] {
        return false;
    }
    let n: usize = s.len;
    if s.lo[p] == s.hi[p] {
        let mut a: [i64; 8] = s.lo;
        let mut b: [i64; 8] = s.hi;
        for j in p..n - 1
            invariant n <= 8, closed(s.lo, a, p, j, n), closed(s.hi, b, p, j, n),
        {
            a[j] = a[j + 1];
            b[j] = b[j + 1];
        }
        s.lo = a;
        s.hi = b;
        s.len = n - 1;
        return true;
    }
    if x == s.lo[p] {
        s.lo[p] = x + 1;
        return true;
    }
    if x == s.hi[p] {
        s.hi[p] = x - 1;
        return true;
    }
    if n == 8 {
        return false;
    }
    let top: i64 = s.hi[p];
    let mut c: [i64; 8] = s.lo;
    let mut d: [i64; 8] = s.hi;
    let mut k: usize = n;
    while k > p + 1
        invariant p + 1 <= k, k <= n, n < 8, shifted(s.lo, c, k, n), shifted(s.hi, d, k, n),
        decreases k,
    {
        c[k] = c[k - 1];
        d[k] = d[k - 1];
        k = k - 1;
    }
    d[p] = x - 1;
    c[p + 1] = x + 1;
    d[p + 1] = top;
    s.lo = c;
    s.hi = d;
    s.len = n + 1;
    true
}"""

# Weak: tests the ends before the one-value span, so removing the only value
# of a span leaves an empty span (`lo > hi`) behind.
BMB_WEAK = """spec fn below(s: &Spans, p: usize, x: i64) -> bool {
    forall|k: usize| k < p ==> #[trigger] s.hi[k] < x
}

spec fn shifted(a: [i64; 8], b: [i64; 8], j: usize, n: usize) -> bool {
    (forall|k: usize| k <= j ==> #[trigger] b[k] == a[k]) && (forall|m: usize| j < m && m <= n ==> #[trigger] b[m] == a[m - 1])
}

spec fn closed(a: [i64; 8], b: [i64; 8], p: usize, j: usize, n: usize) -> bool {
    (forall|k: usize| k < p ==> #[trigger] b[k] == a[k]) && (forall|m: usize| p <= m && m < j ==> #[trigger] b[m] == a[m + 1]) && (forall|q: usize| j <= q && q < n ==> #[trigger] b[q] == a[q])
}

#[effects(pure)]
#[uses(below, canon, closed, shifted)]
fn remove(s: &mut Spans, x: i64) -> bool {
    let mut p: usize = 0;
    while p < s.len && s.hi[p] < x
        invariant p <= s.len, below(s, p, x),
        decreases s.len - p,
    {
        p = p + 1;
    }
    if p == s.len || x < s.lo[p] {
        return false;
    }
    let n: usize = s.len;
    if x == s.lo[p] {
        s.lo[p] = x + 1;
        return true;
    }
    if x == s.hi[p] {
        s.hi[p] = x - 1;
        return true;
    }
    if s.lo[p] == s.hi[p] {
        let mut a: [i64; 8] = s.lo;
        let mut b: [i64; 8] = s.hi;
        for j in p..n - 1
            invariant n <= 8, closed(s.lo, a, p, j, n), closed(s.hi, b, p, j, n),
        {
            a[j] = a[j + 1];
            b[j] = b[j + 1];
        }
        s.lo = a;
        s.hi = b;
        s.len = n - 1;
        return true;
    }
    if n == 8 {
        return false;
    }
    let top: i64 = s.hi[p];
    let mut c: [i64; 8] = s.lo;
    let mut d: [i64; 8] = s.hi;
    let mut k: usize = n;
    while k > p + 1
        invariant p + 1 <= k, k <= n, n < 8, shifted(s.lo, c, k, n), shifted(s.hi, d, k, n),
        decreases k,
    {
        c[k] = c[k - 1];
        d[k] = d[k - 1];
        k = k - 1;
    }
    d[p] = x - 1;
    c[p + 1] = x + 1;
    d[p + 1] = top;
    s.lo = c;
    s.hi = d;
    s.len = n + 1;
    true
}"""

# Gaming: the invariant weakened to what the other operations need to be
# safe, and a split that appends the upper half at the end.
BMB_GAME = """spec fn canon(s: &Spans) -> bool {
    s.len <= 8
}

spec fn below(s: &Spans, p: usize, x: i64) -> bool {
    forall|k: usize| k < p ==> #[trigger] s.hi[k] < x
}

spec fn shifted(a: [i64; 8], b: [i64; 8], j: usize, n: usize) -> bool {
    (forall|k: usize| k <= j ==> #[trigger] b[k] == a[k]) && (forall|m: usize| j < m && m <= n ==> #[trigger] b[m] == a[m - 1])
}

spec fn closed(a: [i64; 8], b: [i64; 8], p: usize, j: usize, n: usize) -> bool {
    (forall|k: usize| k < p ==> #[trigger] b[k] == a[k]) && (forall|m: usize| p <= m && m < j ==> #[trigger] b[m] == a[m + 1]) && (forall|q: usize| j <= q && q < n ==> #[trigger] b[q] == a[q])
}

#[effects(pure)]
#[uses(below, canon, closed, shifted)]
fn remove(s: &mut Spans, x: i64) -> bool {
    let mut p: usize = 0;
    while p < s.len && s.hi[p] < x
        invariant p <= s.len, below(s, p, x),
        decreases s.len - p,
    {
        p = p + 1;
    }
    if p == s.len || x < s.lo[p] {
        return false;
    }
    let n: usize = s.len;
    if s.lo[p] == s.hi[p] {
        let mut a: [i64; 8] = s.lo;
        let mut b: [i64; 8] = s.hi;
        for j in p..n - 1
            invariant n <= 8, closed(s.lo, a, p, j, n), closed(s.hi, b, p, j, n),
        {
            a[j] = a[j + 1];
            b[j] = b[j + 1];
        }
        s.lo = a;
        s.hi = b;
        s.len = n - 1;
        return true;
    }
    if x == s.lo[p] {
        s.lo[p] = x + 1;
        return true;
    }
    if x == s.hi[p] {
        s.hi[p] = x - 1;
        return true;
    }
    if n == 8 {
        return false;
    }
    let top: i64 = s.hi[p];
    s.hi[p] = x - 1;
    s.lo[n] = x + 1;
    s.hi[n] = top;
    s.len = n + 1;
    true
}"""

RUST_REF = """    pub fn remove(&mut self, x: i64) -> bool {
        let mut p = 0;
        while p < self.len && self.hi[p] < x {
            p += 1;
        }
        if p == self.len || x < self.lo[p] {
            return false;
        }
        let n = self.len;
        if self.lo[p] == self.hi[p] {
            for j in p..n - 1 {
                self.lo[j] = self.lo[j + 1];
                self.hi[j] = self.hi[j + 1];
            }
            self.len = n - 1;
            return true;
        }
        if x == self.lo[p] {
            self.lo[p] = x + 1;
            return true;
        }
        if x == self.hi[p] {
            self.hi[p] = x - 1;
            return true;
        }
        if n == 8 {
            return false;
        }
        let mut k = n;
        while k > p + 1 {
            self.lo[k] = self.lo[k - 1];
            self.hi[k] = self.hi[k - 1];
            k -= 1;
        }
        self.lo[p + 1] = x + 1;
        self.hi[p + 1] = self.hi[p];
        self.hi[p] = x - 1;
        self.len = n + 1;
        true
    }
"""

RUST_WEAK = """    pub fn remove(&mut self, x: i64) -> bool {
        for i in 0..self.len {
            if self.lo[i] <= x && x <= self.hi[i] {
                if x == self.lo[i] {
                    self.lo[i] += 1;
                    return true;
                }
                if x == self.hi[i] {
                    self.hi[i] -= 1;
                    return true;
                }
                if self.len == 8 {
                    return false;
                }
                let mut k = self.len;
                while k > i + 1 {
                    self.lo[k] = self.lo[k - 1];
                    self.hi[k] = self.hi[k - 1];
                    k -= 1;
                }
                self.lo[i + 1] = x + 1;
                self.hi[i + 1] = self.hi[i];
                self.hi[i] = x - 1;
                self.len += 1;
                return true;
            }
        }
        false
    }
"""

RUST_WEAK_TESTS = """use solution::Spans;

fn spans(s: &Spans) -> Vec<(i64, i64)> {
    (0..s.count()).map(|i| (s.first(i).unwrap(), s.last(i).unwrap())).collect()
}

#[test]
fn remove_shortens_and_splits() {
    let mut s = Spans::new();
    s.push(1, 10);
    s.push(20, 30);
    assert!(s.remove(1));
    assert!(s.remove(30));
    assert!(s.remove(5));
    assert_eq!(spans(&s), vec![(2, 4), (6, 10), (20, 29)]);
    assert!(!s.remove(5));
    assert!(!s.contains(5));
}

#[test]
fn split_needs_room() {
    let mut s = Spans::new();
    for i in 0..8 {
        s.push(10 * i, 10 * i + 5);
    }
    assert!(!s.remove(2));
    assert_eq!(s.count(), 8);
}
"""

RUST_MUTANTS = [
    RUST_WEAK,
    # Splits by appending the upper half at the end.
    """    pub fn remove(&mut self, x: i64) -> bool {
        for i in 0..self.len {
            if self.lo[i] <= x && x <= self.hi[i] {
                if self.lo[i] == self.hi[i] {
                    for j in i..self.len - 1 {
                        self.lo[j] = self.lo[j + 1];
                        self.hi[j] = self.hi[j + 1];
                    }
                    self.len -= 1;
                } else if x == self.lo[i] {
                    self.lo[i] += 1;
                } else if x == self.hi[i] {
                    self.hi[i] -= 1;
                } else {
                    if self.len == 8 {
                        return false;
                    }
                    self.lo[self.len] = x + 1;
                    self.hi[self.len] = self.hi[i];
                    self.hi[i] = x - 1;
                    self.len += 1;
                }
                return true;
            }
        }
        false
    }
""",
    # Deletes a one-value span by dropping the last span instead.
    """    pub fn remove(&mut self, x: i64) -> bool {
        for i in 0..self.len {
            if self.lo[i] <= x && x <= self.hi[i] {
                if self.lo[i] == self.hi[i] {
                    self.len -= 1;
                } else if x == self.lo[i] {
                    self.lo[i] += 1;
                } else if x == self.hi[i] {
                    self.hi[i] -= 1;
                } else {
                    if self.len == 8 {
                        return false;
                    }
                    let mut k = self.len;
                    while k > i + 1 {
                        self.lo[k] = self.lo[k - 1];
                        self.hi[k] = self.hi[k - 1];
                        k -= 1;
                    }
                    self.lo[i + 1] = x + 1;
                    self.hi[i + 1] = self.hi[i];
                    self.hi[i] = x - 1;
                    self.len += 1;
                }
                return true;
            }
        }
        false
    }
""",
    # Splits a full list by overwriting the last span.
    """    pub fn remove(&mut self, x: i64) -> bool {
        for i in 0..self.len {
            if self.lo[i] <= x && x <= self.hi[i] {
                if self.lo[i] == self.hi[i] {
                    for j in i..self.len - 1 {
                        self.lo[j] = self.lo[j + 1];
                        self.hi[j] = self.hi[j + 1];
                    }
                    self.len -= 1;
                } else if x == self.lo[i] {
                    self.lo[i] += 1;
                } else if x == self.hi[i] {
                    self.hi[i] -= 1;
                } else {
                    let n = if self.len == 8 { 7 } else { self.len };
                    let mut k = n;
                    while k > i + 1 {
                        self.lo[k] = self.lo[k - 1];
                        self.hi[k] = self.hi[k - 1];
                        k -= 1;
                    }
                    self.lo[i + 1] = x + 1;
                    self.hi[i + 1] = self.hi[i];
                    self.hi[i] = x - 1;
                    self.len = n + 1;
                }
                return true;
            }
        }
        false
    }
""",
]
