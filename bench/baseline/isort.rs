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

fn sort(a: &mut [i64]) {
    let mut i = 1;
    while i < a.len() {
        let x = get!(a, i);
        let mut j = i;
        while j > 0 && get!(a, j - 1) > x {
            set!(a, j, get!(a, j - 1));
            j -= 1;
        }
        set!(a, j, x);
        i += 1;
    }
}

fn checksum(a: &[i64]) -> i64 {
    let mut s: i64 = 0;
    let mut k = 0;
    while k < a.len() {
        let v = get!(a, k);
        if v > -1000 && v < 1000 {
            s += v;
        }
        k += 1;
    }
    s
}

fn main() {
    let mut a = [0i64; 30000];
    fill(&mut a, 42);
    sort(&mut a);
    println!("{}", get!(a, 0));
    println!("{}", get!(a, 29999));
    println!("{}", checksum(&a));
}
