use std::io::{self, Read};
use std::collections::VecDeque;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let q = nums.next().unwrap();
    let mut queue: VecDeque<i64> = VecDeque::new();
    for _ in 0..q {
        let op = nums.next().unwrap();
        if op == 1 {
            let x = nums.next().unwrap();
            queue.push_back(x);
        } else {
            match queue.pop_front() {
                Some(v) => println!("{}", v),
                None => println!("-1"),
            }
        }
    }
}
