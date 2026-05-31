use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut iter = input.split_whitespace();
    let n: usize = iter.next().unwrap().parse().unwrap();
    let result = (0..n).fold(0i64, |acc, _| acc ^ iter.next().unwrap().parse::<i64>().unwrap());
    println!("{}", result);
}
