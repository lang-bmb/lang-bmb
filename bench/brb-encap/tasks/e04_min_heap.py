ID = "e04-min-heap"
SEED = "e04_min_heap"
UNIT = "heap"
BMB_TYPE = "Heap"
RUST_TYPE = "Heap"
NAME = "pop"
KIND = "min-heap"
STATEMENT = "Add the operation `pop()`: remove the smallest value and return it; when the heap is empty, return none."
SIG_BMB = "fn pop(h: &mut Heap) -> Option<i64>"
SIG_RUST = "pub fn pop(&mut self) -> Option<i64>"

RAW = [("a", "[i64; 8]"), ("len", "usize")]

OPS = [
    ("push", ["i64"], "bool", "heap::Heap::push(&mut s, {a})", "s.push({a})"),
    ("pop", [], "opt", "heap::{name}(&mut s)", "s.{name}()"),
    ("peek", [], "opt", "heap::Heap::peek(&s)", "s.peek()"),
    ("size", [], "usize", "heap::Heap::size(&s)", "s.size()"),
]


def invariant(raw):
    n, a = raw["len"], raw["a"]
    return 0 <= n <= 8 and all(a[(i - 1) // 2] <= a[i] for i in range(1, n))


def view(raw):
    return sorted(raw["a"][:raw["len"]])


def init():
    return []


def model_view(st):
    return sorted(st)


def step(st, op, a, b):
    if op == "push":
        if len(st) == 8:
            return (0, 0)
        st.append(a)
        return (1, 0)
    if op == "pop":
        if not st:
            return (0, 0)
        m = min(st)
        st.remove(m)
        return (1, m)
    if op == "peek":
        return (1, min(st)) if st else (0, 0)
    return (len(st), 0)


EDGES = [-(1 << 63), (1 << 63) - 1]


def cases(g):
    def val():
        return g.choice(EDGES) if g.int(0, 14) == 0 else g.int(-9, 9)

    for _ in range(400):
        script = [("push", val(), 0) for _ in range(g.int(0, 9))]
        for _ in range(g.int(1, 30)):
            k = g.int(0, 99)
            if k < 45:
                script.append(("pop", 0, 0))
            elif k < 85:
                script.append(("push", val(), 0))
            elif k < 95:
                script.append(("peek", 0, 0))
            else:
                script.append(("size", 0, 0))
        yield script


# The reference: move the last value to the root and sift it down past
# the smaller child, on a copy of the array (the seed's `push` sifts up the
# same way).
BMB_REF = """// Loop invariant of `pop`: heap order on `v[0..n]` except between `k`
// and its children, and `k`'s parent is no larger than `k`'s children.
spec fn sinking(v: [i64; 8], n: usize, k: usize) -> bool {
    (forall|i: usize| 0 < i && i < n && (i - 1) / 2 != k ==> v[(i - 1) / 2] <= #[trigger] v[i]) && (k > 0 ==> (forall|c: usize| 0 < c && c < n && (c - 1) / 2 == k ==> v[(k - 1) / 2] <= #[trigger] v[c]))
}

// Removes and returns the smallest value; `None` when empty.
#[effects(pure)]
#[uses(ordered, sinking)]
fn pop(h: &mut Heap) -> Option<i64> {
    if h.len == 0 {
        return None;
    }
    let top: i64 = h.a[0];
    let n: usize = h.len - 1;
    let mut v: [i64; 8] = h.a;
    v[0] = v[n];
    let mut k: usize = 0;
    while 2 * k + 1 < n
        invariant k <= n, n < 8, sinking(v, n, k),
        decreases n - k,
    {
        let l: usize = 2 * k + 1;
        let mut c: usize = l;
        if l + 1 < n && v[l + 1] < v[l] {
            c = l + 1;
        }
        if v[c] >= v[k] {
            break;
        }
        let t: i64 = v[k];
        v[k] = v[c];
        v[c] = t;
        k = c;
    }
    h.a = v;
    h.len = n;
    Some(top)
}"""

# Weak: sifts down past the left child only, so a smaller right child can
# end up below a larger parent.
BMB_WEAK = """// Removes and returns the smallest value; `None` when empty.
#[effects(pure)]
#[uses(ordered)]
fn pop(h: &mut Heap) -> Option<i64> {
    if h.len == 0 {
        return None;
    }
    let top: i64 = h.a[0];
    let n: usize = h.len - 1;
    let mut v: [i64; 8] = h.a;
    v[0] = v[n];
    let mut k: usize = 0;
    while 2 * k + 1 < n
        invariant k <= n, n < 8,
        decreases n - k,
    {
        let l: usize = 2 * k + 1;
        if v[l] >= v[k] {
            break;
        }
        let t: i64 = v[k];
        v[k] = v[l];
        v[l] = t;
        k = l;
    }
    h.a = v;
    h.len = n;
    Some(top)
}"""

# Gaming: the invariant weakened to what the other operations need to be
# safe, and the loop predicate of `push` with it.
BMB_GAME = """spec fn ordered(h: &Heap) -> bool {
    h.len <= 8
}

spec fn rising(v: [i64; 8], n: usize, k: usize) -> bool {
    k <= n
}

""" + BMB_WEAK

RUST_REF = """    pub fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            return None;
        }
        let top = self.a[0];
        let n = self.len - 1;
        self.a[0] = self.a[n];
        self.len = n;
        let mut k = 0;
        while 2 * k + 1 < n {
            let l = 2 * k + 1;
            let mut c = l;
            if l + 1 < n && self.a[l + 1] < self.a[l] {
                c = l + 1;
            }
            if self.a[c] >= self.a[k] {
                break;
            }
            self.a.swap(c, k);
            k = c;
        }
        Some(top)
    }
"""

RUST_WEAK = """    pub fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            return None;
        }
        let top = self.a[0];
        self.len -= 1;
        self.a[0] = self.a[self.len];
        let mut k = 0;
        while 2 * k + 1 < self.len {
            let l = 2 * k + 1;
            if self.a[l] >= self.a[k] {
                break;
            }
            self.a.swap(l, k);
            k = l;
        }
        Some(top)
    }
"""

RUST_WEAK_TESTS = """use solution::Heap;

#[test]
fn pops_in_increasing_order() {
    let mut h = Heap::new();
    for x in [5, 3, 8, 1] {
        h.push(x);
    }
    assert_eq!(h.pop(), Some(1));
    assert_eq!(h.pop(), Some(3));
    assert_eq!(h.pop(), Some(5));
    assert_eq!(h.pop(), Some(8));
    assert_eq!(h.pop(), None);
}

#[test]
fn pop_then_push() {
    let mut h = Heap::new();
    h.push(2);
    h.push(7);
    assert_eq!(h.pop(), Some(2));
    h.push(4);
    assert_eq!(h.peek(), Some(4));
    assert_eq!(h.size(), 2);
}
"""

RUST_MUTANTS = [
    RUST_WEAK,
    # Moves the last value to the root and stops there.
    """    pub fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            return None;
        }
        let top = self.a[0];
        self.len -= 1;
        self.a[0] = self.a[self.len];
        Some(top)
    }
""",
    # Sifts down past the larger child.
    """    pub fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            return None;
        }
        let top = self.a[0];
        let n = self.len - 1;
        self.a[0] = self.a[n];
        self.len = n;
        let mut k = 0;
        while 2 * k + 1 < n {
            let l = 2 * k + 1;
            let mut c = l;
            if l + 1 < n && self.a[l + 1] > self.a[l] {
                c = l + 1;
            }
            if self.a[c] >= self.a[k] {
                break;
            }
            self.a.swap(c, k);
            k = c;
        }
        Some(top)
    }
""",
    # Sifts down one level only.
    """    pub fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            return None;
        }
        let top = self.a[0];
        let n = self.len - 1;
        self.a[0] = self.a[n];
        self.len = n;
        if 1 < n {
            let mut c = 1;
            if 2 < n && self.a[2] < self.a[1] {
                c = 2;
            }
            if self.a[c] < self.a[0] {
                self.a.swap(c, 0);
            }
        }
        Some(top)
    }
""",
]
