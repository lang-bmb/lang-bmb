ID = "e02-ring"
SEED = "e02_ring"
UNIT = "ring"
BMB_TYPE = "Ring"
RUST_TYPE = "Ring"
NAME = "push_overwrite"
KIND = "ring buffer"
STATEMENT = ("Add the operation `push_overwrite(x)`: add `x` as the newest value. If the ring already holds 8 "
             "values, the oldest one is dropped to make room and returned; otherwise nothing is dropped and the "
             "result is none.")
SIG_BMB = "fn push_overwrite(r: &mut Ring, x: i64) -> Option<i64>"
SIG_RUST = "pub fn push_overwrite(&mut self, x: i64) -> Option<i64>"

RAW = [("buf", "[i64; 8]"), ("head", "usize"), ("len", "usize")]

OPS = [
    ("push", ["i64"], "bool", "ring::Ring::push(&mut s, {a})", "s.push({a})"),
    ("pop", [], "opt", "ring::Ring::pop(&mut s)", "s.pop()"),
    ("get", ["usize"], "opt", "ring::Ring::get(&s, {a})", "s.get({a})"),
    ("size", [], "usize", "ring::Ring::size(&s)", "s.size()"),
    ("push_overwrite", ["i64"], "opt", "ring::{name}(&mut s, {a})", "s.{name}({a})"),
]


def invariant(raw):
    return 0 <= raw["head"] < 8 and 0 <= raw["len"] <= 8


def view(raw):
    return [raw["buf"][(raw["head"] + i) % 8] for i in range(raw["len"])]


def init():
    return []


def model_view(st):
    return list(st)


def step(st, op, a, b):
    if op == "push":
        if len(st) == 8:
            return (0, 0)
        st.append(a)
        return (1, 0)
    if op == "pop":
        return (1, st.pop(0)) if st else (0, 0)
    if op == "get":
        return (1, st[a]) if a < len(st) else (0, 0)
    if op == "size":
        return (len(st), 0)
    st.append(a)
    if len(st) > 8:
        return (1, st.pop(0))
    return (0, 0)


EDGES = [-(1 << 63), -1, 0, (1 << 63) - 1]


def cases(g):
    def val():
        return g.choice(EDGES) if g.int(0, 9) == 0 else g.int(-1000, 1000)

    for _ in range(300):
        script = [("push", val(), 0) for _ in range(g.int(0, 8))]
        # Long scripts: the start position goes round the buffer many times.
        for _ in range(g.int(1, 40)):
            k = g.int(0, 99)
            if k < 45:
                script.append(("push_overwrite", val(), 0))
            elif k < 60:
                script.append(("push", val(), 0))
            elif k < 80:
                script.append(("pop", 0, 0))
            elif k < 93:
                script.append(("get", g.int(0, 9), 0))
            else:
                script.append(("size", 0, 0))
        yield script


BMB_REF = """#[effects(pure)]
#[uses(in_bounds)]
fn push_overwrite(r: &mut Ring, x: i64) -> Option<i64> {
    if r.len < 8 {
        r.buf[(r.head + r.len) % 8] = x;
        r.len = r.len + 1;
        return None;
    }
    let oldest: i64 = r.buf[r.head];
    r.buf[r.head] = x;
    r.head = (r.head + 1) % 8;
    Some(oldest)
}"""

# Weak: when full, the start position moves on without wrapping round.
# `get` still reads through `% 8`, so the values look right; `pop` reads
# `buf[head]` directly.
BMB_WEAK = """#[effects(pure)]
#[uses(in_bounds)]
fn push_overwrite(r: &mut Ring, x: i64) -> Option<i64> {
    if r.len < 8 {
        r.buf[(r.head + r.len) % 8] = x;
        r.len = r.len + 1;
        return None;
    }
    let oldest: Option<i64> = Ring::get(r, 0);
    r.buf[(r.head + r.len) % 8] = x;
    r.head = r.head + 1;
    oldest
}"""

# No gaming control: `pop` indexes `buf[head]`, so any weaker invariant that
# admits `head == 8` makes the existing operations fail the gate.
BMB_GAME = None

RUST_REF = """    pub fn push_overwrite(&mut self, x: i64) -> Option<i64> {
        if self.len < 8 {
            self.buf[(self.head + self.len) % 8] = x;
            self.len += 1;
            return None;
        }
        let oldest = self.buf[self.head];
        self.buf[self.head] = x;
        self.head = (self.head + 1) % 8;
        Some(oldest)
    }
"""

RUST_WEAK = """    pub fn push_overwrite(&mut self, x: i64) -> Option<i64> {
        if self.len < 8 {
            self.buf[(self.head + self.len) % 8] = x;
            self.len += 1;
            return None;
        }
        let oldest = self.get(0);
        self.buf[(self.head + self.len) % 8] = x;
        self.head += 1;
        oldest
    }
"""

RUST_WEAK_TESTS = """use solution::Ring;

#[test]
fn overwrite_drops_the_oldest() {
    let mut r = Ring::new();
    for x in 0..8 {
        assert_eq!(r.push_overwrite(x), None);
    }
    assert_eq!(r.push_overwrite(8), Some(0));
    assert_eq!(r.push_overwrite(9), Some(1));
    let all: Vec<i64> = (0..r.size()).map(|i| r.get(i).unwrap()).collect();
    assert_eq!(all, vec![2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(r.pop(), Some(2));
    assert_eq!(r.size(), 7);
}
"""

RUST_MUTANTS = [
    RUST_WEAK,
    # Drops the newest instead of the oldest.
    """    pub fn push_overwrite(&mut self, x: i64) -> Option<i64> {
        if self.len < 8 {
            self.push(x);
            return None;
        }
        let i = (self.head + 7) % 8;
        let newest = self.buf[i];
        self.buf[i] = x;
        Some(newest)
    }
""",
    # Pops and pushes, but returns nothing when it drops a value.
    """    pub fn push_overwrite(&mut self, x: i64) -> Option<i64> {
        if self.len == 8 {
            self.pop();
        }
        self.push(x);
        None
    }
""",
    # Counts one value too many when full.
    """    pub fn push_overwrite(&mut self, x: i64) -> Option<i64> {
        if self.len < 8 {
            self.push(x);
            return None;
        }
        let oldest = self.buf[self.head];
        self.buf[self.head] = x;
        self.head = (self.head + 1) % 8;
        self.len += 1;
        Some(oldest)
    }
""",
]
