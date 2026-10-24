include!("common.rs");

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn all_pairs(n: i64) -> i64 {
    let mut s: i64 = 0;
    let mut i: i64 = 1;
    while i <= n {
        let mut j: i64 = 1;
        while j <= n {
            s += gcd(i, j);
            j += 1;
        }
        i += 1;
    }
    s
}

fn main() {
    // No indexing here: safe and unchecked builds are identical by design.
    let _ = |a: &mut [i64]| set!(a, 0, get!(a, 0));
    println!("{}", all_pairs(2000));
}
