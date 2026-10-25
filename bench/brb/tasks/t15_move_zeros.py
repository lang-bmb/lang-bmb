ID = "t15-move-zeros"
NAME = "move_zeros"
PARAMS = [("a", "&mut [i64]")]
RET = "()"
STATEMENT = "In place, move every zero element of a to the end, keeping the relative order of the non-zero elements."
DOMAIN = "a is any slice of i64 values (possibly empty)."
BMB_DOMAIN = None


def domain(a):
    return True


def model(a):
    nz = [x for x in a if x != 0]
    a[:] = nz + [0] * (len(a) - len(nz))


def cases(g):
    yield ([],)
    yield ([0, 0],)
    for _ in range(350):
        e = g.elem_mixed()
        zero_rate = g.choice([2, 3, 5])
        yield (g.array(14, lambda: 0 if g.int(0, zero_rate) == 0 else e()),)


BMB_REF = """#[effects(pure)]
fn move_zeros(a: &mut [i64]) -> () {
    let n: usize = a.len();
    let mut w: usize = 0;
    let mut k: usize = 0;
    while k < n
        invariant w <= k, k <= n, n == a.len(),
        decreases n - k,
    {
        if a[k] != 0 {
            a[w] = a[k];
            w = w + 1;
        }
        k = k + 1;
    }
    while w < n
        invariant w <= n, n == a.len(),
        decreases n - w,
    {
        a[w] = 0;
        w = w + 1;
    }
}"""

# Weak: swaps each zero with the last unchecked element (order not kept).
BMB_WEAK = """#[effects(pure)]
fn move_zeros(a: &mut [i64]) -> () {
    let mut i: usize = 0;
    let mut j: usize = a.len();
    while i < j
        invariant i <= j, j <= a.len(),
        decreases j - i,
    {
        if a[i] == 0 {
            j = j - 1;
            a[i] = a[j];
            a[j] = 0;
        } else {
            i = i + 1;
        }
    }
}"""

RUST_REF = """pub fn move_zeros(a: &mut [i64]) {
    let mut w = 0;
    for k in 0..a.len() {
        if a[k] != 0 {
            a[w] = a[k];
            w += 1;
        }
    }
    for x in &mut a[w..] {
        *x = 0;
    }
}
"""

RUST_WEAK = """pub fn move_zeros(a: &mut [i64]) {
    let mut i = 0;
    let mut j = a.len();
    while i < j {
        if a[i] == 0 {
            j -= 1;
            a.swap(i, j);
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moves() {
        let mut v = vec![0, 1, 2];
        move_zeros(&mut v);
        assert_eq!(v, vec![2, 1, 0]);
        let mut w = vec![4, 0];
        move_zeros(&mut w);
        assert_eq!(w, vec![4, 0]);
    }
}
"""

RUST_MUTANTS = [
    "pub fn move_zeros(a: &mut [i64]) { let mut i = 0; let mut j = a.len(); while i < j { if a[i] == 0 { j -= 1; a.swap(i, j); } else { i += 1; } } }",
    "pub fn move_zeros(a: &mut [i64]) { a.sort_by_key(|&x| (x == 0, x)); }",
    "pub fn move_zeros(a: &mut [i64]) { let mut w = 0; for k in 0..a.len() { if a[k] != 0 { a[w] = a[k]; w += 1; } } }",
    "pub fn move_zeros(a: &mut [i64]) { let mut w = a.len(); for k in (0..a.len()).rev() { if a[k] != 0 { w -= 1; a[w] = a[k]; } } for x in &mut a[..w] { *x = 0; } }",
]
