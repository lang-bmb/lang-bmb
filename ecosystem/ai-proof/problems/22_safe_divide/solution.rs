use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap();
    for _ in 0..n {
        let a = nums.next().unwrap();
        let b = nums.next().unwrap();
        println!("{}", a / b);
    }
}
