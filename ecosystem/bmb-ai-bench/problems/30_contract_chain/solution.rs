use std::io::{self, Read};

fn normalize(x: i64, min: i64, max: i64) -> i64 {
    (x - min) * 100 / (max - min)
}

fn scale(x: i64, factor: i64) -> i64 { x * factor }

fn bound(x: i64, limit: i64) -> i64 {
    if x > limit { limit } else { x }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let min = nums.next().unwrap();
    let max = nums.next().unwrap();
    let factor = nums.next().unwrap();
    let limit = nums.next().unwrap();
    let n = nums.next().unwrap();
    for _ in 0..n {
        let x = nums.next().unwrap();
        let result = bound(scale(normalize(x, min, max), factor), limit);
        println!("{}", result);
    }
}
