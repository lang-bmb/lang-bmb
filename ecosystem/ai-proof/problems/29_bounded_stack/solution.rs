use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace();
    let cap: usize = nums.next().unwrap().parse().unwrap();
    let q: i64 = nums.next().unwrap().parse().unwrap();
    let mut stack: Vec<i64> = Vec::new();
    for _ in 0..q {
        let op: i64 = nums.next().unwrap().parse().unwrap();
        if op == 1 {
            let x: i64 = nums.next().unwrap().parse().unwrap();
            if stack.len() < cap { stack.push(x); }
            else { println!("FULL"); }
        } else if op == 2 {
            match stack.pop() {
                Some(v) => println!("{}", v),
                None => println!("EMPTY"),
            }
        } else {
            println!("{}", stack.len());
        }
    }
}
