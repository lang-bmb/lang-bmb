use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut iter = input.split_whitespace();
    let n: usize = iter.next().unwrap().parse().unwrap();
    let prices: Vec<i64> = (0..n).map(|_| iter.next().unwrap().parse().unwrap()).collect();
    let (mut min_p, mut profit) = (prices[0], 0i64);
    for &p in &prices[1..] {
        profit = profit.max(p - min_p);
        min_p = min_p.min(p);
    }
    println!("{}", profit);
}
