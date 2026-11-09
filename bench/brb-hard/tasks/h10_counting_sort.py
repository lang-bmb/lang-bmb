# QuixBugs `bucketsort` (counting sort; defect: the output loop enumerates
# `arr` instead of `counts`).

ID = "h10-counting-sort"
NAME = "counting_sort"
PARAMS = [("a", "&mut [i64]"), ("k", "i64")]
RET = "()"
STATEMENT = ("Every element of a is in 0..k (at least 0, less than k). Sort a in non-decreasing order in place, "
             "keeping every element (duplicates included). For example with k = 5, [3, 0, 3, 1] becomes "
             "[0, 1, 3, 3].")
DOMAIN = "k is in 1..=64; a has at most 1000 elements, each in 0..k."
BMB_DOMAIN = ("spec fn dom(a: &[i64], k: i64) -> bool { 1 <= k && k <= 64 && a.len() <= 1000 && "
              "(forall|i: usize| i < a.len() ==> 0 <= #[trigger] a[i] && a[i] < k) }")


def domain(a, k):
    return 1 <= k <= 64 and len(a) <= 1000 and all(0 <= x < k for x in a)


def model(a, k):
    a.sort()


def cases(g):
    yield ([], 1)
    yield ([0], 1)
    yield ([3, 0, 3, 1], 5)
    for _ in range(450):
        k = g.choice([1, 2, 3, 5, 8, 16, 63, 64, g.int(1, 64)])
        n = g.choice([0, 1, 2, 5, g.int(0, 30), g.int(0, 1000)])
        yield ([g.int(0, k - 1) for _ in range(n)], k)


# Counts per value, then the values written back in order. The write
# position never passes the length (the counts sum to it); the guard spares
# the proof that sum.
BMB_REF = """#[effects(pure)]
fn counting_sort(a: &mut [i64], k: i64) -> ()
    requires 1 <= k && k <= 64 && a.len() <= 1000,
{
    let mut counts: [usize; 64] = [0; 64];
    let mut i: usize = 0;
    while i < a.len()
        invariant i <= a.len() && a.len() <= 1000,
        decreases a.len() - i,
    {
        let x: i64 = a[i];
        if x >= 0 && x < 64 {
            counts[x as usize] = counts[x as usize].saturating_add(1);
        }
        i = i + 1;
    }
    let mut pos: usize = 0;
    let mut v: usize = 0;
    while v < 64
        invariant v <= 64,
        decreases 64 - v,
    {
        let mut c: usize = 0;
        while c < counts[v]
            invariant v < 64,
            decreases counts[v] - c,
        {
            if pos < a.len() {
                a[pos] = v as i64;
                pos = pos + 1;
            }
            c = c + 1;
        }
        v = v + 1;
    }
}"""

# Weak: one slot per value that occurs (duplicates written once), the rest
# of the slice left as it was.
BMB_WEAK = BMB_REF.replace("""        let mut c: usize = 0;
        while c < counts[v]
            invariant v < 64,
            decreases counts[v] - c,
        {
            if pos < a.len() {
                a[pos] = v as i64;
                pos = pos + 1;
            }
            c = c + 1;
        }""", """        if counts[v] > 0 && pos < a.len() {
            a[pos] = v as i64;
            pos = pos + 1;
        }""")

RUST_REF = """pub fn counting_sort(a: &mut [i64], k: i64) {
    let mut counts = vec![0usize; k as usize];
    for &x in a.iter() {
        counts[x as usize] += 1;
    }
    let mut pos = 0;
    for (v, &c) in counts.iter().enumerate() {
        for _ in 0..c {
            a[pos] = v as i64;
            pos += 1;
        }
    }
}
"""

# Weak: the counts sized k - 1 (the top value dropped from the counts); the
# tests never contain k - 1.
RUST_WEAK = """pub fn counting_sort(a: &mut [i64], k: i64) {
    let mut counts = vec![0usize; k as usize];
    for &x in a.iter() {
        if ((x + 1) as usize) < counts.len() {
            counts[x as usize] += 1;
        }
    }
    let mut pos = 0;
    for (v, &c) in counts.iter().enumerate() {
        for _ in 0..c {
            a[pos] = v as i64;
            pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sorts() {
        let mut a = [3, 0, 3, 1];
        counting_sort(&mut a, 5);
        assert_eq!(a, [0, 1, 3, 3]);
        let mut b: [i64; 0] = [];
        counting_sort(&mut b, 1);
        let mut c = [2, 1, 0, 1];
        counting_sort(&mut c, 8);
        assert_eq!(c, [0, 1, 1, 2]);
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs: the output loop enumerates the input instead of the counts.
    "pub fn counting_sort(a: &mut [i64], k: i64) { let mut counts = vec![0usize; k as usize]; for &x in a.iter() { counts[x as usize] += 1; }"
    " let src = a.to_vec(); let mut out: Vec<i64> = Vec::new(); for (i, &c) in src.iter().enumerate() { for _ in 0..c { out.push(i as i64); } }"
    " for (p, v) in out.into_iter().enumerate().take(a.len()) { a[p] = v; } }",
    "pub fn counting_sort(a: &mut [i64], k: i64) { let mut counts = vec![0usize; k as usize]; for &x in a.iter() {"
    " if ((x + 1) as usize) < counts.len() { counts[x as usize] += 1; } } let mut pos = 0; for (v, &c) in counts.iter().enumerate() {"
    " for _ in 0..c { a[pos] = v as i64; pos += 1; } } }",
    "pub fn counting_sort(a: &mut [i64], k: i64) { let mut seen = vec![false; k as usize]; for &x in a.iter() { seen[x as usize] = true; }"
    " let mut pos = 0; for (v, &s) in seen.iter().enumerate() { if s { a[pos] = v as i64; pos += 1; } } }",
    "pub fn counting_sort(a: &mut [i64], k: i64) { let _ = k; a.sort_by(|x, y| y.cmp(x)); }",
]
