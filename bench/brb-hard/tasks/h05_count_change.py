# QuixBugs `possible_change` (defect: no base case for running out of coin
# kinds, `first, *rest = coins` fails on an empty list).

ID = "h05-count-change"
NAME = "count_change"
PARAMS = [("coins", "&[i64]"), ("total", "i64")]
RET = "u64"
STATEMENT = ("coins holds distinct positive coin values, each available in unlimited number. Return the number "
             "of distinct ways to make exactly total from them, where a way is a multiset of coins (the order "
             "of coins does not matter). There is exactly one way to make 0 (no coins). For example "
             "count_change([1, 5, 10, 25], 11) = 4, count_change([2], 3) = 0, count_change([], 0) = 1.")
DOMAIN = ("coins has at most 12 distinct values, each in 1..=500; total is in 0..=500 "
          "(the result then fits in u64).")
BMB_DOMAIN = ("spec fn dom(coins: &[i64], total: i64) -> bool { coins.len() <= 12 && total >= 0 && total <= 500 && "
              "(forall|i: usize| i < coins.len() ==> 1 <= #[trigger] coins[i] && coins[i] <= 500) }")


def domain(coins, total):
    return (len(coins) <= 12 and 0 <= total <= 500 and all(1 <= c <= 500 for c in coins)
            and len(set(coins)) == len(coins))


def model(coins, total):
    ways = [1] + [0] * total
    for c in coins:
        for t in range(c, total + 1):
            ways[t] += ways[t - c]
    return ways[total]


def cases(g):
    yield ([], 0)
    yield ([], 7)
    yield ([1, 5, 10, 25], 11)
    yield ([2], 3)
    yield ([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], 500)
    for _ in range(450):
        k = g.int(0, 2)
        pool = list(range(1, 13)) if k == 0 else (list(range(1, 501)) if k == 1 else [1, 2, 5, 10, 20, 50, 100, 200])
        n = g.int(0, min(12, len(pool)))
        coins = g.rng.sample(pool, n)
        yield (coins, g.choice([0, 1, g.int(0, 30), g.int(0, 500), 500]))


# ways[t] after the coins seen so far; coin by coin (outer loop) so each
# multiset counts once. Counts stay below u64::MAX in the domain, so the
# additions saturate only to spare the proof the bound.
BMB_REF = """#[effects(pure)]
fn count_change(coins: &[i64], total: i64) -> (r: u64)
    requires coins.len() <= 12 && total >= 0 && total <= 500,
{
    let n: usize = total as usize;
    let mut ways: [u64; 501] = [0; 501];
    ways[0] = 1;
    let mut i: usize = 0;
    while i < coins.len()
        invariant i <= coins.len() && n <= 500,
        decreases coins.len() - i,
    {
        let c: i64 = coins[i];
        if c >= 1 && c <= 500 {
            let cu: usize = c as usize;
            let mut t: usize = cu;
            while t <= n
                invariant cu >= 1 && cu <= t && n <= 500,
                decreases n + 1 - t,
            {
                ways[t] = ways[t].saturating_add(ways[t - cu]);
                t = t + 1;
            }
        }
        i = i + 1;
    }
    ways[n]
}"""

# Weak: totals in the outer loop count ordered sequences (compositions).
BMB_WEAK = """#[effects(pure)]
fn count_change(coins: &[i64], total: i64) -> (r: u64)
    requires coins.len() <= 12 && total >= 0 && total <= 500,
{
    let n: usize = total as usize;
    let mut ways: [u64; 501] = [0; 501];
    ways[0] = 1;
    let mut t: usize = 1;
    while t <= n
        invariant t >= 1 && n <= 500,
        decreases n + 1 - t,
    {
        let mut i: usize = 0;
        while i < coins.len()
            invariant i <= coins.len() && t <= 500,
            decreases coins.len() - i,
        {
            let c: i64 = coins[i];
            if c >= 1 && c <= 500 && (c as usize) <= t {
                ways[t] = ways[t].saturating_add(ways[t - (c as usize)]);
            }
            i = i + 1;
        }
        t = t + 1;
    }
    ways[n]
}"""

RUST_REF = """pub fn count_change(coins: &[i64], total: i64) -> u64 {
    let n = total as usize;
    let mut ways = vec![0u64; n + 1];
    ways[0] = 1;
    for &c in coins {
        let c = c as usize;
        for t in c..=n {
            ways[t] += ways[t - c];
        }
    }
    ways[n]
}
"""

# Weak: the QuixBugs recursion with the missing base case patched to
# "no coins left: 0 ways", which is wrong for total 0 with no coins; the
# tests always pass coins.
RUST_WEAK = """pub fn count_change(coins: &[i64], total: i64) -> u64 {
    if coins.is_empty() {
        return 0;
    }
    let n = total as usize;
    let mut ways = vec![0u64; n + 1];
    ways[0] = 1;
    for &c in coins {
        let c = c as usize;
        for t in c..=n {
            ways[t] += ways[t - c];
        }
    }
    ways[n]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ways() {
        assert_eq!(count_change(&[1, 5, 10, 25], 11), 4);
        assert_eq!(count_change(&[2], 3), 0);
        assert_eq!(count_change(&[1, 2], 4), 3);
        assert_eq!(count_change(&[3], 0), 1);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs (recursion without the empty-coins base case; panics).
    "pub fn count_change(coins: &[i64], total: i64) -> u64 { fn go(c: &[i64], t: i64) -> u64 { if t == 0 { return 1; }"
    " if t < 0 { return 0; } let (first, rest) = (c[0], &c[1..]); go(c, t - first) + go(rest, t) } go(coins, total) }",
    "pub fn count_change(coins: &[i64], total: i64) -> u64 { if coins.is_empty() { return 0; } let n = total as usize;"
    " let mut w = vec![0u64; n + 1]; w[0] = 1; for &c in coins { let c = c as usize; for t in c..=n { w[t] += w[t - c]; } } w[n] }",
    "pub fn count_change(coins: &[i64], total: i64) -> u64 { let n = total as usize; let mut w = vec![0u64; n + 1]; w[0] = 1;"
    " for t in 1..=n { for &c in coins { let c = c as usize; if c <= t { w[t] += w[t - c]; } } } w[n] }",
    "pub fn count_change(coins: &[i64], total: i64) -> u64 { let n = total as usize; let mut w = vec![0u64; n + 1]; w[0] = 1;"
    " for &c in coins { let c = c as usize; for t in (c..=n).rev() { w[t] += w[t - c]; } } w[n] }",
]
