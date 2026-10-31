ID = "e01-sorted-set"
SEED = "e01_sorted_set"
UNIT = "sset"
BMB_TYPE = "Set"
RUST_TYPE = "SortedSet"
NAME = "replace"
KIND = "sorted set"
STATEMENT = ("Add the operation `replace(from, to)`: if `from` is in the set and `to` is not, take `from` out "
             "and put `to` in, and return true; otherwise leave the set unchanged and return false.")
SIG_BMB = "fn replace(s: &mut Set, from: i64, to: i64) -> bool"
SIG_RUST = "pub fn replace(&mut self, from: i64, to: i64) -> bool"

# The raw state the judge reads after every operation, in this order.
RAW = [("len", "usize"), ("items", "[i64; 8]")]

# (name, argument types, result kind, BMB call, Rust call); `{a}`, `{b}` are
# the arguments, `{name}` the operation under test.
OPS = [
    ("insert", ["i64"], "bool", "sset::Set::insert(&mut s, {a})", "s.insert({a})"),
    ("remove", ["i64"], "bool", "sset::Set::remove(&mut s, {a})", "s.remove({a})"),
    ("contains", ["i64"], "bool", "sset::Set::contains(&s, {a})", "s.contains({a})"),
    ("size", [], "usize", "sset::Set::size(&s)", "s.size()"),
    ("at", ["usize"], "opt", "sset::Set::at(&s, {a})", "s.at({a})"),
    ("replace", ["i64", "i64"], "bool", "sset::{name}(&mut s, {a}, {b})", "s.{name}({a}, {b})"),
]


def invariant(raw):
    n, xs = raw["len"], raw["items"]
    return 0 <= n <= 8 and all(xs[i] < xs[i + 1] for i in range(n - 1))


def view(raw):
    return list(raw["items"][:raw["len"]])


def init():
    return []


def model_view(st):
    return list(st)


def step(st, op, a, b):
    if op == "insert":
        if len(st) == 8 or a in st:
            return (0, 0)
        st.append(a)
        st.sort()
        return (1, 0)
    if op == "remove":
        if a not in st:
            return (0, 0)
        st.remove(a)
        return (1, 0)
    if op == "contains":
        return (int(a in st), 0)
    if op == "size":
        return (len(st), 0)
    if op == "at":
        return (1, st[a]) if a < len(st) else (0, 0)
    if a in st and b not in st:
        st.remove(a)
        st.append(b)
        st.sort()
        return (1, 0)
    return (0, 0)


EDGES = [-(1 << 63), -(1 << 63) + 1, (1 << 63) - 2, (1 << 63) - 1]


def cases(g):
    def val():
        return g.choice(EDGES) if g.int(0, 19) == 0 else g.int(-4, 12)

    for _ in range(400):
        seen = [val() for _ in range(g.int(0, 9))]
        script = [("insert", x, 0) for x in seen]
        for _ in range(g.int(1, 30)):
            k = g.int(0, 99)
            if k < 40:
                # `from` is mostly a value that was put in.
                a = g.choice(seen) if seen and g.int(0, 2) else val()
                b = val()
                seen.append(b)
                script.append(("replace", a, b))
            elif k < 65:
                seen.append(val())
                script.append(("insert", seen[-1], 0))
            elif k < 75:
                script.append(("remove", val(), 0))
            elif k < 85:
                script.append(("contains", val(), 0))
            elif k < 92:
                script.append(("at", g.int(0, 9), 0))
            else:
                script.append(("size", 0, 0))
        yield script


# The reference: take `from` out and put `to` in through the existing
# operations (after `remove` there is room, and `to` is not there).
BMB_REF = """#[effects(pure)]
fn replace(s: &mut Set, from: i64, to: i64) -> bool {
    if !Set::contains(s, from) || Set::contains(s, to) {
        return false;
    }
    let taken: bool = Set::remove(s, from);
    let added: bool = Set::insert(s, to);
    true
}"""

# Weak: overwrites `from` with `to` in place, which breaks the order unless
# `to` happens to fit between the neighbours.
BMB_WEAK = """#[effects(pure)]
#[uses(sorted)]
fn replace(s: &mut Set, from: i64, to: i64) -> bool {
    if Set::contains(s, to) {
        return false;
    }
    for i in 0..s.len {
        if s.items[i] == from {
            s.items[i] = to;
            return true;
        }
    }
    false
}"""

# Gaming: the weak operation, with the invariant weakened to what the other
# operations need to be safe (the order is dropped).
BMB_GAME = """spec fn sorted(s: &Set) -> bool {
    s.len <= 8
}

""" + BMB_WEAK

RUST_REF = """    pub fn replace(&mut self, from: i64, to: i64) -> bool {
        if !self.contains(from) || self.contains(to) {
            return false;
        }
        self.remove(from);
        self.insert(to);
        true
    }
"""

RUST_WEAK = """    pub fn replace(&mut self, from: i64, to: i64) -> bool {
        if self.contains(to) {
            return false;
        }
        for i in 0..self.len {
            if self.items[i] == from {
                self.items[i] = to;
                return true;
            }
        }
        false
    }
"""

RUST_WEAK_TESTS = """use solution::SortedSet;

#[test]
fn replace_swaps_a_value() {
    let mut s = SortedSet::new();
    for x in [1, 3, 5] {
        s.insert(x);
    }
    assert!(s.replace(3, 4));
    assert!(s.contains(4));
    assert!(!s.contains(3));
    assert_eq!(s.size(), 3);
}

#[test]
fn replace_refuses_missing_or_present() {
    let mut s = SortedSet::new();
    s.insert(1);
    s.insert(2);
    assert!(!s.replace(7, 8));
    assert!(!s.replace(1, 2));
    assert_eq!(s.size(), 2);
}
"""

# Plausible wrong operations: the hidden sequences must catch each.
RUST_MUTANTS = [
    RUST_WEAK,
    # Forgets to check that `to` is new: a duplicate.
    """    pub fn replace(&mut self, from: i64, to: i64) -> bool {
        for i in 0..self.len {
            if self.items[i] == from {
                self.items[i] = to;
                let mut j = i;
                while j > 0 && self.items[j - 1] > self.items[j] {
                    self.items.swap(j - 1, j);
                    j -= 1;
                }
                while j + 1 < self.len && self.items[j] > self.items[j + 1] {
                    self.items.swap(j, j + 1);
                    j += 1;
                }
                return true;
            }
        }
        false
    }
""",
    # Removes before checking `to`: the set shrinks when `to` is present.
    """    pub fn replace(&mut self, from: i64, to: i64) -> bool {
        if !self.remove(from) {
            return false;
        }
        self.insert(to)
    }
""",
    # Appends `to` at the end after removing `from`.
    """    pub fn replace(&mut self, from: i64, to: i64) -> bool {
        if self.contains(to) || !self.remove(from) {
            return false;
        }
        self.items[self.len] = to;
        self.len += 1;
        true
    }
""",
]
