use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut iter = input.split_whitespace();
    let n: usize = iter.next().unwrap().parse().unwrap();
    let k: usize = iter.next().unwrap().parse().unwrap();
    let a: Vec<i64> = (0..n).map(|_| iter.next().unwrap().parse().unwrap()).collect();
    let kk = k % n;
    let rotated: Vec<i64> = a[n-kk..].iter().chain(a[..n-kk].iter()).cloned().collect();
    let s: Vec<String> = rotated.iter().map(|x| x.to_string()).collect();
    println!("{}", s.join(" "));
}
