use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let q = nums.next().unwrap();
    let mut stack: Vec<i64> = Vec::new();
    for _ in 0..q {
        let op = nums.next().unwrap();
        if op == 1 {
            let x = nums.next().unwrap();
            stack.push(x);
        } else if op == 2 {
            match stack.pop() {
                Some(v) => println!("{}", v),
                None => println!("-1"),
            }
        } else {
            match stack.last() {
                Some(v) => println!("{}", v),
                None => println!("-1"),
            }
        }
    }
}
