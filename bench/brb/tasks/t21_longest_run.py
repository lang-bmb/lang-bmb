ID = "t21-longest-run"
NAME = "longest_run"
PARAMS = [("a", "&[i64]")]
RET = "usize"
STATEMENT = "Return the length of the longest run of consecutive equal elements of a (0 for an empty slice, 1 when no two neighbours are equal)."
DOMAIN = "a is any slice of i64 values (possibly empty)."
BMB_DOMAIN = None


def domain(a):
    return True


def model(a):
    best = cur = 0
    for i, x in enumerate(a):
        cur = cur + 1 if i > 0 and a[i - 1] == x else 1
        best = max(best, cur)
    return best


def cases(g):
    yield ([],)
    yield ([1, 2, 2, 2],)
    for _ in range(350):
        e = g.elem_mixed()
        a = []
        for _ in range(g.int(0, 8)):
            a += [e()] * g.int(1, 4)
        yield (a,)


BMB_REF = """spec fn runs_ok(cur: usize, best: usize, k: usize) -> bool { cur <= k && best <= k }

#[effects(pure)]
#[uses(runs_ok)]
fn longest_run(a: &[i64]) -> (r: usize) {
    let mut best: usize = 0;
    let mut cur: usize = 0;
    let mut k: usize = 0;
    while k < a.len()
        invariant k <= a.len(), runs_ok(cur, best, k),
        decreases a.len() - k,
    {
        if k > 0 && a[k] == a[k - 1] {
            cur = cur + 1;
        } else {
            cur = 1;
        }
        if cur > best {
            best = cur;
        }
        k = k + 1;
    }
    best
}"""

# Weak: the best is updated only when a run ends, so a final longest run is missed.
BMB_WEAK = """spec fn runs_ok(cur: usize, best: usize, k: usize) -> bool { cur <= k && best <= k }

#[effects(pure)]
#[uses(runs_ok)]
fn longest_run(a: &[i64]) -> (r: usize) {
    if a.len() == 0 {
        0
    } else {
        let mut best: usize = 1;
        let mut cur: usize = 1;
        let mut k: usize = 1;
        while k < a.len()
            invariant 1 <= k, k <= a.len(), runs_ok(cur, best, k),
            decreases a.len() - k,
        {
            if a[k] == a[k - 1] {
                cur = cur + 1;
            } else {
                if cur > best { best = cur; }
                cur = 1;
            }
            k = k + 1;
        }
        best
    }
}"""

RUST_REF = """pub fn longest_run(a: &[i64]) -> usize {
    let (mut best, mut cur) = (0, 0);
    for k in 0..a.len() {
        cur = if k > 0 && a[k] == a[k - 1] { cur + 1 } else { 1 };
        best = best.max(cur);
    }
    best
}
"""

RUST_WEAK = """pub fn longest_run(a: &[i64]) -> usize {
    if a.is_empty() {
        return 0;
    }
    let (mut best, mut cur) = (1, 1);
    for k in 1..a.len() {
        if a[k] == a[k - 1] {
            cur += 1;
        } else {
            best = best.max(cur);
            cur = 1;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs() {
        assert_eq!(longest_run(&[]), 0);
        assert_eq!(longest_run(&[1, 1, 1, 2]), 3);
        assert_eq!(longest_run(&[1, 2, 3]), 1);
    }
}
"""

RUST_MUTANTS = [
    "pub fn longest_run(a: &[i64]) -> usize { if a.is_empty() { return 0; } let (mut best, mut cur) = (1, 1); for k in 1..a.len() { if a[k] == a[k - 1] { cur += 1; } else { best = best.max(cur); cur = 1; } } best }",
    "pub fn longest_run(a: &[i64]) -> usize { let (mut best, mut cur) = (0, 0); for k in 1..a.len() { cur = if a[k] == a[k - 1] { cur + 1 } else { 0 }; best = best.max(cur); } best }",
    "pub fn longest_run(a: &[i64]) -> usize { let mut v = a.to_vec(); v.dedup(); a.len() - v.len() + (!a.is_empty()) as usize }",
    "pub fn longest_run(a: &[i64]) -> usize { let (mut best, mut cur) = (0, 0); for k in 0..a.len() { cur = if k > 0 && a[k] == a[k - 1] { cur + 1 } else { 1 }; best = best.max(cur); } best.max(1) }",
]
