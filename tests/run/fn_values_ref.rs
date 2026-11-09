// Rust translation of tests/run/fn_values.bmb (backlog 2V-20a), for its
// expected output (`rustc --edition 2021`).
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
    f(x)
}

fn once<F: FnMut(i64) -> i64>(mut f: F, x: i64) -> i64 {
    f(x)
}

fn consume<F: FnOnce(i64) -> i64>(f: F, x: i64) -> i64 {
    f(x)
}

fn halve(x: i64) -> i64 {
    x / 2
}

fn inc(x: i64) -> i64 {
    x + 1
}

fn main() {
    let a = apply(inc, 10);
    assert!(a == 11);
    println!("{a}");
    println!("{}", apply(halve, 41));
    let d = apply(|x: i64| -> i64 { x * 2 }, 21);
    assert!(d == 42);
    println!("{d}");
    let k: i64 = 3;
    let e = apply(move |x: i64| -> i64 { x + k }, 4);
    assert!(e == 7);
    println!("{e}");
    let mut v: Vec<i64> = Vec::new();
    v.push(5);
    println!("{}", apply(move |x: i64| -> i64 { if !v.is_empty() { x } else { 0 } }, 2));
    let mut count: u64 = 0;
    let m = once(move |x: i64| -> i64 { count += 1; let _ = count; x }, 5);
    assert!(m == 5);
    println!("{m}");
    let mut w: Vec<i64> = Vec::new();
    w.push(9);
    println!("{}", consume(move |x: i64| -> i64 { if !w.is_empty() { x } else { 0 } }, 3));
}
