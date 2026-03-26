use std::io::{self, Read};

fn factorial(n: i64) -> i64 {
    let mut result: i64 = 1;
    for i in 2..=n { result *= i; }
    result
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let q = nums.next().unwrap();
    for _ in 0..q {
        let x = nums.next().unwrap();
        println!("{}", factorial(x));
    }
}
