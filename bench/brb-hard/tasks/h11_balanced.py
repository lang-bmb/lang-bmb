# QuixBugs `is_valid_parenthesization` (defect: `return True` at the end
# instead of `return depth == 0`, so unclosed parentheses pass).

ID = "h11-balanced"
NAME = "balanced"
PARAMS = [("p", "&[i64]")]
RET = "bool"
STATEMENT = ("p is a sequence of parentheses written as numbers: 1 for an opening parenthesis and -1 for a "
             "closing one. Return true when the sequence is properly nested: every closing parenthesis closes "
             "an earlier unclosed opening one, and none is left open at the end. For example "
             "balanced([1, 1, -1, -1]) = true, balanced([-1, 1]) = false, balanced([1]) = false, balanced([]) = true.")
DOMAIN = "p has at most 1000 elements, each 1 or -1."
BMB_DOMAIN = ("spec fn dom(p: &[i64]) -> bool { p.len() <= 1000 && "
              "(forall|i: usize| i < p.len() ==> #[trigger] p[i] == 1 || p[i] == -1) }")


def domain(p):
    return len(p) <= 1000 and all(x in (1, -1) for x in p)


def model(p):
    depth = 0
    for x in p:
        depth += x
        if depth < 0:
            return False
    return depth == 0


def cases(g):
    yield ([],)
    for n in range(1, 7):
        for bits in range(1 << n):
            yield ([1 if bits >> i & 1 else -1 for i in range(n)],)
    for _ in range(250):
        n = g.choice([g.int(0, 40), g.int(0, 1000)])
        k = g.int(0, 2)
        if k == 0:
            yield ([g.choice([1, -1]) for _ in range(n)],)
        else:
            depth, out = 0, []
            for _ in range(n):
                x = -1 if depth > 0 and g.int(0, 1) else 1
                depth += x
                out.append(x)
            if k == 1:
                out += [-1] * depth
            yield (out,)


BMB_REF = """#[effects(pure)]
fn balanced(p: &[i64]) -> (r: bool)
    requires p.len() <= 1000,
{
    let mut depth: i64 = 0;
    let mut i: usize = 0;
    while i < p.len()
        invariant i <= p.len() && p.len() <= 1000 && 0 <= depth && depth <= i,
        decreases p.len() - i,
    {
        if p[i] == 1 {
            depth = depth + 1;
        } else {
            if depth == 0 {
                return false;
            }
            depth = depth - 1;
        }
        i = i + 1;
    }
    depth == 0
}"""

# Weak (QuixBugs): anything that never goes below zero passes.
BMB_WEAK = BMB_REF.replace("    depth == 0\n}", "    true\n}")

RUST_REF = """pub fn balanced(p: &[i64]) -> bool {
    let mut depth: i64 = 0;
    for &x in p {
        depth += x;
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}
"""

# Weak: only the final count; the tests never close before opening.
RUST_WEAK = """pub fn balanced(p: &[i64]) -> bool {
    p.iter().sum::<i64>() == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nests() {
        assert!(balanced(&[1, 1, -1, -1]));
        assert!(balanced(&[]));
        assert!(!balanced(&[1]));
        assert!(!balanced(&[1, 1, -1]));
        assert!(balanced(&[1, -1, 1, -1]));
    }
}
"""

RUST_MUTANTS = [
    # QuixBugs.
    "pub fn balanced(p: &[i64]) -> bool { let mut d: i64 = 0; for &x in p { d += x; if d < 0 { return false; } } true }",
    "pub fn balanced(p: &[i64]) -> bool { p.iter().sum::<i64>() == 0 }",
    "pub fn balanced(p: &[i64]) -> bool { let mut d: i64 = 0; for &x in p { d += x; if d < -1 { return false; } } d == 0 }",
    "pub fn balanced(p: &[i64]) -> bool { p.len() % 2 == 0 && p.first() != Some(&-1) && p.last() != Some(&1) }",
]
