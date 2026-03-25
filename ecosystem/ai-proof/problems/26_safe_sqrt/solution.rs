use std::io::{self, Read};

fn isqrt(x: i64) -> i64 {
    if x <= 1 { return x; }
    let mut lo: i64 = 1;
    let mut hi: i64 = 1_000_000_000;
    if hi > x { hi = x; }
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if mid <= x / mid { lo = mid + 1; }
        else { hi = mid - 1; }
    }
    hi
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap();
    for _ in 0..n {
        let x = nums.next().unwrap();
        println!("{}", isqrt(x));
    }
}
