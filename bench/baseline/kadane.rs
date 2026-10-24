include!("common.rs");

fn fill(a: &mut [i64], seed: i64) {
    let mut x = seed;
    let mut i = 0;
    while i < a.len() {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        set!(a, i, x % 1000);
        i += 1;
    }
}

fn max_sub(a: &[i64]) -> i64 {
    let mut cur: i64 = 0;
    let mut best: i64 = -1000;
    let mut k = 0;
    while k < a.len() {
        let v = get!(a, k);
        if v > -1000 && v < 1000 {
            if cur + v > v {
                cur += v;
            } else {
                cur = v;
            }
            if cur > best {
                best = cur;
            }
        }
        k += 1;
    }
    best
}

fn main() {
    let mut a = [0i64; 200000];
    let mut total: i64 = 0;
    let mut r: i64 = 0;
    while r < 300 {
        fill(&mut a, r);
        total += max_sub(&a);
        r += 1;
    }
    println!("{total}");
}
