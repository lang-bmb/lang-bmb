ID = "t23-sort"
NAME = "sort"
PARAMS = [("a", "&mut [i64]")]
RET = "()"
STATEMENT = "Sort a in place into non-decreasing order."
DOMAIN = "a is any slice of i64 values (possibly empty)."
BMB_DOMAIN = None


def domain(a):
    return True


def model(a):
    a.sort()


def cases(g):
    yield ([],)
    for a in g.arrays(350, 16):
        yield (a,)
    yield (list(range(40, 0, -1)),)


BMB_REF = """#[effects(pure)]
fn sort(a: &mut [i64]) -> () {
    let mut i: usize = 1;
    while i < a.len()
        invariant 1 <= i,
        decreases a.len() - i,
    {
        let x: i64 = a[i];
        let mut j: usize = i;
        while j > 0 && a[j - 1] > x
            invariant j <= i, i < a.len(),
            decreases j,
        {
            a[j] = a[j - 1];
            j = j - 1;
        }
        a[j] = x;
        i = i + 1;
    }
}"""

# Weak: a single bubble pass.
BMB_WEAK = """#[effects(pure)]
fn sort(a: &mut [i64]) -> () {
    let mut i: usize = 1;
    while i < a.len()
        invariant 1 <= i,
        decreases a.len() - i,
    {
        if a[i - 1] > a[i] {
            let t: i64 = a[i];
            a[i] = a[i - 1];
            a[i - 1] = t;
        }
        i = i + 1;
    }
}"""

RUST_REF = """pub fn sort(a: &mut [i64]) {
    for i in 1..a.len() {
        let x = a[i];
        let mut j = i;
        while j > 0 && a[j - 1] > x {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[j] = x;
    }
}
"""

RUST_WEAK = """pub fn sort(a: &mut [i64]) {
    for i in 1..a.len() {
        if a[i - 1] > a[i] {
            a.swap(i - 1, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sorts() {
        let mut v = vec![1, 3, 2, 4];
        sort(&mut v);
        assert_eq!(v, vec![1, 2, 3, 4]);
        let mut e: Vec<i64> = vec![];
        sort(&mut e);
        assert!(e.is_empty());
    }
}
"""

RUST_MUTANTS = [
    "pub fn sort(a: &mut [i64]) { for i in 1..a.len() { if a[i - 1] > a[i] { a.swap(i - 1, i); } } }",
    "pub fn sort(a: &mut [i64]) { for i in 1..a.len() { let x = a[i]; let mut j = i; while j > 0 && a[j - 1] < x { a[j] = a[j - 1]; j -= 1; } a[j] = x; } }",
    "pub fn sort(a: &mut [i64]) { let n = a.len(); for i in 1..n / 2 + n % 2 { let x = a[i]; let mut j = i; while j > 0 && a[j - 1] > x { a[j] = a[j - 1]; j -= 1; } a[j] = x; } }",
    "pub fn sort(a: &mut [i64]) { for i in 1..a.len() { let x = a[i]; let mut j = i; while j > 1 && a[j - 1] > x { a[j] = a[j - 1]; j -= 1; } a[j] = x; } }",
]
