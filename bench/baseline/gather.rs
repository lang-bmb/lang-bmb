include!("common.rs");

fn make_perm(p: &mut [usize]) {
    let n = p.len();
    let mut i = 0;
    while i < n {
        set!(p, i, i.wrapping_mul(7919) % n);
        i += 1;
    }
}

fn apply(p: &[usize], src: &[i64], out: &mut [i64]) {
    let mut i = 0;
    while i < p.len() {
        set!(out, i, get!(src, get!(p, i)));
        i += 1;
    }
}

fn main() {
    let mut p = [0usize; 200000];
    let mut a = [0i64; 200000];
    let mut b = [0i64; 200000];
    make_perm(&mut p);
    let mut i = 0;
    let mut v: i64 = 1;
    while i < 200000 {
        set!(a, i, v);
        v = v.wrapping_mul(31).wrapping_add(7);
        i += 1;
    }
    let mut r = 0;
    while r < 200 {
        apply(&p, &a, &mut b);
        apply(&p, &b, &mut a);
        r += 1;
    }
    println!("{}", get!(a, 0));
    println!("{}", get!(a, 199999));
}
