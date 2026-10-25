"""Hidden test-case generation helpers for BRB tasks (ADR-0034, backlog 2H-03).

Every task module defines `cases(g)` that yields argument tuples using a `Gen`.
The generator is deterministic: its random source is seeded from the task id,
so the hidden test set is a pure function of the frozen task file.

Values follow the task's parameter types: `i64` and `u64` are Python ints in
range, `usize` likewise (bounded by what the case needs), `bool` is a Python
bool, `&[i64]` / `&mut [i64]` are lists of ints.

Standard library only.
"""
import random
import zlib

I64_MIN = -(1 << 63)
I64_MAX = (1 << 63) - 1
U64_MAX = (1 << 64) - 1

# Boundary values every integer parameter is tried with (when in domain).
I64_EDGES = [I64_MIN, I64_MIN + 1, I64_MIN + 2, -(1 << 62), -(1 << 32), -1000000007, -65536, -10, -3, -2, -1,
             0, 1, 2, 3, 10, 65536, 1000000007, 1 << 32, 1 << 62, I64_MAX - 2, I64_MAX - 1, I64_MAX]


def wrap_i64(x):
    """Two's-complement wrap (for models that specify wrapping on purpose)."""
    x &= U64_MAX
    return x - (1 << 64) if x > I64_MAX else x


class Gen:
    """Deterministic random source plus the distributions tasks share."""

    def __init__(self, task_id):
        self.rng = random.Random(zlib.crc32(task_id.encode()))

    def int(self, lo, hi):
        return self.rng.randint(lo, hi)

    def choice(self, xs):
        return self.rng.choice(xs)

    def i64(self):
        """A random i64 from a mix of magnitudes (small, medium, huge, edges)."""
        k = self.rng.randrange(6)
        if k == 0:
            return self.rng.randint(-10, 10)
        if k == 1:
            return self.rng.randint(-1000000, 1000000)
        if k == 2:
            return self.rng.randint(-(1 << 40), 1 << 40)
        if k == 3:
            return self.rng.randint(I64_MIN, I64_MAX)
        if k == 4:
            return self.rng.choice(I64_EDGES)
        return self.rng.choice([I64_MAX - self.rng.randint(0, 1000), I64_MIN + self.rng.randint(0, 1000)])

    def i64_in(self, lo, hi):
        """A random value in [lo, hi], biased to the ends and to small values."""
        k = self.rng.randrange(4)
        if k == 0:
            return self.rng.choice([lo, hi, min(hi, lo + 1), max(lo, hi - 1)])
        if k == 1 and lo <= 0 <= hi:
            return max(lo, min(hi, self.rng.randint(-10, 10)))
        return self.rng.randint(lo, hi)

    def array(self, n_max, elem):
        """A list of length 0..n_max (short lengths more likely) of `elem()`."""
        n = self.rng.choice([0, 1, 2, 3, 4, 5, self.rng.randint(0, n_max), self.rng.randint(0, n_max)])
        return [elem() for _ in range(n)]

    def elem_mixed(self):
        """An element generator: one distribution per array (dense duplicates,
        moderate values, any i64, or extremes)."""
        k = self.rng.randrange(4)
        if k == 0:
            return self.small_elem(self.rng.choice([1, 2, 3, 5]))
        if k == 1:
            return lambda: self.rng.randint(-1000000, 1000000)
        if k == 2:
            return self.i64
        return lambda: self.rng.choice(I64_EDGES)

    def arrays(self, count, n_max):
        """`count` arrays with mixed element distributions."""
        return [self.array(n_max, self.elem_mixed()) for _ in range(count)]

    def small_elem(self, spread=3):
        return lambda: self.rng.randint(-spread, spread)

    def sorted_array(self, n_max, elem):
        return sorted(self.array(n_max, elem))


def take(task, n=None):
    """The hidden cases of a task module: in-domain, deduplicated, in order."""
    g = Gen(task.ID)
    seen = set()
    out = []
    for args in task.cases(g):
        key = repr(args)
        if key in seen or not task.domain(*[list(a) if isinstance(a, list) else a for a in args]):
            continue
        seen.add(key)
        out.append(args)
        if n is not None and len(out) >= n:
            break
    return out
