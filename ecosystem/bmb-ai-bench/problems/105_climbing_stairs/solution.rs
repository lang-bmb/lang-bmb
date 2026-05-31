use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let n: i64 = input.trim().parse().unwrap();
    if n == 1 { println!("1"); return; }
    if n == 2 { println!("2"); return; }
    let (mut a, mut b) = (1i64, 2i64);
    for _ in 2..n { let c = a + b; a = b; b = c; }
    println!("{}", b);
}
