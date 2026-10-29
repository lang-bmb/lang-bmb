// Rust translation of tests/run/floats.bmb (backlog T2-03), for its expected
// output: `rustc -O floats_ref.rs && ./floats_ref > floats.out`.
#![allow(clippy::all)]
use std::hint::black_box;

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

fn dist(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn newton(x: f64) -> f64 {
    let mut r = if x > 1.0 { x } else { 1.0 };
    for _ in 0..60 {
        r = 0.5 * (r + x / r);
    }
    r
}

fn kahan(v: &Vec<f64>) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0;
    for i in 0..v.len() {
        let y = v[i] - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    sum
}

fn naive(v: &Vec<f64>) -> f64 {
    let mut sum = 0.0;
    for i in 0..v.len() {
        sum = sum + v[i];
    }
    sum
}

fn first_negative(v: &Vec<f64>) -> Option<f64> {
    for i in 0..v.len() {
        if v[i] < 0.0 {
            return Some(v[i]);
        }
    }
    None
}

fn half(n: i64) -> f64 {
    n as f64 / 2.0
}

fn show(x: f64) {
    println!("{:.6}", x);
}

fn flag(b: bool) {
    println!("{}", b as u64);
}

fn main() {
    show(0.1 + 0.2);
    println!("{:.17}", 0.1 + 0.2);
    println!("{:.20}", 1.0 / 3.0);
    show(dist(Point { x: 1.0, y: 2.0 }, Point { x: 4.0, y: 6.0 }));
    show(newton(2.0));
    println!("{:.20}", newton(2.0) - 2.0f64.sqrt());
    let zero: f64 = black_box(0.0);
    let inf = 1.0 / zero;
    let nan = zero / zero;
    show(inf);
    show(-inf);
    show(nan);
    show(-zero);
    show(inf - inf);
    flag(nan == nan);
    flag(nan != nan);
    flag(nan < 1.0);
    flag(nan >= 1.0);
    flag(-zero == zero);
    flag(nan.is_nan());
    flag(inf.is_finite());
    flag(f64::MAX.is_finite());
    show(nan.min(3.0));
    show(nan.max(-3.0));
    show(2.5f64.min(-1.5));
    show(2.5f64.max(-1.5));
    show((-7.25f64).abs());
    show((-1.0f64).sqrt());
    show(f64::MAX * 2.0);
    println!("{:.320}", f64::MIN_POSITIVE / 4.0);
    println!("{:.20}", f64::EPSILON);
    show(f64::MIN);
    show(half(7));
    show(half(-1000000));
    let byte: u8 = 255;
    show(f64::from(byte));
    show(f64::from(u32::MAX));
    show(f64::from(i32::MIN));
    println!("{:.2}", 0.125);
    println!("{:.2}", 0.375);
    println!("{:.0}", 2.5);
    println!("{:.0}", 3.5);
    println!("{:.0}", -0.5);
    println!("{:.1}", 1.0e21);
    println!("{:.3}", 123456.789e3);
    let mut v: Vec<f64> = Vec::new();
    v.push(1.0e16);
    for _ in 0..10 {
        v.push(1.0);
    }
    v.push(-1.0e16);
    show(naive(&v));
    show(kahan(&v));
    match first_negative(&v) {
        None => println!("0"),
        Some(x) => show(x),
    }
    let ps = [Point { x: -0.5, y: 0.25 }, Point { x: 1.5, y: -2.0 }];
    show(dist(ps[0], ps[1]));
}
