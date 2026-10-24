include!("common.rs");

fn add(a: &[i64], b: &[i64], c: &mut [i64]) {
    let mut i = 0;
    while i < c.len() {
        set!(c, i, get!(a, i).wrapping_add(get!(b, i)));
        i += 1;
    }
}

fn main() {
    let mut a = [0i64; 4096];
    let mut b = [0i64; 4096];
    let mut c = [0i64; 4096];
    let mut i = 0;
    let mut v: i64 = 3;
    while i < 4096 {
        set!(a, i, v);
        set!(b, i, v.wrapping_mul(7));
        v = v.wrapping_mul(31).wrapping_add(11);
        i += 1;
    }
    let mut r = 0;
    while r < 100000 {
        add(&a, &b, &mut c);
        add(&c, &b, &mut a);
        r += 1;
    }
    println!("{}", get!(a, 0));
    println!("{}", get!(a, 4095));
}
