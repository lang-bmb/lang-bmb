use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let lo = nums.next().unwrap();
    let hi = nums.next().unwrap();
    let n = nums.next().unwrap() as usize;
    let result: Vec<String> = (0..n).map(|_| {
        let x = nums.next().unwrap();
        x.clamp(lo, hi).to_string()
    }).collect();
    println!("{}", result.join(" "));
}
