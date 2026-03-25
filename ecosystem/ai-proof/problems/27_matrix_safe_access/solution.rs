use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let rows = nums.next().unwrap() as usize;
    let cols = nums.next().unwrap() as usize;
    let m: Vec<i64> = (0..rows * cols).map(|_| nums.next().unwrap()).collect();
    let q = nums.next().unwrap();
    for _ in 0..q {
        let r = nums.next().unwrap() as usize;
        let c = nums.next().unwrap() as usize;
        println!("{}", m[r * cols + c]);
    }
}
