use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let target = nums.next().unwrap();
    let n = nums.next().unwrap() as usize;
    let count = nums.take(n).filter(|&x| x == target).count();
    println!("{}", count);
}
