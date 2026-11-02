from cases import I64_EDGES, I64_MAX

ID = "f01-ledger"
NAME = "ledger"
PARAMS = [("kinds", "&[i64]"), ("amounts", "&[i64]"), ("start", "i64")]
RET = "i64"
STATEMENT = (
    "Run an account ledger. The balance starts at start. Operation k has kind kinds[k] and amount amounts[k], "
    "applied in order: kind 0 deposits the amount (rejected if the new balance would exceed i64::MAX); "
    "kind 1 withdraws the amount (rejected if the amount is greater than the balance); kind 2 adds interest: "
    "the balance grows by amount for every full 100 in the balance, that is by (balance / 100) * amount with "
    "integer division (rejected if the new balance would exceed i64::MAX); kind 3 charges a fee: the amount is "
    "taken from the balance, or the whole balance if it is smaller (never rejected). A rejected operation leaves "
    "the balance unchanged. After each operation print one line: the new balance, or -1 if the operation was "
    "rejected (print each value with print_i64 in BMB, or println!(\"{}\", value) in Rust). Return the final balance.")
DOMAIN = ("kinds and amounts have the same length; every kind is 0, 1, 2 or 3; every amount is >= 0; start >= 0.")
BMB_DOMAIN = """spec fn dom(kinds: &[i64], amounts: &[i64], start: i64) -> bool {
    kinds.len() == amounts.len() && start >= 0
        && (forall|i: usize| i < kinds.len() ==> 0 <= #[trigger] kinds[i] && kinds[i] <= 3)
        && (forall|j: usize| j < amounts.len() ==> #[trigger] amounts[j] >= 0)
}"""


def domain(kinds, amounts, start):
    return len(kinds) == len(amounts) and start >= 0 and all(0 <= k <= 3 for k in kinds) and all(a >= 0 for a in amounts)


def step(b, kind, a):
    if kind == 0:
        n = b + a
    elif kind == 1:
        n = b - a if a <= b else -1
    elif kind == 2:
        n = b + (b // 100) * a
    else:
        n = max(0, b - a)
    return -1 if n > I64_MAX else n


def model(kinds, amounts, start):
    b, out = start, []
    for kind, a in zip(kinds, amounts):
        n = step(b, kind, a)
        if n < 0:
            out.append(-1)
        else:
            b = n
            out.append(b)
    return b, out


def amount(g):
    k = g.rng.randrange(5)
    if k == 0:
        return g.int(0, 10)
    if k == 1:
        return g.int(0, 100000)
    if k == 2:
        return g.choice([e for e in I64_EDGES if e >= 0])
    if k == 3:
        return g.int(0, I64_MAX)
    return g.choice([99, 100, 101, 1 << 31, 1 << 40, I64_MAX // 100, I64_MAX // 2])


def start(g):
    return g.choice([0, 1, 99, 100, 199, 1000, 1 << 32, I64_MAX // 100, I64_MAX // 2, I64_MAX - 1, I64_MAX,
                     g.int(0, 1000), g.int(0, I64_MAX)])


def cases(g):
    yield ([], [], 0)
    yield ([], [], I64_MAX)
    yield ([0], [1], I64_MAX)
    yield ([0], [0], I64_MAX)
    yield ([1], [5], 5)
    yield ([1], [6], 5)
    yield ([2], [1], 99)
    yield ([2], [3], I64_MAX // 2)
    yield ([2], [I64_MAX], 100)
    yield ([3], [I64_MAX], 7)
    yield ([3, 0, 1], [7, 3, 3], 7)
    for _ in range(420):
        n = g.choice([1, 2, 3, 4, 6, g.int(0, 12)])
        kinds = [g.int(0, 3) for _ in range(n)]
        yield (kinds, [amount(g) for _ in range(n)], start(g))


BMB_REF = """spec fn nonneg(b: i64) -> bool { b >= 0 }

spec fn outcome(r: i64) -> bool { r >= -1 }

spec fn same_len(kinds: &[i64], amounts: &[i64]) -> bool { kinds.len() == amounts.len() }

spec fn all_nonneg(a: &[i64]) -> bool { forall|i: usize| i < a.len() ==> #[trigger] a[i] >= 0 }

#[effects(pure)]
#[uses(nonneg, outcome)]
fn deposit(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if b > 9223372036854775807 - a { -1 } else { b + a }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn withdraw(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if a > b { -1 } else { b - a }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn interest(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    let q: i64 = b / 100;
    if a == 0 {
        b
    } else {
        if q > (9223372036854775807 - b) / a { -1 } else { b + q * a }
    }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn fee(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if a > b { 0 } else { b - a }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn apply(b: i64, kind: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if kind == 0 {
        deposit(b, a)
    } else {
        if kind == 1 {
            withdraw(b, a)
        } else {
            if kind == 2 { interest(b, a) } else { fee(b, a) }
        }
    }
}

#[effects(io)]
#[uses(nonneg, outcome, same_len, all_nonneg)]
fn ledger(kinds: &[i64], amounts: &[i64], start: i64) -> (r: i64)
    requires same_len(kinds, amounts), all_nonneg(amounts), nonneg(start),
{
    let mut b: i64 = start;
    let mut k: usize = 0;
    while k < kinds.len()
        invariant k <= kinds.len(), nonneg(b),
        decreases kinds.len() - k,
    {
        let n: i64 = apply(b, kinds[k], amounts[k]);
        if n < 0 {
            print_i64(-1);
        } else {
            b = n;
            print_i64(b);
        }
        k = k + 1;
    }
    b
}"""

# Weak: a rejected operation prints the unchanged balance instead of -1 (the
# returned balance is right; only the printed lines differ).
BMB_WEAK = """spec fn nonneg(b: i64) -> bool { b >= 0 }

spec fn outcome(r: i64) -> bool { r >= -1 }

spec fn same_len(kinds: &[i64], amounts: &[i64]) -> bool { kinds.len() == amounts.len() }

spec fn all_nonneg(a: &[i64]) -> bool { forall|i: usize| i < a.len() ==> #[trigger] a[i] >= 0 }

#[effects(pure)]
#[uses(nonneg, outcome)]
fn deposit(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if b > 9223372036854775807 - a { -1 } else { b + a }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn withdraw(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if a > b { -1 } else { b - a }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn interest(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    let q: i64 = b / 100;
    if a == 0 {
        b
    } else {
        if q > (9223372036854775807 - b) / a { -1 } else { b + q * a }
    }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn fee(b: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if a > b { 0 } else { b - a }
}

#[effects(pure)]
#[uses(nonneg, outcome)]
fn apply(b: i64, kind: i64, a: i64) -> (r: i64)
    requires nonneg(b), nonneg(a),
    ensures outcome(r),
{
    if kind == 0 {
        deposit(b, a)
    } else {
        if kind == 1 {
            withdraw(b, a)
        } else {
            if kind == 2 { interest(b, a) } else { fee(b, a) }
        }
    }
}

#[effects(io)]
#[uses(nonneg, outcome, same_len, all_nonneg)]
fn ledger(kinds: &[i64], amounts: &[i64], start: i64) -> (r: i64)
    requires same_len(kinds, amounts), all_nonneg(amounts), nonneg(start),
{
    let mut b: i64 = start;
    let mut k: usize = 0;
    while k < kinds.len()
        invariant k <= kinds.len(), nonneg(b),
        decreases kinds.len() - k,
    {
        let n: i64 = apply(b, kinds[k], amounts[k]);
        if n < 0 {
            print_i64(b);
        } else {
            b = n;
            print_i64(b);
        }
        k = k + 1;
    }
    b
}"""

RUST_REF = """fn step(b: i64, kind: i64, a: i64) -> Option<i64> {
    match kind {
        0 => b.checked_add(a),
        1 => if a > b { None } else { Some(b - a) },
        2 => (b / 100).checked_mul(a).and_then(|g| b.checked_add(g)),
        _ => Some(if a > b { 0 } else { b - a }),
    }
}

pub fn ledger(kinds: &[i64], amounts: &[i64], start: i64) -> i64 {
    let mut b = start;
    for (&kind, &a) in kinds.iter().zip(amounts) {
        match step(b, kind, a) {
            Some(n) => {
                b = n;
                println!("{}", b);
            }
            None => println!("-1"),
        }
    }
    b
}
"""

# Weak: interest as balance * amount / 100 (overflows long before the result
# does, and rounds differently); its tests stay on small balances.
RUST_WEAK = """fn step(b: i64, kind: i64, a: i64) -> Option<i64> {
    match kind {
        0 => b.checked_add(a),
        1 => if a > b { None } else { Some(b - a) },
        2 => b.checked_add(b * a / 100),
        _ => Some(if a > b { 0 } else { b - a }),
    }
}

pub fn ledger(kinds: &[i64], amounts: &[i64], start: i64) -> i64 {
    let mut b = start;
    for (&kind, &a) in kinds.iter().zip(amounts) {
        match step(b, kind, a) {
            Some(n) => {
                b = n;
                println!("{}", b);
            }
            None => println!("-1"),
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn steps() {
        assert_eq!(step(10, 0, 5), Some(15));
        assert_eq!(step(i64::MAX, 0, 1), None);
        assert_eq!(step(10, 1, 11), None);
        assert_eq!(step(10, 1, 10), Some(0));
        assert_eq!(step(200, 2, 3), Some(206));
        assert_eq!(step(5, 3, 9), Some(0));
    }
    #[test]
    fn runs() {
        assert_eq!(ledger(&[0, 1, 3], &[5, 20, 1], 10), 14);
    }
}
"""

_LOOP = """
pub fn ledger(kinds: &[i64], amounts: &[i64], start: i64) -> i64 {
    let mut b = start;
    for (&kind, &a) in kinds.iter().zip(amounts) {
        match step(b, kind, a) {
            Some(n) => { b = n; println!("{}", b); }
            None => println!("-1"),
        }
    }
    b
}"""

RUST_MUTANTS = [
    # deposit without the overflow check
    """fn step(b: i64, kind: i64, a: i64) -> Option<i64> {
    match kind { 0 => Some(b + a), 1 => if a > b { None } else { Some(b - a) },
        2 => (b / 100).checked_mul(a).and_then(|g| b.checked_add(g)), _ => Some(if a > b { 0 } else { b - a }) }
}""" + _LOOP,
    # withdrawing the whole balance is rejected
    """fn step(b: i64, kind: i64, a: i64) -> Option<i64> {
    match kind { 0 => b.checked_add(a), 1 => if a >= b { None } else { Some(b - a) },
        2 => (b / 100).checked_mul(a).and_then(|g| b.checked_add(g)), _ => Some(if a > b { 0 } else { b - a }) }
}""" + _LOOP,
    # interest as b * a / 100
    """fn step(b: i64, kind: i64, a: i64) -> Option<i64> {
    match kind { 0 => b.checked_add(a), 1 => if a > b { None } else { Some(b - a) },
        2 => b.checked_mul(a).map(|p| p / 100).and_then(|g| b.checked_add(g)), _ => Some(if a > b { 0 } else { b - a }) }
}""" + _LOOP,
    # a rejected operation prints the unchanged balance
    """fn step(b: i64, kind: i64, a: i64) -> Option<i64> {
    match kind { 0 => b.checked_add(a), 1 => if a > b { None } else { Some(b - a) },
        2 => (b / 100).checked_mul(a).and_then(|g| b.checked_add(g)), _ => Some(if a > b { 0 } else { b - a }) }
}
pub fn ledger(kinds: &[i64], amounts: &[i64], start: i64) -> i64 {
    let mut b = start;
    for (&kind, &a) in kinds.iter().zip(amounts) {
        if let Some(n) = step(b, kind, a) { b = n; }
        println!("{}", b);
    }
    b
}""",
]
