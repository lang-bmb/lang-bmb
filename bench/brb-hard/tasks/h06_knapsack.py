# QuixBugs `knapsack` (defect: `weight < j` skips an item that fills the
# remaining capacity exactly; the fix is `weight <= j`).

ID = "h06-knapsack"
NAME = "knapsack"
PARAMS = [("capacity", "usize"), ("weights", "&[i64]"), ("values", "&[i64]")]
RET = "i64"
STATEMENT = ("Item i has weight weights[i] and value values[i]. Each item may be taken at most once. Return the "
             "largest total value of a selection of items whose total weight is at most capacity (0 when "
             "nothing fits). For example knapsack(100, [60, 50, 20, 20, 8, 3], [10, 8, 4, 4, 3, 2]) = 19 and "
             "knapsack(5, [5], [7]) = 7.")
DOMAIN = ("capacity <= 1000; weights and values have the same length, at most 30; every weight is in 1..=1000 "
          "and every value in 0..=1000000000000000 (so every total fits in i64).")
BMB_DOMAIN = ("spec fn dom(capacity: usize, weights: &[i64], values: &[i64]) -> bool { capacity <= 1000 && "
              "weights.len() == values.len() && weights.len() <= 30 && "
              "(forall|i: usize| i < weights.len() ==> 1 <= #[trigger] weights[i] && weights[i] <= 1000) && "
              "(forall|i: usize| i < values.len() ==> 0 <= #[trigger] values[i] && values[i] <= 1000000000000000) }")


def domain(capacity, weights, values):
    return (capacity <= 1000 and len(weights) == len(values) <= 30 and all(1 <= w <= 1000 for w in weights)
            and all(0 <= v <= 10 ** 15 for v in values))


def model(capacity, weights, values):
    best = [0] * (capacity + 1)
    for w, v in zip(weights, values):
        for c in range(capacity, w - 1, -1):
            best[c] = max(best[c], best[c - w] + v)
    return best[capacity]


def cases(g):
    yield (100, [60, 50, 20, 20, 8, 3], [10, 8, 4, 4, 3, 2])
    yield (5, [5], [7])
    yield (0, [1], [5])
    yield (10, [], [])
    for _ in range(420):
        n = g.int(0, 12)
        k = g.int(0, 3)
        if k == 0:
            ws = [g.int(1, 10) for _ in range(n)]
            cap = g.int(0, 30)
        elif k == 1:
            ws = [g.int(1, 1000) for _ in range(n)]
            cap = g.int(0, 1000)
        elif k == 2:
            ws = [g.int(1, 6) for _ in range(n)]
            cap = sum(ws) if g.int(0, 1) else g.choice(ws) if ws else 0
        else:
            ws = [g.int(1, 50) for _ in range(n)]
            cap = 2 * g.choice(ws) if ws else 0
        vs = [g.choice([g.int(0, 20), g.int(0, 10 ** 15), 10 ** 15]) for _ in range(n)]
        yield (cap, ws, vs)
    for _ in range(30):
        n = 30
        yield (1000, [g.int(1, 1000) for _ in range(n)], [g.int(0, 10 ** 15) for _ in range(n)])


# best[c]: the largest value within weight c over the items seen so far,
# capacities downward so each item counts once. Totals fit in the domain,
# so the addition saturates only to spare the proof the bound.
BMB_REF = """#[effects(pure)]
fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> (r: i64)
    requires capacity <= 1000 && weights.len() == values.len(),
{
    let mut best: [i64; 1001] = [0; 1001];
    let mut i: usize = 0;
    while i < weights.len()
        invariant i <= weights.len() && weights.len() == values.len() && capacity <= 1000,
        decreases weights.len() - i,
    {
        let w: i64 = weights[i];
        let v: i64 = values[i];
        if w >= 1 && w <= 1000 && v >= 0 {
            let wu: usize = w as usize;
            let mut c: usize = capacity;
            while c >= wu
                invariant c <= capacity && capacity <= 1000 && wu >= 1,
                decreases c,
            {
                let take: i64 = best[c - wu].saturating_add(v);
                if take > best[c] {
                    best[c] = take;
                }
                c = c - 1;
            }
        }
        i = i + 1;
    }
    best[capacity]
}"""

# Weak: capacities upward lets an item count more than once.
BMB_WEAK = """#[effects(pure)]
fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> (r: i64)
    requires capacity <= 1000 && weights.len() == values.len(),
{
    let mut best: [i64; 1001] = [0; 1001];
    let mut i: usize = 0;
    while i < weights.len()
        invariant i <= weights.len() && weights.len() == values.len() && capacity <= 1000,
        decreases weights.len() - i,
    {
        let w: i64 = weights[i];
        let v: i64 = values[i];
        if w >= 1 && w <= 1000 && v >= 0 {
            let wu: usize = w as usize;
            let mut c: usize = wu;
            while c <= capacity
                invariant c >= wu && capacity <= 1000 && wu >= 1,
                decreases capacity + 1 - c,
            {
                let take: i64 = best[c - wu].saturating_add(v);
                if take > best[c] {
                    best[c] = take;
                }
                c = c + 1;
            }
        }
        i = i + 1;
    }
    best[capacity]
}"""

RUST_REF = """pub fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> i64 {
    let mut best = vec![0i64; capacity + 1];
    for (&w, &v) in weights.iter().zip(values) {
        let w = w as usize;
        if w > capacity {
            continue;
        }
        for c in (w..=capacity).rev() {
            best[c] = best[c].max(best[c - w] + v);
        }
    }
    best[capacity]
}
"""

# Weak (QuixBugs): `w < c` skips an item that fills the capacity exactly;
# the tests never fill it exactly.
RUST_WEAK = """pub fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> i64 {
    let n = weights.len();
    let mut memo = vec![vec![0i64; capacity + 1]; n + 1];
    for i in 1..=n {
        let (w, v) = (weights[i - 1] as usize, values[i - 1]);
        for c in 1..=capacity {
            memo[i][c] = memo[i - 1][c];
            if w < c {
                memo[i][c] = memo[i][c].max(v + memo[i - 1][c - w]);
            }
        }
    }
    memo[n][capacity]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packs() {
        assert_eq!(knapsack(100, &[60, 50, 20, 20, 8, 3], &[10, 8, 4, 4, 3, 2]), 19);
        assert_eq!(knapsack(10, &[], &[]), 0);
        assert_eq!(knapsack(0, &[1], &[5]), 0);
        assert_eq!(knapsack(9, &[4, 3], &[5, 6]), 11);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs.
    "pub fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> i64 { let n = weights.len();"
    " let mut m = vec![vec![0i64; capacity + 1]; n + 1]; for i in 1..=n { let (w, v) = (weights[i - 1] as usize, values[i - 1]);"
    " for c in 1..=capacity { m[i][c] = m[i - 1][c]; if w < c { m[i][c] = m[i][c].max(v + m[i - 1][c - w]); } } } m[n][capacity] }",
    "pub fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> i64 { let mut b = vec![0i64; capacity + 1];"
    " for (&w, &v) in weights.iter().zip(values) { let w = w as usize; for c in w..=capacity { b[c] = b[c].max(b[c - w] + v); } } b[capacity] }",
    "pub fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> i64 { let mut idx: Vec<usize> = (0..weights.len()).collect();"
    " idx.sort_by(|&a, &b| (values[b] as i128 * weights[a] as i128).cmp(&(values[a] as i128 * weights[b] as i128)));"
    " let (mut left, mut total) = (capacity as i64, 0i64); for i in idx { if weights[i] <= left { left -= weights[i]; total += values[i]; } } total }",
    "pub fn knapsack(capacity: usize, weights: &[i64], values: &[i64]) -> i64 { let mut b = vec![0i64; capacity + 1];"
    " for (&w, &v) in weights.iter().zip(values) { let w = w as usize; if w >= capacity { continue; }"
    " for c in (w..=capacity).rev() { b[c] = b[c].max(b[c - w] + v); } } b[capacity] }",
]
