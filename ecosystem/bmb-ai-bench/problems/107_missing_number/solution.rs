use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut iter = input.split_whitespace();
    let n: i64 = iter.next().unwrap().parse().unwrap();
    let expected = n*(n+1)/2;
    let actual: i64 = (0..n).map(|_| iter.next().unwrap().parse::<i64>().unwrap()).sum();
    println!("{}", expected - actual);
}
