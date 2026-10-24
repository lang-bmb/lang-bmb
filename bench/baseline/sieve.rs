include!("common.rs");

fn count_primes(a: &mut [bool]) -> i64 {
    let n = a.len();
    let mut i = 0;
    while i < n {
        set!(a, i, true);
        i += 1;
    }
    set!(a, 0, false);
    set!(a, 1, false);
    let mut p = 2;
    while p < n {
        if get!(a, p) {
            let mut j = p + p;
            while j < n {
                set!(a, j, false);
                j += p;
            }
        }
        p += 1;
    }
    let mut c: i64 = 0;
    let mut k = 0;
    while k < n {
        if get!(a, k) {
            c += 1;
        }
        k += 1;
    }
    c
}

fn main() {
    let mut a = [false; 200000];
    let mut total: i64 = 0;
    let mut r = 0;
    while r < 500 {
        total += count_primes(&mut a);
        r += 1;
    }
    println!("{total}");
}
