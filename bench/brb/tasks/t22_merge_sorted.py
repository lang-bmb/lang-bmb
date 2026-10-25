ID = "t22-merge-sorted"
NAME = "merge_sorted"
PARAMS = [("a", "&[i64]"), ("b", "&[i64]"), ("out", "&mut [i64]")]
RET = "()"
STATEMENT = ("a and b are sorted in non-decreasing order and out.len() == a.len() + b.len(). Overwrite out with all elements of "
             "a and b in non-decreasing order (out's previous contents do not matter).")
DOMAIN = ("a and b are slices of i64 values, each sorted in non-decreasing order (possibly empty, duplicates allowed); "
          "out has length a.len() + b.len() and arbitrary contents.")
BMB_DOMAIN = ("spec fn dom(a: &[i64], b: &[i64], out: &[i64]) -> bool { out.len() == a.len() + b.len() && "
              "(forall|i: usize, j: usize| i < j && j < a.len() ==> #[trigger] a[i] <= #[trigger] a[j]) && "
              "(forall|p: usize, q: usize| p < q && q < b.len() ==> #[trigger] b[p] <= #[trigger] b[q]) }")


def domain(a, b, out):
    return (len(out) == len(a) + len(b) and all(a[i] <= a[i + 1] for i in range(len(a) - 1))
            and all(b[i] <= b[i + 1] for i in range(len(b) - 1)))


def model(a, b, out):
    out[:] = sorted(a + b)


def cases(g):
    yield ([], [], [])
    for _ in range(350):
        e = g.elem_mixed()
        a = g.sorted_array(9, e)
        b = g.sorted_array(9, e)
        out = [g.int(-99, 99) for _ in range(len(a) + len(b))]
        yield (a, b, out)


BMB_REF = """spec fn shapes(a: &[i64], b: &[i64], out: &[i64]) -> bool { out.len() == a.len() + b.len() }

#[effects(pure)]
#[uses(shapes)]
fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) -> ()
    requires shapes(a, b, out),
{
    let mut i: usize = 0;
    let mut j: usize = 0;
    while i < a.len() || j < b.len()
        invariant i <= a.len(), j <= b.len(), out.len() == a.len() + b.len(),
        decreases a.len() - i + (b.len() - j),
    {
        if j >= b.len() || (i < a.len() && a[i] <= b[j]) {
            out[i + j] = a[i];
            i = i + 1;
        } else {
            out[i + j] = b[j];
            j = j + 1;
        }
    }
}"""

# Weak: after the main loop only the rest of `a` is copied.
BMB_WEAK = """spec fn shapes(a: &[i64], b: &[i64], out: &[i64]) -> bool { out.len() == a.len() + b.len() }

#[effects(pure)]
#[uses(shapes)]
fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) -> ()
    requires shapes(a, b, out),
{
    let mut i: usize = 0;
    let mut j: usize = 0;
    while i < a.len() && j < b.len()
        invariant i <= a.len(), j <= b.len(), out.len() == a.len() + b.len(),
        decreases a.len() - i + (b.len() - j),
    {
        if a[i] <= b[j] {
            out[i + j] = a[i];
            i = i + 1;
        } else {
            out[i + j] = b[j];
            j = j + 1;
        }
    }
    while i < a.len()
        invariant i <= a.len(), j <= b.len(), out.len() == a.len() + b.len(),
        decreases a.len() - i,
    {
        out[i + j] = a[i];
        i = i + 1;
    }
}"""

RUST_REF = """pub fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) {
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        if j >= b.len() || (i < a.len() && a[i] <= b[j]) {
            out[i + j] = a[i];
            i += 1;
        } else {
            out[i + j] = b[j];
            j += 1;
        }
    }
}
"""

RUST_WEAK = """pub fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) {
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            out[i + j] = a[i];
            i += 1;
        } else {
            out[i + j] = b[j];
            j += 1;
        }
    }
    while i < a.len() {
        out[i + j] = a[i];
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merges() {
        let mut out = vec![0; 5];
        merge_sorted(&[2, 5, 9], &[1, 3], &mut out);
        assert_eq!(out, vec![1, 2, 3, 5, 9]);
        let mut e = vec![0; 2];
        merge_sorted(&[1, 2], &[], &mut e);
        assert_eq!(e, vec![1, 2]);
    }
}
"""

RUST_MUTANTS = [
    "pub fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) { let (mut i, mut j) = (0, 0); while i < a.len() && j < b.len() { if a[i] <= b[j] { out[i + j] = a[i]; i += 1; } else { out[i + j] = b[j]; j += 1; } } while i < a.len() { out[i + j] = a[i]; i += 1; } }",
    "pub fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) { out[..a.len()].copy_from_slice(a); out[a.len()..].copy_from_slice(b); }",
    "pub fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) { let mut v = [a, b].concat(); v.sort(); v.dedup(); out[..v.len()].copy_from_slice(&v); }",
    "pub fn merge_sorted(a: &[i64], b: &[i64], out: &mut [i64]) { out[..a.len()].copy_from_slice(a); out[a.len()..].copy_from_slice(b); let h = (out.len() / 2 + 1).min(out.len()); out[..h].sort(); }",
]
