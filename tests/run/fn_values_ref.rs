// Rust translation of tests/run/fn_values.bmb (backlog 2V-20a), for its
// expected output (`rustc --edition 2021`).
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
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
}
