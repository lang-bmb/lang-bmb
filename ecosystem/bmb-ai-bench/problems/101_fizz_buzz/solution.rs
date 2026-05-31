use std::io::{self, BufRead};
fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let n: i64 = lines.next().unwrap().unwrap().trim().split_whitespace()
        .next().unwrap().parse().unwrap();
    for i in 1..=n {
        if i % 15 == 0 { println!("FizzBuzz"); }
        else if i % 3 == 0 { println!("Fizz"); }
        else if i % 5 == 0 { println!("Buzz"); }
        else { println!("{}", i); }
    }
}
