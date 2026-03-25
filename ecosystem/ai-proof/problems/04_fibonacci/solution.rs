use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let n: u64 = input.trim().parse().unwrap();
    if n == 0 { println!("0"); return; }
    if n == 1 { println!("1"); return; }
    let (mut a, mut b): (i64, i64) = (0, 1);
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    println!("{}", b);
}
