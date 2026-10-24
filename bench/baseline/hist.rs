include!("common.rs");

fn fill(a: &mut [usize], seed: usize) {
    let mut x = seed;
    let mut i = 0;
    while i < a.len() {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        set!(a, i, x / 65536 % 256);
        i += 1;
    }
}

fn count(a: &[usize], h: &mut [i64]) {
    let mut i = 0;
    while i < a.len() {
        let v = get!(a, i);
        set!(h, v, get!(h, v).wrapping_add(1));
        i += 1;
    }
}

fn main() {
    let mut a = [0usize; 100000];
    let mut h = [0i64; 256];
    let mut r = 0;
    while r < 300 {
        fill(&mut a, r);
        count(&a, &mut h);
        r += 1;
    }
    println!("{}", get!(h, 0));
    println!("{}", get!(h, 255));
}
