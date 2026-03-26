use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap() as usize;
    let arr: Vec<i64> = nums.take(n).collect();
    let mut max_sum = arr[0];
    let mut cur_sum = arr[0];
    for i in 1..n {
        cur_sum = std::cmp::max(cur_sum + arr[i], arr[i]);
        max_sum = std::cmp::max(max_sum, cur_sum);
    }
    println!("{}", max_sum);
}
