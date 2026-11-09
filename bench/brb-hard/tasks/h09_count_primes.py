# QuixBugs `sieve` (trial division over the primes found so far; defect:
# `any(n % p > 0 ...)` for `all(...)`).

ID = "h09-count-primes"
NAME = "count_primes"
PARAMS = [("n", "u64")]
RET = "u64"
STATEMENT = ("Return the number of primes p with p <= n. For example count_primes(10) = 4 (2, 3, 5, 7), "
             "count_primes(2) = 1, count_primes(1) = 0.")
DOMAIN = "n is a u64 value with n <= 20000."
BMB_DOMAIN = "spec fn dom(n: u64) -> bool { n <= 20000 }"


def domain(n):
    return n <= 20000


def model(n):
    if n < 2:
        return 0
    is_p = [True] * (n + 1)
    is_p[0] = is_p[1] = False
    for i in range(2, int(n ** 0.5) + 1):
        if is_p[i]:
            for j in range(i * i, n + 1, i):
                is_p[j] = False
    return sum(is_p)


def cases(g):
    for n in range(0, 200):
        yield (n,)
    for n in (961, 1369, 3481, 9409, 10201, 19321, 19999, 20000):
        for d in (-1, 0, 1):
            yield (n + d,)
    for _ in range(150):
        yield (g.int(0, 20000),)


BMB_REF = """#[effects(pure)]
fn count_primes(n: u64) -> (r: u64)
    requires n <= 20000,
{
    let mut composite: [bool; 20001] = [false; 20001];
    let mut count: u64 = 0;
    let mut i: u64 = 2;
    while i <= n
        invariant i >= 2 && n <= 20000 && count <= i,
        decreases n + 1 - i,
    {
        if !composite[i as usize] {
            count = count + 1;
            let mut j: u64 = i * i;
            while j <= n
                invariant n <= 20000 && i <= 20000,
                decreases n + 1 - j,
            {
                composite[j as usize] = true;
                j = j + i;
            }
        }
        i = i + 1;
    }
    count
}"""

# Weak: trial division by d with d * d < m, so squares of primes count as
# primes.
BMB_WEAK = """#[effects(pure)]
fn count_primes(n: u64) -> (r: u64)
    requires n <= 20000,
{
    let mut count: u64 = 0;
    let mut m: u64 = 2;
    while m <= n
        invariant m >= 2 && n <= 20000 && count <= m,
        decreases n + 1 - m,
    {
        let mut d: u64 = 2;
        let mut prime: bool = true;
        while d * d < m
            invariant d >= 2 && d <= m && m <= 20000,
            decreases m - d,
        {
            if m % d == 0 {
                prime = false;
            }
            d = d + 1;
        }
        if prime {
            count = count + 1;
        }
        m = m + 1;
    }
    count
}"""

RUST_REF = """pub fn count_primes(n: u64) -> u64 {
    let n = n as usize;
    if n < 2 {
        return 0;
    }
    let mut composite = vec![false; n + 1];
    let mut count = 0;
    for i in 2..=n {
        if !composite[i] {
            count += 1;
            let mut j = i * i;
            while j <= n {
                composite[j] = true;
                j += i;
            }
        }
    }
    count
}
"""

# Weak: `d * d < m` counts the squares of primes as primes; the tests stop
# at n = 3, below the first square (4).
RUST_WEAK = """pub fn count_primes(n: u64) -> u64 {
    (2..=n).filter(|&m| (2..).take_while(|d| d * d < m).all(|d| m % d != 0)).count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts() {
        assert_eq!(count_primes(0), 0);
        assert_eq!(count_primes(1), 0);
        assert_eq!(count_primes(2), 1);
        assert_eq!(count_primes(3), 2);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs: `any` for `all`.
    "pub fn count_primes(n: u64) -> u64 { let mut primes: Vec<u64> = Vec::new(); for m in 2..=n {"
    " if primes.iter().any(|p| m % p > 0) || primes.is_empty() { primes.push(m); } } primes.len() as u64 }",
    "pub fn count_primes(n: u64) -> u64 { (2..=n).filter(|&m| (2..).take_while(|d| d * d < m).all(|d| m % d != 0)).count() as u64 }",
    "pub fn count_primes(n: u64) -> u64 { (1..=n).filter(|&m| (2..).take_while(|d| d * d <= m).all(|d| m % d != 0)).count() as u64 }",
    "pub fn count_primes(n: u64) -> u64 { (2..n).filter(|&m| (2..).take_while(|d| d * d <= m).all(|d| m % d != 0)).count() as u64 }",
]
