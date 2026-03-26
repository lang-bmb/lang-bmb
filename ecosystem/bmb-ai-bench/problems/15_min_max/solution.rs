use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap() as usize;
    let arr: Vec<i64> = nums.take(n).collect();
    let mn = arr.iter().min().unwrap();
    let mx = arr.iter().max().unwrap();
    println!("{} {}", mn, mx);
}
