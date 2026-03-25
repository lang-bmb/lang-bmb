use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut n: i64 = input.trim().parse().unwrap();
    let mut sum: i64 = 0;
    while n > 0 {
        sum += n % 10;
        n /= 10;
    }
    println!("{}", sum);
}
