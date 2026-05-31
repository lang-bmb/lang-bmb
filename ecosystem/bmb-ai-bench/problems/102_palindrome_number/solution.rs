use std::io::{self, Read};
fn is_palindrome(n: i64) -> bool {
    if n < 0 { return false; }
    let mut x = n; let mut rev = 0i64;
    while x > 0 { rev = rev * 10 + x % 10; x /= 10; }
    rev == n
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut iter = input.split_whitespace();
    let t: usize = iter.next().unwrap().parse().unwrap();
    for _ in 0..t {
        let n: i64 = iter.next().unwrap().parse().unwrap();
        println!("{}", if is_palindrome(n) { "yes" } else { "no" });
    }
}
