use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let target = nums.next().unwrap();
    let n = nums.next().unwrap() as usize;
    let arr: Vec<i64> = nums.take(n).collect();
    for i in 0..n {
        for j in (i + 1)..n {
            if arr[i] + arr[j] == target {
                println!("{} {}", i, j);
                return;
            }
        }
    }
}
